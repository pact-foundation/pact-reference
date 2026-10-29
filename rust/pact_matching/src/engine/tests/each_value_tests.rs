use bytes::Bytes;
use pretty_assertions::assert_eq;
use rstest::rstest;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, MatchingRules, RuleLogic};
use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
use pact_models::path_exp::DocPath;
use pact_models::prelude::v4::SynchronousHttp;
use pact_models::v4::http_parts::HttpResponse;
use pact_models::v4::interaction::V4Interaction;

use crate::engine::{build_response_plan, execute_response_plan, PlanMatchingContext};
use crate::engine::context::MatchingConfiguration;
use crate::Mismatch;

fn each_value(example: &str, rule: MatchingRule) -> MatchingRule {
  MatchingRule::EachValue(MatchingRuleDefinition::new(example.to_string(), ValueType::Unknown,
    rule, None, "".to_string()))
}

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

#[test_log::test]
fn applies_the_rules_defined_below_an_each_value_matcher_on_a_map_to_its_values() {
  let rules = body_rules(vec![
    ("$.chips", each_value("{\"main\":{\"amount\":1000}}", MatchingRule::Type), RuleLogic::And),
    ("$.chips.*.amount", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"{"chips":{"main":{"amount":1000}}}"#;

  assert_eq!(Vec::<String>::new(),
    mismatches(&rules, expected, r#"{"chips":{"main":{"amount":500},"pair":{"amount":100}}}"#));
  assert_eq!(vec!["$.chips.pair.amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"chips":{"main":{"amount":500},"pair":{"amount":1.5}}}"#));
}

#[test_log::test]
fn applies_the_rules_defined_below_an_each_value_matcher_on_a_list_to_its_values() {
  let rules = body_rules(vec![
    ("$.items", each_value("[{\"amount\":1000}]", MatchingRule::Type), RuleLogic::And),
    ("$.items[*].amount", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"{"items":[{"amount":1000}]}"#;

  assert_eq!(Vec::<String>::new(),
    mismatches(&rules, expected, r#"{"items":[{"amount":500},{"amount":100}]}"#));
  assert_eq!(vec!["$.items[1].amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"items":[{"amount":500},{"amount":1.5}]}"#));
}

#[test_log::test]
fn applies_the_rules_below_an_each_value_matcher_on_a_map_at_the_root() {
  let rules = body_rules(vec![
    ("$", each_value("{\"main\":{\"amount\":1000}}", MatchingRule::Type), RuleLogic::And),
    ("$.*.amount", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"{"main":{"amount":1000}}"#;

  assert_eq!(Vec::<String>::new(),
    mismatches(&rules, expected, r#"{"main":{"amount":500},"pair":{"amount":100}}"#));
  assert_eq!(vec!["$.pair.amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, expected, r#"{"main":{"amount":500},"pair":{"amount":1.5}}"#));
}

#[test_log::test]
fn applies_the_rules_below_an_each_value_matcher_on_a_list_at_the_root() {
  let rules = body_rules(vec![
    ("$", each_value("[{\"amount\":1000}]", MatchingRule::Type), RuleLogic::And),
    ("$[*].amount", MatchingRule::Integer, RuleLogic::And)
  ]);
  let expected = r#"[{"amount":1000}]"#;

  assert_eq!(Vec::<String>::new(),
    mismatches(&rules, expected, r#"[{"amount":500},{"amount":100}]"#));
  assert_eq!(vec!["$[1].amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, expected, r#"[{"amount":500},{"amount":1.5}]"#));
}

#[test_log::test]
fn uses_the_map_keys_for_the_paths_of_the_values_including_keys_that_need_to_be_quoted() {
  let rules = body_rules(vec![
    ("$.chips", each_value("{\"main\":{\"amount\":1000}}", MatchingRule::Type), RuleLogic::And),
    ("$.chips.*.amount", MatchingRule::Integer, RuleLogic::And)
  ]);

  assert_eq!(vec!["$.chips['big one'].amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, r#"{"chips":{"main":{"amount":1000}}}"#, r#"{"chips":{"big one":{"amount":1.5}}}"#));
}

#[test_log::test]
fn still_applies_the_each_value_rules_to_map_values_that_are_not_objects_or_lists() {
  let rules = body_rules(vec![
    ("$.names", each_value("{\"a\":\"alice\"}", MatchingRule::Regex("^[a-z]+$".to_string())), RuleLogic::And)
  ]);
  let expected = r#"{"names":{"a":"alice"}}"#;

  assert_eq!(Vec::<String>::new(),
    mismatches(&rules, expected, r#"{"names":{"x":"bob","y":"carol"}}"#));
  let result = mismatches(&rules, expected, r#"{"names":{"x":"Bob1"}}"#);
  // reported once, not once for the each value rule and again for the values
  assert_eq!(1, result.len(), "{:?}", result);
}

#[rstest]
#[case("map", "$", "{\"main\":{\"amount\":1000}}", "{\"main\":{\"amount\":1.5}}", "{\"main\":\"1000\"}")]
#[case("list", "$", "[{\"amount\":1000}]", "[{\"amount\":1.5}]", "[\"1000\"]")]
#[test_log::test]
fn still_applies_the_each_value_rules_to_the_values_when_there_are_no_rules_below_them(
  #[case] _kind: &str,
  #[case] path: &str,
  #[case] example: &str,
  #[case] valid: &str,
  #[case] invalid: &str
) {
  let rules = body_rules(vec![(path, each_value(example, MatchingRule::Type), RuleLogic::And)]);

  assert_eq!(Vec::<String>::new(), mismatches(&rules, example, valid));
  assert!(!mismatches(&rules, example, invalid).is_empty());
}

#[test_log::test]
fn merges_the_each_value_rules_with_rules_for_the_list_items() {
  let rules = body_rules(vec![
    ("$.items", each_value("[{\"amount\":1000}]", MatchingRule::Type), RuleLogic::And),
    ("$.items[*]", MatchingRule::Type, RuleLogic::And),
    ("$.items[*].amount", MatchingRule::Integer, RuleLogic::And)
  ]);

  assert_eq!(vec!["$.items[1].amount: Expected 1.5 (Decimal) to be an integer".to_string()],
    mismatches(&rules, r#"{"items":[{"amount":1000}]}"#, r#"{"items":[{"amount":500},{"amount":1.5}]}"#));
}

#[test_log::test]
fn does_not_combine_the_each_value_rules_with_rules_for_the_values_that_use_or_logic() {
  let rules = body_rules(vec![
    ("$.names", each_value("{\"a\":\"alice\"}", MatchingRule::Regex("^[a-z]+$".to_string())), RuleLogic::And),
    ("$.names.*", MatchingRule::Regex("^a.*".to_string()), RuleLogic::Or),
    ("$.names.*", MatchingRule::Regex("^b.*".to_string()), RuleLogic::Or)
  ]);
  let expected = r#"{"names":{"a":"alice"}}"#;

  assert_eq!(Vec::<String>::new(), mismatches(&rules, expected, r#"{"names":{"x":"bob","y":"carol"}}"#));
  assert!(!mismatches(&rules, expected, r#"{"names":{"x":"bob1"}}"#).is_empty());
}

#[test_log::test]
fn does_not_apply_rules_for_other_paths_to_the_values_of_an_each_value_matcher() {
  let rules = body_rules(vec![
    ("$.chips", each_value("{\"main\":{\"amount\":1000}}", MatchingRule::Type), RuleLogic::And),
    ("$.other.amount", MatchingRule::Integer, RuleLogic::And)
  ]);

  assert_eq!(Vec::<String>::new(), mismatches(&rules,
    r#"{"chips":{"main":{"amount":1000}},"other":{"amount":1}}"#,
    r#"{"chips":{"main":{"amount":1.5}},"other":{"amount":1}}"#));
}
