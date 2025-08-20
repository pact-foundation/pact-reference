//! Matching rule implementations

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::str::from_utf8;

use anyhow::anyhow;
use bytes::Bytes;
use itertools::Itertools;
#[cfg(feature = "plugins")] use lazy_static::lazy_static;
use maplit::hashmap;
use onig::Regex;
#[cfg(feature = "plugins")]  use pact_plugin_driver::catalogue_manager::{
  CatalogueEntry,
  CatalogueEntryProviderType,
  CatalogueEntryType,
  register_core_entries
};
use semver::Version;
use serde_json::{self, json, Value};
use tracing::{debug, instrument, trace};

use pact_models::HttpStatus;
use pact_models::matchingrules::{
  Category,
  MatchingRule,
  MatchingRuleCategory,
  RuleList,
  RuleLogic
};
use pact_models::path_exp::DocPath;
#[cfg(feature = "datetime")] use pact_models::time_utils::validate_datetime;

use crate::{CommonMismatch, Either, MatchingContext, merge_result};
use crate::binary_utils::match_content_type;

#[cfg(feature = "plugins")]
lazy_static! {
  /// Content matcher/generator entries to add to the plugin catalogue
  static ref CONTENT_MATCHER_CATALOGUE_ENTRIES: Vec<CatalogueEntry> = {
    let mut entries = vec![];
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_MATCHER,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "xml".to_string(),
      values: hashmap!{
        "content-types".to_string() => "application/.*xml,text/xml".to_string()
      }
    });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_MATCHER,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "json".to_string(),
      values: hashmap!{
        "content-types".to_string() => "application/.*json,application/json-rpc,application/jsonrequest".to_string()
      }
    });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_MATCHER,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "text".to_string(),
      values: hashmap!{
        "content-types".to_string() => "text/plain".to_string()
      }
    });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_MATCHER,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "multipart-form-data".to_string(),
      values: hashmap!{
        "content-types".to_string() => "multipart/form-data,multipart/mixed".to_string()
      }
    });
    // TODO:
    // entries.push(CatalogueEntry {
    //   entry_type: CatalogueEntryType::CONTENT_MATCHER,
    //   provider_type: CatalogueEntryProviderType::CORE,
    //   plugin: None,
    //   key: "form-urlencoded".to_string(),
    //   values: hashmap!{
    //     "content-types".to_string() => "application/x-www-form-urlencoded".to_string()
    //   }
    // });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_GENERATOR,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "json".to_string(),
      values: hashmap!{
        "content-types".to_string() => "application/.*json,application/json-rpc,application/jsonrequest".to_string()
      }
    });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::CONTENT_GENERATOR,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "binary".to_string(),
      values: hashmap!{
        "content-types".to_string() => "application/octet-stream".to_string()
      }
    });
    entries
  };

  static ref MATCHER_CATALOGUE_ENTRIES: Vec<CatalogueEntry> = {
    let mut entries = vec![];
    for matcher in ["v2-regex", "v2-type", "v3-number-type", "v3-integer-type", "v3-decimal-type",
      "v3-date", "v3-time", "v3-datetime", "v2-min-type", "v2-max-type", "v2-minmax-type",
      "v3-includes", "v3-null", "v4-equals-ignore-order", "v4-min-equals-ignore-order",
      "v4-max-equals-ignore-order", "v4-minmax-equals-ignore-order", "v3-content-type",
      "v4-array-contains", "v1-equality", "v4-not-empty", "v4-semver"] {
      entries.push(CatalogueEntry {
        entry_type: CatalogueEntryType::MATCHER,
        provider_type: CatalogueEntryProviderType::CORE,
        plugin: None,
        key: matcher.to_string(),
        values: hashmap!{}
      });
    }
    entries
  };
}

/// Sets up all the core catalogue entries for matchers and generators
pub fn configure_core_catalogue() {
  #[cfg(feature = "plugins")] register_core_entries(CONTENT_MATCHER_CATALOGUE_ENTRIES.as_ref());
  #[cfg(feature = "plugins")] register_core_entries(MATCHER_CATALOGUE_ENTRIES.as_ref());
}

pub(crate) fn display<T: Display>(value: &[T]) -> String {
  let mut buffer = String::default();
  buffer.push('[');
  let string = value.iter().map(|v| v.to_string()).join(", ");
  buffer.push_str(string.as_str());
  buffer.push(']');
  buffer
}

/// Trait for matching rule implementation
#[deprecated(since = "2.0.0-beta.4", note="Use DoMatch instead")]
pub trait Matches<A: Clone> {
  /// If the actual value matches self given the matching rule
  #[deprecated(since = "0.9.2", note="Use matches_with instead")]
  fn matches(&self, actual: &A, matcher: &MatchingRule) -> anyhow::Result<()> {
    self.matches_with(actual.clone(), matcher, false)
  }

  /// If the actual value matches self given the matching rule
  fn matches_with(&self, actual: A, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()>;
}

impl Matches<String> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: String, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.as_str().matches_with(actual.as_str(), matcher, cascaded)
  }
}

impl Matches<&String> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: &String, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.as_str().matches_with(actual.as_str(), matcher, cascaded)
  }
}

impl Matches<&String> for &String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: &String, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.as_str().matches_with(actual.as_str(), matcher, cascaded)
  }
}

impl Matches<&str> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: &str, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_str(), actual, cascaded, false)
  }
}

impl Matches<&str> for &str {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: &str, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(*self, actual, cascaded, false)
  }
}

impl Matches<u64> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.as_str().matches_with(actual, matcher, cascaded)
  }
}

impl Matches<u64> for &str {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("String -> u64: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected '{}' (String) to be the same type as {} (Number)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Number) to be equal to '{}' (String)", actual, self)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      MatchingRule::StatusCode(status) => match_status_code(actual as u16, status),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("String: Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<u64> for u64 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(*self, actual, cascaded, false)
  }
}

impl Matches<f64> for u64 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: f64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("u64 -> f64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected {} (Integer) to be the same type as {} (Decimal)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Decimal) to be equal to {} (Integer)", actual, self)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => Ok(()),
      MatchingRule::Integer => Err(anyhow!("Expected {} to match an integer number", actual)),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<f64> for f64 {
  #[allow(clippy::float_cmp)]
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: f64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    let result = match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if *self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", actual, self))
        }
      },
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => Ok(()),
      MatchingRule::Integer => Err(anyhow!("Expected {} to match an integer number", actual)),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    };
    debug!("f64 -> f64: comparing {} to {} using {:?} == {:?}", self, actual, matcher, result);
    result
  }
}

impl Matches<u64> for f64 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("f64 -> u64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected {} (Decimal) to be the same type as {} (Integer)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Integer) to be equal to {} (Decimal)", actual, self)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Unable to match '{}' using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<u16> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u16, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("String -> u16: comparing '{}' to {} using {:?}", self, actual, matcher);
    self.matches_with(actual as u64, matcher, cascaded)
  }
}

impl Matches<u16> for &str {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u16, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("String -> u16: comparing '{}' to {} using {:?}", self, actual, matcher);
    self.matches_with(actual as u64, matcher, cascaded)
  }
}

impl Matches<u16> for u16 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: u16, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("u16 -> u16: comparing {} to {} using {:?}", self, actual, matcher);
    (*self as u64).matches_with(actual as u64, matcher, cascaded)
  }
}

