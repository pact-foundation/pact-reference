![Logo of the project](https://raw.githubusercontent.com/pact-foundation/pact-reference/master/images/logo.svg)

[![Pact-Rust Build](https://github.com/pact-foundation/pact-reference/workflows/Pact-Rust%20Build/badge.svg)](https://github.com/pact-foundation/pact-reference/actions?query=workflow%3A%22Pact-Rust+Build%22)
[![Windows Build status](https://ci.appveyor.com/api/projects/status/bqlb7ny924lsu6yi?svg=true)](https://ci.appveyor.com/project/pact-foundation/pact-reference)

# Pact-Rust
> Pact implementation written in Rust.

This project provides a Pact implementation written in Rust. It supports the [V3 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3)
and [V4 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4) and provides shared 
libraries that can be used in other languages via foreign function interfaces (FFI).

Note: The mock server crates have moved to https://github.com/pact-foundation/pact-core-mock-server.

## Installing / Getting started

### Rust Projects

The project crates are published to crates.io, and you can use them directly. 

#### Consumer Projects

For a Rust consumer, refer to the [pact_consumer](./pact_consumer) crate.

You can find examples where the [integration to the Pact Broker](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_verifier/tests/tests.rs) is tested using 
Pact tests written with the `pact_consumer` crate.

#### Verifying Providers

To verify a provider, you can use the verifier CLI tool. See [Pact Verifier CLI](./pact_verifier_cli).

### FFI Interface

Other languages can use the core features through FFI shared libraries. These libraries expose functions that
follow a C calling convention.

#### Consumer Projects

For example for a project using Conan and CMake that links to the `pact-ffi` crate, see [Pact-C++](https://github.com/pact-foundation/pact-cplusplus).

#### Verifying Providers

> TODO after merging FFI libs

## Developing

There are 8 main crates to this implementation:

### [pact_models](./pact_models)

This is a library that provides the Pact models and functions for reading and writing pact files.

### [pact_matching](./pact_matching)

This is a library that provides the Pact functions for matching requests and responses.

### [pact_ffi](./pact_ffi)

This is a library that implements exported functions using C bindings for use in foreign languages.

### [pact_mock_server](https://github.com/pact-foundation/pact-core-mock-server/pact_mock_server)

Moved to https://github.com/pact-foundation/pact-core-mock-server.

### [pact_mock_server_cli](https://github.com/pact-foundation/pact-core-mock-server/pact_mock_server_cli)

Moved to https://github.com/pact-foundation/pact-core-mock-server.

### [pact_consumer](./pact_consumer)

This is a library that provides the Pact consumer test support and DSL for use in Rust projects.

### [pact_verifier](./pact_verifier)

This library provides support for verifying a provider against pact files.

### [pact_verifier_cli](./pact_verifier_cli)

Command line excutable that uses the [pact_verifier](./pact_verifier) to be able to verify a running provider against
pact files.

### Building

To build the libraries in this project, you need a working Rust environment. Refer to the [Rust Guide](https://www.rust-lang.org/learn/get-started). Requires
Rust 2024 edition (1.88.0 or later).

The build tool used is `cargo`.

```shell
cd rust
cargo build
```

This will compile all the libraries and put the generated files in `rust/target/debug`.

### Releasing

The released libraries for each module are built by a GH action that attaches the libraries to the GH release for each
crate. To release a crate, run the `release.groovy` script in the crate directory. This will guide you through the
release process for the crate. Then create a GH release using the tag and changelog created by the script.

## Contributing

See [CONTRIBUTING](../CONTRIBUTING.md) (PRs are always welcome!).

## Documentation

Rust crate documentation is published to the Rust documentation site.

* [pact_consumer](https://docs.rs/pact_consumer/)
* [pact_matching](https://docs.rs/pact_matching/)
* [pact_ffi](https://docs.rs/pact_ffi/)
* [pact_models](https://docs.rs/pact_models/)
* [pact_verifier](https://docs.rs/pact_verifier/)
* [pact_verifier_cli](https://docs.rs/pact_verifier_cli/)

Additional documentation can be found at the main [Pact website](https://pact.io).

## Contact

Join us in slack: [![slack](https://slack.pact.io/badge.svg)](https://slack.pact.io)

or

- Twitter: [@pact_up](https://twitter.com/pact_up)
- Stack Overflow: [stackoverflow.com/questions/tagged/pact](https://stackoverflow.com/questions/tagged/pact)

## Licensing

The code in this project is licensed under a MIT license. See [LICENSE](../LICENSE).
