#!/usr/bin/env bash
# The cargo that dist runs in the musl and Windows build jobs (dist reads
# `CARGO`; see .github/build-setup.yml).
#
# - pact_ffi links the musl CRT dynamically: rustc drops a cdylib on a
#   crt-static target. dist appends `+crt-static` to RUSTFLAGS for musl, so
#   CARGO_ENCODED_RUSTFLAGS, which cargo prefers, overrides it for
#   pact_ffi's build only. The CLI stays fully static. precise-builds gives
#   each crate its own cargo invocation.
# - dist treats aarch64-musl in an x86_64 container as a cross build and
#   runs `cargo zigbuild`. The cross-rs image already sets up its musl
#   toolchain, so this runs a plain `cargo build`.
# - On Windows, dist packages pact_ffi.dll but drops pact_ffi.dll.lib, the
#   import library rustc writes beside it. MSVC links a DLL only through
#   its import library. dist zips the archive directory after cargo exits,
#   so the import library copied there ships in the archive.
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

target='' package=''
if [[ ${args[0]-} == build ]]; then
    for ((i = 1; i < ${#args[@]}; i++)); do
        case "${args[i]}" in
            --target) target=${args[i + 1]-} ;;
            --package) package=$(crate_name "${args[i + 1]-}") ;;
        esac
    done
fi

if [[ $target == *-linux-musl && $package == pact_ffi ]]; then
    export CARGO_ENCODED_RUSTFLAGS="-Ctarget-feature=-crt-static"
fi

if [[ $target == *-windows-msvc && $package == pact_ffi ]]; then
    archive_dir="$(cd "$(dirname "$0")/../.." && pwd)/target/distrib/pact_ffi-$target"
    messages=$(mktemp)
    trap 'rm -f "$messages"' EXIT
    # dist reads cargo's JSON messages from stdout.
    cargo "${args[@]}" | tee "$messages"
    implib=$(jq -r 'select(.reason == "compiler-artifact" and .target.name == "pact_ffi")
        | .filenames[] | select(endswith(".dll.lib"))' "$messages")
    [[ -n $implib ]] || { echo "dist-cargo.sh: cargo reported no pact_ffi.dll.lib" >&2; exit 1; }
    [[ -d $archive_dir ]] || { echo "dist-cargo.sh: no archive directory $archive_dir" >&2; exit 1; }
    cp "$(cygpath -u "$implib")" "$archive_dir/"
    exit 0
fi

exec cargo "${args[@]}"
