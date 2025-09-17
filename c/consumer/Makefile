# Common Makefile for CMake-based project

.PHONY: all build clean run

# Default target
all: build

# Configure and build the project
build:
	cmake -S . -B build
	cmake --build build

# Remove build artifacts
clean:
	rm -rf build

# Run the built executable (assumes output is in build/)
run: build
	./build/pact-consumer
