//! `generators` module includes all the classes to deal with V3/V4 spec generators

#[cfg(test)] use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Index;
use std::str::FromStr;

use anyhow::anyhow;
use chrono::{DateTime, Local};
#[cfg(test)] use expectest::prelude::*;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use maplit::hashmap;
#[cfg(not(target_family = "wasm"))] use onig::{Captures, Regex};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
#[cfg(target_family = "wasm")] use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, trace, warn};
use uuid::Uuid;

use crate::bodies::OptionalBody;
use crate::expression_parser::{contains_expressions, DataType, DataValue, MapValueResolver, parse_expression};
use crate::generators::datetime_expressions::{execute_date_expression, execute_datetime_expression, execute_time_expression};
use crate::json_utils::{get_field_as_string, json_to_string, JsonToNum};
use crate::matchingrules::{Category, MatchingRuleCategory};
use crate::PactSpecification;
use crate::path_exp::{DocPath, PathToken};
use crate::time_utils::{parse_pattern, to_chrono_pattern};

mod datetime_expressions;
mod date_expression_parser;
mod time_expression_parser;

/// Trait to represent matching logic to find a matching variant for the Array Contains generator
pub trait VariantMatcher: Debug {
  /// Finds the matching variant for the given value
  fn find_matching_variant(
    &self,
    value: &Value,
    variants: &Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>
  ) -> Option<(usize, HashMap<DocPath, Generator>)>;

  /// Clones this matcher and returns it in a box
  fn boxed(&self) -> Box<dyn VariantMatcher + Send + Sync>;
}

#[derive(Clone, Debug)]
pub struct NoopVariantMatcher;

impl VariantMatcher for NoopVariantMatcher {
  fn find_matching_variant(
    &self,
    _value: &Value,
    _variants: &Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>
  ) -> Option<(usize, HashMap<DocPath, Generator>)> {
    None
  }

  fn boxed(&self) -> Box<dyn VariantMatcher + Send + Sync> {
    Box::new(self.clone())
  }
}

impl Default for NoopVariantMatcher {
  fn default() -> Self {
    NoopVariantMatcher {}
  }
}

/// Format of UUIDs to generate
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum UuidFormat {
  /// Simple UUID (e.g 936DA01f9abd4d9d80c702af85c822a8)
  Simple,
  /// lower-case hyphenated (e.g 936da01f-9abd-4d9d-80c7-02af85c822a8)
  LowerCaseHyphenated,
  /// Upper-case hyphenated (e.g 936DA01F-9ABD-4D9D-80C7-02AF85C822A8)
  UpperCaseHyphenated,
  /// URN (e.g. urn:uuid:936da01f-9abd-4d9d-80c7-02af85c822a8)
  Urn
}

impl Display for UuidFormat {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      UuidFormat::Simple => write!(f, "simple"),
      UuidFormat::LowerCaseHyphenated => write!(f, "lower-case-hyphenated"),
      UuidFormat::UpperCaseHyphenated => write!(f, "upper-case-hyphenated"),
      UuidFormat::Urn => write!(f, "URN"),
    }
  }
}

impl Default for UuidFormat {
  fn default() -> Self {
    UuidFormat::LowerCaseHyphenated
  }
}

impl FromStr for UuidFormat {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "simple" => Ok(UuidFormat::Simple),
      "lower-case-hyphenated" => Ok(UuidFormat::LowerCaseHyphenated),
      "upper-case-hyphenated" => Ok(UuidFormat::UpperCaseHyphenated),
      "URN" => Ok(UuidFormat::Urn),
      _ => Err(anyhow!("'{}' is not a valid UUID format", s))
    }
  }
}

/// Trait to represent a generator
#[derive(Debug, Clone, Eq)]
pub enum Generator {
  /// Generates a random integer between the min and max values
  RandomInt(i32, i32),
  /// Generates a random UUID value
  Uuid(Option<UuidFormat>),
  /// Generates a random sequence of digits
  RandomDecimal(u16),
  /// Generates a random sequence of hexadecimal digits
  RandomHexadecimal(u16),
  /// Generates a random string of the provided size
  RandomString(u16),
  /// Generates a random string that matches the provided regex
  Regex(String),
  /// Generates a random date that matches either the provided format or the ISO format
  Date(Option<String>, Option<String>),
  /// Generates a random time that matches either the provided format or the ISO format
  Time(Option<String>, Option<String>),
  /// Generates a random timestamp that matches either the provided format or the ISO format
  DateTime(Option<String>, Option<String>),
  /// Generates a random boolean value
  RandomBoolean,
  /// Generates a value that is looked up from the provider state context
  ProviderStateGenerator(String, Option<DataType>),
  /// Generates a URL with the mock server as the base URL
  MockServerURL(String, String),
  /// List of variants which can have embedded generators
  ArrayContains(Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>)
}

impl Generator {
  /// Convert this generator to a JSON struct
  pub fn to_json(&self) -> Option<Value> {
    match self {
      Generator::RandomInt(min, max) => Some(json!({ "type": "RandomInt", "min": min, "max": max })),
      Generator::Uuid(format) => if let Some(format) = format {
        Some(json!({ "type": "Uuid", "format": format.to_string() }))
      } else {
        Some(json!({ "type": "Uuid" }))
      },
      Generator::RandomDecimal(digits) => Some(json!({ "type": "RandomDecimal", "digits": digits })),
      Generator::RandomHexadecimal(digits) => Some(json!({ "type": "RandomHexadecimal", "digits": digits })),
      Generator::RandomString(size) => Some(json!({ "type": "RandomString", "size": size })),
      Generator::Regex(ref regex) => Some(json!({ "type": "Regex", "regex": regex })),
      Generator::Date(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => Some(json!({ "type": "Date", "format": format, "expression": exp })),
          (Some(format), None) => Some(json!({ "type": "Date", "format": format })),
          (None, Some(exp)) => Some(json!({ "type": "Date", "expression": exp })),
          (None, None) => Some(json!({ "type": "Date" }))
        }
      },
      Generator::Time(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => Some(json!({ "type": "Time", "format": format, "expression": exp })),
          (Some(format), None) => Some(json!({ "type": "Time", "format": format })),
          (None, Some(exp)) => Some(json!({ "type": "Time", "expression": exp })),
          (None, None) => Some(json!({ "type": "Time" }))
        }
      },
      Generator::DateTime(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => Some(json!({ "type": "DateTime", "format": format, "expression": exp })),
          (Some(format), None) => Some(json!({ "type": "DateTime", "format": format })),
          (None, Some(exp)) => Some(json!({ "type": "DateTime", "expression": exp })),
          (None, None) => Some(json!({ "type": "DateTime" }))
        }
      },
      Generator::RandomBoolean => Some(json!({ "type": "RandomBoolean" })),
      Generator::ProviderStateGenerator(ref expression, ref data_type) => {
        if let Some(data_type) = data_type {
          Some(json!({"type": "ProviderState", "expression": expression, "dataType": data_type}))
        } else {
          Some(json!({"type": "ProviderState", "expression": expression}))
        }
      }
      Generator::MockServerURL(example, regex) => Some(json!({ "type": "MockServerURL", "example": example, "regex": regex })),
      _ => None
    }
  }

  /// Converts a JSON map into a `Generator` struct, returning `None` if it can not be converted.
  pub fn from_map(gen_type: &str, map: &serde_json::Map<String, Value>) -> Option<Generator> {
    match gen_type {
      "RandomInt" => {
        let min = <i32>::json_to_number(map, "min", 0);
        let max = <i32>::json_to_number(map, "max", 10);
        Some(Generator::RandomInt(min, max))
      },
      "Uuid" => if let Some(format) = map.get("format") {
        Some(Generator::Uuid(str::parse(json_to_string(format).as_str()).ok()))
      } else {
        Some(Generator::Uuid(None))
      },
      "RandomDecimal" => Some(Generator::RandomDecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomHexadecimal" => Some(Generator::RandomHexadecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomString" => Some(Generator::RandomString(<u16>::json_to_number(map, "size", 10))),
      "Regex" => map.get("regex").map(|val| Generator::Regex(json_to_string(val))),
      "Date" => Some(Generator::Date(get_field_as_string("format", map), get_field_as_string("expression", map))),
      "Time" => Some(Generator::Time(get_field_as_string("format", map), get_field_as_string("expression", map))),
      "DateTime" => Some(Generator::DateTime(get_field_as_string("format", map), get_field_as_string("expression", map))),
      "RandomBoolean" => Some(Generator::RandomBoolean),
      "ProviderState" => map.get("expression").map(|f|
        Generator::ProviderStateGenerator(json_to_string(f), map.get("dataType")
          .map(|dt| DataType::from(dt.clone())))),
      "MockServerURL" => Some(Generator::MockServerURL(get_field_as_string("example", map).unwrap_or_default(),
                                                       get_field_as_string("regex", map).unwrap_or_default())),
      _ => {
        warn!("'{}' is not a valid generator type", gen_type);
        None
      }
    }
  }

  /// Builds a `Generator` from a `Value` struct
  pub fn from_json(value: &Value) -> anyhow::Result<Generator> {
    match value {
      Value::Object(m) => match m.get("type") {
        Some(match_val) => {
          let val = json_to_string(match_val);
          if let Some(generator) = Generator::from_map(val.as_str(), m) {
            Ok(generator)
          } else {
            Err(anyhow!("Generator missing 'type' field and unable to guess its type"))
          }
        }
        _ => Err(anyhow!("Generator missing 'type' field and unable to guess its type"))
      },
      _ => Err(anyhow!("Generator JSON is not an Object")),
    }
  }

  /// If this generator is compatible with the given generator mode
  pub fn corresponds_to_mode(&self, mode: &GeneratorTestMode) -> bool {
    match self {
      Generator::ProviderStateGenerator(_, _) => mode == &GeneratorTestMode::Provider,
      Generator::MockServerURL(_, _) => mode == &GeneratorTestMode::Consumer,
      _ => true
    }
  }

  /// Returns the type name of this generator
  pub fn name(&self) -> String {
    match self {
      Generator::RandomInt(_, _) => "RandomInt",
      Generator::Uuid(_) => "Uuid",
      Generator::RandomDecimal(_) => "RandomDecimal",
      Generator::RandomHexadecimal(_) => "RandomHexadecimal",
      Generator::RandomString(_) => "RandomString",
      Generator::Regex(_) => "Regex",
      Generator::Date(_, _) => "Date",
      Generator::Time(_, _) => "Time",
      Generator::DateTime(_, _) => "DateTime",
      Generator::RandomBoolean => "RandomBoolean",
      Generator::ProviderStateGenerator(_, _) => "ProviderStateGenerator",
      Generator::MockServerURL(_, _) => "MockServerURL",
      Generator::ArrayContains(_) => "ArrayContains",
    }.to_string()
  }

  /// Returns the values for this generator
  pub fn values(&self) -> HashMap<&'static str, Value> {
    let empty = hashmap!{};
    match self {
      Generator::RandomInt(min, max) => hashmap!{ "min" => json!(min), "max" => json!(max) },
      Generator::Uuid(format) => if let Some(format) = format {
        hashmap!{ "format" => Value::String(format.to_string()) }
      } else {
        empty
      }
      Generator::RandomDecimal(digits) => hashmap!{ "digits" => json!(digits) },
      Generator::RandomHexadecimal(digits) => hashmap!{ "digits" => json!(digits) },
      Generator::RandomString(digits) => hashmap!{ "digits" => json!(digits) },
      Generator::Regex(r) => hashmap!{ "regex" => json!(r) },
      Generator::Date(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => hashmap!{ "format" => Value::String(format.clone()), "expression" => Value::String(exp.clone()) },
          (Some(format), None) => hashmap!{ "format" => Value::String(format.clone()) },
          (None, Some(exp)) => hashmap!{ "expression" => Value::String(exp.clone()) },
          (None, None) => empty
        }
      }
      Generator::Time(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => hashmap!{ "format" => Value::String(format.clone()), "expression" => Value::String(exp.clone()) },
          (Some(format), None) => hashmap!{ "format" => Value::String(format.clone()) },
          (None, Some(exp)) => hashmap!{ "expression" => Value::String(exp.clone()) },
          (None, None) => empty
        }
      }
      Generator::DateTime(format, exp) => {
        match (format, exp) {
          (Some(format), Some(exp)) => hashmap!{ "format" => Value::String(format.clone()), "expression" => Value::String(exp.clone()) },
          (Some(format), None) => hashmap!{ "format" => Value::String(format.clone()) },
          (None, Some(exp)) => hashmap!{ "expression" => Value::String(exp.clone()) },
          (None, None) => empty
        }
      }
      Generator::RandomBoolean => empty,
      Generator::ProviderStateGenerator(exp, data_type) => if let Some(data_type) = data_type {
        hashmap!{ "expression" => Value::String(exp.clone()), "data_type" => data_type.into() }
      } else {
        hashmap!{ "expression" => Value::String(exp.clone()) }
      }
      Generator::MockServerURL(example, regex) => hashmap!{ "example" => json!(example), "regex" => json!(regex) },
      Generator::ArrayContains(variants) => hashmap!{ "variants" => variants.iter().map(|(variant, rules, gens)| {
          Value::Array(vec![json!(variant), rules.to_v3_json(), Value::Object(gens.iter().map(|(key, gen)| {
            (key.to_string(), gen.to_json().unwrap())
          }).collect())])
        }).collect()
      }
    }
  }

  /// Create a generator from a type and a map of attributes
  pub fn create(generator_type: &str, attributes: &Value) -> anyhow::Result<Generator> {
    match attributes {
      Value::Object(o) => Generator::from_map(generator_type, o)
        .ok_or_else(|| anyhow!("Could not create a generator from '{}' and {}", generator_type, attributes)),
      _ => Err(anyhow!("'{}' is not a valid generator type", generator_type))
    }
  }
}

