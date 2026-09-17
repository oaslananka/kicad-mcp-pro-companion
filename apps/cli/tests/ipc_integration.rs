//! CLI <-> daemon local IPC integration tests. The daemon runs in-process
//! (via `kicad_mcp_companion_daemon::run`) against a temp data directory,
//! and the CLI's own `ipc_client` is used to talk to it — exactly the path
//! the real `kicad-mcp-companion` binary takes.

use std::path::{Path, PathBuf};
use std::time::Duration;

use companion_core::config::{self, CliOverrides};
use companion_identity::InMemorySecretStore;
use companion_protocol::{IpcRequest, IpcResponse};
use kicad_mcp_companion_cli::ipc_client::send_request;
use kicad_mcp_companion_daemon::{build_state_with_secret_store, ipc_server};

async fn wait_for_daemon(data_dir: &Path) {
    for _ in 0..100 {
        if send_request(data_dir, IpcRequest::Status).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("daemon did not become reachable within the retry budget");
}

fn fresh_data_dir() -> PathBuf {
    tempfile::tempdir().unwrap().keep()
}

fn spawn_test_daemon(cfg: companion_core::CompanionConfig) {
    let data_dir = cfg.data_dir.clone();
    let state = build_state_with_secret_store(&cfg, InMemorySecretStore::new())
        .expect("test daemon state builds with in-memory secret store");
    tokio::spawn(async move {
        let _ = ipc_server::run_ipc_server(state, data_dir).await;
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_reports_zero_sessions_and_workspaces_on_a_fresh_daemon() {
    let data_dir = fresh_data_dir();
    let cfg = config::load(CliOverrides {
        data_dir: Some(data_dir.clone()),
        ..Default::default()
    })
    .unwrap();

    spawn_test_daemon(cfg);
    wait_for_daemon(&data_dir).await;

    let response = send_request(&data_dir, IpcRequest::Status).await.unwrap();
    match response {
        IpcResponse::Status(view) => {
            assert_eq!(view.active_session_count, 0);
            assert_eq!(view.workspace_count, 0);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::DaemonShutdown)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorize_workspace_then_list_it_then_remove_it() {
    let data_dir = fresh_data_dir();
    let cfg = config::load(CliOverrides {
        data_dir: Some(data_dir.clone()),
        ..Default::default()
    })
    .unwrap();

    spawn_test_daemon(cfg);
    wait_for_daemon(&data_dir).await;

    let workspace_dir = fresh_data_dir();
    let response = send_request(
        &data_dir,
        IpcRequest::AuthorizeWorkspace {
            path: workspace_dir.to_string_lossy().to_string(),
            display_name: "SensorBoard".into(),
        },
    )
    .await
    .unwrap();
    let workspace_id = match response {
        IpcResponse::WorkspaceAuthorized(view) => {
            assert_eq!(view.display_name, "SensorBoard");
            view.workspace_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = send_request(&data_dir, IpcRequest::ListWorkspaces)
        .await
        .unwrap();
    match response {
        IpcResponse::Workspaces(workspaces) => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].workspace_id, workspace_id);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::RemoveWorkspace { workspace_id })
        .await
        .unwrap();
    let response = send_request(&data_dir, IpcRequest::ListWorkspaces)
        .await
        .unwrap();
    match response {
        IpcResponse::Workspaces(workspaces) => assert!(workspaces.is_empty()),
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::DaemonShutdown)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn approve_pause_resume_revoke_session_round_trip() {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use companion_core::{CapabilityProfile, SessionStatus};
    use companion_sessions::{new_unpaired_session, SessionRepository};
    use companion_storage::Storage;

    let data_dir = fresh_data_dir();

    // Seed a PendingApproval session directly via storage before the daemon
    // starts (it would otherwise hold the single-instance lock). There is
    // no IPC request to create a session — sessions only ever originate
    // from a remote transport request (Phase 7), which is exactly the
    // property this test relies on: the CLI can decide on an existing
    // session, never invent one.
    let session_id = {
        let storage = Arc::new(Storage::open(&data_dir).unwrap());
        let repo = SessionRepository::new(storage);
        let clock = companion_core::SystemClock;
        let mut session = new_unpaired_session(
            companion_core::DeviceId::new(),
            "agent:test".into(),
            BTreeSet::new(),
            CapabilityProfile::Inspect,
            "read schematic".into(),
            time::Duration::hours(1),
            &clock,
        );
        session.status = SessionStatus::PendingApproval;
        let id = session.session_id;
        repo.save(&session).unwrap();
        id
        // storage (and its single-instance lock) drops here.
    };

    let cfg = config::load(CliOverrides {
        data_dir: Some(data_dir.clone()),
        ..Default::default()
    })
    .unwrap();
    spawn_test_daemon(cfg);
    wait_for_daemon(&data_dir).await;

    send_request(&data_dir, IpcRequest::ApproveSession { session_id })
        .await
        .unwrap();
    let response = send_request(&data_dir, IpcRequest::ListSessions)
        .await
        .unwrap();
    match response {
        IpcResponse::Sessions(sessions) => {
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0].status, "Active");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::PauseSession { session_id })
        .await
        .unwrap();
    let response = send_request(&data_dir, IpcRequest::ListSessions)
        .await
        .unwrap();
    match response {
        // ListSessions returns every session regardless of status (so a
        // caller can see PendingApproval/Suspended ones to act on them);
        // the session itself is now Suspended, not absent.
        IpcResponse::Sessions(sessions) => {
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0].status, "Suspended");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::ResumeSession { session_id })
        .await
        .unwrap();
    let response = send_request(&data_dir, IpcRequest::ListSessions)
        .await
        .unwrap();
    match response {
        IpcResponse::Sessions(sessions) => {
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0].status, "Active");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    send_request(&data_dir, IpcRequest::RevokeSession { session_id })
        .await
        .unwrap();
    let response = send_request(&data_dir, IpcRequest::ListSessions)
        .await
        .unwrap();
    match response {
        IpcResponse::Sessions(sessions) => {
            assert_eq!(sessions.len(), 1);
            assert_eq!(
                sessions[0].status, "Revoked",
                "revoked session must still be visible, just marked Revoked"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    // Revocation is terminal: approving again must fail, not resurrect it.
    let response = send_request(&data_dir, IpcRequest::ApproveSession { session_id })
        .await
        .unwrap();
    assert!(matches!(response, IpcResponse::Error(_)));

    send_request(&data_dir, IpcRequest::DaemonShutdown)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_session_id_is_a_clean_error_not_a_crash() {
    let data_dir = fresh_data_dir();
    let cfg = config::load(CliOverrides {
        data_dir: Some(data_dir.clone()),
        ..Default::default()
    })
    .unwrap();

    spawn_test_daemon(cfg);
    wait_for_daemon(&data_dir).await;

    let response = send_request(
        &data_dir,
        IpcRequest::ApproveSession {
            session_id: companion_core::SessionId::new(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(response, IpcResponse::Error(_)));

    // The daemon must still be alive and responsive after a denied/errored request.
    let response = send_request(&data_dir, IpcRequest::Status).await.unwrap();
    assert!(matches!(response, IpcResponse::Status(_)));

    send_request(&data_dir, IpcRequest::DaemonShutdown)
        .await
        .unwrap();
}
