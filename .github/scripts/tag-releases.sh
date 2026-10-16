#!/usr/bin/env bash
# Creates `<crate>-v<version>` for each workspace crate whose version has
# no tag, i.e. each crate whose release PR just merged, and pushes it.
# Each tag goes in its own push: GitHub fires no events for a push of
# more than three tags.
#
# Usage: tag-releases.sh <manifest-path> <remote>
set -euo pipefail

manifest=$1
remote=$2

cargo metadata --format-version 1 --no-deps --manifest-path "$manifest" \
    | jq -r '.packages[] | "\(.name)-v\(.version)"' \
    | while read -r tag; do
        git rev-parse -q --verify "refs/tags/$tag" >/dev/null && continue
        git tag -a "$tag" -m "$tag"
        git push -q "$remote" "refs/tags/$tag"
        echo "$tag"
    done
