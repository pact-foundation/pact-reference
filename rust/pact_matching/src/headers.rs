//! Matching functions for headers

use std::collections::HashMap;
use std::iter::FromIterator;

use itertools::Itertools;
use maplit::hashmap;
use pact_models::headers::PARAMETERISED_HEADERS;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;
use tracing::{instrument, debug};
use crate::{CommonMismatch, MatchingContext, Mismatch};
use crate::matchingrules::{compare_lists_with_matchingrules, DoMatch, match_values};

pub(crate) fn strip_whitespace<'a, T: FromIterator<&'a str>>(val: &'a str, split_by: &'a str) -> T {
  val.split(split_by).map(|v| v.trim()).filter(|v| !v.is_empty()).collect()
}

pub(crate) fn parse_charset_parameters(parameters: &[&str]) -> HashMap<String, String> {
  parameters.iter().map(|v| v.split_once('=')
    .map(|(k, v)| (k.trim(), v.trim())))
    .fold(HashMap::new(), |mut map, name_value| {
      if let Some((name, value)) = name_value {
        map.insert(name.to_string(), value.to_string());
      }
      map
    })
}

pub(crate) fn match_parameter_header(
  expected: &str,
  actual: &str,
  header: &str,
  value_type: &str,
  index: usize,
  single_value: bool
) -> Result<(), Vec<String>> {
  let expected_values: Vec<&str> = strip_whitespace(expected, ";");
  let actual_values: Vec<&str> = strip_whitespace(actual, ";");

  let expected_parameters = expected_values.as_slice().split_first().unwrap_or((&"", &[]));
  let actual_parameters = actual_values.as_slice().split_first().unwrap_or((&"", &[]));
  let header_mismatch = if single_value {
    format!("Expected {} '{}' to have value '{}' but was '{}'", value_type, header, expected, actual)
  } else {
    format!("Expected {} '{}' at index {} to have value '{}' but was '{}'", value_type, header, index, expected, actual)
  };

  let mut mismatches = vec![];
  if expected_parameters.0 == actual_parameters.0 {
    let expected_parameter_map = parse_charset_parameters(expected_parameters.1);
    let actual_parameter_map = parse_charset_parameters(actual_parameters.1);
    for (k, v) in expected_parameter_map {
      if actual_parameter_map.contains_key(&k) {
        if v.to_ascii_lowercase() != actual_parameter_map.get(&k).unwrap().to_ascii_lowercase() {
          mismatches.push(header_mismatch.clone());
        }
      } else {
        mismatches.push(header_mismatch.clone());
      }
    }
  } else {
    mismatches.push(header_mismatch);
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches)
  }
}

#[instrument(level = "trace")]
pub(crate) fn match_header_value(
  key: &str,
  index: usize,
  expected: &str,
  actual: &str,
  context: &dyn MatchingContext,
  single_value: bool
) -> Result<(), Vec<CommonMismatch>> {
  let path = DocPath::root().join(key.to_lowercase());
  let indexed_path = path.join(index.to_string());
  let expected = expected.trim();
  let actual = actual.trim();

  let matcher_result = if context.matcher_is_defined(&path) {
    let result = match_values(&path, &context.select_best_matcher(&path), expected, actual);
    if single_value {
      result
    } else {
      result.map_err(|err| err.iter().map(|e| format!("{} for value at index {}", e, index)).collect())
    }
  } else if context.matcher_is_defined(&indexed_path) {
    let result = match_values(&indexed_path, &context.select_best_matcher(&indexed_path), expected, actual);
    if single_value {
      result
    } else {
      result.map_err(|err| err.iter().map(|e| format!("{} for value at index {}", e, index)).collect())
    }
  } else if PARAMETERISED_HEADERS.contains(&key.to_lowercase().as_str()) {
    match_parameter_header(expected, actual, key, "header", index, single_value)
  } else {
    // Normalize whitespace around commas for comparison, since RFC 7230 allows optional whitespace
    // after commas in header values. This ensures "a,b" matches "a, b".
    let normalize_comma_whitespace = |s: &str| -> String {
      s.split(',').map(|v| v.trim()).collect::<Vec<_>>().join(",")
    };
    let normalized_expected = normalize_comma_whitespace(expected);
    let normalized_actual = normalize_comma_whitespace(actual);
    MatchingRule::Equality.match_value(normalized_expected.as_str(), normalized_actual.as_str(), false, false)
      .map_err(|err| {
        if single_value {
          vec![format!("{}", err)]
        } else {
          vec![format!("{} for value at index {}", err, index)]
        }
      })
  };

  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      CommonMismatch {
        path: key.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        description: format!("Mismatch with header '{}': {}", key, message)
      }
    }).collect()
  })
}

