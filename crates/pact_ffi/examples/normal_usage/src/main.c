
#include "pact.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

#define CHK(ptr) {\
    char msg[ERROR_MSG_LEN];\
    int error = pactffi_get_error_message(msg, ERROR_MSG_LEN);\
    if (error < 0) {\
        printf("%s\n", msg);\
        exit(EXIT_FAILURE);\
    }\
}

Message* msg_json() {
    const int id = 0;
    const int spec = PactSpecification_V3;
    const char *json = "{\
        \"description\": \"A basic message.\",\
        \"providerStates\": [\
	    { \"name\": \"state 1\", \"params\": {} },\
	    { \"name\": \"state 2\", \"params\": {} },\
	    { \"name\": \"state 3\", \"params\": {} }\
        ]\
    }";

    Message *msg = pactffi_message_new_from_json(id, json, spec);
    CHK(msg);

    return msg;
}

int main(void) {
    printf("FFI Example\n");

    Message *msg = msg_json();

    ProviderStateIterator *iter = pactffi_message_get_provider_state_iter(msg);
    CHK(iter);

    ProviderState *state = pactffi_provider_state_iter_next(iter);
    while (state != NULL) {
        const char *name = pactffi_provider_state_get_name(state);
        CHK(name);
        printf("Provider State Name: %s\n", name);
        state = pactffi_provider_state_iter_next(iter);
    }

    pactffi_provider_state_iter_delete(iter);
    pactffi_message_delete(msg);

    printf("FFI Example: Done OK\n");

    return EXIT_SUCCESS;
}
