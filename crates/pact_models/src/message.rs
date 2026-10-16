//! The `message` module provides all functionality to deal with message interactions.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::panic::RefUnwindSafe;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use maplit::hashmap;
use serde_json::{json, Value};
use tracing::warn;

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::generators::{Generators, generators_to_json};
use crate::http_parts::HttpPart;
use crate::interaction::Interaction;
use crate::json_utils::{body_from_json, json_to_string};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::PactSpecification;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::interaction::V4Interaction;
use crate::v4::message_parts::MessageContents;
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;

/// Struct that defines a message.
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Message {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    pub id: Option<String>,

    /// Description of this message interaction. This needs to be unique in the pact file.
    pub description: String,

    /// Optional provider state for the interaction.
    /// See `<https://docs.pact.io/getting_started/provider_states>` for more info on provider states.
    pub provider_states: Vec<ProviderState>,

    /// The contents of the message
    pub contents: OptionalBody,

    /// Metadata associated with this message.
    pub metadata: HashMap<String, Value>,

    /// Matching rules
    pub matching_rules: MatchingRules,

    /// Generators
    pub generators: Generators
}

impl Interaction for Message {
  fn type_of(&self) -> String {
    "V3 Asynchronous/Messages".into()
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
    Some(self.clone())
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
    self.contents.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.message_content_type()
  }

  fn is_v4(&self) -> bool {
    false
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>> {
    self.as_v4_async_message().map(|i| i.boxed_v4())
  }

  fn as_v4_mut(&mut self) -> Option<&mut dyn V4Interaction> {
    None
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    None
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    Some(AsynchronousMessage {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: MessageContents {
        contents: self.contents.clone(),
        metadata: self.metadata.iter()
          .map(|(k, v)| (k.clone(), v.clone()))
          .collect(),
        matching_rules: self.matching_rules.rename("body", "content"),
        generators: self.generators.clone()
      },
      .. Default::default()
    })
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    None
  }

  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp> {
    None
  }

  fn as_v4_async_message_mut(&mut self) -> Option<&mut AsynchronousMessage> {
    None
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
    Some(self.matching_rules.clone())
  }
}

impl Message {
    /// Constructs a `Message` from the `Json` struct.
    pub fn from_json(index: usize, json: &Value, spec_version: &PactSpecification) -> anyhow::Result<Message> {
        match spec_version {
            PactSpecification::V3 => {
                let id = json.get("_id").map(|id| json_to_string(id));
                let description = match json.get("description") {
                    Some(v) => match *v {
                        Value::String(ref s) => s.clone(),
                        _ => v.to_string()
                    },
                    None => format!("Message {}", index)
                };
                let provider_states = ProviderState::from_json(json);
                let metadata = match json.get("metaData").or(json.get("metadata")) {
                  Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                      (k.clone(), v.clone())
                  }).collect(),
                  _ => hashmap!{},
                };
              let content_type = metadata.iter()
                .find(|(k, _)| {
                  let key = k.to_ascii_lowercase();
                  key == "contenttype" || key == "content-type"
                })
                .map(|(_, v)| json_to_string(v))
                .map(|s| ContentType::parse(s.as_str()).ok())
                .flatten();

              let body = if let Some(ct) = content_type {
                let mut body = body_from_json(json, "contents", &Some(hashmap!{
                  "content-type".to_string() => vec![ ct.to_string() ]
                }));
                body.set_content_type(&ct);
                body
              } else {
                body_from_json(json, "contents", &None)
              };

              Ok(Message {
                  id,
                  description,
                  provider_states,
                  contents: body,
                  matching_rules: matchers_from_json(json, &None)?,
                  metadata,
                  generators: Generators::default(),
                })
            },
            _ => Err(anyhow!("Messages require Pact Specification version 3"))
        }
    }

    /// Converts this interaction to a `Value` struct.
    /// note: spec version is preserved for compatibility with the RequestResponsePact interface
    /// and for future use
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
      let mut value = json!({
          "description".to_string(): Value::String(self.description.clone()),
          "metadata".to_string(): self.metadata
      });
      {
        let map = value.as_object_mut().unwrap();

        if self.matching_rules.is_not_empty() {
            map.insert("matchingRules".to_string(), matchers_to_json(
            &self.matching_rules.clone(), spec_version));
        }
        if self.generators.is_not_empty() {
          map.insert("generators".to_string(), generators_to_json(
            &self.generators.clone(), spec_version));
        }

        match self.contents {
          OptionalBody::Present(ref body, _, _) => {
            let content_type = self.message_content_type().unwrap_or_default();
            if content_type.is_json() {
            match serde_json::from_slice(body) {
                Ok(json_body) => { map.insert("contents".to_string(), json_body); },
              Err(err) => {
                warn!("Failed to parse json body: {}", err);
                map.insert("contents".to_string(), Value::String(BASE64.encode(body)));
              }
            }
            } else if content_type.is_binary() {
              map.insert("contents".to_string(), Value::String(BASE64.encode(body)));
          } else {
              match from_utf8(body) {
                Ok(s) => map.insert("contents".to_string(), Value::String(s.to_string())),
                Err(_) => map.insert("contents".to_string(), Value::String(BASE64.encode(body)))
            };
            }
          },
          OptionalBody::Empty => { map.insert("contents".to_string(), Value::String("".to_string())); },
          OptionalBody::Missing => (),
          OptionalBody::Null => { map.insert("contents".to_string(), Value::Null); }
        }
        if !self.provider_states.is_empty() {
          map.insert("providerStates".to_string(), Value::Array(self.provider_states.iter().map(|p| p.to_json()).collect()));
        }
      }

      value
  }

  /// Returns the content type of the message by returning the content type associated with
  /// the body, or by looking it up in the message metadata
  pub fn message_content_type(&self) -> Option<ContentType> {
    let body = &self.contents;
    if body.has_content_type() {
      body.content_type()
    } else {
      match self.metadata.iter().find(|(k, _)| {
        let key = k.to_ascii_lowercase();
        key == "contenttype" || key == "content-type"
      }) {
        Some((_, v)) => ContentType::parse(json_to_string(&v).as_str()).ok(),
        None => self.detect_content_type()
      }
    }
  }

  /// Converts this message into a V4 message content
  pub fn as_message_content(&self) -> MessageContents {
    MessageContents {
      contents: self.contents.clone(),
      metadata: self.metadata.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }
}

