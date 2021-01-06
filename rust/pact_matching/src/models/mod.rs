//! The `models` module provides all the structures required to model a Pact.

use std::{fmt, fs};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::default::Default;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, Error, ErrorKind};
use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::str::from_utf8;

use base64::{decode, encode};
use hex::FromHex;
use itertools::{iproduct, Itertools};
use itertools::EitherOrBoth::{Both, Left, Right};
use lazy_static::*;
use log::*;
use maplit::*;
use onig::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::content_types::ContentType;
use crate::models::http_utils::HttpAuth;
use crate::models::json_utils::json_to_string;
use crate::models::message::Message;
use crate::models::message_pact::MessagePact;
use crate::models::provider_states::ProviderState;
use crate::models::v4::{interaction_from_json, V4Pact, V4Interaction};
use crate::models::v4::http_parts::{HttpRequest, HttpResponse};
use std::borrow::Borrow;
use crate::models::generators::{GeneratorCategory, Generator};

pub mod json_utils;
pub mod xml_utils;
#[macro_use] pub mod matchingrules;
#[macro_use] pub mod generators;
pub mod http_utils;
pub mod content_types;
mod expression_parser;

/// Version of the library
pub const PACT_RUST_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Enum defining the pact specification versions supported by the library
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum PactSpecification {
    /// Unknown or unsupported specification version
    Unknown,
    /// First version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1)
    V1,
    /// Second version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1.1)
    V1_1,
    /// Version two of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-2)
    V2,
    /// Version three of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3)
    V3,
    /// Version four of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-4)
    V4
}

impl Default for PactSpecification {
  fn default() -> Self {
    PactSpecification::Unknown
  }
}

impl PactSpecification {
    /// Returns the semantic version string of the specification version.
    pub fn version_str(&self) -> String {
        match *self {
            PactSpecification::V1 => s!("1.0.0"),
            PactSpecification::V1_1 => s!("1.1.0"),
            PactSpecification::V2 => s!("2.0.0"),
            PactSpecification::V3 => s!("3.0.0"),
            PactSpecification::V4 => s!("4.0"),
            _ => s!("unknown")
        }
    }

    /// Returns a descriptive string of the specification version.
    pub fn to_string(&self) -> String {
        match *self {
            PactSpecification::V1 => s!("V1"),
            PactSpecification::V1_1 => s!("V1.1"),
            PactSpecification::V2 => s!("V2"),
            PactSpecification::V3 => s!("V3"),
            PactSpecification::V4 => s!("V4"),
            _ => s!("unknown")
        }
    }
}

/// Struct that defines the consumer of the pact.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Consumer {
    /// Each consumer should have a unique name to identify it.
    pub name: String
}

impl Consumer {
    /// Builds a `Consumer` from the `Json` struct.
    pub fn from_json(pact_json: &Value) -> Consumer {
        let val = match pact_json.get("name") {
            Some(v) => match v.clone() {
                Value::String(s) => s,
                _ => v.to_string()
            },
            None => "consumer".to_string()
        };
        Consumer { name: val.clone() }
    }

    /// Converts this `Consumer` to a `Value` struct.
    pub fn to_json(&self) -> Value {
        json!({ s!("name") : json!(self.name.clone()) })
    }
}

/// Struct that defines a provider of a pact.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Provider {
    /// Each provider should have a unique name to identify it.
    pub name: String
}

impl Provider {
    /// Builds a `Provider` from a `Value` struct.
    pub fn from_json(pact_json: &Value) -> Provider {
        let val = match pact_json.get("name") {
            Some(v) => match v.clone() {
                Value::String(s) => s,
                _ => v.to_string()
            },
            None => "provider".to_string()
        };
        Provider { name: val.clone() }
    }

    /// Converts this `Provider` to a `Value` struct.
    pub fn to_json(&self) -> Value {
        json!({ s!("name") : json!(self.name.clone()) })
    }
}

/// Enum that defines the four main states that a body of a request and response can be in a pact
/// file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptionalBody {
    /// A body is missing if it is not present in the pact file
    Missing,
    /// An empty body that is present in the pact file.
    Empty,
    /// A JSON body that is the null value. This state is to protect other language implementations
    /// from null values. It is treated as `Empty`.
    Null,
    /// A non-empty body that is present in the pact file.
    Present(Vec<u8>, Option<ContentType>)
}

impl OptionalBody {

    /// If the body is present in the pact file and not empty or null.
    pub fn is_present(&self) -> bool {
        match *self {
            OptionalBody::Present(_, _) => true,
            _ => false
        }
    }

    /// Returns the body if present, otherwise returns the empty Vec.
    pub fn value(&self) -> Vec<u8> {
        match *self {
            OptionalBody::Present(ref s, _) => s.clone(),
            _ => vec![]
        }
    }

  /// Returns the body if present as a string, otherwise returns the empty string.
  pub fn str_value(&self) -> &str {
    match *self {
      OptionalBody::Present(ref s, _) => str::from_utf8(s).unwrap_or(""),
      _ => ""
    }
  }

  /// If the body has a content type associated to it
  pub fn has_content_type(&self) -> bool {
    match self {
      OptionalBody::Present(_, content_type) => content_type.is_some(),
      _ => false
    }
  }

  /// Parsed content type of the body
  pub fn content_type(&self) -> Option<ContentType> {
    match self {
      OptionalBody::Present(_, content_type) =>
        content_type.clone(),
      _ => None
    }
  }

  /// Converts this body into a V4 Pact file JSON format
  pub fn to_v4_json(&self) -> Value {
    match self {
      OptionalBody::Present(bytes, _) => {
        let content_type = self.content_type().unwrap_or_default();
        let (contents, encoded) = if content_type.is_json() {
          match serde_json::from_slice(bytes) {
            Ok(json_body) => (json_body, Value::Bool(false)),
            Err(err) => {
              warn!("Failed to parse json body: {}", err);
              (Value::String(encode(bytes)), Value::String("base64".to_string()))
            }
          }
        } else if content_type.is_binary() {
          (Value::String(encode(bytes)), Value::String("base64".to_string()))
        } else {
          match str::from_utf8(bytes) {
            Ok(s) => (Value::String(s.to_string()), Value::Bool(false)),
            Err(_) => (Value::String(encode(bytes)), Value::String("base64".to_string()))
          }
        };
        json!({
          "content": contents,
          "contentType": content_type.to_string(),
          "encoded": encoded
        })
      },
      OptionalBody::Empty => json!({"content": ""}),
      _ => Value::Null
    }
  }
}

impl From<String> for OptionalBody {
  fn from(s: String) -> Self {
    if s.is_empty() {
      OptionalBody::Empty
    } else {
      OptionalBody::Present(Vec::from(s.as_bytes()), None)
    }
  }
}

