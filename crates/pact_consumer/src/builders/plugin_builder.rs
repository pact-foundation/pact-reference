//! Trait for plugin authors to provider a typed interface to configure interactions

use serde_json::Value;

/// Trait for plugin authors to provider a typed interface to configure interactions
pub trait PluginInteractionBuilder {
  /// Construct the map of configuration that is to be passed through to the plugin as a JSON
  /// value.
  fn build(&self) -> Value;
}
