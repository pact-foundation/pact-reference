//! Synchronous interactions as a request message to a sequence of response messages

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use itertools::Itertools;
use serde_json::{json, Map, Value};
use tracing::warn;

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::interaction::Interaction;
use crate::json_utils::{is_empty, json_to_string};
use crate::matchingrules::MatchingRules;
use crate::message::Message;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::interaction::{InteractionMarkup, parse_plugin_config, V4Interaction};
use crate::v4::message_parts::MessageContents;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

/// Synchronous interactions as a request message to a sequence of response messages
#[derive(Debug, Clone, Eq)]
pub struct SynchronousMessage {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider state for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,
  /// Request message
  pub request: MessageContents,
  /// Response messages
  pub response: Vec<MessageContents>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool,

  /// Configuration added by plugins
  pub plugin_config: HashMap<String, HashMap<String, Value>>,

  /// Text markup to use to render the interaction in a UI
  pub interaction_markup: InteractionMarkup,

  /// Transport mechanism used with this message
  pub transport: Option<String>
}

impl SynchronousMessage {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> SynchronousMessage {
    SynchronousMessage {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Parse the JSON into a SynchronousMessages structure
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<SynchronousMessage> {
    if json.is_object() {
      let id = json.get("_id").map(|id| json_to_string(id));
      let key = json.get("key").map(|id| json_to_string(id));
      let description = match json.get("description") {
        Some(v) => match *v {
          Value::String(ref s) => s.clone(),
          _ => v.to_string()
        },
        None => format!("Interaction {}", index)
      };

      let comments = match json.get("comments") {
        Some(v) => match v {
          Value::Object(map) => map.iter()
            .map(|(k, v)| (k.clone(), v.clone())).collect(),
          _ => {
            warn!("Interaction comments must be a JSON Object, but received {}. Ignoring", v);
            Default::default()
          }
        },
        None => Default::default()
      };

      let provider_states = ProviderState::from_json(json);
      let request = json.get("request")
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'request' object"))?;
      let response = json.get("response")
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'response' array"))?
        .as_array()
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'response' array"))?;
      let responses =
        response.iter()
          .map(|message| MessageContents::from_json(message))
          .collect::<Vec<anyhow::Result<MessageContents>>>();

      let plugin_config = parse_plugin_config(json);
      let interaction_markup = json.get("interactionMarkup")
        .map(|markup| InteractionMarkup::from_json(markup)).unwrap_or_default();

      let transport = json.get("transport").map(|value| {
        match value {
          Value::String(s) => s.clone(),
          _ => value.to_string()
        }
      });

      if responses.iter().any(|res| res.is_err()) {
        let errors = responses.iter()
          .filter(|res| res.is_err())
          .map(|res| res.as_ref().unwrap_err().to_string())
          .join(", ");
        Err(anyhow!("Failed to parse SynchronousMessages responses - {}", errors))
      } else {
        Ok(SynchronousMessage {
          id,
          key,
          description,
          provider_states,
          comments,
          request: MessageContents::from_json(request)?,
          response: responses.iter().map(|res| res.as_ref().unwrap().clone()).collect(),
          pending: json.get("pending")
            .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default(),
          plugin_config,
          interaction_markup,
          transport
        })
      }
    } else {
      Err(anyhow!("Expected a JSON object for the interaction, got '{}'", json))
    }
  }
}

impl V4Interaction for SynchronousMessage {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Synchronous_Messages.to_string(),
      "description": self.description.clone(),
      "pending": self.pending,
      "request": self.request.to_json(),
      "response": self.response.iter().map(|m| m.to_json()).collect_vec()
    });
    let map = json.as_object_mut().unwrap();

    if let Some(key) = &self.key {
      map.insert("key".to_string(), Value::String(key.clone()));
    }

