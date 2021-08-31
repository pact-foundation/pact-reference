//! The `pact_matching` crate provides the core logic to performing matching on HTTP requests
//! and responses. It implements the V3 Pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! ## To use it
//!
//! To use it, add it to your dependencies in your cargo manifest.
//!
//! This crate provides two functions: [`match_request`](fn.match_request.html) and [`match_response`](fn.match_response.html).
//! These functions take an expected and actual request or response
//! model from the [`models`)(models/index.html) module, and return a vector of mismatches.
//!
//! To compare any incoming request, it first needs to be converted to a [`models::Request`](models/struct.Request.html) and then can be compared. Same for
//! any response.
//!
//! ## Reading and writing Pact files
//!
//! The [`Pact`](models/struct.Pact.html) struct in the [`models`)(models/index.html) module has methods to read and write pact JSON files. It supports all the specification
//! versions up to V3, but will converted a V1 and V1.1 spec file to a V2 format.
//!
//! ## Matching request and response parts
//!
//! V3 specification matching is supported for both JSON and XML bodies, headers, query strings and request paths.
//!
//! To understand the basic rules of matching, see [Matching Gotchas](https://github.com/realestate-com-au/pact/wiki/Matching-gotchas).
//! For example test cases for matching, see the [Pact Specification Project, version 3](https://github.com/bethesque/pact-specification/tree/version-3).
//!
//! By default, Pact will use string equality matching following Postel's Law. This means
//! that for an actual value to match an expected one, they both must consist of the same
//! sequence of characters. For collections (basically Maps and Lists), they must have the
//! same elements that match in the same sequence, with cases where the additional elements
//! in an actual Map are ignored.
//!
//! Matching rules can be defined for both request and response elements based on a pseudo JSON-Path
//! syntax.
//!
//! ### Matching Bodies
//!
//! For the most part, matching involves matching request and response bodies in JSON or XML format.
//! Other formats will either have their own matching rules, or will follow the JSON one.
//!
//! #### JSON body matching rules
//!
//! Bodies consist of Objects (Maps of Key-Value pairs), Arrays (Lists) and values (Strings, Numbers, true, false, null).
//! Body matching rules are prefixed with `$`.
//!
//! The following method is used to determine if two bodies match:
//!
//! 1. If both the actual body and expected body are empty, the bodies match.
//! 2. If the actual body is non-empty, and the expected body empty, the bodies match.
//! 3. If the actual body is empty, and the expected body non-empty, the bodies don't match.
//! 4. Otherwise do a comparison on the contents of the bodies.
//!
//! ##### For the body contents comparison:
//!
//! 1. If the actual and expected values are both Objects, compare as Maps.
//! 2. If the actual and expected values are both Arrays, compare as Lists.
//! 3. If the expected value is an Object, and the actual is not, they don't match.
//! 4. If the expected value is an Array, and the actual is not, they don't match.
//! 5. Otherwise, compare the values
//!
//! ##### For comparing Maps
//!
//! 1. If the actual map is non-empty while the expected is empty, they don't match.
//! 2. If we allow unexpected keys, and the number of expected keys is greater than the actual keys,
//! they don't match.
//! 3. If we don't allow unexpected keys, and the expected and actual maps don't have the
//! same number of keys, they don't match.
//! 4. Otherwise, for each expected key and value pair:
//!     1. if the actual map contains the key, compare the values
//!     2. otherwise they don't match
//!
//! Postel's law governs if we allow unexpected keys or not.
//!
//! ##### For comparing lists
//!
//! 1. If there is a body matcher defined that matches the path to the list, default
//! to that matcher and then compare the list contents.
//! 2. If the expected list is empty and the actual one is not, the lists don't match.
//! 3. Otherwise
//!     1. compare the list sizes
//!     2. compare the list contents
//!
//! ###### For comparing list contents
//!
//! 1. For each value in the expected list:
//!     1. If the index of the value is less than the actual list's size, compare the value
//!        with the actual value at the same index using the method for comparing values.
//!     2. Otherwise the value doesn't match
//!
//! ##### For comparing values
//!
//! 1. If there is a matcher defined that matches the path to the value, default to that
//! matcher
//! 2. Otherwise compare the values using equality.
//!
//! #### XML body matching rules
//!
//! Bodies consist of a root element, Elements (Lists with children), Attributes (Maps) and values (Strings).
//! Body matching rules are prefixed with `$`.
//!
//! The following method is used to determine if two bodies match:
//!
//! 1. If both the actual body and expected body are empty, the bodies match.
//! 2. If the actual body is non-empty, and the expected body empty, the bodies match.
//! 3. If the actual body is empty, and the expected body non-empty, the bodies don't match.
//! 4. Otherwise do a comparison on the contents of the bodies.
//!
//! ##### For the body contents comparison:
//!
//! Start by comparing the root element.
//!
//! ##### For comparing elements
//!
//! 1. If there is a body matcher defined that matches the path to the element, default
//! to that matcher on the elements name or children.
//! 2. Otherwise the elements match if they have the same name.
//!
//! Then, if there are no mismatches:
//!
//! 1. compare the attributes of the element
//! 2. compare the child elements
//! 3. compare the text nodes
//!
//! ##### For comparing attributes
//!
//! Attributes are treated as a map of key-value pairs.
//!
//! 1. If the actual map is non-empty while the expected is empty, they don't match.
//! 2. If we allow unexpected keys, and the number of expected keys is greater than the actual keys,
//! they don't match.
//! 3. If we don't allow unexpected keys, and the expected and actual maps don't have the
//! same number of keys, they don't match.
//!
//! Then, for each expected key and value pair:
//!
//! 1. if the actual map contains the key, compare the values
//! 2. otherwise they don't match
//!
//! Postel's law governs if we allow unexpected keys or not. Note for matching paths, attribute names are prefixed with an `@`.
//!
//! ###### For comparing child elements
//!
//! 1. If there is a matcher defined for the path to the child elements, then pad out the expected child elements to have the
//! same size as the actual child elements.
//! 2. Otherwise
//!     1. If the actual children is non-empty while the expected is empty, they don't match.
//!     2. If we allow unexpected keys, and the number of expected children is greater than the actual children,
//!     they don't match.
//!     3. If we don't allow unexpected keys, and the expected and actual children don't have the
//!     same number of elements, they don't match.
//!
//! Then, for each expected and actual element pair, compare them using the rules for comparing elements.
//!
//! ##### For comparing text nodes
//!
//! Text nodes are combined into a single string and then compared as values.
//!
//! 1. If there is a matcher defined that matches the path to the text node (text node paths end with `#text`), default to that
//! matcher
//! 2. Otherwise compare the text using equality.
//!
//!
//! ##### For comparing values
//!
//! 1. If there is a matcher defined that matches the path to the value, default to that
//! matcher
//! 2. Otherwise compare the values using equality.
//!
//! ### Matching Paths
//!
//! Paths are matched by the following:
//!
//! 1. If there is a matcher defined for `path`, default to that matcher.
//! 2. Otherwise paths are compared as Strings
//!
//! ### Matching Queries
//!
//! 1. If the actual and expected query strings are empty, they match.
//! 2. If the actual is not empty while the expected is, they don't match.
//! 3. If the actual is empty while the expected is not, they don't match.
//! 4. Otherwise convert both into a Map of keys mapped to a list values, and compare those.
//!
//! #### Matching Query Maps
//!
//! Query strings are parsed into a Map of keys mapped to lists of values. Key value
//! pairs can be in any order, but when the same key appears more than once the values
//! are compared in the order they appear in the query string.
//!
//! ### Matching Headers
//!
//! 1. Do a case-insensitive sort of the headers by keys
//! 2. For each expected header in the sorted list:
//!     1. If the actual headers contain that key, compare the header values
//!     2. Otherwise the header does not match
//!
//! For matching header values:
//!
//! 1. If there is a matcher defined for `header.<HEADER_KEY>`, default to that matcher
//! 2. Otherwise strip all whitespace after commas and compare the resulting strings.
//!
//! #### Matching Request Headers
//!
//! Request headers are matched by excluding the cookie header.
//!
//! #### Matching Request cookies
//!
//! If the list of expected cookies contains all the actual cookies, the cookies match.
//!
//! ### Matching Status Codes
//!
//! Status codes are compared as integer values.
//!
//! ### Matching HTTP Methods
//!
//! The actual and expected methods are compared as case-insensitive strings.
//!
//! ## Matching Rules
//!
//! Pact supports extending the matching rules on each type of object (Request or Response) with a `matchingRules` element in the pact file.
//! This is a map of JSON path strings to a matcher. When an item is being compared, if there is an entry in the matching
//! rules that corresponds to the path to the item, the comparison will be delegated to the defined matcher. Note that the
//! matching rules cascade, so a rule can be specified on a value and will apply to all children of that value.
//!
//! ## Matcher Path expressions
//!
//! Pact does not support the full JSON path expressions, only ones that match the following rules:
//!
//! 1. All paths start with a dollar (`$`), representing the root.
//! 2. All path elements are separated by periods (`.`), except array indices which use square brackets (`[]`).
//! 3. Path elements represent keys.
//! 4. A star (`*`) can be used to match all keys of a map or all items of an array (one level only).
//!
//! So the expression `$.item1.level[2].id` will match the highlighted item in the following body:
//!
//! ```js,ignore
//! {
//!   "item1": {
//!     "level": [
//!       {
//!         "id": 100
//!       },
//!       {
//!         "id": 101
//!       },
//!       {
//!         "id": 102 // <---- $.item1.level[2].id
//!       },
//!       {
//!         "id": 103
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! while `$.*.level[*].id` will match all the ids of all the levels for all items.
//!
//! ### Matcher selection algorithm
//!
//! Due to the star notation, there can be multiple matcher paths defined that correspond to an item. The first, most
//! specific expression is selected by assigning weightings to each path element and taking the product of the weightings.
//! The matcher with the path with the largest weighting is used.
//!
//! * The root node (`$`) is assigned the value 2.
//! * Any path element that does not match is assigned the value 0.
//! * Any property name that matches a path element is assigned the value 2.
//! * Any array index that matches a path element is assigned the value 2.
//! * Any star (`*`) that matches a property or array index is assigned the value 1.
//! * Everything else is assigned the value 0.
//!
//! So for the body with highlighted item:
//!
//! ```js,ignore
//! {
//!   "item1": {
//!     "level": [
//!       {
//!         "id": 100
//!       },
//!       {
//!         "id": 101
//!       },
//!       {
//!         "id": 102 // <--- Item under consideration
//!       },
//!       {
//!         "id": 103
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! The expressions will have the following weightings:
//!
//! | expression | weighting calculation | weighting |
//! |------------|-----------------------|-----------|
//! | $ | $(2) | 2 |
//! | $.item1 | $(2).item1(2) | 4 |
//! | $.item2 | $(2).item2(0) | 0 |
//! | $.item1.level | $(2).item1(2).level(2) | 8 |
//! | $.item1.level\[1\] | $(2).item1(2).level(2)\[1(2)\] | 16 |
//! | $.item1.level\[1\].id | $(2).item1(2).level(2)\[1(2)\].id(2) | 32 |
//! | $.item1.level\[1\].name | $(2).item1(2).level(2)\[1(2)\].name(0) | 0 |
//! | $.item1.level\[2\] | $(2).item1(2).level(2)\[2(0)\] | 0 |
//! | $.item1.level\[2\].id | $(2).item1(2).level(2)\[2(0)\].id(2) | 0 |
//! | $.item1.level\[*\].id | $(2).item1(2).level(2)\[*(1)\].id(2) | 16 |
//! | $.\*.level\[\*\].id | $(2).*(1).level(2)\[*(1)\].id(2) | 8 |
//!
//! So for the item with id 102, the matcher with path `$.item1.level\[1\].id` and weighting 32 will be selected.
//!
//! ## Supported matchers
//!
//! The following matchers are supported:
//!
//! | matcher | example configuration | description |
//! |---------|-----------------------|-------------|
//! | Equality | `{ "match": "equality" }` | This is the default matcher, and relies on the equals operator |
//! | Regex | `{ "match": "regex", "regex": "\\d+" }` | This executes a regular expression match against the string representation of a values. |
//! | Type | `{ "match": "type" }` | This executes a type based match against the values, that is, they are equal if they are the same type. |
//! | MinType | `{ "match": "type", "min": 2 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum. |
//! | MaxType | `{ "match": "type", "max": 10 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the maximum. |
//! | MinMaxType | `{ "match": "type", "min": 1, "max": 10 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum and maximum. |
//! | Timestamp | `{ "match": "timestamp", "timestamp": "yyyy-MM-dd HH:mm:ssZZZZZ" }` | Matches a string value against a Date/Time pattern. |
//! | Time | `{ "match": "time", "time": "HH:mm:ssZZZZZ" }` | Matches a string value against a Time pattern. |
//! | Date | `{ "match": "date", "date": "yyyy-MM-dd" }` | Matches a string value against a Date pattern. |
//! | Include | `{ "match": "include", "value": "ello" }` | Checks if a string value contains the given sub-string. |
//! | Number | `{ "match": "number" }` | Matches any numeric type. |
//! | Integer | `{ "match": "integer" }` | Matches a number if it has no digits after the decimal point. |
//! | Decimal | `{ "match": "decimal" }` | Matches a number if it has at least one digit after the decimal point. |
//! | Null | `{ "match": "null" }` | Matches a JSON NULL value. This only makes sense to use with JSON. |
//! | ContentType | `{ "match": "contentType", "value": "image/jpeg" }` | Checks if the value has the content type of the privided value. This is done by performing a magic test on the first few bytes of the value. |
//! | ArrayContains | `{ "match": "arrayContains", "variants": [...] }` | Checks if all the variants are present in an array. |

