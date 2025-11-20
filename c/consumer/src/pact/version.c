/**
 * Pact FFI Version Check
 *
 * This file provides a simple function to check and print the version of the
 * Pact FFI library being used. It calls the `pactffi_version` function from the
 * Pact FFI and asserts that the returned version string is not NULL. The
 * version is then printed to standard output.
 *
 * This is perhaps the most basic usage of the Pact FFI, serving as a sanity
 * check to ensure that the library is correctly linked and accessible from C
 * code.
 */
#include <assert.h>
#include <stdio.h>

#include "pact.h"

int check_pact_version() {
  const char *actual_version = pactffi_version();

  assert(actual_version != NULL && "pactffi_version() returned NULL");
  printf("Pact FFI version: %s\n", actual_version);
  return 0;
}
