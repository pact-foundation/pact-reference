#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <stdbool.h>

/**
 * Internal function to print a formatted log message to stderr.
 *
 * This function is used by all log level functions to output a message
 * with a given log level prefix. The message is formatted using a
 * variable argument list.
 *
 * @param level The log level string (e.g., "DEBUG", "INFO").
 * @param fmt The format string (as in printf).
 * @param args The variable argument list for formatting.
 */
void _log_internal(const char *level, const char *file, int line, const char *fmt, ...)
{
    fprintf(stderr, "[%s] %s:%d ", level, file, line);
    va_list args;
    va_start(args, fmt);
    vfprintf(stderr, fmt, args);
    va_end(args);
    fprintf(stderr, "\n");
}

enum
{
    LOGGING_VERBOSE_UNKNOWN,
    LOGGING_VERBOSE_ON,
    LOGGING_VERBOSE_OFF
};
static int logging_verbose_state = LOGGING_VERBOSE_UNKNOWN;

/**
 * Checks if verbose debug logging is enabled via the VERBOSE environment variable.
 *
 * This function caches the result after the first check for efficiency.
 *
 * @return true if VERBOSE is set and not empty, false otherwise.
 */
bool _is_verbose_enabled()
{
    if (logging_verbose_state == LOGGING_VERBOSE_UNKNOWN)
    {
        const char *env = getenv("VERBOSE");
        logging_verbose_state = (env && env[0] != '\0') ? LOGGING_VERBOSE_ON : LOGGING_VERBOSE_OFF;
    }
    return logging_verbose_state == LOGGING_VERBOSE_ON;
}
