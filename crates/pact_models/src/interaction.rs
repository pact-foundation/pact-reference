//! Models for Pact interactions

use std::fmt::{self, Debug, Display, Formatter};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::matchingrules::MatchingRules;
use crate::message::Message;
use crate::PactSpecification;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::interaction::{interaction_from_json, V4Interaction};
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;

/// Struct that defined an interaction conflict
#[derive(Debug, Clone)]
pub struct PactConflict {
  /// Description of the interactions
  pub interaction: String,
  /// Conflict description
  pub description: String
}

/// Interaction Trait
pub trait Interaction: Debug {
  /// The type of the interaction
  fn type_of(&self) -> String;

  /// If this is a request/response interaction
  fn is_request_response(&self) -> bool;

  /// Returns the request/response interaction if it is one
  fn as_request_response(&self) -> Option<RequestResponseInteraction>;

  /// If this is a message interaction
  fn is_message(&self) -> bool;

  /// Returns the message interaction if it is one
  fn as_message(&self) -> Option<Message>;

  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  fn id(&self) -> Option<String>;

  /// Description of this interaction. This needs to be unique in the pact file.
  fn description(&self) -> String;

  /// Set the Interaction ID
  fn set_id(&mut self, id: Option<String>);

  /// Set the description of this interaction. This needs to be unique in the pact file.
  fn set_description(&mut self, description: &str);

  /// Optional provider states for the interaction.
  /// See `<https://docs.pact.io/getting_started/provider_states>` for more info on provider states.
  fn provider_states(&self) -> Vec<ProviderState>;

  /// Mutable Optional provider states for the interaction.
  /// See `<https://docs.pact.io/getting_started/provider_states>` for more info on provider states.
  fn provider_states_mut(&mut self) -> &mut Vec<ProviderState>;

  /// Body of the response or message
  #[deprecated(
  since = "0.1.0",
  note = "Some interactions have multiple contents (like request/response), so it is impossible \
      to know which to return for this method"
  )]
  fn contents(&self) -> OptionalBody;

  /// The contents of the part to use for verification. For example, with HTTP interactions, this
  /// will be the response body
  fn contents_for_verification(&self) -> OptionalBody;

  /// Determine the content type of the interaction. If a `Content-Type` header or metadata value is present, the
  /// value of that value will be returned. Otherwise, the contents will be inspected.
  #[deprecated(
  since = "0.1.0",
  note = "Some interactions have multiple contents (like request/response), so it is impossible \
      to know which to return for this method"
  )]
  fn content_type(&self) -> Option<ContentType>;

  /// If this is a V4 interaction
  fn is_v4(&self) -> bool;

  /// Returns the interaction in V4 format
  fn as_v4(&self) -> Option<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>>;

  /// Returns a mutable reference for the  interaction. If the interaction is not a V4 format,
  /// will return None. The `as_v4` method can convert to V4 format (via cloning the data).
  fn as_v4_mut(&mut self) -> Option<&mut dyn V4Interaction>;

  /// If the interaction is V4 HTTP
  fn is_v4_http(&self) -> bool { false }

  /// Returns the interaction in V4 format
  fn as_v4_http(&self) -> Option<SynchronousHttp>;

  /// Returns the interaction in V4 format
  fn as_v4_async_message(&self) -> Option<AsynchronousMessage>;

  /// If the interaction is a V4 message
  fn is_v4_async_message(&self) -> bool { false }

  /// Returns the interaction in V4 format
  fn as_v4_sync_message(&self) -> Option<SynchronousMessage>;

  /// Returns the interaction in V4 format
  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp>;

  /// If the interaction is a V4 synchronous request/response message
  fn is_v4_sync_message(&self) -> bool { false }

  /// Returns the interaction in V4 format
  fn as_v4_async_message_mut(&mut self) -> Option<&mut AsynchronousMessage>;

  /// Returns the interaction in V4 format
  fn as_v4_sync_message_mut(&mut self) -> Option<&mut SynchronousMessage>;

  /// Clones this interaction and wraps it in a Box
  fn boxed(&self) -> Box<dyn Interaction + Send + Sync + RefUnwindSafe>;

  /// Clones this interaction and wraps it in an Arc
  fn arced(&self) -> Arc<dyn Interaction + Send + Sync + RefUnwindSafe>;

  /// Clones this interaction and wraps it in an Arc and Mutex
  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync + RefUnwindSafe>>;

  /// Returns the matching rules associated with this interaction (if there are any)
  #[deprecated(
  since = "0.2.1",
  note = "Some interactions have multiple contents (like request/response), so it is impossible \
      to know which to return for this method"
  )]
  fn matching_rules(&self) -> Option<MatchingRules>;

  /// If this interaction is pending (V4 only)
  fn pending(&self) -> bool { false }
}

impl Display for dyn Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(req_res) = self.as_request_response() {
      std::fmt::Display::fmt(&req_res, f)
    } else if let Some(mp) = self.as_message() {
      std::fmt::Display::fmt(&mp, f)
    } else if let Some(mp) = self.as_v4_http() {
      std::fmt::Display::fmt(&mp, f)
    } else if let Some(mp) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&mp, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Clone for Box<dyn Interaction> {
  fn clone(&self) -> Self {
    if self.is_v4() {
      if let Some(http) = self.as_v4_http() {
        Box::new(http)
      } else if let Some(message) = self.as_v4_async_message() {
        Box::new(message)
      } else {
        panic!("Internal Error - Tried to clone an interaction that was not valid")
      }
    } else if let Some(req_res) = self.as_request_response() {
      Box::new(req_res)
    } else if let Some(mp) = self.as_message() {
      Box::new(mp)
    } else {
      panic!("Internal Error - Tried to clone an interaction that was not valid")
    }
  }
}


/// Converts the JSON struct into an HTTP Interaction
pub fn http_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> anyhow::Result<Box<dyn Interaction + Send + Sync + RefUnwindSafe>> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| i.boxed()),
    _ => Ok(Box::new(RequestResponseInteraction::from_json(0, json, spec)?))
  }
}

/// Converts the JSON struct into a Message Interaction
pub fn message_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> anyhow::Result<Box<dyn Interaction + Send + Sync + RefUnwindSafe>> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| i.boxed()),
    _ => Message::from_json(0, json, spec).map(|i| i.boxed())
  }
}

pub(crate) fn parse_interactions(pact_json: &Value, spec_version: PactSpecification
) -> anyhow::Result<Vec<RequestResponseInteraction>> {
  if let Some(&Value::Array(ref array)) = pact_json.get("interactions") {
    array.iter().enumerate().map(|(index, ijson)| {
      RequestResponseInteraction::from_json(index, ijson, &spec_version)
    }).collect()
  }
  else {
    Ok(vec![])
  }
}
