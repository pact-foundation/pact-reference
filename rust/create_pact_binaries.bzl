load("@rules_rust//cargo:defs.bzl", "cargo_bootstrap_repository")

def create_pact_binaries(name = "provide-a-name", binary = "provide-a-binray"):
    cargo_bootstrap_repository(
        name = name,
        binary = binary,
        cargo_lockfile = "@source//:Cargo.lock",
        cargo_toml = "@source//:Cargo.toml",
    )
