#ifndef PACT_LOGGING_TEST_H
#define PACT_LOGGING_TEST_H

/**
 * @brief Test that Pact logging works and errors are logged when interactions
 * are not verified.
 *
 * See logging_test.c for details. Returns 0 on success.
 */
int pact_logging_stdout();

/**
 * @brief Test that Pact logging to stderr works and errors are logged when
 * interactions are not verified.
 *
 * See logging_test.c for details. Returns 0 on success.
 */
int pact_logging_stderr();

/**
 * @brief Test that Pact logging to a buffer works and errors are logged when
 * interactions are not verified.
 *
 * See logging_test.c for details. Returns 0 on success.
 */
int pact_logging_buffer();

#endif // PACT_LOGGING_TEST_H
