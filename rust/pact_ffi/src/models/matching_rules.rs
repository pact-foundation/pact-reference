//! FFI functions to deal with matching rules

use anyhow::Context;
use pact_models::matchingrules::MatchingRule;
use libc::c_char;

use crate::{ffi_fn, as_ref, safe_str};
use crate::util::{ptr, string};

ffi_fn! {
  /// Get the JSON form of the matching rule.
  ///
  /// The returned string must be deleted with `pactffi_string_delete`.
  ///
  /// # Safety
  ///
  /// This function will fail if it is passed a NULL pointer, or the iterator that owns the
  /// value of the matching rule has been deleted.
  fn pactffi_matching_rule_to_json(rule: *const MatchingRule) -> *const c_char {
    let rule = as_ref!(rule);
    let json = rule.to_json().to_string();
    string::to_c(&json)? as *const c_char
  } {
    ptr::null_to::<c_char>()
  }
}

ffi_fn! {
  /// Get a Matching Rule from its JSON representation. 
  ///
  /// Will return a NULL pointer if the matching rule was invalid.
  /// 
  /// # Safety
  ///
  /// This function will fail if it is passed a NULL pointer, or the iterator that owns the
  /// value of the matching rule has been deleted.
  fn pactffi_matching_rule_from_json(rule: *const c_char) -> *const MatchingRule {
    let rule = safe_str!(rule);
    let value: serde_json::Value = serde_json::from_str(rule).context("error parsing matching rule as JSON")?;
    let result = MatchingRule::from_json(&value);

    match result {
      Ok(rule) => ptr::raw_to(rule) as *const MatchingRule,
      _ => ptr::null_to::<MatchingRule>()
    }
  } {
      ptr::null_to::<MatchingRule>()
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::matchingrules::MatchingRule;

  use crate::models::matching_rules::{pactffi_matching_rule_to_json, pactffi_matching_rule_from_json};

  #[test]
  fn matching_rule_to_json() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let json_ptr = pactffi_matching_rule_to_json(rule_ptr);
    let json = unsafe { CString::from_raw(json_ptr as *mut c_char) };
    expect!(json.to_string_lossy()).to(be_equal_to("{\"match\":\"regex\",\"regex\":\"\\\\d+\"}"));
  }

  #[test]
  fn matching_rule_from_json() {
    let json_string = CString::new("{\"match\":\"regex\",\"regex\":\"\\\\d+\"}").unwrap();
    let rule_ptr = pactffi_matching_rule_from_json(json_string.as_ptr());

    unsafe {
      expect!(rule_ptr.as_ref().unwrap().name()).to(be_eq("regex"));
      expect!(rule_ptr.as_ref().unwrap().has_generators()).to(be_eq(false));
    }
  }
}
