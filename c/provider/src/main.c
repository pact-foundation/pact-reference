/**
 * @file main.c
 * @brief Example of using Pact FFI in C to verify a provider implementation.
 *
 * This file demonstrates how to use the Pact FFI to verify that a provider
 * implementation satisfies the contracts defined by consumers. It sets up an
 * HTTP server, configures the Pact verifier, handles provider states, and runs
 * verification against the consumer contracts.
 *
 * The verification process:
 *
 * 1. Starts a HTTP server that implements the provider API
 * 2. Configures the Pact verifier with provider details and contract files
 * 3. Registers callbacks for provider state setup/teardown
 * 4. Runs verification - the verifier replays each interaction from the
 *    contract against the running provider
 * 5. Reports verification results
 *
 * Unlike the C example consumer, this is not structured in a way that each
 * interaction is verified individually in isolation. Instead, the provider
 * server is started once, and the Pact verifier runs all interactions against
 * it in a single execution. State changes are handled via HTTP requests to a
 * dedicated endpoint.
 *
 * For more information, see the Pact documentation:
 * https://docs.pact.io/implementation_guides/rust/pact_verifier
 */

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
// winsock2.h must be included before windows.h to avoid conflicts
#include <winsock2.h>

#include <windows.h>
#define usleep(us) Sleep((us) / 1000)
#else
#include <unistd.h>
#endif

#include <pact.h>

#include "http_server.h"
#include "logging.h"
#include "provider/provider_state.h"
#include "provider/user_handlers.h"

// Global server context for signal handling
static struct HttpServerContext server_context = {0};

/**
 * @brief Starts the HTTP provider server.
 *
 * This function starts an HTTP server on the specified port that implements
 * the provider API. The server handles user-related endpoints (GET, POST,
 * DELETE /users).
 *
 * @param port Port number to listen on
 * @return 0 on success, -1 on failure
 */
static int start_provider_server(unsigned int port) {
  log_info("Starting provider HTTP server on port %u", port);

  // Initialize provider state
  struct ProviderState *state = get_global_provider_state();
  provider_state_init(state);

  // Start HTTP server with user request router
  if (http_server_start(&server_context, port, user_request_router, NULL) !=
      0) {
    log_err("Failed to start HTTP server");
    return -1;
  }

  return 0;
}

/**
 * @brief Configures and runs the Pact verification.
 *
 * This function sets up the Pact verifier with the provider details,
 * contract files, and provider state callbacks, then runs the verification.
 *
 * @param port Port number where the provider server is running
 * @return 0 on success, non-zero on verification failure
 */
static int run_pact_verification(unsigned int port) {
  log_info("Configuring Pact verifier");

  // Enable Pact FFI logging
  pactffi_log_to_buffer(LevelFilter_Info);

  // Create verifier handle
  VerifierHandle *handle =
      pactffi_verifier_new_for_application("pact-provider", "0.1.0");
  if (!handle) {
    log_err("Failed to create verifier handle");
    return -1;
  }

  // Configure provider details
  char provider_url[256];
  snprintf(provider_url, sizeof(provider_url), "localhost:%u", port);
  pactffi_verifier_set_provider_info(handle, "c-provider", NULL, provider_url,
                                     0, NULL);

  // Add pact file sources
  // In a real scenario, you might fetch pacts from a Pact Broker, but we just
  // use the Pact generated from the consumer tests
  pactffi_verifier_add_file_source(
      handle, "../consumer/pacts/c-consumer-c-provider.json");

  // Configure provider state change URL
  // The verifier will POST to this endpoint to set up/tear down state
  char state_change_url[256];
  snprintf(state_change_url, sizeof(state_change_url),
           "http://localhost:%u/__pact/provider-state", port);
  pactffi_verifier_set_provider_state(handle, state_change_url, 1, 1);

  // Set filter to run all interactions
  pactffi_verifier_set_filter_info(handle, NULL, NULL, 0);

  // Enable colored output
  pactffi_verifier_set_coloured_output(handle, 1);
  // Do not treat missing pacts as an error
  pactffi_verifier_set_no_pacts_is_error(handle, 0);

  log_info("Running Pact verification");
  int result = pactffi_verifier_execute(handle);

  // Print verification logs
  const char *logs = pactffi_verifier_logs(handle);
  if (logs) {
    draw_boxed_message("Pact Verification Logs");
    printf("%s\n", logs);
    pactffi_free_string((char *)logs);
  }

  // Get verification JSON results
  const char *json_result = pactffi_verifier_json(handle);
  if (json_result) {
    log_debug("Verification JSON result: %s", json_result);
    pactffi_free_string((char *)json_result);
  }

  // Clean up
  pactffi_verifier_shutdown(handle);

  if (result == 0) {
    log_info("✓ Pact verification PASSED");
  } else {
    log_warn("✗ Pact verification FAILED with code %d", result);
  }

  return result;
}

/**
 * @brief Main entry point for the provider verification.
 *
 * This function orchestrates the provider verification process:
 * 1. Starts the provider HTTP server
 * 2. Runs the Pact verification
 * 3. Stops the server and reports results
 *
 * @return 0 on success, non-zero on failure
 */
int main(void) {
  draw_boxed_message("Pact C Provider Verification Example");

  const unsigned int provider_port = 8080;
  if (start_provider_server(provider_port) != 0) {
    return EXIT_FAILURE;
  }

  // Give the server a moment to start up
  usleep(100000); // 100ms

  int result = run_pact_verification(provider_port);

  http_server_stop(&server_context);

  if (result == 0) {
    draw_boxed_message("All verifications PASSED");
    return EXIT_SUCCESS;
  } else {
    draw_boxed_message("Some verifications FAILED");
    return EXIT_FAILURE;
  }
}
