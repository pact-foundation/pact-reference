//! Structs to model an HTTP request

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::from_utf8;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use itertools::Itertools;
use maplit::hashmap;
use serde_json::{json, Value};
use tracing::warn;

use crate::{DifferenceType, PactSpecification};
use crate::bodies::OptionalBody;
use crate::generators::{Generators, generators_from_json, generators_to_json};
use crate::http_parts::HttpPart;
use crate::json_utils::{body_from_json, headers_from_json, headers_to_json};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::query_strings::{query_from_json, query_to_json, v3_query_from_json};
use crate::v4::http_parts::HttpRequest;

/// Struct that defines the request.
#[derive(Debug, Clone, Eq)]
pub struct Request {
  /// Request method
  pub method: String,
  /// Request path
  pub path: String,
  /// Request query string
  pub query: Option<HashMap<String, Vec<Option<String>>>>,
  /// Request headers
  pub headers: Option<HashMap<String, Vec<String>>>,
  /// Request body
  pub body: OptionalBody,
  /// Request matching rules
  pub matching_rules: MatchingRules,
  /// Request generators
  pub generators: Generators
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

  fn body_mut(&mut self) -> &mut OptionalBody {
    &mut self.body
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn matching_rules_mut(&mut self) -> &mut MatchingRules {
    &mut self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn generators_mut(&mut self) -> &mut Generators {
    &mut self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.lookup_header_value(&"content-type".to_string())
  }
}

impl Hash for Request {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.method.hash(state);
    self.path.hash(state);
    if let Some(query) = &self.query {
      for (k, v) in query.iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
        k.hash(state);
        v.hash(state);
      }
    }
    if let Some(headers) = &self.headers {
      for (k, v) in headers.iter().sorted_by(|(a, _), (b, _)| Ord::cmp(a, b)) {
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
      method: "GET".to_string(),
      path: "/".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: MatchingRules::default(),
      generators: Generators::default()
    }
  }
}

impl Request {
  /// Builds a `Request` from a `Value` struct.
  pub fn from_json(request_json: &Value, spec_version: &PactSpecification
  ) -> anyhow::Result<Request> {
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
    Ok(Request {
      method: method_val,
      path: path_val,
      query: query_val,
      headers: headers.clone(),
      body: body_from_json(request_json, "body", &headers),
      matching_rules: matchers_from_json(request_json, &Some("requestMatchingRules".to_string()))?,
      generators: generators_from_json(request_json)?,
    })
  }

