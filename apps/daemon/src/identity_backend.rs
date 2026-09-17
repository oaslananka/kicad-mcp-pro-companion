//! Picks the production `SecretStore` for the current platform. No branch
//! here may fall back to plaintext storage — a platform with no adapter
//! yet fails startup explicitly instead. See
//! `docs/security/secure-storage.md`.

use std::path::Path;
use std::sync::Arc;

use companion_identity::DeviceIdentityStore;
use companion_storage::Storage;

use crate::errors::DaemonError;

#[cfg(target_os = "windows")]
pub fn build_identity_store(
    storage: Arc<Storage>,
    data_dir: &Path,
) -> Result<Arc<dyn DeviceIdentityStore + Send + Sync>, DaemonError> {
    let secret_store = companion_identity::DpapiSecretStore::new(data_dir.join("secrets"));
    Ok(Arc::new(
        companion_identity::SqliteDeviceIdentityStore::new(storage, secret_store),
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn build_identity_store(
    _storage: Arc<Storage>,
    _data_dir: &Path,
) -> Result<Arc<dyn DeviceIdentityStore + Send + Sync>, DaemonError> {
    Err(DaemonError::SecretStoreUnavailable)
}