impl Matches<i64> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("String -> i64: comparing {} to {} using {:?}", self, actual, matcher);
    self.as_str().matches_with(actual, matcher, cascaded)
  }
}

impl Matches<i64> for &str {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("String -> i64: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected '{}' (String) to be the same type as {} (Number)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Number) to be equal to '{}' (String)", actual, self)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<i64> for i64 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i64, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("i64 -> i64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if *self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", actual, self))
        }
      },
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<i32> for String {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i32, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.matches_with(actual as i64, matcher, cascaded)
  }
}

impl Matches<i32> for &str {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i32, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.matches_with(actual as i64, matcher, cascaded)
  }
}

impl Matches<i32> for i32 {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: i32, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    (*self as i64).matches_with(actual as i64, matcher, cascaded)
  }
}

impl Matches<bool> for bool {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: bool, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("bool -> bool: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => if actual == *self {
        Ok(())
      } else {
        Err(anyhow!("Expected {} (Boolean) to be equal to {} (Boolean)", actual, self))
      },
      MatchingRule::Boolean => Ok(()),
      _ => if !cascaded || matcher.can_cascade() {
        Err(anyhow!("Boolean: Unable to match {} using {:?}", self, matcher))
      } else {
        Ok(())
      }
    }
  }
}

impl Matches<Bytes> for Bytes {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: Bytes, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self, &actual, cascaded, false)
  }
}

impl Matches<&Bytes> for Bytes {
  #[instrument(level = "trace")]
  fn matches_with(&self, actual: &Bytes, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self, actual, cascaded, false)
  }
}

impl <T: Debug + Display + PartialEq> Matches<&[T]> for &[T] {
  fn matches_with(&self, actual: &[T], matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(*self, actual, cascaded, false)
  }
}

impl <T: Debug + Display + PartialEq + Clone> Matches<&Vec<T>> for &Vec<T> {
  fn matches_with(&self, actual: &Vec<T>, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_slice(), actual.as_slice(), cascaded, false)
  }
}

impl Matches<Vec<u8>> for Vec<u8> {
  fn matches_with(&self, actual: Vec<u8>, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_slice(), actual.as_slice(), cascaded, false)
  }
}

impl Matches<&Vec<u8>> for Vec<u8> {
  fn matches_with(&self, actual: &Vec<u8>, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_slice(), actual.as_slice(), cascaded, false)
  }
}

impl Matches<&[u8]> for Vec<u8> {
  fn matches_with(&self, actual: &[u8], matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_slice(), actual, cascaded, false)
  }
}

impl Matches<&[u8]> for &Vec<u8> {
  fn matches_with(&self, actual: &[u8], matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    matcher.match_value(self.as_slice(), actual, cascaded, false)
  }
}

impl <T: Debug + Display + Clone + PartialEq> Matches<&BTreeMap<String, T>> for BTreeMap<String, T> {
  fn matches_with(&self, actual: &BTreeMap<String, T>, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    debug!("map -> map: comparing [String -> {}] to [String -> {}] using {:?}", std::any::type_name::<T>(),
      std::any::type_name::<T>(), matcher);
    let result = match matcher {
      MatchingRule::Regex(_) => Ok(()),
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual, actual.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual, actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual, actual.len(), min))
        } else if !cascaded && actual.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual, actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", actual.for_mismatch(), self.for_mismatch()))
        }
      }
      MatchingRule::NotEmpty => {
        if actual.is_empty() {
          Err(anyhow!("Expected {} (Array) to not be empty", actual.for_mismatch()))
        } else {
          Ok(())
        }
      }
      MatchingRule::ArrayContains(_) => Ok(()),
      MatchingRule::EachKey(_) => Ok(()),
      MatchingRule::EachValue(_) => Ok(()),
      MatchingRule::Values => Ok(()),
      _ => Err(anyhow!("Unable to match {} using {:?}", self.for_mismatch(), matcher))
    };
    debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
    result
  }
}

/// Trait for matching rule implementation
pub trait DoMatch<T> {
  /// If the actual value matches given the matching rule
  fn match_value(
    &self,
    expected_value: T,
    actual_value: T,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()>;
}

pub(crate) fn value_for_mismatch<T: Display, S: Into<String>>(
  value: T,
  value_type: S,
  show_type: bool
) -> String {
  if show_type {
    format!("{} ({})", value, value_type.into())
  } else {
    format!("{}", value)
  }
}

impl DoMatch<&str> for MatchingRule {
  #[instrument(level = "trace")]
  fn match_value(
    &self,
    expected_value: &str,
    actual_value: &str,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    let result = match self {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(actual_value) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", actual_value, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality | MatchingRule::Values => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to be equal to '{}'", actual_value, expected_value))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Include(substr) => {
        if actual_value.contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to include '{}'", actual_value, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => {
        match actual_value.parse::<f64>() {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match a number", actual_value))
        }
      },
      MatchingRule::Integer => {
        match actual_value.parse::<u64>() {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match an integer number", actual_value))
        }
      },
      #[allow(unused_variables)]
      MatchingRule::Date(s) => {
        #[cfg(feature = "datetime")]
        {
          let format = if s.is_empty() {
            "yyyy-MM-dd"
          } else {
            s.as_str()
          };
          match validate_datetime(&actual_value.to_string(), format) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Expected '{}' to match a date pattern of '{}'", actual_value, format))
          }
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
          let format = if s.is_empty() {
            "HH:mm:ss"
          } else {
            s.as_str()
          };
          match validate_datetime(&actual_value.to_string(), format) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Expected '{}' to match a time pattern of '{}'", actual_value, format))
          }
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
          let format = if s.is_empty() {
            "yyyy-MM-dd'T'HH:mm:ssXXX"
          } else {
            s.as_str()
          };
          match validate_datetime(&actual_value.to_string(), format) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Expected '{}' to match a timestamp pattern of '{}'", actual_value, format))
          }
        }
        #[cfg(not(feature = "datetime"))]
        {
          Err(anyhow!("DateTime matchers require the datetime feature to be enabled"))
        }
      },
      MatchingRule::Boolean => {
        if actual_value == "true" || actual_value == "false" {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to match a boolean", value_for_mismatch(actual_value, "String", show_types)))
        }
      }
      MatchingRule::StatusCode(status) => {
        match actual_value.parse::<u16>() {
          Ok(status_code) => match_status_code(status_code, status),
          Err(err) => Err(anyhow!("Unable to match '{}' using {:?} - {}", actual_value, self, err))
        }
      }
      MatchingRule::NotEmpty => {
        if actual_value.is_empty() {
          Err(anyhow!("Expected {} to not be empty", value_for_mismatch(actual_value, "String", show_types)))
        } else {
          Ok(())
        }
      }
      MatchingRule::Semver => {
        match Version::parse(actual_value) {
          Ok(_) => Ok(()),
          Err(err) => Err(anyhow!("'{}' is not a valid semantic version - {}", actual_value, err))
        }
      }
      MatchingRule::ContentType(content_type) => match_content_type(actual_value.as_bytes(), content_type),
      _ => if !cascaded || self.can_cascade() {
        Err(anyhow!("Unable to match '{}' using {:?}", actual_value, self))
      } else {
        Ok(())
      }
    };
    debug!(cascaded, matcher=?self, "String -> String: comparing '{}' to '{}' ==> {}", expected_value, actual_value, result.is_ok());
    result
  }
}