impl Hash for Generator {
  fn hash<H: Hasher>(&self, state: &mut H) {
    mem::discriminant(self).hash(state);
    match self {
      Generator::RandomInt(min, max) => {
        min.hash(state);
        max.hash(state);
      },
      Generator::RandomDecimal(digits) => digits.hash(state),
      Generator::RandomHexadecimal(digits) => digits.hash(state),
      Generator::RandomString(size) => size.hash(state),
      Generator::Regex(re) => re.hash(state),
      Generator::DateTime(format, exp) => {
        format.hash(state);
        exp.hash(state);
      },
      Generator::Time(format, exp) => {
        format.hash(state);
        exp.hash(state);
      },
      Generator::Date(format, exp) => {
        format.hash(state);
        exp.hash(state);
      },
      Generator::ProviderStateGenerator(str, datatype) => {
        str.hash(state);
        datatype.hash(state);
      },
      Generator::MockServerURL(str1, str2) => {
        str1.hash(state);
        str2.hash(state);
      },
      Generator::ArrayContains(variants) => {
        for (index, rules, generators) in variants {
          index.hash(state);
          rules.hash(state);
          for (s, g) in generators {
            s.hash(state);
            g.hash(state);
          }
        }
      }
      Generator::Uuid(format) => format.hash(state),
      _ => ()
    }
  }
}

impl PartialEq for Generator {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Generator::RandomInt(min1, max1), Generator::RandomInt(min2, max2)) => min1 == min2 && max1 == max2,
      (Generator::RandomDecimal(digits1), Generator::RandomDecimal(digits2)) => digits1 == digits2,
      (Generator::RandomHexadecimal(digits1), Generator::RandomHexadecimal(digits2)) => digits1 == digits2,
      (Generator::RandomString(size1), Generator::RandomString(size2)) => size1 == size2,
      (Generator::Regex(re1), Generator::Regex(re2)) => re1 == re2,
      (Generator::DateTime(format1, exp1), Generator::DateTime(format2, exp2)) => format1 == format2 && exp1 == exp2,
      (Generator::Time(format1, exp1), Generator::Time(format2, exp2)) => format1 == format2 && exp1 == exp2,
      (Generator::Date(format1, exp1), Generator::Date(format2, exp2)) => format1 == format2 && exp1 == exp2,
      (Generator::ProviderStateGenerator(str1, data1), Generator::ProviderStateGenerator(str2, data2)) => str1 == str2 && data1 == data2,
      (Generator::MockServerURL(ex1, re1), Generator::MockServerURL(ex2, re2)) => ex1 == ex2 && re1 == re2,
      (Generator::ArrayContains(variants1), Generator::ArrayContains(variants2)) => variants1 == variants2,
      (Generator::Uuid(format), Generator::Uuid(format2)) => format == format2,
      _ => mem::discriminant(self) == mem::discriminant(other)
    }
  }
}

#[cfg(test)]
fn h(rule: &Generator) -> u64 {
  let mut hasher = DefaultHasher::new();
  rule.hash(&mut hasher);
  hasher.finish()
}

