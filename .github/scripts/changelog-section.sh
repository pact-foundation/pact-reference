#!/usr/bin/env bash
# Prints the body of one version's section from a changelog whose
# sections are `##` headings. release-plz writes `## [<version>]`; the
# entries from before release-plz use `## <version> - <title>`.
#
# Usage: changelog-section.sh <CHANGELOG.md> <version>
# Exits 1 when the changelog has no section for <version>.
set -euo pipefail

changelog=$1
version=$2

awk -v heading="## [${version}]" '
    /^## / {
        if (found) exit
        if (index($0, heading) == 1) { found = 1; next }
    }
    found { print }
    END { exit found ? 0 : 1 }
' "$changelog"
