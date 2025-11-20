/**
 * @file provider_state.c
 * @brief Implementation of provider state management for Pact verification
 *
 * This file implements the provider state management system that allows the
 * Pact verifier to control the provider's application state during contract
 * verification.
 *
 * In this example, we maintain an in-memory list of users as the provider
 * state, though in a real application this would typically involve setting up
 * a test database, configuring mocks, or preparing other application state.
 *
 * The Pact verifier will call a special endpoint to set up and tear down
 * provider states before and after each interaction. In some cases, the state
 * may include parameters that influence how the state is configured.
 *
 * @see provider_state.h for the public interface
 */

#include "provider/provider_state.h"
#include "logging.h"

#include <cjson/cJSON.h>
#include <pact.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/**
 * Global provider state instance.
 *
 * This static variable holds the provider state for the entire application.
 * It is initialized once at startup and manipulated by state setup/teardown
 * functions during verification.
 */
static struct ProviderState global_state;

struct ProviderState *get_global_provider_state(void) { return &global_state; }

void provider_state_init(struct ProviderState *state) {
  log_debug("Initializing provider state");
  state->user_count = 0;
  memset(state->users, 0, sizeof(state->users));
}

void provider_state_clear(struct ProviderState *state) {
  log_debug("Clearing provider state");
  state->user_count = 0;
  memset(state->users, 0, sizeof(state->users));
}

int provider_state_add_user(struct ProviderState *state, int id,
                            const char *name) {
  log_debug("Adding user to provider state: id=%d, name=%s", id, name);

  if (state->user_count >= MAX_USERS) {
    log_err("Provider state user limit reached");
    return -1;
  }

  struct User *user = &state->users[state->user_count++];
  user->id = id;
  strncpy(user->name, name, sizeof(user->name) - 1);
  user->name[sizeof(user->name) - 1] = '\0';

  // Generate created_on timestamp in ISO 8601 format
  time_t now = time(NULL);
  struct tm *tm_info = gmtime(&now);
  strftime(user->created_on, sizeof(user->created_on),
           "%Y-%m-%dT%H:%M:%S+00:00", tm_info);

  return 0;
}

struct User *provider_state_find_user(struct ProviderState *state, int id) {
  log_debug("Finding user in provider state: id=%d", id);

  for (int i = 0; i < state->user_count; i++) {
    if (state->users[i].id == id) {
      return &state->users[i];
    }
  }
  return NULL;
}

int provider_state_remove_user(struct ProviderState *state, int id) {
  log_debug("Removing user from provider state: id=%d", id);

  for (int i = 0; i < state->user_count; i++) {
    if (state->users[i].id == id) {
      // Shift remaining users down
      for (int j = i; j < state->user_count - 1; j++) {
        state->users[j] = state->users[j + 1];
      }
      state->user_count--;
      return 0;
    }
  }
  return -1;
}

int handle_provider_state_setup(const char *state_name, cJSON *params) {
  log_info("Setting up provider state: %s", state_name);

  // Handle the "user exists" state by creating a user with specified parameters
  if (strcmp(state_name, "the user exists") == 0) {
    // Parse id and name from params using cJSON
    int id = 0;
    const char *name = NULL;

    if (params) {
      cJSON *id_item = cJSON_GetObjectItem(params, "id");
      if (id_item && cJSON_IsNumber(id_item)) {
        id = id_item->valueint;
      }

      cJSON *name_item = cJSON_GetObjectItem(params, "name");
      if (name_item && cJSON_IsString(name_item) && name_item->valuestring) {
        name = name_item->valuestring;
      }
    }

    if (id > 0 && name && name[0] != '\0') {
      provider_state_add_user(&global_state, id, name);
      log_info("Added user for state: id=%d, name=%s", id, name);
    } else {
      log_warn(
          "Missing or invalid id/name parameters for 'the user exists' state");
    }
  } else if (strcmp(state_name, "the user doesn't exist") == 0) {
    // Handle the "user doesn't exist" state by ensuring the user is removed
    int id = 0;
    if (params) {
      cJSON *id_item = cJSON_GetObjectItem(params, "id");
      if (id_item && cJSON_IsNumber(id_item)) {
        id = id_item->valueint;
      }
    }

    if (id > 0) {
      provider_state_remove_user(&global_state, id);
      log_info("Ensured user doesn't exist: id=%d", id);
    }
  } else {
    log_debug("No setup needed for state: %s", state_name);
  }

  return 0;
}

int handle_provider_state_teardown(const char *state_name) {
  log_debug("Tearing down provider state: %s", state_name);

  // Clear state after each interaction
  provider_state_clear(&global_state);
  return 0;
}
