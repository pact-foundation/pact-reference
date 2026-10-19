#!/usr/bin/env bash
# Exits 0 when <crate> <version> is on crates.io, non-zero when it is not.
# A network error also fails, so callers treat "unknown" as "not
# published" and fall through to steps that fail safely.
#
# Usage: crate-published.sh <crate> <version>
set -euo pipefail

curl -fsS -o /dev/null -A 'pact-foundation/pact-reference release' \
    "https://crates.io/api/v1/crates/$1/$2" 2>/dev/null
