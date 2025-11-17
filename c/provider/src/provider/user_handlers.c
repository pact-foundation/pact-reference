/**
 * @file user_handlers.c
 * @brief Implementation of HTTP request handlers for the provider API
 *
 * This file implements the request handlers for a simple user management API
 * that serves as the provider in a Pact contract verification scenario.
 *
 * @see user_handlers.h for the public interface
 * @see provider_state.h for state management
 */

#include "provider/user_handlers.h"
#include "logging.h"
#include "provider/provider_state.h"

#include <cjson/cJSON.h>
#include <ctype.h>
#include <microhttpd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/**
 * Static response buffer for formatting JSON responses.
 *
 * Since the verifier runs sequentially, we can safely reuse this buffer
 * across requests without thread-safety concerns.
 */
static char response_buffer[4096];

/**
 * @brief Parses user ID from URL path like "/users/123".
 *
 * This helper function extracts the numeric user ID from a URL path.
 * It finds the last '/' character and attempts to parse the following
 * string as an integer.
 *
 * Examples:
 * - "/users/123" → 123
 * - "/users/456" → 456
 * - "/users/abc" → -1 (invalid, contains non-digits)
 * - "/users/" → -1 (invalid, no ID after slash)
 *
 * @param url The URL path string to parse
 * @return The parsed user ID on success, -1 on failure
 *
 * @note This function validates that all characters after the last '/' are
 * digits
 */
static int parse_user_id_from_path(const char *url) {
  const char *id_start = strrchr(url, '/');
  if (!id_start)
    return -1;

  id_start++; // Skip the '/'

  // Check if it's a valid number
  for (const char *p = id_start; *p; p++) {
    if (!isdigit(*p))
      return -1;
  }

  return atoi(id_start);
}

/**
 * @brief Generates a JSON response for a user.
 *
 * This helper function formats a User structure into a JSON string using cJSON.
 *
 * @param buffer Output buffer to write JSON string to
 * @param buffer_size Size of the output buffer
 * @param user Pointer to User structure to format
 *
 * @note Uses cJSON for proper JSON encoding (handles escaping, etc.)
 * @note The output is null-terminated and truncated if it exceeds buffer_size
 */
