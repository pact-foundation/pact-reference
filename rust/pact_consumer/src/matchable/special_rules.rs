//! Special matching rules, including `SomethingLike`, `Term`, etc.

use pact_matching::models::Matchers;
use regex::Regex;
use serde_json;
#[cfg(test)]
use std::collections::HashMap;
use std::iter::repeat;

use super::Matchable;
use super::json_pattern::JsonPattern;

macro_rules! impl_from_matchable_for_json_pattern {
    ($t:ty) => {
        impl From<$t> for JsonPattern {
            fn from(matchable: $t) -> Self {
                JsonPattern::matchable(matchable)
            }
        }
    }
}

/// Match values based on their data types.
#[derive(Debug)]
pub struct SomethingLike {
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

impl_from_matchable_for_json_pattern!(SomethingLike);

#[test]
fn something_like_is_matchable() {
    let matchable = SomethingLike::new(json_pattern!("hello"));
    assert_eq!(matchable.to_example(), json!("hello"));
    let mut rules = HashMap::new();
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

impl_from_matchable_for_json_pattern!(ArrayLike);

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

impl_from_matchable_for_json_pattern!(Term);

#[test]
fn term_is_matchable() {
    let matchable = Term::new(Regex::new("[Hh]ello").unwrap(), "hello");
    assert_eq!(matchable.to_example(), json!("hello"));

    let mut rules = HashMap::new();
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
