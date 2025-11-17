/**
 * @file http_server.c
 * @brief Implementation of HTTP server using GNU libmicrohttpd
 *
 * This file implements a simple HTTP server wrapper around libmicrohttpd (MHD)
 * which provides a callback-based architecture for handling HTTP requests. This
 * specifically is designed to handle the request lifecycle.
 *
 * Key implementation details:
 *
 * 1. Request Lifecycle:
 *    - libmicrohttpd calls request_handler() multiple times for each request
 *    - First call: We set up connection-specific state (*ptr)
 *    - Subsequent calls: We accumulate POST/PUT data if present
 *    - Final call: We process the complete request and send response
 *
 * 2. POST Data Handling:
 *    - POST/PUT request bodies may arrive in chunks (upload_data)
 *    - We use a PostContext structure to accumulate these chunks
 *    - The context persists across callback invocations via the *ptr parameter
 *
 * 3. Threading Model:
 *    - Server uses MHD_USE_SELECT_INTERNALLY, meaning libmicrohttpd creates
 *      its own thread for handling requests
 *    - The handler callback may be invoked concurrently for different requests
 *    - Ensure your HttpHandler implementation is thread-safe if it accesses
 *      shared state
 *
 * For more details on the libmicrohttpd callback model, see:
 * https://www.gnu.org/software/libmicrohttpd/tutorial.html#Exploring-requests
 *
 * @see http_server.h
 */

#include "http_server.h"
#include "logging.h"

#include <microhttpd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @struct PostContext
 * @brief Internal structure to accumulate POST/PUT request data across
 * callbacks.
 *
 * libmicrohttpd may invoke the request handler callback multiple times for
 * a single POST/PUT request, providing chunks of the request body each time.
 * This structure accumulates those chunks into a single contiguous buffer.
 *
 * The context is allocated on the first callback invocation for POST/PUT
 * requests and freed after the final callback when the request is complete.
 */
struct PostContext {
  /** Accumulated request body data */
  char *data;
  /** Current size of accumulated data in bytes */
  size_t size;
};

/**
 * Dummy value to indicate GET/DELETE requests in *ptr
 */
static int dummy;

/**
 * @brief MHD callback to handle incoming HTTP requests.
 *
 * This is the core request handling function registered with libmicrohttpd.
 * It implements the complex callback protocol required by libmicrohttpd,
 * which involves multiple invocations per request:
 *
 * Call sequence for GET request:
 *
 * 1. First call: *ptr is NULL, we initialize it and return MHD_YES
 * 2. Second call: *upload_data_size is 0, we process the request
 *
 * Call sequence for POST/PUT request:
 *
 * 1. First call: *ptr is NULL, we allocate PostContext and return MHD_YES
 * 2. Middle calls: *upload_data_size > 0, we accumulate data, return MHD_YES
 * 3. Final call: *upload_data_size is 0, we process complete request
 *
 * The *ptr parameter is crucial - it persists across calls for the same
 * request, allowing us to maintain state (like accumulated POST data) between
 * invocations. It must be properly allocated and freed to avoid memory leaks.
 *
 * @param cls User context (HttpServerContext pointer)
 * @param conn MHD connection handle for this request
 * @param url Requested URL/path (e.g., "/api/users")
 * @param method HTTP method (e.g., "GET", "POST", "DELETE")
 * @param version HTTP version string (e.g., "HTTP/1.1")
 * @param upload_data Pointer to chunk of upload data (for POST/PUT)
 * @param upload_data_size Pointer to size of upload_data (set to 0 when done)
 * @param ptr Persistent pointer across calls for this request
 * @return MHD_YES to continue processing, MHD_NO on error
 *
 * @note This function may be called from multiple threads concurrently
 * @see
 * https://www.gnu.org/software/libmicrohttpd/manual/html_node/microhttpd_002dcb.html
 */
