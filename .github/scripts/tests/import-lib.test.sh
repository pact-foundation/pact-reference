#!/usr/bin/env bash
# Rebuilds the import library of a released pact_ffi DLL and checks that
# it defines the same symbols as the import library rustc produced.
# Needs network access and LLVM (llvm-nm, plus what import-lib.sh needs).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../import-lib.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

base=https://github.com/pact-foundation/pact-reference/releases/download/libpact_ffi-v0.5.11
nm=$(command -v llvm-nm || { ls /usr/lib/llvm-*/bin/llvm-nm /opt/homebrew/opt/llvm/bin/llvm-nm 2>/dev/null || true; } | sort -V | tail -1)
symbols() { "$nm" --defined-only "$1" 2>/dev/null | awk 'NF >= 3 && $3 != "@comp.id" { print $2, $3 }' | sort -u; }

for arch in x86_64 aarch64; do
    mkdir "$arch"
    # The DLL keeps its built name; the import library records it.
    curl -fsSL "$base/pact_ffi-windows-$arch.dll.gz" | gunzip > "$arch/pact_ffi.dll"
    curl -fsSL "$base/pact_ffi-windows-$arch.dll.lib.gz" | gunzip > "$arch/released.dll.lib"
    "$script" "$arch/pact_ffi.dll" "$arch" "$arch/rebuilt.dll.lib"
    if ! diff <(symbols "$arch/released.dll.lib") <(symbols "$arch/rebuilt.dll.lib"); then
        echo "FAIL $arch symbols differ"
        exit 1
    fi
done
echo PASS
