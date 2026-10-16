//! `provider_states` module contains all the logic for dealing with provider states.
//! See `https://docs.pact.io/getting_started/provider_states` for more info on provider states.

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use maplit::*;
use serde::{Deserialize, Serialize};
use serde_json::*;
use tracing::warn;

use crate::PactSpecification;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// Struct that encapsulates all the info about a provider state
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct ProviderState {
  /// Description of this provider state
  pub name: String,
  /// Provider state parameters as key value pairs
  pub params: HashMap<String, Value>
}

impl ProviderState {

  /// Creates a default state with the given name
  pub fn default<T: Into<String>>(name: T) -> ProviderState {
    ProviderState {
      name: name.into(),
      params: hashmap!{}
    }
  }

  /// Constructs a provider state from the `Json` struct
  pub fn from_json_v3(pact_json: &Value) -> ProviderState {
    let state = match pact_json.get("name") {
      Some(v) => match *v {
        Value::String(ref s) => s.clone(),
        _ => v.to_string()
      },
      None => {
        warn!("Provider state does not have a 'name' field");
        "unknown provider states".to_string()
      }
    };
    let params = match pact_json.get("params") {
      Some(v) => match *v {
        Value::Object(ref map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => {
          warn!("Provider state parameters must be a map");
          hashmap!{}
        }
      },
      None => hashmap!{}
    };
    ProviderState{
      name: state,
      params
    }
  }

  /// Constructs a list of provider states from the `Json` struct
  pub fn from_json(pact_json: &Value) -> Vec<ProviderState> {
    match pact_json.get("providerStates") {
      Some(v) => match *v {
        Value::Array(ref a) => a.iter().map(|i| ProviderState::from_json_v3(i)).collect(),
        _ => vec![]
      },
      None => match pact_json.get("providerState").or_else(|| pact_json.get("provider_state")) {
        Some(v) => match *v {
          Value::String(ref s) => if s.is_empty() {
            vec![]
          } else {
            vec![ProviderState{ name: s.clone(), params: hashmap!{} }]
          },
          Value::Null => vec![],
          _ => vec![ProviderState{ name: v.to_string(), params: hashmap!{} }]
        },
        None => vec![]
      }
    }
  }

  /// Converts this provider state into a JSON structure
  pub fn to_json(&self) -> Value {
    let mut value = json!({
            "name": Value::String(self.name.clone())
        });
    if !self.params.is_empty() {
      let map = value.as_object_mut().unwrap();
      map.insert("params".into(), Value::Object(
        self.params.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
    }
    value
  }

}

impl Hash for ProviderState {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    for (k, v) in self.params.clone() {
      k.hash(state);
      match v {
        Value::Number(n) => if n.is_u64() {
          n.as_u64().unwrap().hash(state)
        } else if n.is_i64() {
          n.as_i64().unwrap().hash(state)
        } else if n.is_f64() {
          n.as_f64().unwrap().to_string().hash(state)
        },
        Value::String(s) => s.hash(state),
        Value::Bool(b) => b.hash(state),
        _ => ()
      }
    }
  }
}

impl PartialEq for ProviderState {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.params == other.params
  }
}

impl PactJsonVerifier for ProviderState {
  fn verify_json(path: &str, pact_json: &Value, strict: bool, _spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::String(_) => {}
      Value::Object(values) => {
        match values.get("name") {
          None => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
            "Provider state 'name' is required")),
          Some(name) => if !name.is_string() {
            results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
              format!("Provider state 'name' must be a String, got {}", json_type_of(pact_json))))
          }
        }

        if let Some(params) = values.get("params") {
          if !params.is_object() {
            results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
              format!("Provider state 'params' must be an Object, got {}", json_type_of(pact_json))))
          }
        }

        let valid_attr = hashset! { "name", "params" };
        for key in values.keys() {
          if !valid_attr.contains(key.as_str()) {
            results.push(PactFileVerificationResult::new(path.to_owned(),
              if strict { ResultLevel::ERROR } else { ResultLevel::WARNING }, format!("Unknown attribute '{}'", key)))
          }
        }
      }
      _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
        format!("Must be a String or Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use serde_json;
  use serde_json::Value;

  use super::*;

  #[test]
  fn defaults_to_v3_pact_provider_states() {
    let json = r#"{
            "providerStates": [
              {
                "name": "test state",
                "params": { "name": "Testy" }
              },
              {
                "name": "test state 2",
                "params": { "name": "Testy2" }
              }
            ],
            "description" : "test interaction"
        }"#;
    let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
    expect!(provider_states.iter()).to(have_count(2));
    expect!(&provider_states[0]).to(be_equal_to(&ProviderState {
      name: "test state".into(),
      params: hashmap!{ "name".to_string() => Value::String("Testy".into()) }
    }));
    expect!(&provider_states[1]).to(be_equal_to(&ProviderState {
      name: "test state 2".into(),
      params: hashmap!{ "name".to_string() => Value::String("Testy2".into()) }
    }));
  }

  #[test]
  fn falls_back_to_v2_pact_provider_state() {
    let json = r#"{
            "providerState": "test state",
            "description" : "test interaction"
        }"#;
    let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
    expect!(provider_states.iter()).to(have_count(1));
    expect!(&provider_states[0]).to(be_equal_to(&ProviderState {
      name: "test state".to_string(),
      params: hashmap!{}
    }));
  }

  #[test]
  fn pact_with_no_provider_states() {
    let json = r#"{
            "description" : "test interaction"
        }"#;
    let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
    expect!(provider_states.iter()).to(be_empty());
  }
}
