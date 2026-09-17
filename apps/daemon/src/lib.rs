//! The Companion daemon: the single authoritative local runtime. See
//! `docs/architecture/component-boundaries.md`.

pub mod errors;
pub mod handlers;
pub mod identity_backend;
pub mod ipc_server;
pub mod remote_processor;
pub mod state;
pub mod tool_reconciliation;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use companion_audit::AuditRepository;
use companion_core::{CompanionConfig, SystemClock};
use companion_core_bridge::{CoreBridgeClient, CoreBridgeConfig};
use companion_identity::{SecretStore, SqliteDeviceIdentityStore};
use companion_policy::{PolicyEngine, TomlToolRegistry};
use companion_sessions::SessionRepository;
use companion_storage::Storage;
use companion_workspace::WorkspaceRepository;

use crate::state::DaemonState;

const CORE_HEALTH_PROBE_TIMEOUT: Duration = Duration::from_millis(500);

/// Builds daemon state in the documented startup order: storage, identity,
/// then the repositories/engines that depend on storage. Returns the state
/// and the socket/pipe name ready for [`ipc_server::run_ipc_server`].
pub fn build_state(config: &CompanionConfig) -> anyhow::Result<Arc<DaemonState>> {
    std::fs::create_dir_all(&config.data_dir)?;
    let storage = Arc::new(Storage::open(&config.data_dir)?);
    let identity_store =
        identity_backend::build_identity_store(Arc::clone(&storage), &config.data_dir)?;
    build_state_from_parts(config, storage, identity_store)
}

/// Builds daemon state with an explicitly supplied secret-store backend.
///
/// This exists so integration tests and embedders can provide a controlled
/// backend without weakening the production binary's platform-specific
/// secure-storage policy. `run()` never calls this function.
pub fn build_state_with_secret_store<S>(
    config: &CompanionConfig,
    secret_store: S,
) -> anyhow::Result<Arc<DaemonState>>
where
    S: SecretStore + 'static,
{
    std::fs::create_dir_all(&config.data_dir)?;
    let storage = Arc::new(Storage::open(&config.data_dir)?);
    let identity_store = Arc::new(SqliteDeviceIdentityStore::new(
        Arc::clone(&storage),
        secret_store,
    ));
    build_state_from_parts(config, storage, identity_store)
}

fn build_state_from_parts(
    config: &CompanionConfig,
    storage: Arc<Storage>,
    identity_store: Arc<dyn companion_identity::DeviceIdentityStore + Send + Sync>,
) -> anyhow::Result<Arc<DaemonState>> {
    let workspace_repo = Arc::new(WorkspaceRepository::new(Arc::clone(&storage)));
    let session_repo = Arc::new(SessionRepository::new(Arc::clone(&storage)));
    let policy_engine = Arc::new(PolicyEngine::new(TomlToolRegistry::embedded()));
    let audit_repo = Arc::new(AuditRepository::new(Arc::clone(&storage)));
    let core_bridge = Arc::new(CoreBridgeClient::new(CoreBridgeConfig::new(
        config.core_bridge_endpoint.clone(),
    ))?);
    let mut core_health_probe_config = CoreBridgeConfig::new(config.core_bridge_endpoint.clone());
    core_health_probe_config.timeout = CORE_HEALTH_PROBE_TIMEOUT;

    Ok(Arc::new(DaemonState {
        storage,
        identity_store,
        workspace_repo,
        session_repo,
        policy_engine,
        audit_repo,
        core_bridge,
        core_health_probe_config,
        clock: Arc::new(SystemClock),
        shutdown: Arc::new(tokio::sync::Notify::new()),
        transport: std::sync::Mutex::new(None),
        pending_operations: std::sync::Mutex::new(HashMap::new()),
    }))
}

/// Runs the daemon until shutdown is requested (via `DaemonShutdown` IPC
/// request or Ctrl-C). A single-instance conflict on `config.data_dir`
/// surfaces as an error from [`build_state`] before anything else starts.
pub async fn run(config: CompanionConfig) -> anyhow::Result<()> {
    let state = build_state(&config)?;
    tracing::info!(data_dir = %config.data_dir.display(), "daemon starting");

    let shutdown = Arc::clone(&state.shutdown);
    tokio::select! {
        result = ipc_server::run_ipc_server(Arc::clone(&state), config.data_dir.clone()) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received ctrl-c, shutting down");
            shutdown.notify_waiters();
        }
    }

    Ok(())
}
