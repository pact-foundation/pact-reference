
#include "pact.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#ifdef CURL
#include <curl/curl.h>
#endif

#define ERROR_MSG_LEN 256

int main(void) {
    int status = 0;

    /*=======================================================================
     * Begin logger setup.
     *---------------------------------------------------------------------*/

    pactffi_logger_init();
    
    /*=======================================================================
     * Attach a sink pointing info-level output to stdout.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("stdout", LevelFilter_Info);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Attach another sink pointing debug output to a log file.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("file ./pm_ffi.log", LevelFilter_Debug);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Attach another sink to collect log events into a memory buffer.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("buffer", LevelFilter_Trace);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Apply the logger, completing logging setup.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_apply();
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    pactffi_log_message("example C", "debug", "This is a debug message");
    pactffi_log_message("example C", "info", "This is an info message");
    pactffi_log_message("example C", "error", "This is an error message");
    pactffi_log_message("example C", "trace", "This is a trace message");

    const char *logs = pactffi_fetch_log_buffer(NULL);
    if (logs == NULL) {
        printf("Could not get the buffered logs\n");
        return EXIT_FAILURE;
    }

    printf("---- Logs from buffer ----\n");
    printf("%s", logs);
    printf("--------------------------\n");

    int len = strlen(logs);
    if (len == 0) {
        printf("Buffered logs are empty\n");
        return EXIT_FAILURE;
    }
    pactffi_string_delete(logs);

    /**
    * Test the logs from the mock server
    */
    #ifdef CURL
        PactHandle pact = pactffi_new_pact("logging-test", "logging-test");
        int port = pactffi_create_mock_server_for_transport(pact, "127.0.0.1", 0, "http", NULL);

        CURL *curl = curl_easy_init();
        if (curl) {
            char url[32];
            sprintf(url, "http://localhost:%d/", port);
            printf("Executing request against %s\n", url);
            curl_easy_setopt(curl, CURLOPT_URL, url);

            CURLcode res = curl_easy_perform(curl);
            curl_easy_cleanup(curl);

            const char *mockserver_logs =  pactffi_mock_server_logs(port);
            printf("---- Logs from mock server ----\n");
            printf("%s", mockserver_logs);
            printf("--------------------------\n");

            int mockserver_logs_len = strlen(mockserver_logs);
            pactffi_cleanup_mock_server(port);
            if (mockserver_logs_len == 0) {
                printf("Mock server logs are empty\n");
                return EXIT_FAILURE;
            }
        } else {
            printf("CURL is not available\n");
            return EXIT_FAILURE;
        }
    #endif

    return EXIT_SUCCESS;
}
