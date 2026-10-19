#!/usr/bin/env bash
# Builds the MSVC import library (`<name>.dll.lib`) for a DLL from its
# export table. dist ships the DLL but not the import library that rustc
# writes beside it.
#
# Exports outside the executable section are data, and are marked DATA so
# the library gives them no call thunk, as rustc's does.
#
# Usage: legacy-import-lib.sh <dll> <x86_64|aarch64> <out.lib>
#
# Only the legacy asset names (pact_ffi and pact_verifier_cli) need this.
# Delete it with the other legacy-* files once neither crate publishes
# them; see .github/workflows/legacy-artifacts.yml.
set -euo pipefail

dll=$1
arch=$2
out=$3

llvm() {
    local tool=$1
    command -v "$tool" \
        || { ls /usr/lib/llvm-*/bin/"$tool" /opt/homebrew/opt/llvm/bin/"$tool" 2>/dev/null || true; } | sort -V | tail -1
}
readobj=$(llvm llvm-readobj)
dlltool=$(llvm llvm-dlltool)
[[ -n $readobj && -n $dlltool ]] || { echo "legacy-import-lib.sh needs llvm-readobj and llvm-dlltool" >&2; exit 1; }

case "$arch" in
    x86_64) machine=i386:x86-64 ;;
    aarch64) machine=arm64 ;;
    *) echo "unknown arch: $arch" >&2; exit 1 ;;
esac

def=$(mktemp)
trap 'rm -f "$def"' EXIT
{
    echo "LIBRARY $(basename "$dll")"
    echo "EXPORTS"
    # Pass 1 collects the executable sections' address ranges; pass 2
    # prints each export, with DATA when its RVA falls outside them.
    awk '
        function hex(s,    i, v) {
            s = tolower(s); sub(/^0x/, "", s); v = 0
            for (i = 1; i <= length(s); i++) v = v * 16 + index("0123456789abcdef", substr(s, i, 1)) - 1
            return v
        }
        BEGIN { n = 0 }
        NR == FNR {
            if ($1 == "VirtualSize:") size = hex($2)
            if ($1 == "VirtualAddress:") addr = hex($2)
            if ($1 == "IMAGE_SCN_MEM_EXECUTE") { lo[n] = addr; hi[n] = addr + size; n++ }
            next
        }
        $1 == "Name:" { name = $2 }
        $1 == "RVA:" {
            rva = hex($2); data = 1
            for (i = 0; i < n; i++) if (rva >= lo[i] && rva < hi[i]) data = 0
            print "    " name (data ? " DATA" : "")
        }
    ' <("$readobj" --sections "$dll") <("$readobj" --coff-exports "$dll")
} > "$def"

"$dlltool" -m "$machine" -d "$def" -l "$out"