impl HttpPart for Message {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
    unimplemented!()
  }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    unimplemented!()
  }

  fn body(&self) -> &OptionalBody {
    &self.contents
  }

  fn body_mut(&mut self) -> &mut OptionalBody {
    &mut self.contents
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn matching_rules_mut(&mut self) -> &mut MatchingRules {
    &mut self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn generators_mut(&mut self) -> &mut Generators {
    &mut self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| json_to_string(&v[0]))
  }

  fn add_header(&mut self, key: &str, val: Vec<&str>) {
    if !val.is_empty() {
      if val.len() == 1 {
        self.metadata.insert(key.to_string(), Value::String(val.first().unwrap().to_string()));
      } else {
        self.metadata.insert(key.to_string(), Value::Array(val.iter()
          .map(|v| Value::String(v.to_string())).collect()));
      }
    } else {
      self.metadata.insert(key.to_string(), Value::Null);
    }
  }
}

impl Display for Message {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Message ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           self.id, self.description, self.provider_states, self.contents, self.metadata)
  }
}

impl Default for Message {
  fn default() -> Self {
    Message {
      id: None,
      description: "message".to_string(),
      provider_states: vec![],
      contents: OptionalBody::Missing,
      metadata: hashmap!{
          "contentType".into() => "application/json".into()
        },
      matching_rules: MatchingRules::default(),
      generators: Generators::default()
    }
  }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::path::Path;

    use bytes::Bytes;
    use expectest::expect;
    use expectest::prelude::*;
    use serde_json;

    use crate::matchingrules;
    use crate::matchingrules::MatchingRule;

    use super::*;