#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fmt::Formatter;
use std::hash::Hash;
use std::str;
use std::str::from_utf8;

use ansi_term::*;
use ansi_term::Colour::*;
use bytes::Bytes;
use lazy_static::*;
use log::*;
use maplit::hashmap;
use serde_json::{json, Value};

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{apply_generators, GenerateValue, GeneratorCategory, GeneratorTestMode, VariantMatcher};
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleList};
use pact_models::PactSpecification;
use pact_models::request::Request;
use pact_models::response::Response;

use crate::headers::{match_header_value, match_headers};
use crate::matchers::*;
use crate::generators::{DefaultVariantMatcher, generators_process_body};
use crate::matchingrules::DisplayForMismatch;

/// Simple macro to convert a string slice to a `String` struct.
#[macro_export]
macro_rules! s {
    ($e:expr) => ($e.to_string())
}


/// Version of the library
pub const PACT_RUST_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

mod matchers;
pub mod json;
mod xml;
mod binary_utils;
mod headers;
pub mod logging;
mod matchingrules;
mod generators;

#[derive(Debug, Clone)]
/// Context used to apply matching logic
pub struct MatchingContext {
  /// Matching rules that apply when matching with the context
  pub matchers: MatchingRuleCategory,
  /// Configuration to apply when matching with the context
  pub config: DiffConfig,
  /// Specification version to apply when matching with the context
  pub matching_spec: PactSpecification
}