impl <'a> From<&'a str> for OptionalBody {
  fn from(s: &'a str) -> Self {
    if s.is_empty() {
      OptionalBody::Empty
    } else {
      OptionalBody::Present(Vec::from(s.as_bytes()), None)
    }
  }
}

impl Display for OptionalBody {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match *self {
      OptionalBody::Missing => write!(f, "Missing"),
      OptionalBody::Empty => write!(f, "Empty"),
      OptionalBody::Null => write!(f, "Null"),
      OptionalBody::Present(ref s, ref content_type) => {
        if let Some(content_type) = content_type {
          write!(f, "Present({} bytes, {})", s.len(), content_type)
        } else {
          write!(f, "Present({} bytes)", s.len())
        }
      }
    }
  }
}

#[cfg(test)]
mod body_tests {
  use expectest::prelude::*;
  use super::content_types::JSON;

  use super::*;

  #[test]
  fn display_tests() {
    expect!(format!("{}", OptionalBody::Missing)).to(be_equal_to("Missing"));
    expect!(format!("{}", OptionalBody::Empty)).to(be_equal_to("Empty"));
    expect!(format!("{}", OptionalBody::Null)).to(be_equal_to("Null"));
    expect!(format!("{}", OptionalBody::Present("hello".into(), None))).to(be_equal_to("Present(5 bytes)"));
    expect!(format!("{}", OptionalBody::Present("\"hello\"".into(), Some(JSON.clone())))).to(be_equal_to("Present(7 bytes, application/json)"));
  }
}

lazy_static! {
    static ref XMLREGEXP: Regex = Regex::new(r"^\s*<\?xml\s*version.*").unwrap();
    static ref HTMLREGEXP: Regex = Regex::new(r"^\s*(<!DOCTYPE)|(<HTML>).*").unwrap();
    static ref JSONREGEXP: Regex = Regex::new(r#"^\s*(true|false|null|[0-9]+|"\w*|\{\s*(}|"\w+)|\[\s*)"#).unwrap();
    static ref XMLREGEXP2: Regex = Regex::new(r#"^\s*<\w+\s*(:\w+=["”][^"”]+["”])?.*"#).unwrap();
}

fn detect_content_type_from_string(s: &String) -> Option<ContentType> {
  log::debug!("Detecting content type from contents: '{}'", s);
  if is_match(&XMLREGEXP, s.as_str()) {
    Some(content_types::XML.clone())
  } else if is_match(&HTMLREGEXP, s.to_uppercase().as_str()) {
    Some(content_types::HTML.clone())
  } else if is_match(&XMLREGEXP2, s.as_str()) {
    Some(content_types::XML.clone())
  } else if is_match(&JSONREGEXP, s.as_str()) {
    Some(content_types::JSON.clone())
  } else {
    Some(content_types::TEXT.clone())
  }
}

fn detect_content_type_from_bytes(s: &[u8]) -> Option<ContentType> {
  log::debug!("Detecting content type from byte contents");
  let header = if s.len() > 32 {
    &s[0..32]
  } else {
    s
  };
  match from_utf8(header) {
    Ok(s) => {
      if is_match(&XMLREGEXP, s) {
        Some(content_types::XML.clone())
      } else if is_match(&HTMLREGEXP, &*s.to_uppercase()) {
        Some(content_types::HTML.clone())
      } else if is_match(&XMLREGEXP2, s) {
        Some(content_types::XML.clone())
      } else if is_match(&JSONREGEXP, s) {
        Some(content_types::JSON.clone())
      } else {
        Some(content_types::TEXT.clone())
      }
    },
    Err(_) => None
  }
}

/// Enumeration of the types of differences between requests and responses
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum DifferenceType {
  /// Methods differ
  Method,
  /// Paths differ
  Path,
  /// Headers differ
  Headers,
  /// Query parameters differ
  QueryParameters,
  /// Bodies differ
  Body,
  /// Matching Rules differ
  MatchingRules,
  /// Response status differ
  Status
}

/// Trait to specify an HTTP part of a message. It encapsulates the shared parts of a request and
/// response.
pub trait HttpPart {
    /// Returns the headers of the HTTP part.
    fn headers(&self) -> &Option<HashMap<String, Vec<String>>>;

    /// Returns the headers of the HTTP part in a mutable form.
    fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>>;

    /// Returns the body of the HTTP part.
    fn body(&self) -> &OptionalBody;

    /// Returns the matching rules of the HTTP part.
    fn matching_rules(&self) -> &matchingrules::MatchingRules;

    /// Returns the generators of the HTTP part.
    fn generators(&self) -> &generators::Generators;

    /// Lookup up the content type for the part
    fn lookup_content_type(&self) -> Option<String>;

    /// Tries to detect the content type of the body by matching some regular expressions against
    /// the first 32 characters.
    fn detect_content_type(&self) -> Option<ContentType> {
      match *self.body() {
        OptionalBody::Present(ref body, _) => {
          let s: String = match str::from_utf8(body) {
            Ok(s) => s.to_string(),
            Err(_) => String::new()
          };
          detect_content_type_from_string(&s)
        },
        _ => None
      }
    }

  /// Determine the content type of the HTTP part. If a `Content-Type` header is present, the
  /// value of that header will be returned. Otherwise, the body will be inspected.
  fn content_type(&self) -> Option<ContentType> {
    let body = self.body();
    if body.has_content_type() {
      body.content_type()
    } else {
      match self.lookup_content_type() {
        Some(ref h) => match ContentType::parse(h.as_str()) {
          Ok(v) => Some(v),
          Err(_) => self.detect_content_type()
        },
        None => self.detect_content_type()
      }
    }
  }

    /// Checks if the HTTP Part has the given header
    fn has_header(&self, header_name: &String) -> bool {
        self.lookup_header_value(header_name).is_some()
    }

    /// Checks if the HTTP Part has the given header
    fn lookup_header_value(&self, header_name: &String) -> Option<String> {
      match *self.headers() {
        Some(ref h) => h.iter()
          .find(|kv| kv.0.to_lowercase() == header_name.to_lowercase())
          .map(|kv| kv.1.clone().join(", ")),
        None => None
      }
    }

  /// If the body is a textual type (non-binary)
  fn has_text_body(&self) -> bool {
    let body = self.body();
    let str_body = body.str_value();
    body.is_present() && !str_body.is_empty() && str_body.is_ascii()
  }

  /// Convenience method to add a header
  fn add_header(&mut self, key: &str, val: Vec<&str>) {
    let headers = self.headers_mut();
    headers.insert(key.to_string(), val.iter().map(|v| v.to_string()).collect());
  }

  /// Builds a map of generators from the generators and matching rules
  fn build_generators(&self, category: &GeneratorCategory) -> HashMap<String, Generator> {
    let mut generators = hashmap!{};
    if let Some(generators_for_category) = self.generators().categories.get(category) {
      for (path, generator) in generators_for_category {
        generators.insert(path.clone(), generator.clone());
      }
    }
    if let Some(rules) = self.matching_rules().rules_for_category(category.clone().into()) {
      for (path, generator) in rules.generators() {
        generators.insert(path.clone(), generator.clone());
      }
    }
    generators
  }
}

fn is_match(regex: &Regex, string: &str) -> bool {
  if let Some(m) = regex.find(string) {
    m.0 == 0
  } else {
    false
  }
}

/// Struct that defines the request.
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Request {
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// Request query string
    pub query: Option<HashMap<String, Vec<String>>>,
    /// Request headers
    pub headers: Option<HashMap<String, Vec<String>>>,
    /// Request body
    pub body: OptionalBody,
    /// Request matching rules
    pub matching_rules: matchingrules::MatchingRules,
    /// Request generators
    pub generators: generators::Generators
}

impl HttpPart for Request {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
        &self.headers
    }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    if self.headers.is_none() {
      self.headers = Some(hashmap!{});
    }
    self.headers.as_mut().unwrap()
  }

  fn body(&self) -> &OptionalBody {
      &self.body
  }

  fn matching_rules(&self) -> &matchingrules::MatchingRules {
      &self.matching_rules
  }

  fn generators(&self) -> &generators::Generators {
      &self.generators
    }

  fn lookup_content_type(&self) -> Option<String> {
    self.lookup_header_value(&"content-type".to_string())
  }
}