fn find_entry<T>(map: &HashMap<String, T>, key: &str) -> Option<(String, T)> where T: Clone {
  match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
    Some(k) => map.get(k).map(|v| (key.to_string(), v.clone()) ),
    None => None
  }
}

fn match_header_maps(
  expected: HashMap<String, Vec<String>>,
  actual: HashMap<String, Vec<String>>,
  context: &dyn MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  let mut result = hashmap!{};
  for (key, value) in &expected {
    match find_entry(&actual, key) {
      Some((_, actual_values)) => if value.is_empty() && !actual_values.is_empty() {
        result.insert(key.clone(), vec![Mismatch::HeaderMismatch { key: key.clone(),
          expected: "".to_string(),
          actual: format!("{}", actual_values.join(", ")),
          mismatch: format!("Expected an empty header '{}' but actual value was '{}'", key, actual_values.join(", ")) }]);
      } else {
        let mut mismatches = vec![];

        // Special case when the headers only have 1 value to improve messaging
        if value.len() == 1 && actual_values.len() == 1 {
          let comparison_result = match_header_value(key, 0, value.first().unwrap(),
            actual_values.first().unwrap(), context, true)
            .err()
            .unwrap_or_default();
          mismatches.extend(comparison_result.iter().cloned());
        } else {
          let path = DocPath::root().join(key.to_lowercase());
          if context.matcher_is_defined(&path) {
            debug!("match_header_maps: Matcher is defined for path {}", path);
            let values_result = compare_lists_with_matchingrules(&path, &context.select_best_matcher(&path), value, &actual_values, context.clone_with(context.matchers()).as_ref(), &mut |_, expected, actual, context| {
              match_header_value(key, 0, expected, actual, context, false)
            });
            mismatches.extend(values_result.err().unwrap_or_default());
          } else {
            let empty = String::new();
            for (index, val) in value.iter()
              .pad_using(actual_values.len(), |_| &empty)
              .enumerate() {
              if let Some(actual_value) = actual_values.get(index) {
                let comparison_result = match_header_value(key, index, val,
                  actual_value, context, false)
                  .err()
                  .unwrap_or_default();
                mismatches.extend(comparison_result.iter().cloned());
              } else {
                mismatches.push(CommonMismatch {
                  path: key.clone(),
                  expected: val.clone(),
                  actual: "".to_string(),
                  description: format!("Mismatch with header '{}': Expected value '{}' at index {} but was missing (actual has {} value(s))",
                                    key, val, index, actual_values.len())
                });
              }
            }
          }
        }

        result.insert(key.clone(), mismatches.iter().map(|mismatch| mismatch.to_header_mismatch()).collect());
      },
      None => {
        result.insert(key.clone(), vec![Mismatch::HeaderMismatch { key: key.clone(),
          expected: format!("{:?}", value.join(", ")),
          actual: "".to_string(),
          mismatch: format!("Expected a header '{}' but was missing", key) }]);
      }
    }
  }
  result
}

