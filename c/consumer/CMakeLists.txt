
################################################################################
# Preamble
################################################################################

# Minimum CMake version
cmake_minimum_required(VERSION 3.24)

# We use [corrosion](https://github.com/corrosion-rs/corrosion) to integrate
# Rust into our C project.
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5
)
FetchContent_MakeAvailable(Corrosion)

get_filename_component(PACT_FFI_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../rust/pact_ffi" ABSOLUTE)

################################################################################
# Project definition
################################################################################
project(
    pact-consumer
    DESCRIPTION "PACT Consumer in C"
    LANGUAGES C
)

################################################################################
# Dependencies
################################################################################

# cURL is used to make requests.
#
# Note that in a real project, the Pact consumer tests should use the actual
# client being tested in order to validate that it behaves as expected; as
# opposed to testing that cURL behaves as expected.
find_package(CURL REQUIRED)

# Additional dependencies of the Pact FFI
find_package(PkgConfig REQUIRED)
find_package(Threads REQUIRED)
find_package(LibLZMA REQUIRED)
find_package(BZip2 REQUIRED)

# The Pact FFI is a Rust project
corrosion_import_crate(
    MANIFEST_PATH "${PACT_FFI_ROOT}/Cargo.toml"
)
corrosion_experimental_cbindgen(
    TARGET pact_ffi
    HEADER_NAME pact.h
)

################################################################################
# Targets
################################################################################
file(GLOB_RECURSE SRC_FILES src/*.c)
add_executable(pact-consumer ${SRC_FILES})

# Set C standard properties
set_property(TARGET pact-consumer PROPERTY C_STANDARD 23)
set_property(TARGET pact-consumer PROPERTY C_STANDARD_REQUIRED ON)

target_include_directories(pact-consumer PRIVATE ${CMAKE_SOURCE_DIR}/include)

target_link_libraries(
    pact-consumer
    PRIVATE
    pact_ffi
    CURL::libcurl
    lzma
    bz2
)

# Link macOS system frameworks if building on Apple platforms
if(APPLE)
    target_link_libraries(
        pact-consumer
        PRIVATE
        "$<LINK_LIBRARY:FRAMEWORK,Security>"
        "$<LINK_LIBRARY:FRAMEWORK,CoreFoundation>"
        "$<LINK_LIBRARY:FRAMEWORK,IOKit>"
    )
endif()
