#!/usr/bin/env bash
# Builds pact_ffi in release through its CMake project, installs it under
# build/install (lib/ + include/pact.h), then builds and runs every example
# in examples/ against that install.
set -euo pipefail

cd "$(dirname "$0")"
install_dir="$PWD/build/install"

# MARK: Library

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
cmake --build build --config Release --target generate_header
cmake --install build --config Release --prefix "$install_dir"

# MARK: Examples

for example in examples/*/; do
    name=$(basename "$example")
    build_dir="build/examples/$name"
    echo "::group::Example $name"
    cmake -S "$example" -B "$build_dir" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$build_dir" --config Release
    case "$(uname -s)" in
        MINGW* | MSYS* | CYGWIN*)
            PATH="$install_dir/lib:$PATH" "$build_dir/Release/example.exe"
            ;;
        Darwin)
            DYLD_LIBRARY_PATH="$install_dir/lib" "$build_dir/example"
            ;;
        *)
            LD_LIBRARY_PATH="$install_dir/lib" "$build_dir/example"
            ;;
    esac
    echo "::endgroup::"
done
