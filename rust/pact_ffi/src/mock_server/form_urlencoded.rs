//! Form UrlEncoded matching support

use std::collections::HashMap;
use serde_json::Value;
use tracing::{debug, error, trace};

use pact_models::generators::{GeneratorCategory, Generators};
use pact_models::generators::form_urlencoded::QueryParams;
use pact_models::matchingrules::MatchingRuleCategory;
use pact_models::path_exp::DocPath;

use crate::mock_server::bodies::process_json;

/// Process a JSON body with embedded matching rules and generators
pub fn process_form_urlencoded_json(body: String, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> String {
  trace!("process_form_urlencoded_json");
  let json = process_json(body, matching_rules, generators);
  debug!("form_urlencoded json: {json}");
  let values: Value = serde_json::from_str(json.as_str()).unwrap();
  debug!("form_urlencoded values: {values}");
  let params = convert_json_value_to_query_params(values, matching_rules, generators);
  debug!("form_urlencoded params: {:?}", params);
  serde_urlencoded::to_string(params).expect("could not serialize body to form urlencoded string")
}

fn convert_json_value_to_query_params(value: Value, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> QueryParams {
  let mut params: QueryParams = vec![];
  match value {
    Value::Object(map) => {
      for (key, value) in map.iter() {
        let path = DocPath::root().join(key);
        match value {
          Value::Number(value) => params.push((key.clone(), value.to_string())),
          Value::String(value) => params.push((key.clone(), value.to_string())),
          Value::Array(vec) => {
            for (index, value) in vec.iter().enumerate() {
              let path = DocPath::root().join(key).join_index(index);
              match value {
                Value::Number(value) => params.push((key.clone(), value.to_string())),
                Value::String(value) => params.push((key.clone(), value.to_string())),
                _ => handle_form_urlencoded_invalid_value(value, &path, matching_rules, generators),
              }
            }
          },
          _ => handle_form_urlencoded_invalid_value(value, &path, matching_rules, generators),
        }
      }
    },
    _ => ()
  }
  params
}

fn handle_form_urlencoded_invalid_value(value: &Value, path: &DocPath, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) {
  for key in matching_rules.clone().rules.keys() {
    if String::from(key).contains(&String::from(path)) {
      matching_rules.rules.remove(&key);
      generators.categories.entry(GeneratorCategory::BODY).or_insert(HashMap::new()).remove(&key);
    }
  }
  error!("Value '{:?}' is not supported in form urlencoded. Matchers and generators (if defined) are removed", value);
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use rstest::rstest;
  use serde_json::json;

  use pact_models::generators;
  use pact_models::generators::Generator;
  use pact_models::matchingrules_list;
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};

  use super::*;

  #[rstest]
  #[case(
    json!({ "": "empty key" }),
    "=empty+key",
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "": ["first", "second", "third"] }),
    "=first&=second&=third",
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "": { "pact:matcher:type": "includes", "value": "empty" } }),
    "",
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "number_value": -123.45 }),
    "number_value=-123.45".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "string_value": "hello world" }),
    "string_value=hello+world".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_values": [null, 234, "example text", {"key": "value"}, ["value 1", "value 2"]] }),
    "array_values=234&array_values=example+text".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "null_value": null }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "null_value_with_matcher": { "pact:matcher:type": "null" } }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "number_value_with_matcher": { "pact:matcher:type": "number", "min": 0, "max": 10, "value": 123 } }),
    "number_value_with_matcher=123".to_string(),
    matchingrules_list!{"body"; "$.number_value_with_matcher" => [MatchingRule::Number]},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "number_value_with_matcher_and_generator": { "pact:matcher:type": "number", "pact:generator:type": "RandomInt", "min": 0, "max": 10, "value": 123 } }),
    "number_value_with_matcher_and_generator=123".to_string(),
    matchingrules_list!{"body"; "$.number_value_with_matcher_and_generator" => [MatchingRule::Number]},
    generators! {"BODY" => {"$.number_value_with_matcher_and_generator" => Generator::RandomInt(0, 10)}}
  )]
  // Missing value => null will be used => but it is not supported, so matcher is removed.
  #[case(
    json!({ "number_matcher_only": { "pact:matcher:type": "number", "min": 0, "max": 10 } }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "string_value_with_matcher_and_generator": { "pact:matcher:type": "type", "value": "some string", "pact:generator:type": "RandomString", "size": 15 } }),
    "string_value_with_matcher_and_generator=some+string".to_string(),
    matchingrules_list!{"body"; "$.string_value_with_matcher_and_generator" => [MatchingRule::Type]},
    generators! {"BODY" => {"$.string_value_with_matcher_and_generator" => Generator::RandomString(15)}}
  )]
  #[case(
    json!({ "string_value_with_matcher": { "pact:matcher:type": "type", "value": "some string", "size": 15 } }),
    "string_value_with_matcher=some+string".to_string(),
    matchingrules_list!{"body"; "$.string_value_with_matcher" => [MatchingRule::Type]},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_values_with_matcher": { "pact:matcher:type": "eachValue", "value": ["string value"], "rules": [{ "pact:matcher:type": "type", "value": "string" }] } }),
    "array_values_with_matcher=string+value".to_string(),
    matchingrules_list!{"body"; "$.array_values_with_matcher" => [MatchingRule::EachValue(MatchingRuleDefinition::new("[\"string value\"]".to_string(), ValueType::Unknown, MatchingRule::Type, None))]},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_values_with_matcher_and_generator": [
      { "pact:matcher:type": "regex", "value": "a1", "pact:generator:type": "Regex", "regex": "\\w\\d" },
      { "pact:matcher:type": "decimal", "pact:generator:type": "RandomDecimal", "digits": 3, "value": 12.3 }
    ] }),
    "array_values_with_matcher_and_generator=a1&array_values_with_matcher_and_generator=12.3".to_string(),
    matchingrules_list!{
      "body";
      "$.array_values_with_matcher_and_generator[0]" => [MatchingRule::Regex("\\w\\d".to_string())],
      "$.array_values_with_matcher_and_generator[1]" => [MatchingRule::Decimal]
    },
    generators! {"BODY" => {
      "$.array_values_with_matcher_and_generator[0]" => Generator::Regex("\\w\\d".to_string()),
      "$.array_values_with_matcher_and_generator[1]" => Generator::RandomDecimal(3)
    }}
  )]
  #[case(
    json!({ "false": false }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "true": true }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_of_false": [false] }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_of_true": [true] }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_of_objects": [{ "key": "value" }] }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "array_of_arrays": [["value 1", "value 2"]] }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(
    json!({ "object_value": { "key": "value" } }),
    "".to_string(), matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(json!(
    { "boolean_with_matcher_and_generator": { "pact:matcher:type": "boolean", "value": true, "pact:generator:type": "RandomBoolean" } }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  #[case(json!(
    { "object_with_matcher_and_generator": { "pact:matcher:type": "type", "value": {"key": { "pact:matcher:type": "type", "value": "value", "pact:generator:type": "RandomString" }} } }),
    "".to_string(),
    matchingrules_list!{"body"; "$" => []},
    generators! {"BODY" => {}}
  )]
  fn process_form_urlencoded_json_test(#[case] json: Value, #[case] result: String, #[case] expected_matching_rules: MatchingRuleCategory, #[case] expected_generators: Generators) {
    let mut matching_rules = MatchingRuleCategory::empty("body");
    let mut generators = Generators::default();
    expect!(process_form_urlencoded_json(json.to_string(), &mut matching_rules, &mut generators)).to(be_equal_to(result));
    expect!(matching_rules).to(be_equal_to(expected_matching_rules));
    expect!(generators).to(be_equal_to(expected_generators));
  }
}
