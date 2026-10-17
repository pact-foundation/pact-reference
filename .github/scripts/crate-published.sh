#!/usr/bin/env bash
# Exits 0 when <crate> <version> is in the crates.io sparse index, 1 when it
# is not. A network error also exits 1, so callers treat "unknown" as "not
# published" and fall through to steps that fail safely.
#
# Usage: crate-published.sh <crate> <version>
# CRATES_INDEX_URL overrides the index root (default https://index.crates.io).
set -euo pipefail

name=$1
version=$2
index=${CRATES_INDEX_URL:-https://index.crates.io}

lower=$(tr '[:upper:]' '[:lower:]' <<<"$name")
case ${#lower} in
    1) path="1/$lower" ;;
    2) path="2/$lower" ;;
    3) path="3/${lower:0:1}/$lower" ;;
    *) path="${lower:0:2}/${lower:2:2}/$lower" ;;
esac

curl -fsS "$index/$path" 2>/dev/null | jq -e --arg v "$version" 'select(.vers == $v)' >/dev/null
