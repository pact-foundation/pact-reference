#!/bin/bash

set -e

echo -Setup directories --
cargo clean
mkdir -p ../target/artifacts

echo -Install latest version of cross --
cargo install cross --git https://github.com/cross-rs/cross

echo -Install latest version of cross --
linux_targets=(
  x86_64-unknown-linux-gnu
  x86_64-unknown-linux-musl
  aarch64-unknown-linux-gnu
  aarch64-unknown-linux-musl
  arm-unknown-linux-gnueabi
  arm-unknown-linux-gnueabihf
  arm-unknown-linux-musleabi
  arm-unknown-linux-musleabihf
  armv7-unknown-linux-gnueabi
  armv7-unknown-linux-gnueabihf
  armv7-unknown-linux-musleabi
  armv7-unknown-linux-musleabihf
  i586-unknown-linux-gnu
  i686-unknown-linux-gnu
  x86_64-unknown-netbsd
  x86_64-unknown-freebsd
  armv5te-unknown-linux-gnueabi
  armv5te-unknown-linux-musleabi
  arm-linux-androideabi
  armv7-linux-androideabi
  aarch64-linux-android
  i686-linux-android
  x86_64-linux-android
  thumbv7neon-linux-androideabi
  thumbv7neon-unknown-linux-gnueabihf
)

macos_targets=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  aarch64-apple-ios
  aarch64-apple-ios-sim
  x86_64-apple-ios
)
windows_targets=(
  x86_64-pc-windows-msvc
  aarch64-pc-windows-msvc
  i686-pc-windows-msvc
  x86_64-pc-windows-gnu
)

echo -Setup targets --
if [[ ! -n $target ]]; then
  # only build for specific targets on particular os's
  # limited list, due to github actions not supporting
  # docker on macos or windows, so unable to use cross
  # this list is probably different if running locally
  # list taken from .github/workflows/x-plat.yml
  case "$(uname -s)" in
  Darwin)
    targets=("${macos_targets[@]}")
    ;;
  Linux)
    targets=("${linux_targets[@]}")
    ;;
  CYGWIN* | MINGW32* | MSYS* | MINGW*)
    targets=("${windows_targets[@]}")
    ;;
  *)
    echo "ERROR: $(uname -s) is not a supported operating system"
    exit 1
    ;;
  esac
  echo "building for following targets:"
  for target in "${targets[@]}"; do
    echo "${target}"
  done
else
  targets=($target)
fi

for target in "${targets[@]}"; do

  if [[ $target == "aarch64-pc-windows-msvc" ]]; then
    ## The following is a workaround until ring 16 supports windows arm64 or rustls moves to ring 17 (post release)
    ## It also relies on ../cargo.toml having the [patch.crates-io] section at the bottom of the file
    # https://github.com/briansmith/ring/issues/1514#issuecomment-1258562375
    # https://github.com/briansmith/ring/pull/1554
    # https://github.com/rust-lang/rustup/issues/2612#issuecomment-1433876793
    # https://github.com/rustls/rustls/pull/1108
    echo "ring override for aarch64-pc-windows-msvc"
    echo ring = { git = \"https://github.com/awakecoding/ring\", branch = \"0.16.20_alpha\" } >>../cargo.toml
    cd .. && cargo update
    cd $crate
  fi

  if [[ $target == *"musl"* && $crate == 'pact_ffi' ]]; then
    echo "building for musl $target"
    # https://github.com/rust-lang/cargo/issues/7154
    RUSTFLAGS="-C target-feature=-crt-static" cross build --target "${target}" --release
  elif [[ ($CI == "true" && $(uname -s) != "Linux") || $CIRRUS_CI = "true" ]]; then
    # no docker on github actions macos / windows
    # no docker in docker in cirrus
    echo "building for $target with cargo"
    cargo build --target "${target}" --release
  else
    echo "building for $target with cross"
    cross build --target "${target}" --release
  fi

  if [[ $crate == 'pact_ffi' ]];then
    

    if [[ $target == *"windows"* ]]; then
      lib_ext=dll
      lib_name=pact_ffi
    elif [[ $target == *"darwin"* ]]; then
      lib_ext=dylib
      lib_name=libpact_ffi
    elif [[ $target == *"ios"* ]]; then
      lib_ext=a
      lib_name=libpact_ffi
    else
      lib_ext=so
      lib_name=libpact_ffi
    fi

    echo "showing cargo release build for lib${lib_name}.${lib_ext} and checksum for target ${target}"
    ls ../target/${target}/release

    if [[ $target == *"windows"* ]]; then
      if [[ $target != *"gnu"* ]]; then
        ## dll.lib
        lib_ext=dll.lib
        gzip -c ../target/${target}/release/${lib_name}.${lib_ext} >../target/artifacts/${lib_name}-${target}.${lib_ext}.gz
        openssl dgst -sha256 -r ../target/artifacts/${lib_name}-${target}.${lib_ext}.gz >../target/artifacts/${lib_name}-${target}.${lib_ext}.sha256
        ## lib
        lib_ext=lib
        gzip -c ../target/${target}/release/${lib_name}.${lib_ext} >../target/artifacts/${lib_name}-${target}.${lib_ext}.gz
        openssl dgst -sha256 -r ../target/artifacts/${lib_name}-${target}.${lib_ext}.gz >../target/artifacts/${lib_name}-${target}.${lib_ext}.sha256
      fi
    fi

    ## cdylib - shared lib .so / .dll / .dylib / .a depending on platform
    echo "preparing shared lib${lib_name}.${lib_ext} and checksum for target ${target}"
    gzip -c ../target/${target}/release/${lib_name}.${lib_ext} >../target/artifacts/${lib_name}-${target}.${lib_ext}.gz
    openssl dgst -sha256 -r ../target/artifacts/${lib_name}-${target}.${lib_ext}.gz >../target/artifacts/${lib_name}-${target}.${lib_ext}.sha256

    ## static lib .a
    ## only prepare the static lib if it exists
    lib_ext=a
    if [[ -f ../target/${target}/release/${lib_name}.${lib_ext} ]]; then
      echo "preparing static lib${lib_name}.${lib_ext} and checksum for target ${target}"
      gzip -c ../target/${target}/release/${lib_name}.${lib_ext} >../target/artifacts/${lib_name}-${target}.${lib_ext}.gz
      openssl dgst -sha256 -r ../target/artifacts/${lib_name}-${target}.${lib_ext}.gz >../target/artifacts/${lib_name}-${target}.${lib_ext}.sha256
    fi

  else if [[ $crate == 'pact_verifier_cli' || $crate == 'pact_mock_server_cli' ]]; then
      if [[ $target == *"windows"* ]]; then
        ext=.exe
      fi
      gzip -c target/$target/release/$crate$ext > target/artifacts/$crate-$target$ext.gz
      openssl dgst -sha256 -r target/artifacts/$crate-$target.exe.gz > target/artifacts/$crate-$target$ext.gz.sha256
  fi
done
echo "showing final release artefacts"
ls ../target/artifacts
