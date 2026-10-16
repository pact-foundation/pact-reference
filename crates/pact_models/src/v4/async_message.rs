//! Models for asynchronous message interactions

use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use maplit::hashmap;
use serde_json::{json, Map, Value};
use tracing::warn;

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::generators::{Generators, generators_from_json, generators_to_json};
use crate::http_parts::HttpPart;
use crate::interaction::Interaction;
use crate::json_utils::{hash_json, is_empty, json_to_string};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::message::Message;
use crate::PactSpecification;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::http_parts::body_from_json;
use crate::v4::interaction::{InteractionMarkup, parse_plugin_config, V4Interaction};
use crate::v4::message_parts::{MessageContents, metadata_to_headers};
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

/// Asynchronous interactions as a sequence of messages
#[derive(Debug, Clone, Eq)]
pub struct AsynchronousMessage {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider state for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// The contents of the message
  pub contents: MessageContents,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool,

  /// Configuration added by plugins
  pub plugin_config: HashMap<String, HashMap<String, Value>>,

  /// Text markup to use to render the interaction in a UI
  pub interaction_markup: InteractionMarkup,

  /// Transport mechanism used with this message
  pub transport: Option<String>
}

impl AsynchronousMessage {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> AsynchronousMessage {
    AsynchronousMessage {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Returns the content type of the message by returning the content type associated with
  /// the body, or by looking it up in the message metadata
  pub fn message_content_type(&self) -> Option<ContentType> {
    self.contents.message_content_type()
  }

  /// Parse the JSON into an AsynchronousMessage interaction
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<AsynchronousMessage> {
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
      let metadata = match json.get("metadata") {
        Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
          (k.clone(), v.clone())
        }).collect(),
        _ => hashmap! {}
      };
      let as_headers = metadata_to_headers(&metadata);

      let plugin_config = parse_plugin_config(json);
      let interaction_markup = json.get("interactionMarkup")
        .map(|markup| InteractionMarkup::from_json(markup)).unwrap_or_default();

      let transport = json.get("transport").map(|value| {
        match value {
          Value::String(s) => s.clone(),
          _ => value.to_string()
        }
      });

      Ok(AsynchronousMessage {
        id,
        key,
        description,
        provider_states,
        contents: MessageContents {
          metadata,
          contents: body_from_json(json, "contents", &as_headers),
          matching_rules: matchers_from_json(json, &None)?,
          generators: generators_from_json(json)?,
        },
        comments,
        pending: json.get("pending")
          .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default(),
        plugin_config,
        interaction_markup,
        transport
      })
    } else {
      Err(anyhow!("Expected a JSON object for the interaction, got '{}'", json))
    }
  }
}

impl V4Interaction for AsynchronousMessage {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Asynchronous_Messages.to_string(),
      "description": self.description.clone(),
      "pending": self.pending
    });
    let map = json.as_object_mut().unwrap();

    if let Some(key) = &self.key {
      map.insert("key".to_string(), Value::String(key.clone()));
    }

    if let Value::Object(body) = self.contents.contents.to_v4_json() {
      map.insert("contents".to_string(), Value::Object(body));
    }

    if !self.contents.metadata.is_empty() {
      map.insert("metadata".to_string(), Value::Object(
        self.contents.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
      ));
    }

    if !self.provider_states.is_empty() {
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    if !self.contents.matching_rules.is_empty() {
      map.insert("matchingRules".to_string(), matchers_to_json(&self.contents.matching_rules, &PactSpecification::V4));
    }

    if !self.contents.generators.is_empty() {
      map.insert("generators".to_string(), generators_to_json(&self.contents.generators, &PactSpecification::V4));
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
    V4InteractionType::Asynchronous_Messages
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

impl Interaction for AsynchronousMessage {
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
    true
  }

  fn as_message(&self) -> Option<Message> {
    let mut metadata = self.contents.metadata.clone();
    if let Some(content_type) = self.contents.contents.content_type() {
      if let Vacant(entry) = metadata.entry("contentType".to_string()) {
        entry.insert(json!( content_type.to_string() ));
      }
    }

    Some(Message {
      id: self.id.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: self.contents.contents.clone(),
      metadata,
      matching_rules: self.contents.matching_rules.rename("content", "body"),
      generators: self.contents.generators.clone()
    })
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
    self.contents.contents.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.contents.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.message_content_type()
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
    Some(self.clone())
  }

  fn is_v4_async_message(&self) -> bool {
    true
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    None
  }

  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp> {
    None
  }

  fn as_v4_async_message_mut(&mut self) -> Option<&mut AsynchronousMessage> {
    Some(self)
  }

  fn as_v4_sync_message_mut(&mut self) -> Option<&mut SynchronousMessage> {
    None
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
    Some(self.contents.matching_rules.clone())
  }

  fn pending(&self) -> bool {
    self.pending
  }
}

impl Default for AsynchronousMessage {
  fn default() -> Self {
    AsynchronousMessage {
      id: None,
      key: None,
      description: "Asynchronous/Message Interaction".to_string(),
      provider_states: vec![],
      contents: MessageContents {
        contents: OptionalBody::Missing,
        metadata: Default::default(),
        matching_rules: Default::default(),
        generators: Default::default()
      },
      comments: Default::default(),
      pending: false,
      plugin_config: Default::default(),
      interaction_markup: Default::default(),
      transport: None
    }
  }
}

impl PartialEq for AsynchronousMessage {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key &&
      self.description == other.description &&
      self.provider_states == other.provider_states &&
      self.contents == other.contents &&
      self.pending == other.pending
  }
}

impl Hash for AsynchronousMessage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.contents.contents.hash(state);
    for (k, v) in &self.contents.metadata {
      k.hash(state);
      hash_json(v, state);
    }
    self.contents.matching_rules.hash(state);
    self.contents.generators.hash(state);
    self.pending.hash(state);
  }
}

