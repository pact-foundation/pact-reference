//! Special matching rules, including `Like`, `Term`, etc.

use std::iter::repeat;
use std::marker::PhantomData;
use itertools::{Either, Itertools};

use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
use pact_models::path_exp::DocPath;
use regex::Regex;
use serde_json::Value;

use super::json_pattern::JsonPattern;
use super::Pattern;
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

    fn to_example_bytes(&self) -> Vec<u8> {
        self.example.to_example_bytes()
    }

    fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
        rules_out.add_rule(path.clone(), MatchingRule::Type, RuleLogic::And);
        self.example.extract_matching_rules(path, rules_out);
    }
}

impl_from_for_pattern!(Like<JsonPattern>, JsonPattern);
impl_from_for_pattern!(Like<StringPattern>, StringPattern);

#[test]
fn like_is_pattern() {
    use maplit::*;
    use pact_matching::s;
    use serde_json::*;

    let matchable = Like::<JsonPattern>::new(json_pattern!("hello"));
    assert_eq!(matchable.to_example(), json!("hello"));
    let mut rules = MatchingRuleCategory::empty("body");
    matchable.extract_matching_rules(DocPath::root(), &mut rules);
    assert_eq!(rules.to_v2_json(), hashmap!(s!("$.body") => json!({"match": "type"})));
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
/// use pact_consumer::*;
///
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
            example_element,
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

    fn to_example_bytes(&self) -> Vec<u8> {
        let value = self.to_example();
        let s = value.as_str().unwrap_or_default();
        s.as_bytes().to_vec()
    }

    fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
        rules_out.add_rule(
            path.clone(),
            MatchingRule::MinType(self.min_len),
            RuleLogic::And
        );

        let mut fields_path = path.clone();
        fields_path.push_star_index().push_star();
        rules_out.add_rule(
            fields_path,
            MatchingRule::Type,
            RuleLogic::And
        );

        let mut example_path = path.clone();
        example_path.push_star_index();
        self.example_element.extract_matching_rules(
            example_path,
            rules_out,
        );
    }
}

#[test]
fn each_like_is_pattern() {
    use maplit::*;
    use pact_matching::s;
    use serde_json::*;

    let elem = Like::new(json_pattern!("hello"));
    let matchable = EachLike::new(json_pattern!(elem)).with_min_len(2);
    assert_eq!(matchable.to_example(), json!(["hello", "hello"]));

    let mut rules = MatchingRuleCategory::empty("body");
    matchable.extract_matching_rules(DocPath::root(), &mut rules);
    let expected_rules = hashmap!(
        // Ruby omits the `type` here, but the Rust `pact_matching` library
        // claims to want `"type"` when `"min"` is used.
        s!("$.body") => json!({"match": "type", "min": 2}),
        // TODO: Ruby always generates this; I'm not sure what it's intended to
        // do. It looks like it makes child objects in the array match their
        // fields by type automatically?
        s!("$.body[*].*") => json!({"match": "type"}),
        // This is inserted by our nested `Like` rule.
        s!("$.body[*]") => json!({"match": "type"}),
    );
    assert_eq!(rules.to_v2_json(), expected_rules);
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
/// use pact_consumer::*;
///
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
    use serde_json::*;

    #[derive(serde::Serialize)]
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
            regex,
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

    fn to_example_bytes(&self) -> Vec<u8> {
        self.example.clone().into_bytes()
    }

    fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
        rules_out.add_rule(path, MatchingRule::Regex(self.regex.to_string()),
            RuleLogic::And);
    }
}

impl_from_for_pattern!(Term<JsonPattern>, JsonPattern);
impl_from_for_pattern!(Term<StringPattern>, StringPattern);

