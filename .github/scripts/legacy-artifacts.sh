#!/usr/bin/env bash
# Rebuilds the pre-dist asset names from a dist release's archives,
# without recompiling: each binary or library is gzipped under its old
# name with an `openssl dgst -r`-style `.sha256` file.
#
# Usage: legacy-artifacts.sh <tag> <out-dir>
# Needs GH_TOKEN and GITHUB_REPOSITORY, and LLVM for the Windows import
# libraries (see import-lib.sh).
set -euo pipefail

scripts=$(cd "$(dirname "$0")" && pwd)
tag=$1
out=$(mkdir -p "$2" && cd "$2" && pwd)
package=${tag%-v*}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

gzip_and_sum() {
    local src=$1 name=$2
    gzip --no-name --best --stdout "$src" > "$out/$name"
    (cd "$out" && sha256sum --binary "$name" > "$name.sha256")
}

# Prints the path of <file> after unpacking <target>'s archive. Archives
# may or may not wrap their files in a top-level directory.
extract() {
    local target=$1 file=$2 archive path
    if [[ ! -d "$target" ]]; then
        if [[ $target == *windows* ]]; then archive="$package-$target.zip"; else archive="$package-$target.tar.xz"; fi
        gh release download "$tag" --repo "$GITHUB_REPOSITORY" -p "$archive" --clobber
        mkdir "$target"
        case "$archive" in
            *.zip) unzip -q "$archive" -d "$target" ;;
            *) tar xJf "$archive" -C "$target" ;;
        esac
    fi
    path=$(find "$target" -type f -name "$file" | head -1)
    [[ -n "$path" ]] || { echo "$file not found in $archive" >&2; return 1; }
    echo "$path"
}

case "$package" in
    pact_verifier_cli)
        # Linux names always pointed at the static musl builds.
        for entry in \
            x86_64-unknown-linux-musl:pact-verifier:pact-verifier-linux-x86_64.gz \
            aarch64-unknown-linux-musl:pact-verifier:pact-verifier-linux-aarch64.gz \
            x86_64-apple-darwin:pact-verifier:pact-verifier-macos-x86_64.gz \
            aarch64-apple-darwin:pact-verifier:pact-verifier-macos-aarch64.gz \
            x86_64-pc-windows-msvc:pact-verifier.exe:pact-verifier-windows-x86_64.exe.gz \
            aarch64-pc-windows-msvc:pact-verifier.exe:pact-verifier-windows-aarch64.exe.gz
        do
            IFS=: read -r target file name <<<"$entry"
            gzip_and_sum "$(extract "$target" "$file")" "$name"
        done
        ;;
    pact_ffi)
        for entry in \
            x86_64-unknown-linux-gnu:linux-x86_64 \
            aarch64-unknown-linux-gnu:linux-aarch64 \
            x86_64-unknown-linux-musl:linux-x86_64-musl \
            aarch64-unknown-linux-musl:linux-aarch64-musl
        do
            IFS=: read -r target suffix <<<"$entry"
            gzip_and_sum "$(extract "$target" libpact_ffi.so)" "libpact_ffi-$suffix.so.gz"
            gzip_and_sum "$(extract "$target" libpact_ffi.a)" "libpact_ffi-$suffix.a.gz"
        done
        for entry in x86_64-apple-darwin:macos-x86_64 aarch64-apple-darwin:macos-aarch64; do
            IFS=: read -r target suffix <<<"$entry"
            gzip_and_sum "$(extract "$target" libpact_ffi.dylib)" "libpact_ffi-$suffix.dylib.gz"
            gzip_and_sum "$(extract "$target" libpact_ffi.a)" "libpact_ffi-$suffix.a.gz"
        done
        # dist ships the DLL and the static library, but not the DLL's
        # import library; it is rebuilt from the DLL's export table.
        for arch in x86_64 aarch64; do
            target=$arch-pc-windows-msvc
            dll=$(extract "$target" pact_ffi.dll)
            "$scripts/import-lib.sh" "$dll" "$arch" "$target/pact_ffi.dll.lib"
            gzip_and_sum "$dll" "pact_ffi-windows-$arch.dll.gz"
            gzip_and_sum "$target/pact_ffi.dll.lib" "pact_ffi-windows-$arch.dll.lib.gz"
            gzip_and_sum "$(extract "$target" pact_ffi.lib)" "pact_ffi-windows-$arch.lib.gz"
        done
        gh release download "$tag" --repo "$GITHUB_REPOSITORY" -p pact.h -p pact-cpp.h -D "$out" --clobber
        ;;
    *)
        echo "No legacy assets for $package" >&2
        ;;
esac
