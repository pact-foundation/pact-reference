use quick_xml::se::to_utf8_io_writer;
use serde::Serialize;
use pact_matching::Mismatch;
use pact_verifier::verification_result::{VerificationExecutionResult, VerificationMismatchResult};

// XML-specific DTOs — quick_xml's serde serializer does not support enum struct variants,
// so we flatten VerificationMismatchResult and Mismatch into plain structs here.

#[derive(Debug, Serialize)]
#[serde(rename = "report")]
struct XmlReport {
  provider: String,
  result: bool,
  #[serde(skip_serializing_if = "XmlNotices::is_empty")]
  notices: XmlNotices,
  #[serde(skip_serializing_if = "XmlErrors::is_empty")]
  errors: XmlErrors,
  #[serde(skip_serializing_if = "XmlErrors::is_empty")]
  pending_errors: XmlErrors,
  #[serde(skip_serializing_if = "XmlInteractionResults::is_empty")]
  interaction_results: XmlInteractionResults,
}

#[derive(Debug, Default, Serialize)]
struct XmlNotices {
  #[serde(rename = "notice")]
  items: Vec<XmlNotice>,
}

impl XmlNotices {
  fn is_empty(&self) -> bool { self.items.is_empty() }
}

#[derive(Debug, Serialize)]
struct XmlNotice {
  #[serde(rename = "entry")]
  entries: Vec<XmlNoticeEntry>,
}

#[derive(Debug, Serialize)]
struct XmlNoticeEntry {
  key: String,
  value: String,
}

#[derive(Debug, Default, Serialize)]
struct XmlErrors {
  #[serde(rename = "error")]
  items: Vec<XmlError>,
}

impl XmlErrors {
  fn is_empty(&self) -> bool { self.items.is_empty() }
}

#[derive(Debug, Serialize)]
struct XmlError {
  interaction: String,
  mismatch: XmlMismatchResult,
}

#[derive(Debug, Serialize)]
struct XmlMismatchResult {
  #[serde(rename = "@type")]
  kind: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  error_message: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  interaction_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  mismatches: Option<XmlMismatchList>,
}

#[derive(Debug, Serialize)]
struct XmlMismatchList {
  #[serde(rename = "mismatch")]
  items: Vec<XmlMismatch>,
}

