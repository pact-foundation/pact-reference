#!/usr/bin/env bash
# Checks dist-cargo.sh against a fake cargo that records its arguments and
# CARGO_ENCODED_RUSTFLAGS.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script="$here/../dist-cargo.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir "$work/bin"
cat > "$work/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
echo "args=$*"
echo "flags=${CARGO_ENCODED_RUSTFLAGS-unset}"
CARGO
chmod +x "$work/bin/cargo"
export PATH="$work/bin:$PATH"
unset CARGO_ENCODED_RUSTFLAGS

run() { "$script" "$@"; }
check() {
    local label=$1 want_args=$2 want_flags=$3; shift 3
    local out
    out=$(run "$@")
    [[ $out == *"args=$want_args"* ]] || { echo "FAIL $label args: <$out>"; exit 1; }
    [[ $out == *"flags=$want_flags"* ]] || { echo "FAIL $label flags: <$out>"; exit 1; }
}

ffi_spec='path+file:///w/rust/pact_ffi#0.5.12'
cli_spec='path+file:///w/rust/pact_verifier_cli#pact_verifier_cli@1.4.0'

check "pact_ffi on musl links the CRT dynamically" \
    "build --profile dist --target x86_64-unknown-linux-musl --package $ffi_spec" "-Ctarget-feature=-crt-static" \
    build --profile dist --target x86_64-unknown-linux-musl --package "$ffi_spec"
check "the CLI on musl stays static" \
    "build --profile dist --target x86_64-unknown-linux-musl --package $cli_spec" "unset" \
    build --profile dist --target x86_64-unknown-linux-musl --package "$cli_spec"
check "zigbuild becomes a plain build" \
    "build --profile dist --target aarch64-unknown-linux-musl --package pact_ffi" "-Ctarget-feature=-crt-static" \
    zigbuild --profile dist --target aarch64-unknown-linux-musl --package pact_ffi
check "pact_ffi on gnu is untouched" \
    "build --target x86_64-unknown-linux-gnu --package pact_ffi" "unset" \
    build --target x86_64-unknown-linux-gnu --package pact_ffi
check "other commands pass through" "-vV" "unset" -vV
check "pact_ffi_x is not pact_ffi" \
    "build --target x86_64-unknown-linux-musl --package pact_ffi_x" "unset" \
    build --target x86_64-unknown-linux-musl --package pact_ffi_x
echo PASS
