use std::collections::HashMap;

use expectest::prelude::*;
use pretty_assertions::{assert_eq, assert_ne};

use pact_models::{matchingrules, matchingrules_list};
use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{JSON, TEXT};
use pact_models::HttpStatus;
use pact_models::request::Request;

use super::*;

#[test]
fn match_method_returns_nothing_if_the_method_matches() {
  expect!(match_method(&"GET".to_string(), &"GET".to_string())).to(be_ok());
}

#[test]
fn match_method_returns_a_mismatch_if_the_method_does_not_match() {
  expect!(match_method(&"GET".to_string(), &"POST".to_string())).to(
    be_err().value(Mismatch::MethodMismatch { expected: "GET".into(), actual: "POST".into() }));
}

#[test]
fn match_method_returns_nothing_if_the_method_matches_with_different_case() {
  expect!(match_method(&"POST".to_string(), &"post".to_string())).to(be_ok());
}

#[test]
fn match_status_returns_nothing_if_the_status_matches() {
  expect!(match_status(200, 200, &CoreMatchingContext::default())).to(be_ok());
}

#[test]
fn match_status_returns_a_mismatch_if_the_status_does_not_match() {
  expect!(match_status(200, 300, &CoreMatchingContext::default())).to(
    be_err().value(vec![Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }])
  );
}

#[test]
fn match_status_using_matchers() {
  let rules = matchingrules_list! {
    "status"; "" => [ MatchingRule::StatusCode(HttpStatus::Success) ]
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules, &hashmap!{}
  );
  expect!(match_status(200, 204, &context)).to(be_ok());
  let result = match_status(200, 500, &context);
  expect!(result.clone()).to(be_err().value(vec![Mismatch::StatusMismatch {
    expected: 200,
    actual: 500,
    mismatch: "".into()
  }]));
  expect!(result.unwrap_err().first().unwrap().description()).to(
    be_equal_to("Expected status code 500 to be a Successful response (200–299)"));
}

