//! V4 Pact interaction

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::panic::RefUnwindSafe;
use std::str::FromStr;

use anyhow::anyhow;
use serde_json::{json, Value};
use tracing::warn;

use crate::interaction::Interaction;
use crate::json_utils::json_to_string;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

/// Markup added to an interaction by a plugin
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct InteractionMarkup {
  /// Markup contents
  pub markup: String,
  /// Type of markup (CommonMark or HTML)
  pub markup_type: String
}

impl InteractionMarkup {
  /// Load from a JSON value
  pub fn from_json(json: &Value) -> InteractionMarkup {
    match json {
      Value::Object(values) => InteractionMarkup {
        markup: values.get("markup").map(|val| json_to_string(val)).unwrap_or_default(),
        markup_type: values.get("markupType").map(|val| json_to_string(val)).unwrap_or_default()
      },
      _ => {
        warn!("{:?} is not a valid value for InteractionMarkup", json);
        InteractionMarkup::default()
      }
    }
  }

  /// If this markup is empty
  pub fn is_empty(&self) -> bool {
    self.markup.is_empty()
  }

  /// Convert this markup to JSON form
  pub fn to_json(&self) -> Value {
    json!({
      "markup": self.markup,
      "markupType": self.markup_type
    })
  }

  /// Merges this markup with the other
  pub fn merge(&self, other: InteractionMarkup) -> InteractionMarkup {
    if self.is_empty() {
      other
    } else if other.is_empty() {
      self.clone()
    } else {
      if self.markup_type != other.markup_type {
        warn!("Merging different markup types: {} and {}", self.markup_type, other.markup_type);
      }
      let mut buffer = String::new();
      buffer.push_str(self.markup.as_str());
      buffer.push('\n');
      buffer.push_str(other.markup.as_str());
      InteractionMarkup {
        markup: buffer,
        markup_type: self.markup_type.clone()
      }
    }
  }
}

/// V4 Interaction trait
pub trait V4Interaction: Interaction + Send + Sync {
  /// Convert the interaction to a JSON Value
  fn to_json(&self) -> Value;

  /// Convert the interaction to its super trait
  fn to_super(&self) -> &(dyn Interaction + Send + Sync + RefUnwindSafe);

  /// Convert the interaction to its super trait
  fn to_super_mut(&mut self) -> &mut (dyn Interaction + Send + Sync);

  /// Key for this interaction
  fn key(&self) -> Option<String>;

  /// Clones this interaction and wraps it in a box
  fn boxed_v4(&self) -> Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>;

  /// Annotations and comments associated with this interaction
  fn comments(&self) -> HashMap<String, Value>;

  /// Mutable access to the annotations and comments associated with this interaction
  fn comments_mut(&mut self) -> &mut HashMap<String, Value>;

  /// Type of this V4 interaction
  fn v4_type(&self) -> V4InteractionType;

  /// Any configuration added to the interaction from a plugin
  fn plugin_config(&self) -> HashMap<String, HashMap<String, Value>>;

  /// Any configuration added to the interaction from a plugin
  fn plugin_config_mut(&mut self) -> &mut HashMap<String, HashMap<String, Value>>;

  /// Markup added to the interaction to render in UIs
  fn interaction_markup(&self) -> InteractionMarkup;

  /// Markup added to the interaction to render in UIs
  fn interaction_markup_mut(&mut self) -> &mut InteractionMarkup;

  /// Transport used with the interaction
  fn transport(&self) -> Option<String>;

  /// Set the transport used with the interaction
  fn set_transport(&mut self, transport: Option<String>);

  /// Creates a new version with a calculated key
  fn with_unique_key(&self) -> Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>;

  /// Returns the current key if set, otherwise calculates a new one
  fn unique_key(&self) -> String;
}

