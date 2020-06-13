//! The `json` module provides functions to compare and display the differences between JSON bodies

use serde_json::{Value, json};
use super::Mismatch;
use super::DiffConfig;
use difference::*;
use ansi_term::Colour::*;
use std::str::FromStr;
use crate::models::matchingrules::*;
use crate::matchers::*;
use onig::Regex;
use log::*;
use crate::time_utils::validate_datetime;

fn type_of(json: &Value) -> String {
  match json {
    &Value::Object(_) => "Map",
    &Value::Array(_) => "List",
    &Value::Null => "Null",
    &Value::Bool(_) => "Boolean",
    &Value::Number(_) => "Number",
    &Value::String(_) => "String"
  }.to_string()
}

pub(crate) fn value_of(json: &Value) -> String {
    match json {
        &Value::String(ref s) => s.clone(),
        _ => format!("{}", json)
    }
}

impl Matches<Value> for Value {
    fn matches(&self, actual: &Value, matcher: &MatchingRule) -> Result<(), String> {
        let result = match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                let actual_str = match actual {
                  &Value::String(ref s) => s.clone(),
                  _ => actual.to_string()
                };
                if re.is_match(&actual_str) {
                  Ok(())
                } else {
                  Err(format!("Expected '{}' to match '{}'", value_of(actual), regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Include(ref substr) => {
            let actual_str = match actual {
              &Value::String(ref s) => s.clone(),
              _ => actual.to_string()
            };
            if actual_str.contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected '{}' to include '{}'", value_of(actual), substr))
            }
          },
          MatchingRule::Type => {
               match (self, actual) {
                   (&Value::Array(_), &Value::Array(_)) => Ok(()),
                   (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
                   (&Value::Number(_), &Value::Number(_)) => Ok(()),
                   (&Value::Null, &Value::Null) => Ok(()),
                   (&Value::Object(_), &Value::Object(_)) => Ok(()),
                   (&Value::String(_), &Value::String(_)) => Ok(()),
                   (_, _) => Err(format!("Expected '{}' to be the same type as '{}'", value_of(self), value_of(actual))),
               }
          },
          MatchingRule::MinType(min) => {
               match (self, actual) {
                   (&Value::Array(_), &Value::Array(ref actual_array)) => if actual_array.len() < min {
                       Err(format!("Expected '{}' to have at least {} item(s)", value_of(actual), min))
                   } else {
                       Ok(())
                   },
                   (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
                   (&Value::Number(_), &Value::Number(_)) => Ok(()),
                   (&Value::Null, &Value::Null) => Ok(()),
                   (&Value::Object(_), &Value::Object(_)) => Ok(()),
                   (&Value::String(_), &Value::String(_)) => Ok(()),
                   (_, _) => Err(format!("Expected '{}' to be the same type as '{}'", value_of(self), value_of(actual))),
               }
          },
          MatchingRule::MaxType(max) => {
               match (self, actual) {
                   (&Value::Array(_), &Value::Array(ref actual_array)) => if actual_array.len() > max {
                       Err(format!("Expected '{}' to have at most {} item(s)", value_of(actual), max))
                   } else {
                       Ok(())
                   },
                   (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
                   (&Value::Number(_), &Value::Number(_)) => Ok(()),
                   (&Value::Null, &Value::Null) => Ok(()),
                   (&Value::Object(_), &Value::Object(_)) => Ok(()),
                   (&Value::String(_), &Value::String(_)) => Ok(()),
                   (_, _) => Err(format!("Expected '{}' to be the same type as '{}'", value_of(self), value_of(actual))),
               }
          },
          MatchingRule::MinMaxType(min, max) => {
            match (self, actual) {
              (&Value::Array(_), &Value::Array(ref actual_array)) => if actual_array.len() < min {
                Err(format!("Expected '{}' to have at least {} item(s)", value_of(actual), min))
              } else if actual_array.len() > max {
                Err(format!("Expected '{}' to have at most {} item(s)", value_of(actual), max))
              } else {
                Ok(())
              },
              (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
              (&Value::Number(_), &Value::Number(_)) => Ok(()),
              (&Value::Null, &Value::Null) => Ok(()),
              (&Value::Object(_), &Value::Object(_)) => Ok(()),
              (&Value::String(_), &Value::String(_)) => Ok(()),
              (_, _) => Err(format!("Expected '{}' to be the same type as '{}'", value_of(self), value_of(actual))),
            }
          },
          MatchingRule::Equality => {
               if self == actual {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to be equal to '{}'", value_of(self), value_of(actual)))
               }
          },
          MatchingRule::Null => match actual {
            &Value::Null => Ok(()),
            _ => Err(format!("Expected '{}' to be a null value", value_of(actual)))
          },
          MatchingRule::Integer => if actual.is_i64() || actual.is_u64() {
            Ok(())
          } else {
            Err(format!("Expected '{}' to be an integer value", value_of(actual)))
          },
          MatchingRule::Decimal => if actual.is_f64() {
            Ok(())
          } else {
            Err(format!("Expected '{}' to be a decimal value", value_of(actual)))
          },
          MatchingRule::Number => if actual.is_number() {
            Ok(())
          } else {
            Err(format!("Expected '{}' to be a number", value_of(actual)))
          },
          MatchingRule::Date(ref s) => {
            validate_datetime(&value_of(actual), s)
              .map_err(|err| format!("Expected '{}' to match a date format of '{}': {}", actual, s, err))
          },
          MatchingRule::Time(ref s) => {
            validate_datetime(&value_of(actual), s)
              .map_err(|err| format!("Expected '{}' to match a time format of '{}': {}", actual, s, err))
          },
          MatchingRule::Timestamp(ref s) => {
            validate_datetime(&value_of(actual), s)
              .map_err(|err| format!("Expected '{}' to match a timestamp format of '{}': {}", actual, s, err))
          },
          MatchingRule::MustBeOneOf(ref items) => if items.contains(&(actual.as_u64().unwrap() as u16)) {
              Ok(())
          } else {
            println!("validating MustBeOneOf: {:?}, got: {:?}", items, actual);
            Err(format!("'{}' is not present in {:?}", actual, items))
          }
       };
       debug!("JSON -> JSON: Comparing '{}' to '{}' using {:?} -> {:?}", self, actual, matcher, result);
       result
    }
}

impl Matches<Vec<Value>> for Vec<Value> {
    fn matches(&self, actual: &Vec<Value>, matcher: &MatchingRule) -> Result<(), String> {
        let result = match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&Value::Array(actual.clone()).to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected '{:?}' to match '{}'", value_of(&Value::Array(actual.clone())), regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type => Ok(()),
          MatchingRule::MinType(min) => {
               if actual.len() < min {
                   Err(format!("Expected '{}' to have a minimum length of {}", value_of(&Value::Array(actual.clone())), min))
               } else {
                   Ok(())
               }
          },
          MatchingRule::MaxType(max) => {
               if actual.len() > max {
                   Err(format!("Expected '{}' to have a maximum length of {}", value_of(&Value::Array(actual.clone())), max))
               } else {
                   Ok(())
               }
          },
          MatchingRule::MinMaxType(min, max) => {
            if actual.len() < min {
              Err(format!("Expected '{}' to have a minimum length of {}", value_of(&Value::Array(actual.clone())), min))
            } else if actual.len() > max {
              Err(format!("Expected '{}' to have a maximum length of {}", value_of(&Value::Array(actual.clone())), max))
            } else {
              Ok(())
            }
          },
          MatchingRule::Equality => {
               if self == actual {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to be equal to '{}'", value_of(&Value::Array(self.clone())),
                    value_of(&&Value::Array(actual.clone()))))
               }
          },
          _ => Err(format!("Unable to match {:?} using {:?}", self, matcher))
       };
       log::debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
       result
    }
}

/// Matches the expected JSON to the actual, and populates the mismatches vector with any differences
pub fn match_json(expected: &Vec<u8>, actual: &Vec<u8>, config: DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let expected_json = serde_json::from_slice(expected);
    let actual_json = serde_json::from_slice(actual);

    if expected_json.is_err() || actual_json.is_err() {
        match expected_json {
            Err(e) => {
                mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone().into()),
                    actual: Some(actual.clone().into()),
                    mismatch: format!("Failed to parse the expected body: '{}'", e)});
            },
            _ => ()
        }
        match actual_json {
            Err(e) => {
                mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone().into()),
                    actual: Some(actual.clone().into()),
                    mismatch: format!("Failed to parse the actual body: '{}'", e)});
            },
            _ => ()
        }
    } else {
        compare(&vec![s!("$")], &expected_json.unwrap(), &actual_json.unwrap(), &config,
            mismatches, matchers);
    }
}

