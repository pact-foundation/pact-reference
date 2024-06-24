#################################################################################################
# CMAKE VERSION
#################################################################################################

# Set the minimum to 3.15. This is arbitrary and we should probably try to
# test everything with older CMake versions once this is all written, to
# figure out an actual lower-bound.
cmake_minimum_required(VERSION 3.15...3.17)

# Set policies appropriately, so it knows when to warn about policy
# violations.
if(${CMAKE_VERSION} VERSION_LESS 3.17)
    cmake_policy(VERSION ${CMAKE_MAJOR_VERSION}.${CMAKE_MINOR_VERSION})
else()
    cmake_policy(VERSION 3.17)
endif()

#################################################################################################
# CONFIG FILES
#
# Set the location of various config files we'll use throughout.
#################################################################################################

# The path to the cargo config file.
set(CARGO_CONFIG_FILE "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml")

# The path to the cbindgen config file.
set(CBINDGEN_CONFIG_FILE "${CMAKE_CURRENT_SOURCE_DIR}/cbindgen.toml")

#################################################################################################
# PROJECT NAME
#
# This is pulled from the `Cargo.toml` file.
#################################################################################################

# Regex for 'name = "<name>"'
set(NAME_REGEX "name = \"(.+)\"")

# Read in the line containing the name
file(STRINGS ${CARGO_CONFIG_FILE} NAME_STRING REGEX ${NAME_REGEX})

# Pick out just the name
string(REGEX REPLACE ${NAME_REGEX} "\\1" NAME_STRING "${NAME_STRING}")

#################################################################################################
# PROJECT VERSION
#
# This is pulled from the `Cargo.toml` file.
#################################################################################################

# Regex for 'version = "<version>"'
set(VERSION_REGEX "^version = \"(.+)\"$")

# Read in the line containing the version
file(STRINGS ${CARGO_CONFIG_FILE} VERSION_STRING REGEX ${VERSION_REGEX})

# Pick out just the version
string(REGEX REPLACE ${VERSION_REGEX} "\\1" VERSION_STRING "${VERSION_STRING}")

#################################################################################################
# PROJECT DESCRIPTION
#
# This is pulled from the `Cargo.toml` file.
#################################################################################################

# Regex for 'description = "<description>"'
set(DESCRIPTION_REGEX "description = \"(.+)\"")

# Read in the line containing the description
file(STRINGS ${CARGO_CONFIG_FILE} DESCRIPTION_STRING REGEX ${DESCRIPTION_REGEX})

# Pick out just the description
string(REGEX REPLACE ${DESCRIPTION_REGEX} "\\1" DESCRIPTION_STRING "${DESCRIPTION_STRING}")

#################################################################################################
# PROJECT DECLARATION
#################################################################################################

# Print message indicating we found the crate information.
message("Found crate ${NAME_STRING} (version ${VERSION_STRING}): ${DESCRIPTION_STRING}")

# Define the project for the current file.
project(
    ${NAME_STRING}
    VERSION ${VERSION_STRING}
    DESCRIPTION ${DESCRIPTION_STRING}
    LANGUAGES NONE)

#################################################################################################
# CMAKE UTILITIES
#
# Add CMake utilities for finding Cargo and Cbindgen to the module path.
#################################################################################################

set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})

#################################################################################################
# OUT OF SOURCE BUILDS
#
# Require out-of-source builds for this project. It keeps things much simpler
# and cleaner.
#################################################################################################

# Set a path to the CMake config (this file)
file(TO_CMAKE_PATH "${PROJECT_BINARY_DIR}/CMakeLists.txt" LOC_PATH)

# Define the error message to potentially be printed.
set(OOS_MSG "\
You cannot build in a source directory (or any directory with a CMakeLists.txt file). \
Please make a build subdirectory. \
Feel free to remove CMakeCache.txt and CMakeFiles.
")

# If that file path exists, we're doing an in-source build, so we should exit with a fatal
# error complaining only out-of-source builds are supported.
if(EXISTS ${LOC_PATH})
    message(FATAL_ERROR ${OOS_MSG})
endif()

#################################################################################################
# DEFAULT BUILD TYPE
#
# Make release the default build type
#################################################################################################

# The default build type is Release.
set(default_build_type "Release")

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    # Tell the user they're getting the default build type.
    message(STATUS "Setting build type to '${default_build_type}' as none was specified.")

    # Cache the build type.
    set(CMAKE_BUILD_TYPE ${default_build_type} CACHE STRING "Choose the type of build." FORCE)

    # Set the possible values of build type
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release")
endif()

#################################################################################################
# Find Cargo & Cbindgen
#
# Uses custom finders to locate the cargo and cbindgen executables and identify their versions
#################################################################################################

# Uses the finder specified in `cmake/FindCargo.cmake`
find_package(Cargo)

if(NOT CARGO_CHANNEL STREQUAL "nightly")
    message(FATAL_ERROR "${NAME_STRING} requires a nightly Cargo version to build.")
endif()

# Uses the finder specified in `cmake/FindCbindgen.cmake`
find_package(Cbindgen)

# CMake can find Doxygen without a custom finder module.
find_package(Doxygen)

#################################################################################################
# VARIABLES
#
# Sets important variables to be used by the custom targets.
#################################################################################################

# Set cargo build type flag, and name of the folder containing the library file,
# based on the configured build type.
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_BUILD_TYPE "")
    set(TARGET_TYPE "debug")