#[test]
fn term_is_pattern() {
    use maplit::*;
    use serde_json::*;

    let matchable = Term::<JsonPattern>::new(Regex::new("[Hh]ello").unwrap(), "hello");
    assert_eq!(matchable.to_example(), json!("hello"));

    let mut rules = MatchingRuleCategory::empty("body");
    matchable.extract_matching_rules(DocPath::root(), &mut rules);
    let expected_rules = hashmap!(
        "$.body".to_string() => json!({ "match": "regex", "regex": "[Hh]ello" })
    );
    assert_eq!(rules.to_v2_json(), expected_rules);
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

/// A pattern which matches the regular expression `$regex` (specified as a
/// string) literal, and which generates `$example`. This is an alias for `matching_regex!`
///
/// ```
/// use pact_consumer::*;
///
/// # fn main() {
/// json_pattern!({
///   // Match a string consisting of numbers and lower case letters, and
///   // generate `"10a"`.
///   "id_string": term!("^[0-9a-z]+$", "10a")
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

/// A pattern which matches the regular expression `$regex` (specified as a
/// string) literal, and which generates `$example`.
///
/// ```
/// use pact_consumer::*;
///
/// # fn main() {
/// json_pattern!({
///   // Match a string consisting of numbers and lower case letters, and
///   // generate `"10a"`
///   "id_string": matching_regex!("^[0-9a-z]+$", "10a")
/// });
/// # }
/// ```
#[macro_export]
macro_rules! matching_regex {
    ($regex:expr, $example:expr) => {
        {
            $crate::patterns::Term::new($crate::patterns::build_regex($regex), $example)
        }
    }
}

/// Match keys and values in an Object based on associated matching rules
#[derive(Debug)]
pub struct ObjectMatching {
    example: JsonPattern,
    rules: Vec<MatchingRule>
}

impl ObjectMatching {
  /// Create a new ObjectMatching pattern with the provided pattern and list of rules
  pub fn new(example: JsonPattern, rules: Vec<MatchingRule>) -> Self {
    Self {
      example,
      rules
    }
  }
}

impl Pattern for ObjectMatching {
  type Matches = Value;

  fn to_example(&self) -> Self::Matches {
      self.example.to_example()
  }

  fn to_example_bytes(&self) -> Vec<u8> {
      self.example.to_example_bytes()
  }

  fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
    for rule in &self.rules {
        rules_out.add_rule(path.clone(), rule.clone(), RuleLogic::And);
    }

    let child_path = path.join("*");
    let mut child_rules = MatchingRuleCategory::empty("body");
    self.example.extract_matching_rules(DocPath::root(), &mut child_rules);
    for (path, rules) in child_rules.rules {
      let path_tokens = path.tokens().iter().dropping(2);
      let mut rule_path = child_path.clone();
      for segment in path_tokens {
        rule_path.push(segment.clone());
      }
      for rule in &rules.rules {
        rules_out.add_rule(rule_path.clone(), rule.clone(), rules.rule_logic);
      }
    }
  }
}

impl_from_for_pattern!(ObjectMatching, JsonPattern);

#[test]
fn object_matching_is_pattern() {
  use serde_json::*;
  use expectest::prelude::*;
  use pact_models::matchingrules_list;

  let matchable = ObjectMatching::new(
    json_pattern!({
      "key1": "a string we don't care about",
      "key2": "1",
    }),
    vec![
      MatchingRule::EachKey(MatchingRuleDefinition::new(
        "key1".to_string(), ValueType::String, MatchingRule::Regex("[a-z]{3,}[0-9]".to_string()), None
      )),
      MatchingRule::EachValue(MatchingRuleDefinition::new(
        "some string".to_string(), ValueType::Unknown, MatchingRule::Type, None
      ))
    ]
  );
  assert_eq!(matchable.to_example(), json!({
    "key1": "a string we don't care about",
    "key2": "1",
  }));
  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  expect!(rules).to(be_equal_to(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachKey(MatchingRuleDefinition::new("key1".to_string(), ValueType::String,
        MatchingRule::Regex("[a-z]{3,}[0-9]".to_string()), None)),
      MatchingRule::EachValue(MatchingRuleDefinition::new("some string".to_string(), ValueType::Unknown,
        MatchingRule::Type, None))
    ]
  }));
}

#[test]
fn object_matching_into() {
    // Make sure we can convert `ObjectMatching` into different pattern types.
    let _: JsonPattern = ObjectMatching::new(json_pattern!({}), vec![]).into();
}