#[test]
fn match_query_returns_nothing_if_there_are_no_query_strings() {
  let expected = None;
  let actual = None;
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_applies_matching_rules_when_param_has_an_underscore() {
  let expected = hashmap! { "user_id".to_string() => vec![Some("1".to_string())] };
  let actual = hashmap! { "user_id".to_string() => vec![Some("2".to_string())] };
  let rules = matchingrules! {
    "query" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_no_expected_query_string() {
  let expected = None;
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "a".to_string(),
    expected: "".to_string(),
    actual: "[\"b\"]".to_string(),
    mismatch: "Unexpected query parameter 'a' received".to_string(),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_no_actual_query_string() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let expected = Some(query_map);
  let actual = None;
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "a".to_string(),
    expected: "[\"b\"]".to_string(),
    actual: "".to_string(),
    mismatch: "Expected query parameter 'a' but was missing".to_string()
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_actual_query_parameter_that_is_not_expected() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![Some("d".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "c".to_string(),
    expected: "".to_string(),
    actual: "[\"d\"]".to_string(),
    mismatch: "Unexpected query parameter 'c' received".to_string(),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_expected_query_parameter_that_is_not_received() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![Some("d".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "c".to_string(),
    expected: "[\"d\"]".to_string(),
    actual: "".to_string(),
    mismatch: "Expected query parameter 'c' but was missing".to_string(),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_empty_expected_query_parameter_and_a_non_empty_actual() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![Some("d".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "c".to_string(),
    expected: "[]".to_string(),
    actual: "[\"d\"]".to_string(),
    mismatch: "Expected an empty parameter list for 'c' but received [\"d\"]".to_string(),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_the_query_values_have_different_lengths() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![Some("d".to_string()), Some("e".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  query_map.insert("c".to_string(), vec![Some("d".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  assert_eq!(mismatches.len(), 2);
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: "c".to_string(),
    expected: "[\"d\", \"e\"]".to_string(),
    actual: "[\"d\"]".to_string(),
    mismatch: "Expected query parameter 'c' with 2 value(s) but received 1 value(s)".to_string(),
  });
  assert_eq!(mismatches[1], Mismatch::QueryMismatch {
    parameter: "c".to_string(),
    expected: "[\"d\", \"e\"]".to_string(),
    actual: "[\"d\"]".to_string(),
    mismatch: "Expected query parameter 'c' value 'e' but was missing".to_string(),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_the_values_are_not_the_same() {
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("c".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &CoreMatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches.first().unwrap(), &Mismatch::QueryMismatch {
    parameter: "a".to_string(),
    expected: "b".to_string(),
    actual: "c".to_string(),
    mismatch: "Expected 'b' but received 'c' for query parameter 'a'".to_string()
  });
}

#[test]
fn match_query_with_min_type_matching_rules() {
  let expected = hashmap! { "id".to_string() => vec![Some("1".to_string()), Some("2".to_string())] };
  let actual = hashmap! { "id".to_string() => vec![
    Some("1".to_string()),
    Some("2".to_string()),
    Some("3".to_string()),
    Some("4".to_string())
  ]};
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::MinType(2) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_with_min_type_matching_rules_fails() {
  let expected = hashmap! { "id".to_string() => vec![
    Some("1".to_string()),
    Some("2".to_string()),
    Some("3".to_string()),
    Some("4".to_string())
  ]};
  let actual = hashmap! { "id".to_string() => vec![Some("1".to_string())] };
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::MinType(2) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.get("id").unwrap().to_vec()).to(be_equal_to(vec![
    Mismatch::QueryMismatch {
      parameter: "$.id".to_string(),
      expected: "[\"1\",\"2\",\"3\",\"4\"]".to_string(),
      actual: "[\"1\"]".to_string(),
      mismatch: "Expected [1] (size 1) to have minimum size of 2".to_string(),
    }
  ]));
}

#[test]
fn match_query_returns_no_mismatch_if_the_values_are_not_the_same_but_match_by_a_matcher() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "query" => {
        "a" => [ MatchingRule::Regex("\\w+".to_string()) ]
      }
    }.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("c".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &context);
  expect!(result.get("a").unwrap().iter()).to(be_empty());
}

#[test]
fn match_query_returns_a_mismatch_if_the_values_do_not_match_by_a_matcher() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "query" => {
        "a" => [ MatchingRule::Regex("\\d+".to_string()) ]
      }
    }.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("b".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &context);
  expect!(result.iter()).to_not(be_empty());
  assert_eq!(result.get("a").unwrap()[0], Mismatch::QueryMismatch {
    parameter: "a".to_string(),
    expected: "b".to_string(),
    actual: "b".to_string(),
    mismatch: "Expected 'b' to match '\\d+'".to_string()
  });
}

#[test]
fn match_query_with_query_parameters_with_brackets() {
  let matching_rules_json = json!({
    "matchingRules": {
      "query": {
        "Q[]": {
          "matchers": [
            { "match": "regex", "regex": "\\d+" }
          ]
        }
      }
    }
  });

  let matching_rules = matchingrules::matchers_from_json(&matching_rules_json, &None).unwrap();
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matching_rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let expected = hashmap!{
    "Q[]".to_string() => vec![Some("1".to_string()), Some("2".to_string())]
  };
  let actual = hashmap!{
    "Q[]".to_string() => vec![Some("100".to_string()), Some("2000".to_string()), Some("999".to_string())]
  };
  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.get("Q[]").unwrap().iter()).to(be_empty());
}


#[test]
fn match_query_with_array_contains_matching_rules() {
  let expected = hashmap! { "id".to_string() => vec![Some("1".to_string()), Some("3".to_string())] };
  let actual = hashmap! { "id".to_string() => vec![
    Some("1".to_string()),
    Some("2".to_string()),
    Some("3".to_string()),
    Some("4".to_string())
  ]};
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::ArrayContains(vec![]) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_with_array_contains_matching_rules_fails() {
  let expected = hashmap! { "id".to_string() => vec![Some("1".to_string()), Some("3".to_string())] };
  let actual = hashmap! { "id".to_string() => vec![Some("2".to_string()), Some("3".to_string()), Some("4".to_string())] };
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::ArrayContains(vec![]) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.get("id").unwrap().to_vec()).to(be_equal_to(vec![
    Mismatch::QueryMismatch {
      parameter: "$.id".to_string(),
      expected: "1".to_string(),
      actual: "[\"2\",\"3\",\"4\"]".to_string(),
      mismatch: "Variant at index 0 (1) was not found in the actual list".to_string(),
    }
  ]));
}

#[test]
fn match_query_with_each_value_matching_rules() {
  let expected = hashmap! { "id".to_string() => vec![Some("1".to_string()), Some("2".to_string())] };
  let actual = hashmap! { "id".to_string() => vec![Some("3".to_string()), Some("4".to_string()), Some("567".to_string())] };
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::EachValue(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
      MatchingRule::Regex("\\d+".to_string()), None, "".to_string())) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_with_each_value_matching_rules_fails() {
  let expected = hashmap! { "id".to_string() => vec![Some("1".to_string()), Some("2".to_string())] };
  let actual = hashmap! { "id".to_string() => vec![Some("3".to_string()), Some("abc123".to_string()), Some("test".to_string())] };
  let rules = matchingrules! {
    "query" => { "id" => [ MatchingRule::EachValue(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
      MatchingRule::Regex("\\d+".to_string()), None, "".to_string())) ] }
  };
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(),
    &hashmap!{}
  );

  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.get("id").unwrap().to_vec()).to(be_equal_to(vec![
    Mismatch::QueryMismatch {
      parameter: "id".to_string(),
      expected: "2".to_string(),
      actual: "abc123".to_string(),
      mismatch: "Expected 'abc123' to match '\\d+'".to_string(),
    },
    Mismatch::QueryMismatch {
      parameter: "id".to_string(),
      expected: "1".to_string(),
      actual: "test".to_string(),
      mismatch: "Expected 'test' to match '\\d+'".to_string(),
    }
  ]));
}

// Issue 483
#[test_log::test]
fn match_query_with_a_time_matcher() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "query" => {
        "a" => [ MatchingRule::Time("HH:mm".to_string()) ]
      }
    }.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let mut query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("12:13".to_string())]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert("a".to_string(), vec![Some("11:11".to_string())]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &context);
  expect!(result.get("a").unwrap().iter()).to(be_empty());
}

#[tokio::test]
async fn body_does_not_match_if_different_content_types() {
  let expected = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(Bytes::new(), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["text/plain".to_string()] }),
    body: OptionalBody::Missing,
    ..Request::default()
  };
  let result = match_body(&expected, &actual, &CoreMatchingContext::default(),
                          &CoreMatchingContext::default()).await;
  let mismatches = result.mismatches();
  expect!(mismatches.iter()).to_not(be_empty());
  expect!(mismatches[0].clone()).to(be_equal_to(Mismatch::BodyTypeMismatch {
    expected: "application/json".to_string(),
    actual: "text/plain".to_string(),
    mismatch: "".to_string(),
    expected_body: None,
    actual_body: None
  }));
}

#[tokio::test]
async fn body_matching_uses_any_matcher_for_content_type_header() {
  let expected = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(Bytes::from("100"), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/hal+json".to_string()] }),
    body: OptionalBody::Present(Bytes::from("100"), None, None),
    ..Request::default()
  };
  let header_context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "header" => { "Content-Type" => [ MatchingRule::Regex("application/.*json".into()) ] }
    }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
  );
  let result = match_body(&expected, &actual, &CoreMatchingContext::default(), &header_context).await;
  let mismatches = result.mismatches();
  expect!(mismatches.iter()).to(be_empty());
}

