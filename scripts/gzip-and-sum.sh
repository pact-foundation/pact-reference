#!/bin/bash

# Gzip and sum a file.
#
# Usage: gzip_and_sum <orig_file> <target_file> [<digest_file>]
#
# - orig_file: the file to gzip and sum
# - target_file: the file to write the gzipped file to
# - digest_file: the file to write the digest to. If not provided, defaults to
#   <target_file>.sha256
gzip_and_sum() {
    orig_file=$1
    target_file=$2
    digest_file=${3:-$target_file.sha256}

    gzip --stdout --best "$orig_file" > "$target_file"
    openssl dgst -sha256 -r "$target_file" > "$digest_file"
    basenamedir=$(basename $target_file)
    if [[ "$(uname -s)" == "Darwin" ]]; then
        # OSX requires an empty arg passed to -i, but this doesn't work on Lin/Win
        sed -Ei '' "s|\*(.*)$|\*$basenamedir|" "$digest_file"
    else
        sed -Ei "s|\*(.*)$|\*$basenamedir|" "$digest_file"
    fi
}
