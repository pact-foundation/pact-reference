//! V4 specification models

use std::{fmt, io, mem};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::string::ToString;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::*;
use maplit::*;
use nom::lib::std::fmt::Formatter;
use serde_json::{json, Value};

use crate::models::{Consumer, detect_content_type_from_bytes, generators, Interaction, matchingrules, OptionalBody, Pact, PactSpecification, Provider, provider_states, ReadWritePact, RequestResponseInteraction, RequestResponsePact, PACT_RUST_VERSION};
use crate::models::content_types::ContentType;
use crate::models::generators::generators_to_json;
use crate::models::json_utils::{hash_json, json_to_string};
use crate::models::matchingrules::matchers_to_json;
use crate::models::message::Message;
use crate::models::message_pact::MessagePact;
use crate::models::provider_states::ProviderState;
use crate::models::v4::http_parts::{body_from_json, HttpRequest, HttpResponse};

/// V4 Interaction Type
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum V4InteractionType {
  /// Synchronous HTTP Request Response
  Synchronous_HTTP,
  /// Asynchronous Messages
  Asynchronous_Messages,
  /// Synchronous Messages in the form Request message -> Response messages
  Synchronous_Messages
}

impl Default for V4InteractionType {
  fn default() -> Self {
    V4InteractionType::Synchronous_HTTP
  }
}

impl Display for V4InteractionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match *self {
      V4InteractionType::Synchronous_HTTP => f.write_str("Synchronous/HTTP"),
      V4InteractionType::Asynchronous_Messages => f.write_str("Asynchronous/Messages"),
      V4InteractionType::Synchronous_Messages => f.write_str("Synchronous/Messages")
    }
  }
}

impl V4InteractionType {
  /// Returns the V4 interaction type from the string value
  pub fn from_str(type_str: &str) -> Result<V4InteractionType, String> {
    match type_str {
      "Synchronous/HTTP" => Ok(V4InteractionType::Synchronous_HTTP),
      "Asynchronous/Messages" => Ok(V4InteractionType::Asynchronous_Messages),
      "Synchronous/Messages" => Ok(V4InteractionType::Synchronous_Messages),
      _ => Err(format!("'{}' is not a valid V4 interaction type", type_str))
    }
  }
}

pub mod http_parts;

/// V4 Interaction Types
#[derive(Debug, Clone, Eq)]
pub enum V4Interaction {
  /// Synchronous HTTP request/response interaction
  SynchronousHttp {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    id: Option<String>,
    /// Unique key for this interaction
    key: Option<String>,
    /// A description for the interaction. Must be unique within the Pact file
    description: String,
    /// Optional provider states for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    provider_states: Vec<provider_states::ProviderState>,
    /// Request of the interaction
    request: HttpRequest,
    /// Response of the interaction
    response: HttpResponse
  },

  /// Asynchronous interactions as a sequence of messages
  AsynchronousMessages {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    id: Option<String>,
    /// Unique key for this interaction
    key: Option<String>,
    /// A description for the interaction. Must be unique within the Pact file
    description: String,
    /// Optional provider state for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    provider_states: Vec<ProviderState>,
    /// The contents of the message
    contents: OptionalBody,
    /// Metadata associated with this message.
    metadata: HashMap<String, Value>,
    /// Matching rules
    matching_rules: matchingrules::MatchingRules,
    /// Generators
    generators: generators::Generators
  }
}