else()
    set(CARGO_BUILD_TYPE "--release")
    set(TARGET_TYPE "release")
endif()

# There could be something more generic here using `cargo metadata` output to discover
# the workspace root and get the target dir from that, and in a more general context
# that's what ought to be done, but for now we know where the target dir is, and should
# just reuse it.
set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../target")

# The name of the crate itself.
set(CRATE_NAME ${NAME_STRING})

if (${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
    set(MACOSX TRUE)
endif()

# Set the CARGO_LIBRARY_NAME based on whether we're on Windows, Linux or OSX
if(WIN32)
    set(CARGO_LIBRARY_NAME "${CRATE_NAME}.dll")
elseif(MACOSX)
    set(CARGO_LIBRARY_NAME "lib${CRATE_NAME}.dylib")
else()
    set(CARGO_LIBRARY_NAME "lib${CRATE_NAME}.so")
endif()

# The path to the library file.
set(CARGO_LIBRARY_FILE "${CARGO_TARGET_DIR}/${TARGET_TYPE}/${CARGO_LIBRARY_NAME}")

# On Windows, dynamic linking requires both a .dll file (which will be dynamically
# linked), and a library file (which describes _how_ to link to the DLL, and will
# itself be statically linked). We need to make sure both are included on installation.
#
# For Rust specifically, the name of the .lib file is the name of the .dll file with
# .lib appended (so the extension is .dll.lib).
if(WIN32)
    set(CARGO_INSTALL_FILES ${CARGO_LIBRARY_FILE} "${CARGO_LIBRARY_FILE}.lib")
else()
    set(CARGO_INSTALL_FILES ${CARGO_LIBRARY_FILE})
endif()

# Name of the header file.
set(CBINDGEN_HEADER_NAME "pact.h")

# Path to the header file.
set(CBINDGEN_HEADER_FILE "${CMAKE_CURRENT_SOURCE_DIR}/include/${CBINDGEN_HEADER_NAME}")

#################################################################################################
# LIBRARY
#
# Defines the target for building the library file.
#################################################################################################

# Defines the cargo command to build the library file.
add_custom_command(
    OUTPUT ${CARGO_LIBRARY_FILE}
    COMMAND
        ${CARGO_EXECUTABLE}
        --locked
        build
        ${CARGO_BUILD_TYPE}
        --target-dir ${CARGO_TARGET_DIR}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

# Add target for the library file
add_custom_target(build_library ALL
    COMMENT "Building library file with cargo"
    DEPENDS ${CARGO_LIBRARY_FILE})

# Teach CMake to install the library file(s) built by the pact_matching_ffi target
install(FILES ${CARGO_INSTALL_FILES} TYPE LIB)

#################################################################################################
# HEADER
#
# Defines the target for generating the header file.
#################################################################################################

# Define a fake file to ensure the header file is always rebuilt
set(CBINDGEN_HEADER_FILE_FAKE "${CMAKE_HEADER_FILE}.fake")

# Make sure the fake file doesn't actually exist, which would break this mechanism
if(EXISTS ${CBINDGEN_HEADER_FILE_FAKE})
  message(FATAL_ERROR "File \"${CBINDGEN_HEADER_FILE_FAKE}\" found, this should never be created, remove!")
endif()

# From the cmake documentation "If the output of the custom command is not actually created as a
# file on disk it should be marked with the SYMBOLIC source file property."
#
# Not doing this leads to build warnings for the not generated file on windows when using msbuild
set_source_files_properties(
    ${CBINDGEN_HEADER_FILE_FAKE}
    PROPERTIES
        SYMBOLIC TRUE)

# Mark the header file as being a generated header file.
set_source_files_properties(
    ${CBINDGEN_HEADER_FILE}
    PROPERTIES
        GENERATED TRUE
        HEADER_FILE_ONLY TRUE)

# Defines the cbindgen command to generate the header file.
add_custom_command(
    OUTPUT ${CBINDGEN_HEADER_FILE}
    COMMAND
        rustup run nightly
        ${CBINDGEN_EXECUTABLE}
        --config ${CBINDGEN_CONFIG_FILE}
        --crate ${CRATE_NAME}
        --output ${CBINDGEN_HEADER_FILE}
    COMMENT "Generating include/pact.h"
    BYPRODUCTS ${CBINDGEN_HEADER_FILE}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

# Add target for the header file
add_custom_target(generate_header
    COMMENT "Generating header file with cbindgen"
    DEPENDS ${CBINDGEN_HEADER_FILE})

# Teach CMake to install the header file built by the generate_header target
install(FILES "${CBINDGEN_HEADER_FILE}" TYPE INCLUDE)


#################################################################################################
# DOXYGEN
#
# Generates configuration for Doxygen and runs it to generate documentation.
#################################################################################################

# Make sure Doxygen is present.
if(NOT DOXYGEN_FOUND)
    message(WARNING "Could not find Doxygen; FFI documentation will not be generated")
endif()

if(DOXYGEN_FOUND)
  # Configure Doxygen to be appropriate for our C header file.
  set(DOXYGEN_QUIET "YES")
  set(DOXYGEN_HTML_DYNAMIC_MENUS "YES")
  set(DOXYGEN_DYNAMIC_SECTIONS "YES")
  set(DOXYGEN_DISABLE_INDEX "YES")
  set(DOXYGEN_GENERATE_TREEVIEW "YES")
  set(DOXYGEN_ENUM_VALUES_PER_LINE 1)
  set(DOXYGEN_EXT_LINKS_IN_WINDOW "YES")
  set(DOXYGEN_ALPHABETICAL_INDEX "NO")
  set(DOXYGEN_GENERATE_TODOLIST "NO")
  set(DOXYGEN_GENERATE_TESTLIST "NO")
  set(DOXYGEN_GENERATE_BUGLIST "NO")
  set(DOXYGEN_SHOW_USED_FILES "NO")
  set(DOXYGEN_JAVADOC_AUTOBRIEF "YES")
  set(DOXYGEN_JAVADOC_BLOCK "YES")
  set(DOXYGEN_FULL_PATH_NAMES "NO")
  set(DOXYGEN_OPTIMIZE_OUTPUT_FOR_C "YES")

  # Setup Doxygen.
  doxygen_add_docs(
      generate_docs
      ${CBINDGEN_HEADER_FILE}
      ALL
      COMMENT "Generating documentation with doxygen")
endif()

#################################################################################################
# PACKAGE FILES
#
# Defines packaging information for the current project, so it can be used via `find_package`.
#################################################################################################

# Bring in some functions to help generate the config files.
include(CMakePackageConfigHelpers)

# Set the name of the final package configuration file.
set(PACKAGE_CONFIG_NAME "PactFfiConfig.cmake")

# Set the name of the final package version file.
set(PACKAGE_VERSION_NAME "PactFfiConfigVersion.cmake")

# Set the path to the package configuration input file.
set(PACKAGE_CONFIG_INPUT_FILE "cmake/${PACKAGE_CONFIG_NAME}.in")

# Set the path to the package configuration output file.
set(PACKAGE_CONFIG_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE_CONFIG_NAME}")

# Set locations in the installation prefix to put the relevant files.
set(INCLUDE_INSTALL_DIR "include")
set(LIB_INSTALL_DIR "lib")

# Relative path (within install prefix) to place the configuration file.
set(PACKAGE_CONFIG_INSTALL_DIR "${LIB_INSTALL_DIR}/cmake")

# Set the path to the generated package version file.
set(PACKAGE_VERSION_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PACKAGE_VERSION_NAME}")

# Generate the configuration file from the input and put it in the right place
configure_package_config_file(
    # Input file
    ${PACKAGE_CONFIG_INPUT_FILE}
    # Output file
    ${PACKAGE_CONFIG_FILE}
    # Installation destination within the install prefix dir
    INSTALL_DESTINATION ${PACKAGE_CONFIG_INSTALL_DIR}
    # Variables to pass along to the input file
    PATH_VARS INCLUDE_INSTALL_DIR LIB_INSTALL_DIR)

# Generate the version file based on the project version info.
write_basic_package_version_file(${PACKAGE_VERSION_FILE} COMPATIBILITY SameMajorVersion)

# Install config files in the proper directory.
install(FILES ${PACKAGE_CONFIG_FILE} ${PACKAGE_VERSION_FILE} DESTINATION ${PACKAGE_CONFIG_INSTALL_DIR})
