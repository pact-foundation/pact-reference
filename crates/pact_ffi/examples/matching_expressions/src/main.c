
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
    printf("FFI Matching Definition Example\n");

    const char* expression = "matching(datetime, 'yyyy-MM-dd','2000-01-01')";

    printf("  Calling pactffi_parse_matcher_definition ...\n\n");
    const MatchingRuleDefinitionResult* result = pactffi_parse_matcher_definition(expression);
    printf("  pactffi_parse_matcher_definition returned pointer %p\n", result);

    const char* error = pactffi_matcher_definition_error(result);
    printf("  Checking for error %p\n", error);

    if (error == NULL) {
        const char* value = pactffi_matcher_definition_value(result);
        printf("  No error, value = '%s'\n", value);
        pactffi_string_delete((char *) value);

        ExpressionValueType valueType = pactffi_matcher_definition_value_type(result);
        printf("  value type = '%d'\n", valueType);
        switch (valueType) {
            case ExpressionValueType_Unknown: printf("    %d == Unknown\n", valueType); break;
            case ExpressionValueType_String: printf("    %d == String\n", valueType); break;
            case ExpressionValueType_Number: printf("    %d == Number\n", valueType); break;
            case ExpressionValueType_Integer: printf("    %d == Integer\n", valueType); break;
            case ExpressionValueType_Decimal: printf("    %d == Decimal\n", valueType); break;
            case ExpressionValueType_Boolean: printf("    %d == Boolean\n", valueType); break;
        }

        const Generator* generator = pactffi_matcher_definition_generator(result);
        printf("  Generator pointer is %p\n", generator);

        MatchingRuleIterator *iter = pactffi_matcher_definition_iter(result);
        printf("  MatchingRuleIterator pointer is %p\n", iter);

        const MatchingRuleResult *rule = pactffi_matching_rule_iter_next(iter);
        if (rule == NULL) {
            printf("  There are no matching rules, pactffi_matching_rule_iter_next returned NULL\n");
        }

        int count = 0;
        while (rule != NULL) {
            printf("    %d MatchingRuleResult pointer is %p\n", count, rule);

            const char* ref_name = pactffi_matching_rule_reference_name(rule);
            printf("    Matching Rule Reference pointer is %p\n", ref_name);

            if (ref_name == NULL) {
                printf("    Matching Rule is not a reference\n");
                unsigned short rule_id = pactffi_matching_rule_id(rule);
                printf("    Matching Rule ID is %d\n", rule_id);
                const char* rule_value = pactffi_matching_rule_value(rule);
                printf("    Matching Rule value pointer is %p\n", rule_value);
                if (rule_value != NULL) {
                    printf("    Matching Rule value is '%s'\n", rule_value);
                }
                const MatchingRule* rule_ptr = pactffi_matching_rule_pointer(rule);
                printf("    Matching Rule pointer is %p\n", rule_ptr);
                const char* json = pactffi_matching_rule_to_json(rule_ptr);
                printf("      Matching Rule JSON = %s\n", json);
                pactffi_string_delete((char *) json);
            } else {
                printf("    Matching Rule Reference is '%s'\n", ref_name);
            }

            count++;
            rule = pactffi_matching_rule_iter_next(iter);
        }

        printf("  Number of matching rules found = %d\n", count);

        pactffi_matching_rule_iter_delete(iter);
    } else {
        printf("  error is '%s'\n", error);
        pactffi_string_delete((char *) error);
    }

    printf("  Cleaning up\n");
    pactffi_matcher_definition_delete(result);

    printf("\nFFI Matching Definition Example: Done OK\n");

    return EXIT_SUCCESS;
}
