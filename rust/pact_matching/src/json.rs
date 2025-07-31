//! The `json` module provides functions to compare and display the differences between JSON bodies

use std::str::FromStr;

use ansi_term::Colour::*;
use anyhow::anyhow;
use difference::*;
use lazy_static::lazy_static;
use onig::Regex;
use semver::Version;
use serde_json::{json, Value};

use pact_models::http_parts::HttpPart;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;
#[cfg(feature = "datetime")] use pact_models::time_utils::validate_datetime;
use tracing::debug;

use crate::{DiffConfig, MatchingContext, Mismatch, CommonMismatch, merge_result};
use crate::binary_utils::{convert_data, match_content_type};
use crate::matchingrules::{
  compare_lists_with_matchingrules,
  compare_maps_with_matchingrule,
  DoMatch,
  match_values,
  Matches,
  value_for_mismatch
};

lazy_static! {
  static ref DEC_REGEX: Regex = Regex::new(r"\d+\.\d+").unwrap();
}

pub(crate) fn type_of(json: &Value) -> String {
  match json {
    Value::Object(_) => "Object",
    Value::Array(_) => "Array",
    Value::Null => "Null",
    Value::Bool(_) => "Boolean",
    Value::Number(n) => if n.is_i64() || n.is_u64() {
      "Integer"
    } else {
      "Decimal"
    },
    Value::String(_) => "String"
  }.to_string()
}

fn value_of(json: &Value) -> String {
  match json {
    Value::Null => "null".to_string(),
    Value::String(s) => format!("'{}'", s),
    Value::Bool(b) => b.to_string(),
    Value::Number(n) => n.to_string(),
    Value::Object(_) => json.to_string(),
    Value::Array(_) => json.to_string()
  }.to_string()
}

impl Matches<Value> for Value {
  fn matches_with(&self, actual: Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.matches_with(&actual, matcher, cascaded)
  }
}

impl Matches<&Value> for &Value {
  fn matches_with(&self, actual: &Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    (*self).matches_with(actual, matcher, cascaded)
  }
}

impl Matches<&Value> for Value {
  fn matches_with(&self, actual: &Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self, actual, cascaded, true)
  }
}

