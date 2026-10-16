#!/usr/bin/env bash
# Checks check-linkage.sh against the libraries of the last pre-dist
# release: its musl library links musl, its gnu library needs glibc 2.18.
# Needs network access and readelf (binutils or LLVM).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../check-linkage.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

base=https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v0.5.11
curl -fsSL "$base/libpact_ffi-linux-aarch64-musl.so.gz" | gunzip > musl.so
curl -fsSL "$base/libpact_ffi-linux-aarch64.so.gz" | gunzip > gnu.so

expect() {
    local want=$1; shift
    if "$script" "$@" >/dev/null 2>&1; then got=pass; else got=fail; fi
    [[ $got == "$want" ]] || { echo "FAIL $*: expected $want, got $got"; exit 1; }
}
expect pass musl.so musl
expect fail gnu.so musl
expect pass gnu.so gnu 2.28
expect pass gnu.so gnu 2.18
expect fail gnu.so gnu 2.17
echo PASS