static void format_user_json(char *buffer, size_t buffer_size,
                             const struct User *user) {
  cJSON *json = cJSON_CreateObject();
  cJSON_AddNumberToObject(json, "id", user->id);
  cJSON_AddStringToObject(json, "name", user->name);
  cJSON_AddStringToObject(json, "created_on", user->created_on);

  char *json_str = cJSON_PrintUnformatted(json);
  if (json_str) {
    strncpy(buffer, json_str, buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    cJSON_free(json_str);
  }
  cJSON_Delete(json);
}

enum MHD_Result user_request_router(const struct HttpRequest *request,
                                    struct HttpResponse *response) {
  log_debug("Routing request: %s %s", request->method, request->url);

  // Route based on method and URL pattern
  if (strcmp(request->method, "GET") == 0 &&
      strncmp(request->url, "/users/", 7) == 0) {
    return handle_get_user(request, response);
  } else if (strcmp(request->method, "POST") == 0 &&
             strcmp(request->url, "/users") == 0) {
    return handle_create_user(request, response);
  } else if (strcmp(request->method, "DELETE") == 0 &&
             strncmp(request->url, "/users/", 7) == 0) {
    return handle_delete_user(request, response);
  } else if (strcmp(request->method, "POST") == 0 &&
             strcmp(request->url, "/__pact/provider-state") == 0) {
    return handle_provider_state_change(request, response);
  } else {
    log_warn("Unknown route: %s %s", request->method, request->url);
    response->status_code = 404;
    response->body = "{\"detail\":\"Not found\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }
}

enum MHD_Result handle_get_user(const struct HttpRequest *request,
                                struct HttpResponse *response) {
  int user_id = parse_user_id_from_path(request->url);
  log_info("GET request for user ID: %d", user_id);

  if (user_id < 0) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Invalid user ID\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  struct ProviderState *state = get_global_provider_state();
  struct User *user = provider_state_find_user(state, user_id);

  if (!user) {
    log_info("User not found: %d", user_id);
    response->status_code = 404;
    snprintf(response_buffer, sizeof(response_buffer),
             "{\"detail\":\"User not found\"}");
    response->body = response_buffer;
    response->content_type = "application/json";
    return MHD_YES;
  }

  log_info("User found: id=%d, name=%s", user->id, user->name);
  response->status_code = 200;
  format_user_json(response_buffer, sizeof(response_buffer), user);
  response->body = response_buffer;
  response->content_type = "application/json";
  return MHD_YES;
}

enum MHD_Result handle_create_user(const struct HttpRequest *request,
                                   struct HttpResponse *response) {
  log_info("POST request to create user");

  if (!request->body || request->body_size == 0) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Missing request body\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  // Parse JSON request body using cJSON
  cJSON *json = cJSON_Parse(request->body);
  if (!json) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Invalid JSON\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  // Extract the "name" field
  cJSON *name_item = cJSON_GetObjectItem(json, "name");
  if (!name_item || !cJSON_IsString(name_item) || !name_item->valuestring ||
      name_item->valuestring[0] == '\0') {
    cJSON_Delete(json);
    response->status_code = 400;
    response->body = "{\"detail\":\"Missing or invalid name field\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  const char *name = name_item->valuestring;

  // Generate a new user ID (simple incrementing)
  struct ProviderState *state = get_global_provider_state();
  int new_id = 1000 + state->user_count;

  if (provider_state_add_user(state, new_id, name) != 0) {
    cJSON_Delete(json);
    response->status_code = 500;
    response->body = "{\"detail\":\"Failed to create user\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  struct User *user = provider_state_find_user(state, new_id);
  log_info("User created: id=%d, name=%s", new_id, name);

  cJSON_Delete(json);

  response->status_code = 201;
  format_user_json(response_buffer, sizeof(response_buffer), user);
  response->body = response_buffer;
  response->content_type = "application/json";
  return MHD_YES;
}

enum MHD_Result handle_delete_user(const struct HttpRequest *request,
                                   struct HttpResponse *response) {
  int user_id = parse_user_id_from_path(request->url);
  log_info("DELETE request for user ID: %d", user_id);

  if (user_id < 0) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Invalid user ID\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  struct ProviderState *state = get_global_provider_state();

  if (provider_state_remove_user(state, user_id) != 0) {
    log_info("User not found for deletion: %d", user_id);
    response->status_code = 404;
    response->body = "{\"detail\":\"User not found\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  log_info("User deleted: %d", user_id);
  response->status_code = 204;
  response->body = "";
  response->content_type = "application/json";
  return MHD_YES;
}

enum MHD_Result handle_provider_state_change(const struct HttpRequest *request,
                                             struct HttpResponse *response) {
  log_info("POST /__pact/provider-state - Provider state change request");

  if (!request->body || request->body_size == 0) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Missing request body\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  // Parse the state change request using cJSON
  // Expected JSON format: {"state": "state name", "params": {...}, "action":
  // "setup"}
  cJSON *json = cJSON_Parse(request->body);
  if (!json) {
    response->status_code = 400;
    response->body = "{\"detail\":\"Invalid JSON\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  // Extract state name (if present)
  const char *state_name = "";
  cJSON *state_item = cJSON_GetObjectItem(json, "state");
  if (state_item && cJSON_IsString(state_item) && state_item->valuestring) {
    state_name = state_item->valuestring;
  }

  // Extract action (required)
  cJSON *action_item = cJSON_GetObjectItem(json, "action");
  if (!action_item || !cJSON_IsString(action_item) ||
      !action_item->valuestring) {
    cJSON_Delete(json);
    response->status_code = 400;
    response->body = "{\"detail\":\"Missing or invalid action field\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }
  const char *action = action_item->valuestring;

  // Get params object (may be NULL)
  cJSON *params = cJSON_GetObjectItem(json, "params");

  log_debug("State change: state=%s, action=%s", state_name, action);

  // Handle empty state (for interactions with no provider state)
  if (state_name[0] == '\0') {
    log_info("Empty provider state - no setup needed");
    cJSON_Delete(json);
    response->status_code = 200;
    response->body = "{\"result\":\"success\"}";
    response->content_type = "application/json";
    return MHD_YES;
  }

  // Handle setup or teardown
  if (strcmp(action, "setup") == 0) {
    handle_provider_state_setup(state_name, params);
  } else if (strcmp(action, "teardown") == 0) {
    handle_provider_state_teardown(state_name);
  } else {
    log_warn("Unknown action for provider state change: %s", action);
    response->status_code = 400;
    response->body = "{\"detail\":\"Unknown action\"}";
    response->content_type = "application/json";
    cJSON_Delete(json);
    return MHD_YES;
  }

  cJSON_Delete(json);

  response->status_code = 200;
  response->body = "{\"result\":\"success\"}";
  response->content_type = "application/json";
  return MHD_YES;
}
