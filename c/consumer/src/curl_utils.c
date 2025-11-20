#include "curl_utils.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "logging.h"

size_t response_buffer_callback(void *data, size_t size, size_t nmemb,
                                void *clientp) {
  log_debug("response_buffer_callback called with size=%zu, nmemb=%zu", size,
            nmemb);
  size_t realsize = size * nmemb;
  struct ResponseBuffer *mem = (struct ResponseBuffer *)clientp;

  char *ptr = realloc(mem->buffer, mem->size + realsize + 1);
  if (!ptr) {
    // out of memory!
    log_warn("Failed to allocated memory for response buffer");
    return 0;
  }

  mem->buffer = ptr;
  memcpy(&(mem->buffer[mem->size]), data, realsize);
  mem->size += realsize;
  mem->buffer[mem->size] = 0;
  return realsize;
}

void response_buffer_init(struct ResponseBuffer *response) {
  log_debug("response_buffer_init called");
  response->buffer = NULL;
  response->size = 0;
  response->status_code = 0;
}

void response_buffer_free(struct ResponseBuffer *response) {
  log_debug("response_buffer_free called");
  if (response->buffer) {
    free(response->buffer);
    response->buffer = NULL;
  }
  response->size = 0;
}

static int curl_perform_common(const char *url, const char *method,
                               const char *body, struct curl_slist *headers,
                               struct ResponseBuffer *response) {
  log_debug("curl_perform_common called with url=%s, method=%s", url,
            method ? method : "GET");
  CURL *curl = curl_easy_init();
  if (!curl) {
    log_warn("Failed to initialize cURL");
    return 1;
  }

  response_buffer_init(response);
  curl_easy_setopt(curl, CURLOPT_URL, url);
  curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, response_buffer_callback);
  curl_easy_setopt(curl, CURLOPT_WRITEDATA, (void *)response);
  if (headers) {
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
  }

  if (method && strcmp(method, "POST") == 0) {
    curl_easy_setopt(curl, CURLOPT_POST, 1L);
    if (body) {
      curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body);
    }
  } else if (method && strcmp(method, "DELETE") == 0) {
    curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "DELETE");
  }

  CURLcode res = curl_easy_perform(curl);
  log_debug("cURL perform completed with result code %d", res);

  if (res == CURLE_OK && response) {
    long status = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &status);
    response->status_code = status;
  } else if (response) {
    response->status_code = 0;
  }
  curl_easy_cleanup(curl);
  return (res == CURLE_OK) ? 0 : res;
}

int curl_get(const char *url, struct curl_slist *headers,
             struct ResponseBuffer *response) {
  return curl_perform_common(url, NULL, NULL, headers, response);
}

int curl_post(const char *url, const char *body, struct curl_slist *headers,
              struct ResponseBuffer *response) {
  return curl_perform_common(url, "POST", body, headers, response);
}

int curl_delete(const char *url, struct curl_slist *headers,
                struct ResponseBuffer *response) {
  return curl_perform_common(url, "DELETE", NULL, headers, response);
}