fn walk_json(json: &Value, path: &mut dyn Iterator<Item=&str>) -> Option<Value> {
    match path.next() {
        Some(p) => match json {
            &Value::Object(_) => json.get(p).map(|json| json.clone()),
            &Value::Array(ref array) => match usize::from_str(p) {
                Ok(index) => array.get(index).map(|json| json.clone()),
                Err(_) => None
            },
            _ => None
        },
        None => None
    }
}

/// Returns a diff of the expected versus the actual JSON bodies, focusing on a particular path
pub fn display_diff(expected: &String, actual: &String, path: &String) -> String {
    let expected_body = Value::from_str(expected).unwrap();
    let actual_body = Value::from_str(actual).unwrap();
    let path = path.split('.').skip(2);
    let expected_fragment = match walk_json(&expected_body, &mut path.clone()) {
        Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
        None => s!("")
    };
    let actual_fragment = match walk_json(&actual_body, &mut path.clone()) {
        Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
        None => s!("")
    };
    let changeset = Changeset::new(&expected_fragment, &actual_fragment, "\n");
    let mut output = String::new();
    for change in changeset.diffs {
        match change {
            Difference::Same(ref x) => output.push_str(&format!(" {}\n", x)),
            Difference::Add(ref x) => output.push_str(&Green.paint(format!("+{}\n", x)).to_string()),
            Difference::Rem(ref x) => output.push_str(&Red.paint(format!("-{}\n", x)).to_string())
        }
    }
    output
}

