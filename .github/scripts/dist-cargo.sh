#!/usr/bin/env bash
# The cargo that dist runs in the musl build jobs (dist reads `CARGO`; see
# .github/build-setup.yml).
#
# - pact_ffi links the musl CRT dynamically: rustc drops a cdylib on a
#   crt-static target. dist appends `+crt-static` to RUSTFLAGS for musl, so
#   CARGO_ENCODED_RUSTFLAGS, which cargo prefers, overrides it for
#   pact_ffi's build only. The CLI stays fully static. precise-builds gives
#   each crate its own cargo invocation.
# - dist treats aarch64-musl in an x86_64 container as a cross build and
#   runs `cargo zigbuild`. The cross-rs image already sets up its musl
#   toolchain, so this runs a plain `cargo build`.
set -euo pipefail

args=("$@")
[[ ${args[0]-} == zigbuild ]] && args[0]=build

# Prints the crate name of a package spec: `name`, `name@ver`,
# `path+file:///.../name#ver` or `path+file:///...#name@ver`.
crate_name() {
    local spec=$1 fragment
    if [[ $spec == *'#'* ]]; then
        fragment=${spec#*#}
        if [[ $fragment == *@* ]]; then echo "${fragment%@*}"; else basename "${spec%%#*}"; fi
    else
        echo "${spec%@*}"
    fi
}

if [[ ${args[0]-} == build ]]; then
    target='' package=''
    for ((i = 1; i < ${#args[@]}; i++)); do
        case "${args[i]}" in
            --target) target=${args[i + 1]-} ;;
            --package) package=$(crate_name "${args[i + 1]-}") ;;
        esac
    done
    if [[ $target == *-linux-musl && $package == pact_ffi ]]; then
        export CARGO_ENCODED_RUSTFLAGS="-Ctarget-feature=-crt-static"
    fi
fi

exec cargo "${args[@]}"
