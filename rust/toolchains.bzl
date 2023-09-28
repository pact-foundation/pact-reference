"""pact_reference toolchain implementation"""

def _pact_reference_toolchain_impl(ctx):
    return platform_common.ToolchainInfo(
        pact_verifier_cli = ctx.file.pact_verifier_cli,
    )

pact_reference_toolchain = rule(
    implementation = _pact_reference_toolchain_impl,
    doc = "A pact reference toolchain",
    attrs = {
        "pact_verifier_cli": attr.label(
            doc = "A pact reference binary",
            allow_single_file = True,
            mandatory = True,
        ),
    },
)