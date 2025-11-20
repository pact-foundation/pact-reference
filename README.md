![Logo of the project](https://raw.githubusercontent.com/pact-foundation/pact-reference/master/images/logo.svg)

[![Pact-Rust Build](https://github.com/pact-foundation/pact-reference/workflows/Pact-Rust%20Build/badge.svg)](https://github.com/pact-foundation/pact-reference/actions?query=workflow%3A%22Pact-Rust+Build%22)
[![Pact-Rust FFI Build](https://github.com/pact-foundation/pact-reference/actions/workflows/build-ffi.yml/badge.svg)](https://github.com/pact-foundation/pact-reference/actions/workflows/build-ffi.yml)

# Pact Reference Implementation

> Reference implementations for the Pact Specification written in Rust

This project contains a reference implementation of the [Pact specification](https://github.com/pact-foundation/pact-specification) written in Rust, often referred to as the Pact Core.

It exposes the core functionality through various crates, as part of the Pact Rust SDK, as well as through FFI (Foreign Function Interface) and CLI (Command Line Interface) for use in other programming languages.

## Usage

### Rust

For Rust projects, you can use the Rust crates from this library in your project directly. Refer to the [Rust project readme](rust/README.md).

### FFI

This project contains dynamic libraries that expose the core functionality through a C-compatible FFI (Foreign Function Interface).

This project contains a couple of examples which demonstrate how to use the FFI library in a C project, with a focus on showcasing how to call the FFI functions from C:

* [C - Consumer](c/consumer)
* [C - Provider](c/provider)

Most of the Pact SDKs in other languages make use of this FFI library to provide Pact functionality. You can see this in the [Ecosystem graph](https://docs.pact.io/diagrams/ecosystem#rust-ffi-consumers-goldberg-machine), and the following examples:

* [Javascript via pact-js-core](https://github.com/pact-foundation/pact-js-core)
* [Ruby via pact-ruby-ffi](https://github.com/YOU54F/pact-ruby-ffi)
* [PHP via pact-php](https://github.com/pact-foundation/pact-php)
* [GoLang via pact-go](https://github.com/pact-foundation/pact-go)
* [.NET via pact-net](https://github.com/pact-foundation/pact-net)
* [Swift via pact-swift](https://github.com/surpher/PactSwift)
* [Dart via pact-dart](https://github.com/matthewshirley/pact_dart)
* [C++ via pact-cplusplus](https://github.com/pact-foundation/pact-cplusplus)
* [Python via pact-python](https://github.com/pact-foundation/pact-python)

Lastly, you can also refer to [Hello Pact FFI](https://github.com/YOU54F/hello_ffi) for collection of examples in various languages.

### CLI

This project contains code to support exposing the core functionality through CLI (Command Line Interface).

* [pact_mock_server_cli](https://github.com/pact-foundation/pact-core-mock-server/tree/main/pact_mock_server_cli)
* [pact_verifier_cli](https://github.com/pact-foundation/pact-reference/tree/master/rust/pact_verifier_cli)
* [pact-stub-server-cli](https://github.com/pact-foundation/pact-stub-server)

They are distributed in binary & Docker formats.

## Building

To build the libraries in this project, you need a working Rust environment.  Requires minimum Rust 1.59.0.  Refer to the [Rust Guide](https://www.rust-lang.org/learn/get-started).

The build tool used is `cargo`.

```shell
cd rust
cargo build
```

This will compile all the libraries and put the generated files in `rust/target/debug`.

## Contributing

See [CONTRIBUTING](CONTRIBUTING.md) (PRs are always welcome!).

## Documentation

Rust library documentation is published to the Rust documentation site [`docs.rs`](https://docs.rs). Refer to the [Rust project README](rust/README.md).

Additional documentation can be found at the main [Pact website](https://pact.io).

## Contact

Join us in slack: [![slack](https://slack.pact.io/badge.svg)](https://slack.pact.io)

or

* Twitter: [@pact_up](https://twitter.com/pact_up)
* Stack Overflow: [stackoverflow.com/questions/tagged/pact](https://stackoverflow.com/questions/tagged/pact)

## Licensing

The code in this project is licensed under a MIT license. See [LICENSE](LICENSE).