#[tokio::test]
async fn body_matches_if_expected_is_missing() {
  let expected = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Missing,
    ..Request::default()
  };
  let actual = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present("{}".into(), None, None),
    ..Request::default()
  };
  let result = match_body(&expected, &actual, &CoreMatchingContext::default(), &CoreMatchingContext::default()).await;
  expect!(result.mismatches().iter()).to(be_empty());
}

#[tokio::test]
async fn body_matches_with_extended_mime_types() {
  let expected = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/thrift+json".to_string()] }),
    body: OptionalBody::Present(r#"{"test":true}"#.into(), None, None),
    ..Request::default()
  };
  let expected2 = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(r#"{"test":true}"#.into(), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/thrift+json".to_string()] }),
    body: OptionalBody::Present(r#"{"test": true}"#.into(), None, None),
    ..Request::default()
  };
  let actual2 = Request {
    method: "GET".to_string(),
    path: "/".to_string(),
    query: None,
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(r#"{"test": true}"#.into(), None, None),
    ..Request::default()
  };
  let result = match_body(&expected, &actual, &CoreMatchingContext::default(), &CoreMatchingContext::default()).await;
  expect!(result.mismatches().iter()).to(be_empty());
  let result2 = match_body(&expected, &actual2, &CoreMatchingContext::default(), &CoreMatchingContext::default()).await;
  expect!(result2.mismatches().iter()).to_not(be_empty());
  let result3 = match_body(&expected2, &actual, &CoreMatchingContext::default(), &CoreMatchingContext::default()).await;
  expect!(result3.mismatches().iter()).to(be_empty());
}

#[tokio::test]
async fn body_matches_with_nested_matchers() {
  let expected = Request {
    body: OptionalBody::Present(r#"{"some-string":{"some-string":{"some-string":"some string"}}}"#.into(), Some(JSON.clone()), None),
    ..Request::default()
  };
  let actual = Request {
    body: OptionalBody::Present(r#"{
  "john-doe1": {
    "brown-fox": {
      "jumps": "over",
      "the": "lazy dog"
    }
  },
  "john-doe2": {
    "brown-fox2": {
      "jumps": "over",
      "the": "lazy dog"
    }
  }
}"#.into(), Some(JSON.clone()), None),
    ..Request::default()
  };
  let rules = matchingrules! {
    "body" => {
      "$" => [ MatchingRule::Values ],
      "$.*" => [ MatchingRule::Values ],
      "$.*.*" => [ MatchingRule::Values ],
      "$.*.*.*" => [ MatchingRule::Type ]
    }
  };
  let category = rules.rules_for_category("body").unwrap();
  let matching_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &category, &hashmap!{});
  let result = match_body(&expected, &actual, &matching_context, &CoreMatchingContext::default()).await;
  expect!(result.mismatches()).to(be_equal_to(vec![]));
}

// Issue #484
#[test_log::test(tokio::test)]
async fn body_matching_with_number_matchers() {
  let expected = Request {
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(r#"{"key2":456,"key1":321.1}"#.into(), None, None),
    ..Request::default()
  };

  let actual = Request {
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(r#"{"key2":789,"key1":432.1}"#.into(), None, None),
    ..Request::default()
  };
  let actual2 = Request {
    headers: Some(hashmap! { "Content-Type".to_string() => vec!["application/json".to_string()] }),
    body: OptionalBody::Present(r#"{"key2":"456","key1":"321.1"}"#.into(), None, None),
    ..Request::default()
  };

  let rules = matchingrules! {
    "body" => {
      "$" => [ MatchingRule::Type ],
      "$.key1" => [ MatchingRule::Number ],
      "$.key2" => [ MatchingRule::Number ]
    }
  };
  let category = rules.rules_for_category("body").unwrap();
  let matching_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &category, &hashmap!{});

  let result = match_body(&expected, &actual, &matching_context, &CoreMatchingContext::default()).await;
  expect!(result.mismatches().iter()).to(be_empty());

  let result2 = match_body(&expected, &actual2, &matching_context, &CoreMatchingContext::default()).await;
  assert_ne!(result2.mismatches(), vec![]);
}

#[test]
fn partial_equal_for_method_mismatch() {
  let mismatch = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") };
  let mismatch2 = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") };
  let mismatch3 = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("put") };
  let mismatch4 = Mismatch::MethodMismatch { expected: s!("post"), actual: s!("post") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: "".into(), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: "".into(), path: s!("/") }));
}

#[test]
fn partial_equal_for_path_mismatch() {
  let mismatch = Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() };
  let mismatch2 = Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() };
  let mismatch3 = Mismatch::PathMismatch { expected: s!("get"), actual: s!("put"), mismatch: "".into() };
  let mismatch4 = Mismatch::PathMismatch { expected: s!("post"), actual: s!("post"), mismatch: "".into() };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: "".into(), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: "".into(), path: s!("/") }));
}