impl V4Interaction {
  /// Convert the interaction to a JSON Value
  pub fn to_json(&self) -> Value {
    match self {
      V4Interaction::SynchronousHttp { key, description, provider_states, request, response, .. } => {
        let mut json = json!({
          "type": V4InteractionType::Synchronous_HTTP.to_string(),
          "key": key.clone().unwrap_or_else(|| self.calc_hash()),
          "description": description.clone(),
          "request": request.to_json(),
          "response": response.to_json()
        });

        if !provider_states.is_empty() {
          let map = json.as_object_mut().unwrap();
          map.insert("providerStates".to_string(), Value::Array(provider_states.iter().map(|p| p.to_json()).collect()));
        }

        json
      },

      V4Interaction::AsynchronousMessages { key, description, provider_states, contents, metadata, matching_rules, generators, .. } => {
        let mut json = json!({
          "type": V4InteractionType::Asynchronous_Messages.to_string(),
          "key": key.clone().unwrap_or_else(|| self.calc_hash()),
          "description": description.clone()
        });

        if let Value::Object(body) = contents.to_v4_json() {
          let map = json.as_object_mut().unwrap();
          map.insert("contents".to_string(), Value::Object(body));
        }

        if !metadata.is_empty() {
          let map = json.as_object_mut().unwrap();
          map.insert("metadata".to_string(), Value::Object(
            metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
          ));
        }

        if !provider_states.is_empty() {
          let map = json.as_object_mut().unwrap();
          map.insert("providerStates".to_string(), Value::Array(provider_states.iter().map(|p| p.to_json()).collect()));
        }

        if !matching_rules.is_empty() {
          let map = json.as_object_mut().unwrap();
          map.insert("matchingRules".to_string(), matchers_to_json(matching_rules, &PactSpecification::V4));
        }

        if !generators.is_empty() {
          let map = json.as_object_mut().unwrap();
          map.insert("generators".to_string(), generators_to_json(generators, &PactSpecification::V4));
        }

        json
      }
    }
  }

  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Returns all the field values as a tuple if this is a Synchronous Http interaction, returns a error otherwise
  pub fn as_synchronous_http(&self) -> Result<(Option<String>, Option<String>, String, Vec<provider_states::ProviderState>, HttpRequest, HttpResponse), String> {
    match self {
      V4Interaction::SynchronousHttp { id, key, description, provider_states, request, response } =>
        Ok((id.clone(), key.clone(), description.clone(), provider_states.clone(), request.clone(), response.clone())),
      _ => Err("V4 interaction is not a synchronous http interaction".to_string())
    }
  }

  /// Returns all the field values as a tuple if this is a Asynchronous Message interaction, returns a error otherwise
  pub fn as_asynchronous_message(&self) -> Result<(Option<String>, Option<String>, String, Vec<ProviderState>, OptionalBody, HashMap<String, Value>, matchingrules::MatchingRules, generators::Generators), String> {
    match self {
      V4Interaction::AsynchronousMessages { id, key, description, provider_states, contents, metadata, matching_rules, generators } =>
        Ok((id.clone(), key.clone(), description.clone(), provider_states.clone(), contents.clone(), metadata.clone(), matching_rules.clone(), generators.clone() )),
      _ => Err("V4 interaction is not a asynchronous message interaction".to_string())
    }
  }

  /// Returns the distinct key value for this interaction.
  pub fn key(&self) -> Option<String> {
    match self {
      V4Interaction::SynchronousHttp { key, .. } => key.clone(),
      V4Interaction::AsynchronousMessages { key, .. } => key.clone()
    }
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> V4Interaction {
    match self {
      V4Interaction::SynchronousHttp { id, description, provider_states, request, response, .. } => V4Interaction::SynchronousHttp {
        id: id.clone(),
        key: Some(self.calc_hash()),
        description: description.clone(),
        provider_states: provider_states.clone(),
        request: request.clone(),
        response: response.clone()
      },
      V4Interaction::AsynchronousMessages { id, description, provider_states, contents, metadata, matching_rules, generators, .. } => V4Interaction::AsynchronousMessages {
        id: id.clone(),
        key: Some(self.calc_hash()),
        description: description.clone(),
        provider_states: provider_states.clone(),
        contents: contents.clone(),
        metadata: metadata.clone(),
        matching_rules: matching_rules.clone(),
        generators: generators.clone()
      }
    }
  }
}

impl Interaction for V4Interaction {
  fn type_of(&self) -> String {
    match self {
      V4Interaction::SynchronousHttp { .. } => format!("V4 {}", V4InteractionType::Synchronous_HTTP),
      V4Interaction::AsynchronousMessages { .. } => format!("V4 {}", V4InteractionType::Asynchronous_Messages)
    }
  }

