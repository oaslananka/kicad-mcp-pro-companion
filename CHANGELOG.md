# Changelog

All notable changes to this project are documented in this file. Format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/) once it
reaches 1.0.

## [Unreleased]

### Added

- Repository foundation: Rust workspace scaffold, toolchain pin, CI
  pipeline, license, security/contributing docs.
- Architecture, security, and protocol design documentation.
- V1 design spec and implementation plan.
- Domain model, SQLite storage + migrations, layered configuration
  (`crates/core`, `crates/storage`).
- Device identity: Ed25519 keypair, Windows DPAPI secret store,
  fingerprinting (`crates/identity`).
- Canonical workspace path boundary enforcement
  (`crates/workspace`).
- Capability/risk model and deterministic policy engine, with a tool
  registry reconciled against kicad-mcp-pro's real tool list
  (`crates/policy`).
- Explicit session state machine, approvals, persistence
  (`crates/sessions`).
- Local IPC protocol, framing, and versioned transport envelope
  (`crates/protocol`).
- Daemon (identity/policy/session/workspace wiring, local IPC server,
  remote-operation processor with per-operation high-risk approval) and CLI
  (`apps/daemon`, `apps/cli`).
- MCP Streamable HTTP core bridge to kicad-mcp-pro, loopback-only, with an
  in-process mock server for tests (`crates/core-bridge`).
- Transport abstraction, mock transport, reconnect/backoff with jitter
  (`crates/transport`).
- Structured audit trail and local safe-snapshot checkpoints
  (`crates/audit`, `crates/checkpoints`).
- Tauri desktop shell: status, device, pairing, workspaces, sessions (with
  approval dialogs), activity, settings (`apps/desktop`).
- Automated end-to-end vertical slice test covering pairing through
  revocation (`apps/daemon/tests/e2e_vertical_slice.rs`).

## [0.1.0] - Unreleased

Initial pre-release scaffold. Not yet functional end-to-end.
