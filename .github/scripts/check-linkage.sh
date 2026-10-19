#!/usr/bin/env bash
# Fails when a Linux artifact links against the wrong C library:
# - musl: a shared object must need musl's C library (`libc.so`, or
#   Alpine's `libc.musl-<arch>.so.1`), never glibc's libc.so.6; an
#   executable must be fully static.
# - gnu: no symbol may need a glibc version newer than <max-glibc>.
#
# Usage: legacy-check-linkage.sh <file> musl
#        legacy-check-linkage.sh <file> gnu <max-glibc>
#
# Only legacy-artifacts.sh uses this. Delete it after the legacy cut-off
# date set in legacy-artifacts.sh.
set -euo pipefail

file=$1
flavour=$2
readelf=$(command -v readelf || command -v llvm-readelf \
    || { ls /usr/lib/llvm-*/bin/llvm-readelf /opt/homebrew/opt/llvm/bin/llvm-readelf 2>/dev/null || true; } | sort -V | tail -1)
[[ -n $readelf ]] || { echo "legacy-check-linkage.sh needs readelf" >&2; exit 1; }

needed=$("$readelf" -d "$file" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\].*/\1/p')

case "$flavour" in
    musl)
        if [[ "$("$readelf" -h "$file")" == *"EXEC (Executable file)"* || -z "$needed" ]]; then
            [[ -z "$needed" ]] || { echo "::error::$file is not fully static: needs $needed"; exit 1; }
        elif grep -qx 'libc.so.6' <<<"$needed" || ! grep -qxE 'libc\.so|libc\.musl-[a-z0-9_]+\.so\.1' <<<"$needed"; then
            echo "::error::$file does not link musl's libc.so: needs $(tr '\n' ' ' <<<"$needed")"
            exit 1
        fi
        ;;
    gnu)
        max=$3
        newest=$("$readelf" -V "$file" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -1)
        if [[ -n $newest && $(printf '%s\n%s\n' "$newest" "$max" | sort -V | tail -1) != "$max" ]]; then
            echo "::error::$file needs glibc $newest, above the $max floor"
            exit 1
        fi
        ;;
    *) echo "unknown flavour: $flavour" >&2; exit 1 ;;
esac
