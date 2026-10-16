//! Matchers for matching dates, times and date-times

use std::marker::PhantomData;

use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::time_utils::parse_pattern;

use crate::patterns::{JsonPattern, Pattern, StringPattern};

/// Match and generate strings that match a date-time format string.
#[derive(Debug)]
pub struct DateTime<Nested: Pattern> {
  /// The example string we generate when asked.
  example: String,
  /// The format string we use to match.
  format: String,
  /// Since we always store `example` as a string, we need to mention our
  /// `Nested` type somewhere. We can do that using the zero-length
  /// `PhantomData` type.
  phantom: PhantomData<Nested>
}

impl <Nested: Pattern> DateTime<Nested> {
  /// Construct a new `DateTime`, given a format string and the example string to
  /// generate.
  pub fn new<S: Into<String>>(format: S, example: S) -> Self {
    DateTime {
      example: example.into(),
      format: format.into(),
      phantom: PhantomData
    }
  }
}

impl <Nested> Pattern for DateTime<Nested>
where
  Nested: Pattern,
  Nested::Matches: From<String> {
  type Matches = Nested::Matches;

  fn to_example(&self) -> Self::Matches {
    From::from(self.example.clone())
  }

  fn to_example_bytes(&self) -> Vec<u8> {
    self.example.as_bytes().to_vec()
  }

  fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
    rules_out.add_rule(path, MatchingRule::Timestamp(self.format.clone()), RuleLogic::And);
  }
}

#[test]
fn datetime_is_pattern() {
  use serde_json::*;
  use expectest::prelude::*;

  let matchable = DateTime::<JsonPattern>::new("EEE, d MMM yyyy HH:mm:ss Z", "Wed, 4 Jul 2001 12:08:56 -0700");
  expect!(matchable.to_example()).to(be_equal_to("Wed, 4 Jul 2001 12:08:56 -0700"));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  let expected_rules = json!({
    "$": {
      "combine": "AND", "matchers": [
        { "match": "datetime", "format": "EEE, d MMM yyyy HH:mm:ss Z" }
      ]
    }
  });
  expect!(rules.to_v3_json()).to(be_equal_to(expected_rules));
}

impl_from_for_pattern!(DateTime<JsonPattern>, JsonPattern);
impl_from_for_pattern!(DateTime<StringPattern>, StringPattern);

#[test]
fn datetime_into() {
  // Make sure we can convert `DateTime` into different pattern types.
  let _: JsonPattern = DateTime::new("yyy-MM-dd", "2000-01-01").into();
  let _: StringPattern = DateTime::new("yyy-MM-dd", "2000-01-01").into();
}

/// Internal helper function called by `datetime!`. Panics if the datetime format string is invalid.
#[doc(hidden)]
pub fn validate_format_string<S: AsRef<str>>(format_str: S) -> String {
  let format_str = format_str.as_ref();
  match parse_pattern(format_str) {
    Ok(_) => format_str.to_string(),
    Err(msg) => panic!("could not parse datetime format string {:?}: {}", format_str, msg),
  }
}

/// A pattern which macthes the datetime format string `$format` and which generates `$example`.
///
/// ```
/// use pact_consumer::*;
///
/// # fn main() {
/// json_pattern!({
///   "created_date": datetime!("yyyy-MM-dd HH:mm:ss", "2001-01-02 25:33:45")
/// });
/// # }
/// ```
#[macro_export]
macro_rules! datetime {
  ($format:expr, $example:expr) => {
    {
      $crate::patterns::DateTime::new($crate::patterns::validate_format_string($format), $example.into())
    }
  }
}
