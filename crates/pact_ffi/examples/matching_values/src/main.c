
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

int main(void) {
    printf("FFI Matching function Example\n");

    const char* expression = "matching(datetime, 'yyyy-MM-dd', '2000-01-01')";

    printf("  Calling pactffi_parse_matcher_definition ...\n\n");
    const MatchingRuleDefinitionResult* result = pactffi_parse_matcher_definition(expression);
    printf("  pactffi_parse_matcher_definition returned pointer %p\n", result);

    const char* error = pactffi_matcher_definition_error(result);
    printf("  Checking for error %p\n", error);

    if (error == NULL) {
        MatchingRuleIterator *iter = pactffi_matcher_definition_iter(result);
        printf("  MatchingRuleIterator pointer is %p\n", iter);

        const MatchingRuleResult *rule = pactffi_matching_rule_iter_next(iter);
        printf("    MatchingRuleResult pointer is %p\n", rule);
        if (rule != NULL) {
            const MatchingRule* rule_ptr = pactffi_matching_rule_pointer(rule);
            printf("    Matching Rule pointer is %p\n", rule_ptr);

            const char* error = pactffi_matches_string_value(rule_ptr, "2000-01-01", "1999-04-12", 0);
            printf("    %s matches expression result is '%s'\n", "1999-04-12", error);

            const char* error2 = pactffi_matches_string_value(rule_ptr, "2000-01-01", "1999-04-33", 0);
            printf("    %s matches expression result is '%s'\n", "1999-04-33", error2);

            pactffi_string_delete((char *) error2);
        }

        pactffi_matching_rule_iter_delete(iter);
    } else {
        printf("  error is '%s'\n", error);
        pactffi_string_delete((char *) error);
    }

    printf("  Cleaning up\n");
    pactffi_matcher_definition_delete(result);

    printf("\nFFI Matching function Example: Done OK\n");

    return EXIT_SUCCESS;
}
