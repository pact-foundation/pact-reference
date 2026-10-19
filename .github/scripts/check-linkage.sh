#!/usr/bin/env bash
# Fails when a Linux artifact in a dist build links against the wrong C
# library:
# - musl: a shared object must need musl's C library (`libc.so`, or
#   Alpine's `libc.musl-<arch>.so.1`), never glibc's libc.so.6; an
#   executable must be fully static.
# - gnu: no symbol may need a glibc version newer than GLIBC_FLOOR.
#
# Usage: check-linkage.sh <dir>
# <dir> holds dist's `<package>-<target>.tar.xz` archives, e.g. a release
# build's `artifacts-*` artifacts, merged.
set -euo pipefail

# The oldest glibc among supported major distributions: RHEL 8 and its
# rebuilds. dist-workspace.toml builds linux-gnu in manylinux_2_28 to match.
GLIBC_FLOOR=2.28

dir=$1
readelf=$(command -v readelf || command -v llvm-readelf \
    || { ls /usr/lib/llvm-*/bin/llvm-readelf /opt/homebrew/opt/llvm/bin/llvm-readelf 2>/dev/null || true; } | sort -V | tail -1)
[[ -n $readelf ]] || { echo "check-linkage.sh needs readelf" >&2; exit 1; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Checks one ELF file against <flavour>: musl or gnu.
check() {
    local file=$1 flavour=$2 needed newest
    needed=$("$readelf" -d "$file" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\].*/\1/p')
    case "$flavour" in
        musl)
            if [[ "$("$readelf" -h "$file")" == *"EXEC (Executable file)"* || -z "$needed" ]]; then
                [[ -z "$needed" ]] || { echo "::error::$file is not fully static: needs $needed"; return 1; }
            elif grep -qx 'libc.so.6' <<<"$needed" || ! grep -qxE 'libc\.so|libc\.musl-[a-z0-9_]+\.so\.1' <<<"$needed"; then
                echo "::error::$file does not link musl's libc.so: needs $(tr '\n' ' ' <<<"$needed")"
                return 1
            fi
            ;;
        gnu)
            newest=$("$readelf" -V "$file" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -1)
            if [[ -n $newest && $(printf '%s\n%s\n' "$newest" "$GLIBC_FLOOR" | sort -V | tail -1) != "$GLIBC_FLOOR" ]]; then
                echo "::error::$file needs glibc $newest, above the $GLIBC_FLOOR floor"
                return 1
            fi
            ;;
    esac
}

shopt -s nullglob
status=0 checked=0
for archive in "$dir"/*-unknown-linux-gnu.tar.xz "$dir"/*-unknown-linux-musl.tar.xz; do
    name=$(basename "$archive" .tar.xz)
    mkdir "$work/$name"
    tar xJf "$archive" -C "$work/$name"
    # Shared objects and executables; static archives link nothing.
    while IFS= read -r -d '' file; do
        check "$file" "${name##*-}" || status=1
        echo "checked ${file#"$work/"}"
        checked=$((checked + 1))
    done < <(find "$work/$name" -type f \( -name '*.so' -o -perm -u+x \) -print0)
done
(( checked )) || { echo "::error::no Linux archives in $dir"; exit 1; }
exit "$status"
