# Repository Security Automation & Temporary Advisory Management

This document records the repository-level security baseline, quality automation rules, and active security advisory review schedules (Issue #4).

## Enforced Security Automation

- **GitHub Actions Security**: All actions are pinned to immutable commit SHAs with version comments for readability.
- **Least Privilege Tokens**: Workflow permissions default to `{}` or `contents: read`. Elevated permissions (`contents: write`, `security-events: write`) are granted strictly per-job where required.
- **No Persisted Credentials**: `actions/checkout` steps use `persist-credentials: false`.
- **Cargo Audit & pnpm Audit**: `cargo audit` runs as a mandatory CI step for Rust dependencies, and `pnpm audit` checks npm packages in `apps/desktop`.
- **Dependency Review**: Pull requests that introduce moderate-or-higher vulnerable dependencies are automatically rejected.
- **Workflow Security (zizmor)**: zizmor scans GitHub Actions workflows for template injection, overly broad permissions, and insecure configurations.
- **OSV Security Scanning**: OSV-Scanner checks pull requests and runs weekly scheduled scans across the full dependency tree, uploading SARIF reports to GitHub Code Scanning.

---

## Active Temporary Security Exception (Issue #4)

The Tauri desktop application lockfile (`apps/desktop/src-tauri/Cargo.lock`) contains a time-bounded security exception in `apps/desktop/src-tauri/osv-scanner.toml`.

### Advisory RUSTSEC-2024-0429 Analysis

- **Advisory ID**: `RUSTSEC-2024-0429` (CVE-2024-52533 - `glib` `VariantStrIter` out-of-bounds read unsoundness).
- **Patched Release Floor**: `glib >= 0.20.0` (in `gtk-rs 0.20` series).
- **Current Version in Lockfile**: `glib 0.18.5`.

#### Exact Dependency Chain

```
glib v0.18.5
├── atk v0.18.2
│   └── gtk v0.18.2
│       ├── muda v0.19.3
│       │   └── tauri v2.11.5
│       ├── tao v0.35.3
│       │   └── tauri-runtime-wry v2.11.4
│       │       └── tauri v2.11.5
│       ├── tauri v2.11.5
│       ├── tauri-runtime v2.11.3
│       └── wry v0.55.1
```

#### Upstream Blocker

`glib 0.18.5` is locked by `gtk-rs 0.18.x` crate bounds (`gtk v0.18.2`, `gdk v0.18.2`, `gio v0.18.4`, `cairo-rs v0.18.5`, `atk v0.18.2`). `glib 0.18.5` is the highest patch version published in the `0.18` series.

Upgrading `glib` to `>= 0.20.0` requires upgrading the entire Linux webview crate stack (`tao`, `wry`, `muda`, `webkit2gtk`) to `gtk-rs 0.20`. Stable Tauri 2.11.x uses `tao 0.35` and `wry 0.55`, which are anchored to `gtk-rs 0.18`. Therefore, `glib` cannot be upgraded independently within stable Tauri 2.x without breaking upstream compiler bounds.

#### Exposure Assessment

1. Companion desktop shell (`apps/desktop/src-tauri/src/main.rs`) does not call `GVariant` or `VariantStrIter` functions directly.
2. The Tauri desktop process acts strictly as a thin IPC forwarder, passing typed requests over local Unix domain / named sockets (`interprocess::local_socket`) using JSON encoding directly to the local daemon socket.
3. Actual security risk exposure to `VariantStrIter` out-of-bounds read in the desktop application is assessed as **LOW**.

#### Time-Bounded Expiry & Review Schedule

- **Exception Expiry Date**: `2026-10-31`
- **Tracked Issue**: Issue #4 ("security: retire temporary Tauri OSV exceptions before expiry").
- **Exit Criteria**: When upstream Tauri releases a stable build migrating its Linux GTK dependencies to `gtk-rs >= 0.20`, upgrade `apps/desktop/src-tauri`, regenerate lockfiles, verify all security scans, and remove `RUSTSEC-2024-0429` from `osv-scanner.toml`.

---

## Dependency Update Policy

Dependabot is configured weekly for Cargo, npm/pnpm, and GitHub Actions. Routine minor and patch updates are grouped into a single PR per ecosystem. Security updates remain enabled independently. Merging PRs remains an explicit maintainer action; auto-merge is intentionally disabled.
