use std::collections::HashMap;
use std::panic::RefUnwindSafe;

use anyhow::anyhow;
use pact_models::generators::GeneratorTestMode;
use pact_models::pact::Pact;
use serde_json::Value;
use tracing::debug;

use pact_matching::{generate_request, match_request};

/// Check that all requests in `actual` match the patterns provide by
/// `expected`, and raise an error if anything fails.
pub(crate) fn check_requests_match(
    actual_label: &str,
    actual: &Box<dyn Pact + Send + Sync + RefUnwindSafe>,
    expected_label: &str,
    expected: &Box<dyn Pact + Send + Sync + RefUnwindSafe>,
    context: &HashMap<&str, Value>
) -> anyhow::Result<()> {
    // First make sure we have the same number of interactions.
    if expected.interactions().len() != actual.interactions().len() {
        return Err(anyhow!(
                "the pact `{}` has {} interactions, but `{}` has {}",
                expected_label,
                expected.interactions().len(),
                actual_label,
                actual.interactions().len(),
            ));
    }

    // Next, check each interaction to see if it matches.
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .expect("new runtime");
    for (e, a) in expected.interactions().iter().zip(actual.interactions()) {
        let actual_request = a.as_v4_http().unwrap().request.clone();
        debug!("actual_request = {:?}", actual_request);
        let generated_request = runtime.block_on(generate_request(&actual_request, &GeneratorTestMode::Provider, context));
        debug!("generated_request = {:?}", generated_request);
        let mismatches = runtime.block_on(match_request(e.as_v4_http().unwrap().request.clone(),
                generated_request, expected, e))?;
        if !mismatches.all_matched() {
          let mut reasons = String::new();
          for mismatch in mismatches.mismatches() {
            reasons.push_str(&format!("- {}\n", mismatch.description()));
          }
          return Err(anyhow!(
            "the pact `{}` does not match `{}` because:\n{}",
            expected_label,
            actual_label,
            reasons,
          ));
        }
    }

    Ok(())
}

macro_rules! assert_requests_match {
    ($actual:expr, $expected:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                &HashMap::new(),
            );
            if let ::std::result::Result::Err(message) = result {
                panic!("{}", message)
            }
        }
    )
}

macro_rules! assert_requests_do_not_match {
    ($actual:expr, $expected:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                &HashMap::new(),
            );
            if let ::std::result::Result::Ok(()) = result {
                panic!(
                    "pact `{}` unexpectedly matched pattern `{}`",
                    stringify!($actual),
                    stringify!($expected),
                );
            }
        }
    )
}

macro_rules! assert_requests_with_context_match {
    ($actual:expr, $expected:expr, $context:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                $context,
            );
            if let ::std::result::Result::Err(message) = result {
                panic!("{}", message)
            }
        }
    )
}

macro_rules! assert_requests_with_context_do_not_match {
    ($actual:expr, $expected:expr, $context:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                $context,
            );
            if let ::std::result::Result::Ok(()) = result {
                panic!(
                    "pact `{}` unexpectedly matched pattern `{}`",
                    stringify!($actual),
                    stringify!($expected),
                );
            }
        }
    )
}
