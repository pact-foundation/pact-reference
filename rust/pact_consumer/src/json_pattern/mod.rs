//! JSON "patterns", which can be used to either generate JSON documents or
//! match them.

use pact_matching::models::Matchers;
use regex::{Captures, Regex};
use serde_json;
use std::borrow::Cow;
pub use std::collections::HashMap as Map;
use std::fmt::Debug;
use std::iter::{FromIterator, repeat};

#[macro_use]
mod macros;

/// Format a JSON object key for use in a JSON path expression. If we were
/// more concerned about performance, we might try to come up with a scheme
/// to minimize string allocation here.
fn obj_key_for_path(key: &str) -> String {
    lazy_static! {
        // Only use "." syntax for things which are obvious identifiers.
        static ref IDENT: Regex = Regex::new(r#"^[_A-Za-z][_A-Za-z0-9]*$"#)
            .expect("could not parse IDENT regex");
        // Escape these characters when using string syntax.
        static ref ESCAPE: Regex = Regex::new(r#"\\|'"#)
            .expect("could not parse ESCAPE regex");
    }

    if IDENT.is_match(key) {
        format!(".{}", key)
    } else {
        format!("['{}']", ESCAPE.replace_all(key, |caps: &Captures| {
            format!(r#"\{}"#, &caps[0])
        }))
    }
}

#[test]
fn obj_key_for_path_quotes_keys_when_necessary() {
    assert_eq!(obj_key_for_path("foo"), ".foo");
    assert_eq!(obj_key_for_path("_foo"), "._foo");
    assert_eq!(obj_key_for_path("["), "['[']");

    // I don't actually know how the JSON Path specification wants us to handle
    // these cases, but we need to _something_ to avoid panics or passing
    // `Result` around everywhere, so let's go with JavaScript string escape
    // syntax.
    assert_eq!(obj_key_for_path(r#"''"#), r#"['\'\'']"#);
    assert_eq!(obj_key_for_path(r#"a'"#), r#"['a\'']"#);
    assert_eq!(obj_key_for_path(r#"\"#), r#"['\\']"#);
}

/// Abstract interface to types which can:
///
/// 1. Generate example data.
/// 2. Match data returned by tests in various flexible ways, for example,
///    accepting all strings which match a regular expression.
///
/// For an overview of how the matching rules work, and what kinds of special
/// matching rules exist, see the [`pact_matching` documentation][spec].
///
/// The current version of this API will only work for `JsonPattern` and
/// `serde_json::Value`. Extending this scheme to work for XML would require
/// parameterizing the input and output types, and possibly other changes.
///
/// [spec]: https://docs.rs/pact_matching/0.2.2/pact_matching/
pub trait Matchable: Debug {
    /// Convert this `Matchable` into an example data value, stripping out
    /// any special match rules.
    fn to_example(&self) -> serde_json::Value;

    /// Extract the matching rules from this `Matchable`, and insert them into
    /// `rules_out`, using `path` as the base path.
    ///
    /// This API corresponds to the [`Extract` code in Ruby][ruby].
    ///
    /// (The `path` parameter is represented as a `&str` here, which forces each
    /// recursive call to allocate strings. We could optimize this by using a
    /// custom `path` representation which worked like a stack-based linked list
    /// stored in reverse order, but that would add significant complexity.)
    ///
    /// [ruby]:
    /// https://github.com/pact-foundation/pact-support/blob/master/lib/pact/matching_rules/extract.rb
    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    );
}

/// Match values based on their data types.
#[derive(Debug)]
struct SomethingLike {
    example: JsonPattern,
}

impl SomethingLike {
    /// Match all values which have the same type as `example`.
    pub fn new(example: JsonPattern) -> SomethingLike {
        SomethingLike { example: example }
    }
}

impl Matchable for SomethingLike {
    fn to_example(&self) -> serde_json::Value {
        self.example.to_example()
    }

    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    ) {
        rules_out.insert(path.to_owned(), hashmap!(s!("match") => s!("type")));
        self.example.extract_matching_rules(path, rules_out);
    }
}

#[test]
fn something_like_is_matchable() {
    let matchable = SomethingLike::new(json_pattern!("hello"));
    assert_eq!(matchable.to_example(), json!("hello"));
    let mut rules = Map::new();
    matchable.extract_matching_rules("$", &mut rules);
    assert_eq!(json!(rules), json!({"$": {"match": "type"}}));
}

/// Match an array with the specified "shape".
#[derive(Debug)]
pub struct ArrayLike {
    example_element: JsonPattern,
    min_length: usize,
}

impl ArrayLike {
    /// Match arrays containing elements like `example_element`.
    pub fn new(example_element: JsonPattern) -> ArrayLike {
        ArrayLike {
            example_element: example_element,
            min_length: 1,
        }
    }

    /// Use this after `new` to set a minimum length for the matching array.
    pub fn with_min_length(mut self, min_length: usize) -> ArrayLike {
        self.min_length = min_length;
        self
    }
}

impl Matchable for ArrayLike {
    fn to_example(&self) -> serde_json::Value {
        let element = self.example_element.to_example();
        serde_json::Value::Array(repeat(element).take(self.min_length).collect())
    }

    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    ) {
        rules_out.insert(path.to_owned(), hashmap!(
            s!("match") => s!("type"),
            s!("min") => format!("{}", self.min_length),
        ));
        rules_out.insert(format!("{}[*].*", path), hashmap!(
            s!("match") => s!("type"),
        ));
        let new_path = format!("{}[*]", path);
        self.example_element.extract_matching_rules(&new_path, rules_out);
    }
}

#[test]
fn array_like_is_matchable() {
    let elem = SomethingLike::new(json_pattern!("hello"));
    let matchable = ArrayLike::new(json_pattern!(elem))
        .with_min_length(2);
    assert_eq!(matchable.to_example(), json!(["hello", "hello"]));

    let mut rules = Map::new();
    matchable.extract_matching_rules("$", &mut rules);
    let expected_rules = json!({
        // Ruby omits the `type` here, but the Rust `pact_matching` library
        // claims to want `"type"` when `"min"` is used.
        "$": {"match": "type", "min": "2"},
        // TODO: Ruby always generates this; I'm not sure what it's intended to
        // do. It looks like it makes child objects in the array match their
        // fields by type automatically?
        "$[*].*": {"match": "type"},
        // This is inserted by our nested `SomethingLike` rule.
        "$[*]": {"match": "type"},
    });
    assert_eq!(json!(rules), expected_rules);
}

/// Match strings that match a regular expression.
#[derive(Debug)]
pub struct Term {
    example: String,
    regex: Regex,
}

impl Term {
    /// Construct a new `Term`, given a regex and the example string to
    /// generate.
    pub fn new<S: Into<String>>(regex: Regex, example: S) -> Term {
        Term {
            example: example.into(),
            regex: regex,
        }
    }
}

impl Matchable for Term {
    fn to_example(&self) -> serde_json::Value {
        json!(&self.example)
    }

    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    ) {
        rules_out.insert(path.to_owned(), hashmap!(
            s!("match") => s!("regex"),
            s!("regex") => s!(self.regex.as_str()),
        ));
    }
}

#[test]
fn term_is_matchable() {
    let matchable = Term::new(Regex::new("[Hh]ello").unwrap(), "hello");
    assert_eq!(matchable.to_example(), json!("hello"));

    let mut rules = Map::new();
    matchable.extract_matching_rules("$", &mut rules);
    let expected_rules = json!({
        "$": { "match": "regex", "regex": "[Hh]ello" },
    });
    assert_eq!(json!(rules), expected_rules);
}

// These were also provided by the Ruby library, but I'm not sure we need them:
//
// - QueryString
// - QueryHash

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
    /// `SomethingLike` hook into our syntax.
    Matchable(Box<Matchable>),
}

impl JsonPattern {
    /// Construct a JSON `null` value.
    pub fn null() -> JsonPattern {
        JsonPattern::Json(serde_json::Value::Null)
    }

    /// Construct a JSON pattern from any type implementing `Matchable`.
    pub fn matchable<M: Matchable + 'static>(matchable: M) -> JsonPattern {
        JsonPattern::Matchable(Box::new(matchable))
    }
}

impl Matchable for JsonPattern {
    fn to_example(&self) -> serde_json::Value {
        match *self {
            JsonPattern::Json(ref json) => {
                json.to_owned()
            }
            JsonPattern::Array(ref arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_example()).collect())
            }
            JsonPattern::Object(ref obj) => {
                let fields = obj.into_iter()
                    .map(|(k, v)| (k.to_owned(), v.to_example()));
                serde_json::Value::Object(serde_json::Map::from_iter(fields))
            }
            JsonPattern::Matchable(ref matchable) => {
                matchable.to_example()
            }
        }
    }

    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    ) {
        match *self {
            JsonPattern::Json(_) => {},
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
            JsonPattern::Matchable(ref matchable) => {
                matchable.extract_matching_rules(path, rules_out);
            }
        }
    }
}

#[test]
fn json_pattern_is_matchable() {
    use env_logger;
    env_logger::init().expect("could not initialize logger");

    // This is our pattern, combinging both example data and matching rules.
    let matchable = json_pattern!({
        "json": 1,
        "simple": SomethingLike::new(json_pattern!("a")),
        "array": [SomethingLike::new(json_pattern!("b"))],
    });

    // Here's our example JSON, without the matching rules.
    let expected_json = json!({
        "json": 1,
        "simple": "a",
        "array": ["b"],
    });
    assert_eq!(matchable.to_example(), expected_json);

    // Here are our matching rules, for passing to the low-level match engine.
    let expected_rules = json!({
        "$.simple": { "match": "type" },
        "$.array[0]": { "match": "type" },
    });
    let mut rules = Map::new();
    matchable.extract_matching_rules("$", &mut rules);
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

macro_rules! impl_from_for_matchable {
    ( $($t:ty),* ) => {
        $(
            impl From<$t> for JsonPattern {
                fn from(matchable: $t) -> Self {
                    JsonPattern::matchable(matchable)
                }
            }
        )*
    }
}

impl_from_for_matchable!(SomethingLike, ArrayLike, Term);
