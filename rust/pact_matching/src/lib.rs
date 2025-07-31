//! The `pact_matching` crate provides the core logic to performing matching on HTTP requests
//! and responses. It implements the [V3 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3)
//! and [V4 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).
//!
//! ## To use it
//!
//! To use it, add it to your dependencies in your cargo manifest.
//!
//! This crate provides three functions: [`match_request`](fn.match_request.html), [`match_response`](fn.match_response.html)
//! and [`match_message`](fn.match_message.html).
//! These functions take an expected and actual request, response or message
//! model from the [`models`)(models/index.html) module, and return a vector of mismatches.
//!
//! To compare any incoming request, it first needs to be converted to a [`models::Request`](models/struct.Request.html) and then can be compared. Same for
//! any response.
//!
//! ## Crate features
//! All features are enabled by default.
//!
//! * `datetime`: Enables support of date and time expressions and generators. This will add the `chronos` crate as a dependency.
//! * `xml`: Enables support for parsing XML documents. This feature will add the `sxd-document` crate as a dependency.
//! * `plugins`: Enables support for using plugins. This feature will add the `pact-plugin-driver` crate as a dependency.
//! * `multipart`: Enables support for MIME multipart bodies. This feature will add the `multer` crate as a dependency.
//!
//! ## Reading and writing Pact files
//!
//! The [`Pact`](models/struct.Pact.html) struct in the [`models`)(models/index.html) module has methods to read and write pact JSON files. It supports all the specification
//! versions up to V4, but will convert a V1, V1.1 or V2 spec file to a V3 format.
//!
//! ## Matching request and response parts
//!
//! V3 specification matching is supported for both JSON and XML bodies, headers, query strings and request paths.
//!
//! To understand the basic rules of matching, see [Matching Gotchas](https://docs.pact.io/getting_started/matching/gotchas).
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
//! | matcher | Spec Version | example configuration | description |
//! |---------|--------------|-----------------------|-------------|
//! | Equality | V1 | `{ "match": "equality" }` | This is the default matcher, and relies on the equals operator |
//! | Regex | V2 | `{ "match": "regex", "regex": "\\d+" }` | This executes a regular expression match against the string representation of a values. |
//! | Type | V2 | `{ "match": "type" }` | This executes a type based match against the values, that is, they are equal if they are the same type. |
//! | MinType | V2 | `{ "match": "type", "min": 2 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum. |
//! | MaxType | V2 | `{ "match": "type", "max": 10 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the maximum. |
//! | MinMaxType | V2 | `{ "match": "type", "max": 10, "min": 2 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum and maximum. |
//! | Include | V3 | `{ "match": "include", "value": "substr" }` | This checks if the string representation of a value contains the substring. |
//! | Integer | V3 | `{ "match": "integer" }` | This checks if the type of the value is an integer. |
//! | Decimal | V3 | `{ "match": "decimal" }` | This checks if the type of the value is a number with decimal places. |
//! | Number | V3 | `{ "match": "number" }` | This checks if the type of the value is a number. |
//! | Timestamp | V3 | `{ "match": "datetime", "format": "yyyy-MM-dd HH:ss:mm" }` | Matches the string representation of a value against the datetime format |
//! | Time  | V3 | `{ "match": "time", "format": "HH:ss:mm" }` | Matches the string representation of a value against the time format |
//! | Date  | V3 | `{ "match": "date", "format": "yyyy-MM-dd" }` | Matches the string representation of a value against the date format |
//! | Null  | V3 | `{ "match": "null" }` | Match if the value is a null value (this is content specific, for JSON will match a JSON null) |
//! | Boolean  | V3 | `{ "match": "boolean" }` | Match if the value is a boolean value (booleans and the string values `true` and `false`) |
//! | ContentType  | V3 | `{ "match": "contentType", "value": "image/jpeg" }` | Match binary data by its content type (magic file check) |
//! | Values  | V3 | `{ "match": "values" }` | Match the values in a map, ignoring the keys |
//! | ArrayContains | V4 | `{ "match": "arrayContains", "variants": [...] }` | Checks if all the variants are present in an array. |
//! | StatusCode | V4 | `{ "match": "statusCode", "status": "success" }` | Matches the response status code. |
//! | NotEmpty | V4 | `{ "match": "notEmpty" }` | Value must be present and not empty (not null or the empty string) |
//! | Semver | V4 | `{ "match": "semver" }` | Value must be valid based on the semver specification |
//! | Semver | V4 | `{ "match": "semver" }` | Value must be valid based on the semver specification |
//! | EachKey | V4 | `{ "match": "eachKey", "rules": [{"match": "regex", "regex": "\\$(\\.\\w+)+"}], "value": "$.test.one" }` | Allows defining matching rules to apply to the keys in a map |
//! | EachValue | V4 | `{ "match": "eachValue", "rules": [{"match": "regex", "regex": "\\$(\\.\\w+)+"}], "value": "$.test.one" }` | Allows defining matching rules to apply to the values in a collection. For maps, delgates to the Values matcher. |

#![warn(missing_docs)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::fmt::Formatter;
use std::hash::Hash;
use std::panic::RefUnwindSafe;
use std::str;
use std::str::from_utf8;

use ansi_term::*;
use ansi_term::Colour::*;
use anyhow::anyhow;
use bytes::Bytes;
use itertools::{Either, Itertools};
use lazy_static::*;
use maplit::{hashmap, hashset};
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::find_content_matcher;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::PluginInteractionConfig;
use serde_json::{json, Value};
#[allow(unused_imports)] use tracing::{debug, error, info, instrument, trace, warn};

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{apply_generators, GenerateValue, GeneratorCategory, GeneratorTestMode, VariantMatcher};
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleList};
use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;

use crate::engine::{
  body_mismatches,
  build_request_plan,
  build_response_plan,
  execute_request_plan,
  execute_response_plan,
  ExecutionPlan,
  header_mismatches,
  method_mismatch,
  path_mismatch,
  query_mismatches
};
use crate::engine::context::{MatchingConfiguration, PlanMatchingContext};
use crate::generators::bodies::generators_process_body;
use crate::generators::DefaultVariantMatcher;
use crate::headers::{match_header_value, match_headers};
#[cfg(feature = "plugins")] use crate::json::match_json;
use crate::matchingrules::{DisplayForMismatch, DoMatch, match_values, Matches};
#[cfg(feature = "plugins")] use crate::plugin_support::{InteractionPart, setup_plugin_config};
use crate::query::match_query_maps;

/// Simple macro to convert a string slice to a `String` struct.
#[macro_export]
macro_rules! s {
    ($e:expr) => ($e.to_string())
}

/// Version of the library
pub const PACT_RUST_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub mod json;
pub mod matchingrules;
pub mod metrics;
pub mod generators;
pub mod engine;

#[cfg(feature = "xml")] mod xml;
pub mod binary_utils;
pub mod headers;
pub mod query;
pub mod form_urlencoded;
#[cfg(feature = "plugins")] mod plugin_support;

#[cfg(not(feature = "plugins"))]
#[derive(Clone, Debug, PartialEq)]
/// Stub for when plugins feature is not enabled
pub struct PluginInteractionConfig {}

