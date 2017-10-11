//! Our `JsonPattern` type and supporting code.

use pact_matching::models::Matchers;
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap as Map;
use std::iter::FromIterator;

use super::Pattern;
use util::obj_key_for_path;

/// A pattern which can be used to either:
///
/// 1. generate a sample JSON value using `to_json`, or
/// 2. test whether a JSON value matches the pattern.
///
/// Many common Rust types may be converted into JSON patterns using `into()`,
/// or the provided helper functions:
///
/// ```
/// use pact_consumer::prelude::*;
///
/// let s: JsonPattern = "example".into();
/// let b: JsonPattern = true.into();
/// let v: JsonPattern = vec!["a", "b"].into();
/// let n = JsonPattern::null();
/// ```
///
/// For more complicated values, see the `json_pattern!` macro.
#[derive(Debug)]
pub enum JsonPattern {
    /// A regular JSON value, implemented by `serde_json::Value`. Contains
    /// no special matching rules.
    Json(serde_json::Value),
    /// An array of JSON patterns. May contain nested matching rules.
    Array(Vec<JsonPattern>),
    /// An object containing JSON patterns. May contain nested matching rules.
    Object(Map<String, JsonPattern>),
    /// A term which contains an arbitrary matchable. This is where rules like
    /// `Like` hook into our syntax.
    Pattern(Box<Pattern<Matches = serde_json::Value>>),
}

impl JsonPattern {
    /// Construct a JSON `null` value.
    pub fn null() -> JsonPattern {
        JsonPattern::Json(serde_json::Value::Null)
    }

    /// Construct a JSON pattern from a type implementing `Pattern`.
    pub fn pattern<P>(pattern: P) -> JsonPattern
    where
        P: Pattern<Matches = serde_json::Value> + 'static,
    {
        JsonPattern::Pattern(Box::new(pattern))
    }
}

impl Pattern for JsonPattern {
    type Matches = serde_json::Value;

    fn to_example(&self) -> serde_json::Value {
        match *self {
            JsonPattern::Json(ref json) => json.to_owned(),
            JsonPattern::Array(ref arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_example()).collect())
            }
            JsonPattern::Object(ref obj) => {
                let fields = obj.into_iter().map(|(k, v)| (k.to_owned(), v.to_example()));
                serde_json::Value::Object(serde_json::Map::from_iter(fields))
            }
            JsonPattern::Pattern(ref pattern) => pattern.to_example(),
        }
    }

    fn extract_matching_rules(&self, path: &str, rules_out: &mut Matchers) {
        match *self {
            JsonPattern::Json(_) => {}
            JsonPattern::Array(ref arr) => {
                for (i, val) in arr.into_iter().enumerate() {
                    let val_path = format!("{}[{}]", path, i);
                    val.extract_matching_rules(&val_path, rules_out);
                }
            }
            JsonPattern::Object(ref obj) => {
                for (key, val) in obj {
                    let val_path = format!("{}{}", path, obj_key_for_path(key));
                    val.extract_matching_rules(&val_path, rules_out);
                }
            }
            JsonPattern::Pattern(ref pattern) => {
                pattern.extract_matching_rules(path, rules_out);
            }
        }
    }
}

#[test]
fn json_pattern_is_pattern() {
    use env_logger;
    use std::collections::HashMap;

    use super::special_rules::Like;

    let _ = env_logger::init();

    // This is our pattern, combinging both example data and matching rules.
    let pattern = json_pattern!({
        "json": 1,
        "simple": Like::new(json_pattern!("a")),
        "array": [Like::new(json_pattern!("b"))],
    });

    // Here's our example JSON, without the matching rules.
    let expected_json = json!({
        "json": 1,
        "simple": "a",
        "array": ["b"],
    });
    assert_eq!(pattern.to_example(), expected_json);

    // Here are our matching rules, for passing to the low-level match engine.
    let expected_rules = json!({
        "$.simple": { "match": "type" },
        "$.array[0]": { "match": "type" },
    });
    let mut rules = HashMap::new();
    pattern.extract_matching_rules("$", &mut rules);
    assert_eq!(json!(rules), expected_rules);
}

/// This macro will define a `From` implementation for a list of types by first
/// converting them to a `serde_json::Value`, and then wrapping them in
/// `Value::Json`.
macro_rules! impl_from_for_json {
    ( $($t:ty),* ) => {
        $(
            impl From<$t> for JsonPattern {
                fn from(n: $t) -> Self {
                    JsonPattern::Json(n.into())
                }
            }
        )*
    }
}

impl_from_for_json!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64,
    bool, String, serde_json::Map<String, serde_json::Value>
);

impl<'a> From<&'a str> for JsonPattern {
    fn from(s: &'a str) -> Self {
        JsonPattern::Json(s.into())
    }
}

impl<'a> From<Cow<'a, str>> for JsonPattern {
    fn from(s: Cow<'a, str>) -> Self {
        JsonPattern::Json(s.into())
    }
}

impl<T: Into<JsonPattern>> From<Vec<T>> for JsonPattern {
    fn from(arr: Vec<T>) -> Self {
        JsonPattern::Array(arr.into_iter().map(|v| v.into()).collect())
    }
}

impl<'a, T: Clone + Into<JsonPattern>> From<&'a [T]> for JsonPattern {
    fn from(arr: &'a [T]) -> Self {
        JsonPattern::Array(arr.into_iter().map(|v| v.clone().into()).collect())
    }
}

impl<T: Into<JsonPattern>> FromIterator<T> for JsonPattern {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        JsonPattern::Array(iter.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Map<String, JsonPattern>> for JsonPattern {
    fn from(m: Map<String, JsonPattern>) -> Self {
        JsonPattern::Object(m)
    }
}

impl From<serde_json::Value> for JsonPattern {
    fn from(j: serde_json::Value) -> Self {
        JsonPattern::Json(j)
    }
}
