load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository")

def crate_bazel_deps():
    crates_repository(
            name = "crate_index",
            cargo_lockfile = "//:Cargo.lock",
            generator = "@cargo_bazel_bootstrap//:cargo-bazel",
            lockfile = "//:cargo-bazel-lock.json",
            rust_toolchain_cargo_template = "@rust_host_tools//:bin/cargo",
            rust_toolchain_rustc_template = "@rust_host_tools//:bin/rustc",
            manifests = [
                "//:Cargo.toml",
                "//:pact_cli/Cargo.toml",
                "//:pact_consumer/Cargo.toml",
                "//:pact_ffi/Cargo.toml",
                "//:pact_matching/Cargo.toml",
                "//:pact_mock_server/Cargo.toml",
                "//:pact_mock_server_cli/Cargo.toml",
                "//:pact_models/Cargo.toml",
                "//:pact_verifier/Cargo.toml",
                "//:pact_verifier_cli/Cargo.toml",
            ],
            packages = {
               "os_info": crate.spec(
                version = "3.5.1",
                ),
            },
            annotations = {
                "onig_sys": [crate.annotation(
                shallow_since="1686508130 +0100"
                )],
                "onig": [crate.annotation(
                shallow_since="1686508130 +0100"
                )],
            },
            render_config = '{"build_file_template": "//:BUILD.{name}-{version}.bazel", "crate_label_template": "@crate_index__{name}-{version}//:{target}", "crate_repository_template": "crate_index__{name}-{version}", "crates_module_template": "//:{file}", "default_package_name": "None", "platforms_template": "@rules_rust//rust/platform:{triple}"}'
        )

def render_config(
        build_file_template = "//:BUILD.{name}-{version}.bazel",
        crate_label_template = "@{repository}__{name}-{version}//:{target}",
        crate_repository_template = "{repository}__{name}-{version}",
        crates_module_template = "//:{file}",
        default_package_name = None,
        platforms_template = "@rules_rust//rust/platform:{triple}",
        regen_command = None,
        vendor_mode = None):
    return json.encode(struct(
        build_file_template = build_file_template,
        crate_label_template = crate_label_template,
        crate_repository_template = crate_repository_template,
        crates_module_template = crates_module_template,
        default_package_name = default_package_name,
        platforms_template = platforms_template,
        regen_command = regen_command,
        vendor_mode = vendor_mode,
    ))