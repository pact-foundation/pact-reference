//! # Matching Rule definition expressions
//!
//! Parser for parsing matching rule definitions into a value, matching rules and generator tuple.
//!
//! The following are examples of matching rule definitions:
//! * `matching(type,'Name')` - type matcher
//! * `matching(number,100)` - number matcher
//! * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
//!
//! ## Primitive values
//!
//! Primitive (or scalar) values can be strings (quoted with single quotes), numbers, booleans or null.
//! Characters in strings can be escaped using a backslash. The standard escape sequences are
//! supported:
//! * `\'` quote
//! * `\b` backspace
//! * `\f` formfeed
//! * `\n` linefeed
//! * `\r` carriage return
//! * `\t` tab
//! * `\uXXXX` unicode hex code (4 digits)
//! * `\u{X...}` unicode hex code (can be more than 4 digits)
//!
//! ## Expressions
//!
//! The main types of expressions are one of the following:
//!
//! ### matching(TYPE [, CONFIG], EXAMPLE) or matching($'NAME')
//!
//! Expression that defines a matching rule. Each matching rule requires the type of matching rule, and can contain an optional
//! configuration value. The final value is the example value to use.
//!
//! Supported matching rules:
//!
//! | Rule        | Description                                                                                           | Config Value       | Example                                                                       |
//! |-------------|-------------------------------------------------------------------------------------------------------|--------------------|-------------------------------------------------------------------------------|
//! | equalTo     | Value must be equal to the example                                                                    |                    | `matching(equalTo, 'Example value')`                                          |
//! | type        | Value must be the same type as the example                                                            |                    | `matching(type, 'Example value')`                                             |
//! | number      | Value must be a numeric value                                                                         |                    | `matching(number, 100.09)`                                                    |
//! | integer     | Value must be an integer value (no decimals)                                                          |                    | `matching(integer, 100)`                                                      |
//! | decimal     | Value must be a decimal number (must have at least one significant figure after the decimal point)    |                    | `matching(decimnal, 100.01)`                                                  |
//! | datetime    | Value must match a date-time format string                                                            | Format String      | `matching(datetime, 'yyyy-MM-dd HH:mm:ssZZZZZ', '2020-05-21 16:44:32+10:00')` |
//! | date        | Value must match a date format string                                                                 | Format String      | `matching(date, 'yyyy-MM-dd', '2020-05-21')`                                       |
//! | time        | Value must match a time format string                                                                 | Format String      | `matching(time, 'HH:mm', '22:04')`                                            |
//! | regex       | Value must match a regular expression                                                                 | Regular expression | `matching(regex, '\\w{3}\\d+', 'abc123')`                                     |
//! | include     | Value must include the example value as a substring                                                   |                    | `matching(include, 'testing')`                                                |
//! | boolean     | Value must be a boolean                                                                               |                    | `matching(boolean, true)`                                                     |
//! | server      | Value must match the semver specification                                                             |                    | `matching(semver, '1.0.0')`                                                   |
//! | contentType | Value must be of the provided content type. This will preform a magic test on the bytes of the value. | Content type       | `matching(contentType, 'application/xml', '<?xml?><test/>')`                  |
//!
//! The final form is a reference to another key. This is used to setup type matching using an example value, and is normally
//! used for collections. The name of the key must be a string value in single quotes.
//!
//! For example, to configure a type matcher where each value in a list must match the definition of a person:
//!
//! ```json
//! {
//!   "pact:match": "eachValue(matching($'person'))",
//!   "person": {
//!     "name": "Fred",
//!     "age": 100
//!   }
//! }
//! ```
//!
//! ### notEmpty(EXAMPLE)
//!
//! Expression that defines the value the same type as the example, must be present and not empty. This is used to defined
//! required fields.
//!
//! Example: `notEmpty('test')`
//!
//! ### eachKey(EXPRESSION)
//!
//! Configures a matching rule to be applied to each key in a map.
//!
//! For example: `eachKey(matching(regex, '\$(\.\w+)+', '$.test.one'))`
//!
//! ### eachValue(EXPRESSION)
//!
//! Configures a matching rule to be applied to each value in a map or list.
//!
//! For example: `eachValue(matching(type, 100))`
//!
//! ### atLeast(SIZE)
//!
//! Configures a type matching rule to be applied to a map or list (if another rule is not applied),
//! and asserts the length is at least the given size.
//!
//! For example: `atLeast(2)`
//!
//! ### atMost(SIZE)
//!
//! Configures a type matching rule to be applied to a map or list (if another rule is not applied), and asserts the
//! length is at most the given size.
//!
//! For example: `atMost(2)`
//!
//! ## Composing expressions
//!
//! Expressions can be composed by separating them with a comma. For example
//! `atLeast(2), atMost(10), eachValue(matching(regex, '\d+', '1234'))`. This will configure an
//! array to have to have at least 2 items, at most 10, and each item in the array must match the
//! given regex.
//!
//! ## Grammar
//!
//! There is a grammar for the definitions in [ANTLR4 format](https://github.com/pact-foundation/pact-plugins/blob/main/docs/matching-rule-definition.g4).
//!

use std::fmt::{Display, Formatter};
use std::char::REPLACEMENT_CHARACTER;
use std::str::from_utf8;

use anyhow::{anyhow, Error};
use ariadne::{Config, Label, Report, ReportKind, Source};
use bytes::{BufMut, BytesMut};
use itertools::Either;
use logos::{Lexer, Logos, Span};
use semver::Version;
use tracing::{instrument, trace, warn};

use crate::expression_parser::DataType;
use crate::generators::Generator;
use crate::generators::Generator::ProviderStateGenerator;
use crate::matchingrules::MatchingRule;
use crate::matchingrules::MatchingRule::{MaxType, MinType, NotEmpty};

/// Type to associate with an expression element
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
  Unknown,
  String,
  Number,
  Integer,
  Decimal,
  Boolean
}

impl ValueType {
  /// Merge this value type with the other one
  pub fn merge(self, other: ValueType) -> ValueType {
    match (self, other) {
      (ValueType::String, ValueType::String) => ValueType::String,
      (ValueType::Number, ValueType::Number) => ValueType::Number,
      (ValueType::Number, ValueType::Boolean) => ValueType::Number,
      (ValueType::Number, ValueType::Unknown) => ValueType::Number,
      (ValueType::Number, ValueType::Integer) => ValueType::Integer,
      (ValueType::Number, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Number, ValueType::String) => ValueType::String,
      (ValueType::Integer, ValueType::Number) => ValueType::Integer,
      (ValueType::Integer, ValueType::Boolean) => ValueType::Integer,
      (ValueType::Integer, ValueType::Unknown) => ValueType::Integer,
      (ValueType::Integer, ValueType::Integer) => ValueType::Integer,
      (ValueType::Integer, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Integer, ValueType::String) => ValueType::String,
      (ValueType::Decimal, ValueType::Number) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Boolean) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Unknown) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Integer) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::String) => ValueType::String,
      (ValueType::Boolean, ValueType::Number) => ValueType::Number,
      (ValueType::Boolean, ValueType::Integer) => ValueType::Integer,
      (ValueType::Boolean, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Boolean, ValueType::Unknown) => ValueType::Boolean,
      (ValueType::Boolean, ValueType::String) => ValueType::String,
      (ValueType::Boolean, ValueType::Boolean) => ValueType::Boolean,
      (ValueType::String, _) => ValueType::String,
      (_, _) => other
    }
  }
}

impl Display for ValueType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ValueType::Unknown => write!(f, "Unknown"),
      ValueType::String => write!(f, "String"),
      ValueType::Number => write!(f, "Number"),
      ValueType::Integer => write!(f, "Integer"),
      ValueType::Decimal => write!(f, "Decimal"),
      ValueType::Boolean => write!(f, "Boolean")
    }
  }
}

impl Into<DataType> for ValueType {
  fn into(self) -> DataType {
    match self {
      ValueType::Unknown => DataType::RAW,
      ValueType::String => DataType::STRING,
      ValueType::Number => DataType::DECIMAL,
      ValueType::Integer => DataType::INTEGER,
      ValueType::Decimal => DataType::DECIMAL,
      ValueType::Boolean => DataType::BOOLEAN
    }
  }
}

/// Reference to another attribute that defines the structure of the matching rule
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchingReference {
  /// Name of the attribute that the reference is to
  pub name: String
}

/// Matching rule definition constructed from parsing a matching rule definition expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchingRuleDefinition {
  pub value: String,
  pub value_type: ValueType,
  pub rules: Vec<Either<MatchingRule, MatchingReference>>,
  pub generator: Option<Generator>,
  pub expression: String
}

impl MatchingRuleDefinition {
  /// Construct a new MatchingRuleDefinition
  pub fn new(
    value: String,
    value_type: ValueType,
    matching_rule: MatchingRule,
    generator: Option<Generator>,
    expression: String
  ) -> Self {
    MatchingRuleDefinition {
      value,
      value_type,
      rules: vec![ Either::Left(matching_rule) ],
      generator,
      expression
    }
  }

