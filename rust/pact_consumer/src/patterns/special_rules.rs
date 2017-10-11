//! Special matching rules, including `Like`, `Term`, etc.

use pact_matching::models::Matchers;
use regex::Regex;
use serde_json;
#[cfg(test)]
use std::collections::HashMap;
use std::iter::repeat;
use std::marker::PhantomData;

use super::Pattern;
use super::json_pattern::JsonPattern;
use super::string_pattern::StringPattern;

macro_rules! impl_from_for_pattern {
    ($from:ty, $pattern:ident) => {
        impl From<$from> for $pattern {
            fn from(pattern: $from) -> Self {
                $pattern::pattern(pattern)
            }
        }
    }
}

/// Match values based on their data types.
#[derive(Debug)]
pub struct Like<Nested: Pattern> {
    example: Nested,
}

impl<Nested: Pattern> Like<Nested> {
    /// Match all values which have the same type as `example`.
    pub fn new<E: Into<Nested>>(example: E) -> Self {
        Like { example: example.into() }
    }
}

impl<Nested: Pattern> Pattern for Like<Nested> {
    type Matches = Nested::Matches;

    fn to_example(&self) -> Self::Matches {
        self.example.to_example()
    }

    fn extract_matching_rules(&self, path: &str, rules_out: &mut Matchers) {
        rules_out.insert(path.to_owned(), hashmap!(s!("match") => s!("type")));
        self.example.extract_matching_rules(path, rules_out);
    }
}

impl_from_for_pattern!(Like<JsonPattern>, JsonPattern);
impl_from_for_pattern!(Like<StringPattern>, StringPattern);

#[test]
fn like_is_pattern() {
    let matchable = Like::<JsonPattern>::new(json_pattern!("hello"));
    assert_eq!(matchable.to_example(), json!("hello"));
    let mut rules = HashMap::new();
    matchable.extract_matching_rules("$", &mut rules);
    assert_eq!(json!(rules), json!({"$": {"match": "type"}}));
}

#[test]
fn like_into() {
    // Make sure we can convert `Like` into different pattern types.
    let _: JsonPattern = Like::new(json_pattern!("hello")).into();
    // We don't particularly care about having a nice syntax for
    // `StringPattern`, because it's rarely useful in practice.
    let _: StringPattern = Like::new("hello".to_owned()).into();
}

/// Generates the specified value, matches any value of the same data type. This
/// is intended for use inside `json_pattern!`, and it interprets its arguments
/// as a `json_pattern!`.
///
/// ```
/// # #[macro_use] extern crate pact_consumer;
/// # fn main() {
/// json_pattern!({
///   "id": like!(10),
///   "metadata": like!({}),
/// });
/// # }
/// ```
///
/// If you're building `StringPattern` values, you'll need to call
/// `Like::new` manually instead.
#[macro_export]
macro_rules! like {
    ($($json_pattern:tt)+) => {
        $crate::patterns::Like::new(json_pattern!($($json_pattern)+))
    }
}

/// Match an array with the specified "shape".
#[derive(Debug)]
pub struct EachLike {
    example_element: JsonPattern,
    min_len: usize,
}

impl EachLike {
    /// Match arrays containing elements like `example_element`.
    pub fn new(example_element: JsonPattern) -> EachLike {
        EachLike {
            example_element: example_element,
            min_len: 1,
        }
    }

    /// Use this after `new` to set a minimum length for the matching array.
    pub fn with_min_len(mut self, min_len: usize) -> EachLike {
        self.min_len = min_len;
        self
    }
}

impl_from_for_pattern!(EachLike, JsonPattern);

impl Pattern for EachLike {
    type Matches = serde_json::Value;

    fn to_example(&self) -> serde_json::Value {
        let element = self.example_element.to_example();
        serde_json::Value::Array(repeat(element).take(self.min_len).collect())
    }

