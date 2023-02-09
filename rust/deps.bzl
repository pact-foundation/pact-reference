load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")
load("@rules_rust//crate_universe:defs.bzl", "crates_repository", "crate")

def deps():
    rules_rust_dependencies()
    rust_register_toolchains(edition = "2021")
    crate_universe_dependencies(bootstrap = True)
    load_crat()

def load_crat():
    crates_repository(
        name = "crate_index",
        cargo_lockfile = "@source//:Cargo.lock",
        generator = "@cargo_bazel_bootstrap//:cargo-bazel",
        lockfile = "@source//:cargo-bazel-lock.json",
        manifests = [
            "@source//:Cargo.toml",
            "@source//:pact_cli/Cargo.toml",
            "@source//:pact_consumer/Cargo.toml",
            "@source//:pact_ffi/Cargo.toml",
            "@source//:pact_matching/Cargo.toml",
            "@source//:pact_mock_server/Cargo.toml",
            "@source//:pact_mock_server_cli/Cargo.toml",
            "@source//:pact_models/Cargo.toml",
            "@source//:pact_verifier/Cargo.toml",
            "@source//:pact_verifier_cli/Cargo.toml",
        ],
        packages = {
           "os_info": crate.spec(
            version = "3.5.1",
        )},
    )
