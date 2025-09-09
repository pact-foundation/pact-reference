/**
 * @file logging_test.c
 * @brief Test that Pact logs errors when interactions are not verified.
 *
 * This test creates a Pact interaction and starts the mock server, but does NOT
 * execute the test against the mock server. It then cleans up the handle, which
 * should cause a mismatch and log an error.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "pact.h"

#include "logging.h"

static PactHandle create_pact_handle(void) {
  log_info("Creating Pact for logging test");
  PactHandle pact = pactffi_new_pact("c-consumer", "c-provider");
  pactffi_with_specification(pact, PactSpecification_V4);
  return pact;
}

static InteractionHandle create_and_configure_interaction(PactHandle pact) {
  log_info("Defining interaction for logging test");
  InteractionHandle interaction =
      pactffi_new_interaction(pact, "A logging test interaction");
  pactffi_given_with_param(interaction, "the user exists", "id", "999");
  pactffi_upon_receiving(interaction, "A logging test interaction");
  pactffi_with_request(interaction, "GET", "/users/999");
  pactffi_response_status(interaction, 200);
  pactffi_with_header_v2(interaction, InteractionPart_Response, "Content-Type",
                         0, "application/json");
  pactffi_with_body(interaction, InteractionPart_Response, "application/json",
                    "{\"id\":999,\"name\":\"Test\"}");
  return interaction;
}

/**
 * @brief Main entry point for the Pact logging test.
 *
 * This function creates a Pact, defines an interaction, starts the mock server,
 * but does NOT execute any request against it. It then cleans up, which should
 * cause a mismatch and log an error.
 *
 * @return int 0 on success.
 */
int pact_logging_stdout() {
  int result = pactffi_log_to_stdout(LevelFilter_Info);
  if (result != 0)
    log_err("Failed to setup logging to stdout: %d", result);

  PactHandle pact = create_pact_handle();
  create_and_configure_interaction(pact);
  int port = pactffi_create_mock_server_for_transport(pact, "localhost", 0,
                                                      NULL, NULL);
  if (port <= 0)
    log_err("Failed to start mock server for logging test, port: %d", port);

  // Do NOT execute any HTTP request against the mock server

  // Now clean up, which should log a mismatch error
  int matched = pactffi_mock_server_matched(port);
  if (matched) {
    log_warn("Unexpected: mock server matched with no requests");
    pactffi_cleanup_mock_server(port);
    struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
    pactffi_pact_model_delete(pact_ptr);
    return 1;
  }
  const char *mismatch_json = pactffi_mock_server_mismatches(port);
  log_info("Logging Test Mismatches: %s\n", mismatch_json);
  pactffi_cleanup_mock_server(port);
  struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
  pactffi_pact_model_delete(pact_ptr);
  return 0;
}

int pact_logging_stderr() {
  int result = pactffi_log_to_stderr(LevelFilter_Info);
  if (result != 0)
    log_err("Failed setup logging to stderr: %d", result);

  PactHandle pact = create_pact_handle();
  create_and_configure_interaction(pact);
  int port = pactffi_create_mock_server_for_transport(pact, "localhost", 0,
                                                      NULL, NULL);
  if (port <= 0)
    log_err("Failed to start mock server for logging test (stderr), port: %d",
            port);

  // Do NOT execute any HTTP request against the mock server

  int matched = pactffi_mock_server_matched(port);
  if (matched) {
    log_warn("Unexpected: mock server matched with no requests (stderr)");
    pactffi_cleanup_mock_server(port);
    struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
    pactffi_pact_model_delete(pact_ptr);
    return 1;
  }
  const char *mismatch_json = pactffi_mock_server_mismatches(port);
  log_info("Logging Test Mismatches (stderr): %s\n", mismatch_json);
  pactffi_cleanup_mock_server(port);
  struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
  pactffi_pact_model_delete(pact_ptr);
  return 0;
}

int pact_logging_buffer() {
  int result = pactffi_log_to_buffer(LevelFilter_Info);
  if (result != 0)
    log_err("Failed to setup logging to buffer: %d", result);

  PactHandle pact = create_pact_handle();
  create_and_configure_interaction(pact);
  int port = pactffi_create_mock_server_for_transport(pact, "localhost", 0,
                                                      NULL, NULL);
  if (port <= 0)
    log_err("Failed to start mock server for logging test (buffer), port: %d",
            port);

  // Do NOT execute any HTTP request against the mock server

  int matched = pactffi_mock_server_matched(port);
  if (matched) {
    log_warn("Unexpected: mock server matched with no requests (buffer)");
    pactffi_cleanup_mock_server(port);
    struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
    pactffi_pact_model_delete(pact_ptr);
    return 1;
  }
  const char *mismatch_json = pactffi_mock_server_mismatches(port);
  log_info("Logging Test Mismatches (buffer): %s\n", mismatch_json);

  // Fetch the buffer contents for validation
  const char *buffer_logs = pactffi_fetch_log_buffer(NULL);
  if (buffer_logs) {
    printf("Buffer logs:\n%s\n", buffer_logs);
    // If pactffi_fetch_log_buffer allocates, free if needed (see API)
    // pactffi_string_delete((char*)buffer_logs);
  } else {
    printf("No buffer logs captured.\n");
  }

  pactffi_cleanup_mock_server(port);
  struct Pact *pact_ptr = pactffi_pact_handle_to_pointer(pact);
  pactffi_pact_model_delete(pact_ptr);
  return 0;
}
