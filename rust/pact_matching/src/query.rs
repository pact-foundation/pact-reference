//! Module for functions dealing with matching query parameters

use std::collections::HashMap;

use itertools::Itertools;
use maplit::hashmap;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;
use tracing::debug;

use crate::{MatchingContext, merge_result, Mismatch, CommonMismatch};
use crate::matchingrules::{compare_lists_with_matchingrules, match_values, Matches};

/// Match the query parameters as Maps
pub(crate) fn match_query_maps(
  expected: HashMap<String, Vec<Option<String>>>,
  actual: HashMap<String, Vec<Option<String>>>,
  context: &dyn MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  let mut result: HashMap<String, Vec<Mismatch>> = hashmap!{};
  for (key, value) in &expected {
    let expected_value = value.iter().map(|v| v.clone().unwrap_or_default()).collect_vec();
    match actual.get(key) {
      Some(actual_value) => {
        let actual_value = actual_value.iter().map(|v| v.clone().unwrap_or_default()).collect_vec();
        let mismatches: Result<(), Vec<super::Mismatch>> = match_query_values(key, &expected_value, &actual_value, context)
          .map_err(|mismatches| mismatches.iter().map(|mismatch| mismatch.to_query_mismatch()).collect());
        let v = result.entry(key.clone()).or_default();
        v.extend(mismatches.err().unwrap_or_default());
      },
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: format!("{:?}", expected_value),
        actual: "".to_string(),
        mismatch: format!("Expected query parameter '{}' but was missing", key)
      })
    }
  }
  for (key, value) in &actual {
    match expected.get(key) {
      Some(_) => (),
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: "".to_string(),
        actual: format!("{:?}", value.iter().map(|v| v.clone().unwrap_or_default()).collect_vec()),
        mismatch: format!("Unexpected query parameter '{}' received", key)
      })
    }
  }
  result
}

fn match_query_values(
  key: &str,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> Result<(), Vec<CommonMismatch>> {
  let path = DocPath::root().join(key);
  if context.matcher_is_defined(&path) {
    debug!("match_query_values: Matcher defined for query parameter '{}", key);
    compare_lists_with_matchingrules(&path, &context.select_best_matcher(&path), expected, actual, context.clone_with(context.matchers()).as_ref(), &mut |p, expected, actual, context| {
      compare_query_parameter_value(p, expected, actual, 0, context)
    })
  } else {
    if expected.is_empty() && !actual.is_empty() {
      Err(vec![ CommonMismatch {
        path: key.to_string(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
        description: format!("Expected an empty parameter list for '{}' but received {:?}", key, actual)
      } ])
    } else {
      let mismatch = if expected.len() != actual.len() {
        Err(vec![ CommonMismatch {
          path: key.to_string(),
          expected: format!("{:?}", expected),
          actual: format!("{:?}", actual),
          description: format!(
            "Expected query parameter '{}' with {} value(s) but received {} value(s)",
            key, expected.len(), actual.len())
        } ])
      } else {
        Ok(())
      };
      merge_result(compare_query_parameter_values(&path, expected, actual, context), mismatch)
    }
  }
}

fn compare_query_parameter_value(
  path: &DocPath,
  expected: &str,
  actual: &str,
  index: usize,
  context: &dyn MatchingContext
) -> Result<(), Vec<CommonMismatch>> {
  let index = index.to_string();
  let index_path = path.join(index.as_str());
  let matcher_result = if context.matcher_is_defined(&index_path) {
    match_values(&index_path, &context.select_best_matcher(&index_path),
      expected.to_string(), actual.to_string())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false)
      .map_err(|_error| vec![
        format!("Expected query parameter '{}' with value '{}' but was '{}'",
          path.to_vec().last().cloned().unwrap_or_else(|| "??".to_string()),
          expected,
          actual
        )
      ])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      CommonMismatch {
        path: path.first_field().unwrap_or_default().to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        description: message.clone()
      }
    }).collect()
  })
}

fn compare_query_parameter_values(
  path: &DocPath,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> Result<(), Vec<CommonMismatch>> {
  let empty = String::new();
  let result: Vec<CommonMismatch> = expected.iter()
    .pad_using(actual.len(), |_| &empty)
    .enumerate()
    .flat_map(|(index, val)| {
      if index < actual.len() {
        match compare_query_parameter_value(path, val, &actual[index], index, context) {
          Ok(_) => vec![],
          Err(errors) => errors
        }
      } else if context.matcher_is_defined(path) {
        vec![]
      } else {
        let key = path.first_field().unwrap_or_default().to_string();
        vec![ CommonMismatch {
          path: key.clone(),
          expected: format!("{:?}", expected),
          actual: format!("{:?}", actual),
          description: format!("Expected query parameter '{}' value '{}' but was missing", key, val)
        } ]
      }
    })
    .collect();

  if result.is_empty() {
    Ok(())
  } else {
    Err(result)
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::matchingrules;
  use rstest::rstest;

  use crate::{CoreMatchingContext, DiffConfig, MatchingRule};

  #[test]
  fn compare_values_with_type_matcher() {
    let expected = ["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()];
    let actual = ["1".to_string(), "3".to_string()];
    let rules = matchingrules! {
      "query" => { "id" => [ MatchingRule::MinType(2) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("query").unwrap_or_default(),
      &hashmap!{}
    );

    expect!(super::match_query_values("id", &expected, &actual, &context))
      .to(be_ok());
  }

  #[rstest]
  #[case(["abc".to_string()], ["def".to_string()], MatchingRule::Number, false)]
  #[case(["abc".to_string()], ["def".to_string()], MatchingRule::Integer, false)]
  #[case(["abc".to_string()], ["def".to_string()], MatchingRule::Decimal, false)]
  #[case(["100".to_string()], ["101".to_string()], MatchingRule::Number, true)]
  #[case(["100".to_string()], ["101".to_string()], MatchingRule::Integer, true)]
  #[case(["100".to_string()], ["101".to_string()], MatchingRule::Decimal, true)]
  #[case(["100.01".to_string()], ["101.02".to_string()], MatchingRule::Number, true)]
  #[case(["100.01".to_string()], ["101.02".to_string()], MatchingRule::Integer, false)]
  #[case(["100.01".to_string()], ["101.02".to_string()], MatchingRule::Decimal, true)]
  fn compare_values_with_number_matchers(#[case] expected: [String; 1], #[case] actual: [String; 1], #[case] matcher: MatchingRule, #[case] matched: bool) {
    let rules = matchingrules! {
      "query" => { "number" => [ matcher ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("query").unwrap_or_default(),
      &hashmap!{}
    );

    let result = super::match_query_values("number", &expected, &actual, &context);
    if matched {
      expect!(result).to(be_ok());
    } else {
      expect!(result).to(be_err());
    }
  }
}