impl MatchingContext {
  /// Creates a new context with the given config and matching rules
  pub fn new(config: DiffConfig, matchers: &MatchingRuleCategory) -> Self {
    MatchingContext {
      matchers: matchers.clone(),
      config: config.clone(),
      .. MatchingContext::default()
    }
  }

  /// Creates a new empty context with the given config
  pub fn with_config(config: DiffConfig) -> Self {
    MatchingContext {
      config: config.clone(),
      .. MatchingContext::default()
    }
  }

  /// Clones the current context with the provided matching rules
  pub fn clone_with(&self, matchers: &MatchingRuleCategory) -> Self {
    MatchingContext {
      matchers: matchers.clone(),
      config: self.config.clone(),
      matching_spec: self.matching_spec.clone()
    }
  }

  /// If there is a matcher defined at the path in this context
  pub fn matcher_is_defined(&self, path: &[&str]) -> bool {
    self.matchers.matcher_is_defined(path)
  }

  /// Selected the best matcher from the context for the given path
  pub fn select_best_matcher(&self, path: &[&str]) -> RuleList {
    self.matchers.select_best_matcher(path)
  }

  /// If there is a wildcard matcher defined at the path in this context
  #[deprecated(since = "0.8.12", note = "Replaced with values matcher")]
  pub fn wildcard_matcher_is_defined(&self, path: &[&str]) -> bool {
    !self.matchers_for_exact_path(path).filter(|&(val, _)| val.is_wildcard()).is_empty()
  }

  fn matchers_for_exact_path(&self, path: &[&str]) -> MatchingRuleCategory {
    match self.matchers.name {
      Category::HEADER | Category::QUERY => self.matchers.filter(|&(val, _)| {
        path.len() == 1 && Some(path[0]) == val.first_field()
      }),
      Category::BODY => self.matchers.filter(|&(val, _)| {
        val.matches_path_exactly(path)
      }),
      _ => self.matchers.filter(|_| false)
    }
  }

  /// If there is a type matcher defined at the path in this context
  pub fn type_matcher_defined(&self, path: &[&str]) -> bool {
    self.matchers.resolve_matchers_for_path(path).type_matcher_defined()
  }

  /// If there is a values matcher defined at the path in this context
  pub fn values_matcher_defined(&self, path: &[&str]) -> bool {
    self.matchers_for_exact_path(path).values_matcher_defined()
  }

  /// Matches the keys of the expected and actual maps
  pub fn match_keys<T: Display + Debug>(&self, path: &[&str], expected: &HashMap<String, T>, actual: &HashMap<String, T>) -> Result<(), Vec<Mismatch>> {
    let mut expected_keys = expected.keys().cloned().collect::<Vec<String>>();
    expected_keys.sort();
    let mut actual_keys = actual.keys().cloned().collect::<Vec<String>>();
    actual_keys.sort();
    let missing_keys: Vec<String> = expected.keys().filter(|key| !actual.contains_key(*key)).cloned().collect();
    match self.config {
      DiffConfig::AllowUnexpectedKeys if !missing_keys.is_empty() => {
        Err(vec![Mismatch::BodyMismatch {
          path: path.join("."),
          expected: Some(expected.for_mismatch().into()),
          actual: Some(actual.for_mismatch().into()),
          mismatch: format!("Actual map is missing the following keys: {}", missing_keys.join(", ")),
        }])
      }
      DiffConfig::NoUnexpectedKeys if expected_keys != actual_keys => {
        Err(vec![Mismatch::BodyMismatch {
          path: path.join("."),
          expected: Some(expected.for_mismatch().into()),
          actual: Some(actual.for_mismatch().into()),
          mismatch: format!("Expected a Map with keys {} but received one with keys {}",
                            expected_keys.join(", "), actual_keys.join(", ")),
        }])
      }
      _ => Ok(())
    }
  }
}

impl Default for MatchingContext {
  fn default() -> Self {
    MatchingContext {
      matchers: Default::default(),
      config: DiffConfig::AllowUnexpectedKeys,
      matching_spec: PactSpecification::V3
    }
  }
}

