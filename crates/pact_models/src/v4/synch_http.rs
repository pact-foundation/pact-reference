//! Synchronous HTTP interactions

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
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
use crate::v4::http_parts::{HttpRequest, HttpResponse};
use crate::v4::interaction::{InteractionMarkup, parse_plugin_config, V4Interaction};
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::V4InteractionType;

/// V4 HTTP Interaction Type
#[derive(Debug, Clone, Eq)]
pub struct SynchronousHttp {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider states for the interaction.
  /// See `<https://docs.pact.io/getting_started/provider_states>` for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// Request of the interaction
  pub request: HttpRequest,
  /// Response of the interaction
  pub response: HttpResponse,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool,

  /// Configuration added by plugins
  pub plugin_config: HashMap<String, HashMap<String, Value>>,

  /// Text markup to use to render the interaction in a UI
  pub interaction_markup: InteractionMarkup,

  /// Transport mechanism used with this request and response
  pub transport: Option<String>
}

impl SynchronousHttp {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> SynchronousHttp {
    SynchronousHttp {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Parse the JSON into a SynchronousHttp interaction
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<SynchronousHttp> {
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
      let request = json.get("request").cloned().unwrap_or_default();
      let response = json.get("response").cloned().unwrap_or_default();

      let plugin_config = parse_plugin_config(json);

      let interaction_markup = json.get("interactionMarkup")
        .map(|markup| InteractionMarkup::from_json(markup)).unwrap_or_default();

      let transport = json.get("transport").map(|value| {
        match value {
          Value::String(s) => s.clone(),
          _ => value.to_string()
        }
      });

      Ok(SynchronousHttp {
        id,
        key,
        description,
        provider_states,
        request: HttpRequest::from_json(&request)?,
        response: HttpResponse::from_json(&response)?,
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

impl V4Interaction for SynchronousHttp {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Synchronous_HTTP.to_string(),
      "description": self.description.clone(),
      "request": self.request.to_json(),
      "response": self.response.to_json(),
      "pending": self.pending
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
    V4InteractionType::Synchronous_HTTP
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

impl Interaction for SynchronousHttp {
  fn type_of(&self) -> String {
    format!("V4 {}", self.v4_type())
  }

  fn is_request_response(&self) -> bool {
    true
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    Some(RequestResponseInteraction {
      id: self.id.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.as_v3_request(),
      response: self.response.as_v3_response()
    })
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
    self.response.body.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.response.body.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.response.content_type()
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

  fn is_v4_http(&self) -> bool {
    true
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    Some(self.clone())
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    None
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    None
  }

  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp> {
    Some(self)
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
    None
  }

  fn pending(&self) -> bool {
    self.pending
  }
}

impl Default for SynchronousHttp {
  fn default() -> Self {
    SynchronousHttp {
      id: None,
      key: None,
      description: "Synchronous/HTTP Interaction".to_string(),
      provider_states: vec![],
      request: HttpRequest::default(),
      response: HttpResponse::default(),
      comments: Default::default(),
      pending: false,
      plugin_config: Default::default(),
      interaction_markup: Default::default(),
      transport: None
    }
  }
}

impl PartialEq for SynchronousHttp {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key &&
    self.description == other.description &&
    self.provider_states == other.provider_states &&
    self.request == other.request &&
    self.response == other.response &&
    self.pending == other.pending
  }
}

impl Hash for SynchronousHttp {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.request.hash(state);
    self.response.hash(state);
    self.pending.hash(state);
  }
}

impl Display for SynchronousHttp {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Http Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           pending, self.id, self.description, self.provider_states, self.request, self.response)
  }
}

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pretty_assertions::{assert_eq, assert_ne};
  use serde_json::json;

  use crate::bodies::OptionalBody;
  use crate::content_types::ContentType;
  use crate::prelude::ProviderState;
  use crate::v4::http_parts::{HttpRequest, HttpResponse};
  use crate::v4::interaction::V4Interaction;
  use crate::v4::synch_http::SynchronousHttp;

  #[test]
  fn calculate_hash_test() {
    let interaction = SynchronousHttp::from_json(&json!({
      "description": "a retrieve Mallory request",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "request": {
        "headers": {
          "content-type": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/mallory"
      },
      "response": {
        "body": {
          "content": "That is some good Mallory.",
          "contentType": "text/plain",
          "encoded": false
        },
        "headers": {
          "Content-Type": [
            "text/plain"
          ]
        },
        "status": 200
      },
      "transport": "http",
      "type": "Synchronous/HTTP"
    }), 0).unwrap();
    let hash = interaction.calc_hash();
    expect!(interaction.calc_hash()).to(be_equal_to(hash.as_str()));

    let interaction2 = interaction.with_key();
    expect!(interaction2.key.as_ref().unwrap()).to(be_equal_to(hash.as_str()));

    let json = interaction2.to_json();
    assert_eq!(json, json!({
      "description": "a retrieve Mallory request",
      "key": "93371e6e7ae2556",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "request": {
        "headers": {
          "content-type": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/mallory"
      },
      "response": {
        "body": {
          "content": "That is some good Mallory.",
          "contentType": "text/plain",
          "encoded": false
        },
        "headers": {
          "Content-Type": [
            "text/plain"
          ]
        },
        "status": 200
      },
      "transport": "http",
      "type": "Synchronous/HTTP"
    }));
  }

  #[test]
  fn hash_test() {
    let i1 = SynchronousHttp::default();
    expect!(i1.calc_hash()).to(be_equal_to("9cc3bdc81f4d6db3"));

    let i2 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      .. SynchronousHttp::default()
    };
    expect!(i2.calc_hash()).to(be_equal_to("54562b562c985411"));

    let i3 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. SynchronousHttp::default()
    };
    expect!(i3.calc_hash()).to(be_equal_to("c4e8b0f671fc7790"));

    let i4 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        .. HttpRequest::default()
      },
      .. SynchronousHttp::default()
    };
    expect!(i4.calc_hash()).to(be_equal_to("a24b3aa518050bda"));

    let i5 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      .. SynchronousHttp::default()
    };
    expect!(i5.calc_hash()).to(be_equal_to("f0e56013396eaf62"));