impl Display for AsynchronousMessage {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Asynchronous Message Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           pending, self.id, self.description, self.provider_states, self.contents.contents, self.contents.metadata)
  }
}

impl HttpPart for AsynchronousMessage {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
    unimplemented!()
  }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    unimplemented!()
  }

  fn body(&self) -> &OptionalBody {
    &self.contents.contents
  }

  fn body_mut(&mut self) -> &mut OptionalBody {
    &mut self.contents.contents
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.contents.matching_rules
  }

  fn matching_rules_mut(&mut self) -> &mut MatchingRules {
    &mut self.contents.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.contents.generators
  }

  fn generators_mut(&mut self) -> &mut Generators {
    &mut self.contents.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.contents.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| json_to_string(v))
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pretty_assertions::{assert_eq, assert_ne};
  use serde_json::{json, Value};

  use crate::bodies::OptionalBody;
  use crate::interaction::Interaction;
  use crate::matchingrules;
  use crate::matchingrules::MatchingRule;
  use crate::provider_states::ProviderState;
  use crate::v4::async_message::AsynchronousMessage;
  use crate::v4::interaction::V4Interaction;
  use crate::v4::message_parts::MessageContents;

  #[test]
  fn message_contents_without_a_content_attribute_are_not_dropped() {
    // A hand-written pact may put the payload where the body attributes belong. Reading it as a
    // missing body made an interaction verify successfully whatever the provider produced.
    // See https://github.com/pact-foundation/pact-python/issues/1103
    let json = json!({
      "type": "Asynchronous/Messages",
      "description": "a message",
      "contents": {
        "Payload": "exists"
      }
    });
    let message = AsynchronousMessage::from_json(&json, 0).unwrap();
    expect!(message.contents.contents).to(be_equal_to(
      OptionalBody::Present("{\"Payload\":\"exists\"}".into(), None, None)));
  }

  #[test]
  fn when_downgrading_message_to_v3_rename_the_matching_rules_from_content_to_body() {
    let message = AsynchronousMessage {
      contents: MessageContents {
        matching_rules: matchingrules! { "content" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] } },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    let v3 = message.as_message().unwrap();
    expect!(v3.matching_rules).to(be_equal_to(
      matchingrules! { "body" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] }}
    ));
  }

  #[test]
  fn calculate_hash_test() {
    let interaction = AsynchronousMessage::from_json(&json!({
      "description": "a Mallory message",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "contents": {
        "content": "That is some good Mallory.",
        "contentType": "text/plain",
        "encoded": false
      },
      "metadata": {
        "Content-Type": [
          "text/plain"
        ]
      },
      "type": "Asynchronous/Message"
    }), 0).unwrap();
    let hash = interaction.calc_hash();
    expect!(interaction.calc_hash()).to(be_equal_to(hash.as_str()));

    let interaction2 = interaction.with_key();
    expect!(interaction2.key.as_ref().unwrap()).to(be_equal_to(hash.as_str()));

    let json = interaction2.to_json();
    assert_eq!(json!({
      "description": "a Mallory message",
      "key": "24209d8c0b8f36fe",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "contents": {
        "content": "That is some good Mallory.",
        "contentType": "text/plain",
        "encoded": false,
      },
      "metadata": {
        "Content-Type": [ "text/plain" ]
      },
      "type": "Asynchronous/Messages"
    }), json);
  }

  #[test]
  fn hash_test() {
    let i1 = AsynchronousMessage::default();
    expect!(i1.calc_hash()).to(be_equal_to("774c6898f6381239"));

    let i2 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      .. AsynchronousMessage::default()
    };
    expect!(i2.calc_hash()).to(be_equal_to("4f2591cc234f166c"));

    let i3 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. AsynchronousMessage::default()
    };
    expect!(i3.calc_hash()).to(be_equal_to("51c8f79472fc5f71"));

    let i4 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    expect!(i4.calc_hash()).to(be_equal_to("5c7d5c0490b672a"));

    let i5 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        metadata: hashmap! { "ContentType".to_string() => Value::String("text/plain".to_string()) },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    expect!(i5.calc_hash()).to(be_equal_to("325703352b87e5df"));
  }

  #[test]
  fn equals_test() {
    let i1 = AsynchronousMessage::default();
    let i2 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      .. AsynchronousMessage::default()
    };
    let i3 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. AsynchronousMessage::default()
    };
    let i4 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    let i5 = AsynchronousMessage {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        metadata: hashmap! { "ContentType".to_string() => Value::String("text/plain".to_string()) },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };

    assert_eq!(i1, i1);
    assert_eq!(i2, i2);
    assert_eq!(i3, i3);
    assert_eq!(i4, i4);
    assert_eq!(i5, i5);

    assert_ne!(i1, i2);
    assert_ne!(i1, i3);
    assert_ne!(i1, i4);
    assert_ne!(i1, i5);
    assert_ne!(i2, i1);
    assert_ne!(i2, i3);
    assert_ne!(i2, i4);
    assert_ne!(i2, i5);
  }

  #[test]
  fn equals_test_with_different_keys() {
    let i1 = AsynchronousMessage {
      key: Some("i1".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        metadata: hashmap! { "ContentType".to_string() => Value::String("text/plain".to_string()) },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    let i2 = AsynchronousMessage {
      key: Some("i2".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      contents: MessageContents {
        contents: OptionalBody::from("That is some good Mallory."),
        metadata: hashmap! { "ContentType".to_string() => Value::String("text/plain".to_string()) },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };

    assert_eq!(i1, i1);
    assert_eq!(i2, i2);

    assert_ne!(i1, i2);
    assert_ne!(i2, i1);
  }
}