/// Context used to apply matching logic
pub trait MatchingContext: Debug {
  /// If there is a matcher defined at the path in this context
  fn matcher_is_defined(&self, path: &DocPath) -> bool;

  /// Selected the best matcher from the context for the given path
  fn select_best_matcher(&self, path: &DocPath) -> RuleList;

  /// If there is a type matcher defined at the path in this context
  fn type_matcher_defined(&self, path: &DocPath) -> bool;

  /// If there is a values matcher defined at the path in this context
  fn values_matcher_defined(&self, path: &DocPath) -> bool;

  /// If a matcher defined at the path (ignoring parents)
  fn direct_matcher_defined(&self, path: &DocPath, matchers: &HashSet<&str>) -> bool;

  /// Matches the keys of the expected and actual maps
  fn match_keys(&self, path: &DocPath, expected: &BTreeSet<String>, actual: &BTreeSet<String>) -> Result<(), Vec<CommonMismatch>>;

  /// Returns the plugin configuration associated with the context
  fn plugin_configuration(&self) -> &HashMap<String, PluginInteractionConfig>;

  /// Returns the matching rules for the matching context
  fn matchers(&self) -> &MatchingRuleCategory;

  /// Configuration to apply when matching with the context
  fn config(&self) -> DiffConfig;

  /// Clones the current context with the provided matching rules
  fn clone_with(&self, matchers: &MatchingRuleCategory) -> Box<dyn MatchingContext + Send + Sync>;
}

#[derive(Debug, Clone)]
/// Core implementation of a matching context
pub struct CoreMatchingContext {
  /// Matching rules that apply when matching with the context
  pub matchers: MatchingRuleCategory,
  /// Configuration to apply when matching with the context
  pub config: DiffConfig,
  /// Specification version to apply when matching with the context
  pub matching_spec: PactSpecification,
  /// Any plugin configuration available for the interaction
  pub plugin_configuration: HashMap<String, PluginInteractionConfig>
}

impl CoreMatchingContext {
  /// Creates a new context with the given config and matching rules
  pub fn new(
    config: DiffConfig,
    matchers: &MatchingRuleCategory,
    plugin_configuration: &HashMap<String, PluginInteractionConfig>
  ) -> Self {
    CoreMatchingContext {
      matchers: matchers.clone(),
      config,
      plugin_configuration: plugin_configuration.clone(),
      .. CoreMatchingContext::default()
    }
  }

  /// Creates a new empty context with the given config
  pub fn with_config(config: DiffConfig) -> Self {
    CoreMatchingContext {
      config,
      .. CoreMatchingContext::default()
    }
  }

  fn matchers_for_exact_path(&self, path: &DocPath) -> MatchingRuleCategory {
    match self.matchers.name {
      Category::HEADER | Category::QUERY => self.matchers.filter(|&(val, _)| {
        path.len() == 1 && path.first_field() == val.first_field()
      }),
      Category::BODY => self.matchers.filter(|&(val, _)| {
        let p = path.to_vec();
        let p_slice = p.iter().map(|p| p.as_str()).collect_vec();
        val.matches_path_exactly(p_slice.as_slice())
      }),
      _ => self.matchers.filter(|_| false)
    }
  }

  #[allow(dead_code)]
  pub(crate) fn clone_from(context: &(dyn MatchingContext + Send + Sync)) -> Self {
    CoreMatchingContext {
      matchers: context.matchers().clone(),
      config: context.config().clone(),
      plugin_configuration: context.plugin_configuration().clone(),
      .. CoreMatchingContext::default()
    }
  }
}

impl Default for CoreMatchingContext {
  fn default() -> Self {
    CoreMatchingContext {
      matchers: Default::default(),
      config: DiffConfig::AllowUnexpectedKeys,
      matching_spec: PactSpecification::V3,
      plugin_configuration: Default::default()
    }
  }
}

impl MatchingContext for CoreMatchingContext {
  #[instrument(level = "trace", ret, skip_all, fields(path, matchers = ?self.matchers))]
  fn matcher_is_defined(&self, path: &DocPath) -> bool {
    let path = path.to_vec();
    let path_slice = path.iter().map(|p| p.as_str()).collect_vec();
    self.matchers.matcher_is_defined(path_slice.as_slice())
  }

  fn select_best_matcher(&self, path: &DocPath) -> RuleList {
    let path = path.to_vec();
    let path_slice = path.iter().map(|p| p.as_str()).collect_vec();
    self.matchers.select_best_matcher(path_slice.as_slice())
  }

  fn type_matcher_defined(&self, path: &DocPath) -> bool {
    let path = path.to_vec();
    let path_slice = path.iter().map(|p| p.as_str()).collect_vec();
    self.matchers.resolve_matchers_for_path(path_slice.as_slice()).type_matcher_defined()
  }

  fn values_matcher_defined(&self, path: &DocPath) -> bool {
    self.matchers_for_exact_path(path).values_matcher_defined()
  }

  fn direct_matcher_defined(&self, path: &DocPath, matchers: &HashSet<&str>) -> bool {
    let actual = self.matchers_for_exact_path(path);
    if matchers.is_empty() {
      actual.is_not_empty()
    } else {
      actual.as_rule_list().rules.iter().any(|r| matchers.contains(r.name().as_str()))
    }
  }

  fn match_keys(
    &self,
    path: &DocPath,
    expected: &BTreeSet<String>,
    actual: &BTreeSet<String>
  ) -> Result<(), Vec<CommonMismatch>> {
    let mut expected_keys = expected.iter().cloned().collect::<Vec<String>>();
    expected_keys.sort();
    let mut actual_keys = actual.iter().cloned().collect::<Vec<String>>();
    actual_keys.sort();
    let missing_keys: Vec<String> = expected.iter().filter(|key| !actual.contains(*key)).cloned().collect();
    let mut result = vec![];

    if !self.direct_matcher_defined(path, &hashset! { "values", "each-value", "each-key" }) {
      match self.config {
        DiffConfig::AllowUnexpectedKeys if !missing_keys.is_empty() => {
          result.push(CommonMismatch {
            path: path.to_string(),
            expected: expected.for_mismatch(),
            actual: actual.for_mismatch(),
            description: format!("Actual map is missing the following keys: {}", missing_keys.join(", ")),
          });
        }
        DiffConfig::NoUnexpectedKeys if expected_keys != actual_keys => {
          result.push(CommonMismatch {
            path: path.to_string(),
            expected: expected.for_mismatch(),
            actual: actual.for_mismatch(),
            description: format!("Expected a Map with keys [{}] but received one with keys [{}]",
                              expected_keys.join(", "), actual_keys.join(", ")),
          });
        }
        _ => {}
      }
    }

    if self.direct_matcher_defined(path, &Default::default()) {
      let matchers = self.select_best_matcher(path);
      for matcher in matchers.rules {
        match matcher {
          MatchingRule::EachKey(definition) => {
            for sub_matcher in definition.rules {
              match sub_matcher {
                Either::Left(rule) => {
                  for key in &actual_keys {
                    let key_path = path.join(key);
                    if let Err(err) = rule.match_value("", key.as_str(), false, false) {
                      result.push(CommonMismatch {
                        path: key_path.to_string(),
                        expected: "".to_string(),
                        actual: key.clone(),
                        description: err.to_string(),
                      });
                    }
                  }
                }
                Either::Right(name) => {
                  result.push(CommonMismatch {
                    path: path.to_string(),
                    expected: expected.for_mismatch(),
                    actual: actual.for_mismatch(),
                    description: format!("Expected a matching rule, found an unresolved reference '{}'",
                      name.name),
                  });
                }
              }
            }
          }
          _ => {}
        }
      }
    }

    if result.is_empty() {
      Ok(())
    } else {
      Err(result)
    }
  }

