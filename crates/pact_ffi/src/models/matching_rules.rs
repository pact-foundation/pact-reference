//! FFI functions to deal with matching rules

use itertools::Itertools;
use libc::c_char;
use pact_models::matchingrules::{Category, MatchingRule};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;
use tracing::trace;

use crate::{as_mut, as_ref, ffi_fn};
use crate::util::{ptr, string};
use crate::util::ptr::{drop_raw, raw_to};

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
    std::ptr::null()
  }
}

/// Enum defining the categories that matching rules can be applied to
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MatchingRuleCategory {
  /// Request Method
  METHOD,
  /// Request Path
  PATH,
  /// Request/Response Header
  HEADER,
  /// Request Query Parameter
  QUERY,
  /// Body
  BODY,
  /// Response Status
  STATUS,
  /// Message contents (body)
  CONTENTS,
  /// Message metadata
  METADATA
}

impl From<Category> for MatchingRuleCategory {
  #[inline]
  fn from(category: Category) -> MatchingRuleCategory {
    match category {
      Category::METHOD => MatchingRuleCategory::METHOD,
      Category::PATH => MatchingRuleCategory::PATH,
      Category::HEADER => MatchingRuleCategory::HEADER,
      Category::QUERY => MatchingRuleCategory::QUERY,
      Category::BODY => MatchingRuleCategory::BODY,
      Category::STATUS => MatchingRuleCategory::STATUS,
      Category::CONTENTS => MatchingRuleCategory::CONTENTS,
      Category::METADATA => MatchingRuleCategory::METADATA
    }
  }
}

impl From<MatchingRuleCategory> for Category {
  #[inline]
  fn from(category: MatchingRuleCategory) -> Category {
    match category {
      MatchingRuleCategory::METHOD => Category::METHOD,
      MatchingRuleCategory::PATH => Category::PATH,
      MatchingRuleCategory::HEADER => Category::HEADER,
      MatchingRuleCategory::QUERY => Category::QUERY,
      MatchingRuleCategory::BODY => Category::BODY,
      MatchingRuleCategory::STATUS => Category::STATUS,
      MatchingRuleCategory::CONTENTS => Category::CONTENTS,
      MatchingRuleCategory::METADATA => Category::METADATA
    }
  }
}

/// An iterator that enables FFI iteration over the matching rules for a particular matching rule
/// category.
#[derive(Debug)]
pub struct MatchingRuleCategoryIterator {
  rules: Vec<(DocPath, MatchingRule)>,
  current_idx: usize
}

impl MatchingRuleCategoryIterator {
  /// Creates a new iterator over a map of matching rules
  fn new(rules: pact_models::matchingrules::MatchingRuleCategory) -> MatchingRuleCategoryIterator {
    let rules = rules.rules.iter()
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a.to_string().as_str(), b.to_string().as_str()))
      .flat_map(|(k, v)| v.rules.iter().map(|r| (k.clone(), r.clone())));
    MatchingRuleCategoryIterator {
      rules: rules.collect(),
      current_idx: 0
    }
  }

  /// Create a new iterator for the matching rules from a message contents
  pub fn new_from_contents(contents: &MessageContents, category: MatchingRuleCategory) -> Self {
    let category: Category = category.into();
    MatchingRuleCategoryIterator::new(contents.matching_rules.rules_for_category(category).unwrap_or_default())
  }

  /// Create a new iterator for the matching rules from a request
  pub fn new_from_request(request: &HttpRequest, category: MatchingRuleCategory) -> Self {
    let category: Category = category.into();
    MatchingRuleCategoryIterator::new(request.matching_rules.rules_for_category(category).unwrap_or_default())
  }

  /// Create a new iterator for the matching rules from a response
  pub fn new_from_response(response: &HttpResponse, category: MatchingRuleCategory) -> Self {
    let category: Category = category.into();
    MatchingRuleCategoryIterator::new(response.matching_rules.rules_for_category(category).unwrap_or_default())
  }

  fn next(&mut self) -> Option<&(DocPath, MatchingRule)> {
    let value = self.rules.get(self.current_idx);
    self.current_idx += 1;
    value
  }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_matching_rules_iter_delete(iter: *mut MatchingRuleCategoryIterator) {
        ptr::drop_raw(iter);
    }
}

/// A single key-value pair of a path and matching rule exported to the C-side.
#[derive(Debug)]
#[repr(C)]
pub struct MatchingRuleKeyValuePair {
  /// The matching rule path
  pub path: *const c_char,
  /// The matching rule
  pub rule: *const MatchingRule,
}

impl MatchingRuleKeyValuePair {
  fn new(
    key: &str,
    value: &MatchingRule
  ) -> anyhow::Result<MatchingRuleKeyValuePair> {
    Ok(MatchingRuleKeyValuePair {
      path: string::to_c(key)? as *const c_char,
      rule: raw_to(value.clone()) as *const MatchingRule
    })
  }
}

// Ensure that the owned values are freed when the pair is dropped.
impl Drop for MatchingRuleKeyValuePair {
  fn drop(&mut self) {
    string::pactffi_string_delete(self.path as *mut c_char);
    drop_raw(self.rule as *mut MatchingRule);
  }
}

ffi_fn! {
    /// Get the next path and matching rule out of the iterator, if possible.
    ///
    /// The returned pointer must be deleted with `pactffi_matching_rules_iter_pair_delete`.
    ///
    /// # Safety
    ///
    /// The underlying data is owned by the `MatchingRuleKeyValuePair`, so is always safe to use.
    ///
    /// # Error Handling
    ///
    /// If no further data is present, returns NULL.
    fn pactffi_matching_rules_iter_next(iter: *mut MatchingRuleCategoryIterator) -> *const MatchingRuleKeyValuePair {
        let iter = as_mut!(iter);

        match iter.next() {
          Some((path, rule)) => {
            let pair = MatchingRuleKeyValuePair::new(&path.to_string(), rule)?;
            ptr::raw_to(pair)
          }
          None => {
            trace!("iter past the end of the matching rules");
            std::ptr::null_mut()
          }
      }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free a pair of key and value returned from `message_metadata_iter_next`.
    fn pactffi_matching_rules_iter_pair_delete(pair: *const MatchingRuleKeyValuePair) {
        ptr::drop_raw(pair as *mut MatchingRuleKeyValuePair);
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::matchingrules::MatchingRule;

  use crate::models::matching_rules::pactffi_matching_rule_to_json;

  #[test]
  fn matching_rule_json() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let json_ptr = pactffi_matching_rule_to_json(rule_ptr);
    let json = unsafe { CString::from_raw(json_ptr as *mut c_char) };
    expect!(json.to_string_lossy()).to(be_equal_to("{\"match\":\"regex\",\"regex\":\"\\\\d+\"}"));
  }
}
