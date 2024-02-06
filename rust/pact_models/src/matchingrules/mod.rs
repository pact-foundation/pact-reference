//! `matchingrules` module includes all the classes to deal with V3/V4 spec matchers

use std::{fmt, mem};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use anyhow::{anyhow, Context as _};
use itertools::{Either, Itertools};
use maplit::hashmap;
use serde_json::{json, Map, Value};
use tracing::{error, trace};

use crate::{HttpStatus, PactSpecification};
use crate::generators::{Generator, GeneratorCategory, Generators};
use crate::json_utils::{json_to_num, json_to_string};
use crate::matchingrules::expressions::{MatchingReference, MatchingRuleDefinition, ValueType};
use crate::path_exp::{DocPath, PathToken};

pub mod expressions;

fn generator_from_json(json: &Map<String, Value>) -> Option<Generator> {
  if let Some(generator_json) = json.get("generator") {
    match generator_json {
      Value::Object(attributes) => if let Some(generator_type) = attributes.get("type") {
        match generator_type {
          Value::String(generator_type) => Generator::from_map(generator_type.as_str(), attributes),
          _ => None
        }
      } else {
        None
      }
      _ => None
    }
  } else {
    None
  }
}

fn rules_from_json(attributes: &Map<String, Value>) -> anyhow::Result<Vec<Either<MatchingRule, MatchingReference>>> {
  match attributes.get("rules") {
    Some(rules) => match rules {
      Value::Array(rules) => {
        let rules = rules.iter()
          .map(|rule| MatchingRule::from_json(rule));
        if let Some(err) = rules.clone().find(|rule| rule.is_err()) {
          Err(anyhow!("Matching rule configuration is not correct - {}", err.unwrap_err()))
        } else {
          Ok(rules.map(|rule| Either::Left(rule.unwrap())).collect())
        }
      }
      _ => Err(anyhow!("EachKey matcher config is not valid. Was expected an array but got {}", rules))
    }
    None => Ok(vec![])
  }
}

/// Set of all matching rules
#[derive(Debug, Clone, Eq)]
pub enum MatchingRule {
  /// Matcher using equals
  Equality,
  /// Match using a regular expression
  Regex(String),
  /// Match using the type of the value
  Type,
  /// Match using the type of the value and a minimum length for collections
  MinType(usize),
  /// Match using the type of the value and a maximum length for collections
  MaxType(usize),
  /// Match using the type of the value and a minimum and maximum length for collections
  MinMaxType(usize, usize),
  /// Match the value using a timestamp pattern
  Timestamp(String),
  /// Match the value using a time pattern
  Time(String),
  /// Match the value using a date pattern
  Date(String),
  /// Match if the value includes the given value
  Include(String),
  /// Match if the value is a number
  Number,
  /// Match if the value is an integer number
  Integer,
  /// Match if the value is a decimal number
  Decimal,
  /// Match if the value is a null value (this is content specific, for JSON will match a JSON null)
  Null,
  /// Match binary data by its content type (magic file check)
  ContentType(String),
  /// Match array items in any order against a list of variants
  ArrayContains(Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>),
  /// Matcher for values in a map, ignoring the keys
  Values,
  /// Matches boolean values (booleans and the string values `true` and `false`)
  Boolean,
  /// Request status code matcher
  StatusCode(HttpStatus),
  /// Value must be the same type and not empty
  NotEmpty,
  /// Value must a semantic version
  Semver,
  /// Matcher for keys in a map
  EachKey(MatchingRuleDefinition),
  /// Matcher for values in a collection. This delegates to the Values matcher for maps.
  EachValue(MatchingRuleDefinition)
}

impl MatchingRule {

  /// Builds a `MatchingRule` from a `Value` struct
  pub fn from_json(value: &Value) -> anyhow::Result<MatchingRule> {
    match value {
      Value::Object(m) => match m.get("match").or_else(|| m.get("pact:matcher:type")) {
        Some(match_val) => {
          let val = json_to_string(match_val);
          MatchingRule::create(val.as_str(), value)
        }
        None => if let Some(val) = m.get("regex") {
          Ok(MatchingRule::Regex(json_to_string(val)))
        } else if let Some(val) = json_to_num(m.get("min").cloned()) {
          Ok(MatchingRule::MinType(val))
        } else if let Some(val) = json_to_num(m.get("max").cloned()) {
          Ok(MatchingRule::MaxType(val))
        } else if let Some(val) = m.get("timestamp") {
          Ok(MatchingRule::Timestamp(json_to_string(val)))
        } else if let Some(val) = m.get("time") {
          Ok(MatchingRule::Time(json_to_string(val)))
        } else if let Some(val) = m.get("date") {
          Ok(MatchingRule::Date(json_to_string(val)))
        } else {
          Err(anyhow!("Matching rule missing 'match' field and unable to guess its type"))
        }
      },
      _ => Err(anyhow!("Matching rule JSON is not an Object")),
    }
  }

  /// Converts this `MatchingRule` to a `Value` struct
  pub fn to_json(&self) -> Value {
    match self {
      MatchingRule::Equality => json!({ "match": "equality" }),
      MatchingRule::Regex(ref r) => json!({ "match": "regex",
        "regex": r.clone() }),
      MatchingRule::Type => json!({ "match": "type" }),
      MatchingRule::MinType(min) => json!({ "match": "type",
        "min": json!(*min as u64) }),
      MatchingRule::MaxType(max) => json!({ "match": "type",
        "max": json!(*max as u64) }),
      MatchingRule::MinMaxType(min, max) => json!({ "match": "type",
        "min": json!(*min as u64), "max": json!(*max as u64) }),
      MatchingRule::Timestamp(ref t) => json!({ "match": "datetime",
        "format": Value::String(t.clone()) }),
      MatchingRule::Time(ref t) => json!({ "match": "time",
        "format": Value::String(t.clone()) }),
      MatchingRule::Date(ref d) => json!({ "match": "date",
        "format": Value::String(d.clone()) }),
      MatchingRule::Include(ref s) => json!({ "match": "include",
        "value": Value::String(s.clone()) }),
      MatchingRule::Number => json!({ "match": "number" }),
      MatchingRule::Integer => json!({ "match": "integer" }),
      MatchingRule::Decimal => json!({ "match": "decimal" }),
      MatchingRule::Boolean => json!({ "match": "boolean" }),
      MatchingRule::Null => json!({ "match": "null" }),
      MatchingRule::ContentType(ref r) => json!({ "match": "contentType",
        "value": Value::String(r.clone()) }),
      MatchingRule::ArrayContains(variants) => json!({
        "match": "arrayContains",
        "variants": variants.iter().map(|(index, rules, generators)| {
          let mut json = json!({
            "index": index,
            "rules": rules.to_v3_json()
          });
          if !generators.is_empty() {
            json["generators"] = Value::Object(generators.iter()
              .map(|(k, gen)| {
                if let Some(json) = gen.to_json() {
                  Some((String::from(k), json))
                } else {
                  None
                }
              })
              .filter(|item| item.is_some())
              .map(|item| item.unwrap())
              .collect())
          }
          json
        }).collect::<Vec<Value>>()
      }),
      MatchingRule::Values => json!({ "match": "values" }),
      MatchingRule::StatusCode(status) => json!({ "match": "statusCode", "status": status.to_json() }),
      MatchingRule::NotEmpty => json!({ "match": "notEmpty" }),
      MatchingRule::Semver => json!({ "match": "semver" }),
      MatchingRule::EachKey(definition) => {
        let mut json = json!({
          "match": "eachKey",
          "rules": definition.rules.iter()
            .map(|rule| rule.as_ref().expect_left("Expected a matching rule, found an unresolved reference").to_json())
          .collect::<Vec<Value>>()
        });
        let map = json.as_object_mut().unwrap();

        if !definition.value.is_empty() {
          map.insert("value".to_string(), Value::String(definition.value.clone()));
        }

        if let Some(generator) = &definition.generator {
          map.insert("generator".to_string(), generator.to_json().unwrap_or_default());
        }

        Value::Object(map.clone())
      }
      MatchingRule::EachValue(definition) => {
        let mut json = json!({
          "match": "eachValue",
          "rules": definition.rules.iter()
            .map(|rule| rule.as_ref().expect_left("Expected a matching rule, found an unresolved reference").to_json())
          .collect::<Vec<Value>>()
        });
        let map = json.as_object_mut().unwrap();

        if !definition.value.is_empty() {
          map.insert("value".to_string(), Value::String(definition.value.clone()));
        }

        if let Some(generator) = &definition.generator {
          map.insert("generator".to_string(), generator.to_json().unwrap_or_default());
        }

        Value::Object(map.clone())
      }
    }
  }

  /// If there are any generators associated with this matching rule
  pub fn has_generators(&self) -> bool {
    match self {
      MatchingRule::ArrayContains(variants) => variants.iter()
        .any(|(_, _, generators)| !generators.is_empty()),
      _ => false
    }
  }

  /// Return the generators for this rule
  pub fn generators(&self) -> Vec<Generator> {
    match self {
      MatchingRule::ArrayContains(variants) => vec![Generator::ArrayContains(variants.clone())],
      _ => vec![]
    }
  }

  /// Returns the type name of this matching rule
  pub fn name(&self) -> String {
    match self {
      MatchingRule::Equality => "equality",
      MatchingRule::Regex(_) => "regex",
      MatchingRule::Type => "type",
      MatchingRule::MinType(_) => "min-type",
      MatchingRule::MaxType(_) => "max-type",
      MatchingRule::MinMaxType(_, _) => "min-max-type",
      MatchingRule::Timestamp(_) => "datetime",
      MatchingRule::Time(_) => "time",
      MatchingRule::Date(_) => "date",
      MatchingRule::Include(_) => "include",
      MatchingRule::Number => "number",
      MatchingRule::Integer => "integer",
      MatchingRule::Decimal => "decimal",
      MatchingRule::Null => "null",
      MatchingRule::ContentType(_) => "content-type",
      MatchingRule::ArrayContains(_) => "array-contains",
      MatchingRule::Values => "values",
      MatchingRule::Boolean => "boolean",
      MatchingRule::StatusCode(_) => "status-code",
      MatchingRule::NotEmpty => "not-empty",
      MatchingRule::Semver => "semver",
      MatchingRule::EachKey(_) => "each-key",
      MatchingRule::EachValue(_) => "each-value"
    }.to_string()
  }

