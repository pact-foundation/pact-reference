
#include "pact.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

int main(void) {
    printf("Error handling example\n");

    /*=======================================================================
     * Simple empty message creation.
     *---------------------------------------------------------------------*/

    Message *msg = pactffi_message_new();

    if (msg == NULL) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    pactffi_message_delete(msg);


    /*=======================================================================
     * Creating a message from a JSON string.
     *---------------------------------------------------------------------*/

    char *json_str = "{\
        \"description\": \"String\",\
        \"providerState\": \"provider state\",\
        \"matchingRules\": {}\
    }";
    Message *msg_json = pactffi_message_new_from_json(0, json_str, PactSpecification_V3);

    if (msg_json == NULL) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    pactffi_message_delete(msg_json);

    printf("Error handling example: DONE OK\n");

    return EXIT_SUCCESS;
}
