#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_NAME=libpact_ffi

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$RUST_DIR/target"}

# We target the oldest supported version of macOS.
export MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-12}

# All flags passed to this script are passed to cargo.
case $1 in
x86_64-apple-darwin)
    TARGET=$1
    shift
    ;;
aarch64-apple-darwin)
    TARGET=$1
    shift
    ;;
*) ;;
esac
cargo_flags=("$@")

build_target() {
    TARGET=$1

    case $TARGET in
    x86_64-apple-darwin)
        ARCH_SUFFIX=x86_64
        ;;
    aarch64-apple-darwin)
        ARCH_SUFFIX=aarch64
        ;;
    *)
        echo unknown target $TARGET
        exit 1
        ;;
    esac
    cargo build --target $TARGET "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        file "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a"
        file "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.dylib"
        du -sh "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a"
        du -sh "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.dylib"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a" \
            "$ARTIFACTS_DIR/$APP_NAME-macos-$ARCH_SUFFIX.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.dylib" \
            "$ARTIFACTS_DIR/$APP_NAME-macos-$ARCH_SUFFIX.dylib.gz"
    fi
}

if [ ! -z "$TARGET" ]; then
    echo building for target $TARGET
    build_target $TARGET
else
    echo building for all targets
    build_target x86_64-apple-darwin
    build_target aarch64-apple-darwin
fi
