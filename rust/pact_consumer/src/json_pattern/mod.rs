//! JSON "patterns", which can be used to either generate JSON documents or
//! match them.

use pact_matching::Matcher;
use serde_json;
use std::borrow::Cow;
pub use std::collections::HashMap as Map;
use std::iter::FromIterator;

#[macro_use]
mod macros;

/// A pattern which can be used to either:
///
/// 1. generate a sample JSON value using `to_json`, or
/// 2. test whether a JSON value matches the pattern.
///
/// Many common Rust types may be converted into JSON patterns using `into()`,
/// or the provided helper functions:
///
/// ```
/// use pact_consumer::JsonPattern;
///
/// let s: JsonPattern = "example".into();
/// let b: JsonPattern = true.into();
/// let v: JsonPattern = vec!["a", "b"].into();
/// let n = JsonPattern::null();
/// ```
///
/// For more complicated values, see the `json_pattern!` macro.
pub enum JsonPattern {
    /// A regular JSON value, implemented by `serde_json::Value`. Contains
    /// no special matching rules.
    Json(serde_json::Value),
    /// An array of JSON patterns. May contain nested matching rules.
    Array(Vec<JsonPattern>),
    /// An object containing JSON patterns. May contain nested matching rules.
    Object(Map<String, JsonPattern>),
    /// A term which contains both a custom matching rule, and a JSON literal
    /// to use when generating actual JSON values.
    Term(Matcher, Box<JsonPattern>),
}

impl JsonPattern {
    /// Construct a JSON `null` value.
    pub fn null() -> JsonPattern {
        JsonPattern::Json(serde_json::Value::Null)
    }

    /// Convert this value to plain JSON, stripping out any `Matcher` values.
    pub fn to_json(&self) -> serde_json::Value {
        match *self {
            JsonPattern::Json(ref json) => {
                json.to_owned()
            }
            JsonPattern::Array(ref a) => {
                serde_json::Value::Array(a.iter().map(|v| v.to_json()).collect())
            }
            JsonPattern::Object(ref obj) => {
                let fields = obj.into_iter()
                    .map(|(k, v)| (k.to_owned(), v.to_json()));
                serde_json::Value::Object(serde_json::Map::from_iter(fields))
            }
            JsonPattern::Term(ref _matcher, ref v) => {
                v.as_ref().to_json()
            }
        }
    }
}

/// This macro will define a `From` implementation for a list of types by first
/// converting them to a `serde_json::Value`, and then wrapping them in
/// `Value::Json`.
macro_rules! impl_from_for_json_pattern {
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

impl_from_for_json_pattern!(
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