impl Hash for Request {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.method.hash(state);
        self.path.hash(state);
        if self.query.is_some() {
            for (k, v) in self.query.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        if self.headers.is_some() {
            for (k, v) in self.headers.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        self.body.hash(state);
        self.matching_rules.hash(state);
        self.generators.hash(state);
    }
}

impl PartialEq for Request {
  fn eq(&self, other: &Self) -> bool {
    self.method == other.method && self.path == other.path && self.query == other.query &&
      self.headers == other.headers && self.body == other.body &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }

  fn ne(&self, other: &Self) -> bool {
    self.method != other.method || self.path != other.path || self.query != other.query ||
      self.headers != other.headers || self.body != other.body ||
      self.matching_rules != other.matching_rules || self.generators != other.generators
  }
}

impl Display for Request {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Request ( method: {}, path: {}, query: {:?}, headers: {:?}, body: {} )",
      self.method, self.path, self.query, self.headers, self.body)
  }
}

impl Default for Request {
  fn default() -> Self {
    Request {
      method: s!("GET"),
      path: s!("/"),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: matchingrules::MatchingRules::default(),
      generators: generators::Generators::default()
    }
  }
}

fn headers_from_json(request: &Value) -> Option<HashMap<String, Vec<String>>> {
  match request.get("headers") {
    Some(v) => match *v {
      Value::Object(ref m) => Some(m.iter().map(|(key, val)| {
        match val {
          &Value::String(ref s) => (key.clone(), s.clone().split(',').map(|v| s!(v.trim())).collect()),
          &Value::Array(ref v) => (key.clone(), v.iter().map(|val| {
            match val {
              &Value::String(ref s) => s.clone(),
              _ => val.to_string()
            }
          }).collect()),
          _ => (key.clone(), vec![val.to_string()])
        }
      }).collect()),
      _ => None
    },
    None => None
  }
}

