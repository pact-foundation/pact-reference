/**
 * @file provider_state.h
 * @brief Provider state management for Pact verification
 *
 * This module manages the provider's application state during Pact
 * verification. Provider state is a critical concept in Pact testing that
 * allows the provider to prepare its system (database, caches, etc.) to match
 * the preconditions expected by each consumer interaction.
 *
 * When a consumer records a Pact, they specify what state the provider should
 * be in for each interaction. For example:
 *
 * - "Given user 123 exists, when I GET /users/123, then I should receive..."
 * - "Given no users exist, when I POST /users, then I should receive..."
 *
 * During verification, the Pact verifier calls the provider state setup
 * endpoint before replaying each interaction. Once the interaction is complete,
 * it may also call a teardown endpoint to clean up any test data (depending
 * on configuration).
 *
 * In this example, we maintain an in-memory list of users as the provider
 * state, though in a real application this would typically involve setting up a
 * test database, configuring mocks, or preparing other application state.
 *
 * @see https://docs.pact.io/getting_started/provider_states
 */

#ifndef PROVIDER_STATE_H
#define PROVIDER_STATE_H

#include <cjson/cJSON.h>
#include <pact.h>

/**
 * @struct User
 * @brief Representation of a user
 *
 * This structure provides a very simple model of a user reflecting the user
 * service API used in this Pact example.
 *
 * Fields:
 *
 * - id: Unique numeric identifier for the user
 * - name: User's display name (up to 255 characters + null terminator)
 * - created_on: ISO 8601 timestamp of when the user was created
 *
 * Example:
 * @code
 * struct User user = {
 *   .id = 123,
 *   .name = "John Doe",
 *   .created_on = "2025-11-17T10:30:00+00:00"
 * };
 * @endcode
 */
struct User {
  int id;
  char name[256];
  char created_on[64];
};

/** Maximum number of users the provider state can hold */
#define MAX_USERS 100

/**
 * @struct ProviderState
 * @brief Structure to manage provider state for testing.
 *
 * This structure maintains the in-memory state of the provider during
 * Pact verification. It acts as a simple "database" that can be manipulated
 * by provider state setup/teardown functions.
 *
 * The state is expected to be global and shared across all requests during a
 * verification run.
 *
 * @see get_global_provider_state() to access the global instance
 */
struct ProviderState {
  /** Array of users */
  struct User users[MAX_USERS];
  /** Number of users currently stored */
  int user_count;
};

/**
 * @brief Gets the global provider state instance.
 *
 * This function returns a pointer to the singleton ProviderState instance
 * that is shared across all requests and handlers. The global state persists
 * for the lifetime of the application but is cleared between verification
 * interactions.
 *
 * @return Pointer to the global ProviderState structure (never NULL)
 *
 * @note The returned pointer is always valid and points to static storage
 * @see provider_state_init() to initialize the state
 * @see provider_state_clear() to reset the state
 */
struct ProviderState *get_global_provider_state(void);

/**
 * Initializes the provider state.
 *
 * @param state Pointer to ProviderState structure to initialize
 */
void provider_state_init(struct ProviderState *state);

/**
 * Clears all users from the provider state.
 *
 * @param state Pointer to ProviderState structure to clear
 */
void provider_state_clear(struct ProviderState *state);

/**
 * Adds a user to the provider state.
 *
 * This function creates a new user with the specified ID and name,
 * and a creation timestamp set to the current time.
 *
 * @param state Pointer to ProviderState structure
 * @param id User ID
 * @param name User name
 * @return 0 on success, -1 on failure
 */
int provider_state_add_user(struct ProviderState *state, int id,
                            const char *name);

/**
 * Finds a user by ID in the provider state.
 *
 * @param state Pointer to ProviderState structure
 * @param id User ID to find
 * @return Pointer to User structure if found, NULL otherwise
 */
struct User *provider_state_find_user(struct ProviderState *state, int id);

/**
 * Removes a user by ID from the provider state.
 *
 * @param state Pointer to ProviderState structure
 * @param id User ID to remove
 * @return 0 on success, -1 if not found
 */
int provider_state_remove_user(struct ProviderState *state, int id);

/**
 * @brief Handles provider state setup via HTTP request.
 *
 * This function is called by handle_provider_state_change() when the Pact
 * verifier requests a state setup. It interprets the state name and parameters
 * to configure the provider's state appropriately.
 *
 * Supported states in this example:
 *
 * - "the user exists": Creates a user with the specified id and name from
 *   params
 * - "the user doesn't exist": Ensures the user with the specified id is removed
 * - Other states: Logged but no action taken
 *
 * The params string is a JSON object (or NULL) containing state-specific data.
 * For "the user exists", params might be: {"id": 123, "name": "Test User"}
 *
 * @param state_name The name of the state to set up (e.g., "the user exists")
 * @param params cJSON object containing state parameters (can be NULL)
 * @return 0 on success, -1 on failure
 *
 * @note Uses cJSON for parsing state parameters
 * @see handle_provider_state_change() for the HTTP endpoint that calls this
 * @see https://docs.pact.io/getting_started/provider_states
 */
int handle_provider_state_setup(const char *state_name, cJSON *params);

/**
 * @brief Handles provider state teardown via HTTP request.
 *
 * This function is called by handle_provider_state_change() when the Pact
 * verifier requests a state teardown after an interaction has been verified.
 * It cleans up any state that was set up to ensure the next interaction
 * starts with a clean slate.
 *
 * The current implementation simply clears all users from the provider state,
 * regardless of the state_name. This ensures complete isolation between
 * interactions.
 *
 * In a more complex system, you might:
 * - Delete specific test records from a database
 * - Reset mock service expectations
 * - Clear caches or message queues
 * - Restore snapshots of previous state
 *
 * @param state_name The name of the state to tear down (currently unused)
 * @return 0 on success
 *
 * @note Always succeeds in current implementation
 * @see provider_state_clear() for the actual cleanup logic
 */
int handle_provider_state_teardown(const char *state_name);

#endif // PROVIDER_STATE_H
