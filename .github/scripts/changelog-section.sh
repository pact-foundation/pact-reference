#!/usr/bin/env bash
# Prints the body of one version's section from a release-plz changelog,
# where sections start with `## [<version>]`. Entries written before
# release-plz start with a single `#` and also end a section.
#
# Usage: changelog-section.sh <CHANGELOG.md> <version>
# Exits 1 when the changelog has no section for <version>.
set -euo pipefail

changelog=$1
version=$2

awk -v heading="## [${version}]" '
    /^##? / {
        if (found) exit
        if (index($0, heading) == 1) { found = 1; next }
    }
    found { print }
    END { exit found ? 0 : 1 }
' "$changelog"
