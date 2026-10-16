//! Models for synchronous request/response interactions

use std::fmt::{Display, Formatter};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use maplit::hashset;
use serde_json::{json, Value};

use crate::{DifferenceType, PactSpecification};
use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::http_parts::HttpPart;
use crate::interaction::{Interaction, PactConflict};
use crate::json_utils::json_to_string;
use crate::matchingrules::MatchingRules;
use crate::message::Message;
use crate::provider_states::ProviderState;
use crate::request::Request;
use crate::response::Response;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::interaction::V4Interaction;
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// Struct that defines an interaction (request and response pair)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestResponseInteraction {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Description of this interaction. This needs to be unique in the pact file.
  pub description: String,
  /// Optional provider states for the interaction.
  /// See `<https://docs.pact.io/getting_started/provider_states>` for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// Request of the interaction
  pub request: Request,
  /// Response of the interaction
  pub response: Response
}

impl Interaction for RequestResponseInteraction {
  fn type_of(&self) -> String {
    "V3 Synchronous/HTTP".into()
  }

  fn is_request_response(&self) -> bool {
    true
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    Some(self.clone())
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
    false
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>> {
    self.as_v4_http().map(|i| i.boxed_v4())
  }

  fn as_v4_mut(&mut self) -> Option<&mut dyn V4Interaction> {
    None
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    Some(SynchronousHttp {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.as_v4_request(),
      response: self.response.as_v4_response(),
      .. Default::default()
    })
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    None
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
    None
  }
}

impl RequestResponseInteraction {
  /// Constructs an `Interaction` from the `Value` struct.
  pub fn from_json(index: usize, pact_json: &Value, spec_version: &PactSpecification
  ) -> anyhow::Result<RequestResponseInteraction> {
    let id = pact_json.get("_id").map(|id| json_to_string(id));
    let description = match pact_json.get("description") {
      Some(v) => match *v {
        Value::String(ref s) => s.clone(),
        _ => v.to_string()
      },
      None => format!("Interaction {}", index)
    };
    let provider_states = ProviderState::from_json(pact_json);
    let request = match pact_json.get("request") {
      Some(v) => Request::from_json(v, spec_version)?,
      None => Request::default()
    };
    let response = match pact_json.get("response") {
      Some(v) => Response::from_json(v, spec_version)?,
      None => Response::default()
    };
    Ok(RequestResponseInteraction {
      id,
      description,
      provider_states,
      request,
      response,
    })
  }

  /// Converts this interaction to a `Value` struct.
  pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
    let mut value = json!({
            "description".to_string(): Value::String(self.description.clone()),
            "request".to_string(): self.request.to_json(spec_version),
            "response".to_string(): self.response.to_json(spec_version)
        });
    if !self.provider_states.is_empty() {
      let map = value.as_object_mut().unwrap();
      match spec_version {
        PactSpecification::V3 => map.insert("providerStates".to_string(),
                                             Value::Array(self.provider_states.iter().map(|p| p.to_json()).collect())),
        _ => map.insert("providerState".to_string(), Value::String(
          self.provider_states.first().unwrap().name.clone()))
      };
    }
    value
  }

  /// Returns list of conflicts if this interaction conflicts with the other interaction.
  ///
  /// Two interactions conflict if they have the same description and provider state, but they request and
  /// responses are not equal
  pub fn conflicts_with(&self, other: &dyn Interaction) -> Vec<PactConflict> {
    if let Some(other) = other.as_request_response() {
      if self.description == other.description && self.provider_states == other.provider_states {
        let mut conflicts = self.request.differences_from(&other.request).iter()
          .filter(|difference| match difference.0 {
            DifferenceType::MatchingRules | DifferenceType::Body | DifferenceType::Headers => false,
            _ => true
          })
          .map(|difference| PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
          .collect::<Vec<PactConflict>>();
        for difference in self.response.differences_from(&other.response) {
          match difference.0 {
            DifferenceType::MatchingRules | DifferenceType::Body | DifferenceType::Headers => (),
            _ => conflicts.push(PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
          };
        }
        conflicts
      } else {
        vec![]
      }
    } else {
      vec![PactConflict {
        interaction: self.description.clone(),
        description: format!("You can not combine message and request/response interactions")
      }]
    }
  }

  /// Generate the JSON schema properties for the given Pact specification
  pub fn schema(_spec_version: PactSpecification) -> Value {
    json!({})
  }
}

impl Default for RequestResponseInteraction {
  fn default() -> Self {
    RequestResponseInteraction {
      id: None,
      description: "Default Interaction".to_string(),
      provider_states: vec![],
      request: Request::default(),
      response: Response::default()
    }
  }
}

impl Display for RequestResponseInteraction {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Interaction ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           self.id, self.description, self.provider_states, self.request, self.response)
  }
}