impl DoMatch<&Value> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &Value,
    actual_value: &Value,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    let result = match self {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            let actual_str = match actual_value {
              Value::String(s) => s.clone(),
              _ => actual_value.to_string()
            };
            if re.is_match(&actual_str) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", json_to_string(actual_value), regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Include(substr) => {
        let actual_str = match actual_value {
          Value::String(s) => s.clone(),
          _ => actual_value.to_string()
        };
        if actual_str.contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to include '{}'", json_to_string(actual_value), substr))
        }
      },
      MatchingRule::Type => {
        match (expected_value, actual_value) {
          (&Value::Array(_), &Value::Array(_)) => Ok(()),
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected {} ({}) to be the same type as {} ({})",
            value_of(actual_value), type_of(actual_value), value_of(expected_value), type_of(expected_value))),
        }
      },
      MatchingRule::MinType(min) => {
        match (expected_value, actual_value) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() < *min {
            Err(anyhow!("Expected '{}' to have at least {} item(s)", json_to_string(actual_value), min))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected {} ({}) to be the same type as {} ({})",
            value_of(actual_value), type_of(actual_value), value_of(expected_value), type_of(expected_value))),
        }
      },
      MatchingRule::MaxType(max) => {
        match (expected_value, actual_value) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() > *max {
            Err(anyhow!("Expected '{}' to have at most {} item(s)", json_to_string(actual_value), max))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected {} ({}) to be the same type as {} ({})",
            value_of(actual_value), type_of(actual_value), value_of(expected_value), type_of(expected_value))),
        }
      },
      MatchingRule::MinMaxType(min, max) => {
        match (expected_value, actual_value) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() < *min {
            Err(anyhow!("Expected '{}' to have at least {} item(s)", json_to_string(actual_value), min))
          } else if !cascaded && actual_array.len() > *max {
            Err(anyhow!("Expected '{}' to have at most {} item(s)", json_to_string(actual_value), max))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected {} ({}) to be the same type as {} ({})",
            value_of(actual_value), type_of(actual_value), value_of(expected_value), type_of(expected_value))),
        }
      },
      MatchingRule::Equality | MatchingRule::Values => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}",
            value_for_mismatch(value_of(actual_value), type_of(actual_value), show_types),
            value_for_mismatch(value_of(expected_value), type_of(expected_value), show_types)
          ))
        }
      },
      MatchingRule::Null => match actual_value {
        Value::Null => Ok(()),
        _ => Err(anyhow!("Expected {} ({}) to be a null value", value_of(actual_value), type_of(actual_value)))
      },
      MatchingRule::Integer => if actual_value.is_i64() || actual_value.is_u64() {
        Ok(())
      } else {
        Err(anyhow!("Expected {} ({}) to be an integer", value_of(actual_value), type_of(actual_value)))
      },
      MatchingRule::Decimal => if actual_value.is_f64() {
        Ok(())
      } else {
        Err(anyhow!("Expected {} ({}) to be a decimal number", value_of(actual_value), type_of(actual_value)))
      },
      MatchingRule::Number => if actual_value.is_number() {
        Ok(())
      } else {
        Err(anyhow!("Expected {} ({}) to be a number", value_of(actual_value), type_of(actual_value)))
      },
      #[allow(unused_variables)]
      MatchingRule::Date(s) => {
        #[cfg(feature = "datetime")]
        {
          let string = json_to_string(actual_value);
          let format = if s.is_empty() {
            "yyyy-MM-dd"
          } else {
            s.as_str()
          };
          validate_datetime(&string, format)
            .map_err(|err| anyhow!("Expected '{}' to match a date pattern of '{}': {}", string, format, err))
        }
        #[cfg(not(feature = "datetime"))]
        {
          Err(anyhow!("Date matchers require the datetime feature to be enabled"))
        }
      },
      #[allow(unused_variables)]
      MatchingRule::Time(s) => {
        #[cfg(feature = "datetime")]
        {
          let string = json_to_string(actual_value);
          let format = if s.is_empty() {
            "HH:mm:ss"
          } else {
            s.as_str()
          };
          validate_datetime(&string, format)
            .map_err(|err| anyhow!("Expected '{}' to match a time pattern of '{}': {}", string, format, err))
        }
        #[cfg(not(feature = "datetime"))]
        {
          Err(anyhow!("Time matchers require the datetime feature to be enabled"))
        }
      },
      #[allow(unused_variables)]
      MatchingRule::Timestamp(s) => {
        #[cfg(feature = "datetime")]
        {
          let string = json_to_string(actual_value);
          let format = if s.is_empty() {
            "yyyy-MM-dd'T'HH:mm:ssXXX"
          } else {
            s.as_str()
          };
          validate_datetime(&string, format)
            .map_err(|err| anyhow!("Expected '{}' to match a timestamp pattern of '{}': {}", string, format, err))
        }
        #[cfg(not(feature = "datetime"))]
        {
          Err(anyhow!("DateTime matchers require the datetime feature to be enabled"))
        }
      },
      MatchingRule::ContentType(expected_content_type) => {
        match_content_type(&convert_data(actual_value), expected_content_type)
          .map_err(|err| anyhow!("Failed to match data to have a content type of '{}': {}", expected_content_type, err))
      }
      MatchingRule::Boolean => match actual_value {
        Value::Bool(_) => Ok(()),
        Value::String(val) => if val == "true" || val == "false" {
          Ok(())
        } else {
          Err(anyhow!("Expected {} ({}) to match a boolean", value_of(actual_value), type_of(actual_value)))
        }
        _ => Err(anyhow!("Expected {} ({}) to match a boolean", value_of(actual_value), type_of(actual_value)))
      }
      MatchingRule::NotEmpty => match actual_value {
        Value::Null => Err(anyhow!("Expected non-empty but got a NULL")),
        Value::String(s) => if s.is_empty() {
          Err(anyhow!("Expected '' (String) to not be empty"))
        } else {
          Ok(())
        }
        Value::Array(a) => if a.is_empty() {
          Err(anyhow!("Expected [] (Array) to not be empty"))
        } else {
          Ok(())
        }
        Value::Object(o) => if o.is_empty() {
          Err(anyhow!("Expected {{}} (Object) to not be empty"))
        } else {
          Ok(())
        }
        _ => Ok(())
      }
      MatchingRule::Semver => match actual_value {
        Value::String(s) => match Version::parse(s) {
          Ok(_) => Ok(()),
          Err(err) => Err(anyhow!("'{}' is not a valid semantic version - {}", s, err))
        }
        _ => Err(anyhow!("Expected something that matches a semantic version, but got '{}'", actual_value))
      }
      _ => Ok(())
    };
    debug!("JSON -> JSON: Comparing '{}' ({}) to '{}' ({}) using {:?} -> {:?}", expected_value,
      type_of(expected_value), actual_value, type_of(actual_value), self, result);
    result
  }
}

/// Matches the expected JSON to the actual, and populates the mismatches vector with any differences
pub fn match_json(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<super::Mismatch>> {
  let expected_json = serde_json::from_slice(&*expected.body().value().unwrap_or_default());
  let actual_json = serde_json::from_slice(&*actual.body().value().unwrap_or_default());

  if expected_json.is_err() || actual_json.is_err() {
    let mut mismatches = vec![];
    if let Err(e) = expected_json {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".to_string(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the expected body: '{}'", e),
      });
    }
    if let Err(e) = actual_json {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".to_string(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the actual body: '{}'", e),
      });
    }
    Err(mismatches.clone())
  } else {
    compare_json(&DocPath::root(), &expected_json.unwrap(), &actual_json.unwrap(), context)
      .map_err(|mismatches| mismatches.iter().map(|mismatch| mismatch.to_body_mismatch()).collect())
  }
}

fn walk_json(json: &Value, path: &mut dyn Iterator<Item=&str>) -> Option<Value> {
  match path.next() {
    Some(p) => match json {
      Value::Object(_) => json.get(p).cloned(),
      Value::Array(array) => match usize::from_str(p) {
        Ok(index) => array.get(index).cloned(),
        Err(_) => None
      },
      _ => None
    },
    None => None
  }
}

