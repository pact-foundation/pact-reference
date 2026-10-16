//! Module for verifying the state of the Pact JSON (file format verification)

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{fmt, PactSpecification};

/// Level of the result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResultLevel {
  /// A validation error
  ERROR,
  /// Validation warning
  WARNING,
  /// Just a note to display
  NOTICE
}

impl Display for ResultLevel {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      ResultLevel::ERROR => write!(f, "ERROR"),
      ResultLevel::WARNING => write!(f, "WARNING"),
      ResultLevel::NOTICE => write!(f, "NOTICE")
    }
  }
}

/// Single verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactFileVerificationResult {
  /// Path into the JSON
  pub path: String,
  /// Level if the result
  pub level: ResultLevel,
  /// Message associated with the result
  pub message: String
}

impl PactFileVerificationResult {
  /// Create a new result
  pub fn new<P: Into<String>, L: Into<ResultLevel>, M: Into<String>>(path: P, level: L, message: M) -> Self {
    PactFileVerificationResult {
      path: path.into(),
      level: level.into(),
      message: message.into()
    }
  }
}

/// Trait for Pact JSON file format verifiers
pub trait PactJsonVerifier {
  /// Verify the JSON format. Will return an error if the list contains any Error result
  fn verify_json(path: &str, pact_json: &Value, strict: bool, spec_version: PactSpecification) -> Vec<PactFileVerificationResult>;
}

/// Type of the JSON element
pub fn json_type_of(value: &Value) -> String {
  match value {
    Value::Null => "Null",
    Value::Bool(_) => "Bool",
    Value::Number(_) => "Number",
    Value::String(_) => "String",
    Value::Array(_) => "Array",
    Value::Object(_) => "Object"
  }.to_string()
}
