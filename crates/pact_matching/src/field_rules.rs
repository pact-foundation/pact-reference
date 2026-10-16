//! Application of plugin-provided matching rules to single values.
//!
//! A `MatchingRule::Plugin` names a rule the core framework does not implement. Applying it means
//! resolving the name against the plugin catalogue and calling out to whoever owns it - a plugin
//! over gRPC, or a handler this crate registered for one of the core rules. See proposal 006,
//! [Field-level matchers and generators](https://github.com/pact-foundation/pact-plugins/blob/main/docs/proposals/006_Field_level_matchers_and_generators.md).
//!
//! The plugin call is async and every path that applies a matching rule here is synchronous, so
//! this goes through the driver's blocking wrapper rather than building a bridge per call site.

use std::cell::RefCell;

use bytes::Bytes;
use serde_json::Value;

use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;

/// Where the value currently being matched sits, for the duration of the call.
///
/// [`crate::matchingrules::DoMatch::match_value`] is handed the two values and nothing else, but a
/// plugin rule's request carries the path of the value and which part of the interaction it came
/// from. Rather than change a trait with a dozen implementations and several public callers, the
/// places that do know push a scope for the duration of the match: the part of the interaction is
/// pushed once per part being compared, and the path by `match_values` and the matching engine's
/// interpreter for each value under it.
#[derive(Debug)]
pub(crate) struct FieldMatchScope;

thread_local! {
  static FIELD_MATCH_PATH: RefCell<Vec<DocPath>> = const { RefCell::new(Vec::new()) };
  static FIELD_MATCH_CATEGORY: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Guard for the part of the interaction currently being compared. See [`FieldMatchScope`].
#[derive(Debug)]
pub(crate) struct FieldMatchCategoryScope;

impl FieldMatchScope {
  /// Enters a scope for a value at `path`. The scope ends when the returned guard is dropped.
  #[must_use]
  pub(crate) fn path(path: &DocPath) -> FieldMatchScope {
    FIELD_MATCH_PATH.with(|scope| scope.borrow_mut().push(path.clone()));
    FieldMatchScope
  }

  /// Enters a scope for the part of the interaction being compared: `body`, `header`, `query`,
  /// `metadata`, `path` or `status`. The scope ends when the returned guard is dropped.
  #[must_use]
  pub(crate) fn category(category: &str) -> FieldMatchCategoryScope {
    FIELD_MATCH_CATEGORY.with(|scope| scope.borrow_mut().push(category.to_string()));
    FieldMatchCategoryScope
  }
}

impl Drop for FieldMatchScope {
  fn drop(&mut self) {
    FIELD_MATCH_PATH.with(|scope| { scope.borrow_mut().pop(); });
  }
}

impl Drop for FieldMatchCategoryScope {
  fn drop(&mut self) {
    FIELD_MATCH_CATEGORY.with(|scope| { scope.borrow_mut().pop(); });
  }
}

/// Enters a scope from a matching engine plan path.
///
/// The interpreter walks a tree rather than a document, so it does not carry a `DocPath` - but the
/// plan's container nodes are labelled with one for body values (`engine/bodies/json.rs` and its
/// siblings build them that way) and with the part of the interaction above that. Both are
/// recovered from the labels rather than threaded separately through the interpreter.
#[must_use]
pub(crate) fn scope_from_plan_path(plan_path: &[String]) -> (FieldMatchScope, FieldMatchCategoryScope) {
  let path = plan_path.iter().rev()
    .filter(|label| label.starts_with('$'))
    .find_map(|label| DocPath::new(label).ok())
    .unwrap_or_else(DocPath::root);
  let category = plan_path.iter()
    .find_map(|label| match label.as_str() {
      "body" => Some("body"),
      "headers" => Some("header"),
      "query parameters" => Some("query"),
      "metadata" => Some("metadata"),
      "path" => Some("path"),
      "status" => Some("status"),
      _ => None
    })
    .unwrap_or("body");
  (FieldMatchScope::path(&path), FieldMatchScope::category(category))
}

/// The scope currently in effect, defaulting to the root path in the body when there is none
fn current_scope() -> (DocPath, String) {
  let path = FIELD_MATCH_PATH.with(|scope| scope.borrow().last().cloned())
    .unwrap_or_else(DocPath::root);
  let category = FIELD_MATCH_CATEGORY.with(|scope| scope.borrow().last().cloned())
    .unwrap_or_else(|| "body".to_string());
  (path, category)
}

/// A value that can be handed to a plugin as a single field value.
///
/// The distinction between a whole number and a decimal has to survive this conversion: `integer`,
/// `decimal` and `type` are exactly the rules that would break if it did not, and a plugin cannot
/// recover what the boundary erased. `serde_json::Value` already keeps it, so the JSON-shaped
/// values pass straight through; only binary data needs its own case.
pub(crate) trait ToFieldValue {
  /// This value in the form the plugin interface carries
  fn to_field_value(&self) -> DriverFieldValue;
}

#[cfg(feature = "plugins")]
#[cfg(not(target_family = "wasm"))]
pub(crate) type DriverFieldValue = pact_plugin_driver::field::FieldValue;

/// Stand-in for the driver's field value type when plugin support is not compiled in. Nothing
/// constructs a plugin rule call in that build, but the conversions still have to name a type.
#[cfg(any(not(feature = "plugins"), target_family = "wasm"))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DriverFieldValue {
  /// A JSON-like value
  Json(Value),
  /// Raw bytes
  Binary(Bytes)
}

