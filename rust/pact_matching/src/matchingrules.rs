//! `matchingrules` module includes all the classes to deal with V3 format matchers

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::str::{self, from_utf8};

use anyhow::anyhow;
use log::*;
use onig::Regex;
use serde_json::{self, json, Value};

use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};

use crate::{MatchingContext, merge_result, Mismatch};
use crate::binary_utils::match_content_type;
use crate::matchers::{match_values, Matches};

impl <T: Debug + Display + PartialEq + Clone> Matches<&Vec<T>> for &Vec<T> {
  fn matches_with(&self, actual: &Vec<T>, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    let result = match matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            let text: String = actual.iter().map(|v| v.to_string()).collect();
            if re.is_match(text.as_str()) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", text, regex))
            }
          }
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      }
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality | MatchingRule::Values => {
        if *self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {:?} to be equal to {:?}", actual, self))
        }
      }
      _ => Err(anyhow!("Unable to match {:?} using {:?}", self, matcher))
    };
    debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
    result
  }
}

impl Matches<&[u8]> for Vec<u8> {
  fn matches_with(&self, actual: &[u8], matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    let result = match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            let text = from_utf8(actual).unwrap_or_default();
            if re.is_match(text) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", text, regex))
            }
          }
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      }
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if *self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {:?} to be equal to {:?}", actual, self))
        }
      }
      MatchingRule::ContentType(ref expected_content_type) => {
        match_content_type(&actual, expected_content_type)
          .map_err(|err| anyhow!("Expected data to have a content type of '{}' but was {}", expected_content_type, err))
      }
      _ => Err(anyhow!("Unable to match {:?} using {:?}", self, matcher))
    };
    debug!("Comparing list with {} items to one with {} items using {:?} -> {:?}", self.len(), actual.len(), matcher, result);
    result
  }
}

impl Matches<&[u8]> for &Vec<u8> {
  fn matches_with(&self, actual: &[u8], matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    (*self).matches_with(actual, matcher, cascaded)
  }
}

/// Trait to convert a expected or actual complex object into a string that can be used for a mismatch
pub trait DisplayForMismatch {
  /// Return a string representation that can be used in a mismatch to display to the user
  fn for_mismatch(&self) -> String;
}

impl <T: Display> DisplayForMismatch for HashMap<String, T> {
  fn for_mismatch(&self) -> String {
    Value::Object(self.iter().map(|(k, v)| (k.clone(), json!(v.to_string()))).collect()).to_string()
  }
}

impl <T: Display> DisplayForMismatch for Vec<T> {
  fn for_mismatch(&self) -> String {
    Value::Array(self.iter().map(|v| json!(v.to_string())).collect()).to_string()
  }
}

impl <T: Display> DisplayForMismatch for &[T] {
  fn for_mismatch(&self) -> String {
    Value::Array(self.iter().map(|v| json!(v.to_string())).collect()).to_string()
  }
}

/// Delegate to the matching rule defined at the given path to compare the key/value maps.
pub fn compare_maps_with_matchingrule<T: Display + Debug>(
  _rule: &MatchingRule,
  path: &[&str],
  expected: &HashMap<String, T>,
  actual: &HashMap<String, T>,
  context: &MatchingContext,
  callback: &mut dyn FnMut(&Vec<&str>, &T, &T
  ) -> Result<(), Vec<Mismatch>>) -> Result<(), Vec<Mismatch>> {
  let mut result = Ok(());
  if context.values_matcher_defined(&path) {
    trace!("Values matcher is defined for path {:?}", path);
    for (key, value) in actual.iter() {
      let mut p = path.to_vec();
      p.push(key);
      if expected.contains_key(key) {
        result = merge_result(result, callback(&p, &expected[key], value));
      } else if !expected.is_empty() {
        result = merge_result(result, callback(&p, &expected.values().next().unwrap(), value));
      }
    }
  } else {
    result = merge_result(result, context.match_keys(path, &expected, &actual));
    for (key, value) in expected.iter() {
      if actual.contains_key(key) {
        let mut p = path.to_vec();
        p.push(key);
        result = merge_result(result, callback(&p, value, &actual[key]));
      }
    }
  }
  result
}

/// Compare the expected and actual lists using the matching rule's logic
pub fn compare_lists_with_matchingrule<T: Display + Debug + PartialEq + Clone + Sized>(
  rule: &MatchingRule,
  path: &[&str],
  expected: &Vec<T>,
  actual: &Vec<T>,
  context: &MatchingContext,
  callback: &dyn Fn(&[&str], &T, &T, &MatchingContext) -> Result<(), Vec<Mismatch>>
) -> Result<(), Vec<Mismatch>> {
  let mut result = Ok(());
  match rule {
    MatchingRule::ArrayContains(variants) => {
      let variants = if variants.is_empty() {
        expected.iter().enumerate().map(|(index, _)| {
          (index, MatchingRuleCategory::equality("body"), HashMap::default())
        }).collect()
      } else {
        variants.clone()
      };
      for (index, rules, _) in variants {
        match expected.get(index) {
          Some(expected_value) => {
            let context = context.clone_with(&rules);
            let predicate: &dyn Fn(&(usize, &T)) -> bool = &|&(actual_index, value)| {
              debug!("Comparing list item {} with value '{:?}' to '{:?}'", actual_index, value, expected_value);
              callback(&vec!["$"], expected_value, value, &context).is_ok()
            };
            if actual.iter().enumerate().find(predicate).is_none() {
              result = merge_result(result,Err(vec![ Mismatch::BodyMismatch {
                path: path.join("."),
                expected: Some(expected_value.to_string().into()),
                actual: Some(actual.for_mismatch().into()),
                mismatch: format!("Variant at index {} ({}) was not found in the actual list", index, expected_value)
              } ]));
            };
          },
          None => {
            result = merge_result(result,Err(vec![ Mismatch::BodyMismatch {
              path: path.join("."),
              expected: Some(expected.for_mismatch().into()),
              actual: Some(actual.for_mismatch().into()),
              mismatch: format!("ArrayContains: variant {} is missing from the expected list, which has {} items",
                                index, expected.len())
            } ]));
          }
        }
      }
    }
    _ => {
      if let Err(messages) = match_values(path, context, expected, actual) {
        for message in messages {
          result = merge_result(result,Err(vec![ Mismatch::BodyMismatch {
            path: path.join("."),
            expected: Some(expected.for_mismatch().into()),
            actual: Some(actual.for_mismatch().into()),
            mismatch: message.clone()
          } ]));
        }
      }
      let mut expected_list = Vec::new();
      if let Some(expected_example) = expected.first() {
        expected_list.resize(actual.len(), (*expected_example).clone());
      }

      for (index, value) in expected_list.iter().enumerate() {
        let ps = index.to_string();
        debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
        let mut p = path.to_vec();
        p.push(ps.as_str());
        if index < actual.len() {
          result = merge_result(result, callback(&p, value, &actual[index], context));
        } else if !context.matcher_is_defined(&p) {
          result = merge_result(result,Err(vec![ Mismatch::BodyMismatch { path: path.join("."),
            expected: Some(expected.for_mismatch().into()),
            actual: Some(actual.for_mismatch().into()),
            mismatch: format!("Expected {} but was missing", value) } ]))
        }
      }
    }
  }

  result
}