  fn is_request_response(&self) -> bool {
    match self {
      V4Interaction::SynchronousHttp { .. } => true,
      _ => false
    }
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    match self {
      V4Interaction::SynchronousHttp { id, description, provider_states, request, response, .. } =>
        Some(RequestResponseInteraction {
          id: id.clone(),
          description: description.clone(),
          provider_states: provider_states.clone(),
          request: request.as_v3_request(),
          response: response.as_v3_response()
        }),
      _ => None
    }
  }

  fn is_message(&self) -> bool {
    match self {
      V4Interaction::AsynchronousMessages { .. } => true,
      _ => false
    }
  }

  fn is_plugin(&self) -> bool {
    false
  }

  fn as_message(&self) -> Option<Message> {
    match self {
      V4Interaction::AsynchronousMessages { id, description, provider_states, contents, metadata, matching_rules, generators, .. } =>
        Some(Message {
          id: id.clone(),
          description: description.clone(),
          provider_states: provider_states.clone(),
          contents: contents.clone(),
          metadata: metadata.iter().map(|(k, v)| (k.clone(), json_to_string(v))).collect(),
          matching_rules: matching_rules.clone(),
          generators: generators.clone()
        }),
      _ => None
    }
  }

  fn id(&self) -> Option<String> {
    match self {
      V4Interaction::SynchronousHttp { id, .. } => id.clone(),
      V4Interaction::AsynchronousMessages { id, .. } => id.clone()
    }
  }

  fn description(&self) -> String {
    match self {
      V4Interaction::SynchronousHttp { description, .. } => description.clone(),
      V4Interaction::AsynchronousMessages { description, .. } => description.clone()
    }
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    match self {
      V4Interaction::SynchronousHttp { provider_states, .. } => provider_states.clone(),
      V4Interaction::AsynchronousMessages { provider_states, .. } => provider_states.clone()
    }
  }

  fn contents(&self) -> OptionalBody {
    match self {
      V4Interaction::SynchronousHttp { response, .. } => response.body.clone(),
      V4Interaction::AsynchronousMessages { contents, .. } => contents.clone()
    }
  }

  fn content_type(&self) -> Option<ContentType> {
    match self {
      V4Interaction::SynchronousHttp { response, .. } => response.content_type(),
      V4Interaction::AsynchronousMessages { contents, metadata, .. } =>
        calc_content_type(contents, &metadata_to_headers(metadata))
    }
  }

  fn as_v4(&self) -> V4Interaction {
    self.clone()
  }
}

impl Default for V4Interaction {
  fn default() -> Self {
    V4Interaction::SynchronousHttp {
      id: None,
      key: None,
      description: "Synchronous/HTTP Interaction".to_string(),
      provider_states: vec![],
      request: HttpRequest::default(),
      response: HttpResponse::default()
    }
  }
}

impl PartialEq for V4Interaction {
  fn eq(&self, other: &Self) -> bool {
    if mem::discriminant(self) == mem::discriminant(other) {
      match self {
        V4Interaction::SynchronousHttp { description, provider_states, request, response, .. } => {
          let (_, _, other_desc, other_states, other_request, other_response) = other.as_synchronous_http().unwrap();
          description.clone() == other_desc && provider_states.clone() == other_states &&
            request.clone() == other_request && response.clone() == other_response
        }
        V4Interaction::AsynchronousMessages { description, provider_states, contents, metadata, matching_rules, generators, .. } => {
          let (_, _, other_description, other_provider_states, other_contents, other_metadata, other_matching_rules, other_generators) = other.as_asynchronous_message().unwrap();
          description.clone() == other_description && provider_states.clone() == other_provider_states &&
            contents.clone() == other_contents && metadata.clone() == other_metadata &&
            matching_rules.clone() == other_matching_rules &&
            generators.clone() == other_generators
        }
      }
    } else {
      false
    }
  }
}

impl Hash for V4Interaction {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      V4Interaction::SynchronousHttp { description, provider_states, request, response, .. } => {
        description.hash(state);
        provider_states.hash(state);
        request.hash(state);
        response.hash(state);
      }
      V4Interaction::AsynchronousMessages { description, provider_states, contents, metadata, matching_rules, generators, .. } => {
        description.hash(state);
        provider_states.hash(state);
        contents.hash(state);
        for (k, v) in metadata {
          k.hash(state);
          hash_json(v, state);
        }
        matching_rules.hash(state);
        generators.hash(state);
      }
    }
  }
}

