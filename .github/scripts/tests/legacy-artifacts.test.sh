#!/usr/bin/env bash
# Runs legacy-artifacts.sh against archives in dist's layout, served by a
# stub `gh`, and checks the output names against the assets of the last
# pre-dist releases. Needs network access (for the released binaries it
# repackages), LLVM (see import-lib.sh) and readelf.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../legacy-artifacts.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
fixtures="$work/fixtures"
mkdir -p "$fixtures" "$work/bin"

# `gh release download <tag> --repo <repo> -p <pattern>... [-D <dir>] --clobber`
cat > "$work/bin/gh" <<'GH'
#!/usr/bin/env bash
set -euo pipefail
dest=.
patterns=()
shift 3
while (($#)); do
    case "$1" in
        -p) patterns+=("$2"); shift 2 ;;
        -D) dest=$2; shift 2 ;;
        --repo) shift 2 ;;
        *) shift ;;
    esac
done
for p in "${patterns[@]}"; do cp "$FIXTURES/$p" "$dest/"; done
GH
chmod +x "$work/bin/gh"

# Packs <files> as dist does: .tar.xz under a <package>-<target>/ directory,
# .zip under the same directory name.
pack() {
    local package=$1 target=$2; shift 2
    local dir="$work/stage/$package-$target"
    mkdir -p "$dir"
    for f in "$@"; do [[ -e "$dir/$f" ]] || echo "$package $target $f" > "$dir/$f"; done
    if [[ $target == *windows* ]]; then
        (cd "$work/stage" && zip -qr "$fixtures/$package-$target.zip" "$package-$target")
    else
        tar cJf "$fixtures/$package-$target.tar.xz" -C "$work/stage" "$package-$target"
    fi
}

# Real binaries where the script inspects them: Windows DLLs for the import
# libraries, Linux libraries and executables for the linkage checks.
releases=https://github.com/pact-foundation/pact-reference/releases/download
fetch() {
    mkdir -p "$(dirname "$2")"
    curl -fsSL "$releases/$1" | gunzip > "$2"
}
for arch in x86_64 aarch64; do
    fetch "libpact_ffi-v0.5.11/pact_ffi-windows-$arch.dll.gz" "$work/stage/pact_ffi-$arch-pc-windows-msvc/pact_ffi.dll"
    fetch "libpact_ffi-v0.5.11/libpact_ffi-linux-$arch.so.gz" "$work/stage/pact_ffi-$arch-unknown-linux-gnu/libpact_ffi.so"
    fetch "libpact_ffi-v0.5.11/libpact_ffi-linux-$arch-musl.so.gz" "$work/stage/pact_ffi-$arch-unknown-linux-musl/libpact_ffi.so"
    fetch "pact_verifier_cli-v1.3.5/pact-verifier-linux-$arch.gz" "$work/stage/pact_verifier_cli-$arch-unknown-linux-musl/pact-verifier"
done

for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
    pack pact_ffi "$t" libpact_ffi.so libpact_ffi.a
    pack pact_verifier_cli "$t" pact-verifier
done
for t in x86_64-apple-darwin aarch64-apple-darwin; do
    pack pact_ffi "$t" libpact_ffi.dylib libpact_ffi.a
    pack pact_verifier_cli "$t" pact-verifier
done
for t in x86_64-pc-windows-msvc aarch64-pc-windows-msvc; do
    pack pact_ffi "$t" pact_ffi.dll pact_ffi.lib
    pack pact_verifier_cli "$t" pact-verifier.exe
done
echo header > "$fixtures/pact.h"
echo header > "$fixtures/pact-cpp.h"

export FIXTURES=$fixtures GH_TOKEN=unused GITHUB_REPOSITORY=pact-foundation/pact-reference
export PATH="$work/bin:$PATH"

for case in pact_ffi-v0.5.12:legacy-expected-ffi.txt pact_verifier_cli-v1.4.0:legacy-expected-cli.txt; do
    tag=${case%%:*}
    expected="$here/${case#*:}"
    "$script" "$tag" "$work/out-$tag"
    if ! diff "$expected" <(ls "$work/out-$tag" | sort); then
        echo "FAIL $tag asset names differ from $(basename "$expected")"
        exit 1
    fi
done

# Each checksum file names its asset in `openssl dgst -r` style, and each
# asset unpacks to what the archive held.
out="$work/out-pact_verifier_cli-v1.4.0"
read -r sum name < "$out/pact-verifier-linux-x86_64.gz.sha256"
[[ $name == "*pact-verifier-linux-x86_64.gz" && ${#sum} == 64 ]] || { echo "FAIL checksum format: $sum $name"; exit 1; }
cmp -s <(gunzip -c "$out/pact-verifier-linux-x86_64.gz") "$work/stage/pact_verifier_cli-x86_64-unknown-linux-musl/pact-verifier" \
    || { echo "FAIL Linux CLI asset is not the musl build"; exit 1; }

# A glibc-linked musl library must stop the job.
cp "$work/stage/pact_ffi-x86_64-unknown-linux-gnu/libpact_ffi.so" "$work/stage/pact_ffi-x86_64-unknown-linux-musl/libpact_ffi.so"
rm "$fixtures/pact_ffi-x86_64-unknown-linux-musl.tar.xz"
pack pact_ffi x86_64-unknown-linux-musl libpact_ffi.so libpact_ffi.a
if "$script" pact_ffi-v0.5.12 "$work/out-bad" >/dev/null 2>&1; then
    echo "FAIL a glibc-linked musl library was accepted"
    exit 1
fi
echo PASS