    let i5 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      response: HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "text/plain".to_string() ]  }),
        body: OptionalBody::from("That is some good Mallory."),
        .. HttpResponse::default()
      },
      .. SynchronousHttp::default()
    };
    expect!(i5.calc_hash()).to(be_equal_to("93371e6e7ae2556"));

    let i6 = SynchronousHttp::from_json(&json!({
      "description": "a retrieve Mallory request",
      "key": "c73355e81f04c03e",
      "pending": false,
      "providerStates": [
        {
          "name": "there is some good mallory"
        }
      ],
      "request": {
        "headers": {
          "content-type": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/mallory"
      },
      "response": {
        "body": {
          "content": "That is some good Mallory.",
          "contentType": "text/plain",
          "encoded": false
        },
        "headers": {
          "content-type": [
            "text/plain"
          ]
        },
        "status": 200
      },
      "transport": "http",
      "type": "Synchronous/HTTP"
    }), 0).unwrap();
    expect!(i6.key.as_ref()).to(be_some().value("c73355e81f04c03e"));
    expect!(i6.calc_hash()).to(be_equal_to("93371e6e7ae2556"));
    let i7 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap! { "content-type".to_string() => vec![ "application/json".to_string() ] }),
        .. HttpRequest::default()
      },
      response: HttpResponse {
        headers: Some(hashmap! { "content-type".to_string() => vec![ "text/plain".to_string() ] }),
        body: OptionalBody::Present(Bytes::from("That is some good Mallory."), Some(ContentType::from("*/*")), None),
        .. HttpResponse::default()
      },
      transport: Some("http".to_string()),
      .. SynchronousHttp::default()
    }.with_key();
    expect!(i7.key.as_ref()).to(be_some().value("93371e6e7ae2556"));
  }

  #[test]
  fn equals_test() {
    let i1 = SynchronousHttp::default();
    let i2 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      .. SynchronousHttp::default()
    };
    let i3 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      .. SynchronousHttp::default()
    };
    let i4 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        .. HttpRequest::default()
      },
      .. SynchronousHttp::default()
    };
    let i5 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      .. SynchronousHttp::default()
    };
    let i6 = SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      response: HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "text/plain".to_string() ]  }),
        body: OptionalBody::from("That is some good Mallory."),
        .. HttpResponse::default()
      },
      .. SynchronousHttp::default()
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
    let i1 = SynchronousHttp {
      key: Some("i1".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      response: HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "text/plain".to_string() ]  }),
        body: OptionalBody::from("That is some good Mallory."),
        .. HttpResponse::default()
      },
      .. SynchronousHttp::default()
    };
    let i2 = SynchronousHttp {
      key: Some("i2".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ]  }),
        .. HttpRequest::default()
      },
      response: HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "text/plain".to_string() ]  }),
        body: OptionalBody::from("That is some good Mallory."),
        .. HttpResponse::default()
      },
      .. SynchronousHttp::default()
    };

    assert_eq!(i1, i1);
    assert_eq!(i2, i2);

    assert_ne!(i1, i2);
    assert_ne!(i2, i1);
  }
}
