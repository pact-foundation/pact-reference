#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

#include <curl/curl.h>
#include <pact.h>

#include "curl_utils.h"
#include "logging.h"
#include "pact/get_user.h"
#include "pact/get_unknown_user.h"
#include "pact/create_user.h"
#include "pact/delete_user.h"
#include "pact/version.h"
#include "pact/logging_test.h"

struct test_t
{
    const char *name;
    int (*func)();
};

/**
 * @brief Draws a boxed message to the console.
 *
 * This function takes a formatted string and prints it within a box made of
 * ASCII characters. The box width adjusts based on the message length, with
 * a minimum width of 60 characters.
 *
 * @param fmt The format string (like printf).
 * @param ... Additional arguments for the format string.
 */
void draw_boxed_message(const char *fmt, ...)
{
    char message[1024];
    va_list args;
    va_start(args, fmt);
    vsnprintf(message, sizeof(message), fmt, args);
    va_end(args);

    size_t len = strlen(message);
    if (len < 60)
        len = 60;
    size_t box_width = len + 4;

    printf("┌");
    for (size_t i = 0; i < box_width; i++)
        printf("─");
    printf("┐\n");

    printf("│ %-*s │\n", (int)len + 2, message);

    printf("└");
    for (size_t i = 0; i < box_width; i++)
        printf("─");
    printf("┘\n");
}

int main()
{
    struct test_t tests[] = {
        {"Pact FFI Version Check", check_pact_version},
        {"Pact Get User Test", pact_get_user},
        {"Pact Get Unknown User Test", pact_get_unknown_user},
        {"Pact Create User Test", pact_create_user},
        {"Pact Delete User Test", pact_delete_user},

        // The following setup a global logger; so only one should be enabled at a time.
        // {"Pact Logging Test (stdout)", pact_logging_stdout},
        // {"Pact Logging Test (stderr)", pact_logging_stderr},
        {"Pact Logging Test (buffer)", pact_logging_buffer},
    };
    int failed_tests = 0;

    for (size_t i = 0; i < sizeof(tests) / sizeof(tests[0]); i++)
    {
        draw_boxed_message("Starting Test: %s", tests[i].name);
        int result = tests[i].func();
        if (result != 0)
        {
            draw_boxed_message("Test '%s' Failed", tests[i].name);
            failed_tests += 1;
        }
        else
        {
            draw_boxed_message("Test '%s' Passed", tests[i].name);
        }
    }

    if (failed_tests > 0)
    {
        draw_boxed_message("Some tests failed (%d failures).", failed_tests);
        return EXIT_FAILURE;
    }
    else
    {
        draw_boxed_message("All tests passed successfully.", 0);
        return EXIT_SUCCESS;
    }
}
