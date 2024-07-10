//! Module to deal with consumer version selectors

use serde_json::{from_value, Value};

use crate::ConsumerVersionSelector;

/// Parses a vector of JSON into a vector of consumer version selectors
pub fn json_to_selectors(json: Vec<Value>) -> Vec<ConsumerVersionSelector> {
  json.iter().map(|t| from_value(t.clone()))
    .flatten()
    .collect()
}

/// Converts a vector of tags to a vector of consumer version selectors
pub fn consumer_tags_to_selectors(tags: Vec<&str>) -> Vec<ConsumerVersionSelector> {
  tags.iter().map(|t| {
    ConsumerVersionSelector {
      consumer: None,
      fallback_tag: None,
      tag: Some(t.to_string()),
      latest: Some(true),
      branch: None,
      deployed_or_released: None,
      deployed: None,
      released: None,
      main_branch: None,
      environment: None,
      matching_branch: None,
      fallback_branch: None,
    }
  }).collect()
}
