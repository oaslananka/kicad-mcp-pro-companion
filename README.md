# KiCad MCP Pro Companion

KiCad MCP Pro Companion is the trusted local runtime that securely connects
authorized remote AI agents and cloud services to a user's local
[KiCad MCP Pro](https://github.com/oaslananka/kicad-mcp-pro) environment.

> KiCad MCP Pro knows how to operate KiCad.
> KiCad MCP Pro Companion decides whether a remote actor is allowed to ask it to.

## Why Companion exists

kicad-mcp-pro exposes a rich MCP tool surface for driving KiCad from AI
agents, reachable locally over Streamable HTTP. That's the right shape for
"what can be done inside KiCad." It is not, by itself, an answer to "which
remote agent, from where, for how long, with which permissions, is allowed
to ask for it." Companion is a separate project that answers exactly that
question, sitting between remote/cloud AI agents and the local kicad-mcp-pro
installation as a deny-by-default policy boundary.

## Relationship to kicad-mcp-pro

| | Owns |
|---|---|
| [kicad-mcp-pro](https://github.com/oaslananka/kicad-mcp-pro) | What can be done inside KiCad — schematic/PCB tools, ERC/DRC, manufacturing export |
| **kicad-mcp-pro-companion** (this repo) | Who may ask for it, from where, to which workspace, for how long, under which permissions, through which trusted local session |
| Future private cloud service | Accounts, cloud device registry, hosted relay, cloud workspaces, billing, collaboration |

Companion never re-implements KiCad domain logic and never modifies
kicad-mcp-pro. It talks to it purely as an MCP client over
`http://127.0.0.1:3334/mcp` (configurable) — see
[`docs/protocol/README.md`](docs/protocol/README.md).

## Architecture

```
ChatGPT / Claude / Web / Agent
             |
     KiCad MCP Cloud (future, not in this repo)
             |
       encrypted OUTBOUND connection
             |
+--------------------------------+
| KiCad MCP Pro Companion        |
| device identity · pairing      |
| session lifecycle · workspace  |
| capability policy · risk       |
| approvals · audit · checkpoints|
+---------------+----------------+
                |  loopback-only MCP client
        KiCad MCP Pro (external, unmodified)
                |
              KiCad
```

Full detail: [system overview](docs/architecture/system-overview.md),
[component boundaries](docs/architecture/component-boundaries.md),
[session lifecycle](docs/architecture/session-lifecycle.md),
[data flow](docs/architecture/data-flow.md).

## Security model

- No inbound internet-facing port, ever. All cloud connectivity is
  outbound-initiated by Companion.
- The cloud is always treated as untrusted input. Every remote operation
  passes through session validation → workspace authorization → capability
  resolution → risk evaluation → approval (if required) → audit, before it
  ever reaches kicad-mcp-pro.
- A transport connection is not authorization. A paired device is not an
  active session. An active session does not mean unrestricted tool access.
- Device private keys are held in the OS secure store (DPAPI on Windows,
  Keychain on macOS, Secret Service on Linux) — never plaintext in SQLite,
  logs, or CLI/IPC output.
- Reconnecting a transport never resurrects a revoked or expired session.

Details: [threat model](docs/security/threat-model.md),
[trust boundaries](docs/security/trust-boundaries.md),
[secure storage](docs/security/secure-storage.md).

## Current status

**Pre-alpha, functional local vertical slice.** This is an honest snapshot,
not a roadmap dressed up as a status:

- ✅ Architecture, security, and protocol design docs.
- ✅ Rust workspace scaffold and CI (Rust matrix + frontend job).
- ✅ Domain model, SQLite storage + migrations, layered config.
- ✅ Device identity (Ed25519; Windows DPAPI, macOS Keychain, Linux Secret Service), fingerprinting.
- ✅ Workspace path boundary enforcement (traversal/symlink/sibling-collision-proof).
- ✅ Capability/risk model and deterministic policy engine.
- ✅ Session state machine (explicit transitions, TTL, revoke-is-terminal).
- ✅ Daemon + CLI + local IPC (named pipe/Unix socket), single-instance guard.
- ✅ Local kicad-mcp-pro MCP bridge (loopback-only, typed errors, timeouts).
- ✅ Mock transport + reconnect/backoff; daemon-side remote-operation
  processor wired to policy/core-bridge/audit, including per-operation
  high-risk "allow once" approval.
- ✅ Structured audit trail; local safe-snapshot checkpoints.
- ✅ Tauri desktop shell (status/device/pairing/workspaces/sessions incl.
  approval dialogs/activity/settings) — builds and typechecks; not yet
  manually driven through a live GUI session in this environment.
- ✅ Automated end-to-end vertical slice test covering the full pairing →
  session → low-risk operation → audit → high-risk approval → revoke →
  reconnect-still-denied scenario (`apps/daemon/tests/e2e_vertical_slice.rs`).
- ✅ Fail-closed tool-registry drift detection: the curated authorization
  allowlist is reconciled against a SHA-pinned upstream public-tool snapshot;
  live `tools/list` discovery can report drift but never grants capabilities.

What's still ahead: a real cloud relay/control plane (out of scope for this
repo — see below), capability/risk classification of the currently unclassified
upstream tools, and UI polish on the desktop shell.

## Quick start for developers

```bash
git clone https://github.com/oaslananka/kicad-mcp-pro-companion.git
cd kicad-mcp-pro-companion
cp .env.example .env
cargo build --workspace
cargo test --workspace
```

## CLI

```bash
kicad-mcp-companion setup
kicad-mcp-companion daemon start
kicad-mcp-companion pair
kicad-mcp-companion status
kicad-mcp-companion workspace add <path>
kicad-mcp-companion workspace list
kicad-mcp-companion workspace remove <id>
kicad-mcp-companion session list
kicad-mcp-companion session approve <id>
kicad-mcp-companion session deny <id>
kicad-mcp-companion session pause <id>
kicad-mcp-companion session resume <id>
kicad-mcp-companion session revoke <id>
kicad-mcp-companion audit list
```

## Desktop shell

```bash
cd apps/desktop
pnpm install
pnpm tauri dev   # requires the daemon running separately, or via `daemon start`
```

## Development mock mode

There is no production cloud backend yet, and this repository will never
contain one (see [Out of scope](#out-of-scope), below). Development and
testing use an in-process mock relay and a mock kicad-mcp-pro server so the
full pairing → session → operation → audit flow can be exercised without any
external service. Anywhere this is active, the CLI/desktop UI say so
explicitly (e.g. "Using local mock pairing provider") — production behavior
is never simulated silently.

## What is NOT supported yet

Deliberately out of scope for the initial implementation: production hosted
cloud, billing, user account database, team organizations, SSO, real-time
collaborative editing, cloud project storage, GitHub sync, mobile app,
production ChatGPT App pairing, cloud-hosted KiCad, cloud AI inference,
arbitrary shell access, arbitrary filesystem access, and unattended
manufacturing automation. See spec §14 for the full list and rationale.

## Roadmap

Phases 0–9 (foundation, domain/storage, identity, workspaces/policy,
sessions, daemon/CLI/IPC, core bridge, transport, audit/checkpoints,
desktop) are complete, including an automated end-to-end vertical slice
(Phase 10's core deliverable). Tracked in
[the implementation plan](docs/superpowers/plans/2026-09-16-companion-v1.md).
Remaining, tracked as follow-on work rather than blocking V1: a real cloud
relay (a separate future repository — see Out of scope), capability/risk
classification of currently unclassified upstream tools, and desktop UI
polish/manual QA in a live GUI session.

## Security review checklist

Disposition of the project's own pre-release checklist, each backed by a
specific test rather than an assertion:

- [x] **Cloud-originated input cannot bypass policy evaluation** —
  `apps/daemon/src/remote_processor.rs::handle_operation_request` always
  calls the policy engine before any execution path; proven by
  `crates/core-bridge/tests/policy_integration.rs` and the full
  `apps/daemon/tests/e2e_vertical_slice.rs`.
- [x] **A session cannot access an unapproved workspace** —
  `crates/policy/tests/engine.rs::denies_when_workspace_not_in_session_workspace_ids`.
- [x] **No path traversal/symlink escape from a workspace** —
  `crates/workspace/tests/boundary.rs` (9 cases: traversal, sibling-name
  collision, symlink escape, mixed separators, Unicode, Windows drive
  paths).
- [x] **An expired session cannot execute** —
  `crates/policy/tests/engine.rs::denies_when_session_expired`;
  `apps/daemon/tests/e2e_vertical_slice.rs` (via `crates/sessions`
  expiration tests).
- [x] **A revoked session cannot become active after reconnect** —
  `crates/sessions/src/state_machine.rs::reconnect_never_resurrects_an_active_session_directly`
  and `::revoked_session_rejects_all_further_events`;
  `apps/daemon/tests/e2e_vertical_slice.rs` steps 19–23.
- [x] **No unknown tool executes** —
  `crates/policy/tests/engine.rs::denies_unknown_tool_with_no_fallback_allow`.
- [x] **Manufacturing capability is never implied by generic write access** —
  `crates/core/src/capability.rs::design_profile_is_superset_of_inspect_and_excludes_manufacturing`;
  `crates/policy/tests/engine.rs::manufacturing_capability_is_never_implied_by_design_profile`.
- [x] **Secret material is never plaintext-persisted or trivially logged** —
  `SigningKeyMaterial`'s redacted `Debug` impl
  (`crates/identity/src/secret_store.rs::debug_output_never_contains_key_bytes`);
  private keys only ever pass through `SecretStore`
  (`crates/identity/tests/device_lifecycle.rs`,
  `secret_store/windows_dpapi.rs` tests). Not exhaustively audited across
  every possible future log call site — treat this as an invariant to keep
  testing as the codebase grows, not a one-time guarantee.
- [x] **Private keys never reach SQLite as plaintext** — enforced by
  construction (`companion-storage` has no code path that touches key
  material; only `SecretStore` does).
- [x] **Malformed remote input cannot crash the daemon** — envelopes that
  fail to deserialize are logged and dropped in `remote_processor.rs`, not
  unwrapped.
- [x] **No unbounded message/body allocation** —
  `crates/protocol/src/codec.rs::oversized_message_without_newline_is_rejected_not_allocated_unbounded`.
- [x] **No inbound public listener** — local IPC is a named pipe/Unix
  domain socket (`crates/protocol/src/ipc_naming.rs`,
  `apps/daemon/src/ipc_server.rs`); the core bridge only ever dials
  loopback (`crates/core-bridge/src/client.rs::non_loopback_hosts_are_rejected`).
- [x] **UI/CLI cannot bypass daemon authorization** — both are thin IPC
  clients (`apps/cli/src/ipc_client.rs`, `apps/desktop/src-tauri/src/main.rs`)
  with no direct storage/policy access.
- [x] **Transport reconnect cannot silently restore authorization** —
  `ReconnectingTransport` only tracks `TransportState`; session status is a
  fully separate concern, proven end-to-end in the vertical slice test.
- [x] **Multiple daemon instances cannot corrupt state** —
  `crates/storage/tests/single_instance.rs`.
- [x] **Audit records exist for allow/deny/high-risk decisions** —
  every policy decision is recorded before execution in
  `remote_processor.rs::handle_operation_request`; verified in the vertical
  slice test.
- [x] **High-risk operations are actually gated** — `apps/daemon/tests/e2e_vertical_slice.rs`
  asserts the fake kicad-mcp-pro server's call count does **not** increase
  when a high-risk operation is requested, only after `ApproveOperation`.
- [x] **Users can immediately pause/revoke** — `PauseSession`/`RevokeSession`
  IPC handlers apply synchronously before responding; exercised in
  `apps/cli/tests/ipc_integration.rs` and the vertical slice test.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). This project is a security
boundary — new code in the policy/session/workspace/identity crates is held
to a TDD, high-coverage bar.

## Security reporting

Do not open a public issue for a suspected vulnerability — see
[`SECURITY.md`](SECURITY.md).
