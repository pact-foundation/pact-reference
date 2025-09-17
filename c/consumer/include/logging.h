#ifndef LOGGING_H
#define LOGGING_H

#include <stdarg.h>

/**
 * Logs a debug message if the VERBOSE environment variable is set.
 *
 * This function prints a debug-level log message to stderr only when the
 * VERBOSE environment variable is set. The message is formatted similarly
 * to printf.
 *
 * @param fmt The format string (as in printf).
 * @param ... Additional arguments for the format string.
 *
 * Example usage:
 *   log_debug("Value: %d", value);
 */
void _log_internal(const char *level, const char *file, int line, const char *fmt, ...);

/**
 * Checks if verbose debug logging is enabled via the VERBOSE environment
 * variable.
 *
 * This function caches the result after the first check for efficiency.
 *
 * @return true if VERBOSE is set and not empty, false otherwise.
 */
bool _is_verbose_enabled();

#define log_debug(...)                                               \
    do                                                               \
    {                                                                \
        if (_is_verbose_enabled())                                   \
            _log_internal("DEBUG", __FILE__, __LINE__, __VA_ARGS__); \
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
#define log_err(...)                                             \
    do                                                           \
    {                                                            \
        _log_internal("ERROR", __FILE__, __LINE__, __VA_ARGS__); \
        exit(EXIT_FAILURE);                                      \
    } while (0)

#endif // LOGGING_H
