#!/usr/bin/env bash
# Checks crate-published.sh against the crates.io index, and against a
# local index through CRATES_INDEX_URL. Needs network access.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../crate-published.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

expect() {
    local want=$1; shift
    if "$script" "$@"; then got=yes; else got=no; fi
    [[ $got == "$want" ]] || { echo "FAIL $*: expected $want, got $got"; exit 1; }
}
expect yes pact_models 1.3.16
expect no pact_models 9.9.9

# Index paths depend on name length: 1/a, 2/ab, 3/a/abc, ab/cd/abcd...
mkdir -p "$work/1" "$work/3/a" "$work/pa/ct"
echo '{"name":"a","vers":"1.0.0"}' > "$work/1/a"
echo '{"name":"abc","vers":"0.1.0"}' > "$work/3/a/abc"
printf '%s\n' '{"name":"pact_x","vers":"1.0.0"}' '{"name":"pact_x","vers":"1.1.0"}' > "$work/pa/ct/pact_x"
export CRATES_INDEX_URL="file://$work"
expect yes a 1.0.0
expect no a 1.0.1
expect yes abc 0.1.0
expect yes pact_x 1.0.0
expect no pact_x 1.1
expect no missing 1.0.0
echo PASS
