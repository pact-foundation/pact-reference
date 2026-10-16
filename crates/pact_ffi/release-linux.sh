#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_NAME=libpact_ffi

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
install_cross() {
    cargo install cross@0.2.5 --force
}
install_cross_latest() {
    cargo install cross --git https://github.com/cross-rs/cross --force
}
clean_cargo_release_build() {
    rm -rf $CARGO_TARGET_DIR/release/build
}

export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$RUST_DIR/target"}

# All flags passed to this script are passed to cargo.
case $1 in
x86_64-unknown-linux-musl)
    TARGET=$1
    shift
    ;;
aarch64-unknown-linux-musl)
    TARGET=$1
    shift
    ;;
x86_64-unknown-linux-gnu)
    TARGET=$1
    shift
    ;;
aarch64-unknown-linux-gnu)
    TARGET=$1
    shift
    ;;
*) ;;
esac
cargo_flags=("$@")

build_target() {
    TARGET=$1

    case $TARGET in
    x86_64-unknown-linux-musl)
        FILE_SUFFIX=linux-x86_64-musl
        RUSTFLAGS="-C target-feature=-crt-static"
        ;;
    aarch64-unknown-linux-musl)
        FILE_SUFFIX=linux-aarch64-musl
        RUSTFLAGS="-C target-feature=-crt-static"
        ;;
    x86_64-unknown-linux-gnu)
        FILE_SUFFIX=linux-x86_64
        ;;
    aarch64-unknown-linux-gnu)
        FILE_SUFFIX=linux-aarch64
        ;;
    *)
        echo unknown target $TARGET
        exit 1
        ;;
    esac
    RUSTFLAGS=${RUSTFLAGS:-""} cross build --target $TARGET "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        file "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a"
        file "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.so"
        du -sh "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a"
        du -sh "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.so"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.a" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME.so" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.so.gz"
    fi
}

build_header() {
    rustup toolchain install nightly
    rustup run nightly cbindgen \
        --config cbindgen.toml \
        --crate pact_ffi \
        --output "$ARTIFACTS_DIR/pact.h"
    rustup run nightly cbindgen \
        --config cbindgen-c++.toml \
        --crate pact_ffi \
        --output "$ARTIFACTS_DIR/pact-cpp.h"
}

install_cross
if [ ! -z "$TARGET" ]; then
    echo building for target $TARGET
    build_target $TARGET

    # If we are building indiv targets, ensure we build the headers
    # for at least 1 nominated target
    if [ "$TARGET" == "x86_64-unknown-linux-gnu" ]; then
        build_header
    fi
else
    echo building for all targets
    # clean release build to avoid conflicting symbols when building all targets 
    clean_cargo_release_build
    build_target x86_64-unknown-linux-gnu
    clean_cargo_release_build
    build_target aarch64-unknown-linux-gnu
    clean_cargo_release_build
    build_target x86_64-unknown-linux-musl
    clean_cargo_release_build
    build_target aarch64-unknown-linux-musl
    build_header
fi