fn compare(path: &Vec<String>, expected: &Value, actual: &Value, config: &DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    log::debug!("Comparing path {}", path.join("."));
    match (expected, actual) {
        (&Value::Object(ref emap), &Value::Object(ref amap)) => compare_maps(path, emap, amap, config, mismatches, matchers),
        (&Value::Object(_), _) => {
            mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
                expected: Some(value_of(expected).into()),
                actual: Some(value_of(actual).into()),
                mismatch: format!("Type mismatch: Expected {} {} but received {} {}",
                    type_of(expected), expected, type_of(actual), actual)});
        },
        (&Value::Array(ref elist), &Value::Array(ref alist)) => compare_lists(path, elist, alist, config, mismatches, matchers),
        (&Value::Array(_), _) => {
            mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
                expected: Some(value_of(expected).into()),
                actual: Some(value_of(actual).into()),
                mismatch: format!("Type mismatch: Expected {} {} but received {} {}",
                    type_of(expected), value_of(expected), type_of(actual), value_of(actual))});
        },
        (_, _) => compare_values(path, expected, actual, mismatches, matchers)
    }
}

fn compare_maps(path: &Vec<String>, expected: &serde_json::Map<String, Value>, actual: &serde_json::Map<String, Value>,
    config: &DiffConfig, mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    if expected.is_empty() && !actual.is_empty() {
      mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
          expected: Some(value_of(&json!(expected)).into()),
          actual: Some(value_of(&json!(actual)).into()),
          mismatch: format!("Expected an empty Map but received {}", value_of(&json!(actual)))});
    } else {
        match config {
            &DiffConfig::AllowUnexpectedKeys if expected.len() > actual.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
                    expected: Some(value_of(&json!(expected)).into()),
                    actual: Some(value_of(&json!(&actual)).into()),
                    mismatch: format!("Expected a Map with at least {} elements but received {} elements",
                    expected.len(), actual.len())});
            },
            &DiffConfig::NoUnexpectedKeys if expected.len() != actual.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
                    expected: Some(value_of(&json!(expected)).into()),
                    actual: Some(value_of(&json!(&actual)).into()),
                    mismatch: format!("Expected a Map with {} elements but received {} elements",
                    expected.len(), actual.len())});
            },
            _ => ()
        }

        let mut p = path.to_vec();
        p.push(s!("any"));
        if matchers.wildcard_matcher_is_defined("body", &p) {
            for (key, value) in actual.iter() {
                let mut p = path.to_vec();
                p.push(key.clone());
                if expected.contains_key(key) {
                    compare(&p, &expected[key], value, config, mismatches, matchers);
                } else if !expected.is_empty() {
                    compare(&p, &expected.values().next().unwrap(), value, config, mismatches, matchers);
                }
            }
        } else {
            for (key, value) in expected.iter() {
                if actual.contains_key(key) {
                    let mut p = path.to_vec();
                    p.push(key.clone());
                    compare(&p, value, &actual[key], config, mismatches, matchers);
                } else {
                    mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
                        expected: Some(value_of(&json!(expected)).into()),
                        actual: Some(value_of(&json!(&actual)).into()),
                        mismatch: format!("Expected entry {}={} but was missing", key, value_of(value))});
                }
            }
        }
    }
}

