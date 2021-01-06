//! The `message` module provides all functionality to deal with messages.

use std::collections::HashMap;

use maplit::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::content_types::ContentType;
use crate::models::generators::Generators;
use crate::models::matchingrules::MatchingRules;
use crate::models::provider_states::ProviderState;

use super::*;
use super::body_from_json;

/// Struct that defines a message.
#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Message {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    pub id: Option<String>,

    /// Description of this message interaction. This needs to be unique in the pact file.
    pub description: String,

    /// Optional provider state for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    #[serde(rename = "providerStates")]
    #[serde(default)]
    pub provider_states: Vec<ProviderState>,

    /// The contents of the message
    #[serde(default = "missing_body")]
    pub contents: OptionalBody,

    /// Metadata associated with this message.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Matching rules
    #[serde(rename = "matchingRules")]
    #[serde(default)]
    pub matching_rules: matchingrules::MatchingRules,

    /// Generators
    #[serde(default)]
    pub generators: generators::Generators
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

  fn is_plugin(&self) -> bool {
    false
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

  fn provider_states(&self) -> Vec<ProviderState> {
    self.provider_states.clone()
  }

  fn contents(&self) -> OptionalBody {
    self.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    let body = &self.contents;
    if body.has_content_type() {
      body.content_type()
    } else {
      match self.metadata.iter().find(|(k, _)| {
        let key = k.to_ascii_lowercase();
        key == "contenttype" || key == "content-type"
      }) {
        Some((_, v)) => ContentType::parse(v.as_str()).ok(),
        None => self.detect_content_type()
      }
    }
  }

  fn as_v4(&self) -> V4Interaction {
    V4Interaction::AsynchronousMessages {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: self.contents.clone(),
      metadata: self.metadata.iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }
}

impl Message {
    /// Returns a default message
    pub fn default() -> Message {
      Message {
        id: None,
        description: s!("message"),
        provider_states: vec![],
        contents: OptionalBody::Missing,
        metadata: hashmap!{
          "contentType".into() => "application/json".into()
        },
        matching_rules: matchingrules::MatchingRules::default(),
        generators: Generators::default()
      }
    }

    /// Constructs a `Message` from the `Json` struct.
    pub fn from_json(index: usize, json: &Value, spec_version: &PactSpecification) -> Result<Message, String> {
        match spec_version {
            &PactSpecification::V3 => {
                let description = match json.get("description") {
                    Some(v) => match *v {
                        Value::String(ref s) => s.clone(),
                        _ => v.to_string()
                    },
                    None => format!("Message {}", index)
                };
                let provider_states = ProviderState::from_json(json);
                let metadata = match json.get("metaData") {
                    Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                        (k.clone(), match v {
                            &Value::String(ref s) => s.clone(),
                            _ => v.to_string()
                        })
                    }).collect(),
                    _ => hashmap!{}
                };
                Ok(Message {
                  id: None,
                  description,
                  provider_states,
                  contents: body_from_json(json, "contents", &None),
                  matching_rules: matchingrules::matchers_from_json(json, &None),
                  metadata,
                  generators: Generators::default()
                })
            },
            _ => Err(s!("Messages require Pact Specification version 3"))
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

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| v.clone())
  }
}

impl Display for Message {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Message ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           self.id, self.description, self.provider_states, self.contents, self.metadata)
  }
}

fn missing_body() -> OptionalBody {
    OptionalBody::Missing
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use serde_json;

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
            name: s!("provider state"),
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
        expect!(message.contents.str_value()).to(be_equal_to("{\"hello\":\"world\"}"));
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
        expect!(message.contents.str_value()).to(be_equal_to("hello world"));
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
        expect!(message.contents.str_value()).to(be_equal_to(""));
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
        metadata: hashmap!{ s!("contentType") => s!("text/plain") },
        .. Message::default()
      };
      expect!(Interaction::content_type(&message).unwrap_or_default().to_string()).to(be_equal_to("text/plain"));
    }

    #[test]
    fn message_mimetype_defaults_to_json() {
      let message = Message::default();
      expect!(Interaction::content_type(&message).unwrap_or_default().to_string()).to(be_equal_to("application/json"));
    }

    #[test]
    fn ignoring_v1_provider_state_when_deserializing_message() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {},
            "generators": {}
        }"#;

        // This line should panic, because providerState is not the name of the field.
        let message: Message = serde_json::from_str(message_json).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states.iter()).to(be_empty());
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

        let message: Message = serde_json::from_str(message_json).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: s!("provider state"),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }
}