fn headers_to_json(headers: &HashMap<String, Vec<String>>) -> Value {
  json!(headers.iter().fold(BTreeMap::new(), |mut map, kv| {
    map.insert(kv.0.clone(), Value::String(kv.1.join(", ")));
    map
  }))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum JsonParsable {
    JsonStringValue(String),
    KeyValue(HashMap<String, Value>)
}

fn body_from_json(request: &Value, fieldname: &str, headers: &Option<HashMap<String, Vec<String>>>) -> OptionalBody {
  let content_type = match headers {
    &Some(ref h) => match h.iter().find(|kv| kv.0.to_lowercase() == s!("content-type")) {
      Some(kv) => {
        match ContentType::parse(kv.1[0].as_str()) {
          Ok(v) => Some(v),
          Err(_) => None
        }
      },
      None => None
    },
    &None => None
  };

  match request.get(fieldname) {
    Some(v) => match *v {
      Value::String(ref s) => {
        if s.is_empty() {
          OptionalBody::Empty
        } else {
          let content_type = content_type.unwrap_or_else(|| {
            detect_content_type_from_string(s).unwrap_or_default()
          });
          if content_type.is_json() {
            match serde_json::from_str::<JsonParsable>(&s) {
              Ok(_) => OptionalBody::Present(s.clone().into(), Some(content_type)),
              Err(_) => OptionalBody::Present(format!("\"{}\"", s).into(), Some(content_type))
            }
          } else if content_type.is_text() {
            OptionalBody::Present(s.clone().into(), Some(content_type))
          } else {
            match decode(s) {
              Ok(bytes) => OptionalBody::Present(bytes.clone(), None),
              Err(_) => OptionalBody::Present(s.clone().into(), None)
            }
          }
        }
      },
      Value::Null => OptionalBody::Null,
      _ => OptionalBody::Present(v.to_string().into(), None)
    },
    None => OptionalBody::Missing
  }
}

/// Converts a query string map into a query string
pub fn build_query_string(query: HashMap<String, Vec<String>>) -> String {
    query.into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
        .flat_map(|kv| {
            kv.1.iter()
                .map(|v| format!("{}={}", kv.0, encode_query(v)))
                .collect_vec()
        })
        .join("&")
}

fn query_from_json(query_json: &Value, spec_version: &PactSpecification) -> Option<HashMap<String, Vec<String>>> {
    match query_json {
        &Value::String(ref s) => parse_query_string(s),
        _ => {
            log::warn!("Only string versions of request query strings are supported with specification version {}, ignoring.",
                spec_version.to_string());
            None
        }
    }
}

fn v3_query_from_json(query_json: &Value, spec_version: &PactSpecification) -> Option<HashMap<String, Vec<String>>> {
    match query_json {
        &Value::String(ref s) => parse_query_string(s),
        &Value::Object(ref map) => Some(map.iter().map(|(k, v)| {
            (k.clone(), match v {
                &Value::String(ref s) => vec![s.clone()],
                &Value::Array(ref array) => array.iter().map(|item| match item {
                    &Value::String(ref s) => s.clone(),
                    _ => v.to_string()
                }).collect(),
                _ => {
                    log::warn!("Query paramter value '{}' is not valid, ignoring", v);
                    vec![]
                }
            })
        }).collect()),
        _ => {
            log::warn!("Only string or map versions of request query strings are supported with specification version {}, ignoring.",
                spec_version.to_string());
            None
        }
    }
}

fn query_to_json(query: HashMap<String, Vec<String>>, spec_version: &PactSpecification) -> Value {
  match spec_version {
    &PactSpecification::V3 | &PactSpecification::V4 => Value::Object(query.iter().map(|(k, v)| {
      (k.clone(), Value::Array(v.iter().map(|q| Value::String(q.clone())).collect()))}
    ).collect()),
    _ => Value::String(build_query_string(query))
  }
}

impl Request {
    /// Builds a `Request` from a `Value` struct.
    pub fn from_json(request_json: &Value, spec_version: &PactSpecification) -> Request {
        let method_val = match request_json.get("method") {
            Some(v) => match *v {
                Value::String(ref s) => s.to_uppercase(),
                _ => v.to_string().to_uppercase()
            },
            None => "GET".to_string()
        };
        let path_val = match request_json.get("path") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => "/".to_string()
        };
        let query_val = match request_json.get("query") {
            Some(v) => match spec_version {
                &PactSpecification::V3 => v3_query_from_json(v, spec_version),
                _ => query_from_json(v, spec_version)
            },
            None => None
        };
        let headers = headers_from_json(request_json);
        Request {
            method: method_val,
            path: path_val,
            query: query_val,
            headers: headers.clone(),
            body: body_from_json(request_json, "body", &headers),
            matching_rules: matchingrules::matchers_from_json(request_json, &Some(s!("requestMatchingRules"))),
            generators: generators::generators_from_json(request_json)
        }
    }

    /// Converts this `Request` to a `Value` struct.
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
        let mut json = json!({
            s!("method") : Value::String(self.method.to_uppercase()),
            s!("path") : Value::String(self.path.clone())
        });
        {
            let map = json.as_object_mut().unwrap();
            if self.query.is_some() {
                map.insert(s!("query"), query_to_json(self.query.clone().unwrap(), spec_version));
            }
            if self.headers.is_some() {
                map.insert(s!("headers"), headers_to_json(&self.headers.clone().unwrap()));
            }
            match self.body {
              OptionalBody::Present(ref body, _) => if self.content_type().unwrap_or_default().is_json() {
                match serde_json::from_slice(body) {
                  Ok(json_body) => { map.insert(s!("body"), json_body); },
                  Err(err) => {
                    log::warn!("Failed to parse json body: {}", err);
                    map.insert(s!("body"), Value::String(encode(body)));
                  }
                }
              } else {
                match str::from_utf8(body) {
                  Ok(s) => map.insert(s!("body"), Value::String(s.to_string())),
                  Err(_) => map.insert(s!("body"), Value::String(encode(body)))
                };
              },
              OptionalBody::Empty => { map.insert(s!("body"), Value::String(s!(""))); },
              OptionalBody::Missing => (),
              OptionalBody::Null => { map.insert(s!("body"), Value::Null); }
            }
            if self.matching_rules.is_not_empty() {
                map.insert(s!("matchingRules"), matchingrules::matchers_to_json(
                &self.matching_rules.clone(), spec_version));
            }
            if self.generators.is_not_empty() {
              map.insert(s!("generators"), generators::generators_to_json(
                &self.generators.clone(), spec_version));
            }
        }
        json
    }

    /// Returns the default request: a GET request to the root.
    #[deprecated(since="0.6.0", note="please use `default()` from the standard Default trait instead")]
    pub fn default_request() -> Request {
      Request::default()
    }

    /// Return a description of all the differences from the other request
    pub fn differences_from(&self, other: &Request) -> Vec<(DifferenceType, String)> {
        let mut differences = vec![];
        if self.method != other.method {
            differences.push((DifferenceType::Method, format!("Request method {} != {}", self.method, other.method)));
        }
        if self.path != other.path {
            differences.push((DifferenceType::Path, format!("Request path {} != {}", self.path, other.path)));
        }
        if self.query != other.query {
            differences.push((DifferenceType::QueryParameters, format!("Request query {:?} != {:?}", self.query, other.query)));
        }
        let keys = self.headers.clone().map(|m| m.keys().cloned().collect_vec()).unwrap_or_default();
        let other_keys = other.headers.clone().map(|m| m.keys().cloned().collect_vec()).unwrap_or_default();
        if keys != other_keys {
            differences.push((DifferenceType::Headers, format!("Request headers {:?} != {:?}", self.headers, other.headers)));
        }
        if self.body != other.body {
            differences.push((DifferenceType::Body, format!("Request body '{:?}' != '{:?}'", self.body, other.body)));
        }
        if self.matching_rules != other.matching_rules {
            differences.push((DifferenceType::MatchingRules, format!("Request matching rules {:?} != {:?}", self.matching_rules, other.matching_rules)));
        }
        differences
    }

  /// Convert this interaction to V4 format
  pub fn as_v4_request(&self) -> HttpRequest {
    HttpRequest {
      method: self.method.clone(),
      path: self.path.clone(),
      query: self.query.clone(),
      headers: self.headers.clone(),
      body: self.body.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }
}

/// Struct that defines the response.
#[derive(Debug, Clone, Eq)]
pub struct Response {
    /// Response status
    pub status: u16,
    /// Response headers
    pub headers: Option<HashMap<String, Vec<String>>>,
    /// Response body
    pub body: OptionalBody,
    /// Response matching rules
    pub matching_rules: matchingrules::MatchingRules,
    /// Response generators
    pub generators: generators::Generators
}

impl Response {

    /// Build a `Response` from a `Value` struct.
    pub fn from_json(response: &Value, _: &PactSpecification) -> Response {
        let status_val = match response.get("status") {
            Some(v) => v.as_u64().unwrap() as u16,
            None => 200
        };
        let headers = headers_from_json(response);
        Response {
            status: status_val,
            headers: headers.clone(),
            body: body_from_json(response, "body", &headers),
            matching_rules:  matchingrules::matchers_from_json(response, &Some(s!("responseMatchingRules"))),
            generators:  generators::generators_from_json(response)
        }
    }

    /// Returns a default response: Status 200
    #[deprecated(since="0.5.4", note="please use `default()` from the standard Default trait instead")]
    pub fn default_response() -> Response {
      Response::default()
    }

