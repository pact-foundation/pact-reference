//! Structs for storing and returning the result of the verification execution

use std::collections::HashMap;
use std::time::Duration;

use itertools::Itertools;
use serde_json::{json, Map, Value};

use pact_matching::Mismatch;

/// Result of verifying a Pact interaction
#[derive(Clone, Debug)]
pub struct VerificationInteractionResult {
  /// Interaction ID, this will only be set if the Pact was loaded from a Pact broker
  pub interaction_id: Option<String>,
  /// Interaction key (this will be set if the Pact is a V4 Pact)
  pub interaction_key: Option<String>,
  /// Name of the consumer of the Pact the interaction belongs to
  pub consumer: String,
  /// Name of the provider of the Pact the interaction belongs to
  pub provider: String,
  /// Descriptive text of the verification that was preformed
  pub description: String,
  /// Interaction description from the Pact file
  pub interaction_description: String,
  /// Names of the provider states of the interaction, in the order they are defined in the Pact
  pub provider_states: Vec<String>,
  /// Result of the verification
  pub result: Result<(), crate::MismatchResult>,
  /// If the Pact or interaction is pending
  pub pending: bool,
  /// Duration that the verification took
  pub duration: Duration
}

/// Result of verifying a Pact
#[derive(Clone, Debug)]
pub struct VerificationResult {
  /// Results that occurred
  pub results: Vec<VerificationInteractionResult>,
  /// Output from the verification
  pub output: Vec<String>
}

/// Main struct for returning the total verification execution result
#[derive(Debug, Clone, Default)]
pub struct VerificationExecutionResult {
  /// Overall pass/fail result
  pub result: bool,
  /// Notices provided by the Pact Broker
  pub notices: Vec<HashMap<String, String>>,
  /// Collected standard output
  pub output: Vec<String>,
  /// Errors that occurred, but are marked as pending
  pub pending_errors: Vec<(String, VerificationMismatchResult)>,
  /// Errors that occurred that are not considered pending
  pub errors: Vec<(String, VerificationMismatchResult)>,
  /// Result for each interaction that was verified
  pub interaction_results: Vec<VerificationInteractionResult>
}

impl VerificationExecutionResult {
  /// Create a new VerificationExecutionResult with default values
  pub fn new() -> Self {
    VerificationExecutionResult {
      result: true,
      notices: vec![],
      output: vec![],
      pending_errors: vec![],
      errors: vec![],
      interaction_results: vec![],
    }
  }
}

impl Into<Value> for &VerificationExecutionResult {
  fn into(self) -> Value {
    json!({
      "result": self.result,
      "notices": self.notices.iter().map(|m| Value::Object(
        m.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect()
      )).collect_vec(),
      "output": self.output,
      "pendingErrors": self.pending_errors.iter().map(|(e, r)| {
        let err: Value = r.into();
        json!({
          "interaction": e,
          "mismatch": err
        })
      }).collect_vec(),
      "errors": self.errors.iter().map(|(e, r)| {
        let err: Value = r.into();
        json!({
          "interaction": e,
          "mismatch": err
        })
      }).collect_vec(),
      "interactionResults": self.interaction_results.iter().map(|r| {
        let mut attributes = Map::new();
        if let Some(interaction_id) = &r.interaction_id {
          attributes.insert("interactionId".to_string(), Value::String(interaction_id.clone()));
        }
        if let Some(interaction_key) = &r.interaction_key {
          attributes.insert("interactionKey".to_string(), Value::String(interaction_key.clone()));
        }
        attributes.insert("consumer".to_string(), Value::String(r.consumer.clone()));
        attributes.insert("provider".to_string(), Value::String(r.provider.clone()));
        attributes.insert("description".to_string(), Value::String(r.interaction_description.clone()));
        attributes.insert("providerStates".to_string(), Value::Array(
          r.provider_states.iter().map(|s| Value::String(s.clone())).collect()
        ));
        attributes.insert("pending".to_string(), Value::Bool(r.pending));
        match &r.result {
          Ok(_) => {
            attributes.insert("result".to_string(), Value::String("OK".to_string()));
          }
          Err(err) => {
            attributes.insert("result".to_string(), Value::String("Error".to_string()));
            let mismatch: Value = (&VerificationMismatchResult::from(err)).into();
            attributes.insert("mismatch".to_string(), mismatch);
          }
        };
        attributes.insert("duration".to_string(), Value::String(format!("{:?}", r.duration)));
        Value::Object(attributes)
      }).collect_vec()
    })
  }
}

