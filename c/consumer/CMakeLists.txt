
################################################################################
# Preamble
################################################################################

# Minimum CMake version
cmake_minimum_required(VERSION 3.24)

# Include Conan toolchain if it exists
if(EXISTS "${CMAKE_CURRENT_BINARY_DIR}/conan_toolchain.cmake")
    include("${CMAKE_CURRENT_BINARY_DIR}/conan_toolchain.cmake")
endif()

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
find_package(Threads REQUIRED)
find_package(LibLZMA REQUIRED)
find_package(BZip2 REQUIRED)
find_package(ZLIB REQUIRED)

# The Pact FFI is a Rust project
#
# We only import the cdylib crate type here (rather than the default, which
# would prefer the staticlib). pact_ffi pulls in native static libraries via
# dependencies' build scripts (e.g. mlua's vendored Lua), which cargo bakes
# into the cdylib at link time but does not surface to corrosion for the
# staticlib, causing missing symbol errors when linking against libpact_ffi.a.
# PACT_FFI_PREBUILT_DIR names an install prefix holding lib/ and
# include/pact.h. CI sets it to reuse one release build across projects;
# without it, Corrosion builds pact_ffi from source.
if(DEFINED ENV{PACT_FFI_PREBUILT_DIR})
    file(TO_CMAKE_PATH "$ENV{PACT_FFI_PREBUILT_DIR}" PACT_FFI_PREBUILT_DIR)
    add_library(pact_ffi SHARED IMPORTED)
    if(WIN32)
        set_target_properties(pact_ffi PROPERTIES
            IMPORTED_LOCATION "${PACT_FFI_PREBUILT_DIR}/lib/pact_ffi.dll"
            IMPORTED_IMPLIB "${PACT_FFI_PREBUILT_DIR}/lib/pact_ffi.dll.lib")
    elseif(APPLE)
        set_target_properties(pact_ffi PROPERTIES
            IMPORTED_LOCATION "${PACT_FFI_PREBUILT_DIR}/lib/libpact_ffi.dylib")
    else()
        set_target_properties(pact_ffi PROPERTIES
            IMPORTED_LOCATION "${PACT_FFI_PREBUILT_DIR}/lib/libpact_ffi.so")
    endif()
    target_include_directories(pact_ffi INTERFACE "${PACT_FFI_PREBUILT_DIR}/include")
else()
    # We use [corrosion](https://github.com/corrosion-rs/corrosion) to
    # integrate Rust into our C project.
    include(FetchContent)
    FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.5
    )
    FetchContent_MakeAvailable(Corrosion)

    corrosion_import_crate(
        MANIFEST_PATH "${PACT_FFI_ROOT}/Cargo.toml"
        CRATE_TYPES cdylib
    )

    # cbindgen's macro expansion needs nightly; the library itself builds
    # with the toolchain pinned in rust-toolchain.toml.
    find_program(CBINDGEN_EXECUTABLE cbindgen REQUIRED)
    set(PACT_FFI_INCLUDE_DIR "${CMAKE_CURRENT_BINARY_DIR}/pact_ffi/include")
    file(MAKE_DIRECTORY "${PACT_FFI_INCLUDE_DIR}")
    add_custom_command(
        OUTPUT "${PACT_FFI_INCLUDE_DIR}/pact.h"
        COMMAND rustup run nightly "${CBINDGEN_EXECUTABLE}"
            --config "${PACT_FFI_ROOT}/cbindgen.toml"
            --crate pact_ffi
            --output "${PACT_FFI_INCLUDE_DIR}/pact.h"
        WORKING_DIRECTORY "${PACT_FFI_ROOT}"
        COMMENT "Generating pact.h with cbindgen"
    )
    add_custom_target(pact_ffi_header DEPENDS "${PACT_FFI_INCLUDE_DIR}/pact.h")
    add_dependencies(pact_ffi pact_ffi_header)
    target_include_directories(pact_ffi INTERFACE "${PACT_FFI_INCLUDE_DIR}")
endif()

target_link_libraries(
    pact_ffi
    INTERFACE
    Threads::Threads
    LibLZMA::LibLZMA
    BZip2::BZip2
    ZLIB::ZLIB
)

################################################################################
# Targets
################################################################################
file(GLOB_RECURSE SRC_FILES src/*.c)
add_executable(pact-consumer ${SRC_FILES})

# Set C standard properties
set_property(TARGET pact-consumer PROPERTY C_STANDARD 17)
set_property(TARGET pact-consumer PROPERTY C_STANDARD_REQUIRED ON)

target_include_directories(pact-consumer PRIVATE ${CMAKE_SOURCE_DIR}/include)

target_link_libraries(
    pact-consumer
    PRIVATE
    pact_ffi
    CURL::libcurl
)

# Link Windows system libraries if building on Windows
# These are needed by the sysinfo crate used by pact_ffi
if(WIN32)
    target_link_libraries(
        pact-consumer
        PRIVATE
        pdh        # Performance Data Helper (Pdh* functions)
        netapi32   # Network Management API (NetUser*, NetGroup* functions)
        powrprof   # Power Management (CallNtPowerInformation)
        psapi      # Process Status API (GetProcessMemoryInfo, GetModuleFileNameExW)
    )
endif()

# The run recipe loads the library from beside the executable, where
# Corrosion would have built it, so copy the prebuilt one there.
if(DEFINED ENV{PACT_FFI_PREBUILT_DIR})
    add_custom_command(TARGET pact-consumer POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:pact_ffi> $<TARGET_FILE_DIR:pact-consumer>)
endif()

# Link macOS system frameworks if building on Apple platforms
if(APPLE)
    target_link_libraries(
        pact-consumer
        PRIVATE
        "$<LINK_LIBRARY:FRAMEWORK,SystemConfiguration>"
        "$<LINK_LIBRARY:FRAMEWORK,Security>"
        "$<LINK_LIBRARY:FRAMEWORK,CoreFoundation>"
        "$<LINK_LIBRARY:FRAMEWORK,IOKit>"
    )
endif()
