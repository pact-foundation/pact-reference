# Copilot Instructions

## Build, test, and lint commands

The main Rust workspace lives in `rust/`. Run Cargo commands there unless you are working on the compatibility suite.

```bash
cd rust

# Build and test the full workspace
cargo build
cargo test

# Lint and lighter validation
cargo clippy
cargo check
cargo check --no-default-features

# Run tests for one crate
cargo test --package pact_models

# Run one exact test
cargo test --package pact_models -- my_test_name --exact

# Run tests in one module
cargo test --package pact_matching headers::tests

# Run tests with tracing output
RUST_LOG=debug cargo test -- --nocapture

# Release build
cargo build --release
```

WASM builds are only checked for `pact_models` and `pact_matching`:

```bash
cd rust/pact_models && cargo build --target wasm32-wasip2
cd rust/pact_matching && cargo build --target wasm32-wasip2
```

The compatibility suite is a separate crate under `compatibility-suite/` and is run independently from the main workspace:

```bash
cd compatibility-suite
cargo test --test v1_consumer
cargo test --test v1_provider
cargo test --test v2_consumer
cargo test --test v2_provider
cargo test --test v3
cargo test --test v3_provider
cargo test --test v3_message
cargo test --test v4
```

Cross-platform MUSL builds use the project Docker image:

```bash
docker run --rm --user "$(id -u)":"$(id -g)" \
  -v $(pwd):/workspace -w /workspace/rust \
  -t -e TZ=UTC pactfoundation/rust-musl-build \
  ./scripts/ci-musl-build.sh
```

## High-level architecture

This repository is the Rust reference implementation of Pact and exposes the same core through Rust crates, a C-compatible FFI layer, and a verifier CLI.

The crate dependency graph flows upward:

```text
pact_models          <- Pact data structures and pact file IO
    ^
pact_matching        <- Request/response/message matching rules
    ^           \
pact_verifier    pact_consumer   <- Provider verification library / consumer DSL
    ^                ^
pact_verifier_cli  pact_ffi      <- CLI wrapper / foreign-language entry point
```

Key architectural boundaries:

- `pact_models` is the foundation: pact JSON models, parsing, serialization, and shared data structures.
- `pact_matching` builds on `pact_models` and owns matching logic, matcher execution, and plugin-aware matching.
- `pact_consumer` provides the Rust consumer DSL and depends on the external `pact_mock_server` crate.
- `pact_verifier` implements provider verification logic on top of the core model and matching crates.
- `pact_verifier_cli` is a thin executable wrapper over `pact_verifier`.
- `pact_ffi` exposes consumer and verifier capabilities to other languages through a C ABI.
- The mock server crates were moved to `pact-foundation/pact-core-mock-server` and are excluded from this workspace, even though some crates still depend on them as external packages.
- `compatibility-suite/` is separate from `rust/` and contains Cucumber-based spec compliance tests for Pact V1-V4.

## Key conventions

- Most crates use Rust edition 2024 and require Rust 1.88+. `pact_ffi` is the exception and still uses edition 2021.
- Keep dependencies aligned with the layering above. New code should normally depend "downward" into lower-level crates, not sideways or upward.
- Feature flags are intentionally propagated across crates (`datetime`, `xml`, `plugins`, `multipart`, `form_urlencoded`, `tls`, `junit`). When adding an optional capability, check whether it also needs wiring through dependent crates and target-specific builds.
- `pact_models` is patched in `rust/Cargo.toml` via `[patch.crates-io]` to the local path. Changes there automatically affect workspace crates without publishing.
- Logging and test output are centered on `tracing`/`tracing-subscriber`; tests commonly use `test-log` so `RUST_LOG=debug` is useful when debugging failures.
- Common test helpers across crates are `rstest`, `expectest`, `hamcrest2`, and `pretty_assertions`. Follow those existing choices before introducing new test utilities.
- CI exercises more than the default happy path: full workspace build/test, Linux `cargo clippy`, `cargo check --no-default-features`, WASM builds for `pact_models` and `pact_matching`, and a separate run with `PACT_MATCHING_ENGINE=v2`.
- Commit messages follow Conventional Changelog style (`feat:`, `fix:`, `chore:`, `refactor:`).