fn calc_content_type(body: &OptionalBody, headers: &Option<HashMap<String, Vec<String>>>) -> Option<ContentType> {
  body.content_type()
    .or_else(|| headers.as_ref().map(|h| {
      match h.iter().find(|kv| kv.0.to_lowercase() == "content-type") {
        Some((_, v)) => ContentType::parse(v[0].as_str()).ok(),
        None => None
      }
    }).flatten())
    .or_else(|| if body.is_present() {
      detect_content_type_from_bytes(&*body.value())
    } else {
      None
    })
}

/// V4 spec Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone)]
pub struct V4Pact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of messages between the consumer and provider.
  pub interactions: Vec<V4Interaction>,
  /// Metadata associated with this pact.
  pub metadata: BTreeMap<String, Value>
}

impl V4Pact {
  fn metadata_to_json(&self) -> Value {
    let mut md_map: serde_json::Map<String, Value> = self.metadata.iter()
      .map(|(k, v)| {
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, v.clone())
      })
      .collect();

    md_map.insert("pactSpecification".to_string(), json!({"version" : PactSpecification::V4.version_str()}));
    md_map.insert("pactRust".to_string(), json!({"version" : PACT_RUST_VERSION.unwrap_or("unknown")}));
    Value::Object(md_map)
  }
}

impl Pact for V4Pact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i as &dyn Interaction).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), json_to_string(v))).collect())),
        _ => None
      }
    }).filter(|v| v.is_some())
      .map(|v| v.unwrap())
      .collect()
  }

  fn to_json(&self, _: PactSpecification) -> Value {
    json!({
      "consumer": self.consumer.to_json(),
      "provider": self.provider.to_json(),
      "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json()).collect()),
      "metadata": self.metadata_to_json()
    })
  }

  fn as_request_response_pact(&self) -> Result<RequestResponsePact, String> {
    unimplemented!()
  }

  fn as_message_pact(&self) -> Result<MessagePact, String> {
    unimplemented!()
  }

  fn as_v4_pact(&self) -> Result<V4Pact, String> {
    Ok(self.clone())
  }

  fn specification_version(&self) -> PactSpecification {
    PactSpecification::V4
  }
}

impl Default for V4Pact {
  fn default() -> Self {
    V4Pact {
      consumer: Default::default(),
      provider: Default::default(),
      interactions: vec![],
      metadata: Default::default()
    }
  }
}

impl ReadWritePact for V4Pact {
  fn read_pact(path: &Path) -> io::Result<V4Pact> {
    let mut f = File::open(path)?;
    let pact_json = serde_json::from_reader(&mut f);
    match pact_json {
      Ok(ref json) => {
        let metadata = meta_data_from_json(json);
        let consumer = match json.get("consumer") {
          Some(v) => Consumer::from_json(v),
          None => Consumer { name: "consumer".into() }
        };
        let provider = match json.get("provider") {
          Some(v) => Provider::from_json(v),
          None => Provider { name: "provider".into() }
        };
        Ok(V4Pact {
          consumer,
          provider,
          interactions: interactions_from_json(json, &*path.to_string_lossy()),
          metadata
        })
      },
      Err(err) => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - {}", err)))
    }
  }