    fn extract_matching_rules(&self, path: &str, rules_out: &mut Matchers) {
        rules_out.insert(
            path.to_owned(),
            hashmap!(
                s!("match") => s!("type"),
                s!("min") => format!("{}", self.min_len),
            ),
        );
        rules_out.insert(
            format!("{}[*].*", path),
            hashmap!(
                s!("match") => s!("type"),
            ),
        );
        let new_path = format!("{}[*]", path);
        self.example_element.extract_matching_rules(
            &new_path,
            rules_out,
        );
    }
}

#[test]
fn each_like_is_pattern() {
    let elem = Like::new(json_pattern!("hello"));
    let matchable = EachLike::new(json_pattern!(elem)).with_min_len(2);
    assert_eq!(matchable.to_example(), json!(["hello", "hello"]));

    let mut rules = HashMap::new();
    matchable.extract_matching_rules("$", &mut rules);
    let expected_rules = json!({
        // Ruby omits the `type` here, but the Rust `pact_matching` library
        // claims to want `"type"` when `"min"` is used.
        "$": {"match": "type", "min": "2"},
        // TODO: Ruby always generates this; I'm not sure what it's intended to
        // do. It looks like it makes child objects in the array match their
        // fields by type automatically?
        "$[*].*": {"match": "type"},
        // This is inserted by our nested `Like` rule.
        "$[*]": {"match": "type"},
    });
    assert_eq!(json!(rules), expected_rules);
}

// A hidden macro which does the hard work of expanding `each_like!`. We don't
// include this in the docs because it reveals a bunch of implementation
// details.
//
// This is a classic Rust "tt muncher" macro that uses special rules starting
// with `@` to build a recursive token examiner.
#[macro_export]
#[doc(hidden)]
macro_rules! each_like_helper {
    // Parsing base case #1: We made it all the way to the end of our tokens
    // without seeing a top-level comma.
    (@parse [$($found:tt)*] ) => {
        each_like_helper!(@expand [$($found)*] [])
    };

    // Parsing base case #2: We saw a top-level comma, so we're done parsing
    // the JSON pattern.
    (@parse [$($found:tt)*] , $($rest:tt)* ) => {
        each_like_helper!(@expand [$($found)*] [$($rest)*])
    };

    // Parsing recursive case (must come last): We have some other token, so
    // add it to what we've found and continue.
    (@parse [$($found:tt)*] $next:tt $($rest:tt)* ) => {
        each_like_helper!(@parse [$($found)* $next] $($rest)*)
    };

    // We're done parsing, and we didn't find `min`.
    (@expand [$($pattern:tt)*] []) => {
        $crate::patterns::EachLike::new(json_pattern!($($pattern)*))
    };

    // We're done parsing, and we did find `min`.
    (@expand [$($pattern:tt)*] [min = $min_len:expr]) => {
        $crate::patterns::EachLike::new(json_pattern!($($pattern)*))
            .with_min_len($min_len)
    };

    // Entry point. Must come last, because it matches anything.
    ($($tokens:tt)+) => (each_like_helper!(@parse [] $($tokens)+));
}

/// Generates the specified value, matches any value of the same data type. This
/// is intended for use inside `json_pattern!`, and it interprets its arguments
/// as a `json_pattern!`.
///
/// ```
/// # #[macro_use] extern crate pact_consumer;
/// # fn main() {
/// json_pattern!({
///   // Expect an array of strings.
///   "tags": each_like!("tag"),
///
///   // Expect an array of objects, each of which has a name key containing
///   // a string (but match the actual names by type). Require a minimum of
///   // two elements.
///   "people": each_like!({
///     "name": "J. Smith",
///   }, min=2),
/// });
/// # }
/// ```
#[macro_export]
macro_rules! each_like {
    ($($token:tt)+) => { each_like_helper!($($token)+) };
}

