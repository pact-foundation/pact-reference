use pact_matching::match_request;
use pact_matching::models::Pact;

/// Check that all requests in `actual` match the patterns provide by
/// `expected`, and raise an error if anything fails.
pub(crate) fn check_requests_match(
    actual_label: &str,
    actual: &Pact,
    expected_label: &str,
    expected: &Pact,
) -> Result<(), String> {
    // First make sure we have the same number of interactions.
    if expected.interactions.len() != actual.interactions.len() {
        return Err(format!(
                "the pact `{}` has {} interactions, but `{}` has {}",
                expected_label,
                expected.interactions.len(),
                actual_label,
                actual.interactions.len(),
            ));
    }

    // Next, check each interaction to see if it matches.
    for (e, a) in expected.interactions.iter().zip(&actual.interactions) {
        let mismatches = match_request(e.request.clone(), a.request.clone());
        if !mismatches.is_empty() {
            let mut reasons = String::new();
            for mismatch in mismatches {
                reasons.push_str(&format!("- {}\n", mismatch.description()));
            }
            return Err(format!(
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