impl PactJsonVerifier for RequestResponseInteraction {
  fn verify_json(path: &str, pact_json: &Value, strict: bool, spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(description) = values.get("description") {
          if !description.is_string() {
            results.push(PactFileVerificationResult::new(path.to_owned() + "/description", ResultLevel::ERROR,
                                                         format!("Must be a String, got {}", json_type_of(pact_json))))
          }
        } else {
          results.push(PactFileVerificationResult::new(path,
                                                       if strict { ResultLevel::ERROR } else { ResultLevel::WARNING }, "Missing description"))
        }

        let provider_states = if values.contains_key("providerStates") {
          values.get("providerStates").unwrap().clone()
        } else if values.contains_key("providerState") {
          if spec_version >= PactSpecification::V3 {
            results.push(PactFileVerificationResult::new(path, ResultLevel::WARNING,
                                                         format!("'providerState' is deprecated, use 'providerStates' instead")))
          }
          Value::Array(vec![ values.get("providerState").unwrap().clone() ])
        } else if values.contains_key("provider_state") {
          results.push(PactFileVerificationResult::new(path, ResultLevel::WARNING,
                                                       format!("'provider_state' is deprecated, use 'providerStates' instead")));
          Value::Array(vec![ values.get("provider_state").unwrap().clone() ])
        } else {
          Value::Array(vec![])
        };

        match provider_states {
          Value::Array(states) => {
            results.extend(states.iter().enumerate()
              .flat_map(|(index, state)| {
                ProviderState::verify_json(&*format!("{}/providerStates/{}", path, index), state, strict, spec_version)
              }))
          }
          _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
                                                            format!("'providerStates' must be an Array, got {}", json_type_of(&provider_states))))
        }

        let valid_attr = hashset! {
          "_id", "description", "providerState", "provider_state", "providerStates", "request",
          "response" };
        for (key, _) in values {
          if !valid_attr.contains(key.as_str()) {
            results.push(PactFileVerificationResult::new(path,
                                                         if strict { ResultLevel::ERROR } else { ResultLevel::WARNING },
                                                         &format!("Unexpected attribute '{}'", key)));
          }
        }
      }
      _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
                                                        format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::bodies::OptionalBody;
  use crate::generators::{GeneratorCategory, Generators, Generator};
  use crate::matchingrules::{Category, MatchingRule, MatchingRuleCategory, MatchingRules, RuleLogic, RuleList};
  use crate::PactSpecification;
  use crate::provider_states::ProviderState;
  use crate::request::Request;
  use crate::response::Response;
  use crate::sync_interaction::RequestResponseInteraction;
  use crate::path_exp::DocPath;

  #[test]
  fn loading_interaction_from_json() {
    let interaction_json = r#"{
        "description": "String",
        "providerState": "provider state"
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1_1);
    let interaction = interaction.unwrap();

    expect!(interaction.description).to(be_equal_to("String"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
      ProviderState { name: "provider state".to_string(), params: hashmap!{} } ]));
  }

  #[test]
  fn defaults_to_number_if_no_description() {
    let interaction_json = r#"{
        "providerState": "provider state"
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1_1);
    let interaction = interaction.unwrap();

    expect!(interaction.description).to(be_equal_to("Interaction 0"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
      ProviderState { name: "provider state".into(), params: hashmap!{} } ]));
  }

  #[test]
  fn defaults_to_empty_if_no_provider_state() {
    let interaction_json = r#"{
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1);
    let interaction = interaction.unwrap();

    expect!(interaction.provider_states.iter()).to(be_empty());
  }

  #[test]
  fn defaults_to_none_if_provider_state_null() {
    let interaction_json = r#"{
        "providerState": null
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1);
    let interaction = interaction.unwrap();

    expect!(interaction.provider_states.iter()).to(be_empty());
  }

  #[test]
  fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
    let json = json!({
      "_id": "123456789",
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {
        "method": "GET",
        "path": "/"
      },
      "response": {
        "status": 200
      }
    });
    let interaction =
      RequestResponseInteraction::from_json(0, &json, &PactSpecification::V3);
    let interaction = interaction.unwrap();

    expect!(interaction.id).to(be_some().value("123456789".to_string()));
  }

  #[test]
  fn interactions_do_not_conflict_if_they_have_different_descriptions() {
    let interaction1 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
      description: "Test Interaction 2".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  }

  #[test]
  fn interactions_do_not_conflict_if_they_have_different_provider_states() {
    let interaction1 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Bad state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  }

  #[test]
  fn interactions_do_not_conflict_if_they_have_the_same_requests_and_responses() {
    let interaction1 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  }

  #[test]
  fn interactions_conflict_if_they_have_different_requests() {
    let interaction1 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      request: Request { method: "POST".to_string(), .. Request::default() },
      .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  }

  #[test]
  fn interactions_conflict_if_they_have_different_responses() {
    let interaction1 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
      description: "Test Interaction".to_string(),
      provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
      response: Response { status: 400, .. Response::default() },
      .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  }

  // Issue https://github.com/pact-foundation/pact-js-core/issues/400
  #[test]
  fn to_json_with_provider_state_generator_test() {
    let interaction = RequestResponseInteraction {
      description: "a request to get the plain data".to_string(),
      provider_states: vec![
        ProviderState {
          name: "set id".to_string(),
          params: hashmap!{ "id".to_string() => json!("42")}
        }
      ],
      request: Request {
        method: "GET".to_string(),
        path: "/data/42".to_string(),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::PATH => MatchingRuleCategory {
              name: Category::PATH,
              rules: hashmap!{ DocPath::root() => RuleList {
                rules: vec![MatchingRule::Type],
                rule_logic: RuleLogic::And,
                cascaded: false
              }}
            }
          }
        },
        generators: Generators {
          categories: hashmap!{
            GeneratorCategory::PATH => hashmap!{
              DocPath::root() => Generator::ProviderStateGenerator("/data/${id}".to_string(), None)
            }
          }
        },
        .. Request::default()
      },
      response: Response {
        status: 200,
        headers: Some(hashmap!{"Content-Type".to_string() => vec!["text/plain; charset=utf-8".to_string()]}),
        body: OptionalBody::from("data: testData, id: 42"),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::HEADER => MatchingRuleCategory {
              name: Category::HEADER,
              rules: hashmap!{}
            }
          }
        },
        generators: Generators { categories: hashmap!{} }
      },
      .. RequestResponseInteraction::default()
    };

    let json = interaction.to_json(&PactSpecification::V3);
    let request = json.get("request").unwrap();
    let generators = request.get("generators").unwrap();
    expect!(generators.to_string()).to_not(be_equal_to("{}"));
  }
}