#[test]
fn hash_and_partial_eq_for_matching_rule() {
  expect!(h(&Generator::Uuid(None))).to(be_equal_to(h(&Generator::Uuid(None))));
  expect!(h(&Generator::Uuid(Some(UuidFormat::Simple)))).to(be_equal_to(h(&Generator::Uuid(Some(UuidFormat::Simple)))));
  expect!(h(&Generator::Uuid(Some(UuidFormat::Simple)))).to_not(be_equal_to(h(&Generator::Uuid(Some(UuidFormat::LowerCaseHyphenated)))));
  expect!(Generator::Uuid(None)).to(be_equal_to(Generator::Uuid(None)));
  expect!(Generator::Uuid(Some(UuidFormat::Simple))).to(be_equal_to(Generator::Uuid(Some(UuidFormat::Simple))));
  expect!(Generator::Uuid(Some(UuidFormat::Simple))).to_not(be_equal_to(Generator::Uuid(Some(UuidFormat::LowerCaseHyphenated))));
  expect!(Generator::Uuid(None)).to_not(be_equal_to(Generator::RandomBoolean));

  expect!(h(&Generator::RandomBoolean)).to(be_equal_to(h(&Generator::RandomBoolean)));
  expect!(Generator::RandomBoolean).to(be_equal_to(Generator::RandomBoolean));

  let randint1 = Generator::RandomInt(100, 200);
  let randint2 = Generator::RandomInt(200, 200);

  expect!(h(&randint1)).to(be_equal_to(h(&randint1)));
  expect!(&randint1).to(be_equal_to(&randint1));
  expect!(h(&randint1)).to_not(be_equal_to(h(&randint2)));
  expect!(&randint1).to_not(be_equal_to(&randint2));

  let dec1 = Generator::RandomDecimal(100);
  let dec2 = Generator::RandomDecimal(200);

  expect!(h(&dec1)).to(be_equal_to(h(&dec1)));
  expect!(&dec1).to(be_equal_to(&dec1));
  expect!(h(&dec1)).to_not(be_equal_to(h(&dec2)));
  expect!(&dec1).to_not(be_equal_to(&dec2));

  let hexdec1 = Generator::RandomHexadecimal(100);
  let hexdec2 = Generator::RandomHexadecimal(200);

  expect!(h(&hexdec1)).to(be_equal_to(h(&hexdec1)));
  expect!(&hexdec1).to(be_equal_to(&hexdec1));
  expect!(h(&hexdec1)).to_not(be_equal_to(h(&hexdec2)));
  expect!(&hexdec1).to_not(be_equal_to(&hexdec2));

  let str1 = Generator::RandomString(100);
  let str2 = Generator::RandomString(200);

  expect!(h(&str1)).to(be_equal_to(h(&str1)));
  expect!(&str1).to(be_equal_to(&str1));
  expect!(h(&str1)).to_not(be_equal_to(h(&str2)));
  expect!(&str1).to_not(be_equal_to(&str2));

  let regex1 = Generator::Regex("\\d+".into());
  let regex2 = Generator::Regex("\\w+".into());

  expect!(h(&regex1)).to(be_equal_to(h(&regex1)));
  expect!(&regex1).to(be_equal_to(&regex1));
  expect!(h(&regex1)).to_not(be_equal_to(h(&regex2)));
  expect!(&regex1).to_not(be_equal_to(&regex2));

  let datetime1 = Generator::DateTime(Some("yyyy-MM-dd HH:mm:ss".into()), None);
  let datetime2 = Generator::DateTime(Some("yyyy-MM-ddTHH:mm:ss".into()), None);
  let datetime3 = Generator::DateTime(Some("yyyy-MM-ddTHH:mm:ss".into()), Some("today".into()));

  expect!(h(&datetime1)).to(be_equal_to(h(&datetime1)));
  expect!(&datetime1).to(be_equal_to(&datetime1));
  expect!(h(&datetime1)).to_not(be_equal_to(h(&datetime2)));
  expect!(&datetime1).to_not(be_equal_to(&datetime2));
  expect!(h(&datetime1)).to_not(be_equal_to(h(&datetime3)));
  expect!(&datetime1).to_not(be_equal_to(&datetime3));
  expect!(h(&datetime3)).to(be_equal_to(h(&datetime3)));
  expect!(&datetime3).to(be_equal_to(&datetime3));

  let date1 = Generator::Date(Some("yyyy-MM-dd".into()), None);
  let date2 = Generator::Date(Some("yy-MM-dd".into()), None);
  let date3 = Generator::Date(Some("yy-MM-dd".into()), Some("today".into()));

  expect!(h(&date1)).to(be_equal_to(h(&date1)));
  expect!(&date1).to(be_equal_to(&date1));
  expect!(h(&date1)).to_not(be_equal_to(h(&date2)));
  expect!(&date1).to_not(be_equal_to(&date2));
  expect!(h(&date1)).to_not(be_equal_to(h(&date3)));
  expect!(&date1).to_not(be_equal_to(&date3));
  expect!(h(&date3)).to(be_equal_to(h(&date3)));
  expect!(&date3).to(be_equal_to(&date3));

  let time1 = Generator::Time(Some("HH:mm:ss".into()), None);
  let time2 = Generator::Time(Some("hh:mm:ss".into()), None);
  let time3 = Generator::Time(Some("hh:mm:ss".into()), Some("now".into()));

  expect!(h(&time1)).to(be_equal_to(h(&time1)));
  expect!(&time1).to(be_equal_to(&time1));
  expect!(h(&time1)).to_not(be_equal_to(h(&time2)));
  expect!(&time1).to_not(be_equal_to(&time2));
  expect!(h(&time1)).to_not(be_equal_to(h(&time3)));
  expect!(&time1).to_not(be_equal_to(&time3));
  expect!(h(&time3)).to(be_equal_to(h(&time3)));
  expect!(&time3).to(be_equal_to(&time3));

  let psg1 = Generator::ProviderStateGenerator("string one".into(), Some(DataType::BOOLEAN));
  let psg2 = Generator::ProviderStateGenerator("string two".into(), None);
  let psg3 = Generator::ProviderStateGenerator("string one".into(), None);

  expect!(h(&psg1)).to(be_equal_to(h(&psg1)));
  expect!(&psg1).to(be_equal_to(&psg1));
  expect!(h(&psg1)).to_not(be_equal_to(h(&psg2)));
  expect!(h(&psg1)).to_not(be_equal_to(h(&psg3)));
  expect!(&psg1).to_not(be_equal_to(&psg2));
  expect!(&psg1).to_not(be_equal_to(&psg3));

  let msu1 = Generator::MockServerURL("string one".into(), "\\d+".into());
  let msu2 = Generator::MockServerURL("string two".into(), "\\d+".into());
  let msu3 = Generator::MockServerURL("string one".into(), "\\w+".into());

  expect!(h(&msu1)).to(be_equal_to(h(&msu1)));
  expect!(&msu1).to(be_equal_to(&msu1));
  expect!(h(&msu1)).to_not(be_equal_to(h(&msu2)));
  expect!(h(&msu1)).to_not(be_equal_to(h(&msu3)));
  expect!(&msu1).to_not(be_equal_to(&msu2));
  expect!(&msu1).to_not(be_equal_to(&msu3));

  let ac1 = Generator::ArrayContains(vec![]);
  let ac2 = Generator::ArrayContains(vec![(0, MatchingRuleCategory::empty("body"), hashmap!{})]);
  let ac3 = Generator::ArrayContains(vec![(1, MatchingRuleCategory::empty("body"), hashmap!{})]);
  let ac4 = Generator::ArrayContains(vec![(0, MatchingRuleCategory::equality("body"), hashmap!{})]);
  let ac5 = Generator::ArrayContains(vec![(0, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomBoolean })]);
  let ac6 = Generator::ArrayContains(vec![
    (0, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomBoolean }),
    (1, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomDecimal(10) })
  ]);
  let ac7 = Generator::ArrayContains(vec![
    (0, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomBoolean }),
    (1, MatchingRuleCategory::equality("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomDecimal(10) })
  ]);

  expect!(h(&ac1)).to(be_equal_to(h(&ac1)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac1)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac2)).to(be_equal_to(h(&ac2)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac1)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac2)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac3)).to(be_equal_to(h(&ac3)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac1)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac3)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac4)).to(be_equal_to(h(&ac4)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac1)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac4)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac5)).to(be_equal_to(h(&ac5)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac1)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac5)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac6)).to(be_equal_to(h(&ac6)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac1)));
  expect!(h(&ac6)).to_not(be_equal_to(h(&ac7)));
  expect!(h(&ac7)).to(be_equal_to(h(&ac7)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac2)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac3)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac4)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac5)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac6)));
  expect!(h(&ac7)).to_not(be_equal_to(h(&ac1)));

  expect!(&ac1).to(be_equal_to(&ac1));
  expect!(&ac1).to_not(be_equal_to(&ac2));
  expect!(&ac1).to_not(be_equal_to(&ac3));
  expect!(&ac1).to_not(be_equal_to(&ac4));
  expect!(&ac1).to_not(be_equal_to(&ac5));
  expect!(&ac1).to_not(be_equal_to(&ac6));
  expect!(&ac1).to_not(be_equal_to(&ac7));
  expect!(&ac2).to(be_equal_to(&ac2));
  expect!(&ac2).to_not(be_equal_to(&ac1));
  expect!(&ac2).to_not(be_equal_to(&ac3));
  expect!(&ac2).to_not(be_equal_to(&ac4));
  expect!(&ac2).to_not(be_equal_to(&ac5));
  expect!(&ac2).to_not(be_equal_to(&ac6));
  expect!(&ac2).to_not(be_equal_to(&ac7));
  expect!(&ac3).to(be_equal_to(&ac3));
  expect!(&ac3).to_not(be_equal_to(&ac2));
  expect!(&ac3).to_not(be_equal_to(&ac1));
  expect!(&ac3).to_not(be_equal_to(&ac4));
  expect!(&ac3).to_not(be_equal_to(&ac5));
  expect!(&ac3).to_not(be_equal_to(&ac6));
  expect!(&ac3).to_not(be_equal_to(&ac7));
  expect!(&ac4).to(be_equal_to(&ac4));
  expect!(&ac4).to_not(be_equal_to(&ac2));
  expect!(&ac4).to_not(be_equal_to(&ac3));
  expect!(&ac4).to_not(be_equal_to(&ac1));
  expect!(&ac4).to_not(be_equal_to(&ac5));
  expect!(&ac4).to_not(be_equal_to(&ac6));
  expect!(&ac4).to_not(be_equal_to(&ac7));
  expect!(&ac5).to(be_equal_to(&ac5));
  expect!(&ac5).to_not(be_equal_to(&ac2));
  expect!(&ac5).to_not(be_equal_to(&ac3));
  expect!(&ac5).to_not(be_equal_to(&ac4));
  expect!(&ac5).to_not(be_equal_to(&ac1));
  expect!(&ac5).to_not(be_equal_to(&ac6));
  expect!(&ac5).to_not(be_equal_to(&ac7));
  expect!(&ac6).to(be_equal_to(&ac6));
  expect!(&ac6).to_not(be_equal_to(&ac2));
  expect!(&ac6).to_not(be_equal_to(&ac3));
  expect!(&ac6).to_not(be_equal_to(&ac4));
  expect!(&ac6).to_not(be_equal_to(&ac5));
  expect!(&ac6).to_not(be_equal_to(&ac1));
  expect!(&ac6).to_not(be_equal_to(&ac7));
  expect!(&ac7).to(be_equal_to(&ac7));
  expect!(&ac7).to_not(be_equal_to(&ac2));
  expect!(&ac7).to_not(be_equal_to(&ac3));
  expect!(&ac7).to_not(be_equal_to(&ac4));
  expect!(&ac7).to_not(be_equal_to(&ac5));
  expect!(&ac7).to_not(be_equal_to(&ac6));
  expect!(&ac7).to_not(be_equal_to(&ac1));
}


/// If the generators are being applied in the context of a consumer or provider
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeneratorTestMode {
  /// Generate values in the context of the consumer
  Consumer,
  /// Generate values in the context of the provider
  Provider
}


/// Category that the generator is applied to
#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum GeneratorCategory {
  /// Request Method
  METHOD,
  /// Request Path
  PATH,
  /// Request/Response Header
  HEADER,
  /// Request Query Parameter
  QUERY,
  /// Body
  BODY,
  /// Response Status
  STATUS
}

impl FromStr for GeneratorCategory {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "method" => Ok(GeneratorCategory::METHOD),
      "path" => Ok(GeneratorCategory::PATH),
      "header" => Ok(GeneratorCategory::HEADER),
      "query" => Ok(GeneratorCategory::QUERY),
      "body" => Ok(GeneratorCategory::BODY),
      "status" => Ok(GeneratorCategory::STATUS),
      _ => Err(format!("'{}' is not a valid GeneratorCategory", s))
    }
  }
}

impl <'a> Into<&'a str> for GeneratorCategory {
  fn into(self) -> &'a str {
    match self {
      GeneratorCategory::METHOD => "method",
      GeneratorCategory::PATH => "path",
      GeneratorCategory::HEADER => "header",
      GeneratorCategory::QUERY => "query",
      GeneratorCategory::BODY => "body",
      GeneratorCategory::STATUS => "status"
    }
  }
}