impl Into<Value> for VerificationExecutionResult {
  fn into(self) -> Value {
    (&self).into()
  }
}

/// Result of performing a match. This is a reduced version of crate::MismatchResult to make
/// it thread and panic boundary safe
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum VerificationMismatchResult {
  /// Response mismatches
  Mismatches {
    /// Mismatches that occurred
    mismatches: Vec<Mismatch>,
    /// Interaction ID if fetched from a pact broker
    interaction_id: Option<String>
  },
  /// Error occurred
  Error {
    /// Error that occurred
    error: String,
    /// Interaction ID if fetched from a pact broker
    interaction_id: Option<String>
  }
}

impl From<&crate::MismatchResult> for VerificationMismatchResult {
  fn from(result: &crate::MismatchResult) -> Self {
    match result {
      crate::MismatchResult::Mismatches { mismatches, interaction_id, .. } => {
        VerificationMismatchResult::Mismatches {
          mismatches: mismatches.clone(),
          interaction_id: interaction_id.clone()
        }
      }
      crate::MismatchResult::Error(error, interaction_id) => {
        VerificationMismatchResult::Error {
          error: error.clone(),
          interaction_id: interaction_id.clone()
        }
      }
    }
  }
}

impl Into<Value> for &VerificationMismatchResult {
  fn into(self) -> Value {
    match self {
      VerificationMismatchResult::Mismatches { mismatches, interaction_id } => {
        json!({
          "type": "mismatches",
          "mismatches": mismatches.iter().map(|i| i.to_json()).collect_vec(),
          "interactionId": interaction_id.clone().unwrap_or_default()
        })
      }
      VerificationMismatchResult::Error { error, interaction_id } => {
        json!({
          "type": "error",
          "message": error,
          "interactionId": interaction_id.clone().unwrap_or_default()
        })
      }
    }
  }
}

