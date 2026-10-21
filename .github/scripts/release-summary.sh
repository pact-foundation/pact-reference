#!/usr/bin/env bash
# Tidies the markdown summary that `release-plz update` prints on stdout,
# which includes the cargo-semver-checks report for breaking changes.
#
# release-plz writes each report's code fences on the same line as its
# text, which GitHub reads as an unclosed block. The report's paths point
# into release-plz's temporary checkout; they are trimmed to repo paths.
#
# Usage: release-plz update ... | release-summary.sh
set -euo pipefail

awk '
    {
        gsub(/\/[^ ]*\/\.tmp[A-Za-z0-9]+\/[^\/ ]+\//, "")
        if (length($0) > 3 && substr($0, 1, 3) == "```") {
            print "```"
            $0 = substr($0, 4)
        }
        if (length($0) > 3 && substr($0, length($0) - 2) == "```") {
            print substr($0, 1, length($0) - 3)
            $0 = "```"
        }
        print
    }
'
