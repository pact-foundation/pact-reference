//! V4 specification models

use std::fmt::{Display, Formatter};
use std::fmt;

use anyhow::anyhow;
use crate::bodies::OptionalBody;
use std::collections::HashMap;
use std::str::FromStr;
use crate::content_types::{ContentType, detect_content_type_from_bytes};

pub mod http_parts;
pub mod interaction;
pub mod synch_http;
pub mod message_parts;
pub mod sync_message;
pub mod async_message;
pub mod pact;

/// V4 Interaction Type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl FromStr for V4InteractionType {
  type Err = anyhow::Error;

  /// Returns the V4 interaction type from the string value
  fn from_str(type_str: &str) -> anyhow::Result<V4InteractionType> {
    match type_str {
      "Synchronous/HTTP" => Ok(V4InteractionType::Synchronous_HTTP),
      "Asynchronous/Messages" => Ok(V4InteractionType::Asynchronous_Messages),
      "Synchronous/Messages" => Ok(V4InteractionType::Synchronous_Messages),
      _ => Err(anyhow!("'{}' is not a valid V4 interaction type", type_str))
    }
  }
}

/// Calculate the context type of the body, using the content type from the body. If that is
/// not set, look for a Content-Type header otherwise look at the body contents
pub fn calc_content_type(body: &OptionalBody, headers: &Option<HashMap<String, Vec<String>>>) -> Option<ContentType> {
  body.content_type()
    .or_else(|| headers.as_ref().map(|h| {
      match h.iter().find(|kv| kv.0.to_lowercase() == "content-type") {
        Some((_, v)) => ContentType::parse(v[0].as_str()).ok(),
        None => None
      }
    }).flatten())
    .or_else(|| if body.is_present() {
      // TODO:- use shared infer/tree_magic_mini here for consistency?
      detect_content_type_from_bytes(&*body.value().unwrap_or_default())
    } else {
      None
    })
}