lazy_static! {
  static ref BODY_MATCHERS: [
    (fn(content_type: &ContentType) -> bool,
    fn(expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> Result<(), Vec<Mismatch>>); 4]
     = [
      (|content_type| { content_type.is_json() }, json::match_json),
      (|content_type| { content_type.is_xml() }, xml::match_xml),
      (|content_type| { content_type.base_type() == "application/octet-stream" }, binary_utils::match_octet_stream),
      (|content_type| { content_type.base_type() == "multipart/form-data" }, binary_utils::match_mime_multipart)
  ];
}

/// Enum that defines the different types of mismatches that can occur.
#[derive(Debug, Clone)]
pub enum Mismatch {
    /// Request Method mismatch
    MethodMismatch {
        /// Expected request method
        expected: String,
        /// Actual request method
        actual: String
    },
    /// Request Path mismatch
    PathMismatch {
        /// expected request path
        expected: String,
        /// actual request path
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Response status mismatch
    StatusMismatch {
        /// expected response status
      expected: u16,
      /// actual response status
      actual: u16,
      /// description of the mismatch
      mismatch: String
    },
    /// Request query mismatch
    QueryMismatch {
        /// query parameter name
        parameter: String,
        /// expected value
        expected: String,
        /// actual value
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Header mismatch
    HeaderMismatch {
        /// header key
        key: String,
        /// expected value
        expected: String,
        /// actual value
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Mismatch in the content type of the body
    BodyTypeMismatch {
      /// expected content type of the body
      expected: String,
      /// actual content type of the body
      actual: String,
      /// description of the mismatch
      mismatch: String,
      /// expected value
      expected_body: Option<Bytes>,
      /// actual value
      actual_body: Option<Bytes>
    },
    /// Body element mismatch
    BodyMismatch {
      /// path expression to where the mismatch occurred
      path: String,
      /// expected value
      expected: Option<Bytes>,
      /// actual value
      actual: Option<Bytes>,
      /// description of the mismatch
      mismatch: String
    },
    /// Message metadata mismatch
    MetadataMismatch {
      /// key
      key: String,
      /// expected value
      expected: String,
      /// actual value
      actual: String,
      /// description of the mismatch
      mismatch: String
    }
}

impl Mismatch {
  /// Converts the mismatch to a `Value` struct.
  pub fn to_json(&self) -> serde_json::Value {
    match self {
      Mismatch::MethodMismatch { expected: e, actual: a } => {
        json!({
          "type" : "MethodMismatch",
          "expected" : e,
          "actual" : a
        })
      },
      Mismatch::PathMismatch { expected: e, actual: a, mismatch: m } => {
        json!({
          "type" : "PathMismatch",
          "expected" : e,
          "actual" : a,
          "mismatch" : m
        })
      },
      Mismatch::StatusMismatch { expected: e, actual: a, mismatch: m } => {
        json!({
          "type" : "StatusMismatch",
          "expected" : e,
          "actual" : a,
          "mismatch": m
        })
      },
      Mismatch::QueryMismatch { parameter: p, expected: e, actual: a, mismatch: m } => {
        json!({
          "type" : "QueryMismatch",
          "parameter" : p,
          "expected" : e,
          "actual" : a,
          "mismatch" : m
        })
      },
      Mismatch::HeaderMismatch { key: k, expected: e, actual: a, mismatch: m } => {
        json!({
          "type" : "HeaderMismatch",
          "key" : k,
          "expected" : e,
          "actual" : a,
          "mismatch" : m
        })
      },
      Mismatch::BodyTypeMismatch {
        expected,
        actual,
        mismatch,
        expected_body,
        actual_body
      } => {
        json!({
          "type" : "BodyTypeMismatch",
          "expected" : expected,
          "actual" : actual,
          "mismatch" : mismatch,
          "expectedBody": match expected_body {
            Some(v) => serde_json::Value::String(str::from_utf8(v)
              .unwrap_or("ERROR: could not convert to UTF-8 from bytes").into()),
            None => serde_json::Value::Null
          },
          "actualBody": match actual_body {
            Some(v) => serde_json::Value::String(str::from_utf8(v)
              .unwrap_or("ERROR: could not convert to UTF-8 from bytes").into()),
            None => serde_json::Value::Null
          }
        })
      },
      Mismatch::BodyMismatch { path, expected, actual, mismatch } => {
        json!({
          "type" : "BodyMismatch",
          "path" : path,
          "expected" : match expected {
            Some(v) => serde_json::Value::String(str::from_utf8(v).unwrap_or("ERROR: could not convert from bytes").into()),
            None => serde_json::Value::Null
          },
          "actual" : match actual {
            Some(v) => serde_json::Value::String(str::from_utf8(v).unwrap_or("ERROR: could not convert from bytes").into()),
            None => serde_json::Value::Null
          },
          "mismatch" : mismatch
        })
      }
      Mismatch::MetadataMismatch { key, expected, actual, mismatch } => {
        json!({
          "type" : "MetadataMismatch",
          "key" : key,
          "expected" : expected,
          "actual" : actual,
          "mismatch" : mismatch
        })
      }
    }
  }

    /// Returns the type of the mismatch as a string
    pub fn mismatch_type(&self) -> &str {
      match *self {
        Mismatch::MethodMismatch { .. } => "MethodMismatch",
        Mismatch::PathMismatch { .. } => "PathMismatch",
        Mismatch::StatusMismatch { .. } => "StatusMismatch",
        Mismatch::QueryMismatch { .. } => "QueryMismatch",
        Mismatch::HeaderMismatch { .. } => "HeaderMismatch",
        Mismatch::BodyTypeMismatch { .. } => "BodyTypeMismatch",
        Mismatch::BodyMismatch { .. } => "BodyMismatch",
        Mismatch::MetadataMismatch { .. } => "MetadataMismatch"
      }
    }

    /// Returns a summary string for this mismatch
    pub fn summary(&self) -> String {
      match *self {
        Mismatch::MethodMismatch { expected: ref e, .. } => format!("is a {} request", e),
        Mismatch::PathMismatch { expected: ref e, .. } => format!("to path '{}'", e),
        Mismatch::StatusMismatch { expected: ref e, .. } => format!("has status code {}", e),
        Mismatch::QueryMismatch { ref parameter, expected: ref e, .. } => format!("includes parameter '{}' with value '{}'", parameter, e),
        Mismatch::HeaderMismatch { ref key, expected: ref e, .. } => format!("includes header '{}' with value '{}'", key, e),
        Mismatch::BodyTypeMismatch { .. } => s!("has a matching body"),
        Mismatch::BodyMismatch { .. } => s!("has a matching body"),
        Mismatch::MetadataMismatch { .. } => s!("has matching metadata")
      }
    }

    /// Returns a formated string for this mismatch
    pub fn description(&self) -> String {
      match self {
        Mismatch::MethodMismatch { expected: e, actual: a } => format!("expected {} but was {}", e, a),
        Mismatch::PathMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::StatusMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::QueryMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::HeaderMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::BodyTypeMismatch {  expected: e, actual: a, .. } => format!("expected '{}' body but was '{}'", e, a),
        Mismatch::BodyMismatch { path, mismatch, .. } => format!("{} -> {}", path, mismatch),
        Mismatch::MetadataMismatch { mismatch, .. } => mismatch.clone()
      }
    }

    /// Returns a formatted string with ansi escape codes for this mismatch
    pub fn ansi_description(&self) -> String {
      match self {
        Mismatch::MethodMismatch { expected: e, actual: a } => format!("expected {} but was {}", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::PathMismatch { expected: e, actual: a, .. } => format!("expected '{}' but was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::StatusMismatch { expected: e, actual: a, .. } => format!("expected {} but was {}", Red.paint(e.to_string()), Green.paint(a.to_string())),
        Mismatch::QueryMismatch { expected: e, actual: a, parameter: p, .. } => format!("Expected '{}' but received '{}' for query parameter '{}'",
          Red.paint(e.to_string()), Green.paint(a.to_string()), Style::new().bold().paint(p.clone())),
        Mismatch::HeaderMismatch { expected: e, actual: a, key: k, .. } => format!("Expected header '{}' to have value '{}' but was '{}'",
          Style::new().bold().paint(k.clone()), Red.paint(e.to_string()), Green.paint(a.to_string())),
        Mismatch::BodyTypeMismatch {  expected: e, actual: a, .. } => format!("expected '{}' body but was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::BodyMismatch { path, mismatch, .. } => format!("{} -> {}", Style::new().bold().paint(path.clone()), mismatch),
        Mismatch::MetadataMismatch { expected: e, actual: a, key: k, .. } => format!("Expected message metadata '{}' to have value '{}' but was '{}'",
          Style::new().bold().paint(k.clone()), Red.paint(e.to_string()), Green.paint(a.to_string()))
      }
    }
}

impl PartialEq for Mismatch {
  fn eq(&self, other: &Mismatch) -> bool {
    match (self, other) {
      (Mismatch::MethodMismatch { expected: e1, actual: a1 },
        Mismatch::MethodMismatch { expected: e2, actual: a2 }) => {
        e1 == e2 && a1 == a2
      },
      (Mismatch::PathMismatch { expected: e1, actual: a1, .. },
        Mismatch::PathMismatch { expected: e2, actual: a2, .. }) => {
        e1 == e2 && a1 == a2
      },
      (Mismatch::StatusMismatch { expected: e1, actual: a1, .. },
        Mismatch::StatusMismatch { expected: e2, actual: a2, .. }) => {
        e1 == e2 && a1 == a2
      },
      (Mismatch::BodyTypeMismatch { expected: e1, actual: a1, .. },
        Mismatch::BodyTypeMismatch { expected: e2, actual: a2, .. }) => {
        e1 == e2 && a1 == a2
      },
      (Mismatch::QueryMismatch { parameter: p1, expected: e1, actual: a1, .. },
        Mismatch::QueryMismatch { parameter: p2, expected: e2, actual: a2, .. }) => {
        p1 == p2 && e1 == e2 && a1 == a2
      },
      (Mismatch::HeaderMismatch { key: p1, expected: e1, actual: a1, .. },
        Mismatch::HeaderMismatch { key: p2, expected: e2, actual: a2, .. }) => {
        p1 == p2 && e1 == e2 && a1 == a2
      },
      (Mismatch::BodyMismatch { path: p1, expected: e1, actual: a1, .. },
        Mismatch::BodyMismatch { path: p2, expected: e2, actual: a2, .. }) => {
        p1 == p2 && e1 == e2 && a1 == a2
      },
      (Mismatch::MetadataMismatch { key: p1, expected: e1, actual: a1, .. },
        Mismatch::MetadataMismatch { key: p2, expected: e2, actual: a2, .. }) => {
        p1 == p2 && e1 == e2 && a1 == a2
      },
      (_, _) => false
    }
  }
}

impl Display for Mismatch {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.description())
  }
}

fn merge_result(res1: Result<(), Vec<Mismatch>>, res2: Result<(), Vec<Mismatch>>) -> Result<(), Vec<Mismatch>> {
  match (&res1, &res2) {
    (Ok(_), Ok(_)) => res1.clone(),
    (Err(_), Ok(_)) => res1.clone(),
    (Ok(_), Err(_)) => res2.clone(),
    (Err(m1), Err(m2)) => {
      let mut mismatches = m1.clone();
      mismatches.extend_from_slice(&*m2);
      Err(mismatches)
    }
  }
}

/// Result of matching a request body
#[derive(Debug, Clone, PartialEq)]
pub enum BodyMatchResult {
  /// Matched OK
  Ok,
  /// Mismatch in the content type of the body
  BodyTypeMismatch {
    /// Expected content type
    expected_type: String,
    /// Actual content type
    actual_type: String,
    /// Message
    message: String,
    /// Expected body
    expected: Option<Bytes>,
    /// Actual body
    actual: Option<Bytes>
  },
  /// Mismatches with the body contents
  BodyMismatches(HashMap<String, Vec<Mismatch>>)
}

impl BodyMatchResult {
  /// Returns all the mismatches
  pub fn mismatches(&self) -> Vec<Mismatch> {
    match self {
      BodyMatchResult::BodyTypeMismatch { expected_type, actual_type, message, expected, actual } => {
        vec![Mismatch::BodyTypeMismatch {
          expected: expected_type.clone(),
          actual: actual_type.clone(),
          mismatch: message.clone(),
          expected_body: expected.clone(),
          actual_body: actual.clone()
        }]
      },
      BodyMatchResult::BodyMismatches(results) =>
        results.values().flatten().cloned().collect(),
      _ => vec![]
    }
  }

  /// If all the things matched OK
  pub fn all_matched(&self) -> bool {
    match self {
      BodyMatchResult::BodyTypeMismatch { .. } => false,
      BodyMatchResult::BodyMismatches(results) =>
        results.values().all(|m| m.is_empty()),
      _ => true
    }
  }
}

/// Result of matching a request
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMatchResult {
  /// Method match result
  pub method: Option<Mismatch>,
  /// Path match result
  pub path: Option<Vec<Mismatch>>,
  /// Body match result
  pub body: BodyMatchResult,
  /// Query parameter result
  pub query: HashMap<String, Vec<Mismatch>>,
  /// Headers result
  pub headers: HashMap<String, Vec<Mismatch>>
}

impl RequestMatchResult {
  /// Returns all the mismatches
  pub fn mismatches(&self) -> Vec<Mismatch> {
    let mut m = vec![];

    if let Some(ref mismatch) = self.method {
      m.push(mismatch.clone());
    }
    if let Some(ref mismatches) = self.path {
      m.extend_from_slice(mismatches.as_slice());
    }
    for mismatches in self.query.values() {
      m.extend_from_slice(mismatches.as_slice());
    }
    for mismatches in self.headers.values() {
      m.extend_from_slice(mismatches.as_slice());
    }
    m.extend_from_slice(self.body.mismatches().as_slice());

    m
  }

  /// Returns a score based on what was matched
  pub fn score(&self) -> i8 {
    let mut score = 0;
    if self.method.is_none() {
      score += 1;
    } else {
      score -= 1;
    }
    if self.path.is_none() {
      score += 1
    } else {
      score -= 1
    }
    for (_, mismatches) in &self.query {
      if mismatches.is_empty() {
        score += 1;
      } else {
        score -= 1;
      }
    }
    for (_, mismatches) in &self.headers {
      if mismatches.is_empty() {
        score += 1;
      } else {
        score -= 1;
      }
    }
    match &self.body {
      BodyMatchResult::BodyTypeMismatch { .. } => {
        score -= 1;
      },
      BodyMatchResult::BodyMismatches(results) => {
        for (_, mismatches) in results {
          if mismatches.is_empty() {
            score += 1;
          } else {
            score -= 1;
          }
        }
      },
      _ => ()
    }
    score
  }

  /// If all the things matched OK
  pub fn all_matched(&self) -> bool {
    self.method.is_none() && self.path.is_none() &&
      self.query.values().all(|m| m.is_empty()) &&
      self.headers.values().all(|m| m.is_empty()) &&
      self.body.all_matched()
  }

  /// If there was a mismatch with the method or path
  pub fn method_or_path_mismatch(&self) -> bool {
    self.method.is_some() || self.path.is_some()
  }
}

/// Enum that defines the configuration options for performing a match.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffConfig {
    /// If unexpected keys are allowed and ignored during matching.
    AllowUnexpectedKeys,
    /// If unexpected keys cause a mismatch.
    NoUnexpectedKeys
}

/// Matches the actual text body to the expected one.
pub fn match_text(expected: &Option<Bytes>, actual: &Option<Bytes>, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = vec!["$"];
  if context.matcher_is_defined(&path) {
    let mut mismatches = vec![];
    let empty = Bytes::default();
    let expected_str = match from_utf8(expected.as_ref().unwrap_or_else(|| &empty)) {
      Ok(expected) => expected,
      Err(err) => {
        mismatches.push(Mismatch::BodyMismatch {
          path: "$".to_string(),
          expected: expected.clone(),
          actual: actual.clone(),
          mismatch: format!("Could not parse expected value as UTF-8 text: {}", err)
        });
        ""
      }
    };
    let actual_str = match from_utf8(actual.as_ref().unwrap_or_else(|| &empty)) {
      Ok(actual) => actual,
      Err(err) => {
        mismatches.push(Mismatch::BodyMismatch {
          path: "$".to_string(),
          expected: expected.clone(),
          actual: actual.clone(),
          mismatch: format!("Could not parse actual value as UTF-8 text: {}", err)
        });
        ""
      }
    };
    if let Err(messages) = match_values(&path, context, expected_str, actual_str) {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: "$".to_string(),
          expected: expected.clone(),
          actual: actual.clone(),
          mismatch: message.clone()
        })
      }
    };
    if mismatches.is_empty() {
      Ok(())
    } else {
      Err(mismatches)
    }
  } else if expected != actual {
    Err(vec![ Mismatch::BodyMismatch { path: "$".to_string(), expected: expected.clone(),
      actual: actual.clone(),
      mismatch: format!("Expected text '{:?}' but received '{:?}'", expected, actual) } ])
  } else {
    Ok(())
  }
}

/// Matches the actual request method to the expected one.
pub fn match_method(expected: &String, actual: &String) -> Result<(), Mismatch> {
  if expected.to_lowercase() != actual.to_lowercase() {
    Err(Mismatch::MethodMismatch { expected: expected.clone(), actual: actual.clone() })
  } else {
    Ok(())
  }
}

/// Matches the actual request path to the expected one.
pub fn match_path(expected: &String, actual: &String, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = vec![];
  let matcher_result = if context.matcher_is_defined(&path) {
    match_values(&path, context, expected.clone(), actual.clone())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err| vec![err])
      .map_err(|errors| errors.iter().map(|err| err.to_string()).collect())
  };
  matcher_result.map_err(|messages| messages.iter().map(|message| {
    Mismatch::PathMismatch {
      expected: expected.to_string(),
      actual: actual.to_string(), mismatch: message.clone()
    }
  }).collect())
}

