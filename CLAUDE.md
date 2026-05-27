# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the reference implementation of [Pact](https://pact.io) contract testing written in Rust. It provides shared libraries for consumer and provider testing, with FFI bindings for use in other languages. The project supports Pact specification V1–V4.

> Note: The mock server crates (`pact_mock_server`, `pact_mock_server_cli`) have moved to https://github.com/pact-foundation/pact-core-mock-server and are excluded from this workspace.

## Build & Test Commands

All Cargo commands must be run from the `rust/` subdirectory (the workspace root):

```bash
cd rust

# Build all crates
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test --package pact_models

# Run a single test by name (exact match)
cargo test --package pact_models -- my_test_name --exact

# Run tests in a specific module
cargo test --package pact_matching headers::tests

# Run tests with log output
RUST_LOG=debug cargo test -- --nocapture

# Lint
cargo clippy

# Check without building (faster)
cargo check

# Release build (strip=true, opt-level="z")
cargo build --release
```

### WASM targets (pact_models and pact_matching only)

```bash
cargo build --target wasm32-wasip2
```

### Cross-compilation / MUSL static builds

```bash
docker run --rm --user "$(id -u)":"$(id -g)" \
  -v $(pwd):/workspace -w /workspace/rust \
  -t -e TZ=UTC pactfoundation/rust-musl-build \
  ./scripts/ci-musl-build.sh
```

## Crate Architecture

The dependency graph flows upward — each layer depends on those below it:

```
pact_models          ← Pact data structures, reading/writing pact files
    ↑
pact_matching        ← Request/response matching logic
    ↑           ↖
pact_verifier    pact_consumer   ← Verification library / Consumer DSL
    ↑                ↑
pact_verifier_cli  pact_ffi      ← Standalone CLI / C FFI bindings
```

### Crates

- **pact_models**: Core data structures for Pact contracts. Handles JSON serialization of pact files. WASM-compatible (no `tokio`/`reqwest` on WASM). Features: `datetime`, `xml`, `form_urlencoded`.

- **pact_matching**: Implements all matching rules and interaction matching. Supports plugin-based extensibility via `pact-plugin-driver`. Features: `datetime`, `xml`, `plugins`, `multipart`, `form_urlencoded`.

- **pact_consumer**: DSL for writing consumer tests in Rust. Integrates with `pact_mock_server` (external repo). Features: `datetime`, `xml`, `plugins`, `multipart`, `tls`, `colour`.

- **pact_verifier**: Core provider verification logic (library). Compiled as `cdylib` + `rlib`. Features: `datetime`, `xml`, `plugins`, `multipart`.

- **pact_ffi**: C-compatible FFI layer exposing consumer and verifier APIs. Compiled as `cdylib`, `staticlib`, and `rlib`. Uses Rust edition 2021 (differs from other crates, which use 2024).

- **pact_verifier_cli**: Command-line tool wrapping `pact_verifier`. Features include `junit` output format.

## Key Conventions

- **Rust edition**: 2024 for all crates except `pact_ffi` (2021).
- **Minimum Rust version**: 1.88.0 (required for edition 2024).
- **Async runtime**: `tokio` throughout (except WASM builds).
- **Logging**: `tracing` / `tracing-subscriber` (not `log`).
- **Test helpers**: `rstest`, `hamcrest2`, `expectest`, `pretty_assertions` are common dev-dependencies.
- **Workspace patch**: `pact_models` is patched in `rust/Cargo.toml` to always use the local path (`./pact_models`), so changes propagate across all crates without a publish.

## Commit Messages

Follow [Conventional Changelog](https://github.com/bcoe/conventional-changelog-standard/blob/master/convention.md) format:
- `feat: ...` for new features
- `fix: ...` for bug fixes
- `chore: ...` for maintenance
- `refactor: ...` for refactoring

## Compatibility Suite

BDD-style integration tests covering V1–V4 spec compliance live in `/compatibility-suite/`. These use the `cucumber` framework and run separately from the main workspace tests.

## Releasing

Each crate has a `release.groovy` script that guides the release process and updates changelogs. Releases are published to [crates.io](https://crates.io) and tagged on GitHub. CI builds attach platform binaries to each GH release.