#[test]
fn partial_equal_for_status_mismatch() {
  let mismatch = Mismatch::StatusMismatch { expected: 100, actual: 200, mismatch: "".into() };
  let mismatch2 = Mismatch::StatusMismatch { expected: 100, actual: 200, mismatch: "".into() };
  let mismatch3 = Mismatch::StatusMismatch { expected: 100, actual: 300, mismatch: "".into() };
  let mismatch4 = Mismatch::StatusMismatch { expected: 200, actual: 100, mismatch: "".into() };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!("200"), actual: s!("300"), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_body_type_mismatch() {
  let mismatch = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch2 = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch3 = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("put"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch4 = Mismatch::BodyTypeMismatch { expected: s!("post"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_query_mismatch() {
  let mismatch = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch2 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch3 = Mismatch::QueryMismatch { parameter: s!("key2"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch4 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v100"), actual: s!("v2"), mismatch: s!("") };
  let mismatch5 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v200"), mismatch: s!("") };
  let mismatch6 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_header_mismatch() {
  let mismatch = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch2 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch3 = Mismatch::HeaderMismatch { key: s!("key2"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch4 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v100"), actual: s!("v2"), mismatch: s!("") };
  let mismatch5 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v200"), mismatch: s!("") };
  let mismatch6 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_body_mismatch() {
  let mismatch = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch2 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch3 = Mismatch::BodyMismatch { path: s!("key2"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch4 = Mismatch::BodyMismatch { path: s!("key"), expected: None, actual: Some("v2".into()), mismatch: s!("") };
  let mismatch5 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: None, mismatch: s!("") };
  let mismatch6 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!("get"), actual: s!("post"), mismatch: s!("") }));
}

#[test]
fn match_path_returns_nothing_if_the_path_matches() {
  let context = CoreMatchingContext::default();
  let result = match_path(&"/path/one".to_string(), &"/path/one".to_string(), &context);
  expect!(result).to(be_ok());
}

#[test]
fn match_path_returns_a_mismatch_if_the_path_does_not_match() {
  let context = CoreMatchingContext::default();
  let result = match_path(&"/path/one".to_string(), &"/path/two".to_string(), &context);
  expect!(result).to(be_err().value(vec![ Mismatch::PathMismatch {
    expected: s!("/path/one"),
    actual: s!("/path/two"),
    mismatch: s!(""),
  } ]));
}

#[test]
fn match_path_returns_nothing_if_the_path_matches_with_a_matcher() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "path" => { "" => [ MatchingRule::Regex(s!("/path/\\d+")) ] }
    }.rules_for_category("path").unwrap_or_default(), &hashmap!{}
  );
  let result = match_path(&"/path/1234".to_string(), &"/path/5678".to_string(), &context);
  expect!(result).to(be_ok());
}

#[test]
fn match_path_returns_a_mismatch_if_the_path_does_not_match_with_a_matcher() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "path" => { "" => [ MatchingRule::Regex(s!("/path/\\d+")) ] }
    }.rules_for_category("path").unwrap_or_default(), &hashmap!{}
  );
  let result = match_path(&"/path/1234".to_string(), &"/path/abc".to_string(), &context);
  expect!(result).to(be_err().value(vec![ Mismatch::PathMismatch {
    expected: s!("/path/1234"),
    actual: s!("/path/abc"),
    mismatch: s!(""),
  }]));
}

macro_rules! request {
  ($e:expr) => (Request { body: OptionalBody::Present($e.into(), None, None), .. Request::default() })
}

#[tokio::test]
async fn matching_text_body_be_true_when_bodies_are_equal() {
  let expected = request!("body value");
  let actual = request!("body value");
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual,
    &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys)).await;
  expect!(mismatches.mismatches().iter()).to(be_empty());
}

#[tokio::test]
async fn matching_text_body_be_false_when_bodies_are_not_equal() {
  let expected = request!("expected body value");
  let actual = request!("actual body value");
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual,
    &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys)).await.mismatches();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::BodyMismatch {
    path: s!("$"),
    expected: expected.body.value(),
    actual: actual.body.value(),
    mismatch: s!(""),
  });
}