fn compare_query_parameter_value(key: &String, expected: &String, actual: &String, index: usize,
                                 context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let index = index.to_string();
  let path = vec!["$", key.as_str(), index.as_str()];
  let matcher_result = if context.matcher_is_defined(&path) {
    matchers::match_values(&path, context, expected.clone(), actual.clone())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false)
      .map_err(|error| vec![error.to_string()])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: expected.clone(),
        actual: actual.clone(),
        mismatch: message.clone(),
      }
    }).collect()
  })
}

fn compare_query_parameter_values(key: &String, expected: &Vec<String>, actual: &Vec<String>,
                                  context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let result: Vec<Mismatch> = expected.iter().enumerate().flat_map(|(index, val)| {
    if index < actual.len() {
      match compare_query_parameter_value(key, val, &actual[index], index, context) {
        Ok(_) => vec![],
        Err(errors) => errors
      }
    } else {
      vec![ Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
        mismatch: format!("Expected query parameter '{}' value '{}' but was missing", key, val)
      } ]
    }
  }).collect();

  if result.is_empty() {
    Ok(())
  } else {
    Err(result)
  }
}

fn match_query_values(key: &String, expected: &Vec<String>, actual: &Vec<String>, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  if expected.is_empty() && !actual.is_empty() {
    Err(vec![ Mismatch::QueryMismatch { parameter: key.clone(),
      expected: format!("{:?}", expected),
      actual: format!("{:?}", actual),
      mismatch: format!("Expected an empty parameter list for '{}' but received {:?}", key, actual) } ])
  } else {
    let mismatch = if expected.len() != actual.len() {
      Err(vec![ Mismatch::QueryMismatch { parameter: key.clone(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
        mismatch: format!(
          "Expected query parameter '{}' with {} value(s) but received {} value(s)",
          key, expected.len(), actual.len()) } ])
    } else {
      Ok(())
    };
    merge_result(compare_query_parameter_values(key, expected, actual, context), mismatch)
  }
}

fn match_query_maps(expected: HashMap<String, Vec<String>>, actual: HashMap<String, Vec<String>>, context: &MatchingContext) -> HashMap<String, Vec<Mismatch>> {
  let mut result: HashMap<String, Vec<Mismatch>> = hashmap!{};
  for (key, value) in &expected {
    match actual.get(key) {
      Some(actual_value) => {
        let matches = match_query_values(key, value, actual_value, context);
        let v = result.entry(key.clone()).or_default();
        v.extend(matches.err().unwrap_or_default());
      },
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch { parameter: key.clone(),
        expected: format!("{:?}", value),
        actual: "".to_string(),
        mismatch: format!("Expected query parameter '{}' but was missing", key) })
    }
  }
  for (key, value) in &actual {
    match expected.get(key) {
      Some(_) => (),
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch { parameter: key.clone(),
        expected: "".to_string(),
        actual: format!("{:?}", value),
        mismatch: format!("Unexpected query parameter '{}' received", key) })
    }
  }
  result
}

