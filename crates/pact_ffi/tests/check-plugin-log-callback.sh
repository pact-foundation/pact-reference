#!/usr/bin/env bash
# Compile real call sites: including the header alone does not catch an opaque by-value argument.
set -euo pipefail

cd "$(dirname "$0")/.."
build_dir=$(mktemp -d)
trap 'rm -rf "$build_dir"' EXIT
mkdir -p "$build_dir/c" "$build_dir/cpp"

rustup run nightly cbindgen --config cbindgen.toml --crate pact_ffi --output "$build_dir/c/pact.h"
rustup run nightly cbindgen --config cbindgen-c++.toml --crate pact_ffi --output "$build_dir/cpp/pact.h"

"${CC:-cc}" -std=c11 -Wall -Wextra -Werror -fsyntax-only -I "$build_dir/c" tests/plugin_log_callback.c
"${CXX:-c++}" -x c++ -std=c++11 -Wall -Wextra -Werror -fsyntax-only -I "$build_dir/c" tests/plugin_log_callback.c
"${CXX:-c++}" -x c++ -std=c++11 -Wall -Wextra -Werror -fsyntax-only -I "$build_dir/cpp" tests/plugin_log_callback.c