  /// Returns the type name of this matching rule
  pub fn values(&self) -> HashMap<&'static str, Value> {
    let empty = hashmap!{};
    match self {
      MatchingRule::Equality => empty,
      MatchingRule::Regex(r) => hashmap!{ "regex" => Value::String(r.clone()) },
      MatchingRule::Type => empty,
      MatchingRule::MinType(min) => hashmap!{ "min" => json!(min) },
      MatchingRule::MaxType(max) => hashmap!{ "max" => json!(max) },
      MatchingRule::MinMaxType(min, max) => hashmap!{ "min" => json!(min), "max" => json!(max) },
      MatchingRule::Timestamp(f) => hashmap!{ "format" => Value::String(f.clone()) },
      MatchingRule::Time(f) => hashmap!{ "format" => Value::String(f.clone()) },
      MatchingRule::Date(f) => hashmap!{ "format" => Value::String(f.clone()) },
      MatchingRule::Include(s) => hashmap!{ "value" => Value::String(s.clone()) },
      MatchingRule::Number => empty,
      MatchingRule::Integer => empty,
      MatchingRule::Decimal => empty,
      MatchingRule::Null => empty,
      MatchingRule::ContentType(ct) => hashmap!{ "value" => Value::String(ct.clone()) },
      MatchingRule::ArrayContains(variants) => hashmap! { "variants" =>
        variants.iter().map(|(variant, rules, gens)| {
          Value::Array(vec![json!(variant), rules.to_v3_json(), Value::Object(gens.iter().map(|(key, gen)| {
            (key.to_string(), gen.to_json().unwrap())
          }).collect())])
        }).collect()
      },
      MatchingRule::Values => empty,
      MatchingRule::Boolean => empty,
      MatchingRule::StatusCode(sc) => hashmap!{ "status" => sc.to_json() },
      MatchingRule::NotEmpty => empty,
      MatchingRule::Semver => empty,
      MatchingRule::EachKey(definition) | MatchingRule::EachValue(definition) => {
        let mut map = hashmap! {
          "rules" => Value::Array(definition.rules.iter()
            .map(|rule| rule.as_ref().expect_left("Expected a matching rule, found an unresolved reference").to_json())
            .collect())
        };

        if !definition.value.is_empty() {
          map.insert("value", Value::String(definition.value.clone()));
        }

        if let Some(generator) = &definition.generator {
          map.insert("generator", generator.to_json().unwrap_or_default());
        }

        map
      }
    }
  }

  /// Creates a `MatchingRule` from a type and a map of attributes
  pub fn create(rule_type: &str, attributes: &Value) -> anyhow::Result<MatchingRule> {
    trace!("rule_type: {}, attributes: {}", rule_type, attributes);
    let attributes = match attributes {
      Value::Object(values) => values.clone(),
      Value::Null => Map::default(),
      _ => {
        error!("Matching rule attributes {} are not valid", attributes);
        return Err(anyhow!("Matching rule attributes {} are not valid", attributes));
      }
    };
    match rule_type {
      "regex" => match attributes.get(rule_type) {
        Some(s) => Ok(MatchingRule::Regex(json_to_string(s))),
        None => Err(anyhow!("Regex matcher missing 'regex' field")),
      },
      "equality" => Ok(MatchingRule::Equality),
      "include" => match attributes.get("value") {
        Some(s) => Ok(MatchingRule::Include(json_to_string(s))),
        None => Err(anyhow!("Include matcher missing 'value' field")),
      },
      "type" => match (json_to_num(attributes.get("min").cloned()), json_to_num(attributes.get("max").cloned())) {
        (Some(min), Some(max)) => Ok(MatchingRule::MinMaxType(min, max)),
        (Some(min), None) => Ok(MatchingRule::MinType(min)),
        (None, Some(max)) => Ok(MatchingRule::MaxType(max)),
        _ => Ok(MatchingRule::Type)
      },
      "number" => Ok(MatchingRule::Number),
      "integer" => Ok(MatchingRule::Integer),
      "decimal" => Ok(MatchingRule::Decimal),
      "real" => Ok(MatchingRule::Decimal),
      "boolean" => Ok(MatchingRule::Boolean),
      "min" => match json_to_num(attributes.get(rule_type).cloned()) {
        Some(min) => Ok(MatchingRule::MinType(min)),
        None => Err(anyhow!("Min matcher missing 'min' field")),
      },
      "max" => match json_to_num(attributes.get(rule_type).cloned()) {
        Some(max) => Ok(MatchingRule::MaxType(max)),
        None => Err(anyhow!("Max matcher missing 'max' field")),
      },
      "timestamp" | "datetime" => match attributes.get("format").or_else(|| attributes.get(rule_type)) {
        Some(s) => Ok(MatchingRule::Timestamp(json_to_string(s))),
        None => Ok(MatchingRule::Timestamp(String::default())),
      },
      "date" => match attributes.get("format").or_else(|| attributes.get(rule_type)) {
        Some(s) => Ok(MatchingRule::Date(json_to_string(s))),
        None => Ok(MatchingRule::Date(String::default())),
      },
      "time" => match attributes.get("format").or_else(|| attributes.get(rule_type)) {
        Some(s) => Ok(MatchingRule::Time(json_to_string(s))),
        None => Ok(MatchingRule::Time(String::default()))
      },
      "null" => Ok(MatchingRule::Null),
      "contentType" | "content-type" => match attributes.get("value") {
        Some(s) => Ok(MatchingRule::ContentType(json_to_string(s))),
        None => Err(anyhow!("ContentType matcher missing 'value' field")),
      },
      "arrayContains" | "array-contains" => match attributes.get("variants") {
        Some(variants) => match variants {
          Value::Array(variants) => {
            let mut values = Vec::new();
            for variant in variants {
              let index = json_to_num(variant.get("index").cloned()).unwrap_or_default();
              let mut category = MatchingRuleCategory::empty("body");
              if let Some(rules) = variant.get("rules") {
                category.add_rules_from_json(rules)
                  .with_context(||
                    format!("Unable to parse matching rules: {:?}", rules))?;
              } else {
                category.add_rule(
                  DocPath::empty(), MatchingRule::Equality, RuleLogic::And);
              }
              let generators = if let Some(generators_json) = variant.get("generators") {
                let mut g = Generators::default();
                let cat = GeneratorCategory::BODY;
                if let Value::Object(map) = generators_json {
                  for (k, v) in map {
                    if let Value::Object(ref map) = v {
                      let path = DocPath::new(k)?;
                      g.parse_generator_from_map(&cat, map, Some(path));
                    }
                  }
                }
                g.categories.get(&cat).cloned().unwrap_or_default()
              } else {
                HashMap::default()
              };
              values.push((index, category, generators));
            }
            Ok(MatchingRule::ArrayContains(values))
          }
          _ => Err(anyhow!("ArrayContains matcher 'variants' field is not an Array")),
        }
        None => Err(anyhow!("ArrayContains matcher missing 'variants' field")),
      }
      "values" => Ok(MatchingRule::Values),
      "statusCode" | "status-code" => match attributes.get("status") {
        Some(s) => {
          let status = HttpStatus::from_json(s)
            .context("Unable to parse status code for StatusCode matcher")?;
          Ok(MatchingRule::StatusCode(status))
        },
        None => Ok(MatchingRule::StatusCode(HttpStatus::Success))
      },
      "notEmpty" | "not-empty" => Ok(MatchingRule::NotEmpty),
      "semver" => Ok(MatchingRule::Semver),
      "eachKey" | "each-key" => {
        let generator = generator_from_json(&attributes);
        let value = attributes.get("value").cloned().unwrap_or_default();
        let rules = rules_from_json(&attributes)?;
        let definition = MatchingRuleDefinition {
          value: json_to_string(&value),
          value_type: ValueType::Unknown,
          rules,
          generator
        };
        Ok(MatchingRule::EachKey(definition))
      }
      "eachValue" | "each-value" => {
        let generator = generator_from_json(&attributes);
        let value = attributes.get("value").cloned().unwrap_or_default();
        let rules = rules_from_json(&attributes)?;
        let definition = MatchingRuleDefinition {
          value: json_to_string(&value),
          value_type: ValueType::Unknown,
          rules,
          generator
        };
        Ok(MatchingRule::EachValue(definition))
      }
      _ => Err(anyhow!("{} is not a valid matching rule type", rule_type)),
    }
  }

  /// If this matching rule is a values matcher (ignores keys in maps)
  pub fn is_values_matcher(&self) -> bool {
    match self {
      MatchingRule::Values => true,
      MatchingRule::EachValue(_) => true,
      _ => false
    }
  }

  /// If this matcher should cascade to children
  pub fn can_cascade(&self) -> bool {
    match self {
      MatchingRule::Values => false,
      MatchingRule::EachValue(_) => false,
      MatchingRule::EachKey(_) => false,
      _ => true
    }
  }
}

impl Hash for MatchingRule {
  fn hash<H: Hasher>(&self, state: &mut H) {
    mem::discriminant(self).hash(state);
    match self {
      MatchingRule::Regex(s) => s.hash(state),
      MatchingRule::MinType(min) => min.hash(state),
      MatchingRule::MaxType(max) => max.hash(state),
      MatchingRule::MinMaxType(min, max) => {
        min.hash(state);
        max.hash(state);
      }
      MatchingRule::Timestamp(format) => format.hash(state),
      MatchingRule::Time(format) => format.hash(state),
      MatchingRule::Date(format) => format.hash(state),
      MatchingRule::Include(str) => str.hash(state),
      MatchingRule::ContentType(str) => str.hash(state),
      MatchingRule::ArrayContains(variants) => {
        for (index, rules, generators) in variants {
          index.hash(state);
          rules.hash(state);
          for (s, g) in generators {
            s.hash(state);
            g.hash(state);
          }
        }
      }
      _ => ()
    }
  }
}