impl Display for dyn V4Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(i) = self.as_v4_http() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_sync_message() {
      std::fmt::Display::fmt(&i, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Clone for Box<dyn V4Interaction + Send + Sync + RefUnwindSafe> {
  fn clone(&self) -> Self {
    if let Some(http) = self.as_v4_http() {
      Box::new(http)
    } else if let Some(message) = self.as_v4_async_message() {
      Box::new(message)
    } else if let Some(message) = self.as_v4_sync_message() {
      Box::new(message)
    } else {
      panic!("Internal Error - Tried to clone an interaction that was not valid")
    }
  }
}

impl PartialEq for Box<dyn V4Interaction + Send + Sync + RefUnwindSafe> {
  fn eq(&self, other: &Self) -> bool {
    if let Some(http) = self.as_v4_http() {
      if let Some(other) = other.as_v4_http() {
        http == other
      } else {
        false
      }
    } else if let Some(message) = self.as_v4_async_message() {
      if let Some(other) = other.as_v4_async_message() {
        message == other
      } else {
        false
      }
    } else if let Some(message) = self.as_v4_sync_message() {
      if let Some(other) = other.as_v4_sync_message() {
        message == other
      } else {
        false
      }
    } else {
      false
    }
  }
}

/// Load V4 format interactions from JSON struct
pub fn interactions_from_json(json: &Value, source: &str) -> anyhow::Result<Vec<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>>> {
  match json.get("interactions") {
    Some(Value::Array(array)) => {
      let mut interactions = vec![];
      for (index, ijson) in array.iter().enumerate() {
        interactions.push(interaction_from_json(source, index, ijson)?);
      }
      Ok(interactions)
    },
    _ => Ok(vec![])
  }
}

/// Create an interaction from a JSON struct
pub fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> anyhow::Result<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>> {
  match ijson.get("type") {
    Some(i_type) => match FromStr::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        match i_type {
          V4InteractionType::Synchronous_HTTP => SynchronousHttp::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Asynchronous_Messages => AsynchronousMessage::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Synchronous_Messages => SynchronousMessage::from_json(ijson, index).map(|i| i.boxed_v4())
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        Err(anyhow!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source))
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      Err(anyhow!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source))
    }
  }
}

pub(crate) fn parse_plugin_config(json: &Value) -> HashMap<String, HashMap<String, Value>> {
  if let Some(config) = json.get("pluginConfiguration") {
    match config {
      Value::Object(map) => map.iter()
        .map(|(k, v)| {
          let inner_config = match v {
            Value::Object(o) => o.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => {
              warn!("Plugin {} configuration is not correctly formatted, ignoring it", k);
              Default::default()
            }
          };
          (k.clone(), inner_config)
        }).collect(),
      _ => {
        warn!("Plugin configuration is not correctly formatted, ignoring it");
        Default::default()
      }
    }
  } else {
    Default::default()
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::provider_states::ProviderState;
  use crate::v4::interaction::interaction_from_json;

  #[test]
  fn loading_interaction_from_json() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP",
      "description": "String",
      "providerStates": [{ "name": "provider state" }]
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.description()).to(be_equal_to("String"));
    expect!(interaction.provider_states()).to(be_equal_to(vec![
      ProviderState { name: "provider state".into(), params: hashmap!{} } ]));
  }

  #[test]
  fn defaults_to_number_if_no_description() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP"
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.description()).to(be_equal_to("Interaction 0"));
  }

  #[test]
  fn defaults_to_empty_if_no_provider_state() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP"
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.provider_states().iter()).to(be_empty());
  }

  #[test]
  fn defaults_to_none_if_provider_state_null() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP",
      "description": "String",
      "providerStates": null
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.provider_states().iter()).to(be_empty());
  }

  #[test]
  fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
    let json = json!({
      "type": "Synchronous/HTTP",
      "_id": "123456789",
      "description": "Test Interaction",
      "request": {
        "method": "GET",
        "path": "/"
      },
      "response": {
        "status": 200
      }
    });
    let interaction = interaction_from_json("", 0, &json).unwrap();
    expect!(interaction.id()).to(be_some().value("123456789".to_string()));
  }

  // TODO: implement these tests
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_different_descriptions() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction 2"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_different_provider_states() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Bad state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_the_same_requests_and_responses() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_conflict_if_they_have_different_requests() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     request: Request { method: s!("POST"), .. Request::default() },
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  // }
  //
  // #[test]
  // fn interactions_conflict_if_they_have_different_responses() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     response: Response { status: 400, .. Response::default() },
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  // }
}
