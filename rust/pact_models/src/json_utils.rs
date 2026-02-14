//! Collection of utilities for working with JSON

use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::{self, json, Map, Value};

use crate::bodies::OptionalBody;
use crate::content_types::{ContentType, detect_content_type_from_string};
use crate::headers::parse_header;
use crate::path_exp::{DocPath, PathToken};

/// Trait to convert a JSON structure to a number
pub trait JsonToNum<T> {
  /// Converts the JSON field in the map to a Number
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: T) -> T;
}

impl JsonToNum<i32> for i32 {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: i32) -> i32 {
    match map.get(field) {
      Some(Value::Number(num)) => {
         match num.as_i64() {
          Some(num) => num as i32,
          None => default
        }
      },
      _ => default
    }
  }
}

impl JsonToNum<u16> for u16 {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: u16) -> u16 {
    match map.get(field) {
      Some(Value::Number(num)) => {
        match num.as_u64() {
          Some(num) => num as u16,
          None => default
        }
      },
      _ => default
    }
  }
}

/// Converts the JSON struct to a String, first checking if it is a JSON String
pub fn json_to_string(value: &Value) -> String {
  match value {
    Value::String(s) => s.clone(),
    Value::Null => String::default(),
    _ => value.to_string()
  }
}

/// Converts an optional JSON struct to a usize, returning `None` if it is not a numeric type.
pub fn json_to_num(value: Option<Value>) -> Option<usize> {
  if let Some(value) = value {
    match value {
      Value::Number(n) => if n.is_i64() && n.as_i64().unwrap() > 0 { Some(n.as_i64().unwrap() as usize) }
        else if n.is_f64() { Some(n.as_f64().unwrap() as usize) }
        else if n.is_u64() { Some(n.as_u64().unwrap() as usize) }
        else { None },
      Value::String(ref s) => usize::from_str(&s.clone()).ok(),
      _ => None
    }
  } else {
    None
  }
}

/// Hash function for JSON struct
pub fn hash_json<H: Hasher>(v: &Value, state: &mut H) {
  match v {
    Value::Bool(b) => b.hash(state),
    Value::Number(n) => {
      if let Some(num) = n.as_u64() {
        num.hash(state);
      }
      if let Some(num) = n.as_f64() {
        num.to_string().hash(state);
      }
      if let Some(num) = n.as_i64() {
        num.hash(state);
      }
    }
    Value::String(s) => s.hash(state),
    Value::Array(values) => for value in values {
      hash_json(value, state);
    }
    Value::Object(map) => for (k, v) in map {
      k.hash(state);
      hash_json(v, state);
    }
    _ => ()
  }
}

/// Look up a field and return it as a string value
pub fn get_field_as_string(field: &str, map: &Map<String, Value>) -> Option<String> {
  map.get(field).map(|f| json_to_string(f))
}

/// Returns the headers from a JSON struct as Map String -> Vec<String>
pub fn headers_from_json(request: &Value) -> Option<HashMap<String, Vec<String>>> {
  match request.get("headers") {
    Some(Value::Object(m)) => {
      Some(m.iter().map(|(key, val)| {
        match val {
          Value::String(s) => (key.clone(), parse_header(key.as_str(), s.as_str())),
          Value::Array(v) => (key.clone(), v.iter().map(|val| {
            match val {
              Value::String(s) => s.clone(),
              _ => val.to_string()
            }
          }).collect()),
          _ => (key.clone(), vec![val.to_string()])
        }
      }).collect())
    },
    _ => None
  }
}

/// Converts the headers map into a JSON struct
pub fn headers_to_json(headers: &HashMap<String, Vec<String>>) -> Value {
  json!(headers.iter()
    .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
    .fold(BTreeMap::new(), |mut map, kv| {
      map.insert(kv.0.clone(), Value::String(kv.1.join(", ")));
      map
    }))
}

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum JsonParsable {
  JsonStringValue(String),
  KeyValue(HashMap<String, Value>)
}