impl Into<String> for GeneratorCategory {
  fn into(self) -> String {
    let s: &str = self.into();
    s.to_string()
  }
}

impl Into<Category> for GeneratorCategory {
  fn into(self) -> Category {
    match self {
      GeneratorCategory::METHOD => Category::METHOD,
      GeneratorCategory::PATH => Category::PATH,
      GeneratorCategory::HEADER => Category::HEADER,
      GeneratorCategory::QUERY => Category::QUERY,
      GeneratorCategory::BODY => Category::BODY,
      GeneratorCategory::STATUS => Category::STATUS
    }
  }
}


/// Trait for something that can generate a value based on a source value.
pub trait GenerateValue<T> {
  /// Generates a new value based on the source value. An error will be returned if the value can not
  /// be generated.
  fn generate_value(
    &self,
    value: &T,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> anyhow::Result<T>;
}

/// Trait to define a handler for applying generators to data of a particular content type.
pub trait ContentTypeHandler<T> {
  /// Processes the body using the map of generators, returning a (possibly) updated body.
  fn process_body(
    &mut self,
    generators: &HashMap<DocPath, Generator>,
    mode: &GeneratorTestMode,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> Result<OptionalBody, String>;
  /// Applies the generator to the key in the body.
  fn apply_key(
    &mut self,
    key: &DocPath,
    generator: &dyn GenerateValue<T>,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  );
}

/// Data structure for representing a collection of generators
#[derive(Debug, Clone, Eq)]
pub struct Generators {
  /// Map of generator categories to maps of generators
  pub categories: HashMap<GeneratorCategory, HashMap<DocPath, Generator>>
}

impl Generators {
  /// If the generators are empty (that is there are no rules assigned to any categories)
  pub fn is_empty(&self) -> bool {
    self.categories.values().all(|category| category.is_empty())
  }

  /// If the generators are not empty (that is there is at least one rule assigned to a category)
  pub fn is_not_empty(&self) -> bool {
    self.categories.values().any(|category| !category.is_empty())
  }

  /// Loads the generators for a JSON map
  pub fn load_from_map(&mut self, map: &serde_json::Map<String, Value>
  ) -> anyhow::Result<()> {
    for (k, v) in map {
      match v {
        &Value::Object(ref map) =>  match GeneratorCategory::from_str(k) {
          Ok(ref category) => match category {
            &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
              self.parse_generator_from_map(category, map, None);
            },
            _ => for (sub_k, sub_v) in map {
              match sub_v {
                &Value::Object(ref map) =>
                  self.parse_generator_from_map(category, map, Some(DocPath::new(sub_k)?)),
                _ => warn!("Ignoring invalid generator JSON '{}' -> {:?}", sub_k, sub_v)
              }
            }
          },
          Err(err) => warn!("Ignoring generator with invalid category '{}' - {}", k, err)
        },
        _ => warn!("Ignoring invalid generator JSON '{}' -> {:?}", k, v)
      }
    }
    Ok(())
  }

  pub(crate) fn parse_generator_from_map(
    &mut self,
    category: &GeneratorCategory,
    map: &serde_json::Map<String, Value>,
    subcat: Option<DocPath>,
  ) {
    match map.get("type") {
      Some(gen_type) => match gen_type {
        &Value::String(ref gen_type) => match Generator::from_map(gen_type, map) {
          Some(generator) => match subcat {
            Some(s) => self.add_generator_with_subcategory(category, s, generator),
            None => self.add_generator(category, generator)
          },
          None => warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
        },
        _ => warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
      },
      None => warn!("Ignoring invalid generator JSON '{:?}' with no type attribute -> {:?}", category, map)
    }
  }

  fn to_json(&self) -> Value {
    let json_attr = self.categories.iter()
      .fold(serde_json::Map::new(), |mut map, (name, category)| {
      let cat: String = name.clone().into();
      match name {
        GeneratorCategory::PATH | GeneratorCategory::METHOD | GeneratorCategory::STATUS => {
          match category.get(&DocPath::empty()).or_else(|| category.get(&DocPath::root())) {
            Some(generator) => {
              let json = generator.to_json();
              if let Some(json) = json {
                map.insert(cat.clone(), json);
              }
            },
            None => ()
          }
        },
        _ => {
          let mut generators = serde_json::Map::new();
          for (key, val) in category {
            let json = val.to_json();
            if let Some(json) = json {
              generators.insert(String::from(key), json);
            }
          }
          map.insert(cat.clone(), Value::Object(generators));
        }
      }
      map
    });
    Value::Object(json_attr)
  }

  /// Adds the generator to the category (body, headers, etc.)
  pub fn add_generator(&mut self, category: &GeneratorCategory, generator: Generator) {
    self.add_generator_with_subcategory(category, DocPath::empty(), generator);
  }

  /// Adds a generator to the category with a sub-category key (i.e. headers or query parameters)
  pub fn add_generator_with_subcategory(
    &mut self,
    category: &GeneratorCategory,
    subcategory: DocPath,
    generator: Generator,
  ) {
    let category_map = self.categories.entry(category.clone()).or_insert(HashMap::new());
    category_map.insert(subcategory, generator.clone());
  }

  /// Add all the generators
  pub fn add_generators(&mut self, generators: Generators) {
    for (key, values) in &generators.categories {
      let category_map = self.categories.entry(key.clone()).or_insert(HashMap::new());
      for (path, gen) in values {
        category_map.insert(path.clone(), gen.clone());
      }
    }
  }
}

impl Hash for Generators {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (k, v) in self.categories.iter() {
      k.hash(state);
      for (k2, v2) in v.iter() {
        k2.hash(state);
        v2.hash(state);
      }
    }
  }
}

impl PartialEq for Generators {
  fn eq(&self, other: &Self) -> bool {
    self.categories == other.categories
  }

  fn ne(&self, other: &Self) -> bool {
    self.categories != other.categories
  }
}

impl Default for Generators {
  fn default() -> Self {
    Generators {
      categories: hashmap!{}
    }
  }
}

/// If the mode applies, invoke the callback for each of the generators
pub fn apply_generators<F>(
  mode: &GeneratorTestMode,
  generators: &HashMap<DocPath, Generator>,
  closure: &mut F
) where F: FnMut(&DocPath, &Generator) {
  for (key, value) in generators {
    if value.corresponds_to_mode(mode) {
      closure(&key, &value)
    }
  }
}

/// Parses the generators from the Value structure
pub fn generators_from_json(value: &Value) -> anyhow::Result<Generators> {
  let mut generators = Generators::default();
  match value {
    &Value::Object(ref m) => match m.get("generators") {
      Some(gen_val) => match gen_val {
        &Value::Object(ref m) => generators.load_from_map(m)?,
        _ => ()
      },
      None => ()
    },
    _ => ()
  }
  Ok(generators)
}

/// Generates a Value structure for the provided generators
pub fn generators_to_json(generators: &Generators, spec_version: &PactSpecification) -> Value {
  match spec_version {
    PactSpecification::V3 | PactSpecification::V4 => generators.to_json(),
    _ => Value::Null
  }
}

/// Macro to make constructing generators easy
/// Example usage:
/// ```ignore
/// generators! {
///   "HEADER" => {
///     "A" => Generator::Uuid
///   }
/// }
///```
#[macro_export]
macro_rules! generators {
  (
    $( $category:expr => {
      $( $subname:expr => $generator:expr ), *
    }), *
  ) => {{
    let mut _generators = $crate::generators::Generators::default();

  $(
    {
      use std::str::FromStr;
      let _cat = $crate::generators::GeneratorCategory::from_str($category).unwrap();
    $(
      _generators.add_generator_with_subcategory(
        &_cat,
        $crate::path_exp::DocPath::new_unwrap($subname),
        $generator,
      );
    )*
    }
  )*

    _generators
  }};

  (
    $( $category:expr => $generator:expr ), *
  ) => {{
    let mut _generators = $crate::generators::Generators::default();
    $(
      let _cat = $crate::generators::GeneratorCategory::from_str($category).unwrap();
      _generators.add_generator(&_cat, $generator);
    )*
    _generators
  }};
}

pub fn generate_value_from_context(expression: &str, context: &HashMap<&str, Value>, data_type: &Option<DataType>) -> anyhow::Result<DataValue> {
  let result = if contains_expressions(expression) {
    parse_expression(expression, &MapValueResolver { context: context.clone() })
  } else {
    context.get(expression).map(|val| val.clone())
      .ok_or(anyhow!("Value '{}' was not found in the provided context", expression))
  };
  data_type.clone().unwrap_or(DataType::RAW).wrap(result)
}

const DIGIT_CHARSET: &str = "0123456789";
pub fn generate_decimal(digits: usize) -> String {
  let mut rnd = rand::thread_rng();
  let chars: Vec<char> = DIGIT_CHARSET.chars().collect();
  match digits {
    0 => "".to_string(),
    1 => chars.choose(&mut rnd).unwrap().to_string(),
    2 => format!("{}.{}", chars.choose(&mut rnd).unwrap(), chars.choose(&mut rnd).unwrap()),
    _ => {
      let mut sample = String::new();
      for _ in 0..(digits + 1) {
        sample.push(*chars.choose(&mut rnd).unwrap());
      }
      if sample.starts_with("00") {
        let chars = DIGIT_CHARSET[1..].chars();
        sample.insert(0, chars.choose(&mut rnd).unwrap());
      }
      let pos = rnd.gen_range(1..digits - 1);
      let selected_digits = if pos != 1 && sample.starts_with('0') {
        &sample[1..(digits + 1)]
      } else {
        &sample[..digits]
      };
      let generated = format!("{}.{}", &selected_digits[..pos], &selected_digits[pos..]);
      trace!("RandomDecimalGenerator: sample_digits=[{}], pos={}, selected_digits=[{}], generated=[{}]",
             sample, pos, selected_digits, generated);
      generated
    }
  }
}

const HEX_CHARSET: &str = "0123456789ABCDEF";
pub fn generate_hexadecimal(digits: usize) -> String {
  let mut rnd = rand::thread_rng();
  HEX_CHARSET.chars().choose_multiple(&mut rnd, digits).iter().join("")
}

