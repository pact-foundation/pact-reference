//! Functions for dealing with matching rule expressions

use std::ffi::CString;
use std::ptr::null;

use either::Either;
use libc::{c_char, c_int, EXIT_FAILURE, EXIT_SUCCESS};
use pact_models::generators::Generator;
use pact_models::matchingrules::expressions::{
  is_matcher_def,
  MatchingRuleDefinition,
  parse_matcher_def,
  ValueType
};
use pact_models::matchingrules::MatchingRule;
use pact_models::time_utils::validate_datetime;
use tracing::{debug, error, trace};

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::error::set_error_msg;
use crate::util::{ptr, string};

/// Result of parsing a matching rule definition
#[derive(Debug, Clone)]
pub struct MatchingRuleDefinitionResult {
  result: Either<String, MatchingRuleDefinition>
}

ffi_fn! {
  /// Parse a matcher definition string into a MatchingRuleDefinition containing the example value,
  /// and matching rules and any generator.
  ///
  /// The following are examples of matching rule definitions:
  /// * `matching(type,'Name')` - type matcher with string value 'Name'
  /// * `matching(number,100)` - number matcher
  /// * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
  ///
  /// See [Matching Rule definition expressions](https://docs.rs/pact_models/latest/pact_models/matchingrules/expressions/index.html).
  ///
  /// The returned value needs to be freed up with the `pactffi_matcher_definition_delete` function.
  ///
  /// # Errors
  /// If the expression is invalid, the MatchingRuleDefinition error will be set. You can check for
  /// this value with the `pactffi_matcher_definition_error` function.
  ///
  /// # Safety
  ///
  /// This function is safe if the expression is a valid NULL terminated string pointer.
  fn pactffi_parse_matcher_definition(expression: *const c_char) -> *const MatchingRuleDefinitionResult {
    let expression = safe_str!(expression);
    let result = if is_matcher_def(expression) {
      match parse_matcher_def(expression) {
        Ok(definition) => {
          debug!("Parsed matcher definition '{}' to '{:?}'", expression, definition);
          MatchingRuleDefinitionResult {
            result: Either::Right(definition)
          }
        }
        Err(err) => {
          error!("Failed to parse matcher definition '{}': {}", expression, err);
          MatchingRuleDefinitionResult {
            result: Either::Left(err.to_string())
          }
        }
      }
    } else if expression.is_empty() {
      MatchingRuleDefinitionResult {
        result: Either::Left("Expected a matching rule definition, but got an empty string".to_string())
      }
    } else {
      MatchingRuleDefinitionResult {
        result: Either::Right(MatchingRuleDefinition {
          value: expression.to_string(),
          value_type: ValueType::String,
          rules: vec![],
          generator: None,
          expression: expression.to_string()
        })
      }
    };

    ptr::raw_to(result) as *const MatchingRuleDefinitionResult
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Returns any error message from parsing a matching definition expression. If there is no error,
  /// it will return a NULL pointer, otherwise returns the error message as a NULL-terminated string.
  /// The returned string must be freed using the `pactffi_string_delete` function once done with it.
  fn pactffi_matcher_definition_error(definition: *const MatchingRuleDefinitionResult) -> *const c_char {
    let definition = as_ref!(definition);
    if let Either::Left(error) = &definition.result {
      string::to_c(&error)? as *const c_char
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Returns the value from parsing a matching definition expression. If there was an error,
  /// it will return a NULL pointer, otherwise returns the value as a NULL-terminated string.
  /// The returned string must be freed using the `pactffi_string_delete` function once done with it.
  ///
  /// Note that different expressions values can have types other than a string. Use
  /// `pactffi_matcher_definition_value_type` to get the actual type of the value. This function
  /// will always return the string representation of the value.
  fn pactffi_matcher_definition_value(definition: *const MatchingRuleDefinitionResult) -> *const c_char {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      string::to_c(&definition.value)? as *const c_char
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Frees the memory used by the result of parsing the matching definition expression
  fn pactffi_matcher_definition_delete(definition: *const MatchingRuleDefinitionResult) {
    ptr::drop_raw(definition as *mut MatchingRuleDefinitionResult);
  }
}

ffi_fn! {
  /// Returns the generator from parsing a matching definition expression. If there was an error or
  /// there is no associated generator, it will return a NULL pointer, otherwise returns the generator
  /// as a pointer.
  ///
  /// The generator pointer will be a valid pointer as long as `pactffi_matcher_definition_delete`
  /// has not been called on the definition. Using the generator pointer after the definition
  /// has been deleted will result in undefined behaviour.
  fn pactffi_matcher_definition_generator(definition: *const MatchingRuleDefinitionResult) -> *const Generator {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      if let Some(generator) = &definition.generator {
        generator as *const Generator
      } else {
        std::ptr::null()
      }
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

/// The type of value detected after parsing the expression
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExpressionValueType {
  /// If the type is unknown
  Unknown,
  /// String type
  String,
  /// Numeric type
  Number,
  /// Integer numeric type (no significant figures after the decimal point)
  Integer,
  /// Decimal numeric type (at least one significant figure after the decimal point)
  Decimal,
  /// Boolean type
  Boolean
}

impl ExpressionValueType {
  /// Convert the ValueType into an ExpressionValueType which can be returned via FFI
  fn from_value_type(t: ValueType) -> ExpressionValueType {
    match t {
      ValueType::Unknown => ExpressionValueType::Unknown,
      ValueType::String => ExpressionValueType::String,
      ValueType::Number => ExpressionValueType::Number,
      ValueType::Integer => ExpressionValueType::Integer,
      ValueType::Decimal => ExpressionValueType::Decimal,
      ValueType::Boolean => ExpressionValueType::Boolean
    }
  }
}

ffi_fn! {
  /// Returns the type of the value from parsing a matching definition expression. If there was an
  /// error parsing the expression, it will return Unknown.
  fn pactffi_matcher_definition_value_type(definition: *const MatchingRuleDefinitionResult) -> ExpressionValueType {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      ExpressionValueType::from_value_type(definition.value_type)
    } else {
      ExpressionValueType::Unknown
    }
  } {
    ExpressionValueType::Unknown
  }
}

/// The matching rule or reference from parsing the matching definition expression.
///
/// For matching rules, the ID corresponds to the following rules:
/// | Rule | ID |
/// | ---- | -- |
/// | Equality | 1 |
/// | Regex | 2 |
/// | Type | 3 |
/// | MinType | 4 |
/// | MaxType | 5 |
/// | MinMaxType | 6 |
/// | Timestamp | 7 |
/// | Time | 8 |
/// | Date | 9 |
/// | Include | 10 |
/// | Number | 11 |
/// | Integer | 12 |
/// | Decimal | 13 |
/// | Null | 14 |
/// | ContentType | 15 |
/// | ArrayContains | 16 |
/// | Values | 17 |
/// | Boolean | 18 |
/// | StatusCode | 19 |
/// | NotEmpty | 20 |
/// | Semver | 21 |
/// | EachKey | 22 |
/// | EachValue | 23 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchingRuleResult {
  /// The matching rule from the expression.
  MatchingRule(u16, *const c_char, MatchingRule),
  /// A reference to a named item.
  MatchingReference(*const c_char)
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_matching_rule_iter_delete(iter: *mut MatchingRuleIterator) {
        ptr::drop_raw(iter);
    }
}

/// Inner type used to store the values for the matching rule iterator
#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchingRuleIteratorInner {
  /// The matching rule from the expression.
  MatchingRule(MatchingRule, Option<CString>, MatchingRuleResult),
  /// A reference to a named item.
  MatchingReference(CString, MatchingRuleResult)
}

/// An iterator over the matching rules from a matching definition expression.
#[derive(Debug)]
pub struct MatchingRuleIterator {
  current: usize,
  rules: Vec<MatchingRuleIteratorInner>
}

impl MatchingRuleIterator {
  /// Create a new iterator over the matching rules from the parsed definition
  pub fn new(definition: &MatchingRuleDefinition) -> Self {
    MatchingRuleIterator {
      current: 0,
      rules: definition.rules.iter().map(|r| {
        match r {
          Either::Left(rule) => {
            let val = match rule {
              MatchingRule::Equality => None,
              MatchingRule::Regex(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::Type => None,
              MatchingRule::MinType(m) => Some(CString::new(m.to_string()).unwrap()),
              MatchingRule::MaxType(m) => Some(CString::new(m.to_string()).unwrap()),
              MatchingRule::MinMaxType(min, max) => {
                let s = format!("{}:{}", min, max);
                Some(CString::new(s).unwrap())
              },
              MatchingRule::Timestamp(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::Time(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::Date(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::Include(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::Number => None,
              MatchingRule::Integer => None,
              MatchingRule::Decimal => None,
              MatchingRule::Null => None,
              MatchingRule::ContentType(s) => Some(CString::new(s.as_str()).unwrap()),
              MatchingRule::ArrayContains(_) => None,
              MatchingRule::Values => None,
              MatchingRule::Boolean => None,
              MatchingRule::StatusCode(_) => None,
              MatchingRule::NotEmpty => None,
              MatchingRule::Semver => None,
              MatchingRule::EachKey(_) => None,
              MatchingRule::EachValue(_) => None,
              // A plugin rule's configuration is a map of arbitrary keys, so unlike every other
              // rule there is no single scalar to expose. The JSON form carries all of it.
              MatchingRule::Plugin { values, .. } => Some(CString::new(values.to_string()).unwrap())
            };
            let rule_value = val.as_ref().map(|v| v.as_ptr()).unwrap_or_else(|| null());
            let rule_result = MatchingRuleResult::MatchingRule(rule_id(rule), rule_value, rule.clone());
            MatchingRuleIteratorInner::MatchingRule(rule.clone(), val, rule_result)
          },
          Either::Right(reference) => {
            let name = CString::new(reference.name.as_str()).unwrap();
            let p = name.as_ptr();
            MatchingRuleIteratorInner::MatchingReference(name, MatchingRuleResult::MatchingReference(p))
          }
        }
      }).collect()
    }
  }

  /// Get the next matching rule or reference.
  fn next(&mut self) -> Option<&MatchingRuleResult> {
    let idx = self.current;
    self.current += 1;
    self.rules.get(idx).map(|r| {
      match r {
        MatchingRuleIteratorInner::MatchingRule(_, _, c_val) => c_val,
        MatchingRuleIteratorInner::MatchingReference(_, c_val) => c_val
      }
    })
  }
}

/// Returns a unique ID for the matching rule
fn rule_id(rule: &MatchingRule) -> u16 {
  match rule {
    MatchingRule::Equality => 1,
    MatchingRule::Regex(_) => 2,
    MatchingRule::Type => 3,
    MatchingRule::MinType(_) => 4,
    MatchingRule::MaxType(_) => 5,
    MatchingRule::MinMaxType(_, _) => 6,
    MatchingRule::Timestamp(_) => 7,
    MatchingRule::Time(_) => 8,
    MatchingRule::Date(_) => 9,
    MatchingRule::Include(_) => 10,
    MatchingRule::Number => 11,
    MatchingRule::Integer => 12,
    MatchingRule::Decimal => 13,
    MatchingRule::Null => 14,
    MatchingRule::ContentType(_) => 15,
    MatchingRule::ArrayContains(_) => 16,
    MatchingRule::Values => 17,
    MatchingRule::Boolean => 18,
    MatchingRule::StatusCode(_) => 19,
    MatchingRule::NotEmpty => 20,
    MatchingRule::Semver => 21,
    MatchingRule::EachKey(_) => 22,
    MatchingRule::EachValue(_) => 23,
    MatchingRule::Plugin { .. } => 24
  }
}

ffi_fn! {
  /// Returns an iterator over the matching rules from the parsed definition. The iterator needs to
  /// be deleted with the `pactffi_matching_rule_iter_delete` function once done with it.
  ///
  /// If there was an error parsing the expression, this function will return a NULL pointer.
  fn pactffi_matcher_definition_iter(definition: *const MatchingRuleDefinitionResult) -> *mut MatchingRuleIterator {
    let definition = as_ref!(definition);
    if let Either::Right(result) = &definition.result {
      let iter = MatchingRuleIterator::new(result);
      ptr::raw_to(iter)
    } else {
      std::ptr::null_mut()
    }
  } {
    std::ptr::null_mut()
  }
}

ffi_fn! {
    /// Get the next matching rule or reference from the iterator. As the values returned are owned
    /// by the iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_matching_rule_iter_next(iter: *mut MatchingRuleIterator) -> *const MatchingRuleResult {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(result) => result as *const MatchingRuleResult,
          None => {
            trace!("iter past the end of matching rules");
            std::ptr::null()
          }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Return the ID of the matching rule.
    ///
    /// The ID corresponds to the following rules:
    /// | Rule | ID |
    /// | ---- | -- |
    /// | Equality | 1 |
    /// | Regex | 2 |
    /// | Type | 3 |
    /// | MinType | 4 |
    /// | MaxType | 5 |
    /// | MinMaxType | 6 |
    /// | Timestamp | 7 |
    /// | Time | 8 |
    /// | Date | 9 |
    /// | Include | 10 |
    /// | Number | 11 |
    /// | Integer | 12 |
    /// | Decimal | 13 |
    /// | Null | 14 |
    /// | ContentType | 15 |
    /// | ArrayContains | 16 |
    /// | Values | 17 |
    /// | Boolean | 18 |
    /// | StatusCode | 19 |
    /// | NotEmpty | 20 |
    /// | Semver | 21 |
    /// | EachKey | 22 |
    /// | EachValue | 23 |
    /// | Plugin | 24 |
    ///
    /// A Plugin rule is one whose name is not a standard matching rule, so it is resolved against
    /// the plugin catalogue when the rule is applied. Its associated value is the JSON object of
    /// its configuration values, since unlike every other rule it has no single scalar value.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the MatchingRuleResult pointer is a valid pointer and the
    /// iterator has not been deleted.
    fn pactffi_matching_rule_id(rule_result: *const MatchingRuleResult) -> u16 {
        let rule_result = as_ref!(rule_result);
        match rule_result {
          MatchingRuleResult::MatchingRule(id, _, _) => *id,
          MatchingRuleResult::MatchingReference(_) => 0
        }
    } {
        0
    }
}

ffi_fn! {
    /// Returns the associated value for the matching rule. If the matching rule does not have an
    /// associated value, will return a NULL pointer.
    ///
    /// The associated values for the rules are:
    /// | Rule | ID | VALUE |
    /// | ---- | -- | ----- |
    /// | Equality | 1 | NULL |
    /// | Regex | 2 | Regex value |
    /// | Type | 3 | NULL |
    /// | MinType | 4 | Minimum value |
    /// | MaxType | 5 | Maximum value |
    /// | MinMaxType | 6 | "min:max" |
    /// | Timestamp | 7 | Format string |
    /// | Time | 8 | Format string |
    /// | Date | 9 | Format string |
    /// | Include | 10 | String value |
    /// | Number | 11 | NULL |
    /// | Integer | 12 | NULL |
    /// | Decimal | 13 | NULL |
    /// | Null | 14 | NULL |
    /// | ContentType | 15 | Content type |
    /// | ArrayContains | 16 | NULL |
    /// | Values | 17 | NULL |
    /// | Boolean | 18 | NULL |
    /// | StatusCode | 19 | NULL |
    /// | NotEmpty | 20 | NULL |
    /// | Semver | 21 | NULL |
    /// | EachKey | 22 | NULL |
    /// | EachValue | 23 | NULL |
    ///
    /// Will return a NULL pointer if the matching rule was a reference or does not have an
    /// associated value.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the MatchingRuleResult pointer is a valid pointer and the
    /// iterator it came from has not been deleted.
    fn pactffi_matching_rule_value(rule_result: *const MatchingRuleResult) -> *const c_char {
        let rule_result = as_ref!(rule_result);
        match rule_result {
          MatchingRuleResult::MatchingRule(_, value, _) => *value,
          MatchingRuleResult::MatchingReference(_) => std::ptr::null()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Returns the matching rule pointer for the matching rule. Will return a NULL pointer if the
    /// matching rule result was a reference.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the MatchingRuleResult pointer is a valid pointer and the
    /// iterator it came from has not been deleted.
    fn pactffi_matching_rule_pointer(rule_result: *const MatchingRuleResult) -> *const MatchingRule {
        let rule_result = as_ref!(rule_result);
        match rule_result {
          MatchingRuleResult::MatchingRule(_, _, rule) => rule as *const MatchingRule,
          MatchingRuleResult::MatchingReference(_) => std::ptr::null()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Return any matching rule reference to a attribute by name. This is when the matcher should
    /// be configured to match the type of a structure. I.e.,
    ///
    /// ```json
    /// {
    ///   "pact:match": "eachValue(matching($'person'))",
    ///   "person": {
    ///     "name": "Fred",
    ///     "age": 100
    ///   }
    /// }
    /// ```
    ///
    /// Will return a NULL pointer if the matching rule was not a reference.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the MatchingRuleResult pointer is a valid pointer and the
    /// iterator has not been deleted.
    fn pactffi_matching_rule_reference_name(rule_result: *const MatchingRuleResult) -> *const c_char {
        let rule_result = as_ref!(rule_result);
        match rule_result {
          MatchingRuleResult::MatchingRule(_, _, _) => std::ptr::null(),
          MatchingRuleResult::MatchingReference(ref_name) => *ref_name
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Validates the date/time value against the date/time format string. If the value is valid,
    /// this function will return a zero status code (EXIT_SUCCESS).
    /// If the value is not valid, will return a value of 1 (EXIT_FAILURE) and set the
    /// error message which can be retrieved with `pactffi_get_error_message`.
    ///
    /// # Errors
    /// If the function receives a panic, it will return 2 and the message associated with the
    /// panic can be retrieved with `pactffi_get_error_message`.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the value and format parameters point to valid
    /// NULL-terminated strings.
    fn pactffi_validate_datetime(value: *const c_char, format: *const c_char) -> c_int {
      let value = safe_str!(value);
      let format = safe_str!(format);

      if value.is_empty() {
        error!("Date/Time value string is empty");
        set_error_msg("Date/Time value string is empty".to_string());
        EXIT_FAILURE
      } else if format.is_empty() {
        error!("Date/Time format string is empty");
        set_error_msg("Date/Time format string is empty".to_string());
        EXIT_FAILURE
      } else {
        match validate_datetime(value, format) {
          Ok(_) => EXIT_SUCCESS,
          Err(err) => {
            error!("Date/Time string '{}' is not valid: {}", value, err);
            set_error_msg(format!("Date/Time string '{}' does not match pattern '{}'", value, format));
            EXIT_FAILURE
          }
        }
      }
    } {
      2
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::{CStr, CString};

  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::matchingrules::MatchingRule;
  use crate::error::pactffi_get_error_message;

  use crate::models::expressions::{
    ExpressionValueType,
    MatchingRuleDefinitionResult,
    MatchingRuleResult,
    pactffi_matcher_definition_error,
    pactffi_matcher_definition_generator,
    pactffi_matcher_definition_iter,
    pactffi_matcher_definition_value,
    pactffi_matcher_definition_value_type,
    pactffi_matching_rule_id,
    pactffi_matching_rule_iter_delete,
    pactffi_matching_rule_iter_next,
    pactffi_matching_rule_reference_name,
    pactffi_matching_rule_value,
    pactffi_parse_matcher_definition,
    pactffi_validate_datetime
  };

  #[test_log::test]
  fn parse_expression_with_null() {
    let result = pactffi_parse_matcher_definition(std::ptr::null());
    expect!(result.is_null()).to(be_true());
  }

  #[test_log::test]
  fn parse_expression_with_empty_string() {
    let empty = CString::new("").unwrap();
    let result = pactffi_parse_matcher_definition(empty.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    let string = unsafe { CString::from_raw(error as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected a matching rule definition, but got an empty string"));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.left()).to(be_some().value("Expected a matching rule definition, but got an empty string"));
  }

  #[test_log::test]
  fn parse_expression_with_invalid_expression() {
    let value = CString::new("matching(type,").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    let string = unsafe { CString::from_raw(error as *mut c_char) };
    expect!(string.to_string_lossy().contains("expected a primitive value")).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_true());

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_true());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::Unknown));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.left().unwrap().contains("expected a primitive value")).to(be_true());
  }

  #[test_log::test]
  fn parse_expression_with_valid_expression() {
    let value = CString::new("matching(type,'Name')").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Name"));

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_true());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let iter = pactffi_matcher_definition_iter(result);
    expect!(iter.is_null()).to(be_false());
    let rule = pactffi_matching_rule_iter_next(iter);
    expect!(rule.is_null()).to(be_false());
    let r = unsafe { rule.as_ref() }.unwrap();
    match r {
      MatchingRuleResult::MatchingRule(id, v, rule) => {
        expect!(*id).to(be_equal_to(3));
        expect!(v.is_null()).to(be_true());
        expect!(rule).to(be_equal_to(&MatchingRule::Type));
      }
      MatchingRuleResult::MatchingReference(_) => {
        panic!("Expected a matching rule");
      }
    }

    let rule_type = pactffi_matching_rule_id(rule);
    expect!(rule_type).to(be_equal_to(3));
    let rule_value = pactffi_matching_rule_value(rule);
    expect!(rule_value.is_null()).to(be_true());

    let ref_name = pactffi_matching_rule_reference_name(rule);
    expect!(ref_name.is_null()).to(be_true());

    let rule = pactffi_matching_rule_iter_next(iter);
    expect!(rule.is_null()).to(be_true());
    pactffi_matching_rule_iter_delete(iter);

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
  }

  #[test_log::test]
  fn parse_expression_with_normal_string() {
    let value = CString::new("I am not an expression").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("I am not an expression"));

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
    expect!(definition.result.as_ref().right().unwrap().rules.is_empty()).to(be_true());
  }

  #[test_log::test]
  fn parse_expression_with_generator() {
    let value = CString::new("matching(date,'yyyy-MM-dd', '2000-01-02')").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("2000-01-02"));

    let iter = pactffi_matcher_definition_iter(result);
    expect!(iter.is_null()).to(be_false());
    let rule = pactffi_matching_rule_iter_next(iter);
    expect!(rule.is_null()).to(be_false());
    let rule_type = pactffi_matching_rule_id(rule);
    expect!(rule_type).to(be_equal_to(9));
    let rule_value = pactffi_matching_rule_value(rule);
    let string =  unsafe { CStr::from_ptr(rule_value) };
    expect!(string.to_string_lossy()).to(be_equal_to("yyyy-MM-dd"));
    pactffi_matching_rule_iter_delete(iter);

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_false());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
  }

  #[test_log::test]
  fn pactffi_validate_datetime_test() {
    let value = CString::new("").unwrap();
    let format = CString::new("yyyy-MM-dd").unwrap();
    expect!(pactffi_validate_datetime(value.as_ptr(), format.as_ptr())).to(be_equal_to(1));
    let mut buffer = Vec::with_capacity(256);
    let pointer = buffer.as_mut_ptr();
    pactffi_get_error_message(pointer, 256);
    let error = unsafe { CStr::from_ptr(pointer) }.to_str().unwrap();
    expect!(error).to(be_equal_to("Date/Time value string is empty"));

    let value = CString::new("2000-02-28").unwrap();
    let format = CString::new("yyyy-MM-dd").unwrap();
    expect!(pactffi_validate_datetime(value.as_ptr(), format.as_ptr())).to(be_equal_to(0));

    let value = CString::new("2000-02-x").unwrap();
    let format = CString::new("yyyy-MM-dd").unwrap();
    expect!(pactffi_validate_datetime(value.as_ptr(), format.as_ptr())).to(be_equal_to(1));
    pactffi_get_error_message(pointer, 256);
    let error = unsafe { CStr::from_ptr(pointer) }.to_str().unwrap();
    expect!(error).to(be_equal_to("Date/Time string '2000-02-x' does not match pattern 'yyyy-MM-dd'"));
  }
}