impl DoMatch<u64> for MatchingRule {
  #[instrument(level = "trace")]
  fn match_value(
    &self,
    expected_value: u64,
    actual_value: u64,
    cascaded: bool,
    _show_types: bool
  ) -> anyhow::Result<()> {
    debug!("u64 -> u64: comparing {} to {} using {:?}", expected_value, actual_value, self);
    match self {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual_value.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual_value, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", actual_value, expected_value))
        }
      },
      MatchingRule::Include(substr) => {
        if actual_value.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual_value, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual_value)),
      MatchingRule::StatusCode(status) => match_status_code(actual_value as u16, status),
      _ => if !cascaded || self.can_cascade() {
        Err(anyhow!("Unable to match {} using {:?}", actual_value, self))
      } else {
        Ok(())
      }
    }
  }
}

impl <T: Debug + Display + PartialEq> DoMatch<&[T]> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &[T],
    actual_value: &[T],
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    debug!("slice -> slice: comparing [{}] to [{}] using {:?}", std::any::type_name::<T>(),
      std::any::type_name::<T>(), self);
    let result = match self {
      MatchingRule::Regex(_) => Ok(()),
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {} (size {}) to have minimum size of {}", display(actual_value),
            actual_value.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {} (size {}) to have maximum size of {}", display(actual_value),
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {} (size {}) to have minimum size of {}", display(actual_value),
            actual_value.len(), min))
        } else if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {} (size {}) to have maximum size of {}", display(actual_value),
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", actual_value.for_mismatch(), expected_value.for_mismatch()))
        }
      }
      MatchingRule::NotEmpty => {
        if actual_value.is_empty() {
          Err(anyhow!("Expected {} to not be empty", value_for_mismatch(actual_value.for_mismatch(),
            "Array", show_types)))
        } else {
          Ok(())
        }
      }
      MatchingRule::ArrayContains(_) => Ok(()),
      MatchingRule::EachKey(_) => Ok(()),
      MatchingRule::EachValue(_) => Ok(()),
      MatchingRule::Values => Ok(()),
      MatchingRule::Number | MatchingRule::Decimal | MatchingRule::Integer => Ok(()),
      MatchingRule::Time(_) | MatchingRule::Date(_) | MatchingRule::Timestamp(_) => Ok(()),
      MatchingRule::Include(_) => Ok(()),
      MatchingRule::ContentType(_) => Ok(()),
      MatchingRule::Boolean => Ok(()),
      MatchingRule::Semver => Ok(()),
      _ => Err(anyhow!("Unable to match {} using {:?}", actual_value.for_mismatch(), self))
    };
    debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual_value, self, result);
    result
  }
}

impl <T: Debug + Display + PartialEq> DoMatch<&Vec<T>> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &Vec<T>,
    actual_value: &Vec<T>,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    self.match_value(expected_value.as_slice(), actual_value.as_slice(), cascaded, show_types)
  }
}

impl DoMatch<&Bytes> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &Bytes,
    actual_value: &Bytes,
    cascaded: bool,
    _show_types: bool
  ) -> anyhow::Result<()> {
    debug!("Bytes -> Bytes: comparing {} bytes to {} bytes using {:?}", expected_value.len(),
      actual_value.len(), self);
    match self {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            match from_utf8(actual_value.as_ref()) {
              Ok(s) => if re.is_match(s) {
                Ok(())
              } else {
                Err(anyhow!("Expected '{}' to match '{}'", s, regex))
              }
              Err(err) => Err(anyhow!("Could not convert actual bytes into a UTF-8 string - {}", err))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected '{:?}...' ({} bytes) to be equal to '{:?}...' ({} bytes)",
            expected_value.split_at(10).0, expected_value.len(), actual_value.split_at(10).0,
            actual_value.len()))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Include(substr) => {
        match from_utf8(actual_value.as_ref()) {
          Ok(s) => if s.contains(substr) {
            Ok(())
          } else {
            Err(anyhow!("Expected '{}' to include '{}'", s, substr))
          }
          Err(err) => Err(anyhow!("Could not convert actual bytes into a UTF-8 string - {}", err))
        }
      },
      MatchingRule::ContentType(content_type) => match_content_type(actual_value.as_ref(), content_type),
      MatchingRule::NotEmpty => {
        if actual_value.is_empty() {
          Err(anyhow!("Expected [] (0 bytes) to not be empty"))
        } else {
          Ok(())
        }
      }
      _ => if !cascaded || self.can_cascade() {
        Err(anyhow!("Unable to match '{:?}...' ({} bytes) using {:?}", actual_value.split_at(10).0,
          actual_value.len(), self))
      } else {
        Ok(())
      }
    }
  }
}

impl DoMatch<Bytes> for MatchingRule {
  fn match_value(
    &self,
    expected_value: Bytes,
    actual_value: Bytes,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    self.match_value(&expected_value, &actual_value, cascaded, show_types)
  }
}

impl <T: Debug + PartialEq> DoMatch<&BTreeMap<String, T>> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &BTreeMap<String, T>,
    actual_value: &BTreeMap<String, T>,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    debug!("map -> map: comparing [String -> {}] to [String -> {}] using {:?}", std::any::type_name::<T>(),
      std::any::type_name::<T>(), self);
    let result = match self {
      MatchingRule::Regex(_) => Ok(()),
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual_value,
            actual_value.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual_value,
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual_value,
            actual_value.len(), min))
        } else if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual_value,
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected {:?} to be equal to {:?}", actual_value, expected_value))
        }
      }
      MatchingRule::NotEmpty => {
        if actual_value.is_empty() {
          Err(anyhow!("Expected {:?} (Map) to not be empty", actual_value))
        } else {
          Ok(())
        }
      }
      MatchingRule::ArrayContains(_) => Ok(()),
      MatchingRule::EachKey(_) => Ok(()),
      MatchingRule::EachValue(_) => Ok(()),
      MatchingRule::Values => Ok(()),
      _ => Err(anyhow!("Unable to match {:?} using {:?}", actual_value, self))
    };
    debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", expected_value, actual_value, self, result);
    result
  }
}

