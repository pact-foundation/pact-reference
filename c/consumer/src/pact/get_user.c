
/**
 * @file get_user.c
 * @brief Example of using Pact FFI in C to test a consumer against a mock
 * provider.
 *
 * This file demonstrates how to use the Pact FFI to write a consumer contract
 * test in C. It sets up a mock provider, defines expected interactions, and
 * verifies that the consumer code can communicate with the provider as
 * expected. This is a pedagogical example for C developers and for those
 * integrating Pact via the FFI in other languages.
 *
 * The test simulates a GET request to retrieve a user, checks the response, and
 * writes the resulting Pact file for provider verification. Each function is
 * documented to explain its role in the contract test lifecycle.
 *
 * For more information, see the Pact documentation:
 * https://docs.pact.io/5-minute-getting-started-guide#scope-of-a-consumer-pact-test
 *
 * When implementing Pact tests in a project, make sure to test the client code
 * itself to ensure it is functioning as expected, rather than verifying that
 * the cURL library (or any other HTTP client library) works correctly.
 */

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pact.h"

#include "curl_utils.h"
#include "logging.h"

/**
 * @brief Create and configure a new Pact handle for the test.
 *
 * This function initializes a Pact between a C consumer and provider, and sets
 * the specification version to V4. The handle is used for all further Pact
 * operations.
 *
 * The V4 specification has been available for some time and should be preferred
 * for new tests.
 *
 * @return PactHandle The initialized Pact handle.
 */
static PactHandle create_pact_handle(void) {
  log_info("Creating Pact between C consumer and provider");
  PactHandle pact = pactffi_new_pact("c-consumer", "c-provider");
  pactffi_with_specification(pact, PactSpecification_V4);
  return pact;
}

/**
 * @brief Define and configure the expected interaction for the Pact test.
 *
 * This function sets up the expected provider state, request, and response for
 * the GET /users/123 endpoint. It uses Pact matchers to allow flexible matching
 * of the response fields (e.g., integer, string, datetime).
 *
 * @param pact The Pact handle to add the interaction to.
 * @return InteractionHandle The configured interaction handle.
 */
static InteractionHandle create_and_configure_interaction(PactHandle pact) {
  log_info("Defining interaction for GET /users/123");
  InteractionHandle interaction =
      pactffi_new_interaction(pact, "A user request");
  pactffi_given_with_param(interaction, "the user exists", "id", "123");
  pactffi_given_with_param(interaction, "the user exists", "name", "Alice");
  pactffi_upon_receiving(interaction, "A user request");
  pactffi_with_request(interaction, "GET", "/users/123");
  pactffi_response_status(interaction, 200);
  pactffi_with_header_v2(interaction, InteractionPart_Response, "Content-Type",
                         0, "application/json");
  pactffi_with_body(
      interaction, InteractionPart_Response, "application/json",
      "{"
      "\"id\": {\"pact:matcher:type\": \"integer\", \"value\": 123},"
      "\"name\": {\"pact:matcher:type\": \"type\", \"value\": \"Alice\"},"
      "\"created_on\": {\"pact:matcher:type\": \"datetime\"}"
      "}");
  return interaction;
}

/**
 * @brief Run the Pact test against the mock server and making the HTTP request.
 *
 * This function launches the Pact mock server, constructs the request URL, and
 * uses cURL to perform the GET request. It asserts that the response matches
 * expectations and prints the response body for inspection.
 *
 * @param pact The Pact handle for which to start the mock server.
 * @return int The port number of the running mock server, or 1 on failure.
 */
static int run_pact_test(PactHandle pact) {
  log_info("Executing Pact consumer test");
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

  assert(resp.status_code == 200);
  assert(strstr(resp.buffer, "Alice") != NULL);
  log_info("Response: %s\n", resp.buffer);
  response_buffer_free(&resp);

  return port;
}

/**
 * @brief Validate the results of the Pact test and clean up resources.
 *
 * This function checks if the mock server interactions matched the
 * expectations. If there are mismatches, it prints the details and fails the
 * test. It also writes the Pact file to disk for provider verification and
 * cleans up the mock server and Pact model resources.
 *
 * @param port The port number of the running mock server.
 * @param pact The Pact handle to clean up.
 */
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
 * @brief Main entry point for the Pact GET user test.
 *
 * This function orchestrates the creation of the Pact, the definition of the
 * interaction, execution of the test, and validation of the results. It is
 * intended as a reference for developers writing consumer contract tests in C
 * using the Pact FFI.
 *
 * @return int 0 on success.
 */
int pact_get_user() {
  PactHandle pact = create_pact_handle();
  InteractionHandle interaction = create_and_configure_interaction(pact);
  int port = run_pact_test(pact);
  validate_results(port, pact);
  return 0;
}