/// Matches the actual query parameters to the expected ones.
pub fn match_query(expected: Option<HashMap<String, Vec<String>>>, actual: Option<HashMap<String, Vec<String>>>, context: &MatchingContext) -> HashMap<String, Vec<Mismatch>> {
  match (actual, expected) {
    (Some(aqm), Some(eqm)) => match_query_maps(eqm, aqm, context),
    (Some(aqm), None) => aqm.iter().map(|(key, value)| {
      (key.clone(), vec![Mismatch::QueryMismatch { parameter: key.clone(),
        expected: "".to_string(),
        actual: format!("{:?}", value),
        mismatch: format!("Unexpected query parameter '{}' received", key) }])
    }).collect(),
    (None, Some(eqm)) => eqm.iter().map(|(key, value)| {
      (key.clone(), vec![Mismatch::QueryMismatch { parameter: key.clone(),
        expected: format!("{:?}", value),
        actual: "".to_string(),
        mismatch: format!("Expected query parameter '{}' but was missing", key) }])
    }).collect(),
    (None, None) => hashmap!{}
  }
}

fn group_by<I, F, K>(items: I, f: F) -> HashMap<K, Vec<I::Item>>
  where I: IntoIterator, F: Fn(&I::Item) -> K, K: Eq + Hash {
  let mut m = hashmap!{};
  for item in items {
    let key = f(&item);
    let values = m.entry(key).or_insert_with(|| vec![]);
    values.push(item);
  }
  m
}

