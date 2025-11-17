/**
 * @file user_handlers.h
 * @brief HTTP request handlers for the provider API
 *
 * This module implements the user-facing API endpoints for the Pact provider
 * example. In practice, this is the part of the application that serves real
 * HTTP requests and provides the functionality that consumers interact with.
 *
 * This implements a simple user management API with the following endpoints:
 *
 * - GET /users/{id} - Retrieve a user by ID
 * - POST /users - Create a new user
 * - DELETE /users/{id} - Delete a user by ID
 *
 * For Pact's provider state management, we also implement:
 *
 * - POST /__pact/provider-state - Handle provider state changes
 *
 * The handler functions defined here (though typically would be only defined
 * when running tests) allows the Pact verifier to manipulate the provider's
 * state to ensure that each interaction is tested under the correct
 * preconditions.
 *
 * A typical flow for verifying an interaction is:
 *
 * 1. For each provider state:
 *    - Verifier POSTs to /__pact/provider-state with `{"action": "setup",
 *      "state": "$state_name", "params": {...}}`
 *    - Server sets up the state accordingly
 * 2. Verifier executes the actual interaction (e.g., GET /users/123)
 * 3. Verifier validates response matches the contract
 * 4. For each provider state teardown:
 *    - Server POSTs to /__pact/provider-state with `{"action": "teardown",
 *      "state": "$state_name"}`
 *    - Server cleans up the state
 *
 * @see provider_state.h for state management functions
 * @see https://docs.pact.io/getting_started/provider_states
 */

#ifndef USER_HANDLERS_H
#define USER_HANDLERS_H

#include "http_server.h"
#include <microhttpd.h>

/**
 * @brief Main HTTP request router that dispatches to specific user handlers.
 *
 * This function acts as the central request dispatcher for the provider API.
 * It examines the HTTP method and URL pattern to route requests to the
 * appropriate handler function.
 *
 * Routing logic:
 * - GET /users/{id} → handle_get_user()
 * - POST /users → handle_create_user()
 * - DELETE /users/{id} → handle_delete_user()
 * - POST /__pact/provider-state → handle_provider_state_change()
 * - All other routes → 404 Not Found
 *
 * This is the handler function registered with the HTTP server in main.c.
 *
 * @param request Pointer to HttpRequest structure with request details
 * @param response Pointer to HttpResponse structure to fill with response
 * @return MHD_YES on success, MHD_NO on failure
 *
 * @see http_server_start() for where this router is registered
 */
enum MHD_Result user_request_router(const struct HttpRequest *request,
                                    struct HttpResponse *response);

/**
 * @brief Handles GET /users/{id} requests to retrieve user information.
 *
 * This endpoint retrieves a user by their ID from the provider state.
 * The ID is extracted from the URL path (e.g., /users/123 → ID 123).
 *
 * Response codes:
 * - 200 OK: User found, returns JSON with user details
 * - 400 Bad Request: Invalid user ID format (non-numeric)
 * - 404 Not Found: User ID not found in provider state
 *
 * Example successful response:
 * @code{.json}
 * {
 *   "id": 123,
 *   "name": "John Doe",
 *   "created_on": "2025-11-17T10:30:00+00:00"
 * }
 * @endcode
 *
 * @param request Pointer to HttpRequest structure containing the GET request
 * @param response Pointer to HttpResponse structure to populate with user data
 * @return MHD_YES on success (even if user not found)
 *
 * @see parse_user_id_from_path() for URL parsing logic
 * @see provider_state_find_user() for state lookup
 */
enum MHD_Result handle_get_user(const struct HttpRequest *request,
                                struct HttpResponse *response);

/**
 * @brief Handles POST /users requests to create a new user.
 *
 * This endpoint creates a new user in the provider state. The request body
 * should contain a JSON object with at least a "name" field. The server
 * automatically generates a unique ID and timestamp for the new user.
 *
 * Expected request body:
 * @code{.json}
 * {
 *   "name": "Jane Smith"
 * }
 * @endcode
 *
 * Response codes:
 * - 201 Created: User successfully created, returns JSON with user details
 * - 400 Bad Request: Missing request body or invalid/missing "name" field
 * - 500 Internal Server Error: Failed to add user to provider state (e.g.,
 * state full)
 *
 * Example successful response (201):
 * @code{.json}
 * {
 *   "id": 1042,
 *   "name": "Jane Smith",
 *   "created_on": "2025-11-17T10:30:00+00:00"
 * }
 * @endcode
 *
 * @param request Pointer to HttpRequest structure containing the POST body
 * @param response Pointer to HttpResponse structure to populate with created
 * user
 * @return MHD_YES on success
 *
 * @note This implementation uses simple string parsing for JSON. Production
 *       code should use a proper JSON parsing library.
 * @see provider_state_add_user() for adding users to state
 */
enum MHD_Result handle_create_user(const struct HttpRequest *request,
                                   struct HttpResponse *response);

/**
 * @brief Handles DELETE /users/{id} requests to remove a user.
 *
 * This endpoint deletes a user from the provider state by their ID.
 * The ID is extracted from the URL path (e.g., /users/123 → ID 123).
 *
 * Response codes:
 * - 204 No Content: User successfully deleted (no response body)
 * - 400 Bad Request: Invalid user ID format (non-numeric)
 * - 404 Not Found: User ID not found in provider state
 *
 * @param request Pointer to HttpRequest structure containing the DELETE request
 * @param response Pointer to HttpResponse structure (empty body on success)
 * @return MHD_YES on success (even if user not found)
 *
 * @see parse_user_id_from_path() for URL parsing logic
 * @see provider_state_remove_user() for removal from state
 */
enum MHD_Result handle_delete_user(const struct HttpRequest *request,
                                   struct HttpResponse *response);

/**
 * @brief Handles POST /__pact/provider-state requests for provider state
 * changes.
 *
 * This endpoint is called by the Pact verifier to set up or tear down
 * provider state before and after each interaction. It is a critical part
 * of the Pact verification process, allowing tests to be deterministic
 * and isolated from each other.
 *
 * Expected request body format:
 * @code{.json}
 * {
 *   "state": "the user exists",
 *   "params": {"id": 123, "name": "Test User"},
 *   "action": "setup"
 * }
 * @endcode
 *
 * The "action" field can be:
 * - "setup" (or omitted): Prepare the state before running an interaction
 * - "teardown": Clean up the state after running an interaction
 *
 * Common provider states in this example:
 * - "the user exists": Creates a user with the given id and name
 * - "the user doesn't exist": Ensures the user with the given id does not exist
 * - "" (empty): No state setup needed
 *
 * Response:
 * Always returns 200 OK with {"result": "success"} if the state change
 * was processed (even if the state name is unknown).
 *
 * @param request Pointer to HttpRequest structure containing state change
 * request
 * @param response Pointer to HttpResponse structure to populate
 * @return MHD_YES on success
 *
 * @note This endpoint should NOT be publicly accessible in production systems
 * @see handle_provider_state_setup() for state setup implementation
 * @see handle_provider_state_teardown() for state teardown implementation
 * @see https://docs.pact.io/getting_started/provider_states
 */
enum MHD_Result handle_provider_state_change(const struct HttpRequest *request,
                                             struct HttpResponse *response);

#endif // USER_HANDLERS_H