#[tokio::test]
async fn matching_text_body_must_use_defined_matcher() {
  let expected = request!("expected body value");
  let actual = request!("actualbodyvalue");

  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex(s!("\\w+")) ]
      }
    }.rules_for_category("body").unwrap_or_default(), &hashmap!{}
  );
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual, &context).await;
  expect!(mismatches.mismatches().iter()).to(be_empty());

  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex(s!("\\d+")) ]
      }
    }.rules_for_category("body").unwrap_or_default(), &hashmap!{}
  );
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual, &context).await;
  expect!(mismatches.mismatches().iter()).to_not(be_empty());
}

#[test]
fn values_matcher_defined() {
  let context = CoreMatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Values ],
        "$.x" => [ MatchingRule::Type ],
        "$.y" => [ MatchingRule::Values ],
        "$.z" => [ MatchingRule::Type, MatchingRule::Values ],
        "$.x[*].y" => [ MatchingRule::Values ],
        "$.y[*].y" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{});

  let root = DocPath::root();
  let path_x = root.join("x");
  let path_y = root.join("y");
  let path_z = root.join("z");
  expect!(context.values_matcher_defined(&root)).to(be_true());
  expect!(context.values_matcher_defined(&path_x)).to(be_false());
  expect!(context.values_matcher_defined(&path_y)).to(be_true());
  expect!(context.values_matcher_defined(&path_z)).to(be_true());
  expect!(context.values_matcher_defined(&path_x.join("0").join("y"))).to(be_true());
  expect!(context.values_matcher_defined(&path_x.join("0").join("z"))).to(be_false());
  expect!(context.values_matcher_defined(&path_y.join("0").join("y"))).to(be_false());
}