impl GenerateValue<u16> for Generator {
  fn generate_value(
    &self,
    value: &u16,
    context: &HashMap<&str, Value>,
    _matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> anyhow::Result<u16> {
    match self {
      &Generator::RandomInt(min, max) => Ok(rand::thread_rng().gen_range(min as u16..(max as u16).saturating_add(1))),
      &Generator::ProviderStateGenerator(ref exp, ref dt) =>
        match generate_value_from_context(exp, context, dt) {
          Ok(val) => u16::try_from(val),
          Err(err) => Err(err)
        },
      _ => Err(anyhow!("Could not generate a u16 value from {} using {:?}", value, self))
    }
  }
}

pub fn generate_ascii_string(size: usize) -> String {
  rand::thread_rng().sample_iter(&Alphanumeric).map(char::from).take(size).collect()
}

fn strip_anchors(regex: &str) -> &str {
  regex
    .strip_prefix('^').unwrap_or(regex)
    .strip_suffix('$').unwrap_or(regex)
}

#[cfg(not(target_family = "wasm"))]
fn replace_with_regex(example: &String, url: String, re: Regex) -> String {
  re.replace(example, |caps: &Captures| {
    format!("{}{}", url, caps.at(1).unwrap())
  }).to_string()
}

#[cfg(target_family = "wasm")]
fn replace_with_regex(example: &String, url: String, re: Regex) -> String {
  re.replace(example, |caps: &Captures| {
    let m = caps.get(1).unwrap();
    format!("{}{}", url, m.as_str())
  }).to_string()
}

impl GenerateValue<String> for Generator {
  fn generate_value(
    &self,
    _: &String,
    context: &HashMap<&str, Value>,
    _matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> anyhow::Result<String> {
    let mut rnd = rand::thread_rng();
    let result = match self {
      Generator::RandomInt(min, max) => Ok(format!("{}", rnd.gen_range(*min..max.saturating_add(1)))),
      Generator::Uuid(format) => match format.unwrap_or_default() {
        UuidFormat::Simple => Ok(Uuid::new_v4().as_simple().to_string()),
        UuidFormat::LowerCaseHyphenated => Ok(Uuid::new_v4().as_hyphenated().to_string()),
        UuidFormat::UpperCaseHyphenated => Ok(Uuid::new_v4().as_hyphenated().to_string().to_uppercase()),
        UuidFormat::Urn => Ok(Uuid::new_v4().as_urn().to_string())
      },
      Generator::RandomDecimal(digits) => Ok(generate_decimal(*digits as usize)),
      Generator::RandomHexadecimal(digits) => Ok(generate_hexadecimal(*digits as usize)),
      Generator::RandomString(size) => Ok(generate_ascii_string(*size as usize)),
      Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(strip_anchors(regex)) {
          Ok(hir) => {
            match rand_regex::Regex::with_hir(hir, 20) {
              Ok(gen) => Ok(rnd.sample(gen)),
              Err(err) => {
                warn!("Failed to generate a value from regular expression - {}", err);
                Err(anyhow!("Failed to generate a value from regular expression - {}", err))
              }
            }
          },
          Err(err) => {
            warn!("'{}' is not a valid regular expression - {}", regex, err);
            Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
          }
        }
      },
      Generator::Date(ref format, exp) => {
        let base = match context.get("baseDate") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let date = execute_date_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => {
              #[allow(deprecated)]
              Ok(date.date().format(&to_chrono_pattern(&tokens)).to_string())
            },
            Err(err) => {
              warn!("Date format {} is not valid - {}", pattern, err);
              Err(anyhow!("Date format {} is not valid - {}", pattern, err))
            }
          },
          None => Ok(date.naive_local().date().to_string())
        }
      },
      Generator::Time(ref format, exp) => {
        let base = match context.get("baseTime") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let time = execute_time_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => Ok(time.format(&to_chrono_pattern(&tokens)).to_string()),
            Err(err) => {
              warn!("Time format {} is not valid - {}", pattern, err);
              Err(anyhow!("Time format {} is not valid - {}", pattern, err))
            }
          },
          None => Ok(time.time().format("%H:%M:%S").to_string())
        }
      },
      Generator::DateTime(ref format, exp) => {
        let base = match context.get("baseDateTime") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let date_time = execute_datetime_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => Ok(date_time.format(&to_chrono_pattern(&tokens)).to_string()),
            Err(err) => {
              warn!("DateTime format {} is not valid - {}", pattern, err);
              Err(anyhow!("DateTime format {} is not valid - {}", pattern, err))
            }
          },
          None => Ok(date_time.format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string())
        }
      },
      Generator::RandomBoolean => Ok(format!("{}", rnd.gen::<bool>())),
      Generator::ProviderStateGenerator(ref exp, ref dt) =>
        generate_value_from_context(exp, context, dt).map(|val| val.to_string()),
      Generator::MockServerURL(example, regex) => if let Some(mock_server_details) = context.get("mockServer") {
        debug!("Generating URL from Mock Server details");
        match mock_server_details.as_object() {
          Some(mock_server_details) => {
            match get_field_as_string("url", mock_server_details) {
              Some(url) => match Regex::new(regex) {
                Ok(re) => Ok(replace_with_regex(example, url, re)),
                Err(err) => Err(anyhow!("MockServerURL: Failed to generate value: {}", err))
              },
              None => Err(anyhow!("MockServerURL: can not generate a value as there is no mock server URL in the test context"))
            }
          },
          None => Err(anyhow!("MockServerURL: can not generate a value as there is no mock server details in the test context"))
        }
      } else {
        Err(anyhow!("MockServerURL: can not generate a value as there is no mock server details in the test context"))
      },
      Generator::ArrayContains(_) => Err(anyhow!("can only use ArrayContains with lists"))
    };
    debug!("Generator = {:?}, Generated value = {:?}", self, result);
    result
  }
}

impl GenerateValue<Vec<String>> for Generator {
  fn generate_value(
    &self,
    vals: &Vec<String>,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> anyhow::Result<Vec<String>> {
    self.generate_value(&vals.first().cloned().unwrap_or_default(), context, matcher).map(|v| vec![v])
  }
}

impl GenerateValue<Value> for Generator {
  fn generate_value(
    &self, value:
    &Value,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> anyhow::Result<Value> {
    debug!("Generating value from {:?} with context {:?}", self, context);
    let result = match self {
      Generator::RandomInt(min, max) => {
        let rand_int = rand::thread_rng().gen_range(*min..max.saturating_add(1));
        match value {
          Value::String(_) => Ok(json!(format!("{}", rand_int))),
          Value::Number(_) => Ok(json!(rand_int)),
          _ => Err(anyhow!("Could not generate a random int from {}", value))
        }
      },
      Generator::Uuid(format) => match value {
        Value::String(_) => match format.unwrap_or_default() {
          UuidFormat::Simple => Ok(json!(Uuid::new_v4().as_simple().to_string())),
          UuidFormat::LowerCaseHyphenated => Ok(json!(Uuid::new_v4().as_hyphenated().to_string())),
          UuidFormat::UpperCaseHyphenated => Ok(json!(Uuid::new_v4().as_hyphenated().to_string().to_uppercase())),
          UuidFormat::Urn => Ok(json!(Uuid::new_v4().as_urn().to_string()))
        },
        _ => Err(anyhow!("Could not generate a UUID from {}", value))
      },
      Generator::RandomDecimal(digits) => match value {
        Value::String(_) => Ok(json!(generate_decimal(*digits as usize))),
        Value::Number(_) => match generate_decimal(*digits as usize).parse::<f64>() {
          Ok(val) => Ok(json!(val)),
          Err(err) => Err(anyhow!("Could not generate a random decimal from {} - {}", value, err))
        },
        _ => Err(anyhow!("Could not generate a random decimal from {}", value))
      },
      Generator::RandomHexadecimal(digits) => match value {
        Value::String(_) => Ok(json!(generate_hexadecimal(*digits as usize))),
        _ => Err(anyhow!("Could not generate a random hexadecimal from {}", value))
      },
      Generator::RandomString(size) => match value {
        Value::String(_) => Ok(json!(generate_ascii_string(*size as usize))),
        _ => Err(anyhow!("Could not generate a random string from {}", value))
      },
      Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(regex) {
          Ok(hir) => {
            let gen = rand_regex::Regex::with_hir(hir, 20).unwrap();
            Ok(json!(rand::thread_rng().sample::<String, _>(gen)))
          },
          Err(err) => {
            warn!("'{}' is not a valid regular expression - {}", regex, err);
            Err(anyhow!("Could not generate a random string from {} - {}", regex, err))
          }
        }
      },
      Generator::Date(ref format, exp) => {
        let base = match context.get("baseDate") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let date = execute_date_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => {
              #[allow(deprecated)]
              Ok(json!(date.date().format(&to_chrono_pattern(&tokens)).to_string()))
            },
            Err(err) => {
              warn!("Date format {} is not valid - {}", pattern, err);
              Err(anyhow!("Could not generate a random date from {} - {}", pattern, err))
            }
          },
          None => Ok(json!(date.naive_local().date().to_string()))
        }
      },
      Generator::Time(ref format, exp) => {
        let base = match context.get("baseTime") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let time = execute_time_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => Ok(json!(time.format(&to_chrono_pattern(&tokens)).to_string())),
            Err(err) => {
              warn!("Time format {} is not valid - {}", pattern, err);
              Err(anyhow!("Could not generate a random time from {} - {}", pattern, err))
            }
          },
          None => Ok(json!(time.time().format("%H:%M:%S").to_string()))
        }
      },
      Generator::DateTime(ref format, exp) => {
        let base = match context.get("baseDateTime") {
          None => Local::now(),
          Some(d) => json_to_string(d).parse::<DateTime<Local>>()?
        };
        let date_time = execute_datetime_expression(&base, exp.clone().unwrap_or_default().as_str())?;
        match format {
          Some(pattern) => match parse_pattern(pattern) {
            Ok(tokens) => Ok(json!(date_time.format(&to_chrono_pattern(&tokens)).to_string())),
            Err(err) => {
              warn!("DateTime format {} is not valid - {}", pattern, err);
              Err(anyhow!("Could not generate a random date-time from {} - {}", pattern, err))
            }
          },
          None => Ok(json!(date_time.format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string()))
        }
      },
      Generator::RandomBoolean => Ok(json!(rand::thread_rng().gen::<bool>())),
      Generator::ProviderStateGenerator(ref exp, ref dt) =>
        match generate_value_from_context(exp, context, dt) {
          Ok(val) => val.as_json(),
          Err(err) => Err(err)
        },
      Generator::MockServerURL(example, regex) => {
        debug!("context = {:?}", context);
        if let Some(mock_server_details) = context.get("mockServer") {
          match mock_server_details.as_object() {
            Some(mock_server_details) => {
              match get_field_as_string("href", mock_server_details) {
                Some(url) => match Regex::new(regex) {
                  Ok(re) => Ok(Value::String(replace_with_regex(example, url, re))),
                  Err(err) => Err(anyhow!("MockServerURL: Failed to generate value: {}", err))
                },
                None => Err(anyhow!("MockServerURL: can not generate a value as there is no mock server URL in the test context"))
              }
            },
            None => Err(anyhow!("MockServerURL: can not generate a value as the mock server details in the test context is not an Object"))
          }
        } else {
          Err(anyhow!("MockServerURL: can not generate a value as there is no mock server details in the test context"))
        }
      }
      Generator::ArrayContains(variants) => match value {
        Value::Array(vec) => {
          let mut result = vec.clone();
          for (index, value) in vec.iter().enumerate() {
            if let Some((variant, generators)) = matcher.find_matching_variant(value, variants) {
              debug!("Generating values for variant {} and value {}", variant, value);
              let mut handler = JsonHandler { value: value.clone() };
              for (key, generator) in generators {
                handler.apply_key(&key, &generator, context, matcher);
              };
              debug!("Generated value {}", handler.value);
              result[index] = handler.value.clone();
            }
          }
          Ok(Value::Array(result))
        }
        _ => Err(anyhow!("can only use ArrayContains with lists"))
      }
    };
    debug!("Generated value = {:?}", result);
    result
  }
}

