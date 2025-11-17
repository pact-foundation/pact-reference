/**
 * @file http_server.h
 * @brief Simple HTTP server wrapper around GNU libmicrohttpd
 *
 * This module provides a interface for creating HTTP servers backed by the GNU
 * libmicrohttpd (MHD) library.
 *
 * The server supports:
 *
 * - All standard HTTP methods (GET, POST, PUT, DELETE, etc.)
 * - Automatic handling of request body accumulation for POST/PUT requests
 * - Custom request handlers via callbacks
 * - Simple response generation with status codes and content types
 *
 * Basic usage:
 *
 * 1. Define an HttpHandler callback function to process requests
 * 2. Call http_server_start() with your handler
 * 3. The server runs in a background thread managed by libmicrohttpd
 * 4. Call http_server_stop() when done
 *
 * For more information about GNU libmicrohttpd, see:
 * https://www.gnu.org/software/libmicrohttpd/
 *
 * @see https://www.gnu.org/software/libmicrohttpd/tutorial.html
 * @see https://www.gnu.org/software/libmicrohttpd/manual/html_node/
 */

#ifndef HTTP_SERVER_H
#define HTTP_SERVER_H

#include <microhttpd.h>
#include <stddef.h>

/**
 * @struct HttpRequest
 * @brief Structure to hold HTTP request information passed to handlers.
 *
 * This structure contains all the essential information about an incoming
 * HTTP request. It is populated by the server and passed to the custom
 * HttpHandler callback for processing.
 *
 * @note The connection field is the underlying libmicrohttpd connection handle
 * and can be used for advanced operations like reading headers directly via
 * MHD_get_connection_values() if needed.
 */
struct HttpRequest {
  /** HTTP method (GET, POST, DELETE, etc.) */
  const char *method;
  /** Request URL/path */
  const char *url;
  /** Request body (for POST/PUT requests) */
  const char *body;
  /** Size of the request body */
  size_t body_size;
  /** MHD connection handle for advanced operations (see
   * MHD_get_connection_values) */
  struct MHD_Connection *connection;
};

/**
 * @struct HttpResponse
 * @brief Structure to hold HTTP response information.
 *
 * This structure is used by HttpHandler callbacks to specify the response
 * that should be sent back to the client. The handler fills in the fields
 * of this structure, and the server framework handles the actual transmission.
 *
 * The body is copied by libmicrohttpd, so it's safe to use stack-allocated
 * or temporary strings. If body is NULL, an empty response body is sent.
 *
 * @see http_send_response() for the internal function that processes this
 * structure
 */
struct HttpResponse {
  /** HTTP status code (200, 404, 500, etc.) */
  unsigned int status_code;
  /** Response body content */
  const char *body;
  /** Content-Type header value (e.g., "application/json", "text/plain") */
  const char *content_type;
};

/**
 * @typedef HttpHandler
 * @brief Callback function type for handling HTTP requests.
 *
 * This is the core callback interface for implementing custom request handling
 * logic. Your handler function receives a populated HttpRequest structure and
 * must fill in the HttpResponse structure with the appropriate response.
 *
 * Example implementation:
 * @code
 * int my_handler(const struct HttpRequest *request,
 *                struct HttpResponse *response) {
 *   if (strcmp(request->method, "GET") == 0) {
 *     response->status_code = 200;
 *     response->body = "{\"message\": \"Hello, World!\"}";
 *     response->content_type = "application/json";
 *     return MHD_YES;
 *   }
 *   response->status_code = 405;
 *   response->body = "Method not allowed";
 *   response->content_type = "text/plain";
 *   return MHD_NO;
 * }
 * @endcode
 *
 * @param request Pointer to HttpRequest structure with request details
 * (read-only)
 * @param response Pointer to HttpResponse structure to fill with response data
 * @return MHD_YES on success, MHD_NO on failure
 *
 * @note Returning MHD_NO indicates an error but the response will still be sent
 * @see struct HttpRequest
 * @see struct HttpResponse
 */