  fn merge(&self, other: &dyn Pact) -> Result<V4Pact, String> {
    if self.consumer.name == other.consumer().name && self.provider.name == other.provider().name {
      Ok(V4Pact {
        consumer: self.consumer.clone(),
        provider: self.provider.clone(),
        interactions: self.interactions.iter()
          .merge_join_by(other.interactions().iter().map(|i| i.as_v4()), |a, b| {
            match (a.key(), b.key()) {
              (Some(key_a), Some(key_b)) => Ord::cmp(&key_a, &key_b),
              (_, _) => {
                let type_a = dbg!(a.type_of());
                let type_b = dbg!(b.type_of());
                let cmp = Ord::cmp(&type_a, &type_b);
                if cmp == Ordering::Equal {
                  let cmp = Ord::cmp(&a.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                                     &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>());
                  if cmp == Ordering::Equal {
                    Ord::cmp(&a.description(), &b.description())
                  } else {
                    cmp
                  }
                } else {
                  cmp
                }
              }
            }
          })
          .map(|either| match either {
            Left(i) => i.clone(),
            Right(i) => i,
            Both(_, i) => i.clone()
          })
          .collect(),
        metadata: self.metadata.clone()
      })
    } else {
      Err(s!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

/// Creates a V4 Pact from the provided JSON struct
pub fn from_json(source: &str, pact_json: &Value) -> Result<Box<dyn Pact>, String> {
  let metadata = meta_data_from_json(pact_json);
  let consumer = match pact_json.get("consumer") {
    Some(v) => Consumer::from_json(v),
    None => Consumer { name: "consumer".into() }
  };
  let provider = match pact_json.get("provider") {
    Some(v) => Provider::from_json(v),
    None => Provider { name: "provider".into() }
  };
  Ok(Box::new(V4Pact {
    consumer,
    provider,
    interactions: interactions_from_json(pact_json, source),
    metadata
  }))
}

fn interactions_from_json(json: &Value, source: &str) -> Vec<V4Interaction> {
  match json.get("interactions") {
    Some(v) => match *v {
      Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
        interaction_from_json(source, index, ijson).ok()
      }).filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect(),
      _ => vec![]
    },
    None => vec![]
  }
}

/// Create an interaction from a JSON struct
pub fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> Result<V4Interaction, String> {
  match ijson.get("type") {
    Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        let id = ijson.get("_id").map(|id| json_to_string(id));
        let key = ijson.get("key").map(|id| json_to_string(id));
        let description = match ijson.get("description") {
          Some(v) => match *v {
            Value::String(ref s) => s.clone(),
            _ => v.to_string()
          },
          None => format!("Interaction {}", index)
        };
        let provider_states = provider_states::ProviderState::from_json(ijson);
        match i_type {
          V4InteractionType::Synchronous_HTTP => {
            let request = ijson.get("request").cloned().unwrap_or_default();
            let response = ijson.get("response").cloned().unwrap_or_default();
            Ok(V4Interaction::SynchronousHttp {
              id,
              key,
              description,
              provider_states,
              request: HttpRequest::from_json(&request),
              response: HttpResponse::from_json(&response)
            })
          }
          V4InteractionType::Asynchronous_Messages => {
            let metadata = match ijson.get("metadata") {
              Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                (k.clone(), v.clone())
              }).collect(),
              _ => hashmap!{}
            };
            let as_headers = metadata_to_headers(&metadata);
            Ok(V4Interaction::AsynchronousMessages {
              id,
              key,
              description,
              provider_states,
              metadata,
              contents: body_from_json(ijson, "contents", &as_headers),
              matching_rules: matchingrules::matchers_from_json(ijson, &None),
              generators: generators::generators_from_json(ijson)
            })
          }
          V4InteractionType::Synchronous_Messages => {
            warn!("Interaction type '{}' is currently unimplemented. It will be ignored. Source: {}", i_type, source);
            Err(format!("Interaction type '{}' is currently unimplemented. It will be ignored. Source: {}", i_type, source))
          }
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        Err(format!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source))
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      Err(format!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source))
    }
  }
}

fn metadata_to_headers(metadata: &HashMap<String, Value>) -> Option<HashMap<String, Vec<String>>> {
  if let Some(content_type) = metadata.get("contentType") {
    Some(hashmap! {
      "Content-Type".to_string() => vec![ json_to_string(content_type) ]
    })
  } else {
    None
  }
}

fn meta_data_from_json(pact_json: &Value) -> BTreeMap<String, Value> {
  match pact_json.get("metadata") {
    Some(v) => match *v {
      Value::Object(ref obj) => obj.iter()
        .map(|(k, v)| (k.clone(), v.clone())).collect(),
      _ => btreemap!{}
    },
    None => btreemap!{}
  }
}

#[cfg(test)]
mod tests;
