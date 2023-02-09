

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("@bazel_tools//tools/build_defs/repo:git.bzl","git_repository")

def repos():
    maybe(
        http_archive,
        name = "rules_rust",
        sha256 = "d125fb75432dc3b20e9b5a19347b45ec607fabe75f98c6c4ba9badaab9c193ce",
        urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.17.0/rules_rust-v0.17.0.tar.gz"],
    )

    maybe(
        git_repository,
        name = "source",
        remote = "https://github.com/opicaud/pact-reference",
        commit = "e6946c41cb5298d4fda311f326c37e02f43c2479",
        shallow_since = "1676312949 +0100",
        strip_prefix = "rust",
    )
