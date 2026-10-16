
# Find Cargo, possibly in ~/.cargo. Make sure to check in any `bin` subdirectories
# on the program search path
find_program(CARGO_EXECUTABLE cargo PATHS "$ENV{HOME}/.cargo" PATH_SUFFIXES bin)

set(CARGO_VERSION "")
set(CARGO_CHANNEL "stable")

# If we found it, see if we can get its version.
if(CARGO_EXECUTABLE)
    execute_process(COMMAND ${CARGO_EXECUTABLE} -V OUTPUT_VARIABLE CARGO_VERSION_OUTPUT OUTPUT_STRIP_TRAILING_WHITESPACE)
    
    if(CARGO_VERSION_OUTPUT MATCHES "cargo ([0-9]+\\.[0-9]+\\.[0-9]+).*")
        set(CARGO_VERSION ${CMAKE_MATCH_1})
    endif()

    execute_process(COMMAND ${CARGO_EXECUTABLE} -V OUTPUT_VARIABLE CARGO_CHANNEL_OUTPUT OUTPUT_STRIP_TRAILING_WHITESPACE)

    if(CARGO_CHANNEL_OUTPUT MATCHES "cargo [0-9]+\\.[0-9]+\\.[0-9]+-([a-zA-Z]*).*")
        set(CARGO_CHANNEL ${CMAKE_MATCH_1})
    endif()
endif()

# Hides the CARGO_EXECUTABLE variable unless advanced variables are requested
mark_as_advanced(CARGO_EXECUTABLE)

# Require that we find both the executable and the version. Otherwise error out.
include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(
    Cargo
    REQUIRED_VARS
        CARGO_VERSION
        CARGO_CHANNEL
        CARGO_EXECUTABLE
    VERSION_VAR
        CARGO_VERSION
)