impl PartialEq for MatchingRule {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (MatchingRule::Regex(s1), MatchingRule::Regex(s2)) => s1 == s2,
      (MatchingRule::MinType(min1), MatchingRule::MinType(min2)) => min1 == min2,
      (MatchingRule::MaxType(max1), MatchingRule::MaxType(max2)) => max1 == max2,
      (MatchingRule::MinMaxType(min1, max1), MatchingRule::MinMaxType(min2, max2)) => min1 == min2 && max1 == max2,
      (MatchingRule::Timestamp(format1), MatchingRule::Timestamp(format2)) => format1 == format2,
      (MatchingRule::Time(format1), MatchingRule::Time(format2)) => format1 == format2,
      (MatchingRule::Date(format1), MatchingRule::Date(format2)) => format1 == format2,
      (MatchingRule::Include(str1), MatchingRule::Include(str2)) => str1 == str2,
      (MatchingRule::ContentType(str1), MatchingRule::ContentType(str2)) => str1 == str2,
      (MatchingRule::ArrayContains(variants1), MatchingRule::ArrayContains(variants2)) => variants1 == variants2,
      _ => mem::discriminant(self) == mem::discriminant(other)
    }
  }
}

/// Enumeration to define how to combine rules
#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
pub enum RuleLogic {
  /// All rules must match
  And,
  /// At least one rule must match
  Or
}

impl RuleLogic {
  fn to_json(&self) -> Value {
    Value::String(match self {
      RuleLogic::And => "AND",
      RuleLogic::Or => "OR"
    }.into())
  }
}

/// Data structure for representing a list of rules and the logic needed to combine them
#[derive(Debug, Clone, Eq)]
pub struct RuleList {
  /// List of rules to apply
  pub rules: Vec<MatchingRule>,
  /// Rule logic to use to evaluate multiple rules
  pub rule_logic: RuleLogic,
  /// If this rule list has matched the exact path or if it has cascaded (i.e. is a parent)
  pub cascaded: bool
}

impl RuleList {

  /// Creates a new empty rule list
  pub fn empty(rule_logic: RuleLogic) -> RuleList {
    RuleList {
      rules: Vec::new(),
      rule_logic,
      cascaded: false
    }
  }

  /// Creates a default rule list with an equality matcher
  pub fn equality() -> RuleList {
    RuleList {
      rules: vec![ MatchingRule::Equality ],
      rule_logic: RuleLogic::And,
      cascaded: false
    }
  }

  /// Creates a new rule list with the single matching rule
  pub fn new(rule: MatchingRule) -> RuleList {
    RuleList {
      rules: vec![ rule ],
      rule_logic: RuleLogic::And,
      cascaded: false
    }
  }

  /// If the rule list is empty (has no matchers)
  pub fn is_empty(&self) -> bool {
    self.rules.is_empty()
  }

  fn to_v3_json(&self) -> Value {
    json!({
      "combine": self.rule_logic.to_json(),
      "matchers": Value::Array(self.rules.iter().map(|matcher| matcher.to_json()).collect())
    })
  }

  fn to_v2_json(&self) -> Value {
    match self.rules.get(0) {
      Some(rule) => rule.to_json(),
      None => json!({})
    }
  }

  /// If there is a type matcher defined for the rule list
  pub fn type_matcher_defined(&self) -> bool {
    self.rules.iter().any(|rule| match rule {
      MatchingRule::Type => true,
      MatchingRule::MinType(_) => true,
      MatchingRule::MaxType(_) => true,
      MatchingRule::MinMaxType(_, _) => true,
      _ => false
    })
  }

  /// If the values matcher is defined for the rule list
  pub fn values_matcher_defined(&self) -> bool {
    self.rules.iter().any(MatchingRule::is_values_matcher)
  }

  /// Add a matching rule to the rule list
  pub fn add_rule(&mut self, rule: &MatchingRule) {
    self.rules.push(rule.clone())
  }

  /// If this rule list has matched the exact path or if it has cascaded (i.e. is a parent)
  pub fn as_cascaded(&self, b: bool) -> RuleList {
    RuleList {
      cascaded: b,
      .. self.clone()
    }
  }

  /// Add all the rules from the list to this list
  pub fn add_rules(&mut self, rules: &RuleList) {
    for rule in &rules.rules {
      self.add_rule(rule);
    }
  }
}

impl Hash for RuleList {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.rule_logic.hash(state);
    for rule in &self.rules {
      rule.hash(state);
    }
  }
}

impl PartialEq for RuleList {
  fn eq(&self, other: &Self) -> bool {
    self.rule_logic == other.rule_logic &&
      self.rules == other.rules
  }
}

impl Default for RuleList {
  fn default() -> Self {
    RuleList::empty(RuleLogic::And)
  }
}

/// Category that the matching rule is applied to
#[derive(PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
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
  STATUS,
  /// Message contents (body)
  CONTENTS,
  /// Message metadata
  METADATA
}

impl FromStr for Category {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "method" => Ok(Category::METHOD),
      "path" => Ok(Category::PATH),
      "header" => Ok(Category::HEADER),
      "query" => Ok(Category::QUERY),
      "body" => Ok(Category::BODY),
      "status" => Ok(Category::STATUS),
      "contents" => Ok(Category::CONTENTS),
      "metadata" => Ok(Category::METADATA),
      _ => Err(format!("'{}' is not a valid Category", s))
    }
  }
}

impl <'a> Into<&'a str> for Category {
  fn into(self) -> &'a str {
    match self {
      Category::METHOD => "method",
      Category::PATH => "path",
      Category::HEADER => "header",
      Category::QUERY => "query",
      Category::BODY => "body",
      Category::STATUS => "status",
      Category::CONTENTS => "contents",
      Category::METADATA => "metadata"
    }
  }
}

impl Into<String> for Category {
  fn into(self) -> String {
    self.to_string()
  }
}

impl <'a> From<&'a str> for Category {
  fn from(s: &'a str) -> Self {
    Category::from_str(s).unwrap_or_default()
  }
}

impl From<String> for Category {
  fn from(s: String) -> Self {
    Category::from_str(&s).unwrap_or_default()
  }
}

impl Default for Category {
  fn default() -> Self {
    Category::BODY
  }
}

impl Display for Category {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let s: &str = self.clone().into();
    write!(f, "{}", s)
  }
}

/// Data structure for representing a category of matching rules
#[derive(Debug, Clone, Eq, Default)]
pub struct MatchingRuleCategory {
  /// Name of the category
  pub name: Category,
  /// Matching rules for this category
  pub rules: HashMap<DocPath, RuleList>
}

impl MatchingRuleCategory {
  /// Creates an empty category
  pub fn empty<S>(name: S) -> MatchingRuleCategory
    where S: Into<Category>
  {
    MatchingRuleCategory {
      name: name.into(),
      rules: hashmap! {},
    }
  }

  /// Creates a default category
  pub fn equality<S>(name: S) -> MatchingRuleCategory
    where S: Into<Category>
  {
    MatchingRuleCategory {
      name: name.into(),
      rules: hashmap! {
        DocPath::empty() => RuleList::equality()
      }
    }
  }

  /// If the matching rules in the category are empty
  pub fn is_empty(&self) -> bool {
    self.rules.is_empty()
  }

  /// If the matching rules in the category are not empty
  pub fn is_not_empty(&self) -> bool {
    !self.rules.is_empty()
  }

  /// Adds a rule from the Value representation
  pub fn rule_from_json(
    &mut self,
    key: DocPath,
    matcher_json: &Value,
    rule_logic: RuleLogic,
  ) -> anyhow::Result<()> {
    let matching_rule = MatchingRule::from_json(matcher_json)
      .with_context(|| format!("Could not parse matcher JSON {:?}", matcher_json))?;

    let rules = self.rules.entry(key)
      .or_insert_with(|| RuleList::empty(rule_logic));
    rules.rules.push(matching_rule);
    Ok(())
  }

  /// Adds a rule to this category
  pub fn add_rule(
    &mut self,
    key: DocPath,
    matcher: MatchingRule,
    rule_logic: RuleLogic,
  ) {
    let rules = self.rules.entry(key).or_insert_with(|| RuleList::empty(rule_logic));
    rules.rules.push(matcher);
  }

  /// Filters the matchers in the category by the predicate, and returns a new category
  pub fn filter<F>(&self, predicate: F) -> MatchingRuleCategory
    where F : Fn(&(&DocPath, &RuleList)) -> bool {
    MatchingRuleCategory {
      name: self.name.clone(),
      rules: self.rules.iter().filter(predicate)
        .map(|(path, rules)| (path.clone(), rules.clone())).collect()
    }
  }

  fn max_by_path(&self, path: &[&str]) -> RuleList {
    self.rules.iter().map(|(k, v)| (k, v, k.path_weight(path)))
      .filter(|&(_, _, (w, _))| w > 0)
      .max_by_key(|&(_, _, (w, t))| w * t)
      .map(|(_, v, (_, t))| v.as_cascaded(t != path.len()))
      .unwrap_or_default()
  }

  /// Returns a JSON Value representation in V3 format
  pub fn to_v3_json(&self) -> Value {
    Value::Object(self.rules.iter().fold(serde_json::Map::new(), |mut map, (category, rulelist)| {
      match self.name {
        Category::HEADER | Category::QUERY => {
          let name = category.first_field().map(|v| v.to_string())
            .unwrap_or_else(|| category.to_string());
          map.insert(name, rulelist.to_v3_json());
        }
        _ => {
          map.insert(String::from(category), rulelist.to_v3_json());
        }
      }
      map
    }))
  }

  /// Returns a JSON Value representation in V2 format
  pub fn to_v2_json(&self) -> HashMap<String, Value> {
    let mut map = hashmap!{};

    match &self.name {
      Category::PATH => for (_, v) in self.rules.clone() {
        map.insert("$.path".to_string(), v.to_v2_json());
      }
      Category::BODY => for (k, v) in self.rules.clone() {
        map.insert(String::from(k).replace("$", "$.body"), v.to_v2_json());
      }
      Category::HEADER | Category::QUERY => for (k, v) in &self.rules {
        let mut path = DocPath::root();
        path.push_field(self.name.to_string());
        path.push_path(k);
        map.insert(path.to_string(), v.to_v2_json());
      }
      _ => for (k, v) in &self.rules {
        map.insert(format!("$.{}.{}", self.name, k), v.to_v2_json());
      }
    };

    map
  }