#[test]
fn each_like_macro_parsing() {
    #[derive(Serialize)]
    struct Point {
        x: i32,
        y: i32
    }

    // This is something users might plausibly want to do.
    let simple = each_like!(json!(Point { x: 1, y: 2 }));
    assert_eq!(simple.example_element.to_example(), json!({ "x": 1, "y": 2 }));
    assert_eq!(simple.min_len, 1);

    // And now with `min`, which requires quite a bit of complicated macro
    // code to parse.
    let with_min = each_like!(json!(Point { x: 1, y: 2 }), min = 2 + 1);
    assert_eq!(with_min.example_element.to_example(), json!({ "x": 1, "y": 2 }));
    assert_eq!(with_min.min_len, 3);
}

/// Match and generate strings that match a regular expression.
#[derive(Debug)]
pub struct Term<Nested: Pattern> {
    /// The example string we generate when asked.
    example: String,
    /// The regex we use to match.
    regex: Regex,
    /// Since we always store `example` as a string, we need to mention our
    /// `Nested` type somewhere. We can do that using the zero-length
    /// `PhantomData` type.
    phantom: PhantomData<Nested>,
}

impl<Nested: Pattern> Term<Nested> {
    /// Construct a new `Term`, given a regex and the example string to
    /// generate.
    pub fn new<S: Into<String>>(regex: Regex, example: S) -> Self {
        Term {
            example: example.into(),
            regex: regex,
            phantom: PhantomData,
        }
    }
}

impl<Nested> Pattern for Term<Nested>
where
    Nested: Pattern,
    Nested::Matches: From<String>,
{
    type Matches = Nested::Matches;

    fn to_example(&self) -> Self::Matches {
        From::from(self.example.clone())
    }

    fn extract_matching_rules(&self, path: &str, rules_out: &mut Matchers) {
        rules_out.insert(
            path.to_owned(),
            hashmap!(
                s!("match") => s!("regex"),
                s!("regex") => s!(self.regex.as_str()),
            ),
        );
    }
}

impl_from_for_pattern!(Term<JsonPattern>, JsonPattern);
impl_from_for_pattern!(Term<StringPattern>, StringPattern);

#[test]
fn term_is_pattern() {
    let matchable = Term::<JsonPattern>::new(Regex::new("[Hh]ello").unwrap(), "hello");
    assert_eq!(matchable.to_example(), json!("hello"));

    let mut rules = HashMap::new();
    matchable.extract_matching_rules("$", &mut rules);
    let expected_rules = json!({
        "$": { "match": "regex", "regex": "[Hh]ello" },
    });
    assert_eq!(json!(rules), expected_rules);
}

#[test]
fn term_into() {
    // Make sure we can convert `Term` into different pattern types.
    let _: JsonPattern = Term::new(Regex::new("[Hh]ello").unwrap(), "hello").into();
    let _: StringPattern = Term::new(Regex::new("[Hh]ello").unwrap(), "hello").into();
}

/// Internal helper function called by `term!` to build a regex. Panics if the
/// regex is invalid. (We use this partly because it's hard to refer to the
/// `regex` crate from inside a public macro unless our caller imports it.)
#[doc(hidden)]
pub fn build_regex<S: AsRef<str>>(regex_str: S) -> Regex {
    let regex_str = regex_str.as_ref();
    match Regex::new(regex_str) {
        Ok(regex) => regex,
        Err(msg) => panic!("could not parse regex {:?}: {}", regex_str, msg),
    }
}

/// A pattern which macthes the regular expression `$regex` (specified as a
/// string) literal, and which generates `$example`.
///
/// ```
/// # #[macro_use] extern crate pact_consumer;
/// # fn main() {
/// json_pattern!({
///   // Match a string consisting of numbers and lower case letters, and
///   // generate `"10a"`.$crate::patterns::
///   "id_string": term!("^[0-9a-z]$", "10a")
/// });
/// # }
/// ```
#[macro_export]
macro_rules! term {
    ($regex:expr, $example:expr) => {
        {
            $crate::patterns::Term::new($crate::patterns::build_regex($regex), $example)
        }
    }
}