    /// Converts this response to a `Value` struct.
    #[allow(unused_variables)]
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
      let mut json = json!({
        "status" : json!(self.status)
      });
      {
        let map = json.as_object_mut().unwrap();
        if self.headers.is_some() {
          map.insert(s!("headers"), headers_to_json(&self.headers.clone().unwrap()));
        }
        match self.body {
          OptionalBody::Present(ref body, _) => {
            if self.content_type().unwrap_or_default().is_json() {
              match serde_json::from_slice(body) {
                Ok(json_body) => { map.insert(s!("body"), json_body); },
                Err(err) => {
                  log::warn!("Failed to parse json body: {}", err);
                  map.insert(s!("body"), Value::String(encode(body)));
                }
              }
            } else {
              match str::from_utf8(body) {
                Ok(s) => map.insert(s!("body"), Value::String(s.to_string())),
                Err(_) => map.insert(s!("body"), Value::String(encode(body)))
              };
            }
          },
          OptionalBody::Empty => { map.insert(s!("body"), Value::String(s!(""))); },
          OptionalBody::Missing => (),
          OptionalBody::Null => { map.insert(s!("body"), Value::Null); }
        }
        if self.matching_rules.is_not_empty() {
          map.insert(s!("matchingRules"), matchingrules::matchers_to_json(
            &self.matching_rules.clone(), spec_version));
        }
        if self.generators.is_not_empty() {
          map.insert(s!("generators"), generators::generators_to_json(
            &self.generators.clone(), spec_version));
        }
      }
      json
    }

    /// Return a description of all the differences from the other response
    pub fn differences_from(&self, other: &Response) -> Vec<(DifferenceType, String)> {
        let mut differences = vec![];
        if self.status != other.status {
            differences.push((DifferenceType::Status, format!("Response status {} != {}", self.status, other.status)));
        }
        if self.headers != other.headers {
            differences.push((DifferenceType::Headers, format!("Response headers {:?} != {:?}", self.headers, other.headers)));
        }
        if self.body != other.body {
            differences.push((DifferenceType::Body, format!("Response body '{:?}' != '{:?}'", self.body, other.body)));
        }
        if self.matching_rules != other.matching_rules {
            differences.push((DifferenceType::MatchingRules, format!("Response matching rules {:?} != {:?}", self.matching_rules, other.matching_rules)));
        }
        differences
    }

  /// Convert this response to V4 format
  pub fn as_v4_response(&self) -> HttpResponse {
    HttpResponse {
      status: self.status,
      headers: self.headers.clone(),
      body: self.body.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }
}

impl HttpPart for Response {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
        &self.headers
    }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    if self.headers.is_none() {
      self.headers = Some(hashmap!{});
    }
    self.headers.as_mut().unwrap()
  }

  fn body(&self) -> &OptionalBody {
      &self.body
  }

  fn matching_rules(&self) -> &matchingrules::MatchingRules {
      &self.matching_rules
  }

  fn generators(&self) -> &generators::Generators {
      &self.generators
    }

  fn lookup_content_type(&self) -> Option<String> {
    self.lookup_header_value(&"content-type".to_string())
  }
}

impl Hash for Response {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.status.hash(state);
        if self.headers.is_some() {
            for (k, v) in self.headers.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        self.body.hash(state);
        self.matching_rules.hash(state);
        self.generators.hash(state);
    }
}

impl PartialEq for Response {
  fn eq(&self, other: &Self) -> bool {
    self.status == other.status && self.headers == other.headers && self.body == other.body &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }

  fn ne(&self, other: &Self) -> bool {
    self.status != other.status || self.headers != other.headers || self.body != other.body ||
      self.matching_rules != other.matching_rules || self.generators != other.generators
  }
}

impl Display for Response {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Response ( status: {}, headers: {:?}, body: {} )", self.status, self.headers,
           self.body)
  }
}

impl Default for Response {
  fn default() -> Self {
    Response {
      status: 200,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: matchingrules::MatchingRules::default(),
      generators: generators::Generators::default()
    }
  }
}

pub mod provider_states;

/// Struct that defined an interaction conflict
#[derive(Debug, Clone)]
pub struct PactConflict {
    /// Description of the interactions
    pub interaction: String,
    /// Conflict description
    pub description: String
}

/// Interaction Trait
pub trait Interaction {
  /// The type of the interaction
  fn type_of(&self) -> String;
  /// If this is a request/response interaction
  fn is_request_response(&self) -> bool;
  /// Returns the request/response interaction if it is one
  fn as_request_response(&self) -> Option<RequestResponseInteraction>;
  /// If this is a message interaction
  fn is_message(&self) -> bool;
  /// If this is a plugin based interaction
  fn is_plugin(&self) -> bool;
  /// Returns the message interaction if it is one
  fn as_message(&self) -> Option<Message>;
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  fn id(&self) -> Option<String>;
  /// Description of this interaction. This needs to be unique in the pact file.
  fn description(&self) -> String;
  /// Optional provider states for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  fn provider_states(&self) -> Vec<provider_states::ProviderState>;
  /// Body of the response or message
  fn contents(&self) -> OptionalBody;
  /// Determine the content type of the interaction. If a `Content-Type` header or metadata value is present, the
  /// value of that value will be returned. Otherwise, the contents will be inspected.
  fn content_type(&self) -> Option<ContentType>;
  /// Returns the interaction in V4 format
  fn as_v4(&self) -> V4Interaction;
}

impl Debug for dyn Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(req_res) = self.as_request_response() {
      std::fmt::Debug::fmt(&req_res, f)
    } else if let Some(mp) = self.as_message() {
      std::fmt::Debug::fmt(&mp, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Display for dyn Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(req_res) = self.as_request_response() {
      std::fmt::Display::fmt(&req_res, f)
    } else if let Some(mp) = self.as_message() {
      std::fmt::Display::fmt(&mp, f)
    } else {
      Err(fmt::Error)
    }
  }
}

/// Struct that defines an interaction (request and response pair)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestResponseInteraction {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    pub id: Option<String>,
    /// Description of this interaction. This needs to be unique in the pact file.
    pub description: String,
    /// Optional provider states for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    pub provider_states: Vec<provider_states::ProviderState>,
    /// Request of the interaction
    pub request: Request,
    /// Response of the interaction
    pub response: Response
}

impl Interaction for RequestResponseInteraction {
  fn type_of(&self) -> String {
    "V3 Synchronous/HTTP".into()
  }

  fn is_request_response(&self) -> bool {
    true
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    Some(self.clone())
  }

  fn is_message(&self) -> bool {
    false
  }

  fn is_plugin(&self) -> bool {
    false
  }

  fn as_message(&self) -> Option<Message> {
    None
  }

  fn id(&self) -> Option<String> {
    self.id.clone()
  }

  fn description(&self) -> String {
    self.description.clone()
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    self.provider_states.clone()
  }

  fn contents(&self) -> OptionalBody {
    self.response.body.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.response.content_type()
  }

  fn as_v4(&self) -> V4Interaction {
    V4Interaction::SynchronousHttp {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.as_v4_request(),
      response: self.response.as_v4_response()
    }.with_key()
  }
}