  fn plugin_configuration(&self) -> &HashMap<String, PluginInteractionConfig> {
    &self.plugin_configuration
  }

  fn matchers(&self) -> &MatchingRuleCategory {
    &self.matchers
  }

  fn config(&self) -> DiffConfig {
    self.config
  }

  fn clone_with(&self, matchers: &MatchingRuleCategory) -> Box<dyn MatchingContext + Send + Sync> {
    Box::new(CoreMatchingContext {
      matchers: matchers.clone(),
      config: self.config.clone(),
      matching_spec: self.matching_spec,
      plugin_configuration: self.plugin_configuration.clone()
    })
  }
}

#[derive(Debug, Clone, Default)]
/// Matching context for headers. Keys will be applied in a case-insensitive manor
pub struct HeaderMatchingContext {
  inner_context: CoreMatchingContext
}

impl HeaderMatchingContext {
  /// Wraps a MatchingContext, downcasing all the matching path keys
  pub fn new(context: &(dyn MatchingContext + Send + Sync)) -> Self {
    let matchers = context.matchers();
    HeaderMatchingContext {
      inner_context: CoreMatchingContext::new(
        context.config(),
        &MatchingRuleCategory {
          name: matchers.name.clone(),
          rules: matchers.rules.iter()
            .map(|(path, rules)| {
              (path.to_lower_case(), rules.clone())
            })
            .collect()
        },
        &context.plugin_configuration()
      )
    }
  }
}

impl MatchingContext for HeaderMatchingContext {
  fn matcher_is_defined(&self, path: &DocPath) -> bool {
    self.inner_context.matcher_is_defined(path)
  }

  fn select_best_matcher(&self, path: &DocPath) -> RuleList {
    self.inner_context.select_best_matcher(path)
  }

  fn type_matcher_defined(&self, path: &DocPath) -> bool {
    self.inner_context.type_matcher_defined(path)
  }

  fn values_matcher_defined(&self, path: &DocPath) -> bool {
    self.inner_context.values_matcher_defined(path)
  }

  fn direct_matcher_defined(&self, path: &DocPath, matchers: &HashSet<&str>) -> bool {
    self.inner_context.direct_matcher_defined(path, matchers)
  }

  fn match_keys(&self, path: &DocPath, expected: &BTreeSet<String>, actual: &BTreeSet<String>) -> Result<(), Vec<CommonMismatch>> {
    self.inner_context.match_keys(path, expected, actual)
  }

  fn plugin_configuration(&self) -> &HashMap<String, PluginInteractionConfig> {
    self.inner_context.plugin_configuration()
  }

  fn matchers(&self) -> &MatchingRuleCategory {
    self.inner_context.matchers()
  }

  fn config(&self) -> DiffConfig {
    self.inner_context.config()
  }

  fn clone_with(&self, matchers: &MatchingRuleCategory) -> Box<dyn MatchingContext + Send + Sync> {
    Box::new(HeaderMatchingContext::new(
      &CoreMatchingContext {
        matchers: matchers.clone(),
        config: self.inner_context.config.clone(),
        matching_spec: self.inner_context.matching_spec,
        plugin_configuration: self.inner_context.plugin_configuration.clone()
      }
    ))
  }
}

lazy_static! {
  static ref BODY_MATCHERS: [
    (fn(content_type: &ContentType) -> bool,
    fn(expected: &(dyn HttpPart + Send + Sync), actual: &(dyn HttpPart + Send + Sync), context: &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<Mismatch>>); 5]
     = [
      (|content_type| { content_type.is_json() }, json::match_json),
      (|content_type| { content_type.is_xml() }, match_xml),
      (|content_type| { content_type.main_type == "multipart" }, binary_utils::match_mime_multipart),
      (|content_type| { content_type.base_type() == "application/x-www-form-urlencoded" }, form_urlencoded::match_form_urlencoded),
      (|content_type| { content_type.is_binary() || content_type.base_type() == "application/octet-stream" }, binary_utils::match_octet_stream)
  ];
}

fn match_xml(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  #[cfg(feature = "xml")]
  {
    xml::match_xml(expected, actual, context)
  }
  #[cfg(not(feature = "xml"))]
  {
    warn!("Matching XML documents requires the xml feature to be enabled");
    match_text(&expected.body().value(), &actual.body().value(), context)
  }
}

/// Store common mismatch information so it can be converted to different type of mismatches
#[derive(Debug, Clone, PartialOrd, Ord, Eq)]
pub struct CommonMismatch {
  /// path expression to where the mismatch occurred
  pub path: String,
  /// expected value (as a string)
  expected: String,
  /// actual value (as a string)
  actual: String,
  /// Description of the mismatch
  description: String
}

impl CommonMismatch {
  /// Convert common mismatch to body mismatch
  pub fn to_body_mismatch(&self) -> Mismatch {
    Mismatch::BodyMismatch {
      path: self.path.clone(),
      expected: Some(self.expected.clone().into()),
      actual: Some(self.actual.clone().into()),
      mismatch: self.description.clone()
    }
  }

  /// Convert common mismatch to query mismatch
  pub fn to_query_mismatch(&self) -> Mismatch {
    Mismatch::QueryMismatch {
      parameter: self.path.clone(),
      expected: self.expected.clone(),
      actual: self.actual.clone(),
      mismatch: self.description.clone()
    }
  }

  /// Convert common mismatch to header mismatch
  pub fn to_header_mismatch(&self) -> Mismatch {
    Mismatch::HeaderMismatch {
      key: self.path.clone(),
      expected: self.expected.clone().into(),
      actual: self.actual.clone().into(),
      mismatch: self.description.clone()
    }
  }
}

impl Display for CommonMismatch {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.description)
  }
}

impl PartialEq for CommonMismatch {
  fn eq(&self, other: &CommonMismatch) -> bool {
    self.path == other.path && self.expected == other.expected && self.actual == other.actual
  }
}

