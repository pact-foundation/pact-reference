//! Tests for matching rules combined with OR (`RuleLogic::Or`) in the V2 matching engine

use bytes::Bytes;
use pretty_assertions::assert_eq;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, MatchingRules, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::prelude::v4::SynchronousHttp;
use pact_models::v4::http_parts::HttpResponse;
use pact_models::v4::interaction::V4Interaction;

use crate::engine::{build_response_plan, execute_response_plan, PlanMatchingContext};
use crate::engine::context::MatchingConfiguration;
use crate::Mismatch;

fn body_rules(rules: Vec<(&str, MatchingRule, RuleLogic)>) -> MatchingRuleCategory {
  let mut category = MatchingRuleCategory::empty("body");
  for (path, rule, logic) in rules {
    category.add_rule(DocPath::new(path).unwrap(), rule, logic);
  }
  category
}

/// Matches the actual body to the expected one with the V2 engine, and returns the body mismatches
/// as `path: mismatch`
fn mismatches(rules: &MatchingRuleCategory, expected: &str, actual: &str) -> Vec<String> {
  let mut matching_rules = MatchingRules::default();
  matching_rules.add_rules("body", rules.clone());
  let expected_response = HttpResponse {
    body: OptionalBody::Present(Bytes::from(expected.to_string()), Some(JSON.clone()), None),
    matching_rules,
    .. HttpResponse::default()
  };
  let actual_response = HttpResponse {
    body: OptionalBody::Present(Bytes::from(actual.to_string()), Some(JSON.clone()), None),
    .. HttpResponse::default()
  };
  let interaction = SynchronousHttp {
    response: expected_response.clone(),
    .. SynchronousHttp::default()
  };
  let context = PlanMatchingContext {
    interaction: interaction.boxed_v4(),
    config: MatchingConfiguration {
      allow_unexpected_entries: true,
      log_plan_summary: false,
      .. MatchingConfiguration::default()
    },
    .. PlanMatchingContext::default()
  };
  let plan = build_response_plan(&expected_response, &context).unwrap();
  let executed_plan = execute_response_plan(&plan, &actual_response, &context).unwrap();
  let mismatches: Vec<Mismatch> = executed_plan.into();
  mismatches.iter()
    .filter_map(|m| match m {
      Mismatch::BodyMismatch { path, mismatch, .. } => Some(format!("{}: {}", path, mismatch)),
      _ => None
    })
    .collect()
}


fn regex(r: &str) -> MatchingRule { MatchingRule::Regex(r.to_string()) }

#[test_log::test]
fn scalar_with_type_or_null() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Type, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or)
  ]);
  let expected = r#"{"a":"string"}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":"other"}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":null}"#));
  assert_eq!(false, mismatches(&rules, expected, r#"{"a":100}"#).is_empty());
}

#[test_log::test]
fn scalar_with_two_regexes() {
  let rules = body_rules(vec![
    ("$.a", regex("^\\d+$"), RuleLogic::Or),
    ("$.a", regex("^[a-z]+$"), RuleLogic::Or)
  ]);
  let expected = r#"{"a":"100"}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":"200"}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":"abc"}"#));
  assert_eq!(false, mismatches(&rules, expected, r#"{"a":"ABC"}"#).is_empty());
}

#[test_log::test]
fn integer_or_decimal() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Integer, RuleLogic::Or),
    ("$.a", MatchingRule::Decimal, RuleLogic::Or)
  ]);
  let expected = r#"{"a":100}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":1}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":1.5}"#));
  assert_eq!(false, mismatches(&rules, expected, r#"{"a":"1"}"#).is_empty());
}

#[test_log::test]
fn object_with_type_or_null() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Type, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or)
  ]);
  let expected = r#"{"a":{"b":"string"}}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":{"b":"other"}}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":null}"#));
}

#[test_log::test]
fn array_with_type_or_null() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::MinType(1), RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or)
  ]);
  let expected = r#"{"a":[1]}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":[2,3]}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":null}"#));
}

#[test_log::test]
fn object_with_type_or_null_rejects_other_types() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Type, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or)
  ]);
  let expected = r#"{"a":{"b":"string"}}"#;
  let result = mismatches(&rules, expected, r#"{"a":"string"}"#);
  assert_eq!(false, result.is_empty());
  assert!(result.iter().all(|m| m.starts_with("$.a")), "{:?}", result);
}

#[test_log::test]
fn object_with_type_or_null_still_applies_the_nested_rules() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Type, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or),
    ("$.a.id", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"{"a":{"id":100}}"#;
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":{"id":200}}"#));
  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"a":null}"#));
  assert_eq!(vec!["$.a.id: Expected '200' (String) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"a":{"id":"200"}}"#));
  assert_eq!(false, mismatches(&rules, expected, r#"{"a":{}}"#).is_empty());
}

#[test_log::test]
fn a_successful_or_does_not_report_the_errors_from_the_other_branches() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Integer, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or),
    ("$.b", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"{"a":100,"b":100}"#;
  assert_eq!(vec!["$.b: Expected 'x' (String) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"a":null,"b":"x"}"#));
  assert_eq!(vec!["$.b: Expected 'x' (String) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"a":200,"b":"x"}"#));
}

#[test_log::test]
fn a_failed_or_reports_the_errors_from_all_the_branches() {
  let rules = body_rules(vec![
    ("$.a", MatchingRule::Integer, RuleLogic::Or),
    ("$.a", MatchingRule::Null, RuleLogic::Or)
  ]);
  let result = mismatches(&rules, r#"{"a":100}"#, r#"{"a":"x"}"#);
  assert!(result.iter().any(|m| m.contains("to be an integer")), "{:?}", result);
  assert!(result.iter().any(|m| m.to_lowercase().contains("null")), "{:?}", result);
}