/// Returns the body from the JSON struct with the provided field name
pub fn body_from_json(request: &Value, fieldname: &str, headers: &Option<HashMap<String, Vec<String>>>) -> OptionalBody {
  let content_type = match headers {
    Some(h) => match h.iter().find(|kv| kv.0.to_lowercase() == "content-type") {
      Some(kv) => {
        match ContentType::parse(kv.1[0].as_str()) {
          Ok(v) => Some(v),
          Err(_) => None
        }
      },
      None => None
    },
    None => None
  };

  match request.get(fieldname) {
    Some(v) => match v {
      Value::String(s) => {
        if s.is_empty() {
          OptionalBody::Empty
        } else {
          let content_type = content_type.unwrap_or_else(|| {
            detect_content_type_from_string(s).unwrap_or_default()
          });
          if content_type.is_json() {
            match serde_json::from_str::<JsonParsable>(s) {
              Ok(_) => OptionalBody::Present(s.clone().into(), Some(content_type), None),
              Err(_) => OptionalBody::Present(format!("\"{}\"", s).into(), Some(content_type), None)
            }
          } else if content_type.is_text() {
            OptionalBody::Present(s.clone().into(), Some(content_type), None)
          } else {
            match BASE64.decode(s) {
              Ok(bytes) => OptionalBody::Present(bytes.into(), None, None),
              Err(_) => OptionalBody::Present(s.clone().into(), None, None)
            }
          }
        }
      },
      Value::Null => OptionalBody::Null,
      _ => OptionalBody::Present(v.to_string().into(), None, None)
    },
    None => OptionalBody::Missing
  }
}

/// Deep merges the other value into the given value
pub fn json_deep_merge(value: &Value, other: &Value) -> Value {
  match (value, other) {
    (Value::Array(items), Value::Array(other_items)) => {
      let mut values = items.clone();
      values.extend(other_items.iter().cloned());
      Value::Array(values)
    },
    (Value::Object(entries), Value::Object(other_entries)) => {
      let map = entries.iter()
        .chain(other_entries.iter())
        .fold(serde_json::Map::new(), |mut m, (k, v)| {
          let value = if let Some(value) = m.get(k) {
            json_deep_merge(value, v)
          } else {
            v.clone()
          };
          m.insert(k.clone(), value);
          m
      });
      Value::Object(map)
    },
    _ => other.clone()
  }
}

/// If the JSON value is empty
pub fn is_empty(value: &Value) -> bool {
  match value {
    Value::Null => true,
    Value::Bool(_) => false,
    Value::Number(_) => false,
    Value::String(s) => s.is_empty(),
    Value::Array(a) => a.is_empty(),
    Value::Object(o) => o.is_empty()
  }
}

/// Resolve the path expression against the JSON value, returning a list of JSON pointer values
/// that match.
pub fn resolve_path(value: &Value, expression: &DocPath) -> Vec<String> {
  let mut tree = Arena::new();
  let root = tree.new_node("".into());
  query_object_graph(expression.tokens(), &mut tree, root, value.clone());
  let expanded_paths = root.descendants(&tree).fold(Vec::<String>::new(), |mut acc, node_id| {
    let node = tree.index(node_id);
    if !node.get().is_empty() && node.first_child().is_none() {
      let path: Vec<String> = node_id.ancestors(&tree).map(|n| format!("{}", tree.index(n).get())).collect();
      if path.len() == expression.len() {
        acc.push(path.iter().rev().join("/"));
      }
    }
    acc
  });
  expanded_paths
}

