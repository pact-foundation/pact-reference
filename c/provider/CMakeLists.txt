
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
    pact-provider
    DESCRIPTION "PACT Provider in C"
    LANGUAGES C
)

################################################################################
# Dependencies
################################################################################

# cJSON is used for JSON parsing and generation
find_package(cJSON REQUIRED)

# libmicrohttpd is used to create a simple HTTP server for the provider
find_package(libmicrohttpd REQUIRED)

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
    corrosion_experimental_cbindgen(
        TARGET pact_ffi
        HEADER_NAME pact.h
    )
endif()

target_link_libraries(
    pact_ffi
    INTERFACE
    Threads::Threads
    LibLZMA::LibLZMA
    BZip2::BZip2
    ZLIB::ZLIB
)

# Link libintl on macOS (provided by Homebrew gettext)
# CMake's FindIntl module looks for gettext/libintl
if(APPLE)
    find_package(Intl)
    if(Intl_FOUND)
        target_link_libraries(
            pact_ffi
            INTERFACE
            Intl::Intl
        )
    endif()
endif()

################################################################################
# Targets
################################################################################
file(GLOB_RECURSE SRC_FILES src/*.c)
add_executable(pact-provider ${SRC_FILES})

# Set C standard properties
set_property(TARGET pact-provider PROPERTY C_STANDARD 17)
set_property(TARGET pact-provider PROPERTY C_STANDARD_REQUIRED ON)

target_include_directories(pact-provider PRIVATE ${CMAKE_SOURCE_DIR}/include)

target_link_libraries(
    pact-provider
    PRIVATE
    pact_ffi
    cjson::cjson
    libmicrohttpd::libmicrohttpd
)

# Link Windows system libraries if building on Windows
# These are needed by the sysinfo crate used by pact_ffi
if(WIN32)
    target_link_libraries(
        pact-provider
        PRIVATE
        iphlpapi   # IP Helper API (GetIfTable2, FreeMibTable, GetAdaptersAddresses, GetIfEntry2)
        pdh        # Performance Data Helper (Pdh* functions)
        netapi32   # Network Management API (NetUser*, NetGroup* functions)
        powrprof   # Power Management (CallNtPowerInformation)
        psapi      # Process Status API (GetProcessMemoryInfo, GetModuleFileNameExW)
        ws2_32     # Winsock 2 (socket functions)
    )
endif()

# The run recipe loads the library from beside the executable, where
# Corrosion would have built it, so copy the prebuilt one there.
if(DEFINED ENV{PACT_FFI_PREBUILT_DIR})
    add_custom_command(TARGET pact-provider POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:pact_ffi> $<TARGET_FILE_DIR:pact-provider>)
endif()

# Link macOS system frameworks if building on Apple platforms
if(APPLE)
    target_link_libraries(
        pact-provider
        PRIVATE
        "$<LINK_LIBRARY:FRAMEWORK,SystemConfiguration>"
        "$<LINK_LIBRARY:FRAMEWORK,Security>"
        "$<LINK_LIBRARY:FRAMEWORK,CoreFoundation>"
        "$<LINK_LIBRARY:FRAMEWORK,IOKit>"
    )
endif()