impl RequestResponseInteraction {
    /// Constructs an `Interaction` from the `Value` struct.
    pub fn from_json(index: usize, pact_json: &Value, spec_version: &PactSpecification) -> RequestResponseInteraction {
        let id = pact_json.get("_id").map(|id| json_to_string(id));
        let description = match pact_json.get("description") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => format!("Interaction {}", index)
        };
        let provider_states = provider_states::ProviderState::from_json(pact_json);
        let request = match pact_json.get("request") {
            Some(v) => Request::from_json(v, spec_version),
            None => Request::default()
        };
        let response = match pact_json.get("response") {
            Some(v) => Response::from_json(v, spec_version),
            None => Response::default()
        };
      RequestResponseInteraction {
          id,
          description,
          provider_states,
          request,
          response
        }
    }

    /// Converts this interaction to a `Value` struct.
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
        let mut value = json!({
            s!("description"): Value::String(self.description.clone()),
            s!("request"): self.request.to_json(spec_version),
            s!("response"): self.response.to_json(spec_version)
        });
        if !self.provider_states.is_empty() {
            let map = value.as_object_mut().unwrap();
            match spec_version {
                &PactSpecification::V3 => map.insert(s!("providerStates"),
                                                     Value::Array(self.provider_states.iter().map(|p| p.to_json()).collect())),
                _ => map.insert(s!("providerState"), Value::String(
                    self.provider_states.first().unwrap().name.clone()))
            };
        }
        value
    }

    /// Returns list of conflicts if this interaction conflicts with the other interaction.
    ///
    /// Two interactions conflict if they have the same description and provider state, but they request and
    /// responses are not equal
    pub fn conflicts_with(&self, other: &dyn Interaction) -> Vec<PactConflict> {
      if let Some(other) = other.as_request_response() {
        if self.description == other.description && self.provider_states == other.provider_states {
          let mut conflicts = self.request.differences_from(&other.request).iter()
            .filter(|difference| match difference.0 {
              DifferenceType::MatchingRules | DifferenceType::Body => false,
              _ => true
            })
            .map(|difference| PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
            .collect::<Vec<PactConflict>>();
          for difference in self.response.differences_from(&other.response) {
            match difference.0 {
              DifferenceType::MatchingRules | DifferenceType::Body => (),
              _ => conflicts.push(PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
            };
          }
          conflicts
        } else {
          vec![]
        }
      } else {
        vec![PactConflict {
          interaction: self.description.clone(),
          description: format!("You can not combine message and request/response interactions")
        }]
      }
    }
}

impl Default for RequestResponseInteraction {
  fn default() -> Self {
    RequestResponseInteraction {
      id: None,
      description: s!("Default Interaction"),
      provider_states: vec![],
      request: Request::default(),
      response: Response::default()
    }
  }
}

impl Display for RequestResponseInteraction {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Interaction ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           self.id, self.description, self.provider_states, self.request, self.response)
  }
}

/// Trait for a Pact (request/response or message)
pub trait Pact {
  /// Consumer side of the pact
  fn consumer(&self) -> Consumer;
  /// Provider side of the pact
  fn provider(&self) -> Provider;
  /// Interactions in the Pact
  fn interactions(&self) -> Vec<&dyn Interaction>;
  /// Pact metadata
  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>>;
  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> Value;
  /// Attempt to downcast to a concrete Pact
  fn as_request_response_pact(&self) -> Result<RequestResponsePact, String>;
  /// Attempt to downcast to a concrete Message Pact
  fn as_message_pact(&self) -> Result<MessagePact, String>;
  /// Attempt to downcast to a concrete V4 Pact
  fn as_v4_pact(&self) -> Result<V4Pact, String>;
  /// Specification version of this Pact
  fn specification_version(&self) -> PactSpecification;
}

pub mod message;
pub mod message_pact;
pub mod v4;

/// Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestResponsePact {
    /// Consumer side of the pact
    pub consumer: Consumer,
    /// Provider side of the pact
    pub provider: Provider,
    /// List of interactions between the consumer and provider.
    pub interactions: Vec<RequestResponseInteraction>,
    /// Metadata associated with this pact file.
    pub metadata: BTreeMap<String, BTreeMap<String, String>>,
    /// Specification version of this pact
    pub specification_version: PactSpecification
}

impl Pact for RequestResponsePact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i as &dyn Interaction).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.clone()
  }

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> Value {
    json!({
            s!("consumer"): self.consumer.to_json(),
            s!("provider"): self.provider.to_json(),
            s!("interactions"): Value::Array(self.interactions.iter().map(|i| i.to_json(&pact_spec)).collect()),
            s!("metadata"): json!(self.metadata_to_json(&pact_spec))
        })
  }

  fn as_request_response_pact(&self) -> Result<RequestResponsePact, String> {
    Ok(self.clone())
  }

  fn as_message_pact(&self) -> Result<MessagePact, String> {
    Err(format!("Can't convert a Request/response Pact to a different type"))
  }

  fn as_v4_pact(&self) -> Result<V4Pact, String> {
    Err(format!("Can't convert a Request/response Pact to a different type"))
  }

  fn specification_version(&self) -> PactSpecification {
    self.specification_version.clone()
  }
}

fn parse_meta_data(pact_json: &Value) -> BTreeMap<String, BTreeMap<String, String>> {
    match pact_json.get("metadata") {
        Some(v) => match *v {
            Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                let val = match *v {
                    Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                        match *v {
                            Value::String(ref s) => (k.clone(), s.clone()),
                            _ => (k.clone(), v.to_string())
                        }
                    }).collect(),
                    _ => btreemap!{}
                };
                let key = match k.as_str() {
                  "pact-specification" => s!("pactSpecification"),
                  "pact-rust" => s!("pactRust"),
                  _ => k.clone()
                };
                (key, val)
            }).collect(),
            _ => btreemap!{}
        },
        None => btreemap!{}
    }
}

fn parse_interactions(pact_json: &Value, spec_version: PactSpecification) -> Vec<RequestResponseInteraction> {
    match pact_json.get("interactions") {
        Some(v) => match *v {
            Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
              RequestResponseInteraction::from_json(index, ijson, &spec_version)
            }).collect(),
            _ => vec![]
        },
        None => vec![]
    }
}

