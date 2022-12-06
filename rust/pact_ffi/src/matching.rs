//! Module provides FFI functions to match values using Pact matching rules

use bytes::Bytes;
use libc::{c_char, c_uchar};
use pact_models::matchingrules::MatchingRule;
use serde_json::Value;

use pact_matching::matchers::Matches;

use crate::{as_ref, ffi_fn, safe_str};
use crate::util::{ptr, string};

ffi_fn! {
    /// Determines if the string value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get as a NULL terminated string
    /// * actual_value - value to match as a NULL terminated string
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer, and the value parameters must be
    /// valid pointers to a NULL terminated strings.
    fn pactffi_matches_string_value(
      matching_rule: *const MatchingRule,
      expected_value: *const c_char,
      actual_value: *const c_char,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let expected_value = safe_str!(expected_value);
      let actual_value = safe_str!(actual_value);
      let result = expected_value.matches_with(actual_value, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the unsigned integer value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get
    /// * actual_value - value to match
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer.
    fn pactffi_matches_u64_value(
      matching_rule: *const MatchingRule,
      expected_value: u64,
      actual_value: u64,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let result = expected_value.matches_with(actual_value, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the signed integer value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get
    /// * actual_value - value to match
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer.
    fn pactffi_matches_i64_value(
      matching_rule: *const MatchingRule,
      expected_value: i64,
      actual_value: i64,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let result = expected_value.matches_with(actual_value, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the floating point value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get
    /// * actual_value - value to match
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer.
    fn pactffi_matches_f64_value(
      matching_rule: *const MatchingRule,
      expected_value: f64,
      actual_value: f64,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let result = expected_value.matches_with(actual_value, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the boolean value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get, 0 == false and 1 == true
    /// * actual_value - value to match, 0 == false and 1 == true
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer.
    fn pactffi_matches_bool_value(
      matching_rule: *const MatchingRule,
      expected_value: u8,
      actual_value: u8,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let expected_value = expected_value > 0;
      let result = expected_value.matches_with(actual_value > 0, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the binary value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get
    /// * expected_value_len - length of the expected value bytes
    /// * actual_value - value to match
    /// * actual_value_len - length of the actual value bytes
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule, expected value and actual value pointers must be a valid pointers.
    /// expected_value_len and actual_value_len must contain the number of bytes that the value
    /// pointers point to. Passing invalid lengths can lead to undefined behaviour.
    fn pactffi_matches_binary_value(
      matching_rule: *const MatchingRule,
      expected_value: *const c_uchar,
      expected_value_len: usize,
      actual_value: *const c_uchar,
      actual_value_len: usize,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let slice = unsafe { std::slice::from_raw_parts(expected_value, expected_value_len) };
      let expected_value = Bytes::from(slice);
      let slice = unsafe { std::slice::from_raw_parts(actual_value, actual_value_len) };
      let actual_value = Bytes::from(slice);
      let result = expected_value.matches_with(actual_value, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

ffi_fn! {
    /// Determines if the JSON value matches the given matching rule. If the value matches OK,
    /// will return a NULL pointer. If the value does not match, will return a error message as
    /// a NULL terminated string. The error message pointer will need to be deleted with the
    /// `pactffi_string_delete` function once it is no longer required.
    ///
    /// * matching_rule - pointer to a matching rule
    /// * expected_value - value we expect to get as a NULL terminated string
    /// * actual_value - value to match as a NULL terminated string
    /// * cascaded - if the matching rule has been cascaded from a parent. 0 == false, 1 == true
    ///
    /// # Safety
    ///
    /// The matching rule pointer must be a valid pointer, and the value parameters must be
    /// valid pointers to a NULL terminated strings.
    fn pactffi_matches_json_value(
      matching_rule: *const MatchingRule,
      expected_value: *const c_char,
      actual_value: *const c_char,
      cascaded: u8
    ) -> *const c_char {
      let matching_rule = as_ref!(matching_rule);
      let expected_value = safe_str!(expected_value);
      let actual_value = safe_str!(actual_value);

      let expected_json = match serde_json::from_str::<Value>(expected_value) {
        Ok(value) => value,
        Err(err) => {
          let error_message = format!("Failed to parse expected JSON: {}", err);
          return Ok(string::to_c(&error_message)? as *const c_char);
        }
      };
      let actual_json = match serde_json::from_str::<Value>(actual_value) {
        Ok(value) => value,
        Err(err) => {
          let error_message = format!("Failed to parse actual JSON: {}", err);
          return Ok(string::to_c(&error_message)? as *const c_char);
        }
      };

      let result = expected_json.matches_with(actual_json, matching_rule, cascaded > 0);
      match result {
        Ok(_) => ptr::null_to::<c_char>(),
        Err(err) => string::to_c(&err.to_string())? as *const c_char
      }
    } {
      string::to_c("INTERNAL ERROR: function panicked").unwrap_or(ptr::null_mut_to::<c_char>()) as *const c_char
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::{CString};
  use libc::{c_char};

  use expectest::prelude::*;
  use pact_models::matchingrules::MatchingRule;

  use crate::matching::{pactffi_matches_binary_value, pactffi_matches_bool_value, pactffi_matches_f64_value, pactffi_matches_i64_value, pactffi_matches_json_value, pactffi_matches_string_value, pactffi_matches_u64_value};

  #[test_log::test]
  fn pactffi_matches_string_value_test() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = CString::new("1234").unwrap();

    let ok_value = CString::new("123456").unwrap();
    let ok_result = pactffi_matches_string_value(rule_ptr, value.as_ptr(), ok_value.as_ptr(), 0);
    expect!(ok_result.is_null()).to(be_true());

    let err_value = CString::new("123456abcd").unwrap();
    let err_result = pactffi_matches_string_value(rule_ptr, value.as_ptr(), err_value.as_ptr(), 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected '123456abcd' to match '\\d+'"));
  }

  #[test_log::test]
  fn pactffi_matches_u64_value_test() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = 12;

    let ok_result = pactffi_matches_u64_value(rule_ptr, value, 1234, 0);
    expect!(ok_result.is_null()).to(be_true());

    let rule = MatchingRule::Regex("\\s+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let err_result = pactffi_matches_u64_value(rule_ptr, value, 12345, 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected 12345 to match '\\s+'"));
  }

  #[test_log::test]
  fn pactffi_matches_i64_value_test() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = 12;

    let ok_result = pactffi_matches_i64_value(rule_ptr, value, 1234, 0);
    expect!(ok_result.is_null()).to(be_true());

    let err_result = pactffi_matches_i64_value(rule_ptr, value, -1234, 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected -1234 to match '\\d+'"));
  }

  #[test_log::test]
  fn pactffi_matches_f64_value_test() {
    let rule = MatchingRule::Regex("\\d+\\.\\d{2}".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = 1.0;

    let ok_result = pactffi_matches_f64_value(rule_ptr, value, 1234.01, 0);
    expect!(ok_result.is_null()).to(be_true());

    let err_result = pactffi_matches_f64_value(rule_ptr, value, 1234.567, 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected 1234.567 to match '\\d+\\.\\d{2}'"));
  }

  #[test_log::test]
  fn pactffi_matches_bool_value_test() {
    let rule = MatchingRule::Regex("true".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = 1;

    let ok_result = pactffi_matches_bool_value(rule_ptr, value, 1, 0);
    expect!(ok_result.is_null()).to(be_true());

    let err_result = pactffi_matches_bool_value(rule_ptr, value, 0, 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected false to match 'true'"));
  }

  const GIF_1PX: [u8; 35] = [
    0o107, 0o111, 0o106, 0o070, 0o067, 0o141, 0o001, 0o000, 0o001, 0o000, 0o200, 0o000, 0o000, 0o377, 0o377, 0o377,
    0o377, 0o377, 0o377, 0o054, 0o000, 0o000, 0o000, 0o000, 0o001, 0o000, 0o001, 0o000, 0o000, 0o002, 0o002, 0o104,
    0o001, 0o000, 0o073
  ];

  #[test_log::test]
  #[cfg(not(windows))]
  fn pactffi_matches_binary_value_test() {
    let rule = MatchingRule::ContentType("image/gif".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = GIF_1PX.as_ptr();

    let ok_result = pactffi_matches_binary_value(rule_ptr, value, 35, value, 35, 0);
    expect!(ok_result.is_null()).to(be_true());

    let rule = MatchingRule::ContentType("image/png".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let err_result = pactffi_matches_binary_value(rule_ptr, value, 35, value, 35, 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected binary contents to have content type 'image/png' but detected contents was 'image/gif'"));
  }

  #[test_log::test]
  fn pactffi_matches_json_value_test() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let value = CString::new("1234").unwrap();

    let ok_value = CString::new("123456").unwrap();
    let ok_result = pactffi_matches_json_value(rule_ptr, value.as_ptr(), ok_value.as_ptr(), 0);
    expect!(ok_result.is_null()).to(be_true());

    let err_value = CString::new("\"123456abcd\"").unwrap();
    let err_result = pactffi_matches_json_value(rule_ptr, value.as_ptr(), err_value.as_ptr(), 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected '123456abcd' to match '\\d+'"));

    let invalid_json = CString::new("\"123456abcd").unwrap();
    let err_result = pactffi_matches_json_value(rule_ptr, value.as_ptr(), invalid_json.as_ptr(), 0);
    let string = unsafe { CString::from_raw(err_result as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Failed to parse actual JSON: EOF while parsing a string at line 1 column 11"));
  }
}
