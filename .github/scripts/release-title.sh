#!/usr/bin/env bash
# Prints the GitHub release title for a release tag:
# `<crate> <version> - <date>`, the date taken from the version's
# changelog heading. Without a dated heading, `<crate> <version>`.
#
# Usage: release-title.sh <tag>
set -euo pipefail

tag=$1
crate=${tag%-v*}
version=${tag##*-v}

date=$(awk -v heading="## [${version}]" '
    index($0, heading) == 1 {
        if (match($0, /[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]/)) print substr($0, RSTART, RLENGTH)
        exit
    }
' "rust/$crate/CHANGELOG.md")

echo "$crate $version${date:+ - $date}"
