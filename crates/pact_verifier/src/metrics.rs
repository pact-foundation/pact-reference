//! Structs for collecting metrics for verification

/// Metrics data to send after running a verification
#[derive(Clone, Debug)]
pub struct VerificationMetrics {
  /// test framework used to run the tests
  pub test_framework: String,
  /// Name of the application that ran the tests
  pub app_name: String,
  /// Version of the application that ran the tests
  pub app_version: String
}