impl <T: Debug + PartialEq> DoMatch<&HashMap<String, T>> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &HashMap<String, T>,
    actual_value: &HashMap<String, T>,
    cascaded: bool,
    _show_types: bool
  ) -> anyhow::Result<()> {
    debug!("map -> map: comparing [String -> {}] to [String -> {}] using {:?}", std::any::type_name::<T>(),
      std::any::type_name::<T>(), self);
    let result = match self {
      MatchingRule::Regex(_) => Ok(()),
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual_value,
            actual_value.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual_value,
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if !cascaded && actual_value.len() < *min {
          Err(anyhow!("Expected {:?} (size {}) to have minimum size of {}", actual_value,
            actual_value.len(), min))
        } else if !cascaded && actual_value.len() > *max {
          Err(anyhow!("Expected {:?} (size {}) to have maximum size of {}", actual_value,
            actual_value.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if expected_value == actual_value {
          Ok(())
        } else {
          Err(anyhow!("Expected {:?} to be equal to {:?}", actual_value, expected_value))
        }
      }
      MatchingRule::NotEmpty => {
        if actual_value.is_empty() {
          Err(anyhow!("Expected {:?} (Map) to not be empty", actual_value))
        } else {
          Ok(())
        }
      }
      MatchingRule::ArrayContains(_) => Ok(()),
      MatchingRule::EachKey(_) => Ok(()),
      MatchingRule::EachValue(_) => Ok(()),
      MatchingRule::Values => Ok(()),
      _ => Err(anyhow!("Unable to match {:?} using {:?}", actual_value, self))
    };
    debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", expected_value, actual_value, self, result);
    result
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

impl <T: Display> DisplayForMismatch for BTreeMap<String, T> {
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

impl <T: Display> DisplayForMismatch for HashSet<T> {
  fn for_mismatch(&self) -> String {
    let mut values = self.iter().map(|v| v.to_string()).collect::<Vec<String>>();
    values.sort();
    values.for_mismatch()
  }
}

impl <T: Display> DisplayForMismatch for BTreeSet<T> {
  fn for_mismatch(&self) -> String {
    let mut values = self.iter().map(|v| v.to_string()).collect::<Vec<String>>();
    values.sort();
    values.for_mismatch()
  }
}


/// Match the provided values using the path and matching rules
pub fn match_values<E, A>(path: &DocPath, matching_rules: &RuleList, expected: E, actual: A) -> Result<(), Vec<String>>
where E: Matches<A>, A: Clone {
  trace!("match_values: {} -> {}", std::any::type_name::<E>(), std::any::type_name::<A>());
  if matching_rules.is_empty() {
    Err(vec![format!("No matcher found for path '{}'", path)])
  } else {
    let results = matching_rules.rules.iter().map(|rule| {
      expected.matches_with(actual.clone(), rule, matching_rules.cascaded)
    }).collect::<Vec<anyhow::Result<()>>>();
    let result = match matching_rules.rule_logic {
      RuleLogic::And => {
        if results.iter().all(|result| result.is_ok()) {
          Ok(())
        } else {
          Err(results.iter().filter(|result| result.is_err())
            .map(|result| result.as_ref().unwrap_err().to_string()).collect())
        }
      },
      RuleLogic::Or => {
        if results.iter().any(|result| result.is_ok()) {
          Ok(())
        } else {
          Err(results.iter().filter(|result| result.is_err())
            .map(|result| result.as_ref().unwrap_err().to_string()).collect())
        }
      }
    };
    trace!(?result, "match_values: {} -> {}", std::any::type_name::<E>(), std::any::type_name::<A>());
    result
  }
}

#[instrument(level = "trace")]
fn match_status_code(status_code: u16, status: &HttpStatus) -> anyhow::Result<()> {
  let matches = match status {
    HttpStatus::Information => (100..=199).contains(&status_code),
    HttpStatus::Success => (200..=299).contains(&status_code),
    HttpStatus::Redirect => (300..=399).contains(&status_code),
    HttpStatus::ClientError => (400..=499).contains(&status_code),
    HttpStatus::ServerError => (500..=599).contains(&status_code),
    HttpStatus::StatusCodes(status_codes) => status_codes.contains(&status_code),
    HttpStatus::NonError => status_code < 400,
    HttpStatus::Error => status_code >= 400
  };
  let result = if matches {
    Ok(())
  } else {
    Err(anyhow!("Expected status code {} to be a {}", status_code, status))
  };
  trace!(status_code, ?status, matches, ?result, "matching status code");
  result
}

/// Basic matching implementation for string slices
pub fn match_strings(
  path: &DocPath,
  expected: &str,
  actual: &str,
  context: &dyn MatchingContext
) -> Result<(), Vec<CommonMismatch>> {
  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path {}", path);
    match_values(&path, &context.select_best_matcher(&path), expected, actual)
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err|
    vec![format!("String '{}': {}", path, err)]
    )
  };
  debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path, matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      CommonMismatch {
        path: path.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        description: message.clone()
      }
    }).collect()
  })
}

