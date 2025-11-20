/**
 * @file logging.c
 * @brief Implementation of simple logging utilities
 *
 * This file implements a lightweight logging system. It provides formatted
 * output with file/line information and conditional debug logging based on
 * environment variables.
 *
 * Implementation details:
 *
 * 1. Variadic Functions:
 *    The logging functions use variadic arguments (va_list, va_start, va_end)
 *    to accept printf-style format strings with any number of arguments.
 *
 * 2. State Machine for VERBOSE:
 *    Instead of calling getenv("VERBOSE") on every log_debug() call, we use
 *    a static state machine that caches the result after the first check.
 *    This improves performance significantly for applications with many debug
 *    logs.
 *
 * 3. stderr vs stdout:
 *    All logging goes to stderr, which is the standard practice for diagnostic
 *    output. This keeps logs separate from normal program output on stdout,
 *    allowing proper redirection (e.g., `program > output.txt 2> errors.txt`).
 *
 * @see logging.h for the public interface and usage examples
 */

#include "logging.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Internal function to print a formatted log message to stderr.
 *
 * This function handles the actual output of log messages with formatting.
 * It prepends each message with [LEVEL] file:line for debugging context,
 * then formats and prints the user's message using vfprintf.
 *
 * Output format: [LEVEL] file:line message
 * Example: [INFO] main.c:42 Server started on port 8080
 *
 * @param level The log level string (e.g., "DEBUG", "INFO", "WARN", "ERROR")
 * @param file The source file name (__FILE__)
 * @param line The source line number (__LINE__)
 * @param fmt The printf-style format string
 * @param ... Additional arguments for the format string
 */
void _log_internal(const char *level, const char *file, int line,
                   const char *fmt, ...) {
  fprintf(stderr, "[%s] %s:%d ", level, file, line);
  va_list args;
  va_start(args, fmt);
  vfprintf(stderr, fmt, args);
  va_end(args);
  fprintf(stderr, "\n");
}

/**
 * State machine values for caching the VERBOSE environment variable check.
 * This avoids repeated calls to getenv() which can be relatively expensive.
 */
enum { LOGGING_VERBOSE_UNKNOWN, LOGGING_VERBOSE_ON, LOGGING_VERBOSE_OFF };

/**
 * Static variable to cache the verbose logging state.
 * Starts as UNKNOWN and is set to ON or OFF after the first check.
 */
static int logging_verbose_state = LOGGING_VERBOSE_UNKNOWN;

/**
 * @brief Checks if verbose debug logging is enabled via the VERBOSE environment
 * variable.
 *
 * This function implements a simple state machine to cache the result of
 * checking the VERBOSE environment variable:
 *
 * - UNKNOWN: First call, need to check getenv()
 * - ON: VERBOSE is set and non-empty
 * - OFF: VERBOSE is not set or empty
 *
 * Once the state transitions from UNKNOWN to ON/OFF, subsequent calls
 * return the cached result without calling getenv() again. This optimization
 * is important for debug-heavy code that might call log_debug() frequently.
 *
 * To enable debug logging, set the VERBOSE environment variable before
 * running the program:
 *
 * @code
 * export VERBOSE=1
 * ./provider
 * @endcode
 *
 * @return true if VERBOSE is set and not empty, false otherwise
 *
 * @note The state persists for the program lifetime; changes to the
 *       environment variable after startup are not detected
 */
bool _is_verbose_enabled() {
  if (logging_verbose_state == LOGGING_VERBOSE_UNKNOWN) {
    const char *env = getenv("VERBOSE");
    logging_verbose_state =
        (env && env[0] != '\0') ? LOGGING_VERBOSE_ON : LOGGING_VERBOSE_OFF;
  }
  return logging_verbose_state == LOGGING_VERBOSE_ON;
}

/**
 * @brief Draws a boxed message to the console for visual emphasis.
 *
 * This function takes a printf-style format string and prints it within
 * a box made of Unicode box-drawing characters. The box width
 * adjusts based on the message length, with a minimum width of 60 characters.
 *
 * This is useful for highlighting important information like:
 *
 * - Application startup messages
 * - Test result summaries
 * - Section headers in output
 *
 * Example output:
 *
 * @code
 * ┌────────────────────────────────────────────────────────────────┐
 * │  Pact C Provider Verification Example                          │
 * └────────────────────────────────────────────────────────────────┘
 * @endcode
 *
 * @param fmt The printf-style format string
 * @param ... Additional arguments for the format string
 *
 * @note Output goes to stdout (not stderr like other log functions)
 * @note Requires a terminal that supports UTF-8 box-drawing characters
 * @note Messages longer than 256 characters will be truncated
 */
void draw_boxed_message(const char *fmt, ...) {
  char message[256];
  va_list args;
  va_start(args, fmt);
  vsnprintf(message, sizeof(message), fmt, args);
  va_end(args);

  size_t len = strlen(message);
  if (len < 60)
    len = 60;
  size_t box_width = len + 4;

  printf("┌");
  for (size_t i = 0; i < box_width; i++)
    printf("─");
  printf("┐\n");

  printf("│ %-*s │\n", (int)len + 2, message);

  printf("└");
  for (size_t i = 0; i < box_width; i++)
    printf("─");
  printf("┘\n");
}
