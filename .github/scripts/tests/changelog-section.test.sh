#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../changelog-section.sh"
fixture=$(mktemp)
trap 'rm -f "$fixture"' EXIT
cat > "$fixture" <<'MD'
# Changelog

## [1.3.17](https://example.test/1.3.17) - _2026-11-01_

### Bug Fixes

-   Fix thing

## [1.3.16] - _2026-10-01_

-   Older

## 1.3.15 - Pre-release-plz entry

* abc1234 - fix: before the switch
MD

out=$("$script" "$fixture" 1.3.17)
expected=$'\n### Bug Fixes\n\n-   Fix thing\n'
[[ "$out"$'\n' == "$expected" ]] || { echo "FAIL newest: got <$out>"; exit 1; }

out=$("$script" "$fixture" 1.3.16)
[[ "$out" == $'\n-   Older' ]] || { echo "FAIL older (must stop at a pre-release-plz heading): got <$out>"; exit 1; }

if "$script" "$fixture" 9.9.9 >/dev/null; then echo "FAIL missing should exit 1"; exit 1; fi
echo PASS
