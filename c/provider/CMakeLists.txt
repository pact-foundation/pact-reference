
################################################################################
# Preamble
################################################################################

# Minimum CMake version
cmake_minimum_required(VERSION 3.24)

# Include Conan toolchain if it exists
if(EXISTS "${CMAKE_CURRENT_BINARY_DIR}/conan_toolchain.cmake")
    include("${CMAKE_CURRENT_BINARY_DIR}/conan_toolchain.cmake")
endif()

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
corrosion_import_crate(
    MANIFEST_PATH "${PACT_FFI_ROOT}/Cargo.toml"
)
corrosion_experimental_cbindgen(
    TARGET pact_ffi
    HEADER_NAME pact.h
)

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

# Link macOS system frameworks if building on Apple platforms
if(APPLE)
    target_link_libraries(
        pact-provider
        PRIVATE
        "$<LINK_LIBRARY:FRAMEWORK,Security>"
        "$<LINK_LIBRARY:FRAMEWORK,CoreFoundation>"
        "$<LINK_LIBRARY:FRAMEWORK,IOKit>"
    )
endif()