fn compare_lists(path: &Vec<String>, expected: &Vec<Value>, actual: &Vec<Value>, config: &DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let spath = path.join(".");
    if matchers.matcher_is_defined("body", &path) {
        log::debug!("compare_lists: matcher defined for path '{}'", spath);
        let expected_json = Value::Array(expected.clone());
        let actual_json = Value::Array(actual.clone());
        match match_values("body", path, matchers.clone(), &expected_json, &actual_json) {
            Err(messages) => {
              for message in messages {
                mismatches.push(Mismatch::BodyMismatch {
                  path: path.join("."),
                  expected: Some(expected_json.to_string().into()),
                  actual: Some(actual_json.to_string().into()),
                  mismatch: message.clone()
                })
              }
            },
            Ok(_) => ()
        }
        let expected_example = expected.first().unwrap().clone();
        let mut expected_list = Vec::new();
        expected_list.resize(actual.len(), expected_example);
        compare_list_content(path, &expected_list, actual, config, mismatches, matchers);
    } else {
        if expected.is_empty() && !actual.is_empty() {
            mismatches.push(Mismatch::BodyMismatch { path: spath,
                expected: Some(value_of(&json!(expected)).into()),
                actual: Some(value_of(&json!(actual)).into()),
                mismatch: format!("Expected an empty List but received {}", value_of(&json!(actual)))});
        } else {
            compare_list_content(path, expected, actual, config, mismatches, matchers);
            if expected.len() != actual.len() {
                mismatches.push(Mismatch::BodyMismatch { path: spath,
                    expected: Some(value_of(&json!(expected)).into()),
                    actual: Some(value_of(&json!(actual)).into()),
                    mismatch: format!("Expected a List with {} elements but received {} elements",
                        expected.len(), actual.len())});
            }
        }
    }
}

fn compare_list_content(path: &Vec<String>, expected: &Vec<Value>, actual: &Vec<Value>, config: &DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    for (index, value) in expected.iter().enumerate() {
      let ps = index.to_string();
      log::debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
      let mut p = path.to_vec();
      p.push(ps);
      if index < actual.len() {
          compare(&p, value, &actual[index], config, mismatches, matchers);
      } else if !matchers.matcher_is_defined("body", &p) {
          mismatches.push(Mismatch::BodyMismatch { path: path.join("."),
              expected: Some(value_of(&json!(expected)).into()),
              actual: Some(value_of(&json!(actual)).into()),
              mismatch: format!("Expected {} but was missing", value_of(value))});
      }
    }
}

