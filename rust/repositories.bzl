load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive", "http_file")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("@bazel_tools//tools/build_defs/repo:git.bzl","git_repository")

def repos():
    maybe(
        http_archive,
        name = "rules_rust",
        sha256 = "4a9cb4fda6ccd5b5ec393b2e944822a62e050c7c06f1ea41607f14c4fdec57a2",
        urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.25.1/rules_rust-v0.25.1.tar.gz"],
    )

    maybe(
        git_repository,
        name = "source",
        remote = "https://github.com/opicaud/pact-reference",
        branch = "master",
        strip_prefix = "rust",
    )

    maybe(
        http_file,
        name = "pact_verifier_cli_archive",
        sha256 = "77ffc38f4564cfef42f64b9eb33bebfc4d787e65ef7ff7213640a3d63d2cf5a7",
        urls = ["https://github.com/pact-foundation/pact-reference/releases/download/pact_verifier_cli-v1.0.1/pact_verifier_cli-osx-x86_64.gz"],
    )
