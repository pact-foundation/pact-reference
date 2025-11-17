/**
 * @file logging.h
 * @brief Simple logging utilities for the provider application
 *
 * This module provides a lightweight logging system with multiple severity
 * levels and optional debug output controlled by environment variables.
 *
 * It also includes a function to draw boxed messages to the console for
 * visual emphasis.
 *
 * Log Levels:
 *
 * - DEBUG: Detailed diagnostic information (only shown if VERBOSE env var is
 * set)
 * - INFO: Informational messages about normal operation
 * - WARN: Warning messages for potentially problematic situations
 * - ERROR: Error messages for failures (exits the program)
 *
 * Usage Examples:
 *
 * @code
 * log_debug("Connection received from %s:%d", ip, port);  // Only if VERBOSE=1
 * log_info("Server started on port %d", port);
 * log_warn("High memory usage: %d%%", percent);
 * log_err("Failed to open file: %s", filename);  // Exits program
 *
 * draw_boxed_message("Pact Verification Complete");
 * @endcode
 *
 * Environment Variables:
 *
 * - VERBOSE: Set to any non-empty value to enable debug logging
 *
 * @note All log output goes to stderr for proper stream separation
 * @note ERROR level logs terminate the program with EXIT_FAILURE
 */

#ifndef LOGGING_H
#define LOGGING_H

#include <stdarg.h>
#include <stdbool.h>

/**
 * @brief Internal logging function used by all log level macros.
 *
 * This function handles the actual output of log messages with formatting.
 * It is not intended to be called directly; use the log_debug, log_info,
 * log_warn, or log_err macros instead.
 *
 * The output format is: [LEVEL] file:line message
 * All output is sent to stderr to keep it separate from stdout.
 *
 * @param level The log level string ("DEBUG", "INFO", "WARN", "ERROR")
 * @param file The source file name (automatically provided by macros via
 * __FILE__)
 * @param line The source line number (automatically provided by macros via
 * __LINE__)
 * @param fmt The printf-style format string
 * @param ... Additional arguments for the format string
 *
 * @note Do not call directly; use the log_* macros instead
 * @see log_debug, log_info, log_warn, log_err
 */
void _log_internal(const char *level, const char *file, int line,
                   const char *fmt, ...);

/**
 * @brief Checks if verbose debug logging is enabled.
 *
 * This function checks the VERBOSE environment variable to determine if
 * debug logging should be enabled. The result is cached after the first
 * check for efficiency (avoiding repeated getenv() calls).
 *
 * Debug logging is enabled if VERBOSE is set to any non-empty value.
 *
 * @return true if VERBOSE is set and not empty, false otherwise
 *
 * @note The result is cached for performance
 * @note This is an internal function; use log_debug() macro instead
 */
bool _is_verbose_enabled();

/**
 * @brief Logs a debug message (only if VERBOSE is set).
 *
 * Debug messages provide detailed diagnostic information useful during
 * development and troubleshooting. They are only output when the VERBOSE
 * environment variable is set to a non-empty value.
 *
 * @param ... Printf-style format string and arguments
 *
 * Example:
 * @code
 * log_debug("Processing user ID: %d", user_id);
 * log_debug("Request headers: %s", headers);
 * @endcode
 */
#define log_debug(...)                                                         \
  do {                                                                         \
    if (_is_verbose_enabled())                                                 \
      _log_internal("DEBUG", __FILE__, __LINE__, __VA_ARGS__);                 \
  } while (0)

/**
 * Logs an informational message.
 *
 * This function always prints an info-level log message to stderr. The message
 * is formatted similarly to printf.
 *
 * @param fmt The format string (as in printf).
 * @param ... Additional arguments for the format string.
 *
 * Example usage:
 *   log_info("Started process with PID %d", pid);
 */
#define log_info(...) _log_internal("INFO", __FILE__, __LINE__, __VA_ARGS__)

/**
 * Logs a warning message.
 *
 * This function always prints a warning-level log message to stderr. The
 * message is formatted similarly to printf.
 *
 * @param fmt The format string (as in printf).
 * @param ... Additional arguments for the format string.
 *
 * Example usage:
 *   log_warn("Low disk space: %d%% remaining", percent);
 */
#define log_warn(...) _log_internal("WARN", __FILE__, __LINE__, __VA_ARGS__)

/**
 * Logs an error message and exits the program.
 *
 * This function prints an error-level log message to stderr and then terminates
 * the program with a failure status. The message is formatted similarly to
 * printf.
 *
 * @param fmt The format string (as in printf).
 * @param ... Additional arguments for the format string.
 *
 * Example usage:
 *   log_err("Failed to open file: %s", filename);
 */
#define log_err(...)                                                           \
  do {                                                                         \
    _log_internal("ERROR", __FILE__, __LINE__, __VA_ARGS__);                   \
    exit(EXIT_FAILURE);                                                        \
  } while (0)

/**
 * @brief Draws a boxed message to the console.
 *
 * This function takes a formatted string and prints it within a box made of
 * ASCII characters. The box width adjusts based on the message length, with
 * a minimum width of 60 characters.
 *
 * @param fmt The format string (like printf).
 * @param ... Additional arguments for the format string.
 */
void draw_boxed_message(const char *fmt, ...);

#endif // LOGGING_H
