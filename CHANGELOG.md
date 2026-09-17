# Changelog

All notable changes to this project are documented in this file. Format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Configuration File Support & Precedence**: Implemented `<data_dir>/config.toml` configuration layer with full precedence ordering (`CLI flags > Environment Variables > config.toml > Defaults`).
- **Conservative Tool Registry Classification**: Classified conservative read-only KiCad MCP tools (PCB, schematic, validation, and server metadata) with explicit capability mappings and low risk levels while keeping discovery and destructive tools fail-closed.
- **Desktop Settings V1 Screen**: Built functional V1 Settings UI exposing operational runtime parameters, configuration precedence rules, and privacy/security invariants.
- **Desktop Frontend Test Suite**: Established automated unit testing for React/Tauri frontend components using Vitest and React Testing Library, integrated into GitHub Actions CI pipeline.
- **Release Engineering Workflow**: Created `.github/workflows/release.yml` tag-triggered automated release pipeline generating multi-platform CLI/daemon binary packages and SHA-256 checksums (`SHA256SUMS.txt`).
- **Release Documentation**: Added `docs/development/release.md` detailing code signing (macOS Developer ID, Windows Authenticode), notarization, release engineering, and multi-OS manual QA procedures.

### Changed

- Updated GitHub Actions CI workflow to run frontend tests (`pnpm test`).
- Upgraded project release readiness status to V1 Release Candidate.

## [0.1.0] - 2026-09-17

Initial Companion V1 release candidate.