    if !self.provider_states.is_empty() {
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    let comments: Map<String, Value> = self.comments.iter()
      .filter(|(_k, v)| !is_empty(v))
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    if !comments.is_empty() {
      map.insert("comments".to_string(), Value::Object(comments));
    }

    if !self.plugin_config.is_empty() {
      map.insert("pluginConfiguration".to_string(), self.plugin_config.iter()
        .map(|(k, v)|
          (k.clone(), Value::Object(v.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
        ).collect());
    }

    if !self.interaction_markup.is_empty() {
      map.insert("interactionMarkup".to_string(), self.interaction_markup.to_json());
    }

    if let Some(transport) = &self.transport {
      map.insert("transport".to_string(), Value::String(transport.clone()));
    }

    json
  }

  fn to_super(&self) -> &(dyn Interaction + Send + Sync + RefUnwindSafe) {
    self
  }

  fn to_super_mut(&mut self) -> &mut (dyn Interaction + Send + Sync) {
    self
  }

  fn key(&self) -> Option<String> {
    self.key.clone()
  }

  fn boxed_v4(&self) -> Box<dyn V4Interaction + Send + Sync + RefUnwindSafe> {
    Box::new(self.clone())
  }

  fn comments(&self) -> HashMap<String, Value> {
    self.comments.clone()
  }

  fn comments_mut(&mut self) -> &mut HashMap<String, Value> {
    &mut self.comments
  }

  fn v4_type(&self) -> V4InteractionType {
    V4InteractionType::Synchronous_Messages
  }

  fn plugin_config(&self) -> HashMap<String, HashMap<String, Value>> {
    self.plugin_config.clone()
  }

  fn plugin_config_mut(&mut self) -> &mut HashMap<String, HashMap<String, Value>> {
    &mut self.plugin_config
  }

  fn interaction_markup(&self) -> InteractionMarkup {
    self.interaction_markup.clone()
  }

  fn interaction_markup_mut(&mut self) -> &mut InteractionMarkup {
    &mut self.interaction_markup
  }

  fn transport(&self) -> Option<String> {
    self.transport.clone()
  }

  fn set_transport(&mut self, transport: Option<String>) {
    self.transport = transport.clone();
  }

  fn with_unique_key(&self) -> Box<dyn V4Interaction + Send + Sync + RefUnwindSafe> {
    Box::new(self.with_key())
  }

  fn unique_key(&self) -> String {
    match &self.key {
      None => self.calc_hash(),
      Some(key) => key.clone()
    }
  }
}

impl Interaction for SynchronousMessage {
  fn type_of(&self) -> String {
    format!("V4 {}", self.v4_type())
  }

  fn is_request_response(&self) -> bool {
    false
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    None
  }

  fn is_message(&self) -> bool {
    false
  }

  fn as_message(&self) -> Option<Message> {
    None
  }

  fn id(&self) -> Option<String> {
    self.id.clone()
  }

  fn description(&self) -> String {
    self.description.clone()
  }

  fn set_id(&mut self, id: Option<String>) {
    self.id = id;
  }

  fn set_description(&mut self, description: &str) {
    self.description = description.to_string();
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    self.provider_states.clone()
  }

  fn provider_states_mut(&mut self) -> &mut Vec<ProviderState> {
    &mut self.provider_states
  }

  fn contents(&self) -> OptionalBody {
    OptionalBody::Missing
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.response.first().map(|message| message.contents.clone()).unwrap_or_default()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.request.message_content_type()
  }

  fn is_v4(&self) -> bool {
    true
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>> {
    Some(self.boxed_v4())
  }

  fn as_v4_mut(&mut self) -> Option<&mut dyn V4Interaction> {
    Some(self)
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    None
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    None
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    Some(self.clone())
  }

  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp> {
    None
  }

  fn is_v4_sync_message(&self) -> bool {
    true
  }

  fn as_v4_async_message_mut(&mut self) -> Option<&mut AsynchronousMessage> {
    None
  }

  fn as_v4_sync_message_mut(&mut self) -> Option<&mut SynchronousMessage> {
    Some(self)
  }

  fn boxed(&self) -> Box<dyn Interaction + Send + Sync + RefUnwindSafe> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Interaction + Send + Sync + RefUnwindSafe> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync + RefUnwindSafe>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn matching_rules(&self) -> Option<MatchingRules> {
    None
  }

  fn pending(&self) -> bool {
    self.pending
  }
}

impl Default for SynchronousMessage {
  fn default() -> Self {
    SynchronousMessage {
      id: None,
      key: None,
      description: "Synchronous/Message Interaction".to_string(),
      provider_states: vec![],
      comments: Default::default(),
      request: Default::default(),
      response: Default::default(),
      pending: false,
      plugin_config: Default::default(),
      interaction_markup: Default::default(),
      transport: None
    }
  }
}

impl PartialEq for SynchronousMessage {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key &&
      self.description == other.description &&
      self.provider_states == other.provider_states &&
      self.request == other.request &&
      self.response == other.response &&
      self.pending == other.pending
  }
}

impl Hash for SynchronousMessage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.request.hash(state);
    self.response.hash(state);
    self.pending.hash(state);
  }
}