fn query_object_graph(path_exp: &Vec<PathToken>, tree: &mut Arena<String>, root: NodeId, body: Value) {
  let mut body_cursor = body;
  let mut it = path_exp.iter();
  let mut node_cursor = root;
  loop {
    match it.next() {
      Some(token) => {
        match token {
          PathToken::Field(name) => {
            match body_cursor.as_object() {
              Some(map) => match map.get(name) {
                Some(val) => {
                  node_cursor = node_cursor.append_value(name.clone(), tree);
                  body_cursor = val.clone();
                },
                None => return
              },
              None => return
            }
          },
          PathToken::Index(index) => {
            match body_cursor.clone().as_array() {
              Some(list) => if list.len() > *index {
                node_cursor = node_cursor.append_value(format!("{}", index), tree);
                body_cursor = list[*index].clone();
              },
              None => return
            }
          }
          PathToken::Star => {
            match body_cursor.clone().as_object() {
              Some(map) => {
                let remaining = it.by_ref().cloned().collect();
                for (key, val) in map {
                  let node = node_cursor.append_value(key.clone(), tree);
                  body_cursor = val.clone();
                  query_object_graph(&remaining, tree, node, val.clone());
                }
              },
              None => return
            }
          },
          PathToken::StarIndex => {
            match body_cursor.clone().as_array() {
              Some(list) => {
                let remaining = it.by_ref().cloned().collect();
                for (index, val) in list.iter().enumerate() {
                  let node = node_cursor.append_value(format!("{}", index), tree);
                  body_cursor = val.clone();
                  query_object_graph(&remaining, tree, node,val.clone());
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

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use super::*;

  #[test]
  fn json_to_int_test() {
    expect!(<i32>::json_to_number(&serde_json::Map::new(), "any", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "any", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": "5" }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(5));
    expect!(<i32>::json_to_number(&json!({ "min": -5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(-5));
    expect!(<i32>::json_to_number(&json!({ "min": 5.0 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));

    expect!(<u16>::json_to_number(&serde_json::Map::new(), "any", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "any", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": "5" }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(5));
    expect!(<u16>::json_to_number(&json!({ "min": -5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5.0 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
  }

  #[test]
  fn json_to_string_test() {
    expect!(json_to_string(&Value::from_str("\"test string\"").unwrap())).to(be_equal_to("test string".to_string()));
    expect!(json_to_string(&Value::from_str("null").unwrap())).to(be_equal_to("".to_string()));
    expect!(json_to_string(&Value::from_str("100").unwrap())).to(be_equal_to("100".to_string()));
    expect!(json_to_string(&Value::from_str("100.10").unwrap())).to(be_equal_to("100.1".to_string()));
    expect!(json_to_string(&Value::from_str("{}").unwrap())).to(be_equal_to("{}".to_string()));
    expect!(json_to_string(&Value::from_str("[]").unwrap())).to(be_equal_to("[]".to_string()));
    expect!(json_to_string(&Value::from_str("true").unwrap())).to(be_equal_to("true".to_string()));
    expect!(json_to_string(&Value::from_str("false").unwrap())).to(be_equal_to("false".to_string()));
  }

  #[test]
  fn json_to_num_test() {
    expect!(json_to_num(Value::from_str("\"test string\"").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("null").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("{}").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("[]").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("true").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("false").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("100").ok())).to(be_some().value(100));
    expect!(json_to_num(Value::from_str("-100").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("100.10").ok())).to(be_some().value(100));
  }

  #[test]
  fn body_from_text_plain_type_returns_the_same_formatted_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "text/plain"},
          "body": "\"This is a string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/plain".into()), None)));
  }

  #[test]
  fn body_from_text_html_type_returns_the_same_formatted_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "text/html"},
          "body": "\"This is a string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/html".into()), None)));
  }

  #[test]
  fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_string_and_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "This is actually a JSON string"
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()), None)));
  }

  #[test]
  fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_valid_json_string_and_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "\"This is actually a JSON string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()), None)));
  }

  #[test]
  fn body_from_json_returns_the_body_if_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "{\"test\":true}"
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(), Some("application/json".into()), None)));
  }

  #[test]
  fn body_from_json_returns_missing_if_there_is_no_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "*.path": {}
          }
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Missing));
  }

  #[test]
  fn body_from_json_returns_null_if_the_body_is_null() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": null
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Null));
  }

  #[test]
  fn body_from_json_returns_json_string_if_the_body_is_json_but_not_a_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "test": true
          }
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(), None, None)));
  }

  #[test]
  fn body_from_json_returns_empty_if_the_body_is_an_empty_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": ""
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Empty));
  }

  #[test]
  fn body_from_json_returns_the_body_if_the_body_is_a_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": "<?xml version=\"1.0\"?> <body></body>"
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Present("<?xml version=\"1.0\"?> <body></body>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn deep_merge_with_primitives() {
    let value = Value::Bool(true);
    expect!(json_deep_merge(&value, &Value::Null)).to(be_equal_to(Value::Null));
    expect!(json_deep_merge(&value, &Value::String("test".to_string()))).to(be_equal_to(Value::String("test".to_string())));
  }

  #[test]
  fn deep_merge_with_empty_collections() {
    let value = Value::Array(vec![]);
    expect!(json_deep_merge(&value, &Value::Array(vec![]))).to(be_equal_to(Value::Array(vec![])));

    let value = Value::Object(Default::default());
    expect!(json_deep_merge(&value, &Value::Object(Default::default()))).to(be_equal_to(Value::Object(Default::default())));
  }

  #[test]
  fn deep_merge_with_simple_objects() {
    let value = json!({ "a": null });
    expect!(json_deep_merge(&value, &json!({ "b": true }))).to(be_equal_to(json!({ "a": null, "b": true })));
    expect!(json_deep_merge(&value, &json!({ "b": true, "a": false }))).to(be_equal_to(json!({ "a": false, "b": true })));
  }

  #[test]
  fn deep_merge_with_simple_arrays() {
    let value = json!([1, 2, 3]);
    expect!(json_deep_merge(&value, &json!([4, 5]))).to(be_equal_to(json!([1, 2, 3, 4, 5])));
  }


  #[test]
  fn deep_merge_with_collections_with_different_types() {
    let value = json!({
      "a": { "b": true },
      "b": [ true ]
    });
    expect!(json_deep_merge(&value, &json!({ "a": true }))).to(be_equal_to(json!({ "a": true, "b": [ true ] })));
    expect!(json_deep_merge(&value, &json!({ "b": true }))).to(be_equal_to(json!({ "a": { "b": true }, "b": true })));
  }

  #[test]
  fn deep_merge_with_collections_recursively_merges_the_collections() {
    let value = json!({
      "a": { "b": true },
      "b": [ true ]
    });
    let value2 = json!({
      "a": { "b": false },
      "b": [ false ]
    });
    expect!(json_deep_merge(&value, &value2)).to(be_equal_to(json!({
      "a": { "b": false },
      "b": [ true, false ]
    })));
    expect!(json_deep_merge(&value, &value)).to(be_equal_to(json!({
      "a": { "b": true },
      "b": [ true, true ]
    })));
  }

  #[test]
  fn headers_from_json_test() {
    let headers = json!({});
    let result = headers_from_json(&headers);
    expect!(result).to(be_none());

    let headers = json!({
      "headers": null
    });
    let result = headers_from_json(&headers);
    expect!(result).to(be_none());

    let headers = json!({
      "headers": {
        "A": "B",
        "B": "A, B, C",
        "C": ["B"],
        "Date": "Sun, 12 Mar 2023 01:21:35 GMT"
      }
    });
    let result = headers_from_json(&headers);
    expect!(result.unwrap()).to(be_equal_to(hashmap! {
      "A".to_string() => vec!["B".to_string()],
      "B".to_string() => vec!["A, B, C".to_string()],
      "C".to_string() => vec!["B".to_string()],
      "Date".to_string() => vec!["Sun, 12 Mar 2023 01:21:35 GMT".to_string()]
    }));
  }

  #[test]
  fn resolve_path_with_root() {
    expect!(resolve_path(&Value::Null, &DocPath::root())).to(be_equal_to::<Vec<String>>(vec![]));
    expect!(resolve_path(&Value::Bool(true), &DocPath::root())).to(be_equal_to::<Vec<String>>(vec![]));
    expect!(resolve_path(&json!([1, 2, 3]), &DocPath::root())).to(be_equal_to::<Vec<String>>(vec![]));
    expect!(resolve_path(&json!({"a": 1}), &DocPath::root())).to(be_equal_to::<Vec<String>>(vec![]));
  }

  #[test]
  fn resolve_path_with_field() {
    let path = DocPath::new_unwrap("$.a");
    let json = Value::Null;
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = Value::Bool(true);
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = json!({
      "a": 100,
      "b": 200
    });
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/a"]));

    let json = json!([
      {
        "a": 100,
        "b": 200
      }
    ]);
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));
  }

  #[test]
  fn resolve_path_with_index() {
    let path = DocPath::new_unwrap("$[0]");
    let json = Value::Null;
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = Value::Bool(true);
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = json!({
      "a": 100,
      "b": 200
    });
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = json!([
      {
        "a": 100,
        "b": 200
      }
    ]);
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/0"]));
    let path = DocPath::new_unwrap("$[0].b");
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/0/b"]));
  }

  #[test]
  fn resolve_path_with_star() {
    let path = DocPath::new_unwrap("$.*");
    let json = Value::Null;
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = Value::Bool(true);
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));

    let json = json!({
      "a": 100,
      "b": 200
    });
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/a", "/b"]));

    let json = json!([
      {
        "a": 100,
        "b": 200
      },
      {
        "a": 200,
        "b": 300
      }
    ]);
    expect!(resolve_path(&json, &path)).to(be_equal_to::<Vec<String>>(vec![]));
    let path = DocPath::new_unwrap("$[*]");
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/0", "/1"]));
    let path = DocPath::new_unwrap("$[*].b");
    expect!(resolve_path(&json, &path)).to(be_equal_to(vec!["/0/b", "/1/b"]));
  }
}