/// Delegate to the matching rule defined at the given path to compare the key/value maps.
#[tracing::instrument(ret, skip_all, fields(path, rule, cascaded), level = "trace")]
pub fn compare_maps_with_matchingrule<T: Display + Debug + Clone + PartialEq>(
  rule: &MatchingRule,
  cascaded: bool,
  path: &DocPath,
  expected: &BTreeMap<String, T>,
  actual: &BTreeMap<String, T>,
  context: &(dyn MatchingContext + Send + Sync),
  callback: &mut dyn FnMut(&DocPath, &T, &T, &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<CommonMismatch>>
) -> Result<(), Vec<CommonMismatch>> {
  let mut result = Ok(());
  if !cascaded && rule.is_values_matcher() {
    debug!("Values matcher is defined for path {}", path);
    let context = if let MatchingRule::EachValue(def) = rule {
      debug!("Matching {} with EachValue", path);
      let associated_rules = def.rules.iter().filter_map(|rule| {
        match rule {
          Either::Left(rule) => Some(rule.clone()),
          Either::Right(reference) => {
            result = merge_result(result.clone(), Err(vec![CommonMismatch {
              path: path.to_string(),
              expected: format!("{:?}", expected),
              actual: format!("{:?}", actual),
              description: format!("Found an un-resolved reference {}", reference.name)
            }]));
            None
          }
        }
      }).collect();
      let rules = MatchingRuleCategory {
        name: Category::BODY,
        rules: hashmap! {
            path.join("*") => RuleList {
              rules: associated_rules,
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
      };
      context.clone_with(&rules)
    } else {
      context.clone_with(context.matchers())
    };

    for (key, value) in actual.iter() {
      let p = path.join(key);
      if expected.contains_key(key) {
        result = merge_result(result, callback(&p, &expected[key], value, context.as_ref()));
      } else if let Some(first) = expected.values().next() {
        result = merge_result(result, callback(&p, first, value, context.as_ref()));
      }
    }
  } else {
    if let Err(mismatch) = expected.matches_with(actual, rule, cascaded) {
      result = merge_result(result, Err(vec![CommonMismatch {
        path: path.to_string(),
        expected: expected.for_mismatch(),
        actual: actual.for_mismatch(),
        description: mismatch.to_string()
      }]));
    }
    let expected_keys = expected.keys().cloned().collect();
    let actual_keys = actual.keys().cloned().collect();
    result = merge_result(result, context.match_keys(path, &expected_keys, &actual_keys));
    for (key, value) in expected.iter() {
      if actual.contains_key(key) {
        let p = path.join(key);
        result = merge_result(result, callback(&p, value, &actual[key], context));
      }
    }
  }
  result
}

/// Compare the expected and actual lists using the matching rule's logic
#[tracing::instrument(ret, skip_all, fields(path, rule, cascaded), level = "trace")]
pub fn compare_lists_with_matchingrule<T: Display + Debug + PartialEq + Clone + Sized>(
  rule: &MatchingRule,
  path: &DocPath,
  expected: &[T],
  actual: &[T],
  context: &(dyn MatchingContext + Send + Sync),
  cascaded: bool,
  callback: &mut dyn FnMut(&DocPath, &T, &T, &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<CommonMismatch>>
) -> Result<(), Vec<CommonMismatch>> {
  let mut result = vec![];

  if !expected.is_empty() {
    match rule {
      // TODO: need to implement the ignore order matchers (See Pact-JVM core/matchers/src/main/kotlin/au/com/dius/pact/core/matchers/Matchers.kt:133)
      // is EqualsIgnoreOrderMatcher,
      //         is MinEqualsIgnoreOrderMatcher,
      //         is MaxEqualsIgnoreOrderMatcher,
      //         is MinMaxEqualsIgnoreOrderMatcher -> {
      MatchingRule::ArrayContains(variants) => {
        debug!("Matching {} with ArrayContains", path);
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
              if actual.iter().enumerate().find(|&(actual_index, value)| {
                debug!("Comparing list item {} with value '{:?}' to '{:?}'", actual_index, value, expected_value);
                callback(&DocPath::root(), expected_value, value, context.as_ref()).is_ok()
              }).is_none() {
                result.push(CommonMismatch {
                  path: path.to_string(),
                  expected: expected_value.to_string(),
                  actual: actual.for_mismatch(),
                  description: format!("Variant at index {} ({}) was not found in the actual list", index, expected_value)
                });
              };
            },
            None => {
              result.push(CommonMismatch {
                path: path.to_string(),
                expected: expected.for_mismatch(),
                actual: actual.for_mismatch(),
                description: format!("ArrayContains: variant {} is missing from the expected list, which has {} items",
                                  index, expected.len())
              });
            }
          }
        }
      }
      MatchingRule::EachValue(definition) => if !cascaded {
        debug!("Matching {} with EachValue", path);
        let associated_rules = definition.rules.iter().filter_map(|rule| {
          match rule {
            Either::Left(rule) => Some(rule.clone()),
            Either::Right(reference) => {
              result.push(CommonMismatch {
                path: path.to_string(),
                expected: expected.for_mismatch(),
                actual: actual.for_mismatch(),
                description: format!("Found an un-resolved reference {}", reference.name)
              });
              None
            }
          }
        }).collect();
        let rules = MatchingRuleCategory {
          name: Category::BODY,
          rules: hashmap! {
            path.join("*") => RuleList {
              rules: associated_rules,
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        };
        let context = context.clone_with(&rules);
        result.extend(match_list_contents(path, expected, actual, context.as_ref(), callback));
      }
      _ => {
        if let Err(mismatch) = rule.match_value(expected, actual, cascaded, true) {
          result.push(CommonMismatch {
            path: path.to_string(),
            expected: expected.for_mismatch(),
            actual: actual.for_mismatch(),
            description: mismatch.to_string()
          });
        }

        result.extend(match_list_contents(path, expected, actual, context, callback));
      }
    }
  }

  if result.is_empty() {
    Ok(())
  } else {
    Err(result)
  }
}

/// Compare the expected and actual lists using matching rules
pub fn compare_lists_with_matchingrules<T>(
  path: &DocPath,
  matching_rules: &RuleList,
  expected: &[T],
  actual: &[T],
  context: &(dyn MatchingContext + Send + Sync),
  callback: &mut dyn FnMut(&DocPath, &T, &T, &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<CommonMismatch>>
) -> Result<(), Vec<CommonMismatch>>
  where T: Display + Debug + PartialEq + Clone + Sized {
  trace!("compare_lists_with_matchingrules: {} -> {}", std::any::type_name::<T>(), std::any::type_name::<T>());
  let mut mismatches = vec![];
  if matching_rules.is_empty() {
    mismatches.push(CommonMismatch {
      path: path.to_string(),
      expected: format!("{:?}", expected),
      actual: format!("{:?}", actual),
      description: format!("No matcher found for path '{}'", path)
    })
  } else {
    let results = matching_rules.rules.iter().map(|rule| {
      compare_lists_with_matchingrule(&rule, path, expected, actual, context, matching_rules.cascaded, callback)
    }).collect::<Vec<Result<(), Vec<CommonMismatch>>>>();
    match matching_rules.rule_logic {
      RuleLogic::And => for result in results {
        if let Err(err) = result {
          mismatches.extend(err)
        }
      },
      RuleLogic::Or => {
        if results.iter().all(|result| result.is_err()) {
          for result in results {
            if let Err(err) = result {
              mismatches.extend(err)
            }
          }
        }
      }
    }
  }
  trace!(?mismatches, "compare_lists_with_matchingrules: {} -> {}", std::any::type_name::<T>(), std::any::type_name::<T>());

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

fn match_list_contents<T: Display + Debug + PartialEq + Clone + Sized>(
  path: &DocPath,
  expected: &[T],
  actual: &[T],
  context: &(dyn MatchingContext + Send + Sync),
  callback: &mut dyn FnMut(&DocPath, &T, &T, &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<CommonMismatch>>
) -> Vec<CommonMismatch> {
  let mut result = vec![];

  let mut expected_list = expected.to_vec();
  if actual.len() > expected.len() {
    if let Some(first) = expected.first() {
      expected_list.resize(actual.len(), first.clone());
    }
  }

  for (index, value) in expected_list.iter().enumerate() {
    let ps = index.to_string();
    debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
    let p = path.join(ps);
    if index < actual.len() {
      if let Err(mismatches) = callback(&p, value, &actual[index], context) {
        result.extend(mismatches);
      }
    } else if !context.matcher_is_defined(&p) {
      result.push(CommonMismatch {
        path: path.to_string(),
        expected: expected.for_mismatch(),
        actual: actual.for_mismatch(),
        description: format!("Expected {} ({}) but was missing", value, index)
      });
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use std::collections::{BTreeSet, HashMap, HashSet};
  use std::sync::RwLock;

  use expectest::prelude::*;
  use maplit::{btreemap, hashmap};
  #[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::PluginInteractionConfig;
  use serde_json::json;

  use pact_models::{matchingrules, matchingrules_list};
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleList};
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
  use pact_models::path_exp::DocPath;
  use pact_models::prelude::RuleLogic;

  use crate::{CommonMismatch, CoreMatchingContext, DiffConfig, MatchingContext};
  use crate::matchingrules::{compare_lists_with_matchingrule, compare_maps_with_matchingrule};
  #[cfg(not(feature = "plugins"))] use crate::PluginInteractionConfig;

  use super::*;

  #[derive(Debug)]
  struct MockContext {
    pub calls: RwLock<Vec<String>>,
    matchers: MatchingRuleCategory
  }

  impl MatchingContext for MockContext {
    fn matcher_is_defined(&self, path: &DocPath) -> bool {
      let mut w = self.calls.write().unwrap();
      w.push(format!("matcher_is_defined({})", path));
      true
    }

    fn select_best_matcher(&self, _path: &DocPath) -> RuleList {
      todo!()
    }

    fn type_matcher_defined(&self, _path: &DocPath) -> bool {
      todo!()
    }

    fn values_matcher_defined(&self, _path: &DocPath) -> bool {
      todo!()
    }

    fn direct_matcher_defined(&self, _path: &DocPath, _matchers: &HashSet<&str>) -> bool {
      todo!()
    }

    fn match_keys(&self, path: &DocPath, expected: &BTreeSet<String>, actual: &BTreeSet<String>) -> Result<(), Vec<CommonMismatch>> {
      let mut w = self.calls.write().unwrap();
      w.push(format!("match_keys({}, {:?}, {:?})", path, expected, actual));
      Ok(())
    }

    fn plugin_configuration(&self) -> &HashMap<String, PluginInteractionConfig> {
      todo!()
    }

    fn matchers(&self) -> &MatchingRuleCategory {
      &self.matchers
    }

    fn config(&self) -> DiffConfig {
      todo!()
    }

    fn clone_with(&self, _matchers: &MatchingRuleCategory) -> Box<dyn MatchingContext + Send + Sync> {
      let r = self.calls.read().unwrap();
      Box::new(MockContext {
        calls: RwLock::new(r.clone()),
        matchers: MatchingRuleCategory::default()
      })
    }
  }

  #[test]
  fn compare_maps_with_matchingrule_with_no_value_matcher_at_path() {
    let rule = MatchingRule::Type;
    let expected = btreemap!{
      "a".to_string() => "100".to_string(),
      "b".to_string() => "101".to_string()
    };
    let actual = btreemap!{
      "a".to_string() => "101".to_string()
    };

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };
    let result = compare_maps_with_matchingrule(&rule, false, &DocPath::root(),
      &expected, &actual, &context, &mut callback);

    expect!(result).to(be_ok());

    // We expect match keys to be called, then the callback of each key that is also in the
    // actual map
    let v = vec![
      "match_keys($, {\"a\", \"b\"}, {\"a\"})".to_string()
    ];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));
    let v = vec![
      "$.a, 100, 101".to_string()
    ];
    expect!(calls).to(be_equal_to(v));
  }

  #[test]
  fn compare_maps_with_matchingrule_with_value_matcher_at_path() {
    let expected = btreemap!{
      "a".to_string() => "100".to_string()
    };
    let actual = btreemap!{
      "a".to_string() => "101".to_string(),
      "b".to_string() => "102".to_string()
    };

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };
    let result = compare_maps_with_matchingrule(&MatchingRule::Values, false, &DocPath::root(),
      &expected, &actual, &context, &mut callback);
    let rule = MatchingRule::EachValue(MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    });
    let result2 = compare_maps_with_matchingrule(&rule, false, &DocPath::root(),
      &expected, &actual, &context, &mut callback);

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());

    // With a values matcher, we expect the callback to be called for each key in the actual map
    // and no other methods called on the context
    let v: Vec<String> = vec![];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));
    let v = vec![
      "$.a, 100, 101".to_string(),
      "$.b, 100, 102".to_string(),
      "$.a, 100, 101".to_string(),
      "$.b, 100, 102".to_string()
    ];
    expect!(calls).to(be_equal_to(v));
  }

  #[test]
  fn compare_maps_with_matchingrule_with_each_key_matcher_at_path() {
    let expected = btreemap!{
      "a".to_string() => "100".to_string()
    };
    let actual = btreemap!{
      "b".to_string() => "101".to_string(),
      "c".to_string() => "102".to_string()
    };

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };
    let rule = MatchingRule::EachKey(MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    });
    let result = compare_maps_with_matchingrule(&rule, false, &DocPath::root(),
      &expected, &actual, &context, &mut callback);

    expect!(result).to(be_ok());

    let v: Vec<String> = vec!["match_keys($, {\"a\"}, {\"b\", \"c\"})".to_string()];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));
    expect!(calls.iter()).to(be_empty());
  }

  #[test]
  fn compare_maps_with_matchingrule_with_min_type_matcher() {
    let expected = btreemap!{
      "a".to_string() => "100".to_string(),
      "b".to_string() => "101".to_string(),
      "c".to_string() => "102".to_string()
    };
    let actual = btreemap!{
      "b".to_string() => "103".to_string()
    };

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };
    let rule = MatchingRule::MinType(2);
    let result = compare_maps_with_matchingrule(&rule, false, &DocPath::root(),
                                                &expected, &actual, &context, &mut callback);

    expect!(result.unwrap_err()).to(be_equal_to(vec![
      CommonMismatch {
        path: "$".to_string(),
        expected: "{\"a\":\"100\",\"b\":\"101\",\"c\":\"102\"}".to_string(),
        actual: "{\"b\":\"103\"}".to_string(),
        description: "Expected {\"b\": \"103\"} (size 1) to have minimum size of 2".to_string()
      }
    ]));

    let v: Vec<String> = vec!["match_keys($, {\"a\", \"b\", \"c\"}, {\"b\"})".to_string()];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));
    expect!(calls).to(be_equal_to(vec!["$.b, 101, 103".to_string()]));
  }

  #[test]
  fn compare_maps_with_matchingrule_with_max_type_matcher() {
    let expected = btreemap!{
      "a".to_string() => "100".to_string()
    };
    let actual = btreemap!{
      "a".to_string() => "101".to_string(),
      "b".to_string() => "102".to_string(),
      "c".to_string() => "103".to_string()
    };

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };
    let rule = MatchingRule::MaxType(2);
    let result = compare_maps_with_matchingrule(&rule, false, &DocPath::root(),
                                                &expected, &actual, &context, &mut callback);

    expect!(result.unwrap_err()).to(be_equal_to(vec![
      CommonMismatch {
        path: "$".to_string(),
        expected: "{\"a\":\"100\"}".to_string(),
        actual: "{\"a\":\"101\",\"b\":\"102\",\"c\":\"103\"}".to_string(),
        description: "Expected {\"a\": \"101\", \"b\": \"102\", \"c\": \"103\"} (size 3) to have maximum size of 2".to_string()
      }
    ]));

    let v: Vec<String> = vec!["match_keys($, {\"a\"}, {\"a\", \"b\", \"c\"})".to_string()];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));
    expect!(calls).to(be_equal_to(vec![
      "$.a, 100, 101".to_string()
    ]));
  }

  #[test]
  fn compare_lists_with_matchingrule_with_empty_expected_list() {
    let expected = vec![  ];
    let actual = vec![ "one".to_string(), "two".to_string() ];

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };

    let result = compare_lists_with_matchingrule(&MatchingRule::Type,
                                                 &DocPath::root(), &expected, &actual, &context, false, &mut callback);

    expect!(result).to(be_ok());

    let v: Vec<String> = vec![];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v.clone()));
    expect!(calls).to(be_equal_to(v));
  }

  #[test]
  fn compare_lists_with_matchingrule_with_simple_matcher() {
    let expected = vec![ "value one".to_string(), "value two".to_string(), "value three".to_string() ];
    let actual = vec![ "one".to_string(), "two".to_string() ];

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };

    let result = compare_lists_with_matchingrule(&MatchingRule::Type,
      &DocPath::root(), &expected, &actual, &context, false, &mut callback);

    expect!(result).to(be_ok());

    let v: Vec<String> = vec![
      "matcher_is_defined($[2])".to_string()
    ];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));

    let v: Vec<String> = vec![
      "$[0], value one, one".to_string(),
      "$[1], value two, two".to_string()
    ];
    expect!(calls).to(be_equal_to(v));
  }

  #[test]
  fn compare_lists_with_matchingrule_with_each_key_matcher() {
    let expected = vec![ "value one".to_string(), "value two".to_string(), "value three".to_string() ];
    let actual = vec![ "one".to_string(), "two".to_string() ];

    let context = MockContext {
      calls: RwLock::new(vec![]),
      matchers: MatchingRuleCategory::default()
    };
    let mut calls = vec![];
    let mut callback = |p: &DocPath, a: &String, b: &String, _: &(dyn MatchingContext + Send + Sync)| {
      calls.push(format!("{}, {}, {}", p, a, b));
      Ok(())
    };

    let rule = MatchingRule::EachKey(MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    });
    let result = compare_lists_with_matchingrule(&rule, &DocPath::root(),
      &expected, &actual, &context, false, &mut callback);

    expect!(result).to(be_ok());

    let v: Vec<String> = vec![
      "matcher_is_defined($[2])".to_string()
    ];
    expect!(context.calls.read().unwrap().clone()).to(be_equal_to(v));

    let v: Vec<String> = vec![
      "$[0], value one, one".to_string(),
      "$[1], value two, two".to_string()
    ];
    expect!(calls).to(be_equal_to(v));
  }

  #[test_log::test]
  fn each_value_matcher_with_a_regex_on_a_list_of_items() {
    let each_value = MatchingRule::EachValue(
      MatchingRuleDefinition::new(
        "00000000000000000000000000000000".to_string(),
        ValueType::Unknown,
        MatchingRule::Regex(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|\*".to_string()),
        None,
        "".to_string()
      )
    );
    let expected: &[&str] = &["*"];
    let path = DocPath::root();
    let mut matchers = MatchingRuleCategory::empty("body");
    matchers.add_rule(path.clone(), each_value.clone(), RuleLogic::And);
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &matchers, &hashmap!{});

    let mut callback = |p: &DocPath, a: &&str, b: &&str, c: &(dyn MatchingContext + Send + Sync)| {
      match_strings(p, *a, *b, c)
    };
    let result = compare_lists_with_matchingrule(&each_value, &path,
      expected, &["*", "*"], &context, false, &mut callback);
    expect!(result).to(be_ok());

    let result = compare_lists_with_matchingrule(&each_value, &path,
      expected, &["*", "x"], &context, false, &mut callback);
    expect!(result).to(be_err());
  }

  #[test]
  fn select_best_matcher_selects_most_appropriate_by_weight() {
    let matchers = matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex("1".to_string()) ],
        "$.item1" => [ MatchingRule::Regex("3".to_string()) ],
        "$.item2" => [ MatchingRule::Regex("4".to_string()) ],
        "$.item1.level" => [ MatchingRule::Regex("6".to_string()) ],
        "$.item1.level[1]" => [ MatchingRule::Regex("7".to_string()) ],
        "$.item1.level[1].id" => [ MatchingRule::Regex("8".to_string()) ],
        "$.item1.level[1].name" => [ MatchingRule::Regex("9".to_string()) ],
        "$.item1.level[2]" => [ MatchingRule::Regex("10".to_string()) ],
        "$.item1.level[2].id" => [ MatchingRule::Regex("11".to_string()) ],
        "$.item1.level[*].id" => [ MatchingRule::Regex("12".to_string()) ],
        "$.*.level[*].id" => [ MatchingRule::Regex("13".to_string()) ]
      },
      "header" => {
        "item1" => [ MatchingRule::Regex("5".to_string()) ]
      }
    };
    let body_matchers = matchers.rules_for_category("body").unwrap();
    let header_matchers = matchers.rules_for_category("header").unwrap();

    expect!(body_matchers.select_best_matcher(&vec!["$"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("1".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "a"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("1".to_string()))));

    expect!(body_matchers.select_best_matcher(&vec!["$", "item1"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("3".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("4".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item3"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("1".to_string()))));

    expect!(header_matchers.select_best_matcher(&vec!["$", "item1"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("5".to_string()))));

    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("6".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("7".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "2"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("10".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "id"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("8".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "name"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("9".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "other"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("7".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "2", "id"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("11".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "3", "id"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("12".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2", "level", "1", "id"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("13".to_string()))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2", "level", "3", "id"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("13".to_string()))));
  }

  #[test]
  fn select_best_matcher_selects_most_appropriate_when_weight_is_equal() {
    let matchers = matchingrules!{
      "body" => {
          "$.animals" => [ MatchingRule::Regex("1".to_string()) ],
          "$.animals.*" => [ MatchingRule::Regex("2".to_string()) ],
          "$.animals.*.alligator['@phoneNumber']" => [ MatchingRule::Regex("3".to_string()) ]
      },
      "header" => {
          "item1" => [ MatchingRule::Regex("5".to_string()) ]
      }
    };
    let body_matchers = matchers.rules_for_category("body").unwrap();

    expect!(body_matchers.select_best_matcher(&vec!["$", "animals", "0"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("2".to_string()))));
  }

  #[test]
  fn select_best_matcher_selects_handles_missing_type_attribute() {
    let matchers = matchingrules_list! {
        "body";
        "$.item1" => [ MatchingRule::Regex("3".to_string()) ],
        "$.item2" => [ MatchingRule::MinType(4) ],
        "$.item3" => [ MatchingRule::MaxType(4) ],
        "$.item4" => [ ]
      };

    expect!(matchers.select_best_matcher(&vec!["$", "item1"])).to(
      be_equal_to(RuleList::new(MatchingRule::Regex("3".to_string()))));
    expect!(matchers.select_best_matcher(&vec!["$", "item2"])).to(
      be_equal_to(RuleList::new(MatchingRule::MinType(4))));
    expect!(matchers.select_best_matcher(&vec!["$", "item3"])).to(
      be_equal_to(RuleList::new(MatchingRule::MaxType(4))));
    expect!(matchers.select_best_matcher(&vec!["$", "item4"]).is_empty()).to(be_true());
  }

  #[test]
  fn equality_matcher_test() {
    let matcher = MatchingRule::Equality;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "101", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.0, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.0, &matcher, false)).to(be_err());
  }

  #[test]
  fn regex_matcher_test() {
    let matcher = MatchingRule::Regex("^\\d+$".to_string());
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.01f64, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.02f64, &matcher, false)).to(be_err());

    // Test for Issue #214
    let matcher = MatchingRule::Regex("^Greater|GreaterOrEqual$".to_string());
    expect!(matcher.match_value("Greater", "Greater", false, false)).to(be_ok());
    expect!(matcher.match_value("Greater", "GreaterOrEqual", false, false)).to(be_ok());
  }

  #[test]
  fn type_matcher_test() {
    let matcher = MatchingRule::Type;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn min_type_matcher_test() {
    let matcher = MatchingRule::MinType(3);
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn max_type_matcher_test() {
    let matcher = MatchingRule::MaxType(3);
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn minmax_type_matcher_test() {
    let matcher = MatchingRule::MinMaxType(3, 6);
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  #[cfg(feature = "datetime")]
  fn timestamp_matcher_test() {
    let matcher = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ssZZZ".into());

    expect!(matcher.match_value("100", "2013-12-01 14:00:00+10:00", false, false)).to(be_err());
    expect!(matcher.match_value("100", "2013-12-01 14:00:00+1000", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "13-12-01 14:00:00+10:00", false, false)).to(be_err());
    expect!(matcher.match_value("100", "I\'m a timestamp!", false, false)).to(be_err());
    expect!(matcher.match_value("100", "100", false, false)).to(be_err());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_err());

    let matcher = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ssXXX".into());
    expect!(matcher.match_value("2014-01-01 14:00:00+10:00", "2013-12-01 14:00:00+10:00", false, false)).to(be_ok());

    let matcher = MatchingRule::Timestamp("yyyy#MM#dd#HH#mm#ss".into());
    expect!(matcher.match_value("2014-01-01 14:00:00+10:00", "2013#12#01#14#00#00", false, false)).to(be_ok());

    let matcher = MatchingRule::Timestamp("".into());
    expect!(matcher.match_value("", "2013-12-01T14:00:00+10:00", false, false)).to(be_ok());
  }

  #[test]
  #[cfg(feature = "datetime")]
  fn time_matcher_test() {
    let matcher = MatchingRule::Time("HH:mm:ss".into());

    expect!(matcher.match_value("00:00:00", "14:00:00", false, false)).to(be_ok());
    expect!(matcher.match_value("00:00:00", "33:00:00", false, false)).to(be_err());
    expect!(matcher.match_value("00:00:00", "100", false, false)).to(be_err());
    expect!(matcher.match_value("00:00:00", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("00:00:00", "1000", false, false)).to(be_err());

    expect!("00:00:00".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_err());

    let matcher = MatchingRule::Time("mm:ss".into());
    expect!(matcher.match_value("100", "14:01:01", false, false)).to(be_err());
    expect!(matcher.match_value("100", "61:01", false, false)).to(be_err());

    let matcher = MatchingRule::Time("ss:mm:HH".into());
    expect!(matcher.match_value("100", "05:10:14", false, false)).to(be_ok());

    let matcher = MatchingRule::Time("".into());
    expect!(matcher.match_value("100", "14:00:00", false, false)).to(be_ok());
  }

  #[test]
  #[cfg(feature = "datetime")]
  fn date_matcher_test() {
    let matcher = MatchingRule::Date("yyyy-MM-dd".into());
    let matcher2 = MatchingRule::Date("MM/dd/yyyy".into());

    expect!(matcher.match_value("100", "2001-10-01", false, false)).to(be_ok());
    expect!(matcher2.match_value("100", "01/14/2001", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "01-13-01", false, false)).to(be_err());
    expect!(matcher.match_value("100", "100", false, false)).to(be_err());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_err());

    let matcher = MatchingRule::Date("".into());
    expect!(matcher.match_value("", "2001-10-01", false, false)).to(be_ok());
  }

  #[test]
  fn include_matcher_test() {
    let matcher = MatchingRule::Include("10".into());
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "200", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_ok());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn number_matcher_test() {
    let matcher = MatchingRule::Number;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_ok());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn integer_matcher_test() {
    let matcher = MatchingRule::Integer;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_ok());
    expect!(100.matches_with(200, &matcher, false)).to(be_ok());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_err());
  }

  #[test]
  fn decimal_matcher_test() {
    let matcher = MatchingRule::Decimal;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_ok());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_ok());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_ok());
  }

  #[test]
  fn null_matcher_test() {
    let matcher = MatchingRule::Null;
    expect!(matcher.match_value("100", "100", false, false)).to(be_err());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());
    expect!(matcher.match_value("100", "1000", false, false)).to(be_err());

    expect!("100".matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(200, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(100.1f64.matches_with(100.2, &matcher, false)).to(be_err());
  }

  #[test]
  fn regex_matcher_supports_crazy_regexes() {
    let matcher = MatchingRule::Regex(
      r"^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))?)$"
        .into());
    expect!(matcher.match_value("100", "2019-09-27", false, false)).to(be_ok());
  }

  #[test]
  fn boolean_matcher_test() {
    let matcher = MatchingRule::Boolean;
    expect!(matcher.match_value("100", "100", false, false)).to(be_err());
    expect!(matcher.match_value("100", "10a", false, false)).to(be_err());

    expect!("100".to_string().matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!("100".to_string().matches_with("true", &matcher, false)).to(be_ok());
    expect!("100".to_string().matches_with("false", &matcher, false)).to(be_ok());
    expect!(false.matches_with(true, &matcher, false)).to(be_ok());
  }

  #[test]
  fn match_status_code_test() {
    expect!(match_status_code(100, &HttpStatus::Information)).to(be_ok());
    expect!(match_status_code(199, &HttpStatus::Information)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::Information)).to(be_err());
    expect!(match_status_code(200, &HttpStatus::Success)).to(be_ok());
    expect!(match_status_code(400, &HttpStatus::Success)).to(be_err());
    expect!(match_status_code(301, &HttpStatus::Redirect)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::Redirect)).to(be_err());
    expect!(match_status_code(404, &HttpStatus::ClientError)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::ClientError)).to(be_err());
    expect!(match_status_code(503, &HttpStatus::ServerError)).to(be_ok());
    expect!(match_status_code(499, &HttpStatus::ServerError)).to(be_err());
    expect!(match_status_code(200, &HttpStatus::StatusCodes(vec![200, 201, 204]))).to(be_ok());
    expect!(match_status_code(202, &HttpStatus::StatusCodes(vec![200, 201, 204]))).to(be_err());
    expect!(match_status_code(333, &HttpStatus::NonError)).to(be_ok());
    expect!(match_status_code(599, &HttpStatus::NonError)).to(be_err());
    expect!(match_status_code(555, &HttpStatus::Error)).to(be_ok());
    expect!(match_status_code(99, &HttpStatus::Error)).to(be_err());
  }

  #[test]
  fn not_empty_matcher_test() {
    let matcher = MatchingRule::NotEmpty;
    expect!(matcher.match_value("100", "100", false, false)).to(be_ok());
    expect!(matcher.match_value("100", "", false, false)).to(be_err());

    expect!("100".to_string().matches_with(100, &matcher, false)).to(be_err());
    expect!(100.matches_with(100.1, &matcher, false)).to(be_err());
    expect!(vec![100].matches_with(vec![100], &matcher, false)).to(be_ok());
    expect!(vec![100].matches_with(vec![], &matcher, false)).to(be_err());
    expect!(json!([100]).matches_with(&json!([100]), &matcher, false)).to(be_ok());
    expect!(json!([100]).matches_with(&json!([]), &matcher, false)).to(be_err());
    expect!(json!({"num": 100}).matches_with(&json!({"num": 100}), &matcher, false)).to(be_ok());
    expect!(json!({"num": 100}).matches_with(&json!({}), &matcher, false)).to(be_err());
  }

  #[test]
  fn semver_matcher_test() {
    let matcher = MatchingRule::Semver;
    expect!(matcher.match_value("1.0.0", "1.0.0", false, false)).to(be_ok());
    expect!(matcher.match_value("1.0.0", "1", false, false)).to(be_err());
    expect!(matcher.match_value("1.0.0", "1.0.0-beta.1", false, false)).to(be_ok());

    expect!(json!("1.0.0").matches_with(&json!("1.0.0"), &matcher, false)).to(be_ok());
    expect!(json!("1.0.0").matches_with(&json!("1"), &matcher, false)).to(be_err());
  }

  #[test]
  fn content_type_matcher_test() {
    let matcher = MatchingRule::ContentType("text/plain".to_string());
    expect!(matcher.match_value("plain text", "plain text", false, false)).to(be_ok());
    expect!(matcher.match_value("plain text", "different text", false, false)).to(be_ok());

    expect!("plain text".matches_with(100, &matcher, false)).to(be_err());
    {
      let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <note>
        <to>Tove</to>
        <from>Jani</from>
        <heading>Reminder</heading>
        <body>Don't forget me this weekend!</body>
      </note>"#;
      expect!(matcher.match_value("plain text", xml, false, false)).to(be_err());
    }
  }
}