impl Display for SynchronousMessage {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Synchronous Message Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {:?} )",
           pending, self.id, self.description, self.provider_states, self.request, self.response)
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pretty_assertions::{assert_eq, assert_ne};
  use serde_json::{json, Value};

  use crate::bodies::OptionalBody;
  use crate::prelude::ProviderState;
  use crate::v4::interaction::V4Interaction;
  use crate::v4::message_parts::MessageContents;
  use crate::v4::sync_message::SynchronousMessage;

  #[test]
  fn calculate_hash_test() {
    let interaction = SynchronousMessage::from_json(&json!({
      "description": "a retrieve Mallory request",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "request": {
        "contents": {
          "content": "Mallory",
          "contentType": "text/plain",
          "encoded": false
        },
        "metadata": {
          "Content-Type": [
            "application/json"
          ]
        }
      },
      "response": [
        {
          "contents": {
            "content": "That is some good Mallory.",
            "contentType": "text/plain",
            "encoded": false
          },
          "metadata": {
            "Content-Type": [
              "text/plain"
            ]
          }
        }
      ],
      "type": "Synchronous/Messages"
    }), 0).unwrap();
    let hash = interaction.calc_hash();
    expect!(interaction.calc_hash()).to(be_equal_to(hash.as_str()));

    let interaction2 = interaction.with_key();
    expect!(interaction2.key.as_ref().unwrap()).to(be_equal_to(hash.as_str()));

    let json = interaction2.to_json();
    assert_eq!(json!({
      "description": "a retrieve Mallory request",
      "key": "93f58446f133592f",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "request": {
        "contents": {
            "content": "Mallory",
            "contentType": "text/plain",
            "encoded": false
         },
        "metadata": {
             "Content-Type": [
                "application/json"
             ]
        }
      },
      "response": [{
        "contents": {
          "content": "That is some good Mallory.",
          "contentType": "text/plain",
          "encoded": false
        },
        "metadata": {
          "Content-Type": [
            "text/plain"
          ]
        }
      }],
      "type": "Synchronous/Messages"
    }), json);
  }

  #[test]
  fn hash_test() {
    let i1 = SynchronousMessage::default();
    expect!(i1.calc_hash()).to(be_equal_to("2c18fa761d06be45"));

    let i2 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      .. SynchronousMessage::default()
    };
    expect!(i2.calc_hash()).to(be_equal_to("66fbdb308329891b"));

    let i3 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. SynchronousMessage::default()
    };
    expect!(i3.calc_hash()).to(be_equal_to("831a3fa6d0a7ea0c"));

    let i4 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. SynchronousMessage::default()
    };
    expect!(i4.calc_hash()).to(be_equal_to("25420754ce64549d"));

    let i5 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string())  },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. SynchronousMessage::default()
    };
    expect!(i5.calc_hash()).to(be_equal_to("aefc777fdfa238b0"));

    let i6 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      response: vec![MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("text/plain".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      }],
      .. SynchronousMessage::default()
    };
    expect!(i6.calc_hash()).to(be_equal_to("9338c66e694d3d80"));
  }

  #[test]
  fn equals_test() {
    let i1 = SynchronousMessage::default();
    let i2 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      .. SynchronousMessage::default()
    };
    let i3 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. SynchronousMessage::default()
    };
    let i4 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. SynchronousMessage::default()
    };
    let i5 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string())  },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. SynchronousMessage::default()
    };
    let i6 = SynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      response: vec![MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("text/plain".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      }],
      .. SynchronousMessage::default()
    };

    assert_eq!(i1, i1);
    assert_eq!(i2, i2);
    assert_eq!(i3, i3);
    assert_eq!(i4, i4);
    assert_eq!(i5, i5);
    assert_eq!(i6, i6);

    assert_ne!(i1, i2);
    assert_ne!(i1, i3);
    assert_ne!(i1, i4);
    assert_ne!(i1, i5);
    assert_ne!(i1, i6);
    assert_ne!(i2, i1);
    assert_ne!(i2, i3);
    assert_ne!(i2, i4);
    assert_ne!(i2, i5);
    assert_ne!(i2, i6);
  }

  #[test]
  fn equals_test_with_different_keys() {
    let i1 = SynchronousMessage {
      key: Some("i1".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      response: vec![MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("text/plain".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      }],
      .. SynchronousMessage::default()
    };
    let i2 = SynchronousMessage {
      key: Some("i2".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("application/json".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      response: vec![MessageContents {
        metadata: hashmap!{ "Content-Type".to_string() => Value::String("text/plain".to_string()) },
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      }],
      .. SynchronousMessage::default()
    };

    assert_eq!(i1, i1);
    assert_eq!(i2, i2);

    assert_ne!(i1, i2);
    assert_ne!(i2, i1);
  }
}