/// A pattern which can take a JSON pattern and then apply a number of matching rules to the
/// resulting JSON object.
///
/// ```
/// use pact_consumer::*;
/// use pact_consumer::prelude::{each_key, each_value};
///
/// # fn main() {
/// object_matching!(
///   json_pattern!({
///       "key1": "a string",
///       "key2": "1",
///   }),
///   [
///       each_key(matching_regex!("[a-z]{3}[0-9]", "key1")),
///       each_value(like!("value1"))
///   ]
/// );
/// # }
/// ```
#[macro_export]
macro_rules! object_matching {
  ($example:expr, [ $( $rule:expr ),* ]) => {{
      let mut _rules: Vec<pact_models::matchingrules::MatchingRule> = vec![];

      $(
        _rules.push($rule.into());
      )*

      $crate::patterns::ObjectMatching::new(json_pattern!($example), _rules)
  }}
}

#[test]
fn object_matching_test() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;
  use serde_json::json;
  use pretty_assertions::assert_eq;

  let matchable = object_matching!(
    json_pattern!({
        "key1": "a string",
        "key2": "1",
    }),
    [
        each_key(matching_regex!("[a-z]{3}[0-9]", "key1")),
        each_value(like!("value1"))
    ]
  );
  expect!(matchable.to_example()).to(be_equal_to(json!({
    "key1": "a string",
    "key2": "1"
  })));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  assert_eq!(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachKey(MatchingRuleDefinition::new("key1".to_string(), ValueType::String,
        MatchingRule::Regex("[a-z]{3}[0-9]".to_string()), None)),
      MatchingRule::EachValue(MatchingRuleDefinition::new("\"value1\"".to_string(),
        ValueType::String, MatchingRule::Type, None))
    ]
  }, rules);
}

#[test]
fn object_matching_supports_nested_matching_rules() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;
  use serde_json::json;
  use pretty_assertions::assert_eq;

  let matchable = object_matching!(
    json_pattern!({
      "key1": {
        "id": matching_regex!("[0-9]+", "1000"),
        "desc": like!("description")
      }
    }),
    [
        each_key(matching_regex!("[a-z]{3}[0-9]", "key1"))
    ]
  );
  expect!(matchable.to_example()).to(be_equal_to(json!({
    "key1": {
      "id": "1000",
      "desc": "description"
    }
  })));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  assert_eq!(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachKey(MatchingRuleDefinition::new("key1".to_string(), ValueType::String,
        MatchingRule::Regex("[a-z]{3}[0-9]".to_string()), None))
    ],
    "$.*.id" => [ MatchingRule::Regex("[0-9]+".to_string()) ],
    "$.*.desc" => [ MatchingRule::Type ]
  }, rules);
}

/// Apply an associated rule to each key of an Object
#[derive(Debug)]
pub struct EachKey {
  /// The pattern we use to match.
  pattern: StringPattern
}

impl EachKey {
  /// Construct a new `EachKey`, given a pattern and example key.
  pub fn new<Nested: Into<StringPattern>>(pattern: Nested) -> Self {
    EachKey {
      pattern: pattern.into()
    }
  }
}

impl Pattern for EachKey {
  type Matches = String;

  fn to_example(&self) -> Self::Matches {
    self.pattern.to_example()
  }

  fn to_example_bytes(&self) -> Vec<u8> {
    self.to_example().into_bytes()
  }

  fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
    rules_out.add_rule(path, self.into(), RuleLogic::And);
  }
}

impl Into<MatchingRule> for EachKey {
  fn into(self) -> MatchingRule {
    (&self).into()
  }
}

impl Into<MatchingRule> for &EachKey {
  fn into(self) -> MatchingRule {
    let mut tmp = MatchingRuleCategory::empty("body");
    self.pattern.extract_matching_rules(DocPath::root(), &mut tmp);
    MatchingRule::EachKey(MatchingRuleDefinition {
      value: self.to_example(),
      value_type: ValueType::String,
      rules: tmp.rules.values()
        .flat_map(|list| list.rules.iter())
        .map(|rule| Either::Left(rule.clone()))
        .collect(),
      generator: None
    })
  }
}

#[test]
fn each_key_is_pattern() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;

  let matchable = EachKey::new(
    matching_regex!("\\d+", "100")
  );
  expect!(matchable.to_example()).to(be_equal_to("100"));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  expect!(rules).to(be_equal_to(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachKey(MatchingRuleDefinition::new("100".to_string(), ValueType::String,
        MatchingRule::Regex("\\d+".to_string()), None))
    ]
  }));
}