  /// If there is a type matcher defined for the category
  pub fn type_matcher_defined(&self) -> bool {
    self.rules.values().any(|rule_list| rule_list.type_matcher_defined())
  }

  /// If there is a values matcher defined in the rules
  pub fn values_matcher_defined(&self) -> bool {
    self.rules.values().any(|rule_list| rule_list.values_matcher_defined())
  }

  /// If there is a matcher defined for the path
  pub fn matcher_is_defined(&self, path: &[&str]) -> bool {
    let result = !self.resolve_matchers_for_path(path).is_empty();
    trace!("matcher_is_defined: for category {} and path {:?} -> {}", self.name.to_string(), path, result);
    result
  }

  /// filters this category with all rules that match the given path for categories that contain
  /// collections (eg. bodies, headers, query parameters). Returns self otherwise.
  pub fn resolve_matchers_for_path(&self, path: &[&str]) -> MatchingRuleCategory {
    match self.name {
      Category::HEADER| Category::QUERY | Category::BODY |
      Category::CONTENTS | Category::METADATA => self.filter(|(val, _)| {
        val.matches_path(path)
      }),
      _ => self.clone()
    }
  }

  /// Selects the best matcher for the given path by calculating a weighting for each one
  pub fn select_best_matcher(&self, path: &[&str]) -> RuleList {
    match self.name {
      Category::BODY | Category::METADATA => self.max_by_path(path),
      _ => self.resolve_matchers_for_path(path).as_rule_list()
    }
  }

  /// Returns this category as a matching rule list. Returns a None if there are no rules
  pub fn as_rule_list(&self) -> RuleList {
    self.rules.values().next().cloned().unwrap_or_default()
  }

  /// Adds the rules to the category from the provided JSON
  pub fn add_rules_from_json(&mut self, rules: &Value) -> anyhow::Result<()> {
    if self.name == Category::PATH && rules.get("matchers").is_some() {
      let rule_logic = match rules.get("combine") {
        Some(val) => if json_to_string(val).to_uppercase() == "OR" {
          RuleLogic::Or
        } else {
          RuleLogic::And
        },
        None => RuleLogic::And
      };
      if let Some(matchers) = rules.get("matchers") {
        if let Value::Array(array) = matchers {
          for matcher in array {
            self.rule_from_json(DocPath::empty(), &matcher, rule_logic)?;
          }
        }
      }
    } else if let Value::Object(m) = rules {
      if m.contains_key("matchers") {
        self.add_rule_list(DocPath::empty(), rules)?;
      } else {
        for (k, v) in m {
          self.add_rule_list(DocPath::new(k)?, v)?;
        }
      }
    }
    Ok(())
  }

  /// Adds the rules to the category from the provided JSON in V3+ format
  pub fn add_v3_rules_from_json(&mut self, rules: &Value) -> anyhow::Result<()> {
    if self.name == Category::PATH && rules.get("matchers").is_some() {
      let rule_logic = match rules.get("combine") {
        Some(val) => if json_to_string(val).to_uppercase() == "OR" {
          RuleLogic::Or
        } else {
          RuleLogic::And
        },
        None => RuleLogic::And
      };
      if let Some(matchers) = rules.get("matchers") {
        if let Value::Array(array) = matchers {
          for matcher in array {
            self.rule_from_json(DocPath::empty(), &matcher, rule_logic)?;
          }
        }
      }
    } else if let Value::Object(m) = rules {
      if m.contains_key("matchers") {
        self.add_rule_list(DocPath::empty(), rules)?;
      } else if self.name == Category::QUERY || self.name == Category::HEADER {
        for (k, v) in m {
          let mut path = DocPath::root();
          path.push_field(k);
          self.add_rule_list(path, v)?;
        }
      } else {
        for (k, v) in m {
          self.add_rule_list(DocPath::new(k)?, v)?;
        }
      }
    } else {
      return Err(anyhow!("Matching rule JSON {} is not correctly formed", rules));
    }
    Ok(())
  }

  fn add_rule_list(&mut self, k: DocPath, v: &Value) -> anyhow::Result<()> {
    let rule_logic = match v.get("combine") {
      Some(val) => if json_to_string(val).to_uppercase() == "OR" {
        RuleLogic::Or
      } else {
        RuleLogic::And
      },
      None => RuleLogic::And
    };
    if let Some(&Value::Array(ref array)) = v.get("matchers") {
      for matcher in array {
        self.rule_from_json(k.clone(), &matcher, rule_logic)?;
      }
    }
    Ok(())
  }

  /// Returns any generators associated with these matching rules
  pub fn generators(&self) -> HashMap<DocPath, Generator> {
    let mut generators = hashmap!{};
    for (base_path, rules) in &self.rules {
      for rule in &rules.rules {
        if rule.has_generators() {
          for generator in rule.generators() {
            generators.insert(base_path.clone(), generator);
          }
        }
      }
    }
    generators
  }

  /// Clones this category with the new name
  pub fn rename<S>(&self, name: S) -> Self
    where S: Into<Category> {
    MatchingRuleCategory {
      name: name.into(),
      .. self.clone()
    }
  }

  /// Add all the rules from the provided rules
  pub fn add_rules(&mut self, category: MatchingRuleCategory) {
    for (path, rules) in &category.rules {
      if self.rules.contains_key(path) {
        self.rules.get_mut(path).unwrap().add_rules(rules)
      } else {
        self.rules.insert(path.clone(), rules.clone());
      }
    }
  }
}

impl Hash for MatchingRuleCategory {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    for (k, v) in self.rules.iter()
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
      k.hash(state);
      v.hash(state);
    }
  }
}

impl PartialEq for MatchingRuleCategory {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.rules == other.rules
  }

  fn ne(&self, other: &Self) -> bool {
    self.name != other.name || self.rules != other.rules
  }
}

impl PartialOrd for MatchingRuleCategory {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for MatchingRuleCategory {
  fn cmp(&self, other: &Self) -> Ordering {
    self.name.cmp(&other.name)
  }
}

/// Data structure for representing a collection of matchers
#[derive(Debug, Clone, Eq)]
pub struct MatchingRules {
  /// Categories of matching rules
  pub rules: HashMap<Category, MatchingRuleCategory>
}

impl MatchingRules {

  /// If the matching rules are empty (that is there are no rules assigned to any categories)
  pub fn is_empty(&self) -> bool {
    self.rules.values().all(|category| category.is_empty())
  }

  /// If the matching rules are not empty (that is there is at least one rule assigned to a category)
  pub fn is_not_empty(&self) -> bool {
    self.rules.values().any(|category| category.is_not_empty())
  }

  /// Adds the category to the map of rules
  pub fn add_category<S>(&mut self, category: S) -> &mut MatchingRuleCategory
    where S: Into<Category> + Clone
  {
    let category = category.into();
    if !self.rules.contains_key(&category) {
      self.rules.insert(category.clone(), MatchingRuleCategory::empty(category.clone()));
    }
    self.rules.get_mut(&category).unwrap()
  }

  /// Returns all the category names in this rule set
  pub fn categories(&self) -> HashSet<Category> {
    self.rules.keys().cloned().collect()
  }

  /// Returns the category of rules for a given category name
  pub fn rules_for_category<S>(&self, category: S) -> Option<MatchingRuleCategory>
    where S: Into<Category> {
    self.rules.get(&category.into()).cloned()
  }

  /// If there is a matcher defined for the category and path
  pub fn matcher_is_defined<S>(&self, category: S, path: &Vec<&str>) -> bool
    where S: Into<Category> + Clone {
    let result = match self.resolve_matchers(category.clone().into(), path) {
      Some(ref rules) => !rules.is_empty(),
      None => false
    };
    trace!("matcher_is_defined for category {} and path {:?} -> {}", category.into(), path, result);
    result
  }

  /// If there is a wildcard matcher defined for the category and path
  pub fn wildcard_matcher_is_defined<S>(&self, category: S, path: &Vec<&str>) -> bool
    where S: Into<Category> + Clone {
    match self.resolve_wildcard_matchers(category, path) {
      Some(ref rules) => !rules.filter(|&(val, _)| val.is_wildcard()).is_empty(),
      None => false
    }
  }

  /// If there is a type matcher defined for the category and path
  pub fn type_matcher_defined<S>(&self, category: S, path: &Vec<&str>) -> bool
    where S: Into<Category> + Display + Clone {
    let result = match self.resolve_matchers(category.clone(), path) {
      Some(ref rules) => rules.type_matcher_defined(),
      None => false
    };
    trace!("type_matcher_defined for category {} and path {:?} -> {}", category.into(), path, result);
    result
  }

  /// Returns a `Category` filtered with all rules that match the given path.
  pub fn resolve_matchers<S>(&self, category: S, path: &Vec<&str>) -> Option<MatchingRuleCategory>
    where S: Into<Category> {
    self.rules_for_category(category)
      .map(|rules| rules.resolve_matchers_for_path(path))
  }

  /// Returns a list of rules from the body category that match the given path
  pub fn resolve_body_matchers_by_path(&self, path: &Vec<&str>) -> RuleList {
    match self.rules_for_category("body") {
      Some(category) => category.max_by_path(path),
      None => RuleList::default()
    }
  }

  fn resolve_wildcard_matchers<S>(&self, category: S, path: &Vec<&str>) -> Option<MatchingRuleCategory>
    where S: Into<Category> + Clone {
    let category = category.into();
    match category {
      Category::BODY => self.rules_for_category(Category::BODY).map(|category| category.filter(|&(val, _)| {
        val.matches_path_exactly(path)
      })),
      Category::HEADER | Category::QUERY => self.rules_for_category(category.clone()).map(|category| category.filter(|&(val, _)| {
        path.len() == 1 && Some(path[0]) == val.first_field()
      })),
      _ => self.rules_for_category(category)
    }
  }

