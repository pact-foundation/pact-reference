#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_NAME=pact_ffi

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$RUST_DIR/target"}

# All flags passed to this script are passed to cargo.
case $1 in
x86_64-pc-windows-msvc)
    TARGET=$1
    shift
    ;;
aarch64-pc-windows-msvc)
    TARGET=$1
    shift
    ;;
*) ;;
esac
cargo_flags=("$@")
build_target() {
    TARGET=$1

    case $TARGET in
    x86_64-pc-windows-msvc)
        FILE_SUFFIX=windows-x86_64
        ;;
    aarch64-pc-windows-msvc)
        FILE_SUFFIX=windows-aarch64
        ;;
    *)
        echo unknown target $TARGET
        exit 1
        ;;
    esac
    cargo build --target $TARGET "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.dll" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.dll.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.dll.lib" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.dll.lib.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.lib" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.lib.gz"
    fi
}

if [ ! -z "$TARGET" ]; then
    echo building for target $TARGET
    build_target $TARGET
else
    echo building for all targets
    build_target x86_64-pc-windows-msvc
    build_target aarch64-pc-windows-msvc
fi