fn compare_values(path: &Vec<String>, expected: &Value, actual: &Value, mismatches: &mut Vec<super::Mismatch>,
    matchers: &MatchingRules) {
    let matcher_result = if matchers.matcher_is_defined("body", &path) {
        debug!("Calling match_values for path {}", path.join("."));
        match_values("body", path, matchers.clone(), expected, actual)
    } else {
        expected.matches(actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.join("."), matcher_result);
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::BodyMismatch {
              path: path.join("."),
              expected: Some(format!("{}", expected).into()),
              actual: Some(format!("{}", actual).into()),
              mismatch: message.clone()
            })
          }
        },
        Ok(_) => ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use expectest::expect;
    use crate::Mismatch;
    use crate::DiffConfig;

    #[test]
    fn match_json_handles_invalid_expected_json() {
        let mut mismatches = vec![];
        let expected = s!(r#"{"json": "is bad"#);
        let actual = s!("{}");
        match_json(&expected.clone().into(), &actual.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.into()),
            actual: Some(actual.into()), mismatch: s!("")}));
    }

    #[test]
    fn match_json_handles_invalid_actual_json() {
        let mut mismatches = vec![];
        let expected = s!("{}");
        let actual = s!(r#"{json: "is bad"}"#);
        match_json(&expected.clone().into(), &actual.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.into()),
            actual: Some(actual.into()), mismatch: s!("Type mismatch: Expected List [{}] but received Map {}")}));
    }

    fn mismatch_message(mismatch: &Mismatch) -> String {
        match mismatch {
            &Mismatch::BodyMismatch{ path: _, expected: _, actual: _, mismatch: ref m } => m.clone(),
            _ => s!("")
        }
    }

    #[test]
    fn match_json_handles_expecting_a_map_but_getting_a_list() {
        let mut mismatches = vec![];
        let expected = s!(r#"{}"#);
        let actual = s!(r#"[]"#);
        match_json(&expected.clone().into(), &actual.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.into()),
            actual: Some(actual.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Type mismatch: Expected Map {} but received List []")));
    }

    #[test]
    fn match_json_handles_expecting_a_list_but_getting_a_map() {
        let mut mismatches = vec![];
        let expected = s!(r#"[{}]"#);
        let actual = s!(r#"{}"#);
        match_json(&expected.clone().into(), &actual.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.into()),
            actual: Some(actual.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Type mismatch: Expected List [{}] but received Map {}")));
    }

    #[test]
    fn match_json_handles_comparing_strings() {
        let mut mismatches = vec![];
        let val1 = s!(r#""string value""#);
        let val2 = s!(r#""other value""#);
        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(val1.into()),
            actual: Some(val2.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'string value' to be equal to 'other value'")));
    }

    #[test]
    fn match_json_handles_comparing_integers() {
        let mut mismatches = vec![];
        let val1 = s!(r#"100"#);
        let val2 = s!(r#"200"#);
        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(val1.into()),
            actual: Some(val2.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '100' to be equal to '200'")));
    }

    #[test]
    fn match_json_handles_comparing_floats() {
        let mut mismatches = vec![];
        let val1 = s!(r#"100.01"#);
        let val2 = s!(r#"100.02"#);
        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(val1.into()),
            actual: Some(val2.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '100.01' to be equal to '100.02'")));
    }

    #[test]
    fn match_json_handles_comparing_booleans() {
        let mut mismatches = vec![];
        let val1 = s!(r#"true"#);
        let val2 = s!(r#"false"#);
        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(val1.into()),
            actual: Some(val2.into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'true' to be equal to 'false'")));
    }

    #[test]
    fn match_json_handles_comparing_nulls() {
        let mut mismatches = vec![];
        let val1 = s!(r#"null"#);
        let val2 = s!(r#"33"#);
        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(val1.clone().into()),
            actual: Some(val2.clone().into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'null' to be equal to '33'")));
    }

    #[test]
    fn match_json_handles_comparing_lists() {
        let mut mismatches = vec![];
        let val1 = s!(r#"[]"#);
        let val2 = s!(r#"[11,22,33]"#);
        let val3 = s!(r#"[11,44,33]"#);
        let val4 = s!(r#"[11,44,33, 66]"#);

        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val2.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val3.clone().into(), &val3.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected an empty List but received [11,22,33]")));
        mismatches.clear();

        match_json(&val2.clone().into(), &val3.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.1"),
            expected: Some("22".into()), actual: Some("44".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '22' to be equal to '44'")));
        mismatches.clear();

        match_json(&val3.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("[11,44,33]".into()),
            actual: Some("[11,44,33,66]".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a List with 3 elements but received 4 elements")));
        mismatches.clear();

        match_json(&val2.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(2));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.1"),
            expected: Some("22".into()),
            actual: Some("44".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '22' to be equal to '44'")));
        let mismatch = mismatches[1].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("[11,22,33]".into()),
            actual: Some("[11,44,33,66]".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a List with 3 elements but received 4 elements")));
        mismatches.clear();

        match_json(&val2.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
        match_json(&val4.into(), &val2.into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_json_handles_comparing_maps() {
        let mut mismatches = vec![];
        let val1 = s!(r#"{}"#);
        let val2 = s!(r#"{"a": 1, "b": 2}"#);
        let val3 = s!(r#"{"a": 1, "b": 3}"#);
        let val4 = s!(r#"{"a": 1, "b": 2, "c": 3}"#);

        match_json(&val1.clone().into(), &val1.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val2.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val4.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();

        match_json(&val1.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected an empty Map but received {\"a\":1,\"b\":2}")));
        mismatches.clear();

        match_json(&val2.clone().into(), &val3.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.b"),
            expected: Some("2".into()), actual: Some("3".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '2' to be equal to '3'")));
        mismatches.clear();

        match_json(&val2.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(0));
        match_json(&val2.clone().into(), &val4.clone().into(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("{\"a\":1,\"b\":2}".into()),
            actual: Some("{\"a\":1,\"b\":2,\"c\":3}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a Map with 2 elements but received 3 elements")));
        mismatches.clear();

        match_json(&val3.clone().into(), &val4.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.b"),
            expected: Some("3".into()),
            actual: Some("2".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '3' to be equal to '2'")));
        mismatches.clear();

        match_json(&val3.clone().into(), &val4.clone().into(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(2));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("{\"a\":1,\"b\":3}".into()),
            actual: Some("{\"a\":1,\"b\":2,\"c\":3}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a Map with 2 elements but received 3 elements")));
        let mismatch = mismatches[1].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.b"),
            expected: Some("3".into()),
            actual: Some("2".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected '3' to be equal to '2'")));
        mismatches.clear();

        match_json(&val4.clone().into(), &val2.clone().into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(2));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("{\"a\":1,\"b\":2,\"c\":3}".into()),
            actual: Some("{\"a\":1,\"b\":2}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a Map with at least 3 elements but received 2 elements")));
        let mismatch = mismatches[1].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"),
            expected: Some("{\"a\":1,\"b\":2,\"c\":3}".into()),
            actual: Some("{\"a\":1,\"b\":2}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected entry c=3 but was missing")));
        mismatches.clear();

        match_json(&val3.into(), &val2.into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.*" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn equality_matcher_test() {
        let matcher = MatchingRule::Equality;
        expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_err());
        expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_err());
    }

    #[test]
    fn regex_matcher_test() {
        let matcher = MatchingRule::Regex(s!("^\\d+$"));
        expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("10a")), &matcher)).to(be_err());
        expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_ok());
    }

  #[test]
  fn includes_matcher_test() {
    let matcher = MatchingRule::Include(s!("10"));
    expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_ok());
    expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
    expect!(Value::String(s!("100")).matches(&Value::String(s!("1a0")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_ok());
  }

    #[test]
    fn type_matcher_test() {
        let matcher = MatchingRule::Type;
        expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("10a")), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_err());
    }

    #[test]
    fn min_type_matcher_test() {
        let matcher = MatchingRule::MinType(2);
        expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100), json!(100)]), &matcher)).to(be_ok());
        expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100)]), &matcher)).to(be_err());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
    }

    #[test]
    fn max_type_matcher_test() {
        let matcher = MatchingRule::MaxType(1);
        expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100), json!(100)]), &matcher)).to(be_err());
        expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100)]), &matcher)).to(be_ok());
        expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
    }

    #[test]
    fn min_max_type_matcher_test() {
      let matcher = MatchingRule::MinMaxType(2, 3);
      expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100), json!(100)]),
        &matcher)).to(be_ok());
      expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100), json!(100),
        json!(100)]), &matcher)).to(be_ok());
      expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100), json!(100),
        json!(100), json!(100)]), &matcher)).to(be_err());
      expect!(Value::Array(vec![]).matches(&Value::Array(vec![json!(100)]), &matcher)).to(be_err());
      expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_ok());
    }

  #[test]
  fn integer_matcher_test() {
    let matcher = MatchingRule::Integer;
    expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_ok());
    expect!(Value::String(s!("100")).matches(&json!(100.02), &matcher)).to(be_err());
  }

  #[test]
  fn decimal_matcher_test() {
    let matcher = MatchingRule::Decimal;
    expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100.01), &matcher)).to(be_ok());
  }

  #[test]
  fn number_matcher_test() {
    let matcher = MatchingRule::Number;
    expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_ok());
    expect!(Value::String(s!("100")).matches(&json!(100.01), &matcher)).to(be_ok());
  }

  #[test]
  fn null_matcher_test() {
    let matcher = MatchingRule::Null;
    expect!(Value::String(s!("100")).matches(&Value::String(s!("100")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&Value::String(s!("101")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&Value::String(s!("10a")), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!(100.2), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&json!("null"), &matcher)).to(be_err());
    expect!(Value::String(s!("100")).matches(&Value::Null, &matcher)).to(be_ok());
  }

    #[test]
    fn compare_maps_handles_wildcard_matchers() {
        let mut mismatches = vec![];
        let val1 = s!(r#"
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
        let val2 = s!(r#"{
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

        match_json(&val1.into(), &val2.into(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.articles[*].variants.*" => [ MatchingRule::Type ],
                "$.articles[*].variants.*.bundles.*" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
        mismatches.clear();
    }

}
