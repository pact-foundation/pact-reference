//! Models to support plugins

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::generators::GeneratorTestMode;
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
  /// Deep merges the data with any existing data. Objects are merged recursively, while any
  /// other value (including arrays) replaces the existing one, so merging the same data more
  /// than once gives the same result.
  pub fn merge(&mut self, data: &HashMap<String, Value>) {
    for (key, value) in data {
      let value = if let Some(v) = self.configuration.get(key) {
        merge_configuration(v, value)
      } else {
        value.clone()
      };
      self.configuration.insert(key.clone(), value);
    }
  }
}

fn merge_configuration(value: &Value, other: &Value) -> Value {
  match (value, other) {
    (Value::Object(entries), Value::Object(other_entries)) => {
      let mut map = entries.clone();
      for (key, other_value) in other_entries {
        let merged = match map.get(key) {
          Some(existing) => merge_configuration(existing, other_value),
          None => other_value.clone()
        };
        map.insert(key.clone(), merged);
      }
      Value::Object(map)
    },
    _ => other.clone()
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

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use super::PluginData;

  #[test]
  fn merge_combines_configuration_with_different_keys() {
    let mut data = PluginData {
      name: "protobuf".to_string(),
      version: "0.5.0".to_string(),
      configuration: hashmap!{ "hash-1".to_string() => json!({ "protoFile": "a" }) }
    };
    data.merge(&hashmap!{ "hash-2".to_string() => json!({ "protoFile": "b" }) });

    expect!(data.configuration).to(be_equal_to(hashmap!{
      "hash-1".to_string() => json!({ "protoFile": "a" }),
      "hash-2".to_string() => json!({ "protoFile": "b" })
    }));
  }

  #[test]
  fn merge_merges_objects_recursively() {
    let mut data = PluginData {
      name: "avro".to_string(),
      version: "0.1.0".to_string(),
      configuration: hashmap!{ "hash-1".to_string() => json!({ "avroSchema": "a", "other": { "x": 1 } }) }
    };
    data.merge(&hashmap!{ "hash-1".to_string() => json!({ "recordName": "Item", "other": { "y": 2 } }) });

    expect!(data.configuration).to(be_equal_to(hashmap!{
      "hash-1".to_string() => json!({ "avroSchema": "a", "recordName": "Item", "other": { "x": 1, "y": 2 } })
    }));
  }

  #[test]
  fn merge_does_not_duplicate_array_values_when_the_same_data_is_merged_again() {
    let config = hashmap!{ "hash-1".to_string() => json!({ "includes": ["a"] }) };
    let mut data = PluginData {
      name: "protobuf".to_string(),
      version: "0.5.0".to_string(),
      configuration: config.clone()
    };
    data.merge(&config);
    data.merge(&config);

    expect!(data.configuration).to(be_equal_to(config));
  }
}