fn compare_bodies(content_type: &ContentType, expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> BodyMatchResult {
  let mut mismatches = vec![];
  match BODY_MATCHERS.iter().find(|mt| mt.0(&content_type)) {
    Some(ref match_fn) => {
      debug!("Using body matcher for content type '{}'", content_type);
      if let Err(m) = match_fn.1(expected, actual, &context) {
        mismatches.extend_from_slice(&*m);
      }
    },
    None => {
      warn!("No body matcher defined for content type '{}', using plain text matcher", content_type);
      if let Err(m) = match_text(&expected.body().value(), &actual.body().value(), &context) {
        mismatches.extend_from_slice(&*m);
      }
    }
  };
  if mismatches.is_empty() {
    BodyMatchResult::Ok
  } else {
    BodyMatchResult::BodyMismatches(group_by(mismatches, |m| match m {
      Mismatch::BodyMismatch { path: m, ..} => m.to_string(),
      _ => String::default()
    }))
  }
}

fn match_body_content(content_type: &ContentType, expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> BodyMatchResult {
  let expected_body = expected.body();
  let actual_body = actual.body();
  match (expected_body, actual_body) {
    (&OptionalBody::Missing, _) => BodyMatchResult::Ok,
    (&OptionalBody::Null, &OptionalBody::Present(ref b, _)) => {
      BodyMatchResult::BodyMismatches(hashmap!{ "$".into() => vec![Mismatch::BodyMismatch { expected: None, actual: Some(b.clone()),
        mismatch: format!("Expected empty body but received {}", actual_body),
        path: s!("/")}]})
    },
    (&OptionalBody::Empty, &OptionalBody::Present(ref b, _)) => {
      BodyMatchResult::BodyMismatches(hashmap!{ "$".into() => vec![Mismatch::BodyMismatch { expected: None, actual: Some(b.clone()),
        mismatch: format!("Expected empty body but received {}", actual_body),
        path: s!("/")}]})
    },
    (&OptionalBody::Null, _) => BodyMatchResult::Ok,
    (&OptionalBody::Empty, _) => BodyMatchResult::Ok,
    (e, &OptionalBody::Missing) => {
      BodyMatchResult::BodyMismatches(hashmap!{ "$".into() => vec![Mismatch::BodyMismatch {
        expected: e.value(),
        actual: None,
        mismatch: format!("Expected body {} but was missing", e),
        path: s!("/")}]})
    },
    (e, &OptionalBody::Empty) => {
      BodyMatchResult::BodyMismatches(hashmap!{ "$".into() => vec![Mismatch::BodyMismatch {
        expected: e.value(),
        actual: None,
        mismatch: format!("Expected body {} but was empty", e),
        path: s!("/")}]})
    },
    (_, _) => compare_bodies(content_type, expected, actual, context)
  }
}

/// Matches the actual body to the expected one. This takes into account the content type of each.
pub fn match_body(
  expected: &dyn HttpPart,
  actual: &dyn HttpPart,
  context: &MatchingContext,
  header_context: &MatchingContext
) -> BodyMatchResult {
  let expected_content_type = expected.content_type().unwrap_or_default();
  let actual_content_type = actual.content_type().unwrap_or_default();
  debug!("Expected content type is '{}', actual content type is '{}'", expected_content_type,
         actual_content_type);
  let content_type_matcher = header_context.select_best_matcher(&vec!["$", "Content-Type"]);
  debug!("Content type header matcher is '{:?}'", content_type_matcher);
  if expected_content_type.is_unknown() || actual_content_type.is_unknown() ||
    expected_content_type.is_equivalent_to(&actual_content_type) ||
    (!content_type_matcher.is_empty() &&
      match_header_value("Content-Type", expected_content_type.to_string().as_str(),
                         actual_content_type.to_string().as_str(), header_context).is_ok()) {
    match_body_content(&expected_content_type, expected, actual, context)
  } else if expected.body().is_present() {
    BodyMatchResult::BodyTypeMismatch {
      expected_type: expected_content_type.to_string(),
      actual_type: actual_content_type.to_string(),
      message: format!("Expected body with content type {} but was {}", expected_content_type,
                       actual_content_type),
      expected: expected.body().value(),
      actual: actual.body().value()
    }
  } else {
    BodyMatchResult::Ok
  }
}

/// Matches the expected and actual requests
pub fn match_request(expected: Request, actual: Request) -> RequestMatchResult {
  log::info!("comparing to expected {}", expected);
  log::debug!("     body: '{}'", expected.body.str_value());
  log::debug!("     matching_rules: {:?}", expected.matching_rules);
  log::debug!("     generators: {:?}", expected.generators);

  let path_context = MatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                          &expected.matching_rules.rules_for_category("path").unwrap_or_default());
  let body_context = MatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                          &expected.matching_rules.rules_for_category("body").unwrap_or_default());
  let query_context = MatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                          &expected.matching_rules.rules_for_category("query").unwrap_or_default());
  let header_context = MatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                          &expected.matching_rules.rules_for_category("header").unwrap_or_default());
  let result = RequestMatchResult {
    method: match_method(&expected.method, &actual.method).err(),
    path: match_path(&expected.path, &actual.path, &path_context).err(),
    body: match_body(&expected, &actual, &body_context, &header_context),
    query: match_query(expected.query, actual.query, &query_context),
    headers: match_headers(expected.headers, actual.headers, &header_context)
  };

  log::debug!("--> Mismatches: {:?}", result.mismatches());
  result
}

/// Matches the actual response status to the expected one.
pub fn match_status(expected: u16, actual: u16, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = vec![];
  if context.matcher_is_defined(&path) {
    match_values(&path, context, expected, actual)
      .map_err(|messages| messages.iter().map(|message| {
        Mismatch::StatusMismatch {
          expected,
          actual,
          mismatch: message.clone()
        }
      }).collect())
  } else {
    if expected != actual {
      Err(vec![Mismatch::StatusMismatch {
        expected,
        actual,
        mismatch: format!("expected {} but was {}", expected, actual)
      }])
    } else {
      Ok(())
    }
  }
}

/// Matches the actual and expected responses.
pub fn match_response(expected: Response, actual: Response) -> Vec<Mismatch> {
  let mut mismatches = vec![];

  info!("comparing to expected response: {}", expected);

  let status_context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &expected.matching_rules.rules_for_category("status").unwrap_or_default());
  let body_context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &expected.matching_rules.rules_for_category("body").unwrap_or_default());
  let header_context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
    &expected.matching_rules.rules_for_category("header").unwrap_or_default());

  mismatches.extend_from_slice(match_body(&expected, &actual, &body_context, &header_context)
    .mismatches().as_slice());
  if let Err(m) = match_status(expected.status, actual.status, &status_context) {
    mismatches.extend_from_slice(&m);
  }
  let result = match_headers(expected.headers, actual.headers,
                             &header_context);
  for values in result.values() {
    mismatches.extend_from_slice(values.as_slice());
  }

  mismatches
}

/// Matches the actual message contents to the expected one. This takes into account the content type of each.
pub fn match_message_contents(
  expected: &Box<dyn Interaction + Send>,
  actual: &Box<dyn Interaction + Send>,
  context: &MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let expected_message = expected.as_message().unwrap();
  let expected_content_type = expected_message.message_content_type().unwrap_or_default();
  let actual_content_type = actual.as_message()
    .map(|m| HttpPart::content_type(&m)).flatten().unwrap_or_default();
  debug!("Expected content type is '{}', actual content type is '{}'", expected_content_type,
         actual_content_type);
  if expected_content_type.is_equivalent_to(&actual_content_type) {
    let result = if expected.is_v4() || actual.is_v4() {
      let expected = expected.as_v4_async_message().unwrap();
      let actual = actual.as_v4_async_message().unwrap();
      match_body_content(&expected_content_type, &expected, &actual, context)
    } else {
      let actual = actual.as_message().unwrap();
      match_body_content(&expected_content_type, &expected_message, &actual, context)
    };
    match result {
      BodyMatchResult::BodyTypeMismatch { expected_type, actual_type, message, expected, actual } => {
        Err(vec![ Mismatch::BodyTypeMismatch {
          expected: expected_type,
          actual: actual_type,
          mismatch: message,
          expected_body: expected,
          actual_body: actual
        } ])
      },
      BodyMatchResult::BodyMismatches(results) => {
        Err(results.values().flat_map(|values| values.iter().cloned()).collect())
      },
      _ => Ok(())
    }
  } else if expected_message.contents.is_present() {
    Err(vec![ Mismatch::BodyTypeMismatch {
      expected: expected_content_type.to_string(),
      actual: actual_content_type.to_string(),
      mismatch: format!("Expected message with content type {} but was {}",
                        expected_content_type, actual_content_type),
      expected_body: expected_message.contents.value(),
      actual_body: actual.as_message().map(|m| m.contents.value()).unwrap_or_default()
    } ])
  } else {
    Ok(())
  }
}

/// Matches the actual message metadata to the expected one.
pub fn match_message_metadata(
  expected: &Box<dyn Interaction + Send>,
  actual: &Box<dyn Interaction + Send>,
  context: &MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  trace!("Matching message metadata for '{}'", expected.description());
  let mut result = hashmap!{};
  let expected_metadata = if let Some(expected) = expected.as_v4_async_message() {
    expected.contents.metadata
  } else {
    expected.as_message().unwrap().metadata.iter()
      .map(|(k, v)| (k.clone(), v.clone())).collect()
  };
  let actual_metadata = if let Some(actual) = actual.as_v4_async_message() {
    actual.contents.metadata.clone()
  } else {
    actual.as_message().unwrap().metadata.iter()
      .map(|(k, v)| (k.clone(), v.clone())).collect()
  };
  debug!("Matching message metadata. Expected '{:?}', Actual '{:?}'", expected_metadata, actual_metadata);

  if !expected_metadata.is_empty() || context.config == DiffConfig::NoUnexpectedKeys {
    for (key, value) in &expected_metadata {
      match actual_metadata.get(key) {
        Some(actual_value) => {
          result.insert(key.clone(), match_metadata_value(key, value,
            actual_value, context).err().unwrap_or_default());
        },
        None => {
          result.insert(key.clone(), vec![Mismatch::MetadataMismatch { key: key.clone(),
            expected: json_to_string(&value),
            actual: "".to_string(),
            mismatch: format!("Expected message metadata '{}' but was missing", key) }]);
        }
      }
    }
  }
  result
}

