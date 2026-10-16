//! Support functions for dealing with content from plugins

use std::collections::HashMap;
use std::panic::RefUnwindSafe;

use maplit::hashmap;
use pact_plugin_driver::plugin_models::PluginInteractionConfig;
use serde_json::Map;

use pact_models::interaction::Interaction;
use pact_models::pact::Pact;

/// Which part of the interaction should the config be extracted
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum InteractionPart {
  /// No part, use the whole config
  #[default] None,
  /// Request part under the "request" key
  Request,
  /// Response part under the "response" key
  Response
}

pub(crate) fn setup_plugin_config<'a>(
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  interaction: &Box<dyn Interaction + Send + Sync + RefUnwindSafe>,
  part: InteractionPart
) -> HashMap<String, PluginInteractionConfig> {
  pact.plugin_data().iter().map(|data| {
    let interaction_config = if let Some(v4_interaction) = interaction.as_v4() {
      if let Some(config) = v4_interaction.plugin_config().get(&data.name) {
        // In some cases, depending on how the interaction is setup, the plugin configuration
        // could be stored under a request or response key.
        match part {
          InteractionPart::None => config.clone(),
          InteractionPart::Request => if let Some(request_config) = config.get("request") {
            request_config
              .as_object()
              .cloned()
              .unwrap_or_else(|| Map::new())
              .iter()
              .map(|(k, v)| (k.clone(), v.clone()))
              .collect()
          } else {
            config.clone()
          }
          InteractionPart::Response => if let Some(response_config) = config.get("response") {
            response_config
              .as_object()
              .cloned()
              .unwrap_or_else(|| Map::new())
              .iter()
              .map(|(k, v)| (k.clone(), v.clone()))
              .collect()
          } else {
            config.clone()
          }
        }
      } else {
        hashmap!{}
      }
    } else {
      hashmap!{}
    };
    (data.name.clone(), PluginInteractionConfig {
      pact_configuration: data.configuration.clone(),
      interaction_configuration: interaction_config
    })
  }).collect()
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_plugin_driver::plugin_models::PluginInteractionConfig;
  use serde_json::json;
  use pact_models::interaction::Interaction;
  use pact_models::pact::Pact;
  use pact_models::plugins::PluginData;
  use pact_models::v4::interaction::V4Interaction;
  use pact_models::v4::pact::V4Pact;
  use pact_models::v4::synch_http::SynchronousHttp;

  use crate::plugin_support::{InteractionPart, setup_plugin_config};

  #[test]
  fn setup_plugin_config_extracts_plugin_data_from_the_pact_object_for_the_interaction() {
    let plugin1 = PluginData {
      name: "plugin1".to_string(),
      version: "1".to_string(),
      configuration: hashmap!{
        "a".to_string() => json!(100)
      }
    };
    let plugin2 = PluginData {
      name: "plugin2".to_string(),
      version: "2".to_string(),
      configuration: hashmap!{
        "b".to_string() => json!(200)
      }
    };
    let interaction1 = SynchronousHttp {
      plugin_config: hashmap!{
        "plugin1".to_string() => hashmap!{
          "ia".to_string() => json!(1000)
        }
      },
      .. SynchronousHttp::default()
    };
    let interaction2 = SynchronousHttp {
      plugin_config: hashmap!{
        "plugin2".to_string() => hashmap!{
          "ib".to_string() => json!(2000)
        }
      },
      .. SynchronousHttp::default()
    };
    let pact = V4Pact {
      interactions: vec![interaction1.boxed_v4(), interaction2.boxed_v4()],
      plugin_data: vec![plugin1, plugin2],
      .. V4Pact::default()
    };

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::None);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "ia".to_string() => json!(1000)
        }
      },
      "plugin2".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "b".to_string() => json!(200)
        },
        interaction_configuration: hashmap!{}
      }
    }));

    let result = setup_plugin_config(&pact.boxed(), &interaction2.boxed(), InteractionPart::None);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{}
      },
      "plugin2".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "b".to_string() => json!(200)
        },
        interaction_configuration: hashmap!{
          "ib".to_string() => json!(2000)
        }
      }
    }));

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::Request);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "ia".to_string() => json!(1000)
        }
      },
      "plugin2".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "b".to_string() => json!(200)
        },
        interaction_configuration: hashmap!{}
      }
    }));
  }

  #[test]
  fn setup_plugin_config_extracts_plugin_data_from_the_request_part_for_the_interaction() {
    let plugin1 = PluginData {
      name: "plugin1".to_string(),
      version: "1".to_string(),
      configuration: hashmap!{
        "a".to_string() => json!(100)
      }
    };
    let interaction1 = SynchronousHttp {
      plugin_config: hashmap!{
        "plugin1".to_string() => hashmap!{
          "ia".to_string() => json!(1000),
          "request".to_string() => json!({
            "req": "req_value"
          }),
          "response".to_string() => json!({
            "res": "res_value"
          })
        }
      },
      .. SynchronousHttp::default()
    };
    let pact = V4Pact {
      interactions: vec![interaction1.boxed_v4()],
      plugin_data: vec![plugin1],
      .. V4Pact::default()
    };

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::None);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "ia".to_string() => json!(1000),
          "request".to_string() => json!({"req": "req_value"}),
          "response".to_string() => json!({"res": "res_value"})
        }
      }
    }));

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::Request);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "req".to_string() => json!("req_value")
        }
      }
    }));
  }

  #[test]
  fn setup_plugin_config_extracts_plugin_data_from_the_response_part_for_the_interaction() {
    let plugin1 = PluginData {
      name: "plugin1".to_string(),
      version: "1".to_string(),
      configuration: hashmap!{
        "a".to_string() => json!(100)
      }
    };
    let interaction1 = SynchronousHttp {
      plugin_config: hashmap!{
        "plugin1".to_string() => hashmap!{
          "ia".to_string() => json!(1000),
          "request".to_string() => json!({
            "req": "req_value"
          }),
          "response".to_string() => json!({
            "res": "res_value"
          })
        }
      },
      .. SynchronousHttp::default()
    };
    let pact = V4Pact {
      interactions: vec![interaction1.boxed_v4()],
      plugin_data: vec![plugin1],
      .. V4Pact::default()
    };

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::None);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "ia".to_string() => json!(1000),
          "request".to_string() => json!({"req": "req_value"}),
          "response".to_string() => json!({"res": "res_value"})
        }
      }
    }));

    let result = setup_plugin_config(&pact.boxed(), &interaction1.boxed(), InteractionPart::Response);
    expect!(result).to(be_equal_to(hashmap!{
      "plugin1".to_string() => PluginInteractionConfig {
        pact_configuration: hashmap!{
          "a".to_string() => json!(100)
        },
        interaction_configuration: hashmap!{
          "res".to_string() => json!("res_value")
        }
      }
    }));
  }
}