/// Matches the actual headers to the expected ones.
pub fn match_headers(
  expected: Option<HashMap<String, Vec<String>>>,
  actual: Option<HashMap<String, Vec<String>>>,
  context: &(dyn MatchingContext + Send + Sync)
) -> HashMap<String, Vec<Mismatch>> {
  match (actual, expected) {
    (Some(aqm), Some(eqm)) => match_header_maps(eqm, aqm, context),
    (Some(_), None) => hashmap!{},
    (None, Some(eqm)) => eqm.iter().map(|(key, value)| {
      (key.clone(), vec![Mismatch::HeaderMismatch { key: key.clone(),
        expected: format!("{:?}", value.join(", ")),
        actual: "".to_string(),
        mismatch: format!("Expected a header '{}' but was missing", key) }])
    }).collect(),
    (None, None) => hashmap!{}
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::*;
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
  use pretty_assertions::assert_eq;

  use crate::{CoreMatchingContext, DiffConfig, HeaderMatchingContext, Mismatch, CommonMismatch};
  use crate::headers::{match_header_value, match_headers, parse_charset_parameters};

  #[test]
  fn matching_headers_be_true_when_headers_are_equal() {
    let mismatches = match_header_value("HEADER", 0, "HEADER", "HEADER",
      &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn matching_headers_be_false_when_headers_are_not_equal() {
    let mismatches = match_header_value("HEADER", 0, "HEADER", "HEADER2",
      &CoreMatchingContext::default(), true
    ).unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], CommonMismatch {
      path: "HEADER".to_string(),
      expected: "HEADER".to_string(),
      actual: "HEADER2".to_string(),
      description: "".to_string()
    });
  }

  #[test]
  fn mismatch_message_generated_when_headers_are_not_equal() {
    let mismatches = match_header_value("HEADER", 0, "HEADER_VALUE", "HEADER2",
      &CoreMatchingContext::default(), true
    );

    let CommonMismatch { ref description, .. } = mismatches.unwrap_err()[0];
    assert_eq!(description, "Mismatch with header 'HEADER': Expected 'HEADER2' to be equal to 'HEADER_VALUE'");
  }

  #[test]
  fn content_type_header_matches_when_headers_are_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/json;charset=UTF-8",
      "application/json; charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn content_type_header_does_not_match_when_headers_are_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/pdf;charset=UTF-8",
      "application/json;charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_expected_is_empty() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "",
      "application/json;charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_actual_is_empty() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/pdf;charset=UTF-8",
      "", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_charsets_are_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/json;charset=UTF-8",
      "application/json;charset=UTF-16", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_match_when_charsets_are_different_case() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/json;charset=UTF-8",
      "application/json;charset=utf-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn content_type_header_does_not_match_when_charsets_other_parameters_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/json;declaration=\"<950118.AEB0@XIson.com>\"",
      "application/json;charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_match_when_charsets_is_missing_from_expected_header() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "application/json",
      "application/json;charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn mismatched_header_description_reports_content_type_mismatches_correctly() {
    let mismatches = match_header_value("CONTENT-TYPE", 0, "CONTENT-TYPE-VALUE", "HEADER2",
      &CoreMatchingContext::default(), true
    );

    let CommonMismatch { ref description, .. } = mismatches.unwrap_err()[0];
    assert_eq!(description, "Mismatch with header 'CONTENT-TYPE': Expected header 'CONTENT-TYPE' to have value 'CONTENT-TYPE-VALUE' but was 'HEADER2'");
  }

  #[test]
  fn accept_header_matches_when_headers_are_equal() {
    let mismatches = match_header_value("ACCEPT", 0, "application/hal+json;charset=utf-8",
      "application/hal+json;charset=utf-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn accept_header_does_not_match_when_actual_is_empty() {
    let mismatches = match_header_value("ACCEPT", 0, "application/hal+json",
      "", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn accept_header_does_match_when_charset_is_missing_from_expected_header() {
    let mismatches = match_header_value("ACCEPT", 0, "application/hal+json",
      "application/hal+json;charset=utf-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn accept_header_does_not_match_when_charsets_are_not_equal() {
    let mismatches = match_header_value("ACCEPT", 0, "application/hal+json;charset=utf-8",
      "application/hal+json;charset=utf-16", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn accept_header_does_match_when_charsets_are_different_case() {
    let mismatches = match_header_value("ACCEPT", 0, "application/hal+json;charset=utf-8",
      "application/hal+json;charset=UTF-8", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn mismatched_header_description_reports_accept_header_mismatches_correctly() {
    let mismatches = match_header_value("ACCEPT", 0, "ACCEPT-VALUE", "HEADER2",
      &CoreMatchingContext::default(), true
    );
    assert_eq!(mismatches.unwrap_err()[0].description,
      "Mismatch with header 'ACCEPT': Expected header 'ACCEPT' to have value 'ACCEPT-VALUE' but was 'HEADER2'");
  }

  #[test]
  fn accept_header_matching_with_multiple_values() {
    let expected = Some(hashmap! { "accept".to_string() => vec!["application/json".to_string(), "application/hal+json".to_string()] });
    let actual = Some(hashmap! { "accept".to_string() => vec!["application/json".to_string(), "application/hal+json".to_string()] });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test_log::test]
  fn matching_headers_be_true_when_headers_match_by_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "HEADER" => [ MatchingRule::Regex("\\w+".to_string()) ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let mismatches = match_header_value("HEADER", 0, "HEADERX", "HEADERY", &context, true);
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn matching_headers_be_false_when_headers_do_not_match_by_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
          "header" => {
              "HEADER" => [ MatchingRule::Regex("\\d+".to_string()) ]
          }
        }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let mismatches = match_header_value(&"HEADER".to_string(), 0,
      &"HEADER".to_string(), &"HEADER".to_string(), &context, true);
    expect!(mismatches).to(be_err().value(vec![ CommonMismatch {
      path: "HEADER".to_string(),
      expected: "HEADER".to_string(),
      actual: "HEADER".to_string(),
      description: String::default(),
    } ]));
  }

  #[test]
  fn match_header_value_does_match_when_not_well_formed() {
    let mismatches = match_header_value("content-type", 0, "application/json",
      "application/json;", &CoreMatchingContext::default(), true
    );
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn parse_charset_parameters_test() {
    expect!(parse_charset_parameters(&[])).to(be_equal_to(hashmap!{}));
    expect!(parse_charset_parameters(&[""])).to(be_equal_to(hashmap!{}));
    expect!(parse_charset_parameters(&["a"])).to(be_equal_to(hashmap!{}));
    expect!(parse_charset_parameters(&["a="])).to(be_equal_to(hashmap!{ "a".to_string() => String::default() }));
    expect!(parse_charset_parameters(&["a=b"])).to(be_equal_to(hashmap!{ "a".to_string() => "b".to_string() }));
    expect!(parse_charset_parameters(&["a=b", "c=d"])).to(be_equal_to(hashmap!{
      "a".to_string() => "b".to_string(),
      "c".to_string() => "d".to_string()
    }));
  }

  // Issue #238
  #[test_log::test]
  fn matching_headers_with_an_indexed_path() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "HEADER[0]" => [ MatchingRule::Regex("\\w+".to_string()) ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let mismatches = match_header_value("HEADER", 0, "HEADERX", "HEADERY", &context, true);
    expect!(mismatches).to(be_ok());
  }

  #[test_log::test]
  fn match_headers_returns_nothing_if_there_are_no_headers() {
    let expected = None;
    let actual = None;
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test_log::test]
  fn match_headers_applies_matching_rules_when_header_name_has_an_underscore() {
    let expected = hashmap! { "user_id".to_string() => vec!["1".to_string()] };
    let actual = hashmap! { "user_id".to_string() => vec!["2".to_string()] };
    let rules = matchingrules! {
      "header" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    );
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn match_headers_returns_no_mismatch_if_there_is_no_expected_header_and_we_allow_unexpected_keys() {
    let expected = None;
    let actual = Some(hashmap!{
    "a".to_string() => vec!["b".to_string()]
  });
    let result = match_headers(expected, actual,
                               &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_headers_returns_a_mismatch_if_there_is_no_actual_headers() {
    let expected = Some(hashmap! {
      "a".to_string() => vec!["b".to_string()]
    });
    let actual = None;
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::HeaderMismatch {
      key: "a".to_string(),
      expected: "\"b\"".to_string(),
      actual: "".to_string(),
      mismatch: "Expected a header 'a' but was missing".to_string()
    });
  }

  #[test]
  fn match_headers_returns_a_mismatch_if_there_is_an_expected_header_that_is_not_received() {
    let expected = Some(hashmap!{
      "a".to_string() => vec!["b".to_string()],
      "c".to_string() => vec!["d".to_string()]
    });
    let actual = Some(hashmap!{
      "c".to_string() => vec!["d".to_string()]
    });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::HeaderMismatch {
      key: "a".to_string(),
      expected: "\"b\"".to_string(),
      actual: "".to_string(),
      mismatch: "Expected a header 'a' but was missing".to_string(),
    });
  }

  #[test]
  fn match_headers_returns_a_mismatch_if_there_is_an_empty_expected_header_and_a_non_empty_actual() {
    let expected = Some(hashmap!{
      "a".to_string() => vec!["b".to_string()],
      "c".to_string() => vec![]
    });
    let actual = Some(hashmap!{
      "a".to_string() => vec!["b".to_string()],
      "c".to_string() => vec!["d".to_string()]
    });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::HeaderMismatch {
      key: "c".to_string(),
      expected: "".to_string(),
      actual: "d".to_string(),
      mismatch: "Expected an empty header 'c' but actual value was 'd'".to_string(),
    });
  }

  #[test]
  fn match_headers_returns_a_mismatch_if_the_header_values_have_different_lengths() {
    let expected = Some(hashmap!{
      "a".to_string() => vec!["b".to_string()],
      "c".to_string() => vec!["d".to_string(), "e".to_string()]
    });
    let actual = Some(hashmap!{
      "a".to_string() => vec!["b".to_string()],
      "c".to_string() => vec!["d".to_string()]
    });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.len()).to(be_equal_to(1));
    expect!(mismatches[0].clone()).to(be_equal_to(Mismatch::HeaderMismatch {
      key: "c".to_string(),
      expected: "e".to_string(),
      actual: "".to_string(),
      mismatch: "Mismatch with header 'c': Expected value 'e' at index 1".to_string(),
    }));

    let expected = Some(hashmap!{
      "c".to_string() => vec!["d".to_string(), "e".to_string()]
    });
    let actual = Some(hashmap!{
      "c".to_string() => vec!["e".to_string()]
    });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.len()).to(be_equal_to(2));
    expect!(mismatches[0].clone()).to(be_equal_to(Mismatch::HeaderMismatch {
      key: "c".to_string(),
      expected: "d".to_string(),
      actual: "e".to_string(),
      mismatch: "Mismatch with header 'c': Expected 'd' to be equal to 'e' for value at index 0".to_string(),
    }));
    expect!(mismatches[1].clone()).to(be_equal_to(Mismatch::HeaderMismatch {
      key: "c".to_string(),
      expected: "e".to_string(),
      actual: "".to_string(),
      mismatch: "Mismatch with header 'c': Expected value 'e' at index 1 but was missing (actual has 1 value(s))".to_string(),
    }));
  }

  #[test_log::test]
  fn match_header_with_min_type_matching_rules() {
    let expected = hashmap! { "id".to_string() => vec!["1".to_string(), "2".to_string()] };
    let actual = hashmap! { "id".to_string() => vec![
      "1".to_string(),
      "2".to_string(),
      "3".to_string(),
      "4".to_string()
    ]};
    let rules = matchingrules! {
      "header" => { "id" => [ MatchingRule::MinType(2) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    );
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn match_header_with_min_type_matching_rules_fails() {
    let expected = hashmap! { "id".to_string() => vec![
      "1".to_string(),
      "2".to_string(),
      "3".to_string(),
      "4".to_string()
    ]};
    let actual = hashmap! { "id".to_string() => vec!["1".to_string()] };
    let rules = matchingrules! {
      "header" => { "id" => [ MatchingRule::MinType(2) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("header").unwrap_or_default(),
      &hashmap!{}
    );

    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.get("id").unwrap().to_vec()).to(be_equal_to(vec![
      Mismatch::HeaderMismatch {
        key: "$.id".to_string(),
        expected: "[\"1\",\"2\",\"3\",\"4\"]".to_string(),
        actual: "[\"1\"]".to_string(),
        mismatch: "Expected [1] (size 1) to have minimum size of 2".to_string(),
      }
    ]));
  }

  #[test_log::test]
  fn last_modified_header_matches_when_headers_are_equal() {
    let expected = hashmap! { "Last-Modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:35 GMT".to_string()] };
    let actual = hashmap! { "Last-Modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:35 GMT".to_string()]};
    let context = CoreMatchingContext::default();
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test_log::test]
  fn last_modified_header_does_not_match_when_headers_are_not_equal() {
    let expected = hashmap! { "Last-Modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:35 GMT".to_string()] };
    let actual = hashmap! { "Last-Modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:52 GMT".to_string()]};
    let context = CoreMatchingContext::default();
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to_not(be_empty());
  }

  #[test_log::test]
  fn matching_last_modified_header_with_a_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "Last-Modified" => [ MatchingRule::Regex("^[A-Za-z]{3},\\s\\d{2}\\s[A-Za-z]{3}\\s\\d{4}\\s\\d{2}:\\d{2}:\\d{2}\\sGMT$".to_string()) ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! { "last-modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:35 GMT".to_string()] };
    let actual = hashmap! { "Last-Modified".to_string() => vec!["Sun, 12 Mar 2023 01:21:52 GMT".to_string()]};
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  // Issue #305
  #[test_log::test]
  fn content_type_header_mismatch_when_multiple_values() {
    let result = match_header_value("CONTENT-TYPE", 1, "application/json;charset=UTF-8",
      "application/xml;charset=UTF-8", &CoreMatchingContext::default(), false
    );
    let mismatches = result.unwrap_err();
    assert_eq!(mismatches[0].description, "Mismatch with header 'CONTENT-TYPE': Expected header 'CONTENT-TYPE' at index 1 to have value 'application/json;charset=UTF-8' but was 'application/xml;charset=UTF-8'");
  }

  // Issue #331
  #[test_log::test]
  fn match_header_with_a_values_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "X-IMPROVED" => [ MatchingRule::Values ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! {
      "X-IMPROVED".to_string() => vec![
        "like".to_string(),
        "regex".to_string(),
        "values".to_string(),
        "arrayContaining".to_string()
      ]
    };
    let wrong_order = hashmap! {
      "X-IMPROVED".to_string() => vec![
        "regex".to_string(),
        "like".to_string(),
        "values".to_string(),
        "arrayContaining".to_string()
      ]
    };
    let result = match_headers(Some(expected.clone()), Some(wrong_order), &context);
    expect!(result.values().flatten()).to_not(be_empty());

    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches.clone()).to(be_equal_to(vec![
      Mismatch::HeaderMismatch {
        key: "X-IMPROVED".to_string(),
        expected: "like".to_string(),
        actual: "regex".to_string(),
        mismatch: "Mismatch with header 'X-IMPROVED': Expected 'regex' to be equal to 'like' for value at index 0".to_string(),
      },
      Mismatch::HeaderMismatch {
        key: "X-IMPROVED".to_string(),
        expected: "regex".to_string(),
        actual: "like".to_string(),
        mismatch: "Mismatch with header 'X-IMPROVED': Expected 'like' to be equal to 'regex' for value at index 1".to_string(),
      }
    ]));

    let actual = hashmap! {
      "X-IMPROVED".to_string() => vec![
        "like".to_string(),
        "regex".to_string(),
        "values".to_string(),
        "arrayContaining".to_string()
      ]
    };
    let result = match_headers(Some(expected.clone()), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn match_headers_with_array_contains_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "X-Id" => [
            MatchingRule::ArrayContains(vec![])
          ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! { "X-Id".to_string() => vec!["1".to_string(), "3".to_string()] };
    let actual = hashmap! { "X-Id".to_string() => vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]};
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn match_headers_with_array_contains_matcher_fails() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "X-Id" => [
            MatchingRule::ArrayContains(vec![])
          ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! { "X-Id".to_string() => vec!["1".to_string(), "3".to_string()] };
    let actual = hashmap! { "X-Id".to_string() => vec!["2".to_string(), "3".to_string(), "4".to_string()]};
    let result = match_headers(Some(expected), Some(actual), &context);
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches).to(be_equal_to(vec![
      Mismatch::HeaderMismatch {
        key: "$['x-id']".to_string(),
        expected: "1".to_string(),
        actual: "[\"2\",\"3\",\"4\"]".to_string(),
        mismatch: "Variant at index 0 (1) was not found in the actual list".to_string(),
      }
    ]));
  }

  #[test]
  fn match_headers_with_each_value_matcher() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "X-Id" => [
            MatchingRule::EachValue(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
              MatchingRule::Regex("\\d+".to_string()), None, "".to_string()))
          ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! { "X-Id".to_string() => vec!["1".to_string(), "2".to_string()] };
    let actual = hashmap! { "X-Id".to_string() => vec!["3".to_string(), "4".to_string(), "567".to_string()]};
    let result = match_headers(Some(expected), Some(actual), &context);
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn match_headers_with_each_value_matcher_fails() {
    let context = HeaderMatchingContext::new(&CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "X-Id" => [
            MatchingRule::EachValue(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
              MatchingRule::Regex("\\d+".to_string()), None, "".to_string()))
          ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    ));
    let expected = hashmap! { "X-Id".to_string() => vec!["1".to_string(), "2".to_string()] };
    let actual = hashmap! { "X-Id".to_string() => vec!["3".to_string(), "abc123".to_string(), "test".to_string()]};
    let result = match_headers(Some(expected), Some(actual), &context);
    let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
    expect!(mismatches).to(be_equal_to(vec![
      Mismatch::HeaderMismatch {
        key: "X-Id".to_string(),
        expected: "2".to_string(),
        actual: "abc123".to_string(),
        mismatch: "Mismatch with header 'X-Id': Expected 'abc123' to match '\\d+' for value at index 0".to_string(),
      },
      Mismatch::HeaderMismatch {
        key: "X-Id".to_string(),
        expected: "1".to_string(),
        actual: "test".to_string(),
        mismatch: "Mismatch with header 'X-Id': Expected 'test' to match '\\d+' for value at index 0".to_string(),
      }
    ]));
  }
}