fn match_metadata_value(key: &str, expected: &Value, actual: &Value, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  trace!("Comparing metadata values for key '{}'", key);
  let path = vec![key];
  let matcher_result = if context.matcher_is_defined(&path) {
    matchers::match_values(&path, context, expected, actual)
  } else if key.to_ascii_lowercase() == "contenttype" || key.to_ascii_lowercase() == "content-type" {
    debug!("Comparing message context type '{}' => '{}'", expected, actual);
    headers::match_parameter_header(expected.as_str().unwrap_or_default(),
                                    actual.as_str().unwrap_or_default(), key, "metadata")
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err| vec![err.to_string()])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::MetadataMismatch {
        key: key.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        mismatch: format!("Expected metadata key '{}' to have value '{}' but was '{}' - {}", key, expected, actual, message)
      }
    }).collect()
  })
}

/// Matches the actual and expected messages.
pub fn match_message(expected: &Box<dyn Interaction + Send>, actual: &Box<dyn Interaction + Send>) -> Vec<Mismatch> {
  let mut mismatches = vec![];

  if expected.is_message() && actual.is_message() {
    log::info!("comparing to expected message: {:?}", expected);
    let matching_rules = expected.matching_rules().unwrap_or_default();
    let body_context = if expected.is_v4() {
      MatchingContext {
        matchers: matching_rules.rules_for_category("content").unwrap_or_default(),
        config: DiffConfig::AllowUnexpectedKeys,
        matching_spec: PactSpecification::V4
      }
    } else {
      MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                           &matching_rules.rules_for_category("body").unwrap_or_default())
    };
    let metadata_context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                                &matching_rules.rules_for_category("metadata").unwrap_or_default());
    mismatches.extend_from_slice(match_message_contents(expected, actual, &body_context).err().unwrap_or_default().as_slice());
    for values in match_message_metadata(expected, actual, &metadata_context).values() {
      mismatches.extend_from_slice(values.as_slice());
    }
  } else {
    mismatches.push(Mismatch::BodyTypeMismatch {
      expected: "message".into(),
      actual: actual.type_of(),
      mismatch: format!("Cannot compare a {} with a {}", expected.type_of(), actual.type_of()),
      expected_body: None,
      actual_body: None
    });
  }

  mismatches
}

/// Generates the request by applying any defined generators
pub fn generate_request(request: &Request, mode: &GeneratorTestMode, context: &HashMap<&str, Value>) -> Request {
  let mut request = request.clone();

  let generators = request.build_generators(&GeneratorCategory::PATH);
  if !generators.is_empty() {
    trace!("Applying path generator...");
    apply_generators(mode, &generators, &mut |_, generator| {
      if let Ok(v) = generator.generate_value(&request.path, context, &DefaultVariantMatcher.boxed()) {
        request.path = v;
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::HEADER);
  if !generators.is_empty() {
    trace!("Applying header generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(header) = key.first_field() {
        if let Some(ref mut headers) = request.headers {
          if headers.contains_key(header) {
            if let Ok(v) = generator.generate_value(&headers.get(header).unwrap().clone(), context, &DefaultVariantMatcher.boxed()) {
              headers.insert(header.to_string(), v);
            }
          }
        }
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::QUERY);
  if !generators.is_empty() {
    trace!("Applying query generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(param) = key.first_field() {
        if let Some(ref mut parameters) = request.query {
          if let Some(parameter) = parameters.get_mut(param) {
            let mut generated = parameter.clone();
            for (index, val) in parameter.iter().enumerate() {
              if let Ok(v) = generator.generate_value(val, context, &DefaultVariantMatcher.boxed()) {
                generated[index] = v;
              }
            }
            *parameter = generated;
          }
        }
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && request.body.is_present() {
    trace!("Applying body generators...");
    request.body = generators_process_body(mode, &request.body, request.content_type(),
                                         context, &generators, &DefaultVariantMatcher.boxed());
  }

  request
}

/// Generates the response by applying any defined generators
pub fn generate_response(response: &Response, mode: &GeneratorTestMode, context: &HashMap<&str, Value>) -> Response {
  let mut response = response.clone();
  let generators = response.build_generators(&GeneratorCategory::STATUS);
  if !generators.is_empty() {
    trace!("Applying status generator...");
    apply_generators(mode, &generators, &mut |_, generator| {
      if let Ok(v) = generator.generate_value(&response.status, context, &DefaultVariantMatcher.boxed()) {
        debug!("Generated value for response status: {}", v);
        response.status = v;
      }
    });
  }
  let generators = response.build_generators(&GeneratorCategory::HEADER);
  if !generators.is_empty() {
    trace!("Applying header generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(header) = key.first_field() {
        if let Some(ref mut headers) = response.headers {
          if headers.contains_key(header) {
            match generator.generate_value(&headers.get(header).unwrap().clone(), context, &DefaultVariantMatcher.boxed()) {
              Ok(v) => {
                debug!("Generated value for header: {} -> {:?}", header, v);
                headers.insert(header.to_string(), v)
              },
              Err(_) => None
            };
          }
        }
      }
    });
  }
  let generators = response.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && response.body.is_present() {
    trace!("Applying body generators...");
    response.body = generators_process_body(mode, &response.body, response.content_type(),
                                            context, &generators, &DefaultVariantMatcher.boxed());
  }
  response
}

/// Matches the request part of the interaction
pub fn match_interaction_request(expected: Box<dyn Interaction>, actual: Box<dyn Interaction>, _spec_version: &PactSpecification) -> Result<RequestMatchResult, String> {
  if let Some(expected) = expected.as_request_response() {
    Ok(match_request(expected.request, actual.as_request_response().unwrap().request))
  } else {
    Err(format!("match_interaction_request must be called with HTTP request/response interactions, got {}", expected.type_of()))
  }
}

/// Matches the response part of the interaction
pub fn match_interaction_response(expected: Box<dyn Interaction>, actual: Box<dyn Interaction>, _spec_version: &PactSpecification) -> Result<Vec<Mismatch>, String> {
  if let Some(expected) = expected.as_request_response() {
    Ok(match_response(expected.response, actual.as_request_response().unwrap().response))
  } else {
    Err(format!("match_interaction_response must be called with HTTP request/response interactions, got {}", expected.type_of()))
  }
}

/// Matches an interaction
pub fn match_interaction(expected: Box<dyn Interaction + Send>, actual: Box<dyn Interaction + Send>, _spec_version: &PactSpecification) -> Result<Vec<Mismatch>, String> {
  if let Some(expected) = expected.as_request_response() {
    let request_result = match_request(expected.request, actual.as_request_response().unwrap().request);
    let response_result = match_response(expected.response, actual.as_request_response().unwrap().response);
    let mut mismatches = request_result.mismatches();
    mismatches.extend_from_slice(&*response_result);
    Ok(mismatches)
  } else if expected.is_message() {
    Ok(match_message(&expected, &actual))
  } else {
    Err(format!("match_interaction must be called with either an HTTP request/response interaction or a Message, got {}", expected.type_of()))
  }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod generator_tests;