typedef enum MHD_Result (*HttpHandler)(const struct HttpRequest *request,
                                       struct HttpResponse *response);

/**
 * @struct HttpServerContext
 * @brief Context structure for HTTP server instance.
 *
 * This structure maintains the state of an HTTP server instance. It is
 * initialized by http_server_start() and should be treated as opaque by
 * the caller (i.e., don't modify fields directly after initialization).
 *
 * The daemon field is the core libmicrohttpd daemon object that manages
 * the server's listening socket and worker threads. The server uses
 * MHD_USE_SELECT_INTERNALLY mode, which means libmicrohttpd creates its
 * own thread for handling connections.
 *
 * @note Always call http_server_stop() before the context goes out of scope
 * to ensure that any pending responses are completed and for proper cleanup of
 * resources and thread termination.
 *
 * @see http_server_start()
 * @see http_server_stop()
 */
struct HttpServerContext {
  /** The MHD daemon instance */
  struct MHD_Daemon *daemon;
  /** Port number the server is listening on */
  unsigned int port;
  /** Custom request handler callback */
  HttpHandler handler;
  /** User data to pass to handler (can be used for application state) */
  void *user_data;
};

/**
 * @brief Starts an HTTP server on the specified port.
 *
 * This function initializes and starts an HTTP server using libmicrohttpd.
 * The server runs in its own thread (MHD_USE_SELECT_INTERNALLY mode), so this
 * function returns immediately after starting the server. The handler callback
 * will be invoked for each incoming HTTP request.
 *
 * The server listens on all available network interfaces (0.0.0.0). For
 * production use, you may want to bind to a specific interface by modifying
 * the MHD_start_daemon() call in the implementation.
 *
 * @param context Pointer to HttpServerContext to initialize (must not be NULL)
 * @param port Port number to listen on (typically 1024-65535 for unprivileged
 *             users)
 * @param handler Callback function to handle requests (must not be NULL)
 * @param user_data Optional user data to pass to handler (can be NULL)
 * @return 0 on success, -1 on failure (e.g., port already in use)
 *
 * @note The context must remain valid for the lifetime of the server
 * @note Call http_server_stop() to shut down the server gracefully
 *
 * @see http_server_stop()
 * @see HttpHandler
 */
int http_server_start(struct HttpServerContext *context, unsigned int port,
                      HttpHandler handler, void *user_data);

/**
 * @brief Stops the HTTP server gracefully.
 *
 * This function stops the HTTP server and waits for all active connections
 * to complete. It blocks until the libmicrohttpd daemon thread has fully
 * terminated and all resources have been cleaned up.
 *
 * After calling this function, the daemon field in the context is set to NULL,
 * making it safe to call this function multiple times on the same context.
 *
 * @param context Pointer to HttpServerContext to stop (can be NULL, no-op if
 *                daemon is NULL)
 *
 * @note This function blocks until all active connections are closed
 * @note Safe to call multiple times on the same context
 *
 * @see http_server_start()
 */
void http_server_stop(struct HttpServerContext *context);

/**
 * @brief Helper function to send an HTTP response.
 *
 * This is a utility function that wraps the libmicrohttpd response creation
 * and queuing process. It creates a response from the provided body string,
 * adds the Content-Type header if specified, and queues it for transmission.
 *
 * The body string is copied by libmicrohttpd (MHD_RESPMEM_MUST_COPY), so it's
 * safe to pass stack-allocated or temporary strings. If body is NULL, an empty
 * response is sent.
 *
 * @param connection MHD connection handle (from HttpRequest.connection)
 * @param response Pointer to HttpResponse structure with response details
 * @return MHD_YES on success, MHD_NO on failure
 *
 * @note The response is queued; actual transmission happens asynchronously
 */
int http_send_response(struct MHD_Connection *connection,
                       const struct HttpResponse *response);

#endif // HTTP_SERVER_H