/// A pattern which applies another pattern to each key of an object, and which generates an
/// example key. A regex matcher is the only matcher that makes sense to use on keys.
///
/// ```
/// use pact_consumer::*;
///
/// # fn main() {
/// // Each key must match the given regex, and an example key is supplied.
/// use pact_consumer::patterns::each_key;
/// each_key(matching_regex!("[a-z]{3}[0-9]", "key1"));
/// # }
/// ```
pub fn each_key<P>(pattern: P) -> EachKey where P: Into<StringPattern> {
  EachKey::new(pattern.into())
}

#[test]
fn each_key_test() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;

  let matchable = each_key(matching_regex!("[a-z]{3}[0-9]", "key1"));
  expect!(matchable.to_example()).to(be_equal_to("key1"));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  expect!(rules).to(be_equal_to(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachKey(MatchingRuleDefinition::new("key1".to_string(), ValueType::String,
        MatchingRule::Regex("[a-z]{3}[0-9]".to_string()), None))
    ]
  }));
}

/// Apply an associated rule to each value of an Object
#[derive(Debug)]
pub struct EachValue {
  /// The regex we use to match.
  rule: JsonPattern
}

impl EachValue {
  /// Construct a new `EachValue`, given a pattern and example JSON.
  pub fn new<P: Into<JsonPattern>>(pattern: P) -> Self {
    EachValue {
      rule: pattern.into()
    }
  }
}

impl Pattern for EachValue
{
  type Matches = Value;

  fn to_example(&self) -> Self::Matches {
    self.rule.to_example()
  }

  fn to_example_bytes(&self) -> Vec<u8> {
    self.to_example().to_string().into_bytes()
  }

  fn extract_matching_rules(&self, path: DocPath, rules_out: &mut MatchingRuleCategory) {
    rules_out.add_rule(path, self.into(), RuleLogic::And);
  }
}

impl Into<MatchingRule> for EachValue {
  fn into(self) -> MatchingRule {
    (&self).into()
  }
}

impl Into<MatchingRule> for &EachValue {
  fn into(self) -> MatchingRule {
    let mut tmp = MatchingRuleCategory::empty("body");
    self.rule.extract_matching_rules(DocPath::root(), &mut tmp);
    MatchingRule::EachValue(MatchingRuleDefinition {
      value: self.to_example().to_string(),
      value_type: ValueType::String,
      rules: tmp.rules.values()
        .flat_map(|list| list.rules.iter())
        .map(|rule| Either::Left(rule.clone()))
        .collect(),
      generator: None
    })
  }
}

#[test]
fn each_value_is_pattern() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;

  let matchable = EachValue::new(
    matching_regex!("\\d+", "100")
  );
  expect!(matchable.to_example()).to(be_equal_to("100"));

  let mut rules = MatchingRuleCategory::empty("body");
  matchable.extract_matching_rules(DocPath::root(), &mut rules);
  expect!(rules).to(be_equal_to(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachValue(MatchingRuleDefinition::new("\"100\"".to_string(), ValueType::String,
        MatchingRule::Regex("\\d+".to_string()), None))
    ]
  }));
}

/// A pattern which applies another pattern to each value of an object, and which generates an
/// example value.
///
/// ```
/// use pact_consumer::*;
/// use pact_consumer::prelude::each_value;
///
/// # fn main() {
/// // Each value must match the given regex, and an example value is supplied.
/// each_value(matching_regex!("[a-z]{3}[0-9]", "value1"));
/// # }
/// ```
pub fn each_value<P: Into<JsonPattern>>(pattern: P) -> EachValue {
  EachValue::new(pattern.into())
}

#[test]
fn each_value_test() {
  use expectest::prelude::*;
  use pact_models::matchingrules_list;

  let result = each_value(matching_regex!("[a-z]{5}[0-9]", "value1"));
  expect!(result.to_example()).to(be_equal_to("value1"));

  let mut rules = MatchingRuleCategory::empty("body");
  result.extract_matching_rules(DocPath::root(), &mut rules);
  expect!(rules).to(be_equal_to(matchingrules_list! {
    "body"; "$" => [
      MatchingRule::EachValue(MatchingRuleDefinition::new("\"value1\"".to_string(), ValueType::String,
        MatchingRule::Regex("[a-z]{5}[0-9]".to_string()), None))
    ]
  }));
}