/// Returns a diff of the expected versus the actual JSON bodies, focusing on a particular path
pub fn display_diff(expected: &str, actual: &str, path: &str, indent: &str) -> String {
  let expected_body = if expected.is_empty() {
    Value::String("".into())
  } else {
    Value::from_str(expected).unwrap_or_default()
  };
  let actual_body = if actual.is_empty() {
    Value::String("".into())
  } else {
    Value::from_str(actual).unwrap_or_default()
  };
  let mut path = path.split('.').skip(1);
  let next = path.next();
  let expected_fragment = if next.is_none() {
    serde_json::to_string_pretty(&expected_body).unwrap_or_default()
  } else {
    match walk_json(&expected_body, &mut path.clone()) {
      Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
      None => s!("")
    }
  };
  let actual_fragment = if next.is_none() {
    serde_json::to_string_pretty(&actual_body).unwrap_or_default()
  } else {
    match walk_json(&actual_body, &mut path.clone()) {
      Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
      None => s!("")
    }
  };
  let changeset = Changeset::new(&expected_fragment, &actual_fragment, "\n");
  let mut output = String::new();
  for change in changeset.diffs {
      match change {
          Difference::Same(ref x) => output.push_str(&format!("{}{}\n", indent, x)),
          Difference::Add(ref x) => output.push_str(&Green.paint(format!("{}+{}\n", indent, x)).to_string()),
          Difference::Rem(ref x) => output.push_str(&Red.paint(format!("{}-{}\n", indent, x)).to_string())
      }
  }
  output
}