    #[test]
    fn loading_message_from_json() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {}
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: "provider state".to_string(),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn defaults_to_number_if_no_description() {
        let message_json = r#"{
            "providerState": "provider state"
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("Message 0"));
    }

    #[test]
    fn defaults_to_none_if_no_provider_state() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_states.iter()).to(be_empty());
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn defaults_to_none_if_provider_state_null() {
        let message_json = r#"{
            "providerState": null
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_states.iter()).to(be_empty());
    }

    #[test]
    fn returns_an_error_if_the_spec_version_is_less_than_three() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state"
        }"#;
        let result = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V1);
        expect!(result).to(be_err());
    }

    #[test]
    fn message_with_json_body() {
        let message_json = r#"{
            "contents": {
                "hello": "world"
            },
            "metadata": {
                "contentType": "application/json"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.value_as_string()).to(be_some().value("{\"hello\":\"world\"}"));
    }

    #[test]
    fn message_with_non_json_body() {
        let message_json = r#"{
            "contents": "hello world",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.value_as_string()).to(be_some().value("hello world"));
    }

    #[test]
    fn message_with_empty_body() {
        let message_json = r#"{
            "contents": "",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.value_as_string()).to(be_none());
    }

    #[test]
    fn message_with_missing_body() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to(OptionalBody::Missing));
    }

    #[test]
    fn message_with_null_body() {
        let message_json = r#"{
            "contents": null,
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to(OptionalBody::Null));
    }

    #[test]
    fn message_mimetype_is_based_on_the_metadata() {
      let message = Message {
        metadata: hashmap!{ "contentType".to_string() => Value::String("text/plain".to_string()) },
        .. Message::default()
      };
      expect!(message.message_content_type().unwrap_or_default().to_string()).to(be_equal_to("text/plain"));
    }

    #[test]
    fn message_mimetype_defaults_to_json() {
      let message = Message::default();
      expect!(message.message_content_type().unwrap_or_default().to_string()).to(be_equal_to("application/json"));
    }

    #[test]
    fn v1_provider_state_when_deserializing_message() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {},
            "generators": {}
        }"#;

        let message_json: Value = serde_json::from_str(message_json).unwrap();
        let message = Message::from_json(0, &message_json, &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states.len()).to(be_equal_to(1));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn loading_message_from_json_by_deserializing() {
        let message_json = r#"{
            "description": "String",
            "providerStates": [{ "name": "provider state", "params": {} }],
            "matchingRules": {},
            "generators": {}
        }"#;

        let message_json: Value = serde_json::from_str(message_json).unwrap();
        let message = Message::from_json(0, &message_json, &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: "provider state".to_string(),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

  #[test]
  fn when_upgrading_message_pact_to_v4_rename_the_matching_rules_from_body_to_content() {
    let message = Message {
      contents: OptionalBody::Missing,
      matching_rules: matchingrules! { "body" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] } },
      .. Message::default()
    };
    let v4 = message.as_v4_async_message().unwrap();
    expect!(v4.contents.matching_rules).to(be_equal_to(
      matchingrules! { "content" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] }}
    ));
  }

  #[test]
  fn message_with_json_body_serialises() {
    let message_json = r#"{
        "contents": {
            "hello": "world"
        },
        "metadata": {
            "contentType": "application/json"
        }
    }"#;
    let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
    let v = message.to_json(&PactSpecification::V3);
    expect!(v.get("contents").unwrap().get("hello").unwrap().as_str().unwrap()).to(be_equal_to("world"));
  }

  #[test]
  fn message_with_binary_body_serialises() {
    let message_json = r#"{
        "metadata": {
            "contentType": "application/octet-stream"
        }
    }"#;

    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("tests/data")
      .join("message_with_binary_body_serialises.zip")
      .to_owned();

    let content_type = ContentType::parse("application/octet-stream").unwrap();
    let contents = fs::read(file).unwrap();
    let encoded = concat!(
      "UEsDBAoAAAAAAI2rtlKd3GsXCgAAAAoAAAAIABwAZmlsZS50eHRVVAkAA9nqqGDb6qhgdXgLAAEE9QEAAAQUAAAAdGVzdCBkYXRhClBL",
      "AQIeAwoAAAAAAI2rtlKd3GsXCgAAAAoAAAAIABgAAAAAAAEAAACkgQAAAABmaWxlLnR4dFVUBQAD2eqoYHV4CwABBPUBAAAEFAAAAFBLBQYAAAAAAQABAE4",
      "AAABMAAAAAAA=");

    let mut message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
    message.contents = OptionalBody::Present(Bytes::from(contents), Some(content_type), None);
    let json = message.to_json(&PactSpecification::V3);

    expect!(json.get("contents").unwrap().as_str().unwrap()).to(be_equal_to(encoded));
  }

  #[test]
  fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
    let json = json!({
      "_id": "123456789",
      "description": "Test Interaction"
    });
    let interaction = Message::from_json(0, &json, &PactSpecification::V3);
    let interaction = interaction.unwrap();

    expect!(interaction.id).to(be_some().value("123456789".to_string()));
  }
}
