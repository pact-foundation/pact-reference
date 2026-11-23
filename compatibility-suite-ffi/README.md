# FFI compatibility suite

Runs the [Pact compatibility suite](https://github.com/pact-foundation/pact-compatibility-suite) through the
`pact_ffi` C API instead of the Rust crates. It uses the same copy of the suite as `../compatibility-suite`
(`../compatibility-suite/pact-compatibility-suite`), so the two runners check the same features.

The bindings are generated from `pact.h` at build time and the library is loaded at run time, so one build of the
runner can test any `libpact_ffi`.

## Running

Build and install pact_ffi first (this also generates the header, which needs the nightly toolchain for cbindgen):

```bash
.github/scripts/ffi-examples.sh
```

Then, from this directory:

```bash
cargo test --no-fail-fast
```

| Variable          | Default                                                                         |
|-------------------|---------------------------------------------------------------------------------|
| `PACT_FFI_HEADER` | `../rust/pact_ffi/build/install/include/pact.h` (read at build time)            |
| `PACT_FFI_LIB`    | `../rust/pact_ffi/build/install/lib/<libpact_ffi.so, .dylib or pact_ffi.dll>`  |
| `FFI_TRACE_DIR`   | `target/ffi-trace`: each scenario's FFI calls, one file per scenario            |
| `PACT_COMPAT_SUITE` | `../compatibility-suite/pact-compatibility-suite`: the suite checkout `plugins` reads features and fixtures from |

## Tests

| Test         | Features                                                                         |
|--------------|----------------------------------------------------------------------------------|
| `consumer`   | V1–V4 `http_consumer`                                                             |
| `provider`   | V1–V4 `*provider` (HTTP and message providers)                                    |
| `matching`   | V3 `http_matching` and `matching_rules`, V4 `matching_rules`                      |
| `generators` | V3 `generators` and `http_generators`, V4 `generators`                            |
| `message`    | V3 `message_consumer`, V4 `message_consumer`, `synchronous_message_consumer`, `v4` |
| `plugins`    | V4 `plugins/*`: lifecycle, HTTP, message, synchronous message and transports (needs the matt, csv, avro and protobuf plugins installed) |

Scenarios tagged `wip` are skipped. `tests/matching.rs` also skips one known failure: the mock server reads a
0-byte body as no body, so the V4 "not empty matcher with binary data (negative case)" scenario can't get its
mismatch.