  fn load_from_v2_map(&mut self, map: &serde_json::Map<String, Value>
  ) -> anyhow::Result<()> {
    for (key, v) in map {
      if key.starts_with("$.body") {
        if key == "$.body" {
          self.add_v2_rule("body", DocPath::root(), v)?;
        } else {
          self.add_v2_rule("body", DocPath::new(format!("${}", &key[6..]))?, v)?;
        }
      } else if key.starts_with("$.headers") {
        let path = DocPath::new(key)?;
        let header_name = path.tokens().iter().filter_map(|t| match t {
          PathToken::Field(n) => Some(n.clone()),
          _ => None
        }).skip(1).next();
        let header = header_name.ok_or_else(|| anyhow!("'{}' is not a valid path value for a matching rule", key))?;
        self.add_v2_rule("header", DocPath::new(header)?, v)?;
      } else {
        let path = DocPath::new(key)?;
        let mut names = path.tokens().iter().filter_map(|t| match t {
          PathToken::Field(n) => Some(n.clone()),
          _ => None
        });
        let category = names.next().ok_or_else(|| anyhow!("'{}' is not a valid path value for a matching rule", key))?;
        let name = names.next();
        self.add_v2_rule(
          category,
          if let Some(name) = name { DocPath::new(name)? } else { DocPath::empty() },
          v,
        )?;
      }
    }
    Ok(())
  }

  fn load_from_v3_map(&mut self, map: &serde_json::Map<String, Value>
  ) -> anyhow::Result<()> {
    for (k, v) in map {
      let category = self.add_category(k.as_str());
      category.add_v3_rules_from_json(v)?;
    }
    Ok(())
  }

  fn add_v2_rule<S: Into<String>>(
    &mut self,
    category_name: S,
    sub_category: DocPath,
    rule: &Value,
  ) -> anyhow::Result<()> {
    let category = self.add_category(category_name.into());
    category.rule_from_json(sub_category, rule, RuleLogic::And)
  }

  fn to_v3_json(&self) -> Value {
    Value::Object(self.rules.iter()
      .fold(serde_json::Map::new(), |mut map, (name, sub_category)| {
      match name {
        Category::PATH => if let Some(rules) = sub_category.rules.get(&DocPath::empty()).or_else(|| sub_category.rules.get(&DocPath::root())) {
          map.insert(name.to_string(), rules.to_v3_json());
        }
        _ => {
          map.insert(name.to_string(), sub_category.to_v3_json());
        }
      }
      map
    }))
  }

  fn to_v2_json(&self) -> Value {
    Value::Object(self.rules.iter().fold(serde_json::Map::new(), |mut map, (_, category)| {
      for (key, value) in category.to_v2_json() {
        map.insert(key.clone(), value);
      }
      map
    }))
  }

  /// Clones the matching rules, renaming the category
  pub fn rename<S>(&self, old_name: S, new_name: S) -> Self
    where S: Into<Category> {
    let old = old_name.into();
    let new = new_name.into();
    MatchingRules {
      rules: self.rules.iter().map(|(key, value)| {
        if key == &old {
          (new.clone(), value.rename(new.clone()))
        } else {
          (key.clone(), value.clone())
        }
      }).collect()
    }
  }

  /// Add the rules to the category
  pub fn add_rules<S>(&mut self, category: S, rules: MatchingRuleCategory) where S: Into<Category> {
    let category = category.into();
    let entry = self.rules.entry(category.clone())
      .or_insert_with(|| MatchingRuleCategory::empty(category.clone()));
    entry.add_rules(rules);
  }

  /// Merge the rules from the other matching rules into this one.
  pub fn merge(&mut self, other: &MatchingRules) {
    for (category, rules) in &other.rules {
      self.add_rules(category.clone(), rules.clone());
    }
  }
}

impl Hash for MatchingRules {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (k, v) in self.rules.iter()
      .filter(|(_, cat)| cat.is_not_empty()) {
      k.hash(state);
      v.hash(state);
    }
  }
}

impl PartialEq for MatchingRules {
  fn eq(&self, other: &Self) -> bool {
    let self_rules = self.rules.iter()
      .filter(|(_, cat)| cat.is_not_empty())
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
      .collect_vec();
    let other_rules = other.rules.iter()
      .filter(|(_, cat)| cat.is_not_empty())
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
      .collect_vec();
    self_rules == other_rules
  }
}

impl Default for MatchingRules {
  fn default() -> Self {
    MatchingRules {
      rules: hashmap!{}
    }
  }
}

/// Parses the matching rules from the Value structure
pub fn matchers_from_json(value: &Value, deprecated_name: &Option<String>
) -> anyhow::Result<MatchingRules> {
  let matchers_json = match (value.get("matchingRules"), deprecated_name.clone().and_then(|name| value.get(&name))) {
    (Some(v), _) => Some(v),
    (None, Some(v)) => Some(v),
    (None, None) => None
  };

  let mut matching_rules = MatchingRules::default();
  match matchers_json {
    Some(value) => match value {
      &Value::Object(ref m) => {
        if m.keys().next().unwrap_or(&String::default()).starts_with("$") {
          matching_rules.load_from_v2_map(m)?
        } else {
          matching_rules.load_from_v3_map(m)?
        }
      },
      _ => ()
    },
    None => ()
  }
  Ok(matching_rules)
}

/// Generates a Value structure for the provided matching rules
pub fn matchers_to_json(matchers: &MatchingRules, spec_version: &PactSpecification) -> Value {
  match spec_version {
    PactSpecification::V3 | PactSpecification::V4 => matchers.to_v3_json(),
    _ => matchers.to_v2_json()
  }
}

/// Macro to ease constructing matching rules
/// Example usage:
/// ```
/// # use pact_models::matchingrules;
/// # use pact_models::matchingrules::MatchingRule;
/// matchingrules! {
///   "query" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
/// };
/// ```
#[macro_export]
macro_rules! matchingrules {
    ( $( $name:expr => {
        $( $subname:expr => [ $( $matcher:expr ), * ] ),*
    }), * ) => {{
        let mut _rules = $crate::matchingrules::MatchingRules::default();
        $({
            let mut _category = _rules.add_category($name);
            $({
              $({
                _category.add_rule(
                  $crate::path_exp::DocPath::new_unwrap($subname),
                  $matcher,
                  $crate::matchingrules::RuleLogic::And,
                );
              })*
            })*
        })*
        _rules
    }};
}

/// Macro to ease constructing matching rules
/// Example usage:
/// ```
/// # use pact_models::matchingrules_list;
/// # use pact_models::matchingrules::MatchingRule;
/// matchingrules_list! {
///   "body"; "user_id" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ]
/// };
/// ```
#[macro_export]
macro_rules! matchingrules_list {
  ( $name:expr ; $( $subname:expr => [ $( $matcher:expr ), * ] ),* ) => {{
    let mut _category = $crate::matchingrules::MatchingRuleCategory::empty($name);
    $(
      $(
        _category.add_rule(
          $crate::path_exp::DocPath::new_unwrap($subname),
          $matcher,
          $crate::matchingrules::RuleLogic::And,
        );
      )*
    )*
    _category
  }};

  ( $name:expr ; [ $( $matcher:expr ), * ] ) => {{
    let mut _category = $crate::matchingrules::MatchingRuleCategory::empty($name);
    $(
      _category.add_rule(
        $crate::path_exp::DocPath::empty(),
        $matcher,
        $crate::matchingrules::RuleLogic::And,
      );
    )*
    _category
  }};
}

#[cfg(test)]
mod tests {
  use std::collections::hash_map::DefaultHasher;

  use expectest::prelude::*;
  use maplit::hashset;
  use pretty_assertions::assert_eq;
  use serde_json::Value;
  use speculate::speculate;

  use crate::generators::*;

  use super::*;
  use super::super::*;

  fn h<H: Hash>(rule: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    rule.hash(&mut hasher);
    hasher.finish()
  }

