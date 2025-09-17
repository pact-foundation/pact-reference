
# Pact C Consumer Example (`c/consumer`)

This directory serves two purposes:

1. **Example for C Consumers:** Demonstrates how to use the Pact FFI (Foreign Function Interface) in C to verify interactions with a provider.
2. **Reference for FFI Maintainers:** Provides a working example for maintainers of libraries in other languages that use the Pact FFI internally, showing how to integrate and test against the FFI.

It includes utilities, example source code, and contract files for both use cases.

## Directory Structure

- `include/`: Header files for the consumer code, including:
- `src/`: Source files for the consumer implementation:
  - `main.c`: Main entry point
  - `curl_utils.c`, `logging.c`: Utility implementations
  - `pact/`: Various scenarios. This includes running a few interactions against a mock server, as well as how to setup logging.
- `CMakeLists.txt` — CMake build configuration
- `Makefile` — Makefile for building and running

## Building and Running

This project uses CMake for configuration and building, and also provides a Makefile for convenience:

```console
make run
# or equivalently:
cmake -S . -B build
cmake --build build
./build/pact-consumer
```

The results of the tests are printed to stdout and will exit with a non-zero status if any tests fail.

## Prerequisites

The tests require a C and Rust compiler, CMake, and the following development libraries: `curl`, `lzma`, and `bzip2`.

The CMake configuration will automatically build the Pact FFI library from the Rust source, hence the need for the Rust toolchain. In practice, you would typically download the pre-built binaries for your platform from the [Pact FFI releases](https://github.com/pact-foundation/pact-reference/releases), and link against those instead.
