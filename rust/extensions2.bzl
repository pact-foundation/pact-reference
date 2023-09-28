load("@crate_index//:defs.bzl", "crate_repositories")
def _non_module_deps_impl(ctx):
    crate_repositories()


blob = module_extension(
    implementation = _non_module_deps_impl,
)