fn determine_spec_version(file: &String, metadata: &BTreeMap<String, BTreeMap<String, String>>) -> PactSpecification {
  let specification = if metadata.contains_key("pact-specification") {
    metadata.get("pact-specification")
  } else {
    metadata.get("pactSpecification")
  };
  match specification {
    Some(spec) => {
      match spec.get("version") {
        Some(ver) => match lenient_semver::parse(ver) {
          Ok(ver) => match ver.major {
            1 => match ver.minor {
              0 => PactSpecification::V1,
              1 => PactSpecification::V1_1,
              _ => {
                log::warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V1 specification", ver, file);
                PactSpecification::V1
              }
            },
            2 => PactSpecification::V2,
            3 => PactSpecification::V3,
            4 => PactSpecification::V4,
            _ => {
                log::warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V3 specification", ver, file);
                PactSpecification::Unknown
            }
          },
          Err(err) => {
            log::warn!("Could not parse specification version '{}' found in the metadata in the pact file {:?}, assuming V3 specification - {}", ver, file, err);
            PactSpecification::Unknown
          }
        },
        None => {
          log::warn!("No specification version found in the metadata in the pact file {:?}, assuming V3 specification", file);
          PactSpecification::V3
        }
      }
    },
    None => {
      log::warn!("No metadata found in pact file {:?}, assuming V3 specification", file);
      PactSpecification::V3
    }
  }
}

impl RequestResponsePact {

    /// Returns the specification version of this pact
    pub fn spec_version(&self) -> PactSpecification {
        determine_spec_version(&s!("<Pact>"), &self.metadata)
    }

    /// Creates a `Pact` from a `Value` struct.
    pub fn from_json(file: &String, pact_json: &Value) -> RequestResponsePact {
        let metadata = parse_meta_data(pact_json);
        let spec_version = determine_spec_version(file, &metadata);

        let consumer = match pact_json.get("consumer") {
            Some(v) => Consumer::from_json(v),
            None => Consumer { name: s!("consumer") }
        };
        let provider = match pact_json.get("provider") {
            Some(v) => Provider::from_json(v),
            None => Provider { name: s!("provider") }
        };
        RequestResponsePact {
            consumer,
            provider,
            interactions: parse_interactions(pact_json, spec_version.clone()),
            metadata,
            specification_version: spec_version.clone()
        }
    }

    /// Creates a BTreeMap of the metadata of this pact.
    pub fn metadata_to_json(&self, pact_spec: &PactSpecification) -> BTreeMap<String, Value> {
        let mut md_map: BTreeMap<String, Value> = self.metadata.iter()
            .map(|(k, v)| {
                let key = match k.as_str() {
                  "pact-specification" => s!("pactSpecification"),
                  "pact-rust" => s!("pactRust"),
                  _ => k.clone()
                };
                (key, json!(v.iter()
                  .map(|(k, v)| (k.clone(), v.clone()))
                  .collect::<BTreeMap<String, String>>()))
            })
            .collect();

        md_map.insert(s!("pactSpecification"), json!({"version" : pact_spec.version_str()}));
        md_map.insert(s!("pactRust"), json!({"version" : s!(PACT_RUST_VERSION.unwrap_or("unknown"))}));
        md_map
    }

    /// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
    pub fn from_url(url: &String, auth: &Option<HttpAuth>) -> Result<RequestResponsePact, String> {
      http_utils::fetch_json_from_url(url, auth).map(|(ref url, ref json)| RequestResponsePact::from_json(url, json))
    }

    /// Returns a default RequestResponsePact struct
    pub fn default() -> RequestResponsePact {
      RequestResponsePact {
            consumer: Consumer { name: s!("default_consumer") },
            provider: Provider { name: s!("default_provider") },
            interactions: Vec::new(),
            metadata: RequestResponsePact::default_metadata(),
            specification_version: PactSpecification::V3
        }
    }

  /// Returns the default metadata
  pub fn default_metadata() -> BTreeMap<String, BTreeMap<String, String>> {
    btreemap!{
      s!("pact-specification") => btreemap!{ s!("version") => PactSpecification::V3.version_str() },
      s!("pact-rust") => btreemap!{ s!("version") => s!(PACT_RUST_VERSION.unwrap_or("unknown")) }
    }
  }
}

impl ReadWritePact for RequestResponsePact {
  fn read_pact(path: &Path) -> io::Result<RequestResponsePact> {
    let mut f = File::open(path)?;
    let pact_json = serde_json::from_reader(&mut f);
    match pact_json {
      Ok(ref json) => Ok(RequestResponsePact::from_json(&format!("{:?}", path), json)),
      Err(err) => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - {}", err)))
    }
  }

  fn merge(&self, pact: &dyn Pact) -> Result<RequestResponsePact, String> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let conflicts = iproduct!(self.interactions.clone(), pact.interactions().clone())
        .map(|i| i.0.conflicts_with(i.1))
        .filter(|conflicts| !conflicts.is_empty())
        .collect::<Vec<Vec<PactConflict>>>();
      let num_conflicts = conflicts.len();
      if num_conflicts > 0 {
        warn!("The following conflicting interactions where found:");
        for interaction_conflicts in conflicts {
          warn!(" Interaction '{}':", interaction_conflicts.first().unwrap().interaction);
          for conflict in interaction_conflicts {
            warn!("   {}", conflict.description);
          }
        }
        Err(format!("Unable to merge pacts, as there were {} conflict(s) between the interactions. Please clean out your pact directory before running the tests.",
                    num_conflicts))
      } else {
        let interactions: Vec<Result<RequestResponseInteraction, String>> = self.interactions.iter()
          .merge_join_by(pact.interactions().iter(), |a, b| {
            let cmp = Ord::cmp(&a.provider_states.iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                               &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>());
            if cmp == Ordering::Equal {
              Ord::cmp(&a.description, &b.description())
            } else {
              cmp
            }
          })
          .map(|either| match either {
            Left(i) => Ok(i.clone()),
            Right(i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of())),
            Both(_, i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of()))
          })
          .collect();

        let errors: Vec<String> = interactions.iter()
          .filter(|i| i.is_err())
          .map(|i| i.as_ref().unwrap_err().to_string())
          .collect();
        if errors.is_empty() {
          Ok(RequestResponsePact {
            provider: self.provider.clone(),
            consumer: self.consumer.clone(),
            interactions: interactions.iter()
              .filter(|i| i.is_ok())
              .map(|i| i.as_ref().unwrap().clone()).collect(),
            metadata: self.metadata.clone(),
            specification_version: self.specification_version.clone()
          })
        } else {
          Err(format!("Unable to merge pacts: {}", errors.join(", ")))
        }
      }
    } else {
      Err(s!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

fn decode_query(query: &str) -> String {
    let mut chars = query.chars();
    let mut ch = chars.next();
    let mut result = String::new();

    while ch.is_some() {
        let c = ch.unwrap();
        if c == '%' {
            let c1 = chars.next();
            let c2 = chars.next();
            match (c1, c2) {
                (Some(v1), Some(v2)) => {
                    let mut s = String::new();
                    s.push(v1);
                    s.push(v2);
                    let decoded: Result<Vec<u8>, _> = FromHex::from_hex(s.into_bytes());
                    match decoded {
                        Ok(n) => result.push(n[0] as char),
                        Err(_) => {
                            result.push('%');
                            result.push(v1);
                            result.push(v2);
                        }
                    }
                },
                (Some(v1), None) => {
                    result.push('%');
                    result.push(v1);
                },
                _ => result.push('%')
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }

        ch = chars.next();
    }

    result
}

fn encode_query(query: &str) -> String {
  query.chars().map(|ch| {
    match ch {
      ' ' => s!("+"),
      '-' => ch.to_string(),
      'a'..='z' => ch.to_string(),
      'A'..='Z' => ch.to_string(),
      '0'..='9' => ch.to_string(),
      _ => ch.escape_unicode()
          .filter(|u| u.is_digit(16))
          .batching(|it| {
              match it.next() {
                  None => None,
                  Some(x) => Some((x, it.next().unwrap()))
              }
          })
          .map(|u| format!("%{}{}", u.0, u.1))
          .collect()
    }
  }).collect()
}

/// Parses a query string into an optional map. The query parameter name will be mapped to
/// a list of values. Where the query parameter is repeated, the order of the values will be
/// preserved.
pub fn parse_query_string(query: &String) -> Option<HashMap<String, Vec<String>>> {
    if !query.is_empty() {
        Some(query.split("&").map(|kv| {
            if kv.is_empty() {
                vec![]
            } else if kv.contains("=") {
                kv.splitn(2, "=").collect::<Vec<&str>>()
            } else {
                vec![kv]
            }
        }).fold(HashMap::new(), |mut map, name_value| {
            if !name_value.is_empty() {
                let name = decode_query(name_value[0]);
                let value = if name_value.len() > 1 {
                    decode_query(name_value[1])
                } else {
                    s!("")
                };
                map.entry(name).or_insert(vec![]).push(value);
            }
            map
        }))
    } else {
        None
    }
}

/// Converts the JSON struct into an HTTP Interaction
pub fn http_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> Result<Box<dyn Interaction>, String> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| Box::new(i) as Box<dyn Interaction>),
    _ => Ok(Box::new(RequestResponseInteraction::from_json(0, json, spec)))
  }
}

