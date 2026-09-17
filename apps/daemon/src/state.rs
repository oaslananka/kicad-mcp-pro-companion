use std::collections::HashMap;
use std::sync::Arc;

use companion_audit::AuditRepository;
use companion_core::{Capability, Clock, OperationId, OperationRequest, RiskLevel};
use companion_core_bridge::CoreBridgeClient;
use companion_identity::DeviceIdentityStore;
use companion_policy::{PolicyEngine, TomlToolRegistry};
use companion_sessions::SessionRepository;
use companion_storage::Storage;
use companion_transport::Transport;
use companion_workspace::WorkspaceRepository;

/// A high-risk operation the policy engine has flagged with
/// `RequireApproval`. It sits here until a local user calls
/// `ApproveOperation`/`DenyOperation` over IPC — the transport connection
/// alone never grants it. See `docs/architecture/data-flow.md` step 10.
pub struct PendingOperation {
    pub request: OperationRequest,
    pub capability: Capability,
    pub risk: RiskLevel,
}

/// The daemon's shared, authoritative state. Every privileged decision the
/// daemon makes goes through the fields here — the IPC server and the
/// remote-operation processor are thin transport shells around this.
pub struct DaemonState {
    pub storage: Arc<Storage>,
    pub identity_store: Arc<dyn DeviceIdentityStore + Send + Sync>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub session_repo: Arc<SessionRepository>,
    pub policy_engine: Arc<PolicyEngine<TomlToolRegistry>>,
    pub audit_repo: Arc<AuditRepository>,
    pub core_bridge: Arc<CoreBridgeClient>,
    pub clock: Arc<dyn Clock>,
    pub shutdown: Arc<tokio::sync::Notify>,
    /// Set once a (mock or, in future, real) relay transport is connected.
    /// `ApproveOperation`/`DenyOperation` send their result back over
    /// whichever transport is current at decision time.
    pub transport: std::sync::Mutex<Option<Arc<dyn Transport>>>,
    pub pending_operations: std::sync::Mutex<HashMap<OperationId, PendingOperation>>,
}
