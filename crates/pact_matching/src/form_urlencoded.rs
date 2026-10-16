//! Functions for matching `multipart/form` bodies

use itertools::Itertools;
use pact_models::bodies::OptionalBody;
use pact_models::http_parts::HttpPart;
use tracing::debug;

use crate::{MatchingContext, Mismatch};
use crate::query::match_query_maps;

/// Matches the bodies using application/x-www-form-urlencoded encoding
pub(crate) fn match_form_urlencoded(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<super::Mismatch>> {
  let expected_body = expected.body();
  let actual_body = actual.body();
  match expected_body {
    OptionalBody::Missing | OptionalBody::Null => Ok(()),
    OptionalBody::Empty => match actual_body {
      OptionalBody::Empty => Ok(()),
      _ => Err(vec![
        Mismatch::BodyMismatch {
          path: "$".into(),
          expected: expected_body.value(),
          actual: actual_body.value(),
          mismatch: format!("Expected an empty body, but got '{}'", actual_body.value_as_string().unwrap_or(actual_body.display_string()))
        }
      ])
    }
    OptionalBody::Present(b, _, _) => {
      let expected_form = serde_urlencoded::from_bytes::<Vec<(String, String)>>(b)
        .map_err(|err| {
          Mismatch::BodyMismatch {
            path: "$".into(),
            expected: expected_body.value(),
            actual: actual_body.value(),
            mismatch: format!("Could not parse expected body: {}", err)
          }
        });
      let actual_bytes = actual_body.value().unwrap_or_default();
      let actual_form = serde_urlencoded::from_bytes::<Vec<(String, String)>>(actual_bytes.as_ref())
        .map_err(|err| {
          Mismatch::BodyMismatch {
            path: "$".into(),
            expected: expected_body.value(),
            actual: actual_body.value(),
            mismatch: format!("Could not parse actual body: {}", err)
          }
        });
      match (expected_form, actual_form) {
        (Err(m), Err(m2)) => Err(vec![m, m2]),
        (Err(m), Ok(_)) => Err(vec![m]),
        (Ok(_), Err(m2)) => Err(vec![m2]),
        (Ok(e), Ok(a)) => {
          debug!(expected=?e, actual=?a, "Comparing form post body");
          let expected_params = super::group_by(e, |(k, _)| k.clone())
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(|(_, v)| Some(v.clone())).collect_vec()))
            .collect();
          let actual_params = super::group_by(a, |(k, _)| k.clone())
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(|(_, v)| Some(v.clone())).collect_vec()))
            .collect();
          let result: Vec<_> = match_query_maps(expected_params, actual_params, context)
            .values().flat_map(|m| m.iter().map(|mismatch| {
            if let Mismatch::QueryMismatch { parameter, expected, actual, mismatch } = mismatch {
              Mismatch::BodyMismatch {
                path: format!("$.{}", parameter),
                expected: Some(expected.clone().into()),
                actual: Some(actual.clone().into()),
                mismatch: mismatch.replace("query parameter", "form post parameter")
              }
            } else {
              Mismatch::BodyMismatch {
                path: "$".to_string(),
                expected: None,
                actual: None,
                mismatch: mismatch.description()
              }
            }
          })).collect();
          if result.is_empty() {
            Ok(())
          } else {
            Err(result)
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::bodies::OptionalBody;
  use pact_models::content_types::{ContentTypeHint, FORM_URLENCODED};
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::request::Request;
  use pretty_assertions::assert_eq;

  use crate::{CoreMatchingContext, DiffConfig, Mismatch};

  use super::match_form_urlencoded;

  #[test_log::test]
  fn compare_missing_bodies() {
    let expected = Request {
      .. Request::default()
    };
    let actual = Request {
      .. Request::default()
    };
    let result = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default());
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn compare_empty_bodies() {
    let expected = Request {
      body: OptionalBody::Empty,
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Empty,
      .. Request::default()
    };
    let result = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default());
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn when_actual_body_is_not_empty() {
    let expected = Request {
      body: OptionalBody::Empty,
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b&c=d".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let result = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(result.first().unwrap().description()).to(be_equal_to("$ -> Expected an empty body, but got 'a=b&c=d'"));
  }

  #[test_log::test]
  fn match_form_returns_nothing_if_there_are_no_parameters() {
    let expected = Request {
      body: OptionalBody::Present("".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let result = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default());
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_form_applies_matching_rules_when_param_has_an_underscore() {
    let rules = matchingrules! {
      "body" => { "$.user_id" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(), &hashmap!{}
    );
    let expected = Request {
      body: OptionalBody::Present("user_id=1".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("user_id=2".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let result = match_form_urlencoded(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_there_is_no_expected_parameters() {
    let expected = Request {
      body: OptionalBody::Present("".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.a".to_string(),
      expected: Some("".into()),
      actual: Some("[\"b\"]".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.a -> Unexpected form post parameter 'a' received");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_there_is_no_actual_form_parameters() {
    let expected = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.a".to_string(),
      expected: Some("[\"b\"]".into()),
      actual: Some("".into()),
      mismatch: "Expected form post parameter 'a' but was missing".to_string()
    });
    assert_eq!(mismatches[0].description(), "$.a -> Expected form post parameter 'a' but was missing");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_there_is_an_actual_parameter_that_is_not_expected() {
    let expected = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b&c=d".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.c".to_string(),
      expected: Some("".into()),
      actual: Some("[\"d\"]".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.c -> Unexpected form post parameter 'c' received");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_there_is_an_expected_parameter_that_is_not_received() {
    let expected = Request {
      body: OptionalBody::Present("a=b&c=d".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.c".to_string(),
      expected: Some("[\"d\"]".into()),
      actual: Some("".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.c -> Expected form post parameter 'c' but was missing");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_there_is_an_empty_expected_parameter_and_a_non_empty_actual() {
    let expected = Request {
      body: OptionalBody::Present("a=b&c".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b&c=d".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.c".to_string(),
      expected: Some("".into()),
      actual: Some("d".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.c -> Expected form post parameter 'c' with value '' but was 'd'");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_the_values_have_different_lengths() {
    let expected = Request {
      body: OptionalBody::Present("a=b&c=d&c=e".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=b&c=d".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.len()).to(be_equal_to(2));
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.c".to_string(),
      expected: Some("[\"d\", \"e\"]".into()),
      actual: Some("[\"d\"]".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.c -> Expected form post parameter 'c' value 'e' but was missing");
    assert_eq!(mismatches[1], Mismatch::BodyMismatch {
      path: "$.c".to_string(),
      expected: Some("[\"d\", \"e\"]".into()),
      actual: Some("[\"d\"]".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[1].description(), "$.c -> Expected form post parameter 'c' with 2 value(s) but received 1 value(s)");
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_the_values_are_not_the_same() {
    let expected = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=c".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let mismatches = match_form_urlencoded(&expected, &actual, &CoreMatchingContext::default())
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.a".to_string(),
      expected: Some("b".into()),
      actual: Some("c".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.a -> Expected form post parameter 'a' with value 'b' but was 'c'");
  }

  #[test_log::test]
  fn match_form_with_min_type_matching_rules() {
    let expected = Request {
      body: OptionalBody::Present("id=1&id=2".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("id=1&id=2&id=3&id=4".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let rules = matchingrules! {
      "body" => { "$.id" => [ MatchingRule::MinType(2) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(), &hashmap!{}
    );
    let result = match_form_urlencoded(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_form_returns_no_mismatch_if_the_values_are_not_the_same_but_match_by_a_matcher() {
    let expected = Request {
      body: OptionalBody::Present("a=b".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=hgjhghgh".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let rules = matchingrules! {
      "body" => { "$.a" => [ MatchingRule::Regex("\\w+".to_string()) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(), &hashmap!{}
    );
    let result = match_form_urlencoded(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_form_returns_a_mismatch_if_the_values_do_not_match_by_a_matcher() {
    let expected = Request {
      body: OptionalBody::Present("a=1".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let actual = Request {
      body: OptionalBody::Present("a=hgjhghgh".bytes().collect(), Some(FORM_URLENCODED.clone()), Some(ContentTypeHint::TEXT)),
      .. Request::default()
    };
    let rules = matchingrules! {
      "body" => { "$.a" => [ MatchingRule::Regex("\\d+".to_string()) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(), &hashmap!{}
    );
    let mismatches = match_form_urlencoded(&expected, &actual, &context)
      .unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::BodyMismatch {
      path: "$.a".to_string(),
      expected: Some("1".into()),
      actual: Some("hgjhghgh".into()),
      mismatch: "".to_string(),
    });
    assert_eq!(mismatches[0].description(), "$.a -> Expected 'hgjhghgh' to match '\\d+'");
  }
}
