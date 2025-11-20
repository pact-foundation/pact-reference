
# Pact C Consumer Example (`c/consumer`)

This directory serves two purposes:

1. **Example for C Consumers:** Demonstrates how to use the Pact FFI (Foreign Function Interface) in C to verify interactions with a provider.
2. **Reference for FFI Maintainers:** Provides a working example for maintainers of libraries in other languages that use the Pact FFI internally, showing how to integrate and test against the FFI.

It includes utilities, example source code, and contract files for both use cases.

## Directory Structure

- `include/`: Header files for the consumer code
- `src/`: Source files for the consumer implementation:
  - `main.c`: Main entry point
  - `curl_utils.c`, `logging.c`: Utility implementations
  - `pact/`: Various scenarios. This includes running a few interactions against a mock server, as well as how to setup logging.
- `CMakeLists.txt`: CMake build configuration
- `justfile`: Justfile for building and running the consumer tests
- `conanfile.txt`: Conan configuration for dependencies

## Building and Running

This project uses Conan for dependency management and CMake for configuration and building. It also provides a justfile for convenience:

```console
just run
```

The results of the tests are printed to stdout and will exit with a non-zero status if any tests fail. On success, the generated Pact contract files will be available in the `pacts/` directory.

## Prerequisites

The tests require:

- A C and Rust compiler (Rust compiler requires nightly toolchain)
- CMake (version 3.24 or higher)
- Conan (version 2.0 or higher)

The CMake configuration will automatically build the Pact FFI library from the Rust source, hence the need for the Rust toolchain. In practice, you would typically download the pre-built binaries for your platform from the [Pact FFI releases](https://github.com/pact-foundation/pact-reference/releases), and link against those instead.

All other dependencies (curl, liblzma, bzip2, openssl, zlib) will be automatically downloaded and/or built by Conan as needed.

### Installing Conan

If you don't have Conan installed, you can install it via `uv`:

```console
uv tool install conan
```

For more installation options, see the [Conan installation documentation](https://docs.conan.io/2/installation.html).
