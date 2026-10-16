#!/usr/bin/env bash
# Creates `<crate>-v<version>` for each workspace crate whose version has
# no tag, i.e. each crate whose release PR just merged, and pushes it.
# Each tag goes in its own push: GitHub fires no events for a push of
# more than three tags.
#
# Usage: tag-releases.sh <manifest-path> <remote>
set -euo pipefail

scripts=$(cd "$(dirname "$0")" && pwd)
manifest=$1
remote=$2

cargo metadata --format-version 1 --no-deps --manifest-path "$manifest" \
    | jq -r '.packages[] | [.name, .version, .manifest_path] | @tsv' \
    | while IFS=$'\t' read -r name version crate_manifest; do
        tag="$name-v$version"
        git rev-parse -q --verify "refs/tags/$tag" >/dev/null && continue
        # Published but untagged means a missing migration tag or a manual
        # publish, never a release PR merge: tagging would start a release.
        if "$scripts/crate-published.sh" "$name" "$version"; then
            echo "::warning::$name $version is on crates.io but has no $tag tag; not tagging it." >&2
            continue
        fi
        # Tag the commit on master's first-parent line that set this
        # version, which is the release PR's merge (squash or merge commit),
        # even when later commits have landed since.
        commit=$(git log -1 --first-parent --diff-merges=first-parent --no-patch --format=%H -G "^version = \"${version//./\\.}\"" -- "$crate_manifest")
        git tag -a "$tag" -m "$tag" "${commit:-HEAD}"
        git push -q "$remote" "refs/tags/$tag"
        echo "$tag"
    done