impl From<Mismatch> for CommonMismatch {
  fn from(value: Mismatch) -> Self {
    match value {
      Mismatch::MethodMismatch { expected, actual , mismatch} => CommonMismatch {
        path: "".to_string(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      },
      Mismatch::PathMismatch { expected, actual, mismatch } => CommonMismatch {
        path: "".to_string(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      },
      Mismatch::StatusMismatch { expected, actual, mismatch } => CommonMismatch {
        path: "".to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        description: mismatch.clone()
      },
      Mismatch::QueryMismatch { parameter, expected, actual, mismatch } => CommonMismatch {
        path: parameter.clone(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      },
      Mismatch::HeaderMismatch { key, expected, actual, mismatch } => CommonMismatch {
        path: key.clone(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      },
      Mismatch::BodyTypeMismatch { expected, actual, mismatch, .. } => CommonMismatch {
        path: "".to_string(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      },
      Mismatch::BodyMismatch { path, expected, actual, mismatch } => CommonMismatch {
        path: path.clone(),
        expected: String::from_utf8_lossy(expected.unwrap_or_default().as_ref()).to_string(),
        actual: String::from_utf8_lossy(actual.unwrap_or_default().as_ref()).to_string(),
        description: mismatch.clone()
      },
      Mismatch::MetadataMismatch { key, expected, actual, mismatch } => CommonMismatch {
        path: key.clone(),
        expected: expected.clone(),
        actual: actual.clone(),
        description: mismatch.clone()
      }
    }
  }
}

/// Enum that defines the different types of mismatches that can occur.
#[derive(Debug, Clone, PartialOrd, Ord, Eq)]
pub enum Mismatch {
    /// Request Method mismatch
    MethodMismatch {
      /// Expected request method
      expected: String,
      /// Actual request method
      actual: String,
      /// description of the mismatch
      mismatch: String
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
      Mismatch::MethodMismatch { expected: e, actual: a, mismatch: m } => {
        json!({
          "type" : "MethodMismatch",
          "expected" : e,
          "actual" : a,
          "mismatch" : m
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
        Mismatch::BodyTypeMismatch { .. } => "has a matching body".to_string(),
        Mismatch::BodyMismatch { .. } => "has a matching body".to_string(),
        Mismatch::MetadataMismatch { .. } => "has matching metadata".to_string()
      }
    }

    /// Returns a formatted string for this mismatch
    pub fn description(&self) -> String {
      match self {
        Mismatch::MethodMismatch { expected: e, actual: a, mismatch: m } => if m.is_empty() {
          format!("expected {} but was {}", e, a)
        } else {
          m.clone()
        },
        Mismatch::PathMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::StatusMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::QueryMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::HeaderMismatch { mismatch, .. } => mismatch.clone(),
        Mismatch::BodyTypeMismatch {  expected: e, actual: a, .. } =>
          format!("Expected a body of '{}' but the actual content type was '{}'", e, a),
        Mismatch::BodyMismatch { path, mismatch, .. } => format!("{} -> {}", path, mismatch),
        Mismatch::MetadataMismatch { mismatch, .. } => mismatch.clone()
      }
    }

    /// Returns a formatted string with ansi escape codes for this mismatch
    pub fn ansi_description(&self) -> String {
      match self {
        Mismatch::MethodMismatch { expected: e, actual: a, .. } => format!("expected {} but was {}", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::PathMismatch { expected: e, actual: a, .. } => format!("expected '{}' but was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::StatusMismatch { expected: e, actual: a, .. } => format!("expected {} but was {}", Red.paint(e.to_string()), Green.paint(a.to_string())),
        Mismatch::QueryMismatch { expected: e, actual: a, parameter: p, .. } => format!("Expected '{}' but received '{}' for query parameter '{}'",
          Red.paint(e.to_string()), Green.paint(a.to_string()), Style::new().bold().paint(p.clone())),
        Mismatch::HeaderMismatch { expected: e, actual: a, key: k, .. } => format!("Expected header '{}' to have value '{}' but was '{}'",
          Style::new().bold().paint(k.clone()), Red.paint(e.to_string()), Green.paint(a.to_string())),
        Mismatch::BodyTypeMismatch {  expected: e, actual: a, .. } =>
          format!("expected a body of '{}' but the actual content type was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
        Mismatch::BodyMismatch { path, mismatch, .. } => format!("{} -> {}", Style::new().bold().paint(path.clone()), mismatch),
        Mismatch::MetadataMismatch { expected: e, actual: a, key: k, .. } => format!("Expected message metadata '{}' to have value '{}' but was '{}'",
          Style::new().bold().paint(k.clone()), Red.paint(e.to_string()), Green.paint(a.to_string()))
      }
    }
}

impl PartialEq for Mismatch {
  fn eq(&self, other: &Mismatch) -> bool {
    match (self, other) {
      (Mismatch::MethodMismatch { expected: e1, actual: a1, .. },
        Mismatch::MethodMismatch { expected: e2, actual: a2, .. }) => {
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

fn merge_result<T: Clone>(res1: Result<(), Vec<T>>, res2: Result<(), Vec<T>>) -> Result<(), Vec<T>> {
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
    for mismatches in self.query.values() {
      if mismatches.is_empty() {
        score += 1;
      } else {
        score -= 1;
      }
    }
    for mismatches in self.headers.values() {
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
        for mismatches in results.values() {
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

impl From<ExecutionPlan> for RequestMatchResult {
  fn from(plan: ExecutionPlan) -> Self {
    let request = plan.fetch_node(&[":request"]).unwrap_or_default();
    let method = method_mismatch(&request);
    let path = path_mismatch(&request);
    let query = query_mismatches(&request);
    let headers = header_mismatches(&request);
    let body = body_mismatches(&request);
    RequestMatchResult {
      method,
      path,
      body,
      query,
      headers
    }
  }
}

/// Enum that defines the configuration options for performing a match.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffConfig {
    /// If unexpected keys are allowed and ignored during matching.
    AllowUnexpectedKeys,
    /// If unexpected keys cause a mismatch.
    NoUnexpectedKeys
}

/// Matches the actual text body to the expected one.
pub fn match_text(expected: &Option<Bytes>, actual: &Option<Bytes>, context: &dyn MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::root();
  if context.matcher_is_defined(&path) {
    let mut mismatches = vec![];
    let empty = Bytes::default();
    let expected_str = match from_utf8(expected.as_ref().unwrap_or(&empty)) {
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
    let actual_str = match from_utf8(actual.as_ref().unwrap_or(&empty)) {
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
    if let Err(messages) = match_values(&path, &context.select_best_matcher(&path), expected_str, actual_str) {
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
    let expected = expected.clone().unwrap_or_default();
    let actual = actual.clone().unwrap_or_default();
    let e = String::from_utf8_lossy(&expected);
    let a = String::from_utf8_lossy(&actual);
    let mismatch = format!("Expected body '{}' to match '{}' using equality but did not match", e, a);
    Err(vec![
      Mismatch::BodyMismatch {
        path: "$".to_string(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        mismatch
      }
    ])
  } else {
    Ok(())
  }
}

/// Matches the actual request method to the expected one.
pub fn match_method(expected: &str, actual: &str) -> Result<(), Mismatch> {
  if expected.to_lowercase() != actual.to_lowercase() {
    Err(Mismatch::MethodMismatch { expected: expected.to_string(), actual: actual.to_string(), mismatch: "".to_string() })
  } else {
    Ok(())
  }
}

/// Matches the actual request path to the expected one.
pub fn match_path(expected: &str, actual: &str, context: &(dyn MatchingContext + Send + Sync)) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::empty();
  let matcher_result = if context.matcher_is_defined(&path) {
    match_values(&path, &context.select_best_matcher(&path), expected.to_string(), actual.to_string())
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

/// Matches the actual query parameters to the expected ones.
pub fn match_query(
  expected: Option<HashMap<String, Vec<Option<String>>>>,
  actual: Option<HashMap<String, Vec<Option<String>>>>,
  context: &(dyn MatchingContext + Send + Sync)
) -> HashMap<String, Vec<Mismatch>> {
  match (actual, expected) {
    (Some(aqm), Some(eqm)) => match_query_maps(eqm, aqm, context),
    (Some(aqm), None) => aqm.iter().map(|(key, value)| {
      let actual_value = value.iter().map(|v| v.clone().unwrap_or_default()).collect_vec();
      (key.clone(), vec![Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: "".to_string(),
        actual: format!("{:?}", actual_value),
        mismatch: format!("Unexpected query parameter '{}' received", key)
      }])
    }).collect(),
    (None, Some(eqm)) => eqm.iter().map(|(key, value)| {
      let expected_value = value.iter().map(|v| v.clone().unwrap_or_default()).collect_vec();
      (key.clone(), vec![Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: format!("{:?}", expected_value),
        actual: "".to_string(),
        mismatch: format!("Expected query parameter '{}' but was missing", key)
      }])
    }).collect(),
    (None, None) => hashmap!{}
  }
}

fn group_by<I, F, K>(items: I, f: F) -> HashMap<K, Vec<I::Item>>
  where I: IntoIterator, F: Fn(&I::Item) -> K, K: Eq + Hash {
  let mut m = hashmap!{};
  for item in items {
    let key = f(&item);
    let values = m.entry(key).or_insert_with(Vec::new);
    values.push(item);
  }
  m
}

#[instrument(level = "trace", ret, skip_all)]
pub(crate) async fn compare_bodies(
  content_type: &ContentType,
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> BodyMatchResult {
  let mut mismatches = vec![];

  trace!(?content_type, "Comparing bodies");

  #[cfg(feature = "plugins")]
  {
    match find_content_matcher(content_type) {
      Some(matcher) => {
        debug!("Using content matcher {} for content type '{}'", matcher.catalogue_entry_key(), content_type);
        if matcher.is_core() {
          if let Err(m) = match matcher.catalogue_entry_key().as_str() {
            "core/content-matcher/form-urlencoded" => form_urlencoded::match_form_urlencoded(expected, actual, context),
            "core/content-matcher/json" => match_json(expected, actual, context),
            "core/content-matcher/multipart-form-data" => binary_utils::match_mime_multipart(expected, actual, context),
            "core/content-matcher/text" => match_text(&expected.body().value(), &actual.body().value(), context),
            "core/content-matcher/xml" => {
              #[cfg(feature = "xml")]
              {
                xml::match_xml(expected, actual, context)
              }
              #[cfg(not(feature = "xml"))]
              {
                warn!("Matching XML bodies requires the xml feature to be enabled");
                match_text(&expected.body().value(), &actual.body().value(), context)
              }
            },
            "core/content-matcher/binary" => binary_utils::match_octet_stream(expected, actual, context),
            _ => {
              warn!("There is no core content matcher for entry {}", matcher.catalogue_entry_key());
              match_text(&expected.body().value(), &actual.body().value(), context)
            }
          } {
            mismatches.extend_from_slice(&*m);
          }
        } else {
          trace!(plugin_name = matcher.plugin_name(),"Content matcher is provided via a plugin");
          let plugin_config = context.plugin_configuration().get(&matcher.plugin_name()).cloned();
          trace!("Plugin config = {:?}", plugin_config);
          if let Err(map) = matcher.match_contents(expected.body(), actual.body(), &context.matchers(),
                                                   context.config() == DiffConfig::AllowUnexpectedKeys, plugin_config).await {
            // TODO: group the mismatches by key
            for (_key, list) in map {
              for mismatch in list {
                mismatches.push(Mismatch::BodyMismatch {
                  path: mismatch.path.clone(),
                  expected: Some(Bytes::from(mismatch.expected)),
                  actual: Some(Bytes::from(mismatch.actual)),
                  mismatch: mismatch.mismatch.clone()
                });
              }
            }
          }
        }
      }
      None => {
        debug!("No content matcher defined for content type '{}', using core matcher implementation", content_type);
        mismatches.extend(compare_bodies_core(content_type, expected, actual, context));
      }
    }
  }

  #[cfg(not(feature = "plugins"))]
  {
    mismatches.extend(compare_bodies_core(content_type, expected, actual, context));
  }

  if mismatches.is_empty() {
    BodyMatchResult::Ok
  } else {
    BodyMatchResult::BodyMismatches(group_by(mismatches, |m| match m {
      Mismatch::BodyMismatch { path: m, ..} => m.to_string(),
      _ => String::default()
    }))
  }
}

fn compare_bodies_core(
  content_type: &ContentType,
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Vec<Mismatch> {
  let mut mismatches = vec![];
  match BODY_MATCHERS.iter().find(|mt| mt.0(content_type)) {
    Some(match_fn) => {
      debug!("Using body matcher for content type '{}'", content_type);
      if let Err(m) = match_fn.1(expected, actual, context) {
        mismatches.extend_from_slice(&*m);
      }
    },
    None => {
      debug!("No body matcher defined for content type '{}', checking for a content type matcher", content_type);
      let path = DocPath::root();
      if context.matcher_is_defined(&path) && context.select_best_matcher(&path).rules
        .iter().any(|rule| if let MatchingRule::ContentType(_) = rule { true } else { false }) {
        debug!("Found a content type matcher");
        if let Err(m) = binary_utils::match_octet_stream(expected, actual, context) {
          mismatches.extend_from_slice(&*m);
        }
      } else {
        debug!("No body matcher defined for content type '{}', using plain text matcher", content_type);
        if let Err(m) = match_text(&expected.body().value(), &actual.body().value(), context) {
          mismatches.extend_from_slice(&*m);
        }
      }
    }
  };
  mismatches
}

#[instrument(level = "trace", ret, skip_all, fields(%content_type, ?context))]
async fn match_body_content(
  content_type: &ContentType,
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> BodyMatchResult {
  let expected_body = expected.body();
  let actual_body = actual.body();
  match (expected_body, actual_body) {
    (&OptionalBody::Missing, _) => BodyMatchResult::Ok,
    (&OptionalBody::Null, &OptionalBody::Present(ref b, _, _)) => {
      BodyMatchResult::BodyMismatches(hashmap!{ "$".into() => vec![Mismatch::BodyMismatch { expected: None, actual: Some(b.clone()),
        mismatch: format!("Expected empty body but received {}", actual_body),
        path: s!("/")}]})
    },
    (&OptionalBody::Empty, &OptionalBody::Present(ref b, _, _)) => {
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
    (_, _) => compare_bodies(content_type, expected, actual, context).await
  }
}

/// Matches the actual body to the expected one. This takes into account the content type of each.
pub async fn match_body(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync),
  header_context: &(dyn MatchingContext + Send + Sync)
) -> BodyMatchResult {
  let expected_content_type = expected.content_type().unwrap_or_default();
  let actual_content_type = actual.content_type().unwrap_or_default();
  debug!("expected content type = '{}', actual content type = '{}'", expected_content_type,
         actual_content_type);
  let content_type_matcher = header_context.select_best_matcher(&DocPath::root().join("content-type"));
  debug!("content type header matcher = '{:?}'", content_type_matcher);
  if expected_content_type.is_unknown() || actual_content_type.is_unknown() ||
    expected_content_type.is_equivalent_to(&actual_content_type) ||
    expected_content_type.is_equivalent_to(&actual_content_type.base_type()) ||
    (!content_type_matcher.is_empty() &&
      match_header_value("Content-Type", 0, expected_content_type.to_string().as_str(),
                         actual_content_type.to_string().as_str(), header_context, true
      ).is_ok()) {
    match_body_content(&expected_content_type, expected, actual, context).await
  } else if expected.body().is_present() {
    BodyMatchResult::BodyTypeMismatch {
      expected_type: expected_content_type.to_string(),
      actual_type: actual_content_type.to_string(),
      message: format!("Expected a body of '{}' but the actual content type was '{}'", expected_content_type,
                       actual_content_type),
      expected: expected.body().value(),
      actual: actual.body().value()
    }
  } else {
    BodyMatchResult::Ok
  }
}

/// Matches the expected and actual requests
#[allow(unused_variables)]
pub async fn match_request<'a>(
  expected: HttpRequest,
  actual: HttpRequest,
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  interaction: &Box<dyn Interaction + Send + Sync + RefUnwindSafe>
) -> anyhow::Result<RequestMatchResult> {
  debug!("comparing to expected {}", expected);
  debug!("     body: '{}'", expected.body.display_string());
  debug!("     matching_rules:\n{}", expected.matching_rules);
  debug!("     generators: {:?}", expected.generators);

  #[allow(unused_mut, unused_assignments)] let mut plugin_data = hashmap!{};
  #[cfg(feature = "plugins")]
  {
    plugin_data = setup_plugin_config(pact, interaction, InteractionPart::Request);
  };
  trace!("plugin_data = {:?}", plugin_data);

  let use_v2_engine = std::env::var("PACT_MATCHING_ENGINE")
    .map(|val| val.to_lowercase() == "v2")
    .unwrap_or(false);
  if use_v2_engine {
    let config = MatchingConfiguration {
      allow_unexpected_entries: false,
      .. MatchingConfiguration::init_from_env()
    };
    let mut context = PlanMatchingContext {
      pact: pact.as_v4_pact().unwrap_or_default(),
      interaction: interaction.as_v4().unwrap(),
      matching_rules: Default::default(),
      config
    };

    let plan = build_request_plan(&expected, &mut context)?;
    let executed_plan = execute_request_plan(&plan, &actual, &mut context)?;

    if config.log_executed_plan {
      debug!("config = {:?}", config);
      debug!("\n{}", executed_plan.pretty_form());
    }
    if config.log_plan_summary {
      info!("\n{}", executed_plan.generate_summary(config.coloured_output));
    }
    Ok(executed_plan.into())
  } else {
    let path_context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
      &expected.matching_rules.rules_for_category("path").unwrap_or_default(),
      &plugin_data);
    let body_context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap_or_default(),
      &plugin_data);
    let query_context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
      &expected.matching_rules.rules_for_category("query").unwrap_or_default(),
      &plugin_data);
    let header_context = HeaderMatchingContext::new(
      &CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
        &expected.matching_rules.rules_for_category("header").unwrap_or_default(),
        &plugin_data
      )
    );
    let result = RequestMatchResult {
      method: match_method(&expected.method, &actual.method).err(),
      path: match_path(&expected.path, &actual.path, &path_context).err(),
      body: match_body(&expected, &actual, &body_context, &header_context).await,
      query: match_query(expected.query, actual.query, &query_context),
      headers: match_headers(expected.headers, actual.headers, &header_context)
    };

    debug!("--> Mismatches: {:?}", result.mismatches());
    Ok(result)
  }
}

/// Matches the actual response status to the expected one.
#[instrument(level = "trace")]
pub fn match_status(expected: u16, actual: u16, context: &dyn MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::empty();
  let result = if context.matcher_is_defined(&path) {
    match_values(&path, &context.select_best_matcher(&path), expected, actual)
      .map_err(|messages| messages.iter().map(|message| {
        Mismatch::StatusMismatch {
          expected,
          actual,
          mismatch: message.clone()
        }
      }).collect())
  } else if expected != actual {
    Err(vec![Mismatch::StatusMismatch {
      expected,
      actual,
      mismatch: format!("expected {} but was {}", expected, actual)
    }])
  } else {
    Ok(())
  };
  trace!(?result, "matching response status");
  result
}

/// Matches the actual and expected responses.
#[allow(unused_variables)]
pub async fn match_response<'a>(
  expected: HttpResponse,
  actual: HttpResponse,
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  interaction: &Box<dyn Interaction + Send + Sync + RefUnwindSafe>
) -> anyhow::Result<Vec<Mismatch>> {
  let mut mismatches = vec![];

  debug!("comparing to expected response: {}", expected);
  #[allow(unused_mut, unused_assignments)] let mut plugin_data = hashmap!{};
  #[cfg(feature = "plugins")]
  {
    plugin_data = setup_plugin_config(pact, interaction, InteractionPart::Response);
  };
  trace!("plugin_data = {:?}", plugin_data);

  let use_v2_engine = std::env::var("PACT_MATCHING_ENGINE")
    .map(|val| val.to_lowercase() == "v2")
    .unwrap_or(false);
  if use_v2_engine {
    let config = MatchingConfiguration {
      allow_unexpected_entries: true,
      .. MatchingConfiguration::init_from_env()
    };
    let mut context = PlanMatchingContext {
      pact: pact.as_v4_pact().unwrap_or_default(),
      interaction: interaction.as_v4().unwrap(),
      matching_rules: Default::default(),
      config
    };

    let plan = build_response_plan(&expected, &mut context)?;
    let executed_plan = execute_response_plan(&plan, &actual, &mut context)?;

    if config.log_executed_plan {
      debug!("config = {:?}", config);
      debug!("\n{}", executed_plan.pretty_form());
    }
    if config.log_plan_summary {
      info!("\n{}", executed_plan.generate_summary(config.coloured_output));
    }
    Ok(executed_plan.into())
  } else {
    let status_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("status").unwrap_or_default(),
      &plugin_data);
    let body_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap_or_default(),
      &plugin_data);
    let header_context = HeaderMatchingContext::new(
      &CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
        &expected.matching_rules.rules_for_category("header").unwrap_or_default(),
        &plugin_data
      )
    );

    mismatches.extend_from_slice(match_body(&expected, &actual, &body_context, &header_context).await
      .mismatches().as_slice());
    if let Err(m) = match_status(expected.status, actual.status, &status_context) {
      mismatches.extend_from_slice(&m);
    }
    let result = match_headers(expected.headers, actual.headers,
      &header_context);
    for values in result.values() {
      mismatches.extend_from_slice(values.as_slice());
    }

    trace!(?mismatches, "match response");

    Ok(mismatches)
  }
}

/// Matches the actual message contents to the expected one. This takes into account the content type of each.
#[instrument(level = "trace")]
pub async fn match_message_contents(
  expected: &MessageContents,
  actual: &MessageContents,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  let expected_content_type = expected.message_content_type().unwrap_or_default();
  let actual_content_type = actual.message_content_type().unwrap_or_default();
  debug!("expected content type = '{}', actual content type = '{}'", expected_content_type,
         actual_content_type);
  if expected_content_type.is_equivalent_to(&actual_content_type) {
    let result = match_body_content(&expected_content_type, expected, actual, context).await;
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
  } else if expected.contents.is_present() {
    Err(vec![ Mismatch::BodyTypeMismatch {
      expected: expected_content_type.to_string(),
      actual: actual_content_type.to_string(),
      mismatch: format!("Expected message with content type {} but was {}",
                        expected_content_type, actual_content_type),
      expected_body: expected.contents.value(),
      actual_body: actual.contents.value()
    } ])
  } else {
    Ok(())
  }
}

/// Matches the actual message metadata to the expected one.
#[instrument(level = "trace")]
pub fn match_message_metadata(
  expected: &MessageContents,
  actual: &MessageContents,
  context: &dyn MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  debug!("Matching message metadata");
  let mut result = hashmap!{};
  let expected_metadata = &expected.metadata;
  let actual_metadata = &actual.metadata;
  debug!("Matching message metadata. Expected '{:?}', Actual '{:?}'", expected_metadata, actual_metadata);

  if !expected_metadata.is_empty() || context.config() == DiffConfig::NoUnexpectedKeys {
    for (key, value) in expected_metadata {
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

#[instrument(level = "trace")]
fn match_metadata_value(
  key: &str,
  expected: &Value,
  actual: &Value,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  debug!("Comparing metadata values for key '{}'", key);
  let path = DocPath::root().join(key);
  let matcher_result = if context.matcher_is_defined(&path) {
    match_values(&path, &context.select_best_matcher(&path), expected, actual)
  } else if key.to_ascii_lowercase() == "contenttype" || key.to_ascii_lowercase() == "content-type" {
    debug!("Comparing message context type '{}' => '{}'", expected, actual);
    headers::match_parameter_header(expected.as_str().unwrap_or_default(), actual.as_str().unwrap_or_default(),
      key, "metadata", 0, true)
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
#[allow(unused_variables)]
pub async fn match_message<'a>(
  expected: &Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  actual: &Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>) -> Vec<Mismatch> {
  let mut mismatches = vec![];

  if expected.is_message() && actual.is_message() {
    debug!("comparing to expected message: {:?}", expected);
    let expected_message = expected.as_message().unwrap();
    let actual_message = actual.as_message().unwrap();

    let matching_rules = &expected_message.matching_rules;
    #[allow(unused_mut, unused_assignments)] let mut plugin_data = hashmap!{};
    #[cfg(feature = "plugins")]
    {
      plugin_data = setup_plugin_config(pact, expected, InteractionPart::None);
    };

    let body_context = if expected.is_v4() {
      CoreMatchingContext {
        matchers: matching_rules.rules_for_category("content").unwrap_or_default(),
        config: DiffConfig::AllowUnexpectedKeys,
        matching_spec: PactSpecification::V4,
        plugin_configuration: plugin_data.clone()
      }
    } else {
      CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                           &matching_rules.rules_for_category("body").unwrap_or_default(),
                           &plugin_data)
    };

    let metadata_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                                &matching_rules.rules_for_category("metadata").unwrap_or_default(),
                                                &plugin_data);
    let contents = match_message_contents(&expected_message.as_message_content(), &actual_message.as_message_content(), &body_context).await;

    mismatches.extend_from_slice(contents.err().unwrap_or_default().as_slice());
    for values in match_message_metadata(&expected_message.as_message_content(), &actual_message.as_message_content(), &metadata_context).values() {
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

/// Matches synchronous request/response messages
pub async fn match_sync_message<'a>(expected: SynchronousMessage, actual: SynchronousMessage, pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>) -> Vec<Mismatch> {
  let mut mismatches = match_sync_message_request(&expected, &actual, pact).await;
  let response_result = match_sync_message_response(&expected, &expected.response, &actual.response, pact).await;
  mismatches.extend_from_slice(&*response_result);
  mismatches
}

/// Match the request part of a synchronous request/response message
#[allow(unused_variables)]
pub async fn match_sync_message_request<'a>(
  expected: &SynchronousMessage,
  actual: &SynchronousMessage,
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>
) -> Vec<Mismatch> {
  debug!("comparing to expected message request: {:?}", expected);

  let matching_rules = &expected.request.matching_rules;
  #[allow(unused_mut, unused_assignments)] let mut plugin_data = hashmap!{};
  #[cfg(feature = "plugins")]
  {
    plugin_data = setup_plugin_config(pact, &expected.boxed(), InteractionPart::None);
  };

  let body_context = CoreMatchingContext {
    matchers: matching_rules.rules_for_category("content").unwrap_or_default(),
    config: DiffConfig::AllowUnexpectedKeys,
    matching_spec: PactSpecification::V4,
    plugin_configuration: plugin_data.clone()
  };

  let metadata_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                              &matching_rules.rules_for_category("metadata").unwrap_or_default(),
                                              &plugin_data);
  let contents = match_message_contents(&expected.request, &actual.request, &body_context).await;

  let mut mismatches = vec![];
  mismatches.extend_from_slice(contents.err().unwrap_or_default().as_slice());
  for values in match_message_metadata(&expected.request, &actual.request, &metadata_context).values() {
    mismatches.extend_from_slice(values.as_slice());
  }
  mismatches
}

/// Match the response part of a synchronous request/response message
#[allow(unused_variables)]
pub async fn match_sync_message_response<'a>(
  expected: &SynchronousMessage,
  expected_responses: &[MessageContents],
  actual_responses: &[MessageContents],
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>
) -> Vec<Mismatch> {
  debug!("comparing to expected message responses: {:?}", expected_responses);

  let mut mismatches = vec![];

  if expected_responses.len() != actual_responses.len() {
    if !expected_responses.is_empty() && actual_responses.is_empty() {
      mismatches.push(Mismatch::BodyTypeMismatch {
        expected: "message response".into(),
        actual: "".into(),
        mismatch: "Expected a message with a response, but the actual response was empty".into(),
        expected_body: None,
        actual_body: None
      });
    } else if !expected_responses.is_empty() {
      mismatches.push(Mismatch::BodyTypeMismatch {
        expected: "message response".into(),
        actual: "".into(),
        mismatch: format!("Expected a message with {} responses, but the actual response had {}",
                          expected_responses.len(), actual_responses.len()),
        expected_body: None,
        actual_body: None
      });
    }
  } else {
    #[allow(unused_mut, unused_assignments)] let mut plugin_data = hashmap!{};
    #[cfg(feature = "plugins")]
    {
      plugin_data = setup_plugin_config(pact, &expected.boxed(), InteractionPart::None);
    };
    for (expected_response, actual_response) in expected_responses.iter().zip(actual_responses) {
      let matching_rules = &expected_response.matching_rules;
      let body_context = CoreMatchingContext {
        matchers: matching_rules.rules_for_category("content").unwrap_or_default(),
        config: DiffConfig::AllowUnexpectedKeys,
        matching_spec: PactSpecification::V4,
        plugin_configuration: plugin_data.clone()
      };

      let metadata_context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                                  &matching_rules.rules_for_category("metadata").unwrap_or_default(),
                                                  &plugin_data);
      let contents = match_message_contents(expected_response, actual_response, &body_context).await;

      mismatches.extend_from_slice(contents.err().unwrap_or_default().as_slice());
      for values in match_message_metadata(expected_response, actual_response, &metadata_context).values() {
        mismatches.extend_from_slice(values.as_slice());
      }
    }
  }
  mismatches
}

/// Generates the request by applying any defined generators
// TODO: Need to pass in any plugin data
#[instrument(level = "trace")]
pub async fn generate_request(request: &HttpRequest, mode: &GeneratorTestMode, context: &HashMap<&str, Value>) -> HttpRequest {
  trace!(?request, ?mode, ?context, "generate_request");
  let mut request = request.clone();

  let generators = request.build_generators(&GeneratorCategory::PATH);
  if !generators.is_empty() {
    debug!("Applying path generator...");
    apply_generators(mode, &generators, &mut |_, generator| {
      if let Ok(v) = generator.generate_value(&request.path, context, &DefaultVariantMatcher.boxed()) {
        request.path = v;
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::HEADER);
  if !generators.is_empty() {
    debug!("Applying header generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(header) = key.first_field() {
        if let Some(ref mut headers) = request.headers {
          if headers.contains_key(header) {
            if let Ok(v) = generator.generate_value(&headers.get(header).unwrap().clone(), context, &DefaultVariantMatcher.boxed()) {
              headers.insert(header.to_string(), v);
            }
          } else {
            if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
              headers.insert(header.to_string(), vec![ v.to_string() ]);
            }
          }
        } else {
          if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
            request.headers = Some(hashmap!{
              header.to_string() => vec![ v.to_string() ]
            })
          }
        }
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::QUERY);
  if !generators.is_empty() {
    debug!("Applying query generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(param) = key.first_field() {
        if let Some(ref mut parameters) = request.query {
          if let Some(parameter) = parameters.get_mut(param) {
            let mut generated = parameter.clone();
            for (index, val) in parameter.iter().enumerate() {
              let value = val.clone().unwrap_or_default();
              if let Ok(v) = generator.generate_value(&value, context, &DefaultVariantMatcher.boxed()) {
                generated[index] = Some(v);
              }
            }
            *parameter = generated;
          } else if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
            parameters.insert(param.to_string(), vec![ Some(v.to_string()) ]);
          }
        } else if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
          request.query = Some(hashmap!{
            param.to_string() => vec![ Some(v.to_string()) ]
          })
        }
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && request.body.is_present() {
    debug!("Applying body generators...");
    match generators_process_body(mode, &request.body, request.content_type(),
                                  context, &generators, &DefaultVariantMatcher {}, &vec![], &hashmap!{}).await {
      Ok(body) => request.body = body,
      Err(err) => error!("Failed to generate the body, will use the original: {}", err)
    }
  }

  request
}

/// Generates the response by applying any defined generators
// TODO: Need to pass in any plugin data
pub async fn generate_response(response: &HttpResponse, mode: &GeneratorTestMode, context: &HashMap<&str, Value>) -> HttpResponse {
  trace!(?response, ?mode, ?context, "generate_response");
  let mut response = response.clone();
  let generators = response.build_generators(&GeneratorCategory::STATUS);
  if !generators.is_empty() {
    debug!("Applying status generator...");
    apply_generators(mode, &generators, &mut |_, generator| {
      if let Ok(v) = generator.generate_value(&response.status, context, &DefaultVariantMatcher.boxed()) {
        debug!("Generated value for status: {}", v);
        response.status = v;
      }
    });
  }
  let generators = response.build_generators(&GeneratorCategory::HEADER);
  if !generators.is_empty() {
    debug!("Applying header generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(header) = key.first_field() {
        if let Some(ref mut headers) = response.headers {
          if headers.contains_key(header) {
            if let Ok(v) = generator.generate_value(&headers.get(header).unwrap().clone(), context, &DefaultVariantMatcher.boxed()) {
              headers.insert(header.to_string(), v);
            }
          } else {
            if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
              headers.insert(header.to_string(), vec![ v.to_string() ]);
            }
          }
        } else {
          if let Ok(v) = generator.generate_value(&"".to_string(), context, &DefaultVariantMatcher.boxed()) {
            response.headers = Some(hashmap!{
              header.to_string() => vec![ v.to_string() ]
            })
          }
        }
      }
    });
  }
  let generators = response.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && response.body.is_present() {
    debug!("Applying body generators...");
    match generators_process_body(mode, &response.body, response.content_type(),
      context, &generators, &DefaultVariantMatcher{}, &vec![], &hashmap!{}).await {
      Ok(body) => response.body = body,
      Err(err) => error!("Failed to generate the body, will use the original: {}", err)
    }
  }
  response
}

/// Matches the request part of the interaction
pub async fn match_interaction_request(
  expected: Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  actual: Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  _spec_version: &PactSpecification
) -> anyhow::Result<RequestMatchResult> {
  if let Some(http_interaction) = expected.as_v4_http() {
    let request = actual.as_v4_http()
      .ok_or_else(|| anyhow!("Could not unpack actual request as a V4 Http Request"))?.request;
    match_request(http_interaction.request, request, &pact, &expected).await
  } else {
    Err(anyhow!("match_interaction_request must be called with HTTP request/response interactions, got {}", expected.type_of()))
  }
}

/// Matches the response part of the interaction
pub async fn match_interaction_response(
  expected: Box<dyn Interaction + Sync + RefUnwindSafe>,
  actual: Box<dyn Interaction + Sync + RefUnwindSafe>,
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  _spec_version: &PactSpecification
) -> anyhow::Result<Vec<Mismatch>> {
  if let Some(expected) = expected.as_v4_http() {
    let expected_response = expected.response.clone();
    let expected = expected.boxed();
    let response = actual.as_v4_http()
      .ok_or_else(|| anyhow!("Could not unpack actual response as a V4 Http Response"))?.response;
    match_response(expected_response, response, &pact, &expected).await
  } else {
    Err(anyhow!("match_interaction_response must be called with HTTP request/response interactions, got {}", expected.type_of()))
  }
}

/// Matches an interaction
pub async fn match_interaction(
  expected: Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  actual: Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  _spec_version: &PactSpecification
) -> anyhow::Result<Vec<Mismatch>> {
  if let Some(expected) = expected.as_v4_http() {
    let expected_request = expected.request.clone();
    let expected_response = expected.response.clone();
    let expected = expected.boxed();
    let request = actual.as_v4_http()
      .ok_or_else(|| anyhow!("Could not unpack actual request as a V4 Http Request"))?.request;
    let request_result = match_request(expected_request, request, &pact, &expected).await?;
    let response = actual.as_v4_http()
      .ok_or_else(|| anyhow!("Could not unpack actual response as a V4 Http Response"))?.response;
    let response_result = match_response(expected_response, response, &pact, &expected).await?;
    let mut mismatches = request_result.mismatches();
    mismatches.extend_from_slice(&*response_result);
    Ok(mismatches)
  } else if expected.is_message() || expected.is_v4() {
    Ok(match_message(&expected, &actual, &pact).await)
  } else {
    Err(anyhow!("match_interaction must be called with either an HTTP request/response interaction or a Message, got {}", expected.type_of()))
  }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod generator_tests;
