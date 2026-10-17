#!/usr/bin/env bash
# Fails when <crate> requires a workspace crate version that has no
# `<crate>-v<version>` tag, i.e. one that has not been released.
#
# Usage: release-deps.sh <crate>   (from the repository root, tags fetched)
set -euo pipefail

crate=$1
metadata=$(cargo metadata --format-version 1 --no-deps --manifest-path rust/Cargo.toml)
members=$(jq -r '.packages[].name' <<<"$metadata")
status=0

while IFS=$'\t' read -r dep req; do
    grep -qx "$dep" <<<"$members" || continue
    version=$(sed -E 's/^[~^=>< ]*//; s/,.*$//' <<<"$req")
    [[ $version =~ ^[0-9]+\.[0-9]+$ ]] && version="$version.0"
    if ! git rev-parse -q --verify "refs/tags/$dep-v$version" >/dev/null; then
        echo "::error::$crate requires $dep $req, but $dep-v$version is not released. Release $dep first."
        status=1
    fi
done < <(jq -r --arg c "$crate" '
    .packages[] | select(.name == $c) | .dependencies[]
    | select(.kind != "dev") | [.name, .req] | @tsv' <<<"$metadata")

exit "$status"
