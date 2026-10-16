#!/usr/bin/env bash
# Generates pact.h and pact-cpp.h in the current directory; dist uploads
# them with each pact_ffi release.
set -euo pipefail

ffi_dir=$(cd "$(dirname "$0")" && pwd)
# cbindgen expands macros with -Zunpretty=expanded, a nightly-only flag.
rustup run nightly rustc --version >/dev/null 2>&1 \
    || rustup toolchain install nightly --profile minimal
(cd "$ffi_dir" && rustup run nightly cbindgen --config cbindgen.toml --crate pact_ffi --output "$OLDPWD/pact.h")
(cd "$ffi_dir" && rustup run nightly cbindgen --config cbindgen-c++.toml --crate pact_ffi --output "$OLDPWD/pact-cpp.h")