const IMAGE_BYTES: [u8; 16] = [ 0o107, 0o111, 0o106, 0o070, 0o067, 0o141, 0o001, 0o000, 0o001,
  0o000, 0o200, 0o000, 0o000, 0o377, 0o377, 0o377 ];

#[test]
fn compare_bodies_core_should_check_for_content_type_matcher() {
  let content_type = ContentType::parse("application/gif").unwrap();
  let matching_rules = matchingrules!{ "body" => { "$" => [ MatchingRule::ContentType("application/gif".to_string()) ] } };
  let expected = Request {
    body: OptionalBody::Present(Bytes::from_static(&IMAGE_BYTES), Some(content_type.clone()), None),
    matching_rules: matching_rules.clone(),
    .. Request::default()
  };
  let actual = Request {
    body: OptionalBody::Present(Bytes::from_static(&IMAGE_BYTES), Some(content_type.clone()), None),
    .. Request::default()
  };
  let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &matching_rules.rules_for_category("body").unwrap(),
    &hashmap!{}
  );

  let result = compare_bodies_core(&content_type, &expected, &actual, &context);

  expect!(result.len()).to(be_equal_to(1));
  expect!(result.first().unwrap().description()).to(be_equal_to("$ -> Expected binary contents to have content type 'application/gif' but detected contents was 'image/gif'"));
}

#[test_log::test]
fn match_metadata_value_test() {
  let expected = json!("value-a");
  let actual = json!("value-a");
  let rules = matchingrules!{};
  let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category(Category::METADATA).unwrap_or_default(), &hashmap!{});

  let result = match_metadata_value("key", &expected, &actual, &context);
  expect!(result).to(be_ok());

  let actual_error = json!("value-b");
  let result = match_metadata_value("key", &expected, &actual_error, &context);
  expect!(result).to(be_err());
}

#[test_log::test]
fn match_metadata_value_with_matching_rule_test() {
  let expected = json!("value-a");
  let actual = json!("value-b");
  let rules = matchingrules!{
     "metadata" => { "key" => [ MatchingRule::Regex("value-[a-z]".to_string()) ] }
  };
  let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category(Category::METADATA).unwrap_or_default(), &hashmap!{});

  let result = match_metadata_value("key", &expected, &actual, &context);
  expect!(result).to(be_ok());

  let actual_error = json!("value-9");
  let result = match_metadata_value("key", &expected, &actual_error, &context);
  expect!(result).to(be_err());
}

#[test_log::test]
fn match_metadata_value_with_content_type_test() {
  let expected = json!("application/something");
  let actual = json!("application/something;a=b;c=d");
  let rules = matchingrules!{};
  let context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
    &rules.rules_for_category(Category::METADATA).unwrap_or_default(), &hashmap!{});

  let result = match_metadata_value("content-type", &expected, &actual, &context);
  expect!(result).to(be_ok());

  let expected = json!("application/something;a=b;c=d");
  let actual = json!("application/something");
  let result = match_metadata_value("key", &expected, &actual, &context);
  expect!(result).to(be_err());
}