  /// Converts this `Request` to a `Value` struct.
  pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
    let mut json = json!({
            "method".to_string() : Value::String(self.method.to_uppercase()),
            "path".to_string() : Value::String(self.path.clone())
        });
    {
      let map = json.as_object_mut().unwrap();

      if self.query.is_some() {
        map.insert("query".to_string(), query_to_json(self.query.clone().unwrap(), spec_version));
      }

      if self.headers.is_some() {
        map.insert("headers".to_string(), headers_to_json(&self.headers.clone().unwrap()));
      }

      match self.body {
        OptionalBody::Present(ref body, _, _) => if self.content_type().unwrap_or_default().is_json() {
          match serde_json::from_slice(body) {
            Ok(json_body) => { map.insert("body".to_string(), json_body); },
            Err(err) => {
              warn!("Failed to parse json body: {}", err);
              map.insert("body".to_string(), Value::String(BASE64.encode(body)));
            }
          }
        } else {
          match from_utf8(body) {
            Ok(s) => map.insert("body".to_string(), Value::String(s.to_string())),
            Err(_) => map.insert("body".to_string(), Value::String(BASE64.encode(body)))
          };
        },
        OptionalBody::Empty => { map.insert("body".to_string(), Value::String(String::default())); },
        OptionalBody::Missing => (),
        OptionalBody::Null => { map.insert("body".to_string(), Value::Null); }
      }

      if self.matching_rules.is_not_empty() {
        map.insert("matchingRules".to_string(), matchers_to_json(
          &self.matching_rules.clone(), spec_version));
      }

      if self.generators.is_not_empty() {
        map.insert("generators".to_string(), generators_to_json(
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
    let mut keys = self.headers.clone().map(|m| m.keys().cloned().collect_vec()).unwrap_or_default();
    let mut other_keys = other.headers.clone().map(|m| m.keys().cloned().collect_vec()).unwrap_or_default();
    keys.sort();
    other_keys.sort();
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

#[cfg(test)]
mod tests {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::Value;

  use crate::bodies::OptionalBody;
  use crate::content_types::{HTML, JSON, XML};
  use crate::generators::{Generator, GeneratorCategory, Generators};
  use crate::http_parts::HttpPart;
  use crate::matchingrules::{Category, MatchingRule, MatchingRuleCategory, MatchingRules, RuleList, RuleLogic};
  use crate::PactSpecification;
  use crate::path_exp::DocPath;
  use crate::request::Request;

  #[test]
  fn request_from_json_defaults_to_get() {
    let request_json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    let request = Request::from_json(&request_json, &PactSpecification::V1);
    expect!(request.unwrap().method).to(be_equal_to("GET"));
  }

  #[test]
  fn request_from_json_defaults_to_root_for_path() {
    let request_json : serde_json::Value = serde_json::from_str(r#"
      {
          "method": "PUT",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    println!("request_json: {}", request_json);
    let request = Request::from_json(&request_json, &PactSpecification::V1_1);
    assert_eq!(request.unwrap().path, "/".to_string());
  }

  #[test]
  fn request_content_type_is_based_on_the_content_type_header() {
    let request = Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      ..Request::default()
    };
    expect!(request.content_type().unwrap_or_default().to_string()).to(be_equal_to("*/*"));
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/html"));
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json; charset=UTF-8".to_string()] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json;charset=utf-8"));
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        headers: Some(hashmap!{ "CONTENT-TYPE".to_string() => vec!["application/json; charset=UTF-8".to_string()] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json;charset=utf-8"));
    expect!(Request {
        body: OptionalBody::Present("{\"json\": true}".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("{}".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("[]".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("[1,2,3]".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("\"string\"".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<json>false</json>".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/xml"));
    expect!(Request {
        body: OptionalBody::Present("<json>false</json>".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/xml"));
    expect!(Request {
        body: OptionalBody::Present("this is not json".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/plain"));
    expect!(Request {
        body: OptionalBody::Present("<html><body>this is also not json</body></html>".into(), None, None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/html"));
  }

  #[test]
  fn content_type_struct_test() {
    let request = Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      ..Request::default()
    };
    expect!(request.content_type()).to(be_none());
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }), .. request.clone() }.content_type())
      .to(be_some().value(HTML.clone()));
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }), .. request.clone() }.content_type())
      .to(be_some().value(JSON.clone()));
    expect!(Request {
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/hal+json".to_string()] }), .. request.clone() }
        .content_type().map(|c| c.base_type()))
      .to(be_some().value(JSON.clone()));
    expect!(Request {
        headers: Some(hashmap!{ "CONTENT-TYPE".to_string() => vec!["application/xml".to_string()] }), .. request.clone() }.content_type())
      .to(be_some().value(XML.clone()));
    expect!(Request {
        headers: Some(hashmap!{ "CONTENT-TYPE".to_string() => vec!["application/stuff+xml".to_string()] }), ..
        request.clone() }.content_type().map(|c| c.base_type()))
      .to(be_some().value(XML.clone()));
  }

  #[test]
  fn request_to_json_with_defaults() {
    let request = Request::default();
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to("{\"method\":\"GET\",\"path\":\"/\"}"));
  }

  #[test]
  fn request_to_json_converts_methods_to_upper_case() {
    let request = Request { method: "post".to_string(), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(be_equal_to("{\"method\":\"POST\",\"path\":\"/\"}"));
  }

  #[test]
  fn request_to_json_with_a_query() {
    let request = Request { query: Some(hashmap!{
        "a".to_string() => vec![Some("1".to_string()), Some("2".to_string())],
        "b".to_string() => vec![Some("3".to_string())]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":"a=1&b=3&a=2"}"#)
    );
  }

  #[test]
  fn request_to_json_with_a_query_must_encode_the_query() {
    let request = Request { query: Some(hashmap!{
        "datetime".to_string() => vec![Some("2011-12-03T10:15:30+01:00".to_string())],
        "description".to_string() => vec![Some("hello world!".to_string())] }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":"datetime=2011-12-03T10%3a15%3a30%2b01%3a00&description=hello+world%21"}"#)
    );
  }

  #[test]
  fn request_to_json_with_a_query_must_encode_the_query_with_utf8_chars() {
    let request = Request { query: Some(hashmap!{
        "a".to_string() => vec![Some("b=c&d❤".to_string())]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":"a=b%3dc%26d%27%64"}"#)
    );
  }

  #[test]
  fn request_to_json_with_a_query_v3() {
    let request = Request { query: Some(hashmap!{
        "a".to_string() => vec![Some("1".to_string()), Some("2".to_string())],
        "b".to_string() => vec![Some("3".to_string())]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["1","2"],"b":["3"]}}"#)
    );
  }

  #[test]
  fn request_to_json_with_a_query_v3_must_not_encode_the_query() {
    let request = Request { query: Some(hashmap!{
        "datetime".to_string() => vec![Some("2011-12-03T10:15:30+01:00".to_string())],
        "description".to_string() => vec![Some("hello world!".to_string())] }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":{"datetime":["2011-12-03T10:15:30+01:00"],"description":["hello world!"]}}"#)
    );
  }

  #[test]
  fn request_to_json_with_a_query_v3_must_not_encode_the_query_with_utf8_chars() {
    let request = Request { query: Some(hashmap!{
        "a".to_string() => vec![Some("b=c&d❤".to_string())]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["b=c&d❤"]}}"#)
    );
  }

  #[test]
  fn request_to_json_with_headers() {
    let request = Request { headers: Some(hashmap!{
        "HEADERA".to_string() => vec!["VALUEA".to_string()],
        "HEADERB".to_string() => vec!["VALUEB1, VALUEB2".to_string()]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"headers":{"HEADERA":"VALUEA","HEADERB":"VALUEB1, VALUEB2"},"method":"GET","path":"/"}"#)
    );
  }

  #[test]
  fn request_to_json_with_json_body() {
    let request = Request { headers: Some(hashmap!{
        "Content-Type".to_string() => vec!["application/json".to_string()]
    }), body: OptionalBody::Present(r#"{"key": "value"}"#.into(), None, None), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"body":{"key":"value"},"headers":{"Content-Type":"application/json"},"method":"GET","path":"/"}"#)
    );
  }


  #[test]
  fn request_to_json_with_non_json_body() {
    let request = Request { headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/plain".to_string()] }),
      body: OptionalBody::Present("This is some text".into(), None, None), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"body":"This is some text","headers":{"Content-Type":"text/plain"},"method":"GET","path":"/"}"#)
    );
  }

  #[test]
  fn request_to_json_with_empty_body() {
    let request = Request { body: OptionalBody::Empty, .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"body":"","method":"GET","path":"/"}"#)
    );
  }

  #[test]
  fn request_to_json_with_null_body() {
    let request = Request { body: OptionalBody::Null, .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
      be_equal_to(r#"{"body":null,"method":"GET","path":"/"}"#)
    );
  }

  fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
  }

  #[test]
  fn hash_for_request() {
    let request1 = Request::default();
    let request2 = Request { method: "POST".to_string(), .. Request::default() };
    let request3 = Request { headers: Some(hashmap!{
        "H1".to_string() => vec!["A".to_string()]
    }), .. Request::default() };
    let request4 = Request { headers: Some(hashmap!{
        "H1".to_string() => vec!["B".to_string()]
    }), .. Request::default() };
    expect!(hash(&request1)).to(be_equal_to(hash(&request1)));
    expect!(hash(&request3)).to(be_equal_to(hash(&request3)));
    expect!(hash(&request1)).to_not(be_equal_to(hash(&request2)));
    expect!(hash(&request3)).to_not(be_equal_to(hash(&request4)));
  }

  #[test]
  fn request_headers_do_not_conflict_if_they_have_been_serialised_and_deserialised_to_json() {
    // headers are serialised in a hashmap; serializing and deserializing can can change the
    // internal order of the keys in the hashmap, and this can confuse the differences_from code.
    let original_request = Request {
      method: "".to_string(),
      path: "".to_string(),
      query: None,
      headers: Some(hashmap! {
          "accept".to_string() => vec!["application/xml".to_string(), "application/json".to_string()],
          "user-agent".to_string() => vec!["test".to_string(), "test2".to_string()],
          "content-type".to_string() => vec!["text/plain".to_string()]
        }),
      body: OptionalBody::Missing,
      matching_rules: Default::default(),
      generators: Default::default(),
    };

    let json = serde_json::to_string(&original_request
      .to_json(&PactSpecification::V3)).expect("could not serialize");

    let serialized_and_deserialized_request: Value =
      serde_json::from_str(&json).expect("could not deserialize");

    expect!(original_request
        .differences_from(&Request::from_json(&serialized_and_deserialized_request, &PactSpecification::V3).unwrap())
        .iter())
      .to(be_empty());
  }

  // Issue https://github.com/pact-foundation/pact-js-core/issues/400
  #[test]
  fn to_json_with_provider_state_generator_test() {
    let request = Request {
      method: "GET".to_string(),
      path: "/data/42".to_string(),
      matching_rules: MatchingRules {
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
      },
      generators: Generators {
        categories: hashmap!{
          GeneratorCategory::PATH => hashmap!{
            DocPath::root() => Generator::ProviderStateGenerator("/data/${id}".to_string(), None)
          }
        }
      },
      .. Request::default()
    };

    let json = request.to_json(&PactSpecification::V3);

    let generators = json.get("generators").unwrap();
    expect!(generators.to_string()).to_not(be_equal_to("{}"));

    let rules = json.get("matchingRules").unwrap();
    expect!(rules.to_string()).to_not(be_equal_to("{}"));
  }
}