impl Into<Value> for VerificationMismatchResult {
  fn into(self) -> Value {
    (&self).into()
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pretty_assertions::assert_eq;
  use serde_json::{json, Value};

  use pact_matching::Mismatch;
  use pact_models::sync_interaction::RequestResponseInteraction;

  use crate::{MismatchResult, VerificationExecutionResult};
  use crate::verification_result::{VerificationInteractionResult, VerificationMismatchResult};

  #[test]
  fn match_result_to_json() {
    let mismatch = VerificationMismatchResult::Mismatches {
      mismatches: vec![
        Mismatch::BodyMismatch {
          path: "1.2.3.4".to_string(),
          expected: Some("100".into()),
          actual: Some("200".into()),
          mismatch: "Expected 100 but got 200".to_string()
        }
      ],
      interaction_id: None
    };
    let json: Value = mismatch.into();
    expect!(json).to(be_equal_to(json!({
      "interactionId": "",
      "mismatches": [
        {
          "actual": "200",
          "expected": "100",
          "mismatch": "Expected 100 but got 200",
          "path": "1.2.3.4",
          "type": "BodyMismatch"
        }
      ],
      "type": "mismatches"
    })));

    let error = VerificationMismatchResult::Error {
      error: "It went bang, Mate!".to_string(),
      interaction_id: Some("1234".to_string())
    };
    let json: Value = error.into();
    expect!(json).to(be_equal_to(json!({
      "interactionId": "1234",
      "message": "It went bang, Mate!",
      "type": "error"
    })));
  }

  #[test]
  fn verification_execution_result_to_json() {
    let result = VerificationExecutionResult {
      result: false,
      notices: vec![
        hashmap!{
          "comment".to_string() => "This is a comment".to_string()
        }
      ],
      output: vec![
        "line 1".to_string(),
        "line 2".to_string(),
        "line 3".to_string(),
        "line 4".to_string()
      ],
      pending_errors: vec![
        (
          "interaction 1".to_string(),
          VerificationMismatchResult::Error {
            error: "Boom!".to_string(),
            interaction_id: None
          }
        )
      ],
      errors: vec![
        (
          "interaction 2".to_string(),
          VerificationMismatchResult::Error {
            error: "Boom!".to_string(),
            interaction_id: None
          }
        )
      ],
      interaction_results: vec![],
    };

    let json: Value = result.into();

    assert_eq!(json, json!({
      "errors": [
        {
          "interaction": "interaction 2".to_string(),
          "mismatch": {
            "interactionId": "".to_string(),
            "message": "Boom!".to_string(),
            "type": "error".to_string()
          }
        }
      ],
      "interactionResults": [],
      "notices": [
        {
          "comment": "This is a comment".to_string()
        }
      ],
      "output": [
        "line 1".to_string(),
        "line 2".to_string(),
        "line 3".to_string(),
        "line 4".to_string()
      ],
      "pendingErrors": [
        {
          "interaction": "interaction 1".to_string(),
          "mismatch": {
            "interactionId": "".to_string(),
            "message": "Boom!".to_string(),
            "type": "error".to_string()
          }
        }
      ],
      "result": false
    }));
  }

  #[test]
  fn verification_execution_result_to_json_includes_interaction_details() {
    let result = VerificationExecutionResult {
      interaction_results: vec![
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: None,
          consumer: "Consumer A".to_string(),
          provider: "Provider".to_string(),
          provider_states: vec![],
          description: "".to_string(),
          interaction_description: "result-1".to_string(),
          result: Ok(()),
          pending: false,
          duration: Default::default(),
        },
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: None,
          consumer: "Consumer A".to_string(),
          provider: "Provider".to_string(),
          provider_states: vec![],
          description: "".to_string(),
          interaction_description: "result-2".to_string(),
          result: Err(MismatchResult::Error("test".to_string(), None)),
          pending: false,
          duration: Default::default(),
        },
        VerificationInteractionResult {
          interaction_id: Some("test-id".to_string()),
          interaction_key: None,
          consumer: "Consumer B".to_string(),
          provider: "Provider".to_string(),
          provider_states: vec!["state 1".to_string(), "state 2".to_string()],
          description: "".to_string(),
          interaction_description: "result-3".to_string(),
          result: Ok(()),
          pending: true,
          duration: Default::default(),
        },
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: Some("test-key".to_string()),
          consumer: "Consumer B".to_string(),
          provider: "Provider".to_string(),
          provider_states: vec![],
          description: "".to_string(),
          interaction_description: "result-4".to_string(),
          result: Err(MismatchResult::Mismatches {
            mismatches: vec![
              Mismatch::StatusMismatch {
                expected: 200,
                actual: 500,
                mismatch: "expected 200 but was 500".to_string()
              }
            ],
            expected: Box::new(RequestResponseInteraction::default()),
            actual: Box::new(RequestResponseInteraction::default()),
            interaction_id: None
          }),
          pending: false,
          duration: Default::default(),
        }
      ],
      .. VerificationExecutionResult::default()
    };

    let json: Value = result.into();

    assert_eq!(json!({
      "errors": [],
      "interactionResults": [
        {
          "consumer": "Consumer A",
          "description": "result-1",
          "duration": "0ns",
          "pending": false,
          "provider": "Provider",
          "providerStates": [],
          "result": "OK",
        },
        {
          "consumer": "Consumer A",
          "description": "result-2",
          "duration": "0ns",
          "mismatch": {
            "interactionId": "",
            "message": "test",
            "type": "error"
          },
          "pending": false,
          "provider": "Provider",
          "providerStates": [],
          "result": "Error",
        },
        {
          "consumer": "Consumer B",
          "description": "result-3",
          "duration": "0ns",
          "interactionId": "test-id",
          "pending": true,
          "provider": "Provider",
          "providerStates": ["state 1", "state 2"],
          "result": "OK",
        },
        {
          "consumer": "Consumer B",
          "description": "result-4",
          "duration": "0ns",
          "interactionKey": "test-key",
          "mismatch": {
            "interactionId": "",
            "mismatches": [
              {
                "actual": 500,
                "expected": 200,
                "mismatch": "expected 200 but was 500",
                "type": "StatusMismatch"
              }
            ],
            "type": "mismatches"
          },
          "pending": false,
          "provider": "Provider",
          "providerStates": [],
          "result": "Error",
        }
      ],
      "notices": [],
      "output": [],
      "pendingErrors": [],
      "result": false
    }), json);
  }
}