impl ToFieldValue for Value {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Json(self.clone())
  }
}

impl ToFieldValue for &str {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Json(Value::String(self.to_string()))
  }
}

impl ToFieldValue for String {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Json(Value::String(self.clone()))
  }
}

impl ToFieldValue for u64 {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Json(Value::from(*self))
  }
}

impl ToFieldValue for u16 {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Json(Value::from(*self))
  }
}

impl ToFieldValue for Bytes {
  fn to_field_value(&self) -> DriverFieldValue {
    DriverFieldValue::Binary(self.clone())
  }
}

/// Applies a plugin-provided matching rule to a pair of values, resolving the rule name against
/// the plugin catalogue. Any mismatches the rule reports are joined into the error, since that is
/// the shape [`crate::matchingrules::DoMatch`] works in.
#[cfg(feature = "plugins")]
#[cfg(not(target_family = "wasm"))]
pub(crate) fn apply_plugin_rule<T: ToFieldValue>(
  rule: &MatchingRule,
  name: &str,
  expected: &T,
  actual: &T
) -> anyhow::Result<()> {
  use anyhow::anyhow;
  use itertools::Itertools;
  use pact_plugin_driver::field::{FieldContext, find_field_matcher};
  use tracing::debug;

  let (path, category) = current_scope();
  let matcher = find_field_matcher(name)
    .map_err(|err| anyhow!("Could not apply the '{}' matching rule - {}", name, err))?;
  let context = FieldContext::new(&path, category.as_str());

  debug!(%path, %category, "Applying the '{}' matching rule provided by {}", name, matcher.plugin_name());
  match matcher.match_field_blocking(rule, &expected.to_field_value(), &actual.to_field_value(), &context) {
    Ok(()) => Ok(()),
    Err(mismatches) => Err(anyhow!("{}", mismatches.iter()
      .map(|mismatch| mismatch.mismatch.as_str())
      .join(", ")))
  }
}

#[cfg(any(not(feature = "plugins"), target_family = "wasm"))]
pub(crate) fn apply_plugin_rule<T: ToFieldValue>(
  _rule: &MatchingRule,
  name: &str,
  _expected: &T,
  _actual: &T
) -> anyhow::Result<()> {
  Err(anyhow::anyhow!("'{}' is not a standard matching rule, and this build of pact_matching \
    does not have plugin support enabled, so a plugin-provided rule can not be resolved", name))
}