  #[test]
  fn hash_and_partial_eq_for_matching_rule() {
    expect!(h(&MatchingRule::Equality)).to(be_equal_to(h(&MatchingRule::Equality)));
    expect!(MatchingRule::Equality).to(be_equal_to(MatchingRule::Equality));
    expect!(MatchingRule::Equality).to_not(be_equal_to(MatchingRule::Type));

    expect!(h(&MatchingRule::Type)).to(be_equal_to(h(&MatchingRule::Type)));
    expect!(MatchingRule::Type).to(be_equal_to(MatchingRule::Type));

    expect!(h(&MatchingRule::Number)).to(be_equal_to(h(&MatchingRule::Number)));
    expect!(MatchingRule::Number).to(be_equal_to(MatchingRule::Number));

    expect!(h(&MatchingRule::Integer)).to(be_equal_to(h(&MatchingRule::Integer)));
    expect!(MatchingRule::Integer).to(be_equal_to(MatchingRule::Integer));

    expect!(h(&MatchingRule::Decimal)).to(be_equal_to(h(&MatchingRule::Decimal)));
    expect!(MatchingRule::Decimal).to(be_equal_to(MatchingRule::Decimal));

    expect!(h(&MatchingRule::Null)).to(be_equal_to(h(&MatchingRule::Null)));
    expect!(MatchingRule::Null).to(be_equal_to(MatchingRule::Null));

    let regex1 = MatchingRule::Regex("\\d+".into());
    let regex2 = MatchingRule::Regex("\\w+".into());

    expect!(h(&regex1)).to(be_equal_to(h(&regex1)));
    expect!(&regex1).to(be_equal_to(&regex1));
    expect!(h(&regex1)).to_not(be_equal_to(h(&regex2)));
    expect!(&regex1).to_not(be_equal_to(&regex2));

    let min1 = MatchingRule::MinType(100);
    let min2 = MatchingRule::MinType(200);

    expect!(h(&min1)).to(be_equal_to(h(&min1)));
    expect!(&min1).to(be_equal_to(&min1));
    expect!(h(&min1)).to_not(be_equal_to(h(&min2)));
    expect!(&min1).to_not(be_equal_to(&min2));

    let max1 = MatchingRule::MaxType(100);
    let max2 = MatchingRule::MaxType(200);

    expect!(h(&max1)).to(be_equal_to(h(&max1)));
    expect!(&max1).to(be_equal_to(&max1));
    expect!(h(&max1)).to_not(be_equal_to(h(&max2)));
    expect!(&max1).to_not(be_equal_to(&max2));

    let minmax1 = MatchingRule::MinMaxType(100, 200);
    let minmax2 = MatchingRule::MinMaxType(200, 200);

    expect!(h(&minmax1)).to(be_equal_to(h(&minmax1)));
    expect!(&minmax1).to(be_equal_to(&minmax1));
    expect!(h(&minmax1)).to_not(be_equal_to(h(&minmax2)));
    expect!(&minmax1).to_not(be_equal_to(&minmax2));

    let datetime1 = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ss".into());
    let datetime2 = MatchingRule::Timestamp("yyyy-MM-ddTHH:mm:ss".into());

    expect!(h(&datetime1)).to(be_equal_to(h(&datetime1)));
    expect!(&datetime1).to(be_equal_to(&datetime1));
    expect!(h(&datetime1)).to_not(be_equal_to(h(&datetime2)));
    expect!(&datetime1).to_not(be_equal_to(&datetime2));

    let date1 = MatchingRule::Date("yyyy-MM-dd".into());
    let date2 = MatchingRule::Date("yy-MM-dd".into());

    expect!(h(&date1)).to(be_equal_to(h(&date1)));
    expect!(&date1).to(be_equal_to(&date1));
    expect!(h(&date1)).to_not(be_equal_to(h(&date2)));
    expect!(&date1).to_not(be_equal_to(&date2));

    let time1 = MatchingRule::Time("HH:mm:ss".into());
    let time2 = MatchingRule::Time("hh:mm:ss".into());

    expect!(h(&time1)).to(be_equal_to(h(&time1)));
    expect!(&time1).to(be_equal_to(&time1));
    expect!(h(&time1)).to_not(be_equal_to(h(&time2)));
    expect!(&time1).to_not(be_equal_to(&time2));

    let inc1 = MatchingRule::Include("string one".into());
    let inc2 = MatchingRule::Include("string two".into());

    expect!(h(&inc1)).to(be_equal_to(h(&inc1)));
    expect!(&inc1).to(be_equal_to(&inc1));
    expect!(h(&inc1)).to_not(be_equal_to(h(&inc2)));
    expect!(&inc1).to_not(be_equal_to(&inc2));

    let content1 = MatchingRule::ContentType("one".into());
    let content2 = MatchingRule::ContentType("two".into());

    expect!(h(&content1)).to(be_equal_to(h(&content1)));
    expect!(&content1).to(be_equal_to(&content1));
    expect!(h(&content1)).to_not(be_equal_to(h(&content2)));
    expect!(&content1).to_not(be_equal_to(&content2));

    let ac1 = MatchingRule::ArrayContains(vec![]);
    let ac2 = MatchingRule::ArrayContains(vec![(0, MatchingRuleCategory::empty("body"), hashmap!{})]);
    let ac3 = MatchingRule::ArrayContains(vec![(1, MatchingRuleCategory::empty("body"), hashmap!{})]);
    let ac4 = MatchingRule::ArrayContains(vec![(0, MatchingRuleCategory::equality("body"), hashmap!{})]);
    let ac5 = MatchingRule::ArrayContains(vec![(0, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomBoolean })]);
    let ac6 = MatchingRule::ArrayContains(vec![
      (0, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomBoolean }),
      (1, MatchingRuleCategory::empty("body"), hashmap!{ DocPath::new_unwrap("A") => Generator::RandomDecimal(10) })
    ]);
    let ac7 = MatchingRule::ArrayContains(vec![
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

  #[test]
  fn rules_are_empty_when_there_are_no_categories() {
    expect!(MatchingRules::default().is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_empty_when_there_are_only_empty_categories() {
    expect!(MatchingRules {
      rules: hashmap!{
        "body".into() => MatchingRuleCategory::empty("body"),
        "header".into() => MatchingRuleCategory::empty("header"),
        "query".into() => MatchingRuleCategory::empty("query")
      }
    }.is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_not_empty_when_there_is_a_nonempty_category() {
    expect!(MatchingRules {
      rules: hashmap!{
        "body".into() => MatchingRuleCategory::empty("body"),
        "header".into() => MatchingRuleCategory::empty("headers"),
        "query".into() => MatchingRuleCategory {
            name: "query".into(),
            rules: hashmap!{
              DocPath::empty() => RuleList {
                rules: vec![ MatchingRule::Equality ],
                rule_logic: RuleLogic::And,
                cascaded: false
              }
            }
        },
      }
    }.is_empty()).to(be_false());
  }

  #[test]
  fn matchers_from_json_test() {
    let matching_rules = matchers_from_json(&Value::Null, &None);
    let matching_rules = matching_rules.unwrap();
    expect!(matching_rules.rules.iter()).to(be_empty());
  }

  #[test]
  fn loads_v2_matching_rules() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "$.path": { "match": "regex", "regex": "\\w+" },
      "$.query.Q1": { "match": "regex", "regex": "\\d+" },
      "$.header.HEADERY": {"match": "include", "value": "ValueA"},
      "$.body.animals": {"min": 1, "match": "type"},
      "$.body.animals[*].*": {"match": "type"},
      "$.body.animals[*].children": {"min": 1},
      "$.body.animals[*].children[*].*": {"match": "type"}
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);
    let matching_rules = matching_rules.unwrap();

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{
      Category::PATH, Category::QUERY, Category::HEADER, Category::BODY
    }));
    expect!(matching_rules.rules_for_category("path")).to(be_some().value(MatchingRuleCategory {
      name: "path".into(),
      rules: hashmap! { DocPath::empty() => RuleList { rules: vec![ MatchingRule::Regex("\\w+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("query")).to(be_some().value(MatchingRuleCategory {
      name: "query".into(),
      rules: hashmap!{ DocPath::new_unwrap("Q1") => RuleList { rules: vec![ MatchingRule::Regex("\\d+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("header")).to(be_some().value(MatchingRuleCategory {
      name: "header".into(),
      rules: hashmap!{ DocPath::new_unwrap("HEADERY") => RuleList { rules: vec![
        MatchingRule::Include("ValueA".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("body")).to(be_some().value(MatchingRuleCategory {
      name: "body".into(),
      rules: hashmap!{
        DocPath::new_unwrap("$.animals") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].children") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].children[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And, cascaded: false }
      }
    }));
  }

  #[test]
  fn load_from_v2_map_supports_headers_and_query_parameters_in_encoded_format() {
    let matching_rules_json = json!({
      "$.query.Q1": { "match": "regex", "regex": "1" },
      "$.query.x-test": { "match": "regex", "regex": "2" },
      "$.query['x-test-2']": { "match": "regex", "regex": "3" },
      "$.header.HEADERY": { "match": "regex", "regex": "4" },
      "$.header.x-test": { "match": "regex", "regex": "5" },
      "$.header['x-test-2']": { "match": "regex", "regex": "6" }
    });
    let matching_rules_map = matching_rules_json.as_object().unwrap();

    let mut matching_rules = MatchingRules::default();
    matching_rules.load_from_v2_map(&matching_rules_map).unwrap();

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{
      Category::QUERY, Category::HEADER
    }));
    assert_eq!(matching_rules.rules_for_category("header").unwrap(), MatchingRuleCategory {
      name: "header".into(),
      rules: hashmap!{
        DocPath::new_unwrap("HEADERY") => RuleList { rules: vec![ MatchingRule::Regex("4".to_string()) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("x-test") => RuleList { rules: vec![ MatchingRule::Regex("5".to_string()) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("x-test-2") => RuleList { rules: vec![ MatchingRule::Regex("6".to_string()) ], rule_logic: RuleLogic::And, cascaded: false }
      }
    });
    assert_eq!(matching_rules.rules_for_category("query").unwrap(), MatchingRuleCategory {
      name: "query".into(),
      rules: hashmap!{
        DocPath::new_unwrap("Q1") => RuleList { rules: vec![ MatchingRule::Regex("1".to_string()) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("x-test") => RuleList { rules: vec![ MatchingRule::Regex("2".to_string()) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("x-test-2") => RuleList { rules: vec![ MatchingRule::Regex("3".to_string()) ], rule_logic: RuleLogic::And, cascaded: false }
      }
    });
  }

  #[test]
  fn loads_v3_matching_rules() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "path": {
        "matchers": [
          { "match": "regex", "regex": "\\w+" }
        ]
      },
      "query": {
        "Q1": {
          "matchers": [
            { "match": "regex", "regex": "\\d+" }
          ]
        }
      },
      "header": {
        "HEADERY": {
          "combine": "OR",
          "matchers": [
            {"match": "include", "value": "ValueA"},
            {"match": "include", "value": "ValueB"}
          ]
        }
      },
      "body": {
        "$.animals": {
          "matchers": [{"min": 1, "match": "type"}]
        },
        "$.animals[*].*": {
          "matchers": [{"match": "type"}]
        },
        "$.animals[*].children": {
          "matchers": [{"min": 1}]
        },
        "$.animals[*].children[*].*": {
          "matchers": [{"match": "type"}]
        }
      }
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);
    let matching_rules = matching_rules.unwrap();

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{
      Category::PATH, Category::QUERY, Category::HEADER, Category::BODY
    }));
    expect!(matching_rules.rules_for_category("path")).to(be_some().value(MatchingRuleCategory {
      name: "path".into(),
      rules: hashmap! { DocPath::empty() => RuleList { rules: vec![ MatchingRule::Regex("\\w+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("query")).to(be_some().value(MatchingRuleCategory {
      name: "query".into(),
      rules: hashmap!{ DocPath::root().join("Q1") => RuleList { rules: vec![ MatchingRule::Regex("\\d+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("header")).to(be_some().value(MatchingRuleCategory {
      name: "header".into(),
      rules: hashmap!{ DocPath::root().join("HEADERY") => RuleList { rules: vec![
        MatchingRule::Include("ValueA".to_string()),
        MatchingRule::Include("ValueB".to_string()) ], rule_logic: RuleLogic::Or, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("body")).to(be_some().value(MatchingRuleCategory {
      name: "body".into(),
      rules: hashmap!{
        DocPath::new_unwrap("$.animals") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].children") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And, cascaded: false },
        DocPath::new_unwrap("$.animals[*].children[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And, cascaded: false }
      }
    }));
  }

  #[test]
  fn correctly_loads_v3_matching_rules_with_incorrect_path_format() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "path": {
        "": {
          "matchers": [
            { "match": "regex", "regex": "\\w+" }
          ]
        }
      }
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);
    let matching_rules = matching_rules.unwrap();

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ Category::PATH }));
    expect!(matching_rules.rules_for_category("path")).to(be_some().value(MatchingRuleCategory {
      name: "path".into(),
      rules: hashmap! { DocPath::empty() => RuleList { rules: vec![ MatchingRule::Regex("\\w+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
  }

  speculate! {
    describe "generating matcher JSON" {
      before {
        let matchers = matchingrules!{
          "body" => {
            "$.a.b" => [ MatchingRule::Type ]
          },
          "path" => { "" => [ MatchingRule::Regex("/path/\\d+".to_string()) ] },
          "query" => {
            "a" => [ MatchingRule::Regex("\\w+".to_string()) ]
          },
          "header" => {
            "item1" => [ MatchingRule::Regex("5".to_string()) ]
          }
        };
      }

      it "generates V2 matcher format" {
        expect!(matchers.to_v2_json().to_string()).to(be_equal_to(
          "{\"$.body.a.b\":{\"match\":\"type\"},\
          \"$.header.item1\":{\"match\":\"regex\",\"regex\":\"5\"},\
          \"$.path\":{\"match\":\"regex\",\"regex\":\"/path/\\\\d+\"},\
          \"$.query.a\":{\"match\":\"regex\",\"regex\":\"\\\\w+\"}}"
        ));
      }

      it "generates V3 matcher format" {
        expect!(matchers.to_v3_json().to_string()).to(be_equal_to(
          "{\"body\":{\"$.a.b\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]}},\
          \"header\":{\"item1\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"5\"}]}},\
          \"path\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"/path/\\\\d+\"}]},\
          \"query\":{\"a\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"\\\\w+\"}]}}}"
        ));
      }
    }
  }

  #[test]
  fn loads_v3_matching_rules_supports_headers_and_query_parameters_with_brackets() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "query": {
        "Q[]": {
          "matchers": [
            { "match": "regex", "regex": "\\d+" }
          ]
        }
      },
      "header": {
        "Y[]": {
          "combine": "OR",
          "matchers": [
            {"match": "include", "value": "ValueA"},
            {"match": "include", "value": "ValueB"}
          ]
        }
      }
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);
    let matching_rules = matching_rules.unwrap();

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{
      Category::QUERY, Category::HEADER
    }));
    expect!(matching_rules.rules_for_category("query")).to(be_some().value(MatchingRuleCategory {
      name: "query".into(),
      rules: hashmap!{ DocPath::root().join("Q[]") => RuleList { rules: vec![ MatchingRule::Regex("\\d+".to_string()) ], rule_logic: RuleLogic::And, cascaded: false } }
    }));
    expect!(matching_rules.rules_for_category("header")).to(be_some().value(MatchingRuleCategory {
      name: "header".into(),
      rules: hashmap!{ DocPath::root().join("Y[]") => RuleList { rules: vec![
        MatchingRule::Include("ValueA".to_string()),
        MatchingRule::Include("ValueB".to_string()) ], rule_logic: RuleLogic::Or, cascaded: false } }
    }));
  }

  #[test]
  fn matching_rule_from_json_test() {
    expect!(MatchingRule::from_json(&Value::from_str("\"test string\"").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("null").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("{}").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("[]").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("true").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("false").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("100").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("100.10").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("{\"stuff\": 100}").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"stuff\"}").unwrap())).to(be_err());

    expect!(MatchingRule::from_json(&Value::from_str("{\"regex\": \"[0-9]\"}").unwrap())).to(
      be_ok().value(MatchingRule::Regex("[0-9]".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"min\": 100}").unwrap())).to(
      be_ok().value(MatchingRule::MinType(100)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"max\": 100}").unwrap())).to(
      be_ok().value(MatchingRule::MaxType(100)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"timestamp\": \"yyyy\"}").unwrap())).to(
      be_ok().value(MatchingRule::Timestamp("yyyy".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"date\": \"yyyy\"}").unwrap())).to(
      be_ok().value(MatchingRule::Date("yyyy".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"time\": \"hh:mm\"}").unwrap())).to(
      be_ok().value(MatchingRule::Time("hh:mm".to_string())));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"regex\", \"regex\": \"[0-9]\"}").unwrap())).to(
      be_ok().value(MatchingRule::Regex("[0-9]".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"regex\"}").unwrap())).to(be_err());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"equality\"}").unwrap())).to(
      be_ok().value(MatchingRule::Equality));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"include\", \"value\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Include("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"include\"}").unwrap())).to(be_err());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"min\": 1}").unwrap())).to(
      be_ok().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"max\": \"1\"}").unwrap())).to(
      be_ok().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"min\": 1, \"max\": \"1\"}").unwrap())).to(
      be_ok().value(MatchingRule::MinMaxType(1, 1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\"}").unwrap())).to(
      be_ok().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"value\": 100}").unwrap())).to(
      be_ok().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"min\", \"min\": 1}").unwrap())).to(
      be_ok().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"max\", \"max\": \"1\"}").unwrap())).to(
      be_ok().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"min\"}").unwrap())).to(be_err());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"max\"}").unwrap())).to(be_err());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"number\"}").unwrap())).to(
      be_ok().value(MatchingRule::Number));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"integer\"}").unwrap())).to(
      be_ok().value(MatchingRule::Integer));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"decimal\"}").unwrap())).to(
      be_ok().value(MatchingRule::Decimal));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"real\"}").unwrap())).to(
      be_ok().value(MatchingRule::Decimal));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"boolean\"}").unwrap())).to(
      be_ok().value(MatchingRule::Boolean));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"timestamp\", \"timestamp\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Timestamp("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"timestamp\"}").unwrap())).to(be_ok());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"time\", \"time\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Time("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"time\"}").unwrap())).to(be_ok());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"date\", \"date\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Date("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"date\"}").unwrap())).to(be_ok());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"null\"}").unwrap())).to(
      be_ok().value(MatchingRule::Null));

    let json = json!({
      "match": "arrayContains",
      "variants": []
    });
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(MatchingRule::ArrayContains(vec![])));

    let json = json!({
      "match": "arrayContains",
      "variants": [
        {
          "index": 0,
          "rules": {
            "matchers": [ { "match": "equality" } ]
          }
        }
      ]
    });
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(
      MatchingRule::ArrayContains(
        vec![
          (0, matchingrules_list! { "body"; [ MatchingRule::Equality ] }, HashMap::default())
        ])
    ));

    let json = json!({
      "match": "arrayContains",
      "variants": [
        {
          "index": 0,
          "rules": {
            "matchers": [ { "match": "equality" } ]
          },
          "generators": {
            "a": { "type": "Uuid" }
          }
        }
      ]
    });
    let generators = hashmap!{ DocPath::new_unwrap("a") => Generator::Uuid(None) };
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(
      MatchingRule::ArrayContains(
        vec![
          (0, matchingrules_list! { "body"; [ MatchingRule::Equality ] }, generators)
        ])
    ));

    let json = json!({
      "match": "statusCode",
      "status": "success"
    });
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(
      MatchingRule::StatusCode(HttpStatus::Success)
    ));

    let json = json!({
      "match": "statusCode",
      "status": [200, 201, 204]
    });
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(
      MatchingRule::StatusCode(HttpStatus::StatusCodes(vec![200, 201, 204]))
    ));
  }

  #[test]
  fn matching_rule_from_json_supports_integration_form() {
    let json = json!({
      "pact:matcher:type": "each-value",
      "value": {
        "price": 1.23
      },
      "rules": [
        {
          "pact:matcher:type": "decimal"
        }
      ]
    });
    expect!(MatchingRule::from_json(&json)).to(be_ok().value(
      MatchingRule::EachValue(MatchingRuleDefinition::new("{\"price\": 1.23}".to_string(),
        ValueType::Unknown, MatchingRule::Decimal, None)))
    );
  }

  #[test]
  fn date_time_matchers_can_parse_the_updated_spec_format() {
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"timestamp\", \"format\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Timestamp("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"datetime\", \"format\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Timestamp("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"time\", \"format\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Time("A".to_string())));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"date\", \"format\": \"A\"}").unwrap())).to(
      be_ok().value(MatchingRule::Date("A".to_string())));
  }

  #[test]
  fn matching_rule_to_json_test() {
    expect!(MatchingRule::StatusCode(HttpStatus::ClientError).to_json()).to(
      be_equal_to(json!({
        "match": "statusCode",
        "status": "clientError"
      })));
    expect!(MatchingRule::StatusCode(HttpStatus::StatusCodes(vec![400, 401, 404])).to_json()).to(
      be_equal_to(json!({
        "match": "statusCode",
        "status": [400, 401, 404]
      })));

    expect!(MatchingRule::Timestamp("YYYY".to_string()).to_json()).to(
      be_equal_to(json!({
        "match": "datetime",
        "format": "YYYY"
      })));
    expect!(MatchingRule::Date("YYYY".to_string()).to_json()).to(
      be_equal_to(json!({
        "match": "date",
        "format": "YYYY"
      })));
    expect!(MatchingRule::Time("HH".to_string()).to_json()).to(
      be_equal_to(json!({
        "match": "time",
        "format": "HH"
      })));
  }

  #[test]
  fn matcher_is_defined_returns_false_when_there_are_no_matchers() {
    let matchers = matchingrules!{};
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
    let matchers = matchingrules!{
      "body" => { }
    };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules! {
      "body" => {
        "$.a.b" => [ MatchingRule::Type ]
      }
    };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_true());
  }

  #[test]
  fn matcher_is_defined_returns_false_when_the_path_is_empty() {
    let matchers = matchingrules! {
      "body" => {
        "$.a.b" => [ MatchingRule::Type ]
      }
    };
    expect!(matchers.matcher_is_defined("body", &vec![])).to(be_false());
  }

  #[test]
  fn matcher_is_defined_returns_true_when_the_parent_of_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b", "c"])).to(be_true());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_there_are_no_matchers() {
    let matchers = matchingrules!{};
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
    let matchers = matchingrules!{
      "body" => { }
    };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_path_does_have_a_matcher_entry_and_it_is_not_a_wildcard() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ MatchingRule::Type ],
                "$.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry_and_it_is_a_widcard() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_true());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_parent_of_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b", "c"])).to(be_false());
  }

  #[test]
  fn min_and_max_values_get_serialised_to_json_as_numbers() {
    expect!(MatchingRule::MinType(1).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"min\":1}"));
    expect!(MatchingRule::MaxType(1).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"max\":1}"));
    expect!(MatchingRule::MinMaxType(1, 10).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"max\":10,\"min\":1}"));
  }

  #[test]
  fn rule_list_values_matcher_defined() {
    let rule_list_with_value_matcher = RuleList {
      rules: vec![ MatchingRule::Type, MatchingRule::Values, MatchingRule::Null ],
      rule_logic: RuleLogic::And,
      cascaded: false
    };

    let rule_list_with_each_value_matcher = RuleList {
      rules: vec![MatchingRule::Type, MatchingRule::EachValue(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![],
        generator: None
      }), MatchingRule::Null ],
      rule_logic: RuleLogic::And,
      cascaded: false
    };

    let rule_list_with_no_value_matcher = RuleList {
      rules: vec![ MatchingRule::Type, MatchingRule::Boolean, MatchingRule::Null ],
      rule_logic: RuleLogic::And,
      cascaded: false
    };

    expect!(rule_list_with_value_matcher.values_matcher_defined()).to(be_true());
    expect!(rule_list_with_each_value_matcher.values_matcher_defined()).to(be_true());
    expect!(rule_list_with_no_value_matcher.values_matcher_defined()).to(be_false());
  }

  // Issue https://github.com/pact-foundation/pact-js-core/issues/400
  #[test]
  fn to_json_with_matching_rule_on_path_test() {
    let matching_rules = MatchingRules {
      rules: hashmap!{
        Category::PATH => MatchingRuleCategory {
          name: Category::PATH,
          rules: hashmap!{
            DocPath::root() => RuleList {
              rules: vec![MatchingRule::Type],
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        }
      }
    };

    let json = matchers_to_json(&matching_rules, &PactSpecification::V3);
    expect!(json).to(be_equal_to(json!({
      "path": {
        "combine": "AND",
        "matchers": [ { "match": "type" } ]
      }
    })));
  }

  // Issue #355
  #[test]
  fn to_json_with_matching_rules_for_headers_and_query_parameters() {
    let matching_rules = MatchingRules {
      rules: hashmap!{
        Category::HEADER => MatchingRuleCategory {
          name: Category::HEADER,
          rules: hashmap!{
            DocPath::new_unwrap("$.A") => RuleList {
              rules: vec![MatchingRule::Type],
              rule_logic: RuleLogic::And,
              cascaded: false
            },
            DocPath::new_unwrap("$['se-token']") => RuleList {
              rules: vec![MatchingRule::Type],
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        },
        Category::QUERY => MatchingRuleCategory {
          name: Category::QUERY,
          rules: hashmap!{
            DocPath::new_unwrap("$.A") => RuleList {
              rules: vec![MatchingRule::Type],
              rule_logic: RuleLogic::And,
              cascaded: false
            },
            DocPath::new_unwrap("$['se-token']") => RuleList {
              rules: vec![MatchingRule::Type],
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        }
      }
    };

    let json = matchers_to_json(&matching_rules, &PactSpecification::V3);
    assert_eq!(json, json!({
      "header": {
        "A": {
          "combine": "AND",
          "matchers": [ { "match": "type" } ]
        },
        "se-token": {
          "combine": "AND",
          "matchers": [ { "match": "type" } ]
        }
      },
      "query": {
        "A": {
          "combine": "AND",
          "matchers": [ { "match": "type" } ]
        },
        "se-token": {
          "combine": "AND",
          "matchers": [ { "match": "type" } ]
        }
      }
    }));

    let json = matchers_to_json(&matching_rules, &PactSpecification::V2);
    assert_eq!(json, json!({
      "$.header.A": {
        "match": "type"
      },
      "$.header['se-token']": {
        "match": "type"
      },
      "$.query.A": {
        "match": "type"
      },
      "$.query['se-token']": {
        "match": "type"
      }
    }));
  }

  #[test]
  fn hash_test_for_matchingrules() {
    let m1 = MatchingRules::default();
    expect!(h(&m1)).to(be_equal_to(15130871412783076140));

    let m2 = MatchingRules {
      rules: hashmap!{
        Category::PATH => MatchingRuleCategory {
          name: Category::PATH,
          rules: hashmap!{
            DocPath::root() => RuleList {
              rules: vec![MatchingRule::Include("/1/2/3".to_string())],
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        }
      }
    };
    expect!(h(&m2)).to(be_equal_to(4156854679723555579));

    let m3 = MatchingRules {
      rules: hashmap! {
        Category::PATH => MatchingRuleCategory::empty(Category::PATH),
        Category::HEADER => MatchingRuleCategory::empty(Category::HEADER)
      }
    };
    expect!(h(&m3)).to(be_equal_to(15130871412783076140));
  }

  #[test]
  fn equals_test_for_matchingrules() {
    let m1 = MatchingRules::default();
    expect!(h(&m1)).to(be_equal_to(15130871412783076140));

    let m2 = MatchingRules {
      rules: hashmap!{
        Category::PATH => MatchingRuleCategory {
          name: Category::PATH,
          rules: hashmap!{
            DocPath::root() => RuleList {
              rules: vec![MatchingRule::Include("/1/2/3".to_string())],
              rule_logic: RuleLogic::And,
              cascaded: false
            }
          }
        }
      }
    };
    expect!(h(&m2)).to(be_equal_to(4156854679723555579));

    let m3 = MatchingRules {
      rules: hashmap! {
        Category::PATH => MatchingRuleCategory::empty(Category::PATH),
        Category::HEADER => MatchingRuleCategory::empty(Category::HEADER)
      }
    };

    assert_eq!(m1, m1);
    assert_eq!(m2, m2);
    assert_eq!(m3, m3);
    assert_eq!(m1, m3);

    assert_ne!(m1, m2);
    assert_ne!(m2, m3);
  }

  #[test]
  fn hash_test_for_matchingrule_category() {
    let m1 = MatchingRuleCategory::default();
    expect!(h(&m1)).to(be_equal_to(6185506036438099345));

    let m2 = MatchingRuleCategory {
      name: Category::PATH,
      rules: hashmap!{
        DocPath::root() => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        }
      }
    };
    expect!(h(&m2)).to(be_equal_to(5907609104530210046));

    let m3 = MatchingRuleCategory::empty(Category::HEADER);
    expect!(h(&m3)).to(be_equal_to(11876854719037224982));

    let m4 = MatchingRuleCategory::empty(Category::PATH);
    expect!(h(&m4)).to(be_equal_to(2206609067086327257));

    let m5 = MatchingRuleCategory {
      name: Category::PATH,
      rules: hashmap!{
        DocPath::root() => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        },
        DocPath::root().join("a") => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        }
      }
    };
    expect!(h(&m5)).to(be_equal_to(2282399821086239745));

    let m6 = MatchingRuleCategory {
      name: Category::PATH,
      rules: hashmap!{
        DocPath::root().join("a") => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        },
        DocPath::root() => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        }
      }
    };
    expect!(h(&m6)).to(be_equal_to(2282399821086239745));
  }

  #[test]
  fn equals_test_for_matchingrule_category() {
    let m1 = MatchingRuleCategory::default();
    let m2 = MatchingRuleCategory {
      name: Category::PATH,
      rules: hashmap!{
        DocPath::root() => RuleList {
          rules: vec![MatchingRule::Include("/1/2/3".to_string())],
          rule_logic: RuleLogic::And,
          cascaded: false
        }
      }
    };
    let m3 = MatchingRuleCategory::empty(Category::PATH);
    let m4 = MatchingRuleCategory::empty(Category::HEADER);

    assert_eq!(m1, m1);
    assert_eq!(m2, m2);
    assert_eq!(m3, m3);
    assert_eq!(m4, m4);

    assert_ne!(m1, m2);
    assert_ne!(m1, m3);
    assert_ne!(m1, m4);
    assert_ne!(m2, m3);
  }

  #[test]
  fn matchingrules_merge() {
    let mut m1 = MatchingRules::default();
    let m2 = matchingrules!{
      "body" => {
        "$.a.b" => [ MatchingRule::Type ]
      }
    };
    let m3 = matchingrules!{
      "body" => {
        "$.a.c" => [ MatchingRule::Equality ]
      }
    };
    let m4 = matchingrules!{
      "header" => {
        "$.x-test" => [ MatchingRule::Regex(".*".to_owned()) ]
      }
    };

    assert_eq!(m1.rules.len(), 0);
    assert_eq!(m1.rules.values().map(|v| v.rules.len()).sum::<usize>(), 0);
    m1.merge(&m2);
    assert_eq!(m1.rules.len(), 1);
    assert_eq!(m1.rules.values().map(|v| v.rules.len()).sum::<usize>(), 1);
    m1.merge(&m3);
    assert_eq!(m1.rules.len(), 1);
    assert_eq!(m1.rules.values().map(|v| v.rules.len()).sum::<usize>(), 2);
    m1.merge(&m4);
    assert_eq!(m1.rules.len(), 2);
    assert_eq!(m1.rules.values().map(|v| v.rules.len()).sum::<usize>(), 3);

    assert_eq!(
      m1,
      matchingrules!{
        "body" => {
          "$.a.b" => [ MatchingRule::Type ],
          "$.a.c" => [ MatchingRule::Equality ]
        },
        "header" => {
          "$.x-test" => [ MatchingRule::Regex(".*".to_owned()) ]
        }
      }
    )
  }
}