/// Converts the JSON struct into a Message Interaction
pub fn message_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> Result<Box<dyn Interaction>, String> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| Box::new(i) as Box<dyn Interaction>),
    _ => Message::from_json(0, json, spec)
      .map(|i| Box::new(i) as Box<dyn Interaction>)
  }
}

/// Reads the pact file and parses the resulting JSON into a `Pact` struct
pub fn read_pact(file: &Path) -> io::Result<Box<dyn Pact>> {
  let mut f = File::open(file)?;
  let pact_json = serde_json::from_reader(&mut f);
  match pact_json {
    Ok(ref json) => {
      match json {
        Value::Object(map) => if map.contains_key("messages") {
          match MessagePact::from_json(&format!("{:?}", file), json) {
            Ok(pact) => Ok(Box::new(pact)),
            Err(err) => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - {}", err)))
          }
        } else {
          let metadata = parse_meta_data(json);
          let file_str = file.to_string_lossy();
          let spec_version = determine_spec_version(&file_str.to_string(), &metadata);
          match spec_version {
            PactSpecification::V4 => v4::from_json(&file_str, json)
              .map_err(|err| Error::new(ErrorKind::Other, err)),
            _ => Ok(Box::new(RequestResponsePact::from_json(&format!("{:?}", file), json)))
          }
        },
        _ => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - it is not a valid pact file")))
      }
    },
    Err(err) => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - {}", err)))
  }
}

/// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
pub fn load_pact_from_url<'a>(url: &String, auth: &Option<HttpAuth>) -> Result<Box<dyn Pact>, String> {
  match http_utils::fetch_json_from_url(url, auth) {
    Ok((ref url, ref json)) => match json {
      Value::Object(map) => if map.contains_key("messages") {
        match MessagePact::from_json(&format!("{:?}", url), json) {
          Ok(pact) => Ok(Box::new(pact)),
          Err(err) => Err(err)
        }
      } else {
        let metadata = parse_meta_data(json);
        let spec_version = determine_spec_version(url, &metadata);
        match spec_version {
          PactSpecification::V4 => v4::from_json(&url, json),
          _ => Ok(Box::new(RequestResponsePact::from_json(url, json)))
        }
      },
      _ => Err(format!("Failed to parse Pact JSON from URL '{}' - it is not a valid pact file", url))
    },
    Err(err) => Err(err)
  }
}

/// Trait for objects that can represent Pacts and can be read and written
pub trait ReadWritePact {
  /// Reads the pact file and parses the resulting JSON into a `Pact` struct
  fn read_pact(path: &Path) -> io::Result<Self> where Self: std::marker::Sized;

  /// Merges this pact with the other pact, and returns a new Pact with the interactions sorted.
  /// Returns an error if there is a merge conflict, which will occur if the other pact is a different
  /// type, or if a V3 Pact then if any interaction has the
  /// same description and provider state and the requests and responses are different.
  fn merge(&self, other: &dyn Pact) -> Result<Self, String> where Self: std::marker::Sized;

  /// Determines the default file name for the pact. This is based on the consumer and
  /// provider names.
  fn default_file_name(&self) -> String;
}

/// Writes the pact out to the provided path. If there is an existing pact at the path, the two
/// pacts will be merged together. Returns an error if the file can not be written or the pacts
/// can no be merged.
pub fn write_pact<T: ReadWritePact + Pact + Debug>(pact: &T, path: &Path, pact_spec: PactSpecification) -> io::Result<()> {
  fs::create_dir_all(path.parent().unwrap())?;
  if path.exists() {
    debug!("Merging pact with file {:?}", path);
    let existing_pact = read_pact(path)?;

    if existing_pact.specification_version() < pact.specification_version() {
      warn!("Note: Existing pact is an older specification version ({:?}), and will be upgraded",
        existing_pact.specification_version());
    }

    match pact.merge(existing_pact.borrow()) {
      Ok(ref merged_pact) => {
        let mut file = File::create(path)?;
        let result = serde_json::to_string_pretty(&merged_pact.to_json(pact_spec))?;
        file.write_all(result.as_bytes())?;
        Ok(())
      },
      Err(ref message) => Err(Error::new(ErrorKind::Other, message.clone()))
    }
  } else {
    debug!("Writing new pact file to {:?}", path);
    let mut file = File::create(path)?;
    let result = serde_json::to_string_pretty(&pact.to_json(pact_spec))?;
    file.write_all(result.as_bytes())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests;
