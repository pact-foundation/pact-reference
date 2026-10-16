//! HTTP parts of a Request/Response interaction

use std::collections::HashMap;
use std::str::from_utf8;

use maplit::hashmap;

use crate::bodies::OptionalBody;
use crate::content_types::{ContentType, detect_content_type_from_string};
use crate::generators::{Generator, GeneratorCategory, Generators};
use crate::matchingrules::{Category, MatchingRules};
use crate::path_exp::DocPath;

/// Trait to specify an HTTP part of an interaction. It encapsulates the shared parts of a request
/// and response.
pub trait HttpPart {
  /// Returns the headers of the HTTP part.
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>>;

  /// Returns the headers of the HTTP part in a mutable form.
  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>>;

  /// Returns the body of the HTTP part.
  fn body(&self) -> &OptionalBody;

  /// Returns a mutable pointer to the body of the HTTP part.
  fn body_mut(&mut self) -> &mut OptionalBody;

  /// Returns the matching rules of the HTTP part.
  fn matching_rules(&self) -> &MatchingRules;

  /// Returns the matching rules of the HTTP part.
  fn matching_rules_mut(&mut self) -> &mut MatchingRules;

  /// Returns the generators of the HTTP part.
  fn generators(&self) -> &Generators;

  /// Returns the generators of the HTTP part.
  fn generators_mut(&mut self) -> &mut Generators;

  /// Lookup up the content type for the part
  fn lookup_content_type(&self) -> Option<String>;

  /// Tries to detect the content type of the body by matching some regular expressions against
  /// the first 32 characters.
  fn detect_content_type(&self) -> Option<ContentType> {
    match *self.body() {
      OptionalBody::Present(ref body, _, _) => {
        let s: String = match from_utf8(body) {
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
  fn has_header(&self, header_name: &str) -> bool {
    self.lookup_header_value(header_name).is_some()
  }

  /// Checks if the HTTP Part has the given header
  fn lookup_header_value(&self, header_name: &str) -> Option<String> {
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
    let str_body = body.value_as_string();
    str_body.map(|b| !b.is_empty() && b.is_ascii()).unwrap_or(false)
  }

  /// Convenience method to add a header
  fn add_header(&mut self, key: &str, val: Vec<&str>) {
    let headers = self.headers_mut();
    headers.insert(key.to_string(), val.iter().map(|v| v.to_string()).collect());
  }

  /// Builds a map of generators from the generators and matching rules
  fn build_generators(&self, category: &GeneratorCategory) -> HashMap<DocPath, Generator> {
    let mut generators = hashmap!{};
    if let Some(generators_for_category) = self.generators().categories.get(category) {
      for (path, generator) in generators_for_category {
        generators.insert(path.clone(), generator.clone());
      }
    }
    let mr_category: Category = category.clone().into();
    if let Some(rules) = self.matching_rules().rules_for_category(mr_category) {
      for (path, generator) in rules.generators() {
        generators.insert(path.clone(), generator.clone());
      }
    }
    generators
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;

  use crate::bodies::OptionalBody;
  use crate::http_parts::HttpPart;
  use crate::request::Request;

  #[test]
  fn http_part_has_header_test() {
    let request = Request { method: "GET".to_string(), path: "/".to_string(), query: None,
      headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json; charset=UTF-8".to_string()] }),
      body: OptionalBody::Missing, .. Request::default() };
    expect!(request.has_header("Content-Type")).to(be_true());
    expect!(request.lookup_header_value("Content-Type")).to(be_some().value("application/json; charset=UTF-8"));
  }
}