/// Compares the actual JSON to the expected one
pub fn compare_json(
  path: &DocPath,
  expected: &Value,
  actual: &Value,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<CommonMismatch>> {
  debug!("compare: Comparing path {}", path);
  match (expected, actual) {
    (&Value::Object(ref emap), &Value::Object(ref amap)) => compare_maps(path, emap, amap, context),
    (&Value::Object(_), _) => {
      Err(vec![ CommonMismatch {
        path: path.to_string(),
        expected: json_to_string(expected),
        actual: json_to_string(actual),
        description: format!("Type mismatch: Expected {} ({}) to be the same type as {} ({})",
          value_of(actual), type_of(actual), value_of(expected), type_of(expected))
      } ])
    }
    (&Value::Array(ref elist), &Value::Array(ref alist)) => compare_lists(path, elist, alist, context),
    (&Value::Array(_), _) => {
      Err(vec![ CommonMismatch {
        path: path.to_string(),
        expected: json_to_string(expected),
        actual: json_to_string(actual),
        description: format!("Type mismatch: Expected {} ({}) to be the same type as {} ({})",
          value_of(actual), type_of(actual), value_of(expected), type_of(expected)),
      } ])
    }
    (_, _) => compare_values(path, expected, actual, context)
  }
}

fn compare_maps(
  path: &DocPath,
  expected: &serde_json::Map<String, Value>,
  actual: &serde_json::Map<String, Value>,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<CommonMismatch>> {
  let spath = path.to_string();
  debug!("compare_maps: Comparing maps at {}: {:?} -> {:?}", spath, expected, actual);
  if expected.is_empty() && context.config() == DiffConfig::NoUnexpectedKeys && !actual.is_empty() {
    debug!("compare_maps: Expected map is empty, but actual is not");
    Err(vec![ CommonMismatch {
      path: spath,
      expected: json_to_string(&json!(expected)),
      actual: json_to_string(&json!(actual)),
      description: format!("Expected an empty Map but received {}", json_to_string(&json!(actual))),
    } ])
  } else {
    let mut result = Ok(());
    let expected = expected.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let actual = actual.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    if context.matcher_is_defined(path) {
      debug!("compare_maps: Matcher is defined for path {}", path);
      let rule_list = context.select_best_matcher(path);
      for matcher in rule_list.rules {
        let result1 = compare_maps_with_matchingrule(&matcher, rule_list.cascaded, path, &expected, &actual, context, &mut |p, expected, actual, context| {
          compare_json(p, expected, actual, context)
        });
        result = merge_result(result, result1);
      }
    } else {
      let expected_keys = expected.keys().cloned().collect();
      let actual_keys = actual.keys().cloned().collect();
      result = merge_result(result, context.match_keys(path, &expected_keys, &actual_keys));
      for (key, value) in expected.iter() {
        let p = path.join(key);
        if actual.contains_key(key) {
          result = merge_result(result, compare_json(&p, value, &actual[key], context));
        }
      }
    };
    result
  }
}

fn compare_lists(
  path: &DocPath,
  expected: &[Value],
  actual: &[Value],
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<CommonMismatch>> {
  let spath = path.to_string();
  if context.matcher_is_defined(path) {
    debug!("compare_lists: matcher defined for path '{}'", path);
    compare_lists_with_matchingrules(path, &context.select_best_matcher(path), expected, actual, context, &mut |p, expected, actual, context| {
        compare_json(p, expected, actual, context)
    })
  } else if expected.is_empty() && !actual.is_empty() {
    Err(vec![ CommonMismatch {
      path: spath,
      expected: json_to_string(&json!(expected)),
      actual: json_to_string(&json!(actual)),
      description: format!("Expected an empty List but received {}", json_to_string(&json!(actual))),
    } ])
  } else {
    let result = compare_list_content(path, expected, actual, context);
    if expected.len() != actual.len() {
      merge_result(result, Err(vec![ CommonMismatch {
        path: spath,
        expected: json_to_string(&json!(expected)),
        actual: json_to_string(&json!(actual)),
        description: format!("Expected a List with {} elements but received {} elements",
                          expected.len(), actual.len()),
      } ]))
    } else {
      result
    }
  }
}

fn compare_list_content(
  path: &DocPath,
  expected: &[Value],
  actual: &[Value],
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<CommonMismatch>> {
  let mut result = Ok(());
  for (index, value) in expected.iter().enumerate() {
    let ps = index.to_string();
    debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
    let p = path.join(ps);
    if index < actual.len() {
      result = merge_result(result, compare_json(&p, value, &actual[index], context));
    } else if !context.matcher_is_defined(&p) {
      result = merge_result(result,Err(vec![ CommonMismatch {
        path: path.to_string(),
        expected: json_to_string(&json!(expected)),
        actual: json_to_string(&json!(actual)),
        description: format!("Expected {} but was missing", json_to_string(value)) } ]))
    }
  }
  result
}

fn compare_values(
  path: &DocPath,
  expected: &Value,
  actual: &Value,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<CommonMismatch>> {
  let matcher_result = if context.matcher_is_defined(path) {
    debug!("compare_values: Calling match_values for path {}", path);
    match_values(path, &context.select_best_matcher(&path), expected, actual)
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false)
      .map_err(|err| vec![err.to_string()])
  };
  debug!("compare_values: Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.to_string(), matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      CommonMismatch {
        path: path.to_string(),
        expected: format!("{}", expected),
        actual: format!("{}", actual),
        description: message.clone()
      }
    }).collect()
  })
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use expectest::expect;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::{matchingrules, matchingrules_list};
  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
  use pact_models::request::Request;

  use crate::{CoreMatchingContext, DiffConfig};
  use crate::Mismatch;
  use crate::Mismatch::BodyMismatch;

  use super::*;

  macro_rules! request {
    ($e:expr) => (Request { body: OptionalBody::Present($e.into(), None, None), .. Request::default() })
  }

  #[test]
  fn match_json_handles_invalid_expected_json() {
    let expected = request!(r#"{"json": "is bad"#);
    let actual = request!("{}");
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(vec![Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("") }]));
  }

  #[test]
  fn match_json_handles_invalid_actual_json() {
    let expected = request!("{}");
    let actual = request!(r#"{json: "is bad"}"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(
      vec![
        Mismatch::BodyMismatch {
          path: s!("$"),
          expected: expected.body.value(),
          actual: actual.body.value(),
          mismatch: s!("Type mismatch: Expected List [{}] but received Map {}")
        }
      ]
    ));
  }

  fn mismatch_message(mismatch: &Result<(), Vec<Mismatch>>) -> String {
    match mismatch {
      Err(mismatches) => match &mismatches.first() {
        Some(Mismatch::BodyMismatch { mismatch, .. }) => mismatch.clone(),
        _ => "".into()
      },
      _ => "".into()
    }
  }

  #[test]
  fn match_json_handles_expecting_a_map_but_getting_a_list() {
    let expected = request!(r#"{}"#);
    let actual = request!(r#"[]"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Type mismatch: Expected [] (Array) to be the same type as {} (Object)"));
    expect!(result).to(be_err().value(vec![Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: "".to_string()
    }]));
  }

  #[test]
  fn match_json_handles_expecting_a_list_but_getting_a_map() {
    let expected = request!(r#"[{}]"#);
    let actual = request!(r#"{}"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Type mismatch: Expected {} (Object) to be the same type as [{}] (Array)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: "".to_string()
    }]));
  }

  #[test]
  fn match_json_handles_comparing_strings() {
    let val1 = request!(r#""string value""#);
    let val2 = request!(r#""other value""#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 'other value' (String) to be equal to 'string value' (String)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: "".to_string()
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_integers() {
    let val1 = request!(r#"100"#);
    let val2 = request!(r#"200"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 200 (Integer) to be equal to 100 (Integer)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: "".to_string()
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_floats() {
    let val1 = request!(r#"100.01"#);
    let val2 = request!(r#"100.02"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 100.02 (Decimal) to be equal to 100.01 (Decimal)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: "".to_string()
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_booleans() {
    let val1 = request!(r#"true"#);
    let val2 = request!(r#"false"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected false (Boolean) to be equal to true (Boolean)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: "".to_string()
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_nulls() {
    let val1 = request!(r#"null"#);
    let val2 = request!(r#"33"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 33 (Integer) to be equal to null (Null)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$".to_string(),
      expected: val1.clone().body.value(),
      actual: val2.clone().body.value(),
      mismatch: "".to_string()
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_lists() {
    let val1 = request!(r#"[]"#);
    let val2 = request!(r#"[11,22,33]"#);
    let val3 = request!(r#"[11,44,33]"#);
    let val4 = request!(r#"[11,44,33, 66]"#);

    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val3.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected an empty List but received [11,22,33]".to_string()));
    expect!(result).to(be_err());

    let result = match_json(&val2.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected 44 (Integer) to be equal to 22 (Integer)".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$[1]".to_string(),
        expected: Some("22".into()), actual: Some("44".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected a List with 3 elements but received 4 elements".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[11,44,33]".into()),
        actual: Some("[11,44,33,66]".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$[1]".to_string(),
        expected: Some("22".into()),
        actual: Some("44".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$[1] -> Expected 44 (Integer) to be equal to 22 (Integer)".to_string()));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[11,22,33]".into()),
        actual: Some("[11,44,33,66]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Expected a List with 3 elements but received 4 elements".to_string()));

    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules! {
        "body" => {
            "$" => [ MatchingRule::Type ]
        }
    }.rules_for_category("body").unwrap(), &hashmap!{});
    let result = match_json(&val2.clone(), &val4.clone(), &context);
    expect!(result).to(be_ok());
    let result = match_json(&val4, &val2, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_json_handles_comparing_maps() {
    let val1 = request!(r#"{}"#);
    let val2 = request!(r#"{"a": 1, "b": 2}"#);
    let val3 = request!(r#"{"a": 1, "b": 3}"#);
    let val4 = request!(r#"{"a": 1, "b": 2, "c": 3}"#);

    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val4.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected an empty Map but received {\"a\":1,\"b\":2}"));

    let result = match_json(&val2.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 3 (Integer) to be equal to 2 (Integer)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$.b".to_string(),
        expected: Some("2".into()), actual: Some("3".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected a Map with keys [a, b] but received one with keys [a, b, c]".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\"]".into()),
        actual: Some("[\"a\",\"b\",\"c\"]".into()), mismatch: "Expected a Map with keys [a, b] but received one with keys [a, b, c]".to_string()
    } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result).as_str()).to(be_equal_to("Expected 2 (Integer) to be equal to 3 (Integer)"));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$.b".to_string(),
        expected: Some("3".into()),
        actual: Some("2".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\"]".into()),
        actual: Some("[\"a\",\"b\",\"c\"]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Expected a Map with keys [a, b] but received one with keys [a, b, c]".to_string()));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$.b".to_string(),
        expected: Some("3".into()),
        actual: Some("2".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$.b -> Expected 2 (Integer) to be equal to 3 (Integer)".to_string()));

    let result = match_json(&val4.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(1));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\",\"c\"]".into()),
        actual: Some("[\"a\",\"b\"]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Actual map is missing the following keys: c".to_string()));

    let result = match_json(&val3, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$.*" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_ok());
  }

    #[test]
    fn equality_matcher_test() {
        let matcher = MatchingRule::Equality;
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_err());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_err());
    }

    #[test]
    fn regex_matcher_test() {
        let matcher = MatchingRule::Regex("^\\d+$".into());
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("10a".into()), &matcher, false)).to(be_err());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_ok());
    }

  #[test]
  fn includes_matcher_test() {
    let matcher = MatchingRule::Include("10".into());
    expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(Value::String("1a0".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_ok());
  }

    #[test]
    fn type_matcher_test() {
        let matcher = MatchingRule::Type;
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("10a".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_err());
    }

    #[test]
    fn min_type_matcher_test() {
        let matcher = MatchingRule::MinType(2);
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, false)).to(be_ok());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_err());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, true)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

    #[test]
    fn max_type_matcher_test() {
        let matcher = MatchingRule::MaxType(1);
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, false)).to(be_err());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, true)).to(be_ok());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

    #[test]
    fn min_max_type_matcher_test() {
      let matcher = MatchingRule::MinMaxType(2, 3);
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]),
        &matcher, false)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100)]), &matcher, false)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100), json!(100)]), &matcher, false)).to(be_err());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100), json!(100)]), &matcher, true)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_err());
      expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

  #[test]
  fn integer_matcher_test() {
    let matcher = MatchingRule::Integer;
    expect!(Value::String("100".into()).matches_with(&Value::String("100.0".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(&json!("100"), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.02), &matcher, false)).to(be_err());
  }

  #[test]
  fn decimal_matcher_test() {
    let matcher = MatchingRule::Decimal;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.01), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(&json!("100.01"), &matcher, false)).to(be_err());
  }

  #[test]
  fn number_matcher_test() {
    let matcher = MatchingRule::Number;
    expect!(Value::String("100".into()).matches_with(&Value::String("100x".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(&json!("100"), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.01), &matcher, false)).to(be_ok());
  }

  #[test]
  fn boolean_matcher_test() {
    let matcher = MatchingRule::Boolean;
    expect!(Value::Bool(true).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::Bool(true).matches_with(&Value::Bool(false), &matcher, false)).to(be_ok());
    expect!(Value::Bool(true).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::Bool(true).matches_with(&Value::String("true".into()), &matcher, false)).to(be_ok());
    expect!(Value::Bool(true).matches_with(&Value::String("false".into()), &matcher, false)).to(be_ok());
  }

  #[test]
  fn null_matcher_test() {
    let matcher = MatchingRule::Null;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::String("10a".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.2), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!("null"), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::Null, &matcher, false)).to(be_ok());
  }

  #[test]
  fn content_type_matcher_test() {
    let matcher = MatchingRule::ContentType("text/plain".to_string());
    expect!(Value::String("plain text".into()).matches_with(&Value::String("plain text".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("plain text".into()).matches_with(&Value::String("different text".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("plain text".into()).matches_with(&json!(100), &matcher, false)).to(be_ok());
    expect!(Value::String("plain text".into()).matches_with(&json!(100.01), &matcher, false)).to(be_ok());
    {
      let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <note>
        <to>Tove</to>
        <from>Jani</from>
        <heading>Reminder</heading>
        <body>Don't forget me this weekend!</body>
      </note>"#;
      expect!(Value::String("plain text".into()).matches_with(Value::String(xml.into()), &matcher, false)).to(be_err());
    }
  }

  #[test_log::test]
  fn compare_maps_handles_wildcard_matchers() {
    let val1 = request!(r#"
    {
      "articles": [
        {
          "variants": {
            "001": {
              "bundles": {
                "001-A": {
                  "description": "someDescription",
                  "referencedArticles": [
                    {
                        "bundleId": "someId"
                    }
                  ]
                }
              }
            }
          }
        }
      ]
    }"#);
    let val2 = request!(r#"{
      "articles": [
        {
          "variants": {
            "002": {
              "bundles": {
                "002-A": {
                  "description": "someDescription",
                  "referencedArticles": [
                    {
                        "bundleId": "someId"
                    }
                  ]
                }
              }
            }
          }
        }
      ]
    }"#);

    let matching_rules = matchingrules! {
      "body" => {
        "$.articles[*].variants" => [ MatchingRule::Values ],
        "$.articles[*].variants.*.bundles" => [ MatchingRule::Values ],
        "$.articles[*].variants.*.bundles.*.referencedArticles[*]" => [ MatchingRule::Type ]
      }
    };
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                       &matching_rules.rules_for_category("body").unwrap(),
                                       &hashmap!{});
    let result = match_json(&val1, &val2, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_lists_with_array_contains_matcher() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 3, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$" => [ MatchingRule::ArrayContains(vec![]) ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_lists_without_array_contains_matcher_fails() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 3, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err());
  }

  #[test]
  fn compare_lists_with_array_contains_matcher_fails() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$" => [ MatchingRule::ArrayContains(vec![]) ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_err().value(vec![
      BodyMismatch {
        path: "$".to_string(),
        expected: Some("3".into()),
        actual: Some("[\"10\",\"22\",\"6\",\"1\",\"5\",\"2\"]".into()),
        mismatch: "Variant at index 2 (3) was not found in the actual list".to_string()
      }
    ]));
  }

  #[test]
  fn compare_lists_with_each_value_matcher() {
    let expected = request!(r#"
    [1, 2]
    "#);
    let actual = request!(r#"
    [3, 4, 567]
    "#);

    let rules = matchingrules! {
      "body" => {
        "$" => [
          MatchingRule::EachValue(
            MatchingRuleDefinition::new("100".to_string(), ValueType::String, MatchingRule::Integer, None, "".to_string())
          )
        ]
      }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(),
      &hashmap!{}
    );

    let result = match_json(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_lists_with_each_value_matcher_fails() {
    let expected = request!(r#"
    [1, 2]
    "#);
    let actual = request!(r#"
    [3, "abc123", "test"]
    "#);

    let rules = matchingrules! {
      "body" => {
        "$" => [
          MatchingRule::EachValue(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
            MatchingRule::Integer, None, "".to_string()))
        ]
      }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("body").unwrap_or_default(),
      &hashmap!{}
    );

    let result = match_json(&expected, &actual, &context);
    expect!(result).to(be_err().value(vec![
      BodyMismatch {
        path: "$[1]".to_string(),
        expected: Some("2".into()),
        actual: Some("\"abc123\"".into()),
        mismatch: "Expected 'abc123' (String) to be an integer number".to_string(),
      },
      BodyMismatch {
        path: "$[2]".to_string(),
        expected: Some("1".into()),
        actual: Some("\"test\"".into()),
        mismatch: "Expected 'test' (String) to be an integer number".to_string(),
      }
    ]));
  }

  #[test]
  fn compare_lists_with_array_contains_matcher_with_more_complex_object() {
    let expected = request!(r#"
    {
      "class": [ "order" ],
      "properties": {
          "orderNumber": 42,
          "itemCount": 3,
          "status": "pending"
      },
      "entities": [
        {
          "class": [ "info", "customer" ],
          "properties": {
            "customerId": "pj123",
            "name": "Peter Joseph"
          }
        }
      ],
      "actions": [
        {
          "name": "add-item",
          "title": "Add Item",
          "method": "POST",
          "href": "http://api.x.io/orders/42/items"
        }
      ],
      "links": [
        { "rel": [ "next" ], "href": "http://api.x.io/orders/43" }
      ]
    }
    "#);
    let actual = request!(r#"
    {
      "class": [ "order" ],
      "properties": {
          "orderNumber": 12,
          "itemCount": 6,
          "status": "pending"
      },
      "entities": [
        {
          "class": [ "items", "collection" ],
          "rel": [ "http://x.io/rels/order-items" ],
          "href": "http://api.x.io/orders/12/items"
        },
        {
          "class": [ "info", "customer" ],
          "rel": [ "http://x.io/rels/customer" ],
          "properties": {
            "customerId": "rh565421",
            "name": "Ron Haich"
          },
          "links": [
            { "rel": [ "self" ], "href": "http://api.x.io/customers/rh565421" }
          ]
        }
      ],
      "actions": [
        {
          "name": "add-item",
          "title": "Add Item",
          "method": "POST",
          "href": "http://api.x.io/orders/12/items",
          "type": "application/x-www-form-urlencoded",
          "fields": [
            { "name": "orderNumber", "type": "hidden", "value": "12" },
            { "name": "productCode", "type": "text" },
            { "name": "quantity", "type": "number" }
          ]
        },
        {
          "name": "delete-order",
          "title": "Delete Order",
          "method": "DELETE",
          "href": "http://api.x.io/orders/12"
        },
        {
          "name": "update-order",
          "title": "Update Order",
          "method": "POST",
          "href": "http://api.x.io/orders/12"
        }
      ],
      "links": [
        { "rel": [ "self" ], "href": "http://api.x.io/orders/12" },
        { "rel": [ "previous" ], "href": "http://api.x.io/orders/11" },
        { "rel": [ "next" ], "href": "http://api.x.io/orders/13" }
      ]
    }
    "#);

    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules! {
      "body" => {
        "$.entities" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.properties.customerId" => [ MatchingRule::Type ], "$.properties.name" => [ MatchingRule::Type ]
          }, HashMap::default())])
        ],
        "$.properties.orderNumber" => [ MatchingRule::Integer ],
        "$.properties.itemCount" => [ MatchingRule::Integer ],
        "$.actions" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.href" => [ MatchingRule::Regex(".*/orders/\\d+/items".to_string()) ]
          }, HashMap::default())])
        ],
        "$.links" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.href" => [ MatchingRule::Regex(".*/orders/\\d+".to_string()) ]
          }, HashMap::default())])
        ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{});
    let result = match_json(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_maps_handles_empty_expected_maps() {
    let expected_json = json!({});
    let expected = expected_json.as_object().unwrap();
    let actual_json = json!({"foo": "bar"});
    let actual = actual_json.as_object().unwrap();
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                       &MatchingRuleCategory::empty("body"), &hashmap!{});
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    expect!(result).to(be_ok());

    let context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                       &MatchingRuleCategory::empty("body"), &hashmap!{});
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    expect!(result).to(be_err());
  }

  #[test_log::test]
  fn compare_maps_with_each_value_matcher() {
    let expected_json = json!({
      "id1": "book1"
    });
    let expected = expected_json.as_object().unwrap();
    let actual_json = json!({
      "id1001": "book1100",
      "id2": "book2"
    });
    let actual = actual_json.as_object().unwrap();

    let matchingrules = matchingrules_list! {
       "body"; "$" => [
        MatchingRule::EachValue(MatchingRuleDefinition::new("{\"id1\":\"book1\"}".to_string(),
          ValueType::Unknown, MatchingRule::Regex("\\w+\\d+".to_string()), None, "".to_string()))
      ]
    };

    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &matchingrules, &hashmap!{});
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    expect!(result).to(be_ok());

    let invalid_json = json!({
      "id1001": "book1100",
      "id2": 1
    });
    let invalid = invalid_json.as_object().unwrap();
    let result = compare_maps(&DocPath::root(), expected, invalid, &context);
    expect!(result).to(be_err());
  }
}

