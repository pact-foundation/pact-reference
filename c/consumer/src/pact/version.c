#include <stdio.h>
#include <pact.h>
#include <assert.h>

int check_pact_version()
{
    const char *actual_version = pactffi_version();

    assert(actual_version != NULL && "pactffi_version() returned NULL");
    printf("Pact FFI version: %s\n", actual_version);
    return 0;
}
