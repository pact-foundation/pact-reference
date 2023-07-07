load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")
load("@rules_rust//crate_universe:defs.bzl", "crates_repository", "crate")

def deps(json = "cargo-bazel-lock.json"):
    rules_rust_dependencies()
    rust_register_toolchains(edition = "2021")
    crate_universe_dependencies(bootstrap = True)
    load_crat(json)

def load_crat(json):
    crates_repository(
        name = "crate_index",
        cargo_lockfile = "@source//:Cargo.lock",
        lockfile = "@//:"+json,
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
         annotations = {
            "onig_sys": [crate.annotation(
            shallow_since="1686508130 +0100"
            )],
            "onig": [crate.annotation(
            shallow_since="1686508130 +0100"
            )],
        }
    )
