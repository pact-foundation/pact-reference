//! Models to support plugins

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::generators::GeneratorTestMode;
use crate::json_utils::json_deep_merge;
use crate::path_exp::DocPath;

/// Plugin configuration persisted in the pact file metadata
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PluginData {
  /// Plugin name
  pub name: String,
  /// Plugin version
  pub version: String,
  /// Any configuration supplied by the plugin
  #[serde(default)]
  pub configuration: HashMap<String, Value>
}

impl PluginData {
  /// Deep merges the data with any existing data
  pub fn merge(&mut self, data: &HashMap<String, Value>) {
    for (key, value) in data {
      let value = if let Some(v) = self.configuration.get(key) {
        json_deep_merge(v, value)
      } else {
        value.clone()
      };
      self.configuration.insert(key.clone(), value);
    }
  }
}

impl PluginData {
  /// Convert this plugin data to a JSON value
  pub fn to_json(&self) -> anyhow::Result<Value> {
    serde_json::to_value(self)
      .map_err(|err| anyhow!("Could not convert plugin data to JSON - {}", err))
  }
}

/// Support for plugin-provided matching rules and generators, supplied by the host framework.
///
/// Resolving a plugin-provided rule or generator means looking its name up in the plugin
/// catalogue, which lives in the plugin driver - and the driver depends on this crate, not the
/// other way around. So the two places in `pact_models` that need to reach the catalogue go
/// through a handler the host registers with [`set_plugin_support`]:
///
/// * applying a [`crate::generators::Generator::Plugin`], since the `GenerateValue`
///   implementations live here;
/// * resolving the `config-key` of a plugin rule while parsing a matching rule definition
///   expression.
///
/// With no handler registered, both fail with an error telling the user to load the plugin.
pub trait PluginSupport: Debug + Send + Sync {
  /// The values key that a single positional configuration argument in
  /// `matching(NAME, CONFIG, EXAMPLE)` maps to. This is the `config-key` value on the rule's
  /// catalogue entry; `None` means the entry does not set one, and the caller uses `value`.
  fn config_key(&self, rule_name: &str) -> Option<String>;

  /// Apply a plugin-provided generator to a single value, returning the generated value.
  ///
  /// `mode` and `path` come from the [`crate::generators::GeneratorScope`] in effect, and are
  /// `None`/the root path when the generator is applied from somewhere that does not establish
  /// one.
  fn generate(
    &self,
    name: &str,
    values: &Value,
    example: &Value,
    mode: Option<GeneratorTestMode>,
    path: &DocPath,
    context: &HashMap<&str, Value>
  ) -> anyhow::Result<Value>;
}

lazy_static! {
  static ref PLUGIN_SUPPORT: RwLock<Option<Arc<dyn PluginSupport>>> = RwLock::new(None);
}

/// Registers the handler for plugin-provided matching rules and generators. Hosts that support
/// plugins call this once during setup; see [`PluginSupport`].
pub fn set_plugin_support(support: Arc<dyn PluginSupport>) {
  let mut guard = PLUGIN_SUPPORT.write().unwrap();
  *guard = Some(support);
}

/// The registered plugin support handler, if the host has set one up
pub fn plugin_support() -> Option<Arc<dyn PluginSupport>> {
  PLUGIN_SUPPORT.read().unwrap().clone()
}

/// The values key a single positional configuration argument for the given plugin rule maps to,
/// defaulting to `value` when there is no handler or the catalogue entry does not set a
/// `config-key`. See [`PluginSupport::config_key`].
pub fn plugin_rule_config_key(rule_name: &str) -> String {
  plugin_support()
    .and_then(|support| support.config_key(rule_name))
    .unwrap_or_else(|| "value".to_string())
}