#[cfg(test)]
mod tests2 {
  use expectest::prelude::*;
  use maplit::hashmap;
  use rstest::rstest;
  use serde_json::{json, Value};

  use pact_models::matchingrules_list;
  use pact_models::matchingrules::MatchingRule;
  use crate::{CoreMatchingContext, DiffConfig};

  use super::*;

  #[rstest]
  //                                                    config,                          actual_json,                                           is_ok
  #[case::no_unexpected_keys_same_values(               DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "bb", "c": "ccc"}),              true)]
  #[case::no_unexpected_keys_missing_first_value(       DiffConfig::NoUnexpectedKeys,    json!({"b": "bb", "c": "ccc"}),                        true)] // should be err
  #[case::no_unexpected_keys_missing_last_value(        DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "bb"}),                          true)] // should be err
  #[case::no_unexpected_keys_duplicated_first_value(    DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "bb", "c": "ccc", "d": "a"}),    true)]
  #[case::no_unexpected_keys_duplicated_second_value(   DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "bb", "c": "ccc", "d": "bb"}),   false)] // should be ok, for consistency
  #[case::no_unexpected_keys_additional_value_begin(    DiffConfig::NoUnexpectedKeys,    json!({"d": "dddd", "a": "a", "b": "bb", "c": "ccc"}), false)] // the mismatch doesn't make sense
  #[case::no_unexpected_keys_additional_value_end(      DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "bb", "c": "ccc", "d": "dddd"}), false)] // the mismatch doesn't make sense
  #[case::no_unexpected_keys_updated_value(             DiffConfig::NoUnexpectedKeys,    json!({"a": "a", "b": "b", "c": "cc"}),                false)]
  #[case::no_unexpected_keys_swap_value(                DiffConfig::NoUnexpectedKeys,    json!({"a": "bb", "b": "ccc", "c": "a"}),              false)] // should be ok
  #[case::allow_unexpected_keys_same_values_change_keys(DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "bb": "bb", "ccc": "ccc"}),           false)] // should be ok
  #[case::allow_unexpected_keys_missing_first_value(    DiffConfig::AllowUnexpectedKeys, json!({"b": "bb", "c": "ccc"}),                        true)] // should be err
  #[case::allow_unexpected_keys_missing_last_value(     DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "b": "bb"}),                          true)] // should be err
  #[case::allow_unexpected_keys_duplicated_first_value( DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "b": "bb", "c": "ccc", "d": "a"}),    true)]
  #[case::allow_unexpected_keys_duplicated_second_value(DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "b": "bb", "c": "ccc", "d": "bb"}),   false)] // should be ok, for consistency
  #[case::allow_unexpected_keys_additional_value_begin( DiffConfig::AllowUnexpectedKeys, json!({"d": "dddd", "a": "a", "b": "bb", "c": "ccc"}), false)] // the mismatch doesn't make sense
  #[case::allow_unexpected_keys_additional_value_end(   DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "b": "bb", "c": "ccc", "d": "dddd"}), false)] // the mismatch doesn't make sense
  #[case::allow_unexpected_keys_updated_value(          DiffConfig::AllowUnexpectedKeys, json!({"a": "a", "b": "b", "c":  "cc"}),               false)]
  #[case::allow_unexpected_keys_swap_value(             DiffConfig::AllowUnexpectedKeys, json!({"a": "bb", "b": "ccc", "c":  "a"}),             false)] // should be ok
  fn compare_maps_with_values_matcher(#[case] config: DiffConfig, #[case] actual_json: Value, #[case] is_ok: bool) {
    let expected_json = json!({"a": "a", "b": "bb", "c": "ccc"});
    let matchingrules = matchingrules_list! {
       "body"; "$" => [
        MatchingRule::Values
      ]
    };
    let context = CoreMatchingContext::new(config, &matchingrules, &hashmap!{});
    let expected = expected_json.as_object().unwrap();
    let actual = actual_json.as_object().unwrap();
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    if is_ok {
      expect!(result).to(be_ok());
    } else {
      expect!(result).to(be_err());
    }
  }