/// Implementation of a content type handler for JSON
pub struct JsonHandler {
  /// JSON document to apply the generators to.
  pub value: Value
}

impl JsonHandler {
  fn query_object_graph(&self, path_exp: &Vec<PathToken>, tree: &mut Arena<String>, root: NodeId, body: Value) {
    let mut body_cursor = body;
    let mut it = path_exp.iter();
    let mut node_cursor = root;
    loop {
      match it.next() {
        Some(token) => {
          match token {
            &PathToken::Field(ref name) => {
              match body_cursor.clone().as_object() {
                Some(map) => match map.get(name) {
                  Some(val) => {
                    let node = tree.new_node(name.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    node_cursor = node;
                  },
                  None => return
                },
                None => return
              }
            },
            &PathToken::Index(index) => {
              match body_cursor.clone().as_array() {
                Some(list) => if list.len() > index {
                  let node = tree.new_node(format!("{}", index));
                  node_cursor.append(node, tree);
                  body_cursor = list[index].clone();
                  node_cursor = node;
                },
                None => return
              }
            }
            &PathToken::Star => {
              match body_cursor.clone().as_object() {
                Some(map) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (key, val) in map {
                    let node = tree.new_node(key.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node, val.clone());
                  }
                },
                None => return
              }
            },
            &PathToken::StarIndex => {
              match body_cursor.clone().as_array() {
                Some(list) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (index, val) in list.iter().enumerate() {
                    let node = tree.new_node(format!("{}", index));
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node,val.clone());
                  }
                },
                None => return
              }
            },
            _ => ()
          }
        },
        None => break
      }
    }
  }
}

impl ContentTypeHandler<Value> for JsonHandler {
  fn process_body(
    &mut self,
    generators: &HashMap<DocPath, Generator>,
    mode: &GeneratorTestMode,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> Result<OptionalBody, String> {
    for (key, generator) in generators {
      if generator.corresponds_to_mode(mode) {
        debug!("Applying generator {:?} to key {}", generator, key);
        self.apply_key(key, generator, context, matcher);
      }
    };
    Ok(OptionalBody::Present(self.value.to_string().into(), Some("application/json".into()), None))
  }

  fn apply_key(
    &mut self,
    key: &DocPath,
    generator: &dyn GenerateValue<Value>,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>,
  ) {
    let path_exp = key;
    let mut tree = Arena::new();
    let root = tree.new_node("".into());
    self.query_object_graph(path_exp.tokens(), &mut tree, root, self.value.clone());
    let expanded_paths = root.descendants(&tree).fold(Vec::<String>::new(), |mut acc, node_id| {
      let node = tree.index(node_id);
      if !node.get().is_empty() && node.first_child().is_none() {
        let path: Vec<String> = node_id.ancestors(&tree).map(|n| format!("{}", tree.index(n).get())).collect();
        if path.len() == path_exp.len() {
          acc.push(path.iter().rev().join("/"));
        }
      }
      acc
    });

    if !expanded_paths.is_empty() {
      for pointer_str in expanded_paths {
        match self.value.pointer_mut(&pointer_str) {
          Some(json_value) => match generator.generate_value(&json_value.clone(), context, matcher) {
            Ok(new_value) => *json_value = new_value,
            Err(_) => ()
          },
          None => ()
        }
      }
    } else if path_exp.len() == 1 {
      match generator.generate_value(&self.value.clone(), context, matcher) {
        Ok(new_value) => self.value = new_value,
        Err(_) => ()
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::ops::Add;
  use std::str::FromStr;

  use chrono::Duration;
  use expectest::expect;
  use expectest::prelude::*;
  use hamcrest2::*;
  use test_log::test;

  use crate::generators::Generator::{RandomDecimal, RandomInt, Regex};

  use super::*;
  use super::Generator;

  #[test]
  fn rules_are_empty_when_there_are_no_categories() {
    expect!(Generators::default().is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_empty_when_there_are_only_empty_categories() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap!{}
            }
        }.is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_not_empty_when_there_is_a_nonempty_category() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap! {
                    DocPath::new_unwrap("a") => Generator::RandomInt(1, 10)
                }
            }
        }.is_empty()).to(be_false());
  }

  #[test]
  fn matchers_from_json_test() {
    let generators = generators_from_json(&Value::Null);
    expect!(generators.unwrap().categories.iter()).to(be_empty());
  }

  #[test]
  fn generators_macro_test() {
    expect!(generators!{}).to(be_equal_to(Generators::default()));

    let mut expected = Generators::default();
    expected.add_generator(&GeneratorCategory::STATUS, Generator::RandomInt(400, 499));
    expect!(generators!{
      "STATUS" => Generator::RandomInt(400, 499)
    }).to(be_equal_to(expected));

    expected = Generators::default();
    expected.add_generator_with_subcategory(
      &GeneratorCategory::BODY,
      DocPath::new_unwrap("$.a.b"),
      Generator::RandomInt(1, 10),
    );
    expect!(generators!{
      "BODY" => {
        "$.a.b" => Generator::RandomInt(1, 10)
      }
    }).to(be_equal_to(expected));
  }

