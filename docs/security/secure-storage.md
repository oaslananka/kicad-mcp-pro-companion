# Secure Storage

## What must never be plaintext-persisted

Device private key material must never be written to SQLite, a plain config
file, logs, telemetry, or returned through the local IPC API or CLI output
(including under `--verbose`).

## `SecretStore` abstraction

```rust
pub trait SecretStore: Send + Sync {
    fn store_device_key(&self, device_id: &DeviceId, key: &SigningKeyMaterial) -> Result<(), SecretStoreError>;
    fn load_device_key(&self, device_id: &DeviceId) -> Result<Option<SigningKeyMaterial>, SecretStoreError>;
    fn delete_device_key(&self, device_id: &DeviceId) -> Result<(), SecretStoreError>;
}
```

`SigningKeyMaterial` intentionally has no `Debug`/`Display` impl that prints
bytes, and is wrapped so it zeroizes on drop.

## Platform adapters

| Platform | Production backend |
|---|---|
| Windows | DPAPI / Windows Credential Manager |
| macOS | Keychain |
| Linux | Secret Service (libsecret) |

V1 targets the current development platform (Windows) first with a real
DPAPI-backed adapter, plus a `InMemorySecretStore` that is explicitly
`#[cfg(test)]`/test-only and documented as such. macOS/Linux adapters follow
the same trait and are added without touching any caller. **No production
code path silently falls back to plaintext storage if a platform adapter is
unavailable** — if no adapter is configured for the running platform, device
identity creation fails with a typed `IDENTITY_SECRET_STORE_UNAVAILABLE`
error rather than degrading to plaintext.

## Device identity lifecycle

1. On daemon start, check whether a device identity exists (`storage`
   metadata table has a device row + `SecretStore` has a matching key).
2. If not, generate an Ed25519 keypair using the OS CSPRNG (via a mature,
   reviewed crate — no custom RNG or crypto).
3. Persist the private key via `SecretStore` only.
4. Persist non-secret device metadata (device id, public key, display name,
   created_at, fingerprint) via `storage`.
5. Compute a human-readable fingerprint from the public key (e.g. a
   truncated, grouped hash) for display during pairing.
6. Never log the private key. Never include it in any error's contextual
   data. Never send it to the cloud — only the public key and signatures
   ever cross the transport boundary.

## Constant-time and redaction rules

- Any comparison involving a secret (pairing code, token) uses
  constant-time comparison, not `==` on a `String`/`&[u8]`.
- Types that wrap sensitive data implement a redacted `Debug` (e.g.
  `SigningKeyMaterial(REDACTED)`) so an accidental `{:?}` in a log statement
  cannot leak it. This is covered by a dedicated unit test per sensitive
  type, not just a code-review convention.

## What SQLite is for

SQLite holds everything else: device metadata (public fields only), pairing
metadata, authorized workspaces, sessions, approvals, audit events,
settings, checkpoint metadata. See [`crates/storage`](../../crates/storage)
for the schema and migrations.