  #[rstest]
  //                                                     config,                          actual_json,                              is_ok
  #[case::no_unexpected_keys_same_values(                DiffConfig::NoUnexpectedKeys,    json!(["a", "bb", "ccc"]),                true)]
  #[case::no_unexpected_keys_missing_first_value(        DiffConfig::NoUnexpectedKeys,    json!(["bb", "ccc"]),                     false)]
  #[case::no_unexpected_keys_missing_last_value(         DiffConfig::NoUnexpectedKeys,    json!(["a", "bb"]),                       true)] // should be err
  #[case::no_unexpected_keys_duplicated_first_value(     DiffConfig::NoUnexpectedKeys,    json!(["a", "bb", "ccc", "a", "a", "a"]), true)]
  #[case::no_unexpected_keys_duplicated_other_values(    DiffConfig::NoUnexpectedKeys,    json!(["a", "bb", "ccc", "bb", "ccc"]),   false)] // should be ok, for consistency
  #[case::no_unexpected_keys_additional_value_begin(     DiffConfig::NoUnexpectedKeys,    json!(["dddd", "a", "bb", "ccc"]),        false)] // the mismatch doesn't make sense
  #[case::no_unexpected_keys_additional_value_end(       DiffConfig::NoUnexpectedKeys,    json!(["a", "bb", "ccc", "dddd"]),        false)] // the mismatch doesn't make sense
  #[case::no_unexpected_keys_swap_value(                 DiffConfig::NoUnexpectedKeys,    json!(["ccc", "bb", "a"]),                false)] // should be ok
  #[case::allow_unexpected_keys_same_values(             DiffConfig::AllowUnexpectedKeys, json!(["a", "bb", "ccc"]),                true)]
  #[case::allow_unexpected_keys_missing_first_value(     DiffConfig::AllowUnexpectedKeys, json!(["bb", "ccc"]),                     false)]
  #[case::allow_unexpected_keys_missing_last_value(      DiffConfig::AllowUnexpectedKeys, json!(["a", "bb"]),                       true)] // should be err
  #[case::allow_unexpected_keys_duplicated_first_value(  DiffConfig::AllowUnexpectedKeys, json!(["a", "bb", "ccc", "a", "a", "a"]), true)]
  #[case::allow_unexpected_keys_duplicated_sother_values(DiffConfig::AllowUnexpectedKeys, json!(["a", "bb", "ccc", "bb", "ccc"]),   false)] // should be ok, for consistency
  #[case::allow_unexpected_keys_additional_value_begin(  DiffConfig::AllowUnexpectedKeys, json!(["dddd", "a", "bb", "ccc"]),        false)] // the mismatch doesn't make sense
  #[case::allow_unexpected_keys_additional_value_end(    DiffConfig::AllowUnexpectedKeys, json!(["a", "bb", "ccc", "dddd"]),        false)] // the mismatch doesn't make sense
  #[case::allow_unexpected_keys_swap_value(              DiffConfig::AllowUnexpectedKeys, json!(["ccc", "bb", "a"]),                false)] // should be ok
  fn compare_lists_with_values_matcher(#[case] config: DiffConfig, #[case] actual_json: Value, #[case] is_ok: bool) {
    let expected_json = json!(["a", "bb", "ccc"]);
    let matchingrules = matchingrules_list! {
       "body"; "$" => [
        MatchingRule::Values
      ]
    };
    let context = CoreMatchingContext::new(config, &matchingrules, &hashmap!{});
    let expected = expected_json.as_array().unwrap();
    let actual = actual_json.as_array().unwrap();
    let result = compare_lists(&DocPath::root(), expected, actual, &context);
    if is_ok {
      expect!(result).to(be_ok());
    } else {
      expect!(result).to(be_err());
    }
  }
}
