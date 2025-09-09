
/**
 * @file get_unknown_user.c
 * @brief Example of using Pact FFI in C to test a consumer against a mock
 * provider for a 'user not found' scenario.
 *
 * This file demonstrates how to use the Pact FFI to write a consumer contract
 * test in C for the case where a requested user does not exist.
 *
 * This example is intentionally similar in structure to `get_user.c`. Please
 * read `get_user.c` first for a detailed walkthrough of the general workflow,
 * setup, and rationale for using Pact FFI in C.
 *
 * This file focuses on the differences required to test error scenarios,
 * specifically when the provider returns a 404 Not Found with a JSON error
 * body. It is intended as a reference for users who want to extend their
 * contract tests to cover negative cases and error handling.
 *
 * Key differences from `get_user.c`:
 *
 * - The provider state is set to indicate the user does not exist.
 * - The expected response status is 404 (Not Found) instead of 200.
 * - The response body is a JSON object with an error message: {"detail": "User
 *   not found"}.
 * - The test asserts that the client receives a 404 and the correct error
 *   message.
 *
 * For more information, see the Pact documentation:
 *   https://docs.pact.io/5-minute-getting-started-guide#scope-of-a-consumer-pact-test
 */

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pact.h"

#include "curl_utils.h"
#include "logging.h"

// See get_user.c for detailed documentation of the general Pact setup and
// lifecycle functions.
static PactHandle create_pact_handle(void) {
  log_info("Creating Pact between C consumer and provider");
  PactHandle pact = pactffi_new_pact("c-consumer", "c-provider");
  pactffi_with_specification(pact, PactSpecification_V4);
  return pact;
}

/**
 * @brief Define the interaction for the 'user not found' Pact test.
 *
 * This function sets up the expected provider state, request, and response for
 * the GET /users/123 endpoint when the user does not exist. Unlike the success
 * case, the provider state is set to "the user doesn't exist", the response
 * status is 404, and the response body contains an error message.
 *
 * @param pact The Pact handle to add the interaction to.
 * @return InteractionHandle The configured interaction handle.
 */
static InteractionHandle create_and_configure_interaction(PactHandle pact) {
  log_info("Defining interaction for GET /users/123 (user not found)");
  InteractionHandle interaction =
      pactffi_new_interaction(pact, "A request for an unknown user");
  pactffi_given_with_param(interaction, "the user doesn't exist", "id", "123");
  pactffi_upon_receiving(interaction, "A request for an unknown user");
  pactffi_with_request(interaction, "GET", "/users/123");
  pactffi_response_status(interaction, 404);
  pactffi_with_header_v2(interaction, InteractionPart_Response, "Content-Type",
                         0, "application/json");
  pactffi_with_body(interaction, InteractionPart_Response, "application/json",
                    "{\"detail\": \"User not found\"}");
  return interaction;
}

/**
 * @brief Run the Pact test for the 'user not found' scenario.
 *
 * This function launches the Pact mock server, constructs the request URL, and
 * uses cURL to perform the GET request. It asserts that the response is a 404
 * with the expected error message. See get_user.c for the general structure.
 *
 * @param pact The Pact handle for which to start the mock server.
 * @return int The port number of the running mock server, or 1 on failure.
 */
static int run_pact_test(PactHandle pact) {
  log_info("Executing Pact consumer test for unknown user");
  int port = pactffi_create_mock_server_for_transport(pact, "localhost", 0,
                                                      NULL, NULL);
  if (port <= 0)
    log_err("Failed to start mock server, port: %d", port);

  char url[256];
  snprintf(url, sizeof(url), "http://localhost:%d/users/123", port);
  struct ResponseBuffer resp;
  response_buffer_init(&resp);
  int rc = curl_get(url, NULL, &resp);
  if (rc != 0) {
    log_warn("cURL GET request failed with code %d", rc);
    response_buffer_free(&resp);
    return 1;
  }

  assert(resp.status_code == 404);
  assert(strstr(resp.buffer, "User not found") != NULL);
  log_info("Response: %s\n", resp.buffer);
  response_buffer_free(&resp);

  return port;
}

// See get_user.c for documentation of result validation and cleanup.
static void validate_results(int port, PactHandle pact) {
  log_info("Validating Pact test results");
  int matched = pactffi_mock_server_matched(port);
  if (!matched) {
    const char *mismatch_json = pactffi_mock_server_mismatches(port);
    printf("Mismatches: %s\n", mismatch_json);
    assert(0 && "Pact interaction did not match");
  }
  int write_ok = pactffi_write_pact_file(port, "./pacts", 0);
  if (write_ok != 0)
    log_err("Failed to write pact file, error code: %d", write_ok);

  pactffi_cleanup_mock_server(port);
  struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
  pactffi_pact_model_delete(pact_ptr);
}

/**
 * @brief Main entry point for the Pact GET unknown user test.
 *
 * This function orchestrates the creation of the Pact, the definition of the
 * 'user not found' interaction, execution of the test, and validation of the
 * results. See get_user.c for the general workflow; this function highlights
 * the error case.
 *
 * @return int 0 on success.
 */
int pact_get_unknown_user() {
  PactHandle pact = create_pact_handle();
  InteractionHandle interaction = create_and_configure_interaction(pact);
  int port = run_pact_test(pact);
  validate_results(port, pact);
  return 0;
}