static enum MHD_Result request_handler(void *cls, struct MHD_Connection *conn,
                                       const char *url, const char *method,
                                       const char *version,
                                       const char *upload_data,
                                       size_t *upload_data_size, void **ptr) {
  struct HttpServerContext *context = (struct HttpServerContext *)cls;

  if (*ptr == NULL) {
    log_debug("New request: %s %s", method, url);

    if (strcmp(method, "POST") == 0 || strcmp(method, "PUT") == 0) {
      // Create the PostContext to accumulate upload data
      struct PostContext *post_ctx = calloc(1, sizeof(struct PostContext));
      if (!post_ctx) {
        log_warn("Failed to allocate memory for PostContext");
        return MHD_NO;
      }
      *ptr = post_ctx;
    } else {
      // No need to accumulate data, use dummy marker
      *ptr = &dummy;
    }

    return MHD_YES;
  }

  if (*upload_data_size > 0) {
    // Accumulate POST/PUT data chunks. We can assume *ptr is a valid
    // PostContext here as it should have been set in the first call.
    struct PostContext *post_ctx = (struct PostContext *)*ptr;
    char *new_data =
        realloc(post_ctx->data, post_ctx->size + *upload_data_size + 1);
    if (!new_data) {
      log_warn("Failed to allocate memory for upload data");
      return MHD_NO;
    }
    post_ctx->data = new_data;
    memcpy(post_ctx->data + post_ctx->size, upload_data, *upload_data_size);
    post_ctx->size += *upload_data_size;
    post_ctx->data[post_ctx->size] = '\0'; // Null-terminate for safety

    *upload_data_size = 0; // Mark this chunk as consumed
    return MHD_YES;        // Request more data if available
  }

  // At this point, we are on the final call for this request, and we build the
  // final HttpRequest structure to pass to the user's handler. We also prepare
  // a default HttpResponse in case of an error.
  struct PostContext *post_ctx =
      (struct PostContext *)*ptr; // Note: may be dummy
  struct HttpRequest request;
  struct HttpResponse response = {.status_code = 500,
                                  .body = "Internal Server Error",
                                  .content_type = "text/plain"};
  if (*ptr != &dummy) {
    log_debug("Completed receiving data (%zu bytes) for %s %s", post_ctx->size,
              method, url);
    request = (struct HttpRequest){
        .method = method,
        .url = url,
        .body = post_ctx->data,
        .body_size = post_ctx->size,
        .connection = conn,
    };
  } else {
    // No body data for GET/DELETE requests
    log_debug("Processing %s %s with no body", method, url);
    request = (struct HttpRequest){
        .method = method,
        .url = url,
        .body = NULL,
        .body_size = 0,
        .connection = conn,
    };
  }

  if (context->handler) {
    context->handler(&request, &response);
  }

  int ret = http_send_response(conn, &response);

  // Clean up POST context if allocated (but not if it's just our dummy marker)
  if (post_ctx && post_ctx != (struct PostContext *)&dummy) {
    free(post_ctx->data);
    free(post_ctx);
  }

  return ret;
}

int http_server_start(struct HttpServerContext *context, unsigned int port,
                      HttpHandler handler, void *user_data) {
  log_info("Starting HTTP server on port %u", port);

  context->port = port;
  context->handler = handler;
  context->user_data = user_data;

  context->daemon =
      MHD_start_daemon(MHD_USE_SELECT_INTERNALLY, port, NULL, NULL,
                       &request_handler, context, MHD_OPTION_END);

  if (context->daemon == NULL) {
    log_err("Failed to start HTTP server on port %u", port);
    return -1;
  }

  log_info("HTTP server started successfully on port %u", port);
  return 0;
}

void http_server_stop(struct HttpServerContext *context) {
  if (context->daemon) {
    log_info("Stopping HTTP server");
    MHD_stop_daemon(context->daemon);
    context->daemon = NULL;
  }
}

int http_send_response(struct MHD_Connection *connection,
                       const struct HttpResponse *response) {
  struct MHD_Response *mhd_response;
  int ret;

  size_t body_len = response->body ? strlen(response->body) : 0;

  mhd_response = MHD_create_response_from_buffer(
      body_len, (void *)response->body, MHD_RESPMEM_MUST_COPY);
  if (!mhd_response) {
    log_warn("Failed to create HTTP response");
    return MHD_NO;
  }

  if (response->content_type) {
    MHD_add_response_header(mhd_response, "Content-Type",
                            response->content_type);
  }

  ret = MHD_queue_response(connection, response->status_code, mhd_response);
  MHD_destroy_response(mhd_response);

  return ret;
}
