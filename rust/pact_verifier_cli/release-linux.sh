#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME=pact-verifier

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"

CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$RUST_DIR/target"}

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
    echo build x86_64-unknown-linux-musl target for musl/glibc compat bins
    exit 0
    ;;
aarch64-unknown-linux-gnu)
    echo build aarch64-unknown-linux-musl target for musl/glibc compat bins
    exit 0
    ;;
*) ;;
esac
cargo_flags=("$@")

clean_cargo_release_build() {
    rm -rf $CARGO_TARGET_DIR/release/build
}

build_target() {
    TARGET=$1
    cross build --target $TARGET "${cargo_flags[@]}"

    case $TARGET in
    x86_64-unknown-linux-musl)
        FILE_SUFFIX=linux-x86_64
        ;;
    aarch64-unknown-linux-musl)
        FILE_SUFFIX=linux-aarch64
        ;;
    *)
        echo unknown target $TARGET
        exit 1
        ;;
    esac

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        file "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME"
        du -sh "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME"
        gzip_and_sum "$CARGO_TARGET_DIR/$TARGET/release/$APP_NAME" \
            "$ARTIFACTS_DIR/$APP_NAME-$FILE_SUFFIX.gz"
    fi
}

install_cross() {
    cargo install cross@0.2.5 --force
}
install_cross_latest() {
    cargo install cross --git https://github.com/cross-rs/cross --force
}

install_cross
if [ "$(uname -s)" == "Darwin" ]; then
    install_cross_latest
fi

if [ ! -z "$TARGET" ]; then
    echo building for target $TARGET
    build_target $TARGET
else
    echo building for all targets
    clean_cargo_release_build
    build_target x86_64-unknown-linux-musl
    clean_cargo_release_build
    build_target aarch64-unknown-linux-musl
fi