  /// Merges two matching rules definitions. This is used when multiple matching rules are
  /// provided for a single element.
  pub fn merge(&self, other: &MatchingRuleDefinition) -> MatchingRuleDefinition {
    trace!("Merging {:?} with {:?}", self, other);
    if !self.value.is_empty() && !other.value.is_empty() {
      warn!("There are multiple matching rules with values for the same value. There is no \
        reliable way to combine them, so the later value ('{}') will be ignored.", other.value)
    }

    if self.generator.is_some() && other.generator.is_some() {
      warn!("There are multiple generators for the same value. There is no reliable way to combine \
       them, so the later generator ({:?}) will be ignored.", other.generator)
    }

    MatchingRuleDefinition {
      value: if self.value.is_empty() { other.value.clone() } else { self.value.clone() },
      value_type: self.value_type.merge(other.value_type),
      rules: [self.rules.clone(), other.rules.clone()].concat(),
      generator: self.generator.as_ref().or_else(|| other.generator.as_ref()).cloned(),
      expression: if self.expression.is_empty() {
        other.expression.clone()
      } else if other.expression.is_empty() || self.expression == other.expression {
        self.expression.clone()
      } else {
        format!("{}, {}", self.expression, other.expression)
      }
    }
  }

  pub fn expression(&self) -> String {
    self.expression.clone()
  }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum MatcherDefinitionToken {
  #[token("matching")]
  Matching,

  #[token("notEmpty")]
  NotEmpty,

  #[token("eachKey")]
  EachKey,

  #[token("eachValue")]
  EachValue,

  #[token("atLeast")]
  AtLeast,

  #[token("atMost")]
  AtMost,

  #[token("(")]
  LeftBracket,

  #[token(")")]
  RightBracket,

  #[token(",")]
  Comma,

  #[regex(r"'(?:[^']|\\')*'")]
  String,

  #[regex("[a-zA-Z]+")]
  Id,

  #[regex("-[0-9]+", |lex| lex.slice().parse().ok())]
  Int(i64),

  #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
  Num(usize),

  #[regex(r"-?[0-9]\.[0-9]+")]
  Decimal,

  #[regex(r"\.[0-9]+")]
  DecimalPart,

  #[regex(r"true|false")]
  Boolean,

  #[regex(r"null")]
  Null,

  #[token("$")]
  Dollar
}

/// Parse a matcher definition into a MatchingRuleDefinition containing the example value, matching rules and any
/// generator.
/// The following are examples of matching rule definitions:
/// * `matching(type,'Name')` - type matcher
/// * `matching(number,100)` - number matcher
/// * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
#[instrument(level = "debug", ret)]
pub fn parse_matcher_def(v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  if v.is_empty() {
    Err(anyhow!("Expected a matching rule definition, but got an empty string"))
  } else {
    let mut lex = MatcherDefinitionToken::lexer(v);
    matching_definition(&mut lex, v)
  }
}

/// Determines if a sting starts with a valid matching rule definition. This is used in the case
/// where a value can be a matching rule definition or a plain string value
pub fn is_matcher_def(v: &str) -> bool {
  if v.is_empty() {
    false
  } else {
    let mut lex = MatcherDefinitionToken::lexer(v);
    let next = lex.next();
    if let Some(Ok(token)) = next {
      if token == MatcherDefinitionToken::Matching || token == MatcherDefinitionToken::NotEmpty ||
        token == MatcherDefinitionToken::EachKey || token == MatcherDefinitionToken::EachValue {
        true
      } else {
        false
      }
    } else {
      false
    }
  }
}

// matchingDefinition returns [ MatchingRuleDefinition value ] :
//     matchingDefinitionExp ( COMMA matchingDefinitionExp )* EOF
//     ;
fn matching_definition(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  let mut value = matching_definition_exp(lex, v)?;
  while let Some(Ok(next)) = lex.next() {
    if next == MatcherDefinitionToken::Comma {
      value = value.merge(&matching_definition_exp(lex, v)?);
    } else {
      return Err(anyhow!("expected comma, got '{}'", lex.slice()));
    }
  }

  let remainder = lex.remainder();
  if !remainder.is_empty() {
    Err(anyhow!("expected not more tokens, got '{}' with '{}' remaining", lex.slice(), remainder))
  } else {
    Ok(value)
  }
}

// matchingDefinitionExp returns [ MatchingRuleDefinition value ] :
//     (
//       'matching' LEFT_BRACKET matchingRule RIGHT_BRACKET
//       | 'notEmpty' LEFT_BRACKET string RIGHT_BRACKET
//       | 'eachKey' LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET
//       | 'eachValue' LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET
//       | 'atLeast' LEFT_BRACKET DIGIT+ RIGHT_BRACKET
//       | 'atMost' LEFT_BRACKET DIGIT+ RIGHT_BRACKET
//     )
//     ;
fn matching_definition_exp(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  let next = lex.next();
  if let Some(Ok(token)) = &next {
    if token == &MatcherDefinitionToken::Matching {
      let (value, value_type, matching_rule, generator, reference) = parse_matching(lex, v)?;
      if let Some(reference) = reference {
        Ok(MatchingRuleDefinition {
          value,
          value_type: ValueType::Unknown,
          rules: vec![ Either::Right(reference) ],
          generator,
          expression: v.to_string()
        })
      } else {
        Ok(MatchingRuleDefinition {
          value,
          value_type,
          rules: vec![ Either::Left(matching_rule.unwrap()) ],
          generator,
          expression: v.to_string()
        })
      }
    } else if token == &MatcherDefinitionToken::NotEmpty {
      let (value, value_type, generator) = parse_not_empty(lex, v)?;
      Ok(MatchingRuleDefinition {
        value,
        value_type,
        rules: vec![Either::Left(NotEmpty)],
        generator,
        expression: v.to_string()
      })
    } else if token == &MatcherDefinitionToken::EachKey {
      let definition = parse_each_key(lex, v)?;
      Ok(definition)
    } else if token == &MatcherDefinitionToken::EachValue {
      let definition = parse_each_value(lex, v)?;
      Ok(definition)
    } else if token == &MatcherDefinitionToken::AtLeast {
      let length = parse_length_param(lex, v)?;
      Ok(MatchingRuleDefinition {
        value: String::default(),
        value_type: ValueType::Unknown,
        rules: vec![Either::Left(MinType(length))],
        generator: None,
        expression: v.to_string()
      })
    } else if token == &MatcherDefinitionToken::AtMost {
      let length = parse_length_param(lex, v)?;
      Ok(MatchingRuleDefinition {
        value: String::default(),
        value_type: ValueType::Unknown,
        rules: vec![Either::Left(MaxType(length))],
        generator: None,
        expression: v.to_string()
      })
    } else {
      let mut buffer = BytesMut::new().writer();
      let span = lex.span();
      let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
        .with_config(Config::default().with_color(false))
        .with_message(format!("Expected a type of matching rule definition, but got '{}'", lex.slice()))
        .with_label(Label::new(("expression", span)).with_message("Expected a matching rule definition here"))
        .with_note("valid matching rule definitions are: matching, notEmpty, eachKey, eachValue, atLeast, atMost")
        .finish();
      report.write(("expression", Source::from(v)), &mut buffer)?;
      let message = from_utf8(&*buffer.get_ref())?.to_string();
      Err(anyhow!(message))
    }
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected a type of matching rule definition but got the end of the expression"))
      .with_label(Label::new(("expression", span)).with_message("Expected a matching rule definition here"))
      .with_note("valid matching rule definitions are: matching, notEmpty, eachKey, eachValue, atLeast, atMost")
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

// LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET {
//   if ($e.value != null) {
//     $value = new MatchingRuleDefinition(null, ValueType.Unknown, List.of((Either<MatchingRule, MatchingReference>) new Either.A(new EachValueMatcher($e.value))), null);
//   }
// }
fn parse_each_value(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  let next = lex.next()
    .ok_or_else(|| end_of_expression(v, "an opening bracket"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let result = matching_definition_exp(lex, v)?;
    let next = lex.next().ok_or_else(|| end_of_expression(v, "a closing bracket"))?;
    if let Ok(MatcherDefinitionToken::RightBracket) = next {
      Ok(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachValue(result)) ],
        generator: None,
        expression: v.to_string()
      })
    } else {
      Err(anyhow!(error_message(lex, v, "Expected a closing bracket", "Expected a closing bracket before this")?))
    }
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected an opening bracket, got '{}'", lex.slice()))
      .with_label(Label::new(("expression", span)).with_message("Expected an opening bracket before this"))
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

fn error_message(lex: &mut Lexer<MatcherDefinitionToken>, v: &str, error: &str, additional: &str) -> Result<String, Error> {
  let mut buffer = BytesMut::new().writer();
  let span = lex.span();
  let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
    .with_config(Config::default().with_color(false))
    .with_message(format!("{}, got '{}'", error, lex.slice()))
    .with_label(Label::new(("expression", span)).with_message(additional))
    .finish();
  report.write(("expression", Source::from(v)), &mut buffer)?;
  let message = from_utf8(&*buffer.get_ref())?.to_string();
  Ok(message)
}

// LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET
fn parse_each_key(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  let next = lex.next()
    .ok_or_else(|| end_of_expression(v, "an opening bracket"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let result = matching_definition_exp(lex, v)?;
    let next = lex.next().ok_or_else(|| end_of_expression(v, "a closing bracket"))?;
    if let Ok(MatcherDefinitionToken::RightBracket) = next {
      Ok(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachKey(result)) ],
        generator: None,
        expression: v.to_string()
      })
    } else {
      let mut buffer = BytesMut::new().writer();
      let span = lex.span();
      let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
        .with_config(Config::default().with_color(false))
        .with_message(format!("Expected a closing bracket, got '{}'", lex.slice()))
        .with_label(Label::new(("expression", span)).with_message("Expected a closing bracket before this"))
        .finish();
      report.write(("expression", Source::from(v)), &mut buffer)?;
      let message = from_utf8(&*buffer.get_ref())?.to_string();
      Err(anyhow!(message))
    }
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected an opening bracket, got '{}'", lex.slice()))
      .with_label(Label::new(("expression", span)).with_message("Expected an opening bracket before this"))
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

// LEFT_BRACKET primitiveValue RIGHT_BRACKET
fn parse_not_empty(
  lex: &mut Lexer<MatcherDefinitionToken>,
  v: &str
) -> anyhow::Result<(String, ValueType, Option<Generator>)> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "'('"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let result = parse_primitive_value(lex, v, false)?;
    let next = lex.next().ok_or_else(|| end_of_expression(v, "')'"))?;
    if let Ok(MatcherDefinitionToken::RightBracket) = next {
      Ok(result)
    } else {
      Err(anyhow!("expected closing bracket, got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("expected '(', got '{}'", lex.remainder()))
  }
}

// LEFT_BRACKET matchingRule RIGHT_BRACKET
fn parse_matching(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "'('"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let result = parse_matching_rule(lex, v)?;
    let next = lex.next().ok_or_else(|| end_of_expression(v, "')'"))?;
    if let Ok(MatcherDefinitionToken::RightBracket) = next {
      Ok(result)
    } else {
      Err(anyhow!(error_message(lex, v, "Expected a closing bracket", "Expected a closing bracket before this")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected an opening bracket", "Expected an opening bracket before this")?))
  }
}

// matchingRule returns [ String value, ValueType type, MatchingRule rule, Generator generator, MatchingReference reference ] :
//   (
//     ( 'equalTo' { $rule = EqualsMatcher.INSTANCE; }
//     | 'type'  { $rule = TypeMatcher.INSTANCE; } )
//     COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
//   | 'number' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.NUMBER); } COMMA val=( DECIMAL_LITERAL | INTEGER_LITERAL ) { $value = $val.getText(); $type = ValueType.Number; }
//   | 'integer' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.INTEGER); } COMMA val=INTEGER_LITERAL { $value = $val.getText(); $type = ValueType.Integer; }
//   | 'decimal' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.DECIMAL); } COMMA val=DECIMAL_LITERAL { $value = $val.getText(); $type = ValueType.Decimal; }
//   | matcherType=( 'datetime' | 'date' | 'time' ) COMMA format=string {
//     if ($matcherType.getText().equals("datetime")) { $rule = new TimestampMatcher($format.contents); }
//     if ($matcherType.getText().equals("date")) { $rule = new DateMatcher($format.contents); }
//     if ($matcherType.getText().equals("time")) { $rule = new TimeMatcher($format.contents); }
//     } COMMA s=string { $value = $s.contents; $type = ValueType.String; }
//   | 'regex' COMMA r=string COMMA s=string { $rule = new RegexMatcher($r.contents); $value = $s.contents; $type = ValueType.String; }
//   | 'include' COMMA s=string { $rule = new IncludeMatcher($s.contents); $value = $s.contents; $type = ValueType.String; }
//   | 'boolean' COMMA BOOLEAN_LITERAL { $rule = BooleanMatcher.INSTANCE; $value = $BOOLEAN_LITERAL.getText(); $type = ValueType.Boolean; }
//   | 'semver' COMMA s=string { $rule = SemverMatcher.INSTANCE; $value = $s.contents; $type = ValueType.String; }
//   | 'contentType' COMMA ct=string COMMA s=string { $rule = new ContentTypeMatcher($ct.contents); $value = $s.contents; $type = ValueType.Unknown; }
//   | DOLLAR ref=string { $reference = new MatchingReference($ref.contents); $type = ValueType.Unknown; }
//   ;
fn parse_matching_rule(lex: &mut logos::Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  let next = lex.next()
    .ok_or_else(|| end_of_expression(v, "a matcher (equalTo, regex, etc.)"))?;
  if let Ok(MatcherDefinitionToken::Id) = next {
    match lex.slice() {
      "equalTo" => parse_equality(lex, v),
      "regex" => parse_regex(lex, v),
      "type" => parse_type(lex, v),
      "datetime" => parse_datetime(lex, v),
      "date" => parse_date(lex, v),
      "time" => parse_time(lex, v),
      "include" => parse_include(lex, v),
      "number" => parse_number(lex, v),
      "integer" => parse_integer(lex, v),
      "decimal" => parse_decimal(lex, v),
      "boolean" => parse_boolean(lex, v),
      "contentType" => parse_content_type(lex, v),
      "semver" => parse_semver(lex, v),
      _ => {
        let mut buffer = BytesMut::new().writer();
        let span = lex.span();
        let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
          .with_config(Config::default().with_color(false))
          .with_message(format!("Expected the type of matcher, got '{}'", lex.slice()))
          .with_label(Label::new(("expression", span)).with_message("This is not a valid matcher type"))
          .with_note("Valid matchers are: equalTo, regex, type, datetime, date, time, include, number, integer, decimal, boolean, contentType, semver")
          .finish();
        report.write(("expression", Source::from(v)), &mut buffer)?;
        let message = from_utf8(&*buffer.get_ref())?.to_string();
        Err(anyhow!(message))
      }
    }
  } else if let Ok(MatcherDefinitionToken::Dollar) = next {
    parse_reference(lex, v)
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected the type of matcher, got '{}'", lex.slice()))
      .with_label(Label::new(("expression", span)).with_message("Expected a matcher (equalTo, regex, etc.) here"))
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

fn parse_reference(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  let name = parse_string(lex, v)?;
  Ok((name.clone(), ValueType::Unknown, None, None, Some(MatchingReference { name })))
}

// COMMA s=string { $rule = SemverMatcher.INSTANCE; $value = $s.contents; $type = ValueType.String; }
fn parse_semver(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let value = parse_string(lex, v)?;

  match Version::parse(value.as_str()) {
    Ok(_) => Ok((value, ValueType::String, Some(MatchingRule::Semver), None, None)),
    Err(err) => {
      let mut buffer = BytesMut::new().writer();
      let span = lex.span();
      let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
        .with_config(Config::default().with_color(false))
        .with_message(format!("Expected a semver compatible string, got {} - {}", lex.slice(), err))
        .with_label(Label::new(("expression", span)).with_message("This is not a valid semver value"))
        .finish();
      report.write(("expression", Source::from(v)), &mut buffer)?;
      let message = from_utf8(&*buffer.get_ref())?.to_string();
      Err(anyhow!(message))
    }
  }
}

//     COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
fn parse_equality(
  lex: &mut Lexer<MatcherDefinitionToken>,
  v: &str
) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let (value, value_type, generator) = parse_primitive_value(lex, v, false)?;
  Ok((value, value_type, Some(MatchingRule::Equality), generator, None))
}

// COMMA r=string COMMA s=(string { $rule = new RegexMatcher($r.contents); $value = $s.contents; $type = ValueType.String; } | 'fromProviderState' fromProviderState)
fn parse_regex(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let regex = parse_string(lex, v)?;
  parse_comma(lex, v)?;

  let remainder = lex.remainder().trim_start();
  let (value, value_type, generator) = if remainder.starts_with("fromProviderState") {
    lex.next();
    from_provider_state(lex, v)?
  } else {
    (parse_string(lex, v)?, ValueType::String, None)
  };

  Ok((value, value_type, Some(MatchingRule::Regex(regex)), generator, None))
}

// COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
fn parse_type(
  lex: &mut Lexer<MatcherDefinitionToken>,
  v: &str
) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let (value, value_type, generator) = parse_primitive_value(lex, v, false)?;
  Ok((value, value_type, Some(MatchingRule::Type), generator, None))
}

// COMMA format=string COMMA s=(string | 'fromProviderState' fromProviderState)
fn parse_datetime(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let format = parse_string(lex, v)?;
  parse_comma(lex, v)?;

  let remainder = lex.remainder().trim_start();
  let (value, value_type, generator) = if remainder.starts_with("fromProviderState") {
    lex.next();
    from_provider_state(lex, v)?
  } else {
    (parse_string(lex, v)?, ValueType::String, Some(Generator::DateTime(Some(format.clone()), None)))
  };

  Ok((value, value_type, Some(MatchingRule::Timestamp(format.clone())), generator, None))
}

// COMMA format=string COMMA s=(string | 'fromProviderState' fromProviderState)
fn parse_date(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let format = parse_string(lex, v)?;
  parse_comma(lex, v)?;

  let remainder = lex.remainder().trim_start();
  let (value, value_type, generator) = if remainder.starts_with("fromProviderState") {
    lex.next();
    from_provider_state(lex, v)?
  } else {
    (parse_string(lex, v)?, ValueType::String, Some(Generator::Date(Some(format.clone()), None)))
  };

  Ok((value, value_type, Some(MatchingRule::Date(format.clone())), generator, None))
}

// COMMA format=string COMMA s=(string | 'fromProviderState' fromProviderState)
fn parse_time(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let format = parse_string(lex, v)?;
  parse_comma(lex, v)?;

  let remainder = lex.remainder().trim_start();
  let (value, value_type, generator) = if remainder.starts_with("fromProviderState") {
    lex.next();
    from_provider_state(lex, v)?
  } else {
    (parse_string(lex, v)?, ValueType::String, Some(Generator::Time(Some(format.clone()), None)))
  };

  Ok((value, value_type, Some(MatchingRule::Time(format.clone())), generator, None))
}

// COMMA s=string { $rule = new IncludeMatcher($s.contents); $value = $s.contents; $type = ValueType.String; }
fn parse_include(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let value = parse_string(lex, v)?;
  Ok((value.clone(), ValueType::String, Some(MatchingRule::Include(value)), None, None))
}

// COMMA ct=string COMMA s=string { $rule = new ContentTypeMatcher($ct.contents); $value = $s.contents; $type = ValueType.Unknown; }
fn parse_content_type(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let ct = parse_string(lex, v)?;
  parse_comma(lex, v)?;
  let value = parse_string(lex, v)?;
  Ok((value, ValueType::Unknown, Some(MatchingRule::ContentType(ct)), None, None))
}

// primitiveValue returns [ String value, ValueType type ] :
//   string { $value = $string.contents; $type = ValueType.String; }
//   | v=DECIMAL_LITERAL { $value = $v.getText(); $type = ValueType.Decimal; }
//   | v=INTEGER_LITERAL { $value = $v.getText(); $type = ValueType.Integer; }
//   | v=BOOLEAN_LITERAL { $value = $v.getText(); $type = ValueType.Boolean; }
//   | STRING_LITERAL {
//     String contents = $STRING_LITERAL.getText();
//     $contents = contents.substring(1, contents.length() - 1);
//   }
//   | 'null'
//   | 'fromProviderState' fromProviderState
//   ;
fn parse_primitive_value(
  lex: &mut Lexer<MatcherDefinitionToken>,
  v: &str,
  already_called: bool
) -> anyhow::Result<(String, ValueType, Option<Generator>)> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "expected a primitive value"))?;
  match next {
    Ok(MatcherDefinitionToken::String) => Ok((lex.slice().trim_matches('\'').to_string(), ValueType::String, None)),
    Ok(MatcherDefinitionToken::Null) => Ok((String::new(), ValueType::String, None)),
    Ok(MatcherDefinitionToken::Int(_)) => {
      // Logos is returning an INT token when a Decimal should match. We need to now parse the
      // remaining pattern if it is a decimal
      if lex.remainder().starts_with('.') {
        let int_part = lex.slice();
        let _ = lex.next().ok_or_else(|| end_of_expression(v, "expected a number"))?;
        Ok((format!("{}{}", int_part, lex.slice()), ValueType::Decimal, None))
      } else {
        Ok((lex.slice().to_string(), ValueType::Integer, None))
      }
    },
    Ok(MatcherDefinitionToken::Num(_)) => {
      // Logos is returning an NUM token when a Decimal should match. We need to now parse the
      // remaining pattern if it is a decimal
      if lex.remainder().starts_with('.') {
        let int_part = lex.slice();
        let _ = lex.next().ok_or_else(|| end_of_expression(v, "expected a number"))?;
        Ok((format!("{}{}", int_part, lex.slice()), ValueType::Decimal, None))
      } else {
        Ok((lex.slice().to_string(), ValueType::Integer, None))
      }
    },
    Ok(MatcherDefinitionToken::Decimal) => Ok((lex.slice().to_string(), ValueType::Decimal, None)),
    Ok(MatcherDefinitionToken::Boolean) => Ok((lex.slice().to_string(), ValueType::Boolean, None)),
    Ok(MatcherDefinitionToken::Id) if lex.slice() == "fromProviderState" && !already_called => {
      from_provider_state(lex, v)
    },
    _ => Err(anyhow!(error_message(lex, v, "Expected a primitive value", "Expected a primitive value here")?))
  }
}

// COMMA val=( DECIMAL_LITERAL | INTEGER_LITERAL | 'fromProviderState' fromProviderState)
#[allow(clippy::if_same_then_else)]
fn parse_number(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let next = lex.next().ok_or_else(|| end_of_expression(v, "expected a number"))?;
  if let Ok(MatcherDefinitionToken::Decimal) = next {
    Ok((lex.slice().to_string(), ValueType::Number,  Some(MatchingRule::Number), None, None))
  } else if let Ok(MatcherDefinitionToken::Int(_) | MatcherDefinitionToken::Num(_)) = next {
    // Logos is returning an INT token when a Decimal should match. We need to now parse the
    // remaining pattern if it is a decimal
    if lex.remainder().starts_with('.') {
      let int_part = lex.slice();
      let _ = lex.next().ok_or_else(|| end_of_expression(v, "expected a number"))?;
      Ok((format!("{}{}", int_part, lex.slice()), ValueType::Number, Some(MatchingRule::Number), None, None))
    } else {
      Ok((lex.slice().to_string(), ValueType::Number, Some(MatchingRule::Number), None, None))
    }
  } else if let Ok(MatcherDefinitionToken::Id) = next {
    if lex.slice() == "fromProviderState" {
      let (value, value_type, generator) = from_provider_state(lex, v)?;
      Ok((value, value_type, Some(MatchingRule::Number), generator, None))
    } else {
      Err(anyhow!(error_message(lex, v, "Expected a number", "Expected a number here")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected a number", "Expected a number here")?))
  }
}

// COMMA val=INTEGER_LITERAL { $value = $val.getText(); $type = ValueType.Integer; }
fn parse_integer(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let next = lex.next().ok_or_else(|| end_of_expression(v, "expected an integer"))?;
  if let Ok(MatcherDefinitionToken::Int(_) | MatcherDefinitionToken::Num(_)) = next {
    Ok((lex.slice().to_string(), ValueType::Integer, Some(MatchingRule::Integer), None, None))
  } else if let Ok(MatcherDefinitionToken::Id) = next {
    if lex.slice() == "fromProviderState" {
      let (value, value_type, generator) = from_provider_state(lex, v)?;
      Ok((value, value_type, Some(MatchingRule::Integer), generator, None))
    } else {
      Err(anyhow!(error_message(lex, v, "Expected an integer", "Expected an integer here")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected an integer", "Expected an integer here")?))
  }
}

// COMMA val=DECIMAL_LITERAL { $value = $val.getText(); $type = ValueType.Decimal; }
#[allow(clippy::if_same_then_else)]
fn parse_decimal(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let next = lex.next().ok_or_else(|| end_of_expression(v, "expected a decimal number"))?;
  if let Ok(MatcherDefinitionToken::Int(_) | MatcherDefinitionToken::Num(_)) = next {
    // Logos is returning an INT token when a Decimal should match. We need to now parse the
    // remaining pattern if it is a decimal
    if lex.remainder().starts_with('.') {
      let int_part = lex.slice();
      let _ = lex.next().ok_or_else(|| end_of_expression(v, "expected a number"))?;
      Ok((format!("{}{}", int_part, lex.slice()), ValueType::Decimal, Some(MatchingRule::Decimal), None, None))
    } else {
      Ok((lex.slice().to_string(), ValueType::Decimal, Some(MatchingRule::Decimal), None, None))
    }
  } else if let Ok(MatcherDefinitionToken::Decimal) = next {
    Ok((lex.slice().to_string(), ValueType::Decimal, Some(MatchingRule::Decimal), None, None))
  } else if let Ok(MatcherDefinitionToken::Id) = next {
    if lex.slice() == "fromProviderState" {
      let (value, value_type, generator) = from_provider_state(lex, v)?;
      Ok((value, value_type, Some(MatchingRule::Number), generator, None))
    } else {
      Err(anyhow!(error_message(lex, v, "Expected a decimal number", "Expected a decimal number here")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected a decimal number", "Expected a decimal number here")?))
  }
}

// COMMA BOOLEAN_LITERAL { $rule = BooleanMatcher.INSTANCE; $value = $BOOLEAN_LITERAL.getText(); $type = ValueType.Boolean; }
fn parse_boolean(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex, v)?;
  let next = lex.next().ok_or_else(|| end_of_expression(v, "expected a boolean"))?;
  if let Ok(MatcherDefinitionToken::Boolean) = next {
    Ok((lex.slice().to_string(), ValueType::Boolean, Some(MatchingRule::Boolean), None, None))
  } else {
    Err(anyhow!(error_message(lex, v, "Expected a boolean", "Expected a boolean here")?))
  }
}

fn parse_string(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<String> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "a string"))?;
  if let Ok(MatcherDefinitionToken::String) = next {
    let span = lex.span();
    let raw_str = lex.slice().trim_matches('\'');
    process_raw_string(raw_str, span, v)
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected a string value, got {}", lex.slice()))
      .with_label(Label::new(("expression", span.clone())).with_message("Expected this to be a string"))
      .with_note(format!("Surround the value in quotes: {}'{}'{}", &v[..span.start], lex.slice(), &v[span.end..]))
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

fn process_raw_string(raw_str: &str, span: Span, v: &str) -> anyhow::Result<String> {
  let mut buffer = String::with_capacity(raw_str.len());
  let mut chars = raw_str.chars();
  while let Some(ch) = chars.next() {
    if ch == '\\' {
      match chars.next() {
        None => buffer.push(ch),
        Some(ch2) => {
          match ch2 {
            '\\' => buffer.push(ch),
            'b' => buffer.push('\x08'),
            'f' => buffer.push('\x0C'),
            'n' => buffer.push('\n'),
            'r' => buffer.push('\r'),
            't' => buffer.push('\t'),
            'u' => {
              let code1 = char_or_error(chars.next(), &span, v)?;
              let mut b = String::with_capacity(4);
              if code1 == '{' {
                loop {
                  let c = char_or_error(chars.next(), &span, v)?;
                  if c == '}' {
                    break;
                  } else {
                    b.push(c);
                  }
                }
              } else {
                b.push(code1);
                let code2 = char_or_error(chars.next(), &span, v)?;
                b.push(code2);
                let code3 = char_or_error(chars.next(), &span, v)?;
                b.push(code3);
                let code4 = char_or_error(chars.next(), &span, v)?;
                b.push(code4);
              }
              let code = match u32::from_str_radix(b.as_str(), 16) {
                Ok(c) => c,
                Err(err) => return string_error(&err, &span, v)
              };
              let c = char::from_u32(code).unwrap_or(REPLACEMENT_CHARACTER);
              buffer.push(c);
            }
            _ => {
              buffer.push(ch);
              buffer.push(ch2);
            }
          }
        }
      }
    } else {
      buffer.push(ch);
    }
  }
  Ok(buffer)
}

fn string_error(err: &dyn std::error::Error, span: &Span, v: &str) -> anyhow::Result<String> {
  let mut buffer = BytesMut::new().writer();
  let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
    .with_config(Config::default().with_color(false))
    .with_message(format!("Invalid unicode character escape sequence: {}", err))
    .with_label(Label::new(("expression", span.clone())).with_message("This string contains an invalid escape sequence"))
    .with_note("Unicode escape sequences must be in the form \\uXXXX (4 digits) or \\u{X..} (enclosed in braces)")
    .finish();
  report.write(("expression", Source::from(v)), &mut buffer)?;
  let message = from_utf8(&*buffer.get_ref())?.to_string();
  Err(anyhow!(message))
}

fn char_or_error(ch: Option<char>, span: &Span, v: &str) -> anyhow::Result<char> {
  match ch {
    Some(ch) => Ok(ch),
    None => {
      let mut buffer = BytesMut::new().writer();
      let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
        .with_config(Config::default().with_color(false))
        .with_message("Invalid unicode character escape sequence")
        .with_label(Label::new(("expression", span.clone())).with_message("This string contains an invalid escape sequence"))
        .with_note("Unicode escape sequences must be in the form \\uXXXX (4 digits) or \\u{X..} (enclosed in braces)")
        .finish();
      report.write(("expression", Source::from(v)), &mut buffer)?;
      let message = from_utf8(&*buffer.get_ref())?.to_string();
      Err(anyhow!(message))
    }
  }
}

fn parse_comma(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<()> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "a comma"))?;
  if let Ok(MatcherDefinitionToken::Comma) = next {
    Ok(())
  } else {
    let mut buffer = BytesMut::new().writer();
    let span = lex.span();
    let report = Report::build(ReportKind::Error, ("expression", span.start..span.start))
      .with_config(Config::default().with_color(false))
      .with_message(format!("Expected a comma, got '{}'", lex.slice()))
      .with_label(Label::new(("expression", span)).with_message("Expected a comma before this"))
      .finish();
    report.write(("expression", Source::from(v)), &mut buffer)?;
    let message = from_utf8(&*buffer.get_ref())?.to_string();
    Err(anyhow!(message))
  }
}

fn end_of_expression(v: &str, expected: &str) -> Error {
  let mut buffer = BytesMut::new().writer();
  let i = v.len();
  let report = Report::build(ReportKind::Error, ("expression", i..i))
    .with_config(Config::default().with_color(false))
    .with_message(format!("Expected {}, got the end of the expression", expected))
    .with_label(Label::new(("expression", i..i)).with_message(format!("Expected {} here", expected)))
    .finish();
  report.write(("expression", Source::from(v)), &mut buffer).unwrap();
  let message = from_utf8(&*buffer.get_ref()).unwrap().to_string();
  anyhow!(message)
}

// LEFT_BRACKET DIGIT+ RIGHT_BRACKET
fn parse_length_param(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<usize> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "an opening bracket"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let next = lex.next().ok_or_else(|| end_of_expression(v, "an unsized integer"))?;
    if let Ok(MatcherDefinitionToken::Num(length)) = next {
      let next = lex.next().ok_or_else(|| end_of_expression(v, "')'"))?;
      if let Ok(MatcherDefinitionToken::RightBracket) = next {
        Ok(length)
      } else {
        Err(anyhow!(error_message(lex, v, "Expected a closing bracket", "Expected a closing bracket before this")?))
      }
    } else {
      Err(anyhow!(error_message(lex, v, "Expected an unsigned number", "Expected an unsigned number here")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected an opening bracket", "Expected an opening bracket here")?))
  }
}

// '(' exp=STRING_LITERAL COMMA v=primitiveValue ')'
fn from_provider_state(lex: &mut Lexer<MatcherDefinitionToken>, v: &str) -> anyhow::Result<(String, ValueType, Option<Generator>)> {
  let next = lex.next().ok_or_else(|| end_of_expression(v, "'('"))?;
  if let Ok(MatcherDefinitionToken::LeftBracket) = next {
    let expression = parse_string(lex, v)?;
    parse_comma(lex, v)?;
    let (value, val_type, _) = parse_primitive_value(lex, v, true)?;
    let next = lex.next().ok_or_else(|| end_of_expression(v, "')'"))?;
    if let Ok(MatcherDefinitionToken::RightBracket) = next {
      Ok((value, val_type, Some(ProviderStateGenerator(expression, Some(val_type.into())))))
    } else {
      Err(anyhow!(error_message(lex, v, "Expected a closing bracket", "Expected a closing bracket before this")?))
    }
  } else {
    Err(anyhow!(error_message(lex, v, "Expected an opening bracket", "Expected an opening bracket before this")?))
  }
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use trim_margin::MarginTrimmable;

  use crate::generators::Generator::{Date, DateTime, Time};
  use crate::matchingrules::MatchingRule;
  use crate::matchingrules::MatchingRule::{Regex, Type};

  use super::*;

  macro_rules! as_string {
    ($e:expr) => {{ $e.map_err(|err| err.to_string()) }};
  }

  #[test]
  fn does_not_start_with_matching() {
    expect!(super::parse_matcher_def("")).to(be_err());
    expect!(super::parse_matcher_def("a, b, c")).to(be_err());
    expect!(super::parse_matcher_def("matching some other text")).to(be_err());
  }

  #[test]
  fn parse_type_matcher() {
    let exp = "matching(type,'Name')";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(), ValueType::String, MatchingRule::Type, None, exp.to_string())));
    let exp = "matching( type, 'Name' )";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(), ValueType::String, MatchingRule::Type, None, exp.to_string())));
    let exp = "matching(type,123.4)";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("123.4".to_string(), ValueType::Decimal, MatchingRule::Type, None, exp.to_string())));
    let exp = "matching(type, fromProviderState('exp', 3))";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("3".to_string(), ValueType::Integer, MatchingRule::Type,
        Some(ProviderStateGenerator("exp".to_string(), Some(DataType::INTEGER))), exp.to_string())));
  }

  #[test]
  fn parse_number_matcher() {
    let exp = "matching(number,100)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(), ValueType::Number, MatchingRule::Number, None, exp.to_string())));
    let exp = "matching(number,200.22)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("200.22".to_string(), ValueType::Number, MatchingRule::Number, None, exp.to_string())));
    let exp = "matching(integer,-100)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("-100".to_string(), ValueType::Integer, MatchingRule::Integer, None, exp.to_string())));
    let exp = "matching(decimal,100)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(), ValueType::Decimal, MatchingRule::Decimal, None, exp.to_string())));
    let exp = "matching(decimal,100.22)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100.22".to_string(), ValueType::Decimal, MatchingRule::Decimal, None, exp.to_string())));
    let exp = "matching(number, fromProviderState('exp', 3))";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("3".to_string(), ValueType::Integer, MatchingRule::Number,
        Some(ProviderStateGenerator("exp".to_string(), Some(DataType::INTEGER))), exp.to_string())));
  }

  #[test]
  fn parse_datetime_matcher() {
    let exp = "matching(datetime, 'yyyy-MM-dd','2000-01-01')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("2000-01-01".to_string(),
                   ValueType::String,
                   MatchingRule::Timestamp("yyyy-MM-dd".to_string()),
                   Some(DateTime(Some("yyyy-MM-dd".to_string()), None)), exp.to_string())));
    let exp = "matching(date, 'yyyy-MM-dd','2000-01-01')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("2000-01-01".to_string(),
                   ValueType::String,
                   MatchingRule::Date("yyyy-MM-dd".to_string()),
                   Some(Date(Some("yyyy-MM-dd".to_string()), None)), exp.to_string())));
    let exp = "matching(time, 'HH:mm:ss','12:00:00')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("12:00:00".to_string(),
                   ValueType::String,
                   MatchingRule::Time("HH:mm:ss".to_string()),
                   Some(Time(Some("HH:mm:ss".to_string()), None)), exp.to_string())));
    let exp = "matching(datetime, 'yyyy-MM-dd', fromProviderState('exp', '2000-01-01'))";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("2000-01-01".to_string(),
                   ValueType::String,
                   MatchingRule::Timestamp("yyyy-MM-dd".to_string()),
                   Some(ProviderStateGenerator("exp".to_string(), Some(DataType::STRING))),
                   exp.to_string())));
  }

  #[test]
  fn parse_regex_matcher() {
    let exp = "matching(regex,'\\w+', 'Fred')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Fred".to_string(),
                                              ValueType::String,
                                              MatchingRule::Regex("\\w+".to_string()),
                                              None, exp.to_string())));
    let exp = "matching(regex, '\\d+', fromProviderState('exp', 123))";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("123".to_string(), ValueType::Integer, MatchingRule::Regex("\\d+".to_string()),
        Some(ProviderStateGenerator("exp".to_string(), Some(DataType::INTEGER))), exp.to_string())));
  }

  #[test]
  fn parse_boolean_matcher() {
    let exp = "matching(boolean,true)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("true".to_string(),
                                              ValueType::Boolean,
                                              MatchingRule::Boolean,
                                              None, exp.to_string())));
  }

  #[test]
  fn parse_include_matcher() {
    let exp = "matching(include,'Name')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(),
                                              ValueType::String,
                                              MatchingRule::Include("Name".to_string()),
                                              None, exp.to_string())));
  }

  #[test]
  fn parse_equals_matcher() {
    let exp = "matching(equalTo,'Name')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(),
                                              ValueType::String,
                                              MatchingRule::Equality,
                                              None, exp.to_string())));
    let exp = "matching(equalTo,123.4)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("123.4".to_string(),
                                              ValueType::Decimal,
                                              MatchingRule::Equality,
                                              None, exp.to_string())));
    let exp = "matching(equalTo, fromProviderState('exp', 3))";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("3".to_string(), ValueType::Integer, MatchingRule::Equality,
        Some(ProviderStateGenerator("exp".to_string(), Some(DataType::INTEGER))), exp.to_string())));
  }

  #[test]
  fn parse_content_type_matcher() {
    let exp = "matching(contentType,'Name', 'Value')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Value".to_string(),
                                              ValueType::Unknown,
                                              MatchingRule::ContentType("Name".to_string()),
                                              None, exp.to_string())));
  }

  #[test]
  fn parse_not_empty() {
    let exp = "notEmpty('Value')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Value".to_string(),
                                              ValueType::String,
                                              MatchingRule::NotEmpty,
                                              None, exp.to_string())));
    let exp = "notEmpty(100)";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(),
                                              ValueType::Integer,
                                              MatchingRule::NotEmpty,
                                              None, exp.to_string())));
    let exp = "notEmpty(fromProviderState('exp', 3))";
    expect!(parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("3".to_string(), ValueType::Integer, MatchingRule::NotEmpty,
        Some(ProviderStateGenerator("exp".to_string(), Some(DataType::INTEGER))), exp.to_string())));
  }

  #[test]
  fn parse_comma() {
    expect!(super::parse_comma(&mut MatcherDefinitionToken::lexer(", notEmpty('Value')"), ", notEmpty('Value')")).to(be_ok());

    let mut lex = super::MatcherDefinitionToken::lexer("100 notEmpty(100)");
    lex.next();
    expect!(as_string!(super::parse_comma(&mut lex, "100 notEmpty(100)"))).to(
      be_err().value(
        "|Error: Expected a comma, got 'notEmpty'
            |   ╭─[ expression:1:5 ]
            |   │
            | 1 │ 100 notEmpty(100)
            |   │     ────┬─── \u{0020}
            |   │         ╰───── Expected a comma before this
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));

    let mut lex2 = super::MatcherDefinitionToken::lexer("100");
    lex2.next();
    expect!(as_string!(super::parse_comma(&mut lex2, "100"))).to(
      be_err().value(
        "|Error: Expected a comma, got the end of the expression
            |   ╭─[ expression:1:4 ]
            |   │
            | 1 │ 100
            |   │    │\u{0020}
            |   │    ╰─ Expected a comma here
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));
  }

  #[test]
  fn merging_types() {
    expect!(ValueType::String.merge(ValueType::Unknown)).to(be_equal_to(ValueType::String));
    expect!(ValueType::Unknown.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Unknown.merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Number .merge(ValueType::Unknown)).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Unknown.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Integer.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Unknown.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Unknown.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Boolean.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Unknown.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Unknown));
    expect!(ValueType::String .merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Number .merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Integer.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Decimal.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Boolean.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Number .merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Integer.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Decimal.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Boolean.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Number )).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Integer)).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Decimal)).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Boolean)).to(be_equal_to(ValueType::String));
    expect!(ValueType::Number .merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Number .merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Number .merge(ValueType::Boolean)).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Integer.merge(ValueType::Number )).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Integer.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Integer.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Decimal.merge(ValueType::Number )).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Integer)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Boolean.merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Boolean.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Boolean.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
  }

  #[test]
  fn parse_semver_matcher() {
    let exp = "matching(semver, '1.0.0')";
    expect!(super::parse_matcher_def(exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("1.0.0".to_string(),
                                              ValueType::String,
                                              MatchingRule::Semver,
                                              None, exp.to_string())));

    expect!(as_string!(super::parse_matcher_def("matching(semver, '100')"))).to(
      be_err().value(
        "|Error: Expected a semver compatible string, got '100' - unexpected end of input while parsing major version number
            |   ╭─[ expression:1:18 ]
            |   │
            | 1 │ matching(semver, '100')
            |   │                  ──┬── \u{0020}
            |   │                    ╰──── This is not a valid semver value
            |───╯
            |
            ".trim_margin().unwrap()));

    expect!(as_string!(super::parse_matcher_def("matching(semver, 100)"))).to(
      be_err().value(
        "|Error: Expected a string value, got 100
            |   ╭─[ expression:1:18 ]
            |   │
            | 1 │ matching(semver, 100)
            |   │                  ─┬─ \u{0020}
            |   │                   ╰─── Expected this to be a string
            |   │\u{0020}
            |   │ Note: Surround the value in quotes: matching(semver, '100')
            |───╯
            |
            ".trim_margin().unwrap()
      ));
  }

  #[test]
  fn parse_matching_rule_test() {
    let mut lex = super::MatcherDefinitionToken::lexer("type, '1.0.0')");
    expect!(super::parse_matching_rule(&mut lex, "matching(type, '1.0.0')").unwrap()).to(
      be_equal_to(("1.0.0".to_string(), ValueType::String, Some(Type), None, None)));

    let mut lex = super::MatcherDefinitionToken::lexer("match(");
    lex.next();
    lex.next();
    expect!(as_string!(super::parse_matching_rule(&mut lex, "matching("))).to(
      be_err().value(
        "|Error: Expected a matcher (equalTo, regex, etc.), got the end of the expression
            |   ╭─[ expression:1:10 ]
            |   │
            | 1 │ matching(
            |   │          │\u{0020}
            |   │          ╰─ Expected a matcher (equalTo, regex, etc.) here
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = super::MatcherDefinitionToken::lexer("match(100, '100')");
    lex.next();
    lex.next();
    expect!(as_string!(super::parse_matching_rule(&mut lex, "match(100, '100')"))).to(
      be_err().value(
        "|Error: Expected the type of matcher, got '100'
            |   ╭─[ expression:1:7 ]
            |   │
            | 1 │ match(100, '100')
            |   │       ─┬─ \u{0020}
            |   │        ╰─── Expected a matcher (equalTo, regex, etc.) here
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = super::MatcherDefinitionToken::lexer("match(testABBC, '100')");
    lex.next();
    lex.next();
    expect!(as_string!(super::parse_matching_rule(&mut lex, "match(testABBC, '100')"))).to(
      be_err().value(
        "|Error: Expected the type of matcher, got 'testABBC'
            |   ╭─[ expression:1:7 ]
            |   │
            | 1 │ match(testABBC, '100')
            |   │       ────┬─── \u{0020}
            |   │           ╰───── This is not a valid matcher type
            |   │\u{0020}
            |   │ Note: Valid matchers are: equalTo, regex, type, datetime, date, time, include, number, integer, decimal, boolean, contentType, semver
            |───╯
            |
            ".trim_margin().unwrap()));
  }

  #[test]
  fn parse_matching_rule_with_reference_test() {
    let mut lex = super::MatcherDefinitionToken::lexer("$'bob'");
    expect!(super::parse_matching_rule(&mut lex, "matching($'bob')").unwrap()).to(
      be_equal_to(("bob".to_string(), ValueType::Unknown, None, None, Some(MatchingReference {
        name: "bob".to_string()
      }))));

    let mut lex = super::MatcherDefinitionToken::lexer("match($");
    lex.next();
    lex.next();
    expect!(as_string!(super::parse_matching_rule(&mut lex, "matching($"))).to(
      be_err().value(
        "|Error: Expected a string, got the end of the expression
            |   ╭─[ expression:1:11 ]
            |   │
            | 1 │ matching($
            |   │           │\u{0020}
            |   │           ╰─ Expected a string here
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = super::MatcherDefinitionToken::lexer("match($100)");
    lex.next();
    lex.next();
    expect!(as_string!(super::parse_matching_rule(&mut lex, "match($100)"))).to(
      be_err().value(
        "|Error: Expected a string value, got 100
            |   ╭─[ expression:1:8 ]
            |   │
            | 1 │ match($100)
            |   │        ─┬─ \u{0020}
            |   │         ╰─── Expected this to be a string
            |   │\u{0020}
            |   │ Note: Surround the value in quotes: match($'100')
            |───╯
            |
            ".trim_margin().unwrap()));
  }

  #[test]
  fn matching_definition_exp_test() {
    let exp = "notEmpty('test')";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::matching_definition_exp(&mut lex, exp)).to(
      be_ok().value(MatchingRuleDefinition {
        value: "test".to_string(),
        value_type: ValueType::String,
        rules: vec![ Either::Left(NotEmpty) ],
        generator: None,
        expression: exp.to_string()
      })
    );

    let exp = "matching(regex, '.*', 'aaabbb')";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::matching_definition_exp(&mut lex, exp)).to(
      be_ok().value(MatchingRuleDefinition {
        value: "aaabbb".to_string(),
        value_type: ValueType::String,
        rules: vec![ Either::Left(Regex(".*".to_string())) ],
        generator: None,
        expression: exp.to_string()
      })
    );

    let exp = "matching($'test')";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::matching_definition_exp(&mut lex, exp)).to(
      be_ok().value(MatchingRuleDefinition {
        value: "test".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Right(MatchingReference { name: "test".to_string() }) ],
        generator: None,
        expression: exp.to_string(),
      })
    );

    let exp = "eachKey(matching(regex, '.*', 'aaabbb'))";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::matching_definition_exp(&mut lex, exp)).to(
      be_ok().value(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachKey(MatchingRuleDefinition {
          value: "aaabbb".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Regex(".*".to_string())) ],
          generator: None,
          expression: exp.to_string()
        })) ],
        generator: None,
        expression: exp.to_string()
      })
    );

    let exp = "eachValue(matching(regex, '.*', 'aaabbb'))";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::matching_definition_exp(&mut lex, exp)).to(
      be_ok().value(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachValue(MatchingRuleDefinition {
          value: "aaabbb".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Regex(".*".to_string())) ],
          generator: None,
          expression: exp.to_string()
        })) ],
        generator: None,
        expression: exp.to_string()
      })
    );

    let mut lex = MatcherDefinitionToken::lexer("100");
    lex.next();
    expect!(as_string!(super::matching_definition_exp(&mut lex, "100"))).to(
      be_err().value(
        "|Error: Expected a type of matching rule definition but got the end of the expression
            |   ╭─[ expression:1:4 ]
            |   │
            | 1 │ 100
            |   │    │\u{0020}
            |   │    ╰─ Expected a matching rule definition here
            |   │\u{0020}
            |   │ Note: valid matching rule definitions are: matching, notEmpty, eachKey, eachValue, atLeast, atMost
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("somethingElse('to test')");
    expect!(as_string!(super::matching_definition_exp(&mut lex, "somethingElse('to test')"))).to(
      be_err().value(
        "|Error: Expected a type of matching rule definition, but got 'somethingElse'
            |   ╭─[ expression:1:1 ]
            |   │
            | 1 │ somethingElse('to test')
            |   │ ──────┬────── \u{0020}
            |   │       ╰──────── Expected a matching rule definition here
            |   │\u{0020}
            |   │ Note: valid matching rule definitions are: matching, notEmpty, eachKey, eachValue, atLeast, atMost
            |───╯
            |
            ".trim_margin().unwrap()));
  }

  #[test]
  fn parse_each_key_test() {
    let exp = "(matching($'bob'))";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::parse_each_key(&mut lex, exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachKey(MatchingRuleDefinition {
          value: "bob".to_string(),
          value_type: ValueType::Unknown,
          rules: vec![ Either::Right(MatchingReference { name: "bob".to_string() }) ],
          generator: None,
          expression: exp.to_string()
        }))
        ],
        generator: None,
        expression: exp.to_string()
      }));

    let mut lex = MatcherDefinitionToken::lexer("eachKey");
    lex.next();
    expect!(as_string!(super::parse_each_key(&mut lex, "eachKey"))).to(
      be_err().value(
        "|Error: Expected an opening bracket, got the end of the expression
            |   ╭─[ expression:1:8 ]
            |   │
            | 1 │ eachKey
            |   │        │\u{0020}
            |   │        ╰─ Expected an opening bracket here
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey matching");
    lex.next();
    expect!(as_string!(super::parse_each_key(&mut lex, "eachKey matching"))).to(
      be_err().value(
        "|Error: Expected an opening bracket, got 'matching'
            |   ╭─[ expression:1:9 ]
            |   │
            | 1 │ eachKey matching
            |   │         ────┬─── \u{0020}
            |   │             ╰───── Expected an opening bracket before this
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey(matching(type, 'test') stuff");
    lex.next();
    expect!(as_string!(super::parse_each_key(&mut lex, "eachKey(matching(type, 'test') stuff"))).to(
      be_err().value(
        "|Error: Expected a closing bracket, got 'stuff'
            |   ╭─[ expression:1:32 ]
            |   │
            | 1 │ eachKey(matching(type, 'test') stuff
            |   │                                ──┬── \u{0020}
            |   │                                  ╰──── Expected a closing bracket before this
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey(matching(type, 'test')");
    lex.next();
    expect!(as_string!(super::parse_each_key(&mut lex, "eachKey(matching(type, 'test')"))).to(
      be_err().value(
        "|Error: Expected a closing bracket, got the end of the expression
            |   ╭─[ expression:1:31 ]
            |   │
            | 1 │ eachKey(matching(type, 'test')
            |   │                               │\u{0020}
            |   │                               ╰─ Expected a closing bracket here
            |───╯
            |
            ".trim_margin().unwrap()));
  }

  #[test]
  fn parse_each_value_test() {
    let exp = "(matching($'bob'))";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    expect!(super::parse_each_value(&mut lex, exp).unwrap()).to(
      be_equal_to(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::EachValue(MatchingRuleDefinition {
          value: "bob".to_string(),
          value_type: ValueType::Unknown,
          rules: vec![ Either::Right(MatchingReference { name: "bob".to_string() }) ],
          generator: None,
          expression: exp.to_string()
        }))
        ],
        generator: None,
        expression: exp.to_string()
      }));

    let mut lex = MatcherDefinitionToken::lexer("eachKey");
    lex.next();
    expect!(as_string!(super::parse_each_value(&mut lex, "eachKey"))).to(
      be_err().value(
        "|Error: Expected an opening bracket, got the end of the expression
            |   ╭─[ expression:1:8 ]
            |   │
            | 1 │ eachKey
            |   │        │\u{0020}
            |   │        ╰─ Expected an opening bracket here
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey matching");
    lex.next();
    expect!(as_string!(super::parse_each_value(&mut lex, "eachKey matching"))).to(
      be_err().value(
        "|Error: Expected an opening bracket, got 'matching'
            |   ╭─[ expression:1:9 ]
            |   │
            | 1 │ eachKey matching
            |   │         ────┬─── \u{0020}
            |   │             ╰───── Expected an opening bracket before this
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey(matching(type, 'test') stuff");
    lex.next();
    expect!(as_string!(super::parse_each_value(&mut lex, "eachKey(matching(type, 'test') stuff"))).to(
      be_err().value(
        "|Error: Expected a closing bracket, got 'stuff'
            |   ╭─[ expression:1:32 ]
            |   │
            | 1 │ eachKey(matching(type, 'test') stuff
            |   │                                ──┬── \u{0020}
            |   │                                  ╰──── Expected a closing bracket before this
            |───╯
            |
            ".trim_margin().unwrap()));

    let mut lex = MatcherDefinitionToken::lexer("eachKey(matching(type, 'test')");
    lex.next();
    expect!(as_string!(super::parse_each_value(&mut lex, "eachKey(matching(type, 'test')"))).to(
      be_err().value(
        "|Error: Expected a closing bracket, got the end of the expression
            |   ╭─[ expression:1:31 ]
            |   │
            | 1 │ eachKey(matching(type, 'test')
            |   │                               │\u{0020}
            |   │                               ╰─ Expected a closing bracket here
            |───╯
            |
            ".trim_margin().unwrap()));
  }

  #[test_log::test]
  fn parse_multiple_matcher_definitions() {
    let exp = "eachKey(matching(regex, '\\$(\\.\\w+)+', '$.test.one')), eachValue(matching(type, null))";
    assert_eq!(super::parse_matcher_def(exp).unwrap(),
      MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![
          Either::Left(MatchingRule::EachKey(MatchingRuleDefinition {
            value: "$.test.one".to_string(),
            value_type: ValueType::String,
            rules: vec![Either::Left(MatchingRule::Regex("\\$(\\.\\w+)+".to_string()))],
            generator: None,
            expression: exp.to_string()
          })),
          Either::Left(MatchingRule::EachValue(MatchingRuleDefinition {
            value: "".to_string(),
            value_type: ValueType::String,
            rules: vec![Either::Left(MatchingRule::Type)],
            generator: None,
            expression: exp.to_string()
          }))
        ],
        generator: None,
        expression: exp.to_string()
      });
  }

  #[test_log::test]
  fn merge_definitions() {
    let basic = MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    };
    let with_value = MatchingRuleDefinition {
      value: "value".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    };
    let with_type = MatchingRuleDefinition {
      value: "value".to_string(),
      value_type: ValueType::String,
      rules: vec![],
      generator: None,
      expression: "".to_string()
    };
    let with_generator = MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::String,
      rules: vec![],
      generator: Some(Date(None, None)),
      expression: "".to_string()
    };
    let with_matching_rule = MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::String,
      rules: vec![ Either::Left(Type) ],
      generator: None,
      expression: "".to_string()
    };
    expect!(basic.merge(&basic)).to(be_equal_to(basic.clone()));
    expect!(basic.merge(&with_value)).to(be_equal_to(with_value.clone()));
    expect!(basic.merge(&with_type)).to(be_equal_to(with_type.clone()));
    expect!(basic.merge(&with_generator)).to(be_equal_to(with_generator.clone()));
    expect!(basic.merge(&with_matching_rule)).to(be_equal_to(with_matching_rule.clone()));
    expect!(with_matching_rule.merge(&with_matching_rule)).to(be_equal_to(MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::String,
      rules: vec![ Either::Left(Type), Either::Left(Type) ],
      generator: None,
      expression: "".to_string()
    }));

    let each_key = MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![
        Either::Left(MatchingRule::EachKey(MatchingRuleDefinition {
          value: "$.test.one".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Regex("\\$(\\.\\w+)+".to_string())) ],
          generator: None,
          expression: "".to_string()
        }))
      ],
      generator: None,
      expression: "".to_string()
    };
    let each_value = MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![
        Either::Left(MatchingRule::EachValue(MatchingRuleDefinition {
          value: "".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Type) ],
          generator: None,
          expression: "".to_string()
        }))
      ],
      generator: None,
      expression: "".to_string()
    };
    expect!(each_key.merge(&each_value)).to(be_equal_to(MatchingRuleDefinition {
      value: "".to_string(),
      value_type: ValueType::Unknown,
      rules: vec![
        Either::Left(MatchingRule::EachKey(MatchingRuleDefinition {
          value: "$.test.one".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Regex("\\$(\\.\\w+)+".to_string())) ],
          generator: None,
          expression: "".to_string()
        })),
        Either::Left(MatchingRule::EachValue(MatchingRuleDefinition {
          value: "".to_string(),
          value_type: ValueType::String,
          rules: vec![ Either::Left(Type) ],
          generator: None,
          expression: "".to_string()
        }))
      ],
      generator: None,
      expression: "".to_string()
    }));
  }

  #[rstest]
  //     expression,                                      expected
  #[case("''",                                            "")]
  #[case("'Example value'",                               "Example value")]
  #[case("'yyyy-MM-dd HH:mm:ssZZZZZ'",                    "yyyy-MM-dd HH:mm:ssZZZZZ")]
  #[case("'2020-05-21 16:44:32+10:00'",                   "2020-05-21 16:44:32+10:00")]
  #[case("'\\w{3}\\d+'",                                  "\\w{3}\\d+")]
  #[case("'<?xml?><test/>'",                              "<?xml?><test/>")]
  #[case(r"'\$(\.\w+)+'",                                 r"\$(\.\w+)+")]
  #[case(r"'we don\'t currently support parallelograms'", r"we don\'t currently support parallelograms")]
  #[case(r"'\b backspace'",                               "\x08 backspace")]
  #[case(r"'\f formfeed'",                                "\x0C formfeed")]
  #[case(r"'\n linefeed'",                                "\n linefeed")]
  #[case(r"'\r carriage return'",                         "\r carriage return")]
  #[case(r"'\t tab'",                                     "\t tab")]
  #[case(r"'\u0109 unicode hex code'",                   "\u{0109} unicode hex code")]
  #[case(r"'\u{1DF0B} unicode hex code'",                "\u{1DF0B} unicode hex code")]
  fn parse_string_test(#[case] expression: &str, #[case] expected: &str) {
    let mut lex = MatcherDefinitionToken::lexer(expression);
    expect!(parse_string(&mut lex, expression)).to(be_ok().value(expected.to_string()));
  }

  #[rstest]
  //     expression,                                      expected
  #[case("",                                              "")]
  #[case("Example value",                                 "Example value")]
  #[case(r"not escaped \$(\.\w+)+",                       r"not escaped \$(\.\w+)+")]
  #[case(r"escaped \\",                                   r"escaped \")]
  #[case(r"slash at end \",                               r"slash at end \")]
  fn process_raw_string_test(#[case] expression: &str, #[case] expected: &str) {
    expect!(process_raw_string(expression, 0..(expression.len()), expression)).to(be_ok().value(expected.to_string()));
  }

  #[test]
  fn process_raw_string_error_test() {
    assert_eq!(
      ">Error: Invalid unicode character escape sequence
       >   ╭─[ expression:1:2 ]
       >   │
       > 1 │ 'invalid escape \\u in string'
       >   │  ─────────────┬──────────── \u{0020}
       >   │               ╰────────────── This string contains an invalid escape sequence
       >   │\u{0020}
       >   │ Note: Unicode escape sequences must be in the form \\uXXXX (4 digits) or \\u{X..} (enclosed in braces)
       >───╯
       >".trim_margin_with(">").unwrap(),
      process_raw_string(r"\u", 1..27, r"'invalid escape \u in string'").unwrap_err().to_string());

    expect!(process_raw_string(r"\u0", 0..2, r"\u0")).to(be_err());
    expect!(process_raw_string(r"\u00", 0..3, r"\u00")).to(be_err());
    expect!(process_raw_string(r"\u000", 0..4, r"\u000")).to(be_err());
    expect!(process_raw_string(r"\u{000", 0..4, r"\u{000")).to(be_err());
  }

  #[test]
  fn parse_at_least_test() {
    let exp = "atLeast(1)";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    assert_eq!(super::matching_definition_exp(&mut lex, exp).unwrap(),
      MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::MinType(1)) ],
        generator: None,
        expression: exp.to_string()
      }
    );

    let mut lex = MatcherDefinitionToken::lexer("atLeast");
    let result = super::matching_definition(&mut lex, "atLeast");
    assert_eq!(as_string!(result).unwrap_err(),
        "|Error: Expected an opening bracket, got the end of the expression
        |   ╭─[ expression:1:8 ]
        |   │
        | 1 │ atLeast
        |   │        │\u{0020}
        |   │        ╰─ Expected an opening bracket here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atLeast(-10)");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atLeast(-10)")).unwrap_err(),
        "|Error: Expected an unsigned number, got '-10'
        |   ╭─[ expression:1:9 ]
        |   │
        | 1 │ atLeast(-10)
        |   │         ─┬─ \u{0020}
        |   │          ╰─── Expected an unsigned number here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atLeast('10')");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atLeast('10')")).unwrap_err(),
        "|Error: Expected an unsigned number, got ''10''
        |   ╭─[ expression:1:9 ]
        |   │
        | 1 │ atLeast('10')
        |   │         ──┬─ \u{0020}
        |   │           ╰─── Expected an unsigned number here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atLeast(10");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atLeast(10")).unwrap_err(),
        "|Error: Expected ')', got the end of the expression
        |   ╭─[ expression:1:11 ]
        |   │
        | 1 │ atLeast(10
        |   │           │\u{0020}
        |   │           ╰─ Expected ')' here
        |───╯
        |
        ".trim_margin().unwrap());
  }

  #[test]
  fn parse_at_most_test() {
    let exp = "atMost(100)";
    let mut lex = MatcherDefinitionToken::lexer(exp);
    assert_eq!(super::matching_definition_exp(&mut lex, exp).unwrap(),
      MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(MatchingRule::MaxType(100)) ],
        generator: None,
        expression: exp.to_string()
      }
    );

    let mut lex = MatcherDefinitionToken::lexer("atMost");
    let result = super::matching_definition(&mut lex, "atMost");
    assert_eq!(as_string!(result).unwrap_err(),
        "|Error: Expected an opening bracket, got the end of the expression
        |   ╭─[ expression:1:7 ]
        |   │
        | 1 │ atMost
        |   │       │\u{0020}
        |   │       ╰─ Expected an opening bracket here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atMost(-10)");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atMost(-10)")).unwrap_err(),
        "|Error: Expected an unsigned number, got '-10'
        |   ╭─[ expression:1:8 ]
        |   │
        | 1 │ atMost(-10)
        |   │        ─┬─ \u{0020}
        |   │         ╰─── Expected an unsigned number here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atMost('10')");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atMost('10')")).unwrap_err(),
        "|Error: Expected an unsigned number, got ''10''
        |   ╭─[ expression:1:8 ]
        |   │
        | 1 │ atMost('10')
        |   │        ──┬─ \u{0020}
        |   │          ╰─── Expected an unsigned number here
        |───╯
        |
        ".trim_margin().unwrap());

    let mut lex = MatcherDefinitionToken::lexer("atMost(10");
    assert_eq!(as_string!(super::matching_definition_exp(&mut lex, "atMost(10")).unwrap_err(),
        "|Error: Expected ')', got the end of the expression
        |   ╭─[ expression:1:10 ]
        |   │
        | 1 │ atMost(10
        |   │          │\u{0020}
        |   │          ╰─ Expected ')' here
        |───╯
        |
        ".trim_margin().unwrap());
  }
}
