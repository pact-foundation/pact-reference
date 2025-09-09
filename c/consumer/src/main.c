#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <curl/curl.h>
#include <pact.h>

#include "curl_utils.h"
#include "logging.h"
#include "pact/create_user.h"
#include "pact/delete_user.h"
#include "pact/get_unknown_user.h"
#include "pact/get_user.h"
#include "pact/logging_test.h"
#include "pact/version.h"

struct test_t {
  const char *name;
  int (*func)();
};

int main() {
  struct test_t tests[] = {
      {"Pact FFI Version Check", check_pact_version},
      {"Pact Get User Test", pact_get_user},
      {"Pact Get Unknown User Test", pact_get_unknown_user},
      {"Pact Create User Test", pact_create_user},
      {"Pact Delete User Test", pact_delete_user},

      // The following setup a global logger; so only one should be enabled at a
      // time.
      // {"Pact Logging Test (stdout)", pact_logging_stdout},
      // {"Pact Logging Test (stderr)", pact_logging_stderr},
      {"Pact Logging Test (buffer)", pact_logging_buffer},
  };
  int failed_tests = 0;

  for (size_t i = 0; i < sizeof(tests) / sizeof(tests[0]); i++) {
    draw_boxed_message("Starting Test: %s", tests[i].name);
    int result = tests[i].func();
    if (result != 0) {
      draw_boxed_message("Test '%s' Failed", tests[i].name);
      failed_tests += 1;
    } else {
      draw_boxed_message("Test '%s' Passed", tests[i].name);
    }
  }

  if (failed_tests > 0) {
    draw_boxed_message("Some tests failed (%d failures).", failed_tests);
    return EXIT_FAILURE;
  } else {
    draw_boxed_message("All tests passed successfully.", 0);
    return EXIT_SUCCESS;
  }
}
