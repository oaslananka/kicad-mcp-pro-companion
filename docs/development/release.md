# Release Engineering & Code Signing Procedures

This document outlines the automated release pipeline, signing readiness configuration, and manual QA procedures for KiCad MCP Pro Companion releases.

## Release Pipeline Overview

Releases are triggered automatically when a version tag matching `v*` (e.g., `v0.1.0-rc1` or `v0.1.0`) is pushed to GitHub.

Workflow file: `.github/workflows/release.yml`

### Pipeline Stages

1. **Build Binaries**: Compiles release binaries for CLI (`companion-cli`) and Daemon (`companion-daemon`) on:
   - Linux `x86_64-unknown-linux-gnu`
   - macOS `aarch64-apple-darwin`
   - Windows `x86_64-pc-windows-msvc`
2. **Package Artifacts**: Packages binaries alongside `README.md` and `LICENSE` into `.tar.gz` (Linux/macOS) and `.zip` (Windows) archives.
3. **Consolidate & Checksum**: Collects all platform archives, computes cryptographic SHA-256 checksums (`SHA256SUMS.txt`).
4. **Publish Release**: Uploads artifacts and checksums to a GitHub Release with automatically generated release notes.

---

## Code Signing & Notarization Readiness

Production releases on macOS and Windows require code signing to prevent OS gatekeeper/smartscreen warnings.

### macOS (Apple Developer ID)

For macOS releases, code signing and Apple notarization require the following GitHub Secrets:

- `APPLE_CERTIFICATE`: Base64-encoded Developer ID Application `.p12` certificate.
- `APPLE_CERTIFICATE_PASSWORD`: Password for the `.p12` certificate.
- `APPLE_NOTARIZATION_USERNAME`: Apple ID email address.
- `APPLE_NOTARIZATION_PASSWORD`: App-specific password generated at appleid.apple.com.
- `APPLE_TEAM_ID`: 10-character Apple Developer Team ID.

When these secrets are provided, the Tauri build and macOS binary steps sign the executables with `codesign` and submit the final app bundle to `xcrun notarytool`.

### Windows (Authenticode)

For Windows releases, code signing requires:

- `WINDOWS_PFX_BASE64`: Base64-encoded Code Signing Certificate (`.pfx`).
- `WINDOWS_PFX_PASSWORD`: Password for the PFX certificate.

Alternatively, Azure Trusted Signing can be configured using `azure/trusted-signing-action`.

*Note: Unsigned release builds produced without these secrets display explicit unsigned notices and must be tested in developer mode.*

---

## Manual QA Verification Matrix

Before promoting a release candidate (`v*`) to a stable production release, run the following verification matrix on clean test machines for each supported OS:

### Linux / macOS / Windows Test Flow

1. **Clean Installation**: Ensure no previous config or database exists in `<data_dir>`.
2. **First Launch & Identity**: Start `companion-daemon` or `companion-desktop`. Confirm Device ID is generated and stored securely in native secret storage (Keyring / Secret Service / DPAPI).
3. **Setup & Core Detection**: Run `companion-cli setup` or `companion-cli status`. Verify KiCad MCP Pro core bridge detection.
4. **Workspace Management**: Authorize a KiCad project directory. Attempt relative path escape (`../`) to verify fail-closed path boundary enforcement.
5. **Pairing & Remote Sessions**: Initiate pairing flow, approve session request, verify session status is Active.
6. **Read-only Tool Execution**: Execute a classified read-only tool (e.g. `pcb_get_layers`). Confirm execution succeeds and audit log records event.
7. **High-Risk Operation Approval**: Execute a high-risk tool (e.g. manufacturing Gerber export). Confirm operation is blocked pending explicit user approval.
8. **Revocation & Reconnect**: Revoke session from desktop/CLI. Attempt reconnection and verify revoked session cannot reactivate.