#[derive(Debug, Serialize)]
struct XmlMismatch {
  #[serde(rename = "@type")]
  kind: String,
  description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  expected: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  actual: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  path: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  key: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  parameter: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct XmlInteractionResults {
  #[serde(rename = "interaction")]
  items: Vec<XmlInteractionResult>,
}

impl XmlInteractionResults {
  fn is_empty(&self) -> bool { self.items.is_empty() }
}

#[derive(Debug, Serialize)]
struct XmlInteractionResult {
  description: String,
  result: String,
  pending: bool,
  duration_ms: u128,
}

impl From<(&str, &VerificationExecutionResult)> for XmlReport {
  fn from((provider, exec_result): (&str, &VerificationExecutionResult)) -> Self {
    XmlReport {
      provider: provider.to_string(),
      result: exec_result.result,
      notices: XmlNotices {
        items: exec_result.notices.iter().map(|n| {
          let mut entries: Vec<XmlNoticeEntry> = n.iter().map(|(k, v)| XmlNoticeEntry {
            key: k.clone(),
            value: v.clone(),
          }).collect();
          entries.sort_by(|a, b| a.key.cmp(&b.key));
          XmlNotice { entries }
        }).collect()
      },
      errors: XmlErrors {
        items: exec_result.errors.iter().map(|(interaction, mismatch)| XmlError {
          interaction: interaction.clone(),
          mismatch: XmlMismatchResult::from(mismatch),
        }).collect()
      },
      pending_errors: XmlErrors {
        items: exec_result.pending_errors.iter().map(|(interaction, mismatch)| XmlError {
          interaction: interaction.clone(),
          mismatch: XmlMismatchResult::from(mismatch),
        }).collect()
      },
      interaction_results: XmlInteractionResults {
        items: exec_result.interaction_results.iter().map(|ir| XmlInteractionResult {
          description: ir.interaction_description.clone(),
          result: if ir.result.is_ok() { "OK".to_string() } else { "Error".to_string() },
          pending: ir.pending,
          duration_ms: ir.duration.as_millis(),
        }).collect()
      },
    }
  }
}

impl From<&VerificationMismatchResult> for XmlMismatchResult {
  fn from(result: &VerificationMismatchResult) -> Self {
    match result {
      VerificationMismatchResult::Mismatches { mismatches, interaction_id } => XmlMismatchResult {
        kind: "mismatches".to_string(),
        error_message: None,
        interaction_id: interaction_id.clone(),
        mismatches: Some(XmlMismatchList {
          items: mismatches.iter().map(XmlMismatch::from).collect(),
        }),
      },
      VerificationMismatchResult::Error { error, interaction_id } => XmlMismatchResult {
        kind: "error".to_string(),
        error_message: Some(error.clone()),
        interaction_id: interaction_id.clone(),
        mismatches: None,
      },
    }
  }
}

impl From<&Mismatch> for XmlMismatch {
  fn from(m: &Mismatch) -> Self {
    match m {
      Mismatch::MethodMismatch { expected, actual, mismatch } => XmlMismatch {
        kind: "MethodMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None, key: None, parameter: None,
      },
      Mismatch::PathMismatch { expected, actual, mismatch } => XmlMismatch {
        kind: "PathMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None, key: None, parameter: None,
      },
      Mismatch::StatusMismatch { expected, actual, mismatch } => XmlMismatch {
        kind: "StatusMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.to_string()),
        actual: Some(actual.to_string()),
        path: None, key: None, parameter: None,
      },
      Mismatch::QueryMismatch { parameter, expected, actual, mismatch } => XmlMismatch {
        kind: "QueryMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None, key: None,
        parameter: Some(parameter.clone()),
      },
      Mismatch::HeaderMismatch { key, expected, actual, mismatch } => XmlMismatch {
        kind: "HeaderMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None,
        key: Some(key.clone()),
        parameter: None,
      },
      Mismatch::BodyTypeMismatch { expected, actual, mismatch, .. } => XmlMismatch {
        kind: "BodyTypeMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None, key: None, parameter: None,
      },
      Mismatch::BodyMismatch { path, expected, actual, mismatch } => XmlMismatch {
        kind: "BodyMismatch".to_string(),
        description: mismatch.clone(),
        expected: expected.as_ref().map(|b| String::from_utf8_lossy(b).into_owned()),
        actual: actual.as_ref().map(|b| String::from_utf8_lossy(b).into_owned()),
        path: Some(path.clone()),
        key: None, parameter: None,
      },
      Mismatch::MetadataMismatch { key, expected, actual, mismatch } => XmlMismatch {
        kind: "MetadataMismatch".to_string(),
        description: mismatch.clone(),
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        path: None,
        key: Some(key.clone()),
        parameter: None,
      },
    }
  }
}

pub(crate) fn to_xml_string(
  result: &VerificationExecutionResult,
  provider: &str
) -> anyhow::Result<String> {
  let report = XmlReport::from((provider, result));
  let mut buf = Vec::new();
  to_utf8_io_writer(&mut buf, &report)?;
  Ok(String::from_utf8(buf)?)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::time::Duration;

  use insta::assert_snapshot;
  use pact_matching::Mismatch;
  use pact_verifier::MismatchResult;
  use pact_verifier::verification_result::{
    VerificationExecutionResult, VerificationInteractionResult, VerificationMismatchResult,
  };

  use super::*;

  fn to_xml(result: &VerificationExecutionResult, provider: &str) -> String {
    let report = XmlReport::from((provider, result));
    let mut buf = Vec::new();
    to_utf8_io_writer(&mut buf, &report).unwrap();
    String::from_utf8(buf).unwrap()
  }

  #[test]
  fn empty_result() {
    assert_snapshot!(to_xml(&VerificationExecutionResult::new(), "My Provider"));
  }

  #[test]
  fn passing_result_with_interactions() {
    let result = VerificationExecutionResult {
      result: true,
      interaction_results: vec![
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: None,
          description: "request 1".to_string(),
          interaction_description: "GET /foo returns 200".to_string(),
          result: Ok(()),
          pending: false,
          duration: Duration::from_millis(42),
        },
        VerificationInteractionResult {
          interaction_id: Some("interaction-123".to_string()),
          interaction_key: Some("key-abc".to_string()),
          description: "request 2".to_string(),
          interaction_description: "POST /bar returns 201".to_string(),
          result: Ok(()),
          pending: false,
          duration: Duration::from_millis(15),
        },
      ],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }

  #[test]
  fn failing_with_error_mismatch() {
    let result = VerificationExecutionResult {
      result: false,
      errors: vec![
        (
          "GET /foo returns 200".to_string(),
          VerificationMismatchResult::Error {
            error: "Connection refused".to_string(),
            interaction_id: Some("abc123".to_string()),
          },
        ),
      ],
      interaction_results: vec![
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: None,
          description: "request 1".to_string(),
          interaction_description: "GET /foo returns 200".to_string(),
          result: Err(MismatchResult::Error("Connection refused".to_string(), None)),
          pending: false,
          duration: Duration::from_millis(5),
        },
      ],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }

  #[test]
  fn failing_with_body_mismatch() {
    let result = VerificationExecutionResult {
      result: false,
      errors: vec![
        (
          "GET /foo returns 200".to_string(),
          VerificationMismatchResult::Mismatches {
            mismatches: vec![
              Mismatch::StatusMismatch {
                expected: 200,
                actual: 404,
                mismatch: "Expected status 200 but was 404".to_string(),
              },
              Mismatch::BodyMismatch {
                path: "$.price".to_string(),
                expected: Some("100".into()),
                actual: Some("200".into()),
                mismatch: "Expected 100 but got 200".to_string(),
              },
            ],
            interaction_id: None,
          },
        ),
      ],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }

  #[test]
  fn pending_errors() {
    let result = VerificationExecutionResult {
      result: true,
      pending_errors: vec![
        (
          "GET /pending-foo returns 200".to_string(),
          VerificationMismatchResult::Error {
            error: "Provider state setup failed".to_string(),
            interaction_id: None,
          },
        ),
      ],
      interaction_results: vec![
        VerificationInteractionResult {
          interaction_id: None,
          interaction_key: None,
          description: "pending request".to_string(),
          interaction_description: "GET /pending-foo returns 200".to_string(),
          result: Err(MismatchResult::Error("Provider state setup failed".to_string(), None)),
          pending: true,
          duration: Duration::from_millis(3),
        },
      ],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }

  #[test]
  fn with_notices() {
    let mut notice = HashMap::new();
    notice.insert("text".to_string(), "This pact is being verified because it is the latest version".to_string());
    notice.insert("type".to_string(), "info".to_string());
    let result = VerificationExecutionResult {
      result: true,
      notices: vec![notice],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }

  #[test]
  fn all_mismatch_types() {
    let result = VerificationExecutionResult {
      result: false,
      errors: vec![
        (
          "Complex interaction".to_string(),
          VerificationMismatchResult::Mismatches {
            mismatches: vec![
              Mismatch::MethodMismatch {
                expected: "GET".to_string(),
                actual: "POST".to_string(),
                mismatch: "Expected method GET but received POST".to_string(),
              },
              Mismatch::PathMismatch {
                expected: "/foo".to_string(),
                actual: "/bar".to_string(),
                mismatch: "Expected path /foo but received /bar".to_string(),
              },
              Mismatch::HeaderMismatch {
                key: "Content-Type".to_string(),
                expected: "application/json".to_string(),
                actual: "text/plain".to_string(),
                mismatch: "Expected header Content-Type=application/json but received text/plain".to_string(),
              },
              Mismatch::QueryMismatch {
                parameter: "page".to_string(),
                expected: "1".to_string(),
                actual: "2".to_string(),
                mismatch: "Expected query parameter page=1 but received page=2".to_string(),
              },
              Mismatch::BodyTypeMismatch {
                expected: "application/json".to_string(),
                actual: "text/plain".to_string(),
                mismatch: "Expected body content type application/json but received text/plain".to_string(),
                expected_body: None,
                actual_body: None,
              },
              Mismatch::MetadataMismatch {
                key: "contentType".to_string(),
                expected: "application/json".to_string(),
                actual: "text/xml".to_string(),
                mismatch: "Expected metadata contentType=application/json but received text/xml".to_string(),
              },
            ],
            interaction_id: Some("complex-123".to_string()),
          },
        ),
      ],
      ..VerificationExecutionResult::default()
    };
    assert_snapshot!(to_xml(&result, "My Provider"));
  }
}