  #[test]
  fn generator_from_json_test() {
    expect!(Generator::from_map("", &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map("Invalid", &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map("uuid", &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map("Uuid", &serde_json::Map::new())).to(be_some().value(Generator::Uuid(None)));
    expect!(Generator::from_map("Uuid", &json!({ "format": "simple"}).as_object().unwrap())).to(be_some().value(Generator::Uuid(Some(UuidFormat::Simple))));
    expect!(Generator::from_map("Uuid", &json!({ "format": "other"}).as_object().unwrap())).to(be_some().value(Generator::Uuid(None)));
    expect!(Generator::from_map("RandomBoolean", &serde_json::Map::new())).to(be_some().value(Generator::RandomBoolean));
  }

  #[test]
  fn randomint_generator_from_json_test() {
    expect!(Generator::from_map("RandomInt", &serde_json::Map::new())).to(be_some().value(Generator::RandomInt(0, 10)));
    expect!(Generator::from_map("RandomInt", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 10)));
    expect!(Generator::from_map("RandomInt", &json!({ "max": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 5)));
    expect!(Generator::from_map("RandomInt", &json!({ "min": 5, "max": 6 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 6)));
    expect!(Generator::from_map("RandomInt", &json!({ "min": 0, "max": 1234567890 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 1234567890)));
  }

  #[test]
  fn random_decimal_generator_from_json_test() {
    expect!(Generator::from_map("RandomDecimal", &serde_json::Map::new())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map("RandomDecimal", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map("RandomDecimal", &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(5)));
  }

  #[test]
  fn random_hexadecimal_generator_from_json_test() {
    expect!(Generator::from_map("RandomHexadecimal", &serde_json::Map::new())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map("RandomHexadecimal", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map("RandomHexadecimal", &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(5)));
  }

  #[test]
  fn random_string_generator_from_json_test() {
    expect!(Generator::from_map("RandomString", &serde_json::Map::new())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map("RandomString", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map("RandomString", &json!({ "size": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(5)));
  }

  #[test]
  fn regex_generator_from_json_test() {
    expect!(Generator::from_map("Regex", &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map("Regex", &json!({ "min": 5 }).as_object().unwrap())).to(be_none());
    expect!(Generator::from_map("Regex", &json!({ "regex": "\\d+" }).as_object().unwrap())).to(be_some().value(Generator::Regex("\\d+".to_string())));
    expect!(Generator::from_map("Regex", &json!({ "regex": 5 }).as_object().unwrap())).to(be_some().value(Generator::Regex("5".to_string())));
  }

  #[test]
  fn date_generator_from_json_test() {
    expect!(Generator::from_map("Date", &serde_json::Map::new())).to(be_some().value(Generator::Date(None, None)));
    expect!(Generator::from_map("Date", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(None, None)));
    expect!(Generator::from_map("Date", &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Date(Some("yyyy-MM-dd".to_string()), None)));
    expect!(Generator::from_map("Date", &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(Some("5".to_string()), None)));
    expect!(Generator::from_map("Date", &json!({ "expression": "now" }).as_object().unwrap())).to(be_some().value(Generator::Date(None, Some("now".to_string()))));
  }

  #[test]
  fn time_generator_from_json_test() {
    expect!(Generator::from_map("Time", &serde_json::Map::new())).to(be_some().value(Generator::Time(None, None)));
    expect!(Generator::from_map("Time", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(None, None)));
    expect!(Generator::from_map("Time", &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Time(Some("yyyy-MM-dd".to_string()), None)));
    expect!(Generator::from_map("Time", &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(Some("5".to_string()), None)));
    expect!(Generator::from_map("Time", &json!({ "expression": "now" }).as_object().unwrap())).to(be_some().value(Generator::Time(None, Some("now".to_string()))));
  }

  #[test]
  fn datetime_generator_from_json_test() {
    expect!(Generator::from_map("DateTime", &serde_json::Map::new())).to(be_some().value(Generator::DateTime(None, None)));
    expect!(Generator::from_map("DateTime", &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(None, None)));
    expect!(Generator::from_map("DateTime", &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some("yyyy-MM-dd".to_string()), None)));
    expect!(Generator::from_map("DateTime", &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some("5".to_string()), None)));
    expect!(Generator::from_map("DateTime", &json!({ "expression": "now" }).as_object().unwrap())).to(be_some().value(Generator::DateTime(None, Some("now".to_string()))));
  }

  #[test]
  fn provider_state_generator_from_json_test() {
    expect!(Generator::from_map("ProviderState", &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map("ProviderState", &json!({ "expression": "5" }).as_object().unwrap())).to(
      be_some().value(Generator::ProviderStateGenerator("5".into(), None)));
    expect!(Generator::from_map("ProviderState", &json!({ "expression": "5", "dataType": "INTEGER" }).as_object().unwrap())).to(
      be_some().value(Generator::ProviderStateGenerator("5".into(), Some(DataType::INTEGER))));
  }

  #[test]
  fn generator_to_json_test() {
    expect!(Generator::RandomInt(5, 15).to_json().unwrap()).to(be_equal_to(json!({
      "type": "RandomInt",
      "min": 5,
      "max": 15
    })));
    expect!(Generator::Uuid(None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Uuid"
    })));
    expect!(Generator::Uuid(Some(UuidFormat::Simple)).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Uuid",
      "format": "simple"
    })));
    expect!(Generator::RandomDecimal(5).to_json().unwrap()).to(be_equal_to(json!({
      "type": "RandomDecimal",
      "digits": 5
    })));
    expect!(Generator::RandomHexadecimal(5).to_json().unwrap()).to(be_equal_to(json!({
      "type": "RandomHexadecimal",
      "digits": 5
    })));
    expect!(Generator::RandomString(5).to_json().unwrap()).to(be_equal_to(json!({
      "type": "RandomString",
      "size": 5
    })));
    expect!(Generator::Regex("\\d+".into()).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Regex",
      "regex": "\\d+"
    })));
    expect!(Generator::RandomBoolean.to_json().unwrap()).to(be_equal_to(json!({
      "type": "RandomBoolean"
    })));

    expect!(Generator::Date(Some("yyyyMMdd".into()), None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Date",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Date(Some("yyyyMMdd".into()), Some("now".into())).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Date",
      "format": "yyyyMMdd",
      "expression": "now"
    })));
    expect!(Generator::Date(None, None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Date"
    })));
    expect!(Generator::Time(Some("yyyyMMdd".into()), None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Time",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Time(Some("yyyyMMdd".into()), Some("now".into())).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Time",
      "format": "yyyyMMdd",
      "expression": "now"
    })));
    expect!(Generator::Time(None, None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "Time"
    })));
    expect!(Generator::DateTime(Some("yyyyMMdd".into()), None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "DateTime",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::DateTime(Some("yyyyMMdd".into()), Some("now".into())).to_json().unwrap()).to(be_equal_to(json!({
      "type": "DateTime",
      "format": "yyyyMMdd",
      "expression": "now"
    })));
    expect!(Generator::DateTime(None, None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "DateTime"
    })));
    expect!(Generator::ProviderStateGenerator("$a".into(), Some(DataType::INTEGER)).to_json().unwrap()).to(be_equal_to(json!({
      "type": "ProviderState",
      "expression": "$a",
      "dataType": "INTEGER"
    })));
    expect!(Generator::ProviderStateGenerator("$a".into(), None).to_json().unwrap()).to(be_equal_to(json!({
      "type": "ProviderState",
      "expression": "$a"
    })));
    expect!(Generator::MockServerURL("http://localhost:1234/path".into(), "(.*)/path".into()).to_json().unwrap()).to(be_equal_to(json!({
      "type": "MockServerURL",
      "example": "http://localhost:1234/path",
      "regex": "(.*)/path"
    })));
  }

  #[test]
  fn generators_to_json_test() {
    let mut generators = Generators::default();
    generators.add_generator(&GeneratorCategory::STATUS, RandomInt(200, 299));
    generators.add_generator(&GeneratorCategory::PATH, Regex("\\d+".into()));
    generators.add_generator(&GeneratorCategory::METHOD, RandomInt(200, 299));
    generators.add_generator_with_subcategory(&GeneratorCategory::BODY, DocPath::new_unwrap("$.1"), RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::BODY, DocPath::new_unwrap("$.2"), RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::HEADER, DocPath::new_unwrap("A"), RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::HEADER, DocPath::new_unwrap("B"), RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::QUERY, DocPath::new_unwrap("a"), RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::QUERY, DocPath::new_unwrap("b"), RandomDecimal(4));
    let json = generators.to_json();
    expect(json).to(be_equal_to(json!({
      "body": {
        "$.1": {"digits": 4, "type": "RandomDecimal"},
        "$.2": {"digits": 4, "type": "RandomDecimal"}
      },
      "header": {
        "A": {"digits": 4, "type": "RandomDecimal"},
        "B": {"digits": 4, "type": "RandomDecimal"}
      },
      "method": {"max": 299, "min": 200, "type": "RandomInt"},
      "path": {"regex": "\\d+", "type": "Regex"},
      "query": {
        "a": {"digits": 4, "type": "RandomDecimal"},
        "b": {"digits": 4, "type": "RandomDecimal"}
      },
      "status": {"max": 299, "min": 200, "type": "RandomInt"}
    })));
  }

  #[test]
  fn path_generator_with_root_path_to_json_test() {
    let mut generators = Generators::default();
    generators.add_generator_with_subcategory(&GeneratorCategory::PATH, DocPath::root(), RandomDecimal(1));
    let json = generators.to_json();
    expect(json).to(be_equal_to(json!({
      "path": {"digits": 1, "type": "RandomDecimal"}
    })));
  }

  #[test]
  fn generate_decimal_test() {
    assert_that!(generate_decimal(4), matches_regex(r"^\d{1,3}\.\d{1,3}$"));
    assert_that!(generate_hexadecimal(4), matches_regex(r"^[0-9A-F]{4}$"));
  }

  #[test]
  fn generate_int_with_max_int_test() {
    assert_that!(Generator::RandomInt(0, i32::max_value()).generate_value(&0,
      &hashmap!{}, &NoopVariantMatcher.boxed()).unwrap().to_string(), matches_regex(r"^\d+$"));
  }

  #[test]
  fn provider_state_generator_test() {
    expect!(Generator::ProviderStateGenerator("${a}".into(), Some(DataType::INTEGER)).generate_value(&0,
      &hashmap!{ "a".into() => json!(1234) }, &NoopVariantMatcher.boxed())).to(be_ok().value(1234));
  }

  #[test]
  fn date_generator_test() {
    let generated = Generator::Date(None, None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}$"));

    let generated2 = Generator::Date(Some("yyyy-MM-ddZ".into()), None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}[-+]\d{4}$"));

    let now = Local::now();
    let generated3 = Generator::Date(Some("yyyy-MM-dd".into()), Some("+1 day".into())).generate_value(
      &"".to_string(),
      &hashmap!{
        "baseDate" => json!(now.to_string())
      },
      &NoopVariantMatcher.boxed()
    );
    #[allow(deprecated)]
    expect!(generated3.unwrap()).to(be_equal_to(now.add(Duration::days(1)).date().format("%Y-%m-%d").to_string()));
  }

  #[test]
  fn time_generator_test() {
    let generated = Generator::Time(None, None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}$"));

    let generated2 = Generator::Time(Some("HH:mm:ssZ".into()), None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}[-+]\d+$"));

    let now = Local::now();
    let generated3 = Generator::Time(Some("HH:mm:ss".into()), Some("+1 hour".into())).generate_value(
      &"".to_string(),
      &hashmap!{
        "baseTime" => json!(now.to_string())
      },
      &NoopVariantMatcher.boxed()
    );
    expect!(generated3.unwrap()).to(be_equal_to(now.add(Duration::hours(1)).time().format("%H:%M:%S").to_string()));
  }

  #[test]
  fn datetime_generator_test() {
    let generated = Generator::DateTime(None, None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}[-+]\d+$"));

    let generated2 = Generator::DateTime(Some("yyyy-MM-dd HH:mm:ssZ".into()), None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[-+]\d+$"));

    let now = Local::now();
    let generated3 = Generator::DateTime(Some("yyyy-MM-dd HH:mm:ss".into()), Some("+1 day".into())).generate_value(
      &"".to_string(),
      &hashmap!{
        "baseDateTime" => json!(now.to_string())
      },
      &NoopVariantMatcher.boxed()
    );
    expect!(generated3.unwrap()).to(be_equal_to(now.add(Duration::days(1)).format("%Y-%m-%d %H:%M:%S").to_string()));
  }

  #[test]
  fn regex_generator_test() {
    let generated = Generator::Regex(r"\d{4}\w{1,4}".into()).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}\w{1,4}$"));

    let generated = Generator::Regex(r"\d{1,2}/\d{1,2}".into()).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{1,2}/\d{1,2}$"));

    let generated = Generator::Regex(r"^\d{1,2}/\d{1,2}$".into()).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^\d{1,2}/\d{1,2}$"));
  }

  #[test]
  fn uuid_generator_test() {
    let generated = Generator::Uuid(None).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$"));

    let generated = Generator::Uuid(Some(UuidFormat::Simple)).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^[a-f0-9]{32}$"));

    let generated = Generator::Uuid(Some(UuidFormat::LowerCaseHyphenated)).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$"));

    let generated = Generator::Uuid(Some(UuidFormat::UpperCaseHyphenated)).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^[A-F0-9]{8}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{12}$"));

    let generated = Generator::Uuid(Some(UuidFormat::Urn)).generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    assert_that!(generated.unwrap(), matches_regex(r"^urn:uuid:[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$"));
  }

  #[test]
  fn random_decimal_generator_test() {
    for _ in 1..10 {
      let generated = Generator::RandomDecimal(10).generate_value(&"".to_string(), &hashmap! {}, &NoopVariantMatcher.boxed()).unwrap();
      expect!(generated.clone().len()).to(be_equal_to(11));
      assert_that!(generated.clone(), matches_regex(r"^\d+\.\d+$"));
      let mut chars = generated.chars();
      let first_char = chars.next().unwrap();
      let second_char = chars.next().unwrap();
      println!("{}: '{}' != '0' || ('{}' == '0' && '{}' == '.')", generated, first_char, first_char, second_char);
      expect!(first_char != '0' || (first_char == '0' && second_char == '.')).to(be_true());
    }
  }

  #[test]
  fn handle_edge_case_when_digits_is_1() {
    let generated = Generator::RandomDecimal(1).generate_value(&"".to_string(), &hashmap! {}, &NoopVariantMatcher.boxed()).unwrap();
    assert_that!(generated, matches_regex(r"^\d$"));
  }

  #[test]
  fn handle_edge_case_when_digits_is_2() {
    let generated = Generator::RandomDecimal(2).generate_value(&"".to_string(), &hashmap! {}, &NoopVariantMatcher.boxed()).unwrap();
    assert_that!(generated, matches_regex(r"^\d\.\d$"));
  }

  #[test]
  fn mock_server_url_generator_test() {
    let generator = Generator::MockServerURL("http://localhost:1234/path".into(), ".*(/path)$".into());
    let generated = generator.generate_value(&"".to_string(), &hashmap! {
        "mockServer" => json!({
          "url": "http://192.168.2.1:2345/p",
          "port": 2345
        })
      }, &NoopVariantMatcher.boxed());
    expect!(generated.unwrap()).to(be_equal_to("http://192.168.2.1:2345/p/path"));
    let generated = generator.generate_value(&"".to_string(), &hashmap!{}, &NoopVariantMatcher.boxed());
    expect!(generated).to(be_err());
  }

  #[test]
  fn applies_the_generator_to_a_json_map_entry() {
    let map = json!({"a": 100, "b": "B", "c": "C"});
    let mut json_handler = JsonHandler { value: map };

    json_handler.apply_key(&DocPath::new_unwrap("$.b"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
  }

  #[test]
  fn does_not_apply_the_generator_when_field_is_not_in_map() {
    let map = json!({"a": 100, "b": "B", "c": "C"});
    let mut json_handler = JsonHandler { value: map };

    json_handler.apply_key(&DocPath::new_unwrap("$.d"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(json_handler.value).to(be_equal_to(json!({"a": 100, "b": "B", "c": "C"})));
  }

  #[test]
  fn does_not_apply_the_generator_when_not_a_map() {
    let map = json!(100);
    let mut json_handler = JsonHandler { value: map };

    json_handler.apply_key(&DocPath::new_unwrap("$.d"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(json_handler.value).to(be_equal_to(json!(100)));
  }

  #[test]
  fn applies_the_generator_to_a_list_item() {
    let list = json!([100, 200, 300]);
    let mut json_handler = JsonHandler { value: list };

    json_handler.apply_key(&DocPath::new_unwrap("$[1]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value[1]).to_not(be_equal_to(&json!(200)));
  }

  #[test]
  fn does_not_apply_the_generator_when_index_is_not_in_list() {
    let list = json!([100, 200, 300]);
    let mut json_handler = JsonHandler { value: list };

    json_handler.apply_key(&DocPath::new_unwrap("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(json_handler.value).to(be_equal_to(json!([100, 200, 300])));
  }

  #[test]
  fn does_not_apply_the_generator_when_not_a_list() {
    let list = json!(100);
    let mut json_handler = JsonHandler { value: list };

    json_handler.apply_key(&DocPath::new_unwrap("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(json_handler.value).to(be_equal_to(json!(100)));
  }

  #[test]
  fn applies_the_generator_to_the_root() {
    let value = json!(100);
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::root(), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value).to_not(be_equal_to(&json!(100)));
}

  #[test]
  fn applies_the_generator_to_the_object_graph() {
    let value = json!({
    "a": ["A", {"a": "A", "b": {"1": "1", "2": "2"}, "c": "C"}, "C"],
    "b": "B",
    "c": "C"
  });
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::new_unwrap("$.a[1].b['2']"), &Generator::RandomInt(3, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value["a"][1]["b"]["2"]).to_not(be_equal_to(&json!("2")));
  }

  #[test]
  fn does_not_apply_the_generator_to_the_object_graph_when_the_expression_does_not_match() {
    let value = json!({
    "a": "A",
    "b": "B",
    "c": "C"
  });
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::new_unwrap("$.a[1].b['2']"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value).to(be_equal_to(&json!({
    "a": "A",
    "b": "B",
    "c": "C"
  })));
  }

  #[test]
  fn applies_the_generator_to_all_map_entries() {
    let value = json!({
    "a": "A",
    "b": "B",
    "c": "C"
  });
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::new_unwrap("$.*"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value["a"]).to_not(be_equal_to(&json!("A")));
    expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
    expect!(&json_handler.value["c"]).to_not(be_equal_to(&json!("C")));
  }

  #[test]
  fn applies_the_generator_to_all_list_items() {
    let value = json!(["A", "B", "C"]);
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::new_unwrap("$[*]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value[0]).to_not(be_equal_to(&json!("A")));
    expect!(&json_handler.value[1]).to_not(be_equal_to(&json!("B")));
    expect!(&json_handler.value[2]).to_not(be_equal_to(&json!("C")));
  }

  #[test]
  fn applies_the_generator_to_the_object_graph_with_wildcard() {
    let value = json!({
    "a": ["A", {"a": "A", "b": ["1", "2"], "c": "C"}, "C"],
    "b": "B",
    "c": "C"
  });
    let mut json_handler = JsonHandler { value };

    json_handler.apply_key(&DocPath::new_unwrap("$.*[1].b[*]"), &Generator::RandomInt(3, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&json_handler.value["a"][0]).to(be_equal_to(&json!("A")));
    expect!(&json_handler.value["a"][1]["a"]).to(be_equal_to(&json!("A")));
    expect!(&json_handler.value["a"][1]["b"][0]).to_not(be_equal_to(&json!("1")));
    expect!(&json_handler.value["a"][1]["b"][1]).to_not(be_equal_to(&json!("2")));
    expect!(&json_handler.value["a"][1]["c"]).to(be_equal_to(&json!("C")));
    expect!(&json_handler.value["a"][2]).to(be_equal_to(&json!("C")));
    expect!(&json_handler.value["b"]).to(be_equal_to(&json!("B")));
    expect!(&json_handler.value["c"]).to(be_equal_to(&json!("C")));
  }

  // Issue https://github.com/pact-foundation/pact-js-core/issues/400
  #[test]
  fn to_json_with_provider_state_generator_test() {
    let generators = Generators {
      categories: hashmap!{
        GeneratorCategory::PATH => hashmap!{
          DocPath::root() => Generator::ProviderStateGenerator("/data/${id}".to_string(), None)
        }
      }
    };

    let json = generators_to_json(&generators, &PactSpecification::V3);
    expect!(json.to_string()).to_not(be_equal_to("{}"));
  }
}

#[cfg(test)]
mod tests2 {
  use expectest::prelude::*;
  use maplit::hashmap;
  use rstest::rstest;
  use serde_json::{json, Value};

  use crate::expression_parser::DataType;
  use crate::generators::generate_value_from_context;

  #[rstest]
  //     expression, value,          data_type,               expected
  #[case("${a}",     json!("value"), Some(DataType::STRING),  json!("value"))]
  #[case("${a}",     json!("value"), Some(DataType::RAW),     json!("value"))]
  #[case("${a}",     json!("value"), None,                    json!("value"))]
  #[case("${a}",     json!(100),     Some(DataType::STRING),  json!("100"))]
  #[case("${a}",     json!(100),     Some(DataType::RAW),     json!(100))]
  #[case("${a}",     json!(100),     Some(DataType::INTEGER), json!(100))]
  #[case("${a}",     json!(100),     None,                    json!(100))]
  #[case("/${a}",    json!("value"), Some(DataType::STRING),  json!("/value"))]
  #[case("/${a}",    json!("value"), Some(DataType::RAW),     json!("/value"))]
  #[case("/${a}",    json!("value"), None,                    json!("/value"))]
  #[case("/${a}",    json!(100),     Some(DataType::STRING),  json!("/100"))]
  #[case("/${a}",    json!(100),     Some(DataType::RAW),     json!("/100"))]
  #[case("/${a}",    json!(100),     None,                    json!("/100"))]
  #[case("a",        json!("value"), Some(DataType::STRING),  json!("value"))]
  #[case("a",        json!("value"), Some(DataType::RAW),     json!("value"))]
  #[case("a",        json!("value"), None,                    json!("value"))]
  #[case("a",        json!(100),     Some(DataType::STRING),  json!("100"))]
  #[case("a",        json!(100),     Some(DataType::RAW),     json!(100))]
  #[case("a",        json!(100),     Some(DataType::INTEGER), json!(100))]
  #[case("a",        json!(100),     None,                    json!(100))]
  fn generate_value_from_context_test(#[case] expression: &str, #[case] value: Value, #[case] data_type: Option<DataType>, #[case] expected: Value) {
    let context = hashmap!{ "a" => value };
    let result = generate_value_from_context(expression, &context, &data_type);
    expect!(result.as_ref()).to(be_ok());
    let result_value = result.unwrap();
    expect!(result_value.as_json().unwrap()).to(be_equal_to(expected));
  }
}
