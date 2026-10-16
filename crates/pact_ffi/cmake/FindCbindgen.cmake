
# Find Cbindgen, possibly in ~/.cargo. Make sure to check in any `bin` subdirectories
# on the program search path
find_program(CBINDGEN_EXECUTABLE cbindgen PATHS "$ENV{HOME}/.cargo" PATH_SUFFIXES bin)

# If we found it, see if we can get its version.
if(CBINDGEN_EXECUTABLE)
    execute_process(COMMAND ${CBINDGEN_EXECUTABLE} -V OUTPUT_VARIABLE CBINDGEN_VERSION_OUTPUT OUTPUT_STRIP_TRAILING_WHITESPACE)
    
    if(CBINDGEN_VERSION_OUTPUT MATCHES "cbindgen ([0-9]+\\.[0-9]+\\.[0-9]+).*")
        set(CBINDGEN_VERSION ${CMAKE_MATCH_1})
    endif()
endif()

# Hides the CBINDGEN_EXECUTABLE variable unless advanced variables are requested
mark_as_advanced(CBINDGEN_EXECUTABLE)

# Require that we find both the executable and the version. Otherwise error out.
include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(Cbindgen REQUIRED_VARS CBINDGEN_EXECUTABLE CBINDGEN_VERSION VERSION_VAR CBINDGEN_VERSION)
