#ifndef CURL_UTILS_H
#define CURL_UTILS_H

#include <curl/curl.h>
#include <stddef.h>

/**
 * Structure to hold response data
 *
 * The structure should be initialized with `response_buffer_init` and freed
 * with `response_buffer_free` when no longer needed.
 */
struct ResponseBuffer {
  /** Pointer to the response body buffer (allocated, must be freed by caller)
   */
  char *buffer;
  /** Size of the response body in bytes */
  size_t size;
  /** HTTP status code returned by the server (e.g., 200, 404) */
  long status_code;
};

/**
 * Initializes a ResponseBuffer structure.
 *
 * This function sets the initial state of the ResponseBuffer, allocating no
 * memory. The buffer pointer is set to NULL, size to 0, and status_code to 0.
 *
 * @param response Pointer to the ResponseBuffer structure to initialize.
 *
 * Example usage:
 *   struct ResponseBuffer resp;
 *   response_buffer_init(&resp);
 */
void response_buffer_init(struct ResponseBuffer *response);

/**
 * Frees resources associated with a ResponseBuffer structure.
 *
 * This function frees the memory allocated for the response buffer, if any,
 * and resets the size to 0. The buffer pointer is set to NULL.
 *
 * @param response Pointer to the ResponseBuffer structure to free.
 *
 * Example usage:
 *   struct ResponseBuffer resp;
 *   response_buffer_init(&resp);
 *   // ... perform cURL operations that fill resp ...
 *   response_buffer_free(&resp);
 */
void response_buffer_free(struct ResponseBuffer *response);

/**
 * Callback function to capture response data from a cURL request.
 *
 * This function is called by libcurl whenever data is received that needs to be
 * saved. It may be called multiple times for a single request.
 *
 * @param data      Pointer to the received data.
 * @param size      Size of each data element (usually 1).
 * @param nmemb     Number of data elements.
 * @param clientp   Pointer to a user-defined data structure (should be a
 *                  ResponseBuffer*).
 *
 * @return The number of bytes actually handled. Returning a value different
 * from size*nmemb will signal an error to cURL.
 *
 * Example usage:
 *   struct ResponseBuffer resp = {0};
 *   curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, response_buffer_callback);
 *   curl_easy_setopt(curl, CURLOPT_WRITEDATA, (void *)&resp);
 */
size_t response_buffer_callback(void *data, size_t size, size_t nmemb,
                                void *clientp);

/**
 * Performs a HTTP GET request to the specified URL.
 *
 * @param url       The URL to send the GET request to.
 * @param headers   Optional list of headers (as a struct curl_slist*). Pass
 *                  NULL for no headers.
 * @param response  Pointer to a ResponseBuffer struct to receive the response
 *                  body and HTTP status code. The buffer will be allocated and
 *                  must be freed by the caller using free().
 *
 * @return 0 on success, non-zero on failure (see cURL error codes).
 *
 * Example usage:
 *   struct ResponseBuffer resp = {0};
 *   struct curl_slist *headers = NULL;
 *   headers = curl_slist_append(headers, "Authorization: Bearer token");
 *   int rc = curl_get("https://example.com", headers, &resp);
 *   if (rc == 0) {
 *     printf("Status: %ld\n", resp.status_code);
 *     printf("%s", resp.buffer);
 *   }
 *   free(resp.buffer);
 *   curl_slist_free_all(headers);
 */
int curl_get(const char *url, struct curl_slist *headers,
             struct ResponseBuffer *response);

/**
 * Performs a HTTP POST request to the specified URL with a request body.
 *
 * @param url       The URL to send the POST request to.
 * @param body      The request body to send (as a null-terminated string). Pass
 *                  NULL for no body.
 * @param headers   Optional list of headers (as a struct curl_slist*). Pass
 *                  NULL for no headers.
 * @param response  Pointer to a ResponseBuffer struct to receive the response
 *                  body and HTTP status code. The buffer will be allocated and
 *                  must be freed by the caller using free().
 *
 * @return 0 on success, non-zero on failure (see cURL error codes).
 *
 * Example usage:
 *   struct ResponseBuffer resp = {0};
 *   struct curl_slist *headers = NULL;
 *   headers = curl_slist_append(headers, "Content-Type: application/json");
 *   int rc = curl_post(
 *     "https://example.com/api",
 *     "{\"foo\":\"bar\"}",
 *     headers,
 *     &resp
 *   );
 *   if (rc == 0) { printf("Status: %ld\n", resp.status_code);
 *     printf("%s", resp.buffer);
 *   }
 *   free(resp.buffer);
 *   curl_slist_free_all(headers);
 */
int curl_post(const char *url, const char *body, struct curl_slist *headers,
              struct ResponseBuffer *response);

/**
 * Performs a HTTP DELETE request to the specified URL.
 *
 * @param url      The URL to send the DELETE request to.
 * @param headers  Optional list of headers (as a struct curl_slist*). Pass NULL
 *                 for no headers.
 * @param response Pointer to a ResponseBuffer struct to receive the response
 *                 body and HTTP status code. The buffer will be allocated and
 *                 must be freed by the caller using free().
 *
 * @return 0 on success, non-zero on failure (see cURL error codes).
 *
 * Example usage:
 *   struct ResponseBuffer resp = {0};
 *   int rc = curl_delete("https://example.com/resource/1", NULL, &resp);
 *   if (rc == 0) {
 *     printf("Status: %ld\n", resp.status_code);
 *     printf("%s", resp.buffer);
 *   }
 *   free(resp.buffer);
 */
int curl_delete(const char *url, struct curl_slist *headers,
                struct ResponseBuffer *response);

#endif // CURL_UTILS_H
