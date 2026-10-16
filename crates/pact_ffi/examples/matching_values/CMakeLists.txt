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
# PROJECT DECLARATION
#################################################################################################

project(PMFFI_MATCHING_VALUES
        VERSION "0.1.0"
        DESCRIPTION "A basic example of C using Pact matchers"
        LANGUAGES C)

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

set(default_build_type "Release")
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  message(STATUS "Setting build type to '${default_build_type}' as none was specified.")
  set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE STRING "Choose the type of build." FORCE)
  # Set the possible values of build type
  set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release")
endif()

#################################################################################################
# FIND PACT FFI
#
# This ensures CMake can find the pact FFI library file
#################################################################################################

# Sets the search path to the location of the package config
get_filename_component(REAL_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(SEARCH_PATH "${REAL_ROOT}/build/install/lib/cmake")

# Find the pact FFI package and load the imported target
find_package(PactFfi REQUIRED CONFIG PATHS ${SEARCH_PATH})

#################################################################################################
# BUILD
#################################################################################################

# Define the executable
add_executable(example src/main.c)

# Link to pact FFI
if(WIN32)
    target_link_libraries(example PRIVATE PactFfi)
else()
    target_link_libraries(example PRIVATE PactFfi pthread dl m)
endif()
