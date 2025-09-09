#ifndef PACT_VERSION_H
#define PACT_VERSION_H

/**
 * Checks that the linked Pact FFI library version is compatible.
 *
 * This function retrieves the version of the Pact FFI library at runtime
 * and compares it to the expected version. If the versions are incompatible,
 * an error message is printed to stderr.
 *
 * @return 0 if the versions are compatible, non-zero otherwise.
 */
int check_pact_version();

#endif // PACT_VERSION_H
