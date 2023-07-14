load("//:repositories2.bzl", "crate_bazel_deps")

def _non_module_deps_impl(ctx):
    crate_bazel_deps()


pact_ref_deps = module_extension(
    implementation = _non_module_deps_impl,
)
