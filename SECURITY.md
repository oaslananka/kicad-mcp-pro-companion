# Security Policy

KiCad MCP Pro Companion is a security boundary by design: it decides
whether a remote AI agent or cloud service may operate on a user's local
KiCad MCP Pro installation. Its threat model is documented in
[`docs/security/threat-model.md`](docs/security/threat-model.md) and its
trust boundaries in
[`docs/security/trust-boundaries.md`](docs/security/trust-boundaries.md).

## Supported versions

This project is pre-1.0 (`0.x`). Security fixes land on `main` and the
latest tagged `0.x` release. There is no long-term-support branch yet.

| Version | Supported |
|---|---|
| 0.1.x   | ✅ |
| < 0.1   | ❌ |

## Reporting a vulnerability

Please **do not** open a public GitHub issue for a suspected vulnerability.

Instead, use GitHub's private vulnerability reporting for this repository
(Security tab → "Report a vulnerability"), or open a
[GitHub Security Advisory](https://github.com/oaslananka/kicad-mcp-pro-companion/security/advisories/new).

Please include:

- A description of the issue and its potential impact.
- Steps to reproduce, or a minimal proof of concept.
- Whether the issue affects the local trust boundary (session/workspace/
  capability enforcement, secret storage) or the transport layer.

We aim to acknowledge reports within 5 business days. Disclosure timing is
coordinated with the reporter; we ask for a reasonable window to ship a fix
before public disclosure.

## Automated dependency and workflow checks

Repository CI includes cargo-audit, pnpm audit, OSV-Scanner, Dependency Review, and zizmor. The current configuration and any explicitly time-bounded dependency risk exceptions are documented in [`docs/development/security-automation.md`](docs/development/security-automation.md).

## Scope

In scope: this repository (daemon, CLI, desktop shell, all `crates/*`).

Out of scope: the KiCad MCP Pro server itself (report upstream at
[oaslananka/kicad-mcp-pro](https://github.com/oaslananka/kicad-mcp-pro)),
and any future hosted cloud service (not yet implemented; will have its own
policy when it exists).
