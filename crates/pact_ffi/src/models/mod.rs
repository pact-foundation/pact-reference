//! FFI functions to support Pact models.

use std::panic::RefUnwindSafe;
use std::sync::Mutex;

use libc::c_char;
use pact_models::pact::load_pact_from_json;
use serde_json::Value;
use tracing::error;

use crate::{ffi_fn, safe_str, as_ref};
use crate::models::iterators::PactInteractionIterator;
use crate::models::pact_specification::PactSpecification;
use crate::util::ptr;

pub mod async_message;
pub mod consumer;
pub mod contents;
pub mod expressions;
pub mod generators;
pub mod http_interaction;
pub mod interactions;
pub mod iterators;
pub mod matching_rules;
pub mod message;
pub mod message_pact;
pub mod pact_specification;
pub mod provider;
pub mod provider_state;
pub mod sync_message;

/// Opaque type for use as a pointer to a Pact model
#[derive(Debug)]
pub struct Pact {
  inner: Mutex<Box<dyn pact_models::pact::Pact + Send + Sync>>
}

impl Pact {
  /// Create a new FFI Pact wrapper for the given Pact model
  pub fn new(pact: Box<dyn pact_models::pact::Pact + Send + Sync>) -> Self {
    Pact {
      inner: Mutex::new(pact)
    }
  }
}

ffi_fn! {
  /// Parses the provided JSON into a Pact model. The returned Pact model must be freed with the
  /// `pactffi_pact_model_delete` function when no longer needed.
  ///
  /// # Error Handling
  ///
  /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
  fn pactffi_parse_pact_json(json: *const c_char) -> *mut Pact {
    let json_str = safe_str!(json);
    match serde_json::from_str::<Value>(json_str) {
      Ok(pact_json) => {
        let pact = load_pact_from_json("<FFI>", &pact_json)?;
        ptr::raw_to(Pact::new(pact))
      },
      Err(err) => {
        error!("Failed to parse the Pact JSON - {}", err);
        std::ptr::null_mut()
      }
    }
  } {
    std::ptr::null_mut()
  }
}

ffi_fn! {
  /// Frees the memory used by the Pact model
  fn pactffi_pact_model_delete(pact: *mut Pact) {
    ptr::drop_raw(pact);
  }
}

ffi_fn! {
  /// Returns an iterator over all the interactions in the Pact. The iterator will have to be
  /// deleted using the `pactffi_pact_interaction_iter_delete` function. The iterator will
  /// contain a copy of the interactions, so it will not be affected but mutations to the Pact
  /// model and will still function if the Pact model is deleted.
  ///
  /// # Safety
  /// This function is safe as long as the Pact pointer is a valid pointer.
  ///
  /// # Errors
  /// On any error, this function will return a NULL pointer.
  fn pactffi_pact_model_interaction_iterator(pact: *mut Pact) -> *mut PactInteractionIterator {
    let pact = as_ref!(pact);
    let inner = pact.inner.lock().unwrap();
    ptr::raw_to(PactInteractionIterator::new(inner.boxed()))
  } {
    std::ptr::null_mut()
  }
}

ffi_fn! {
  /// Returns the Pact specification enum that the Pact is for.
  fn pactffi_pact_spec_version(pact: *const Pact) -> PactSpecification {
    let pact = as_ref!(pact);
    let inner = pact.inner.lock().unwrap();
    inner.specification_version().into()
  } {
    PactSpecification::Unknown
  }
}

/// Opaque type for use as a pointer to a Pact interaction model
#[derive(Debug)]
pub struct PactInteraction {
  inner: Mutex<Box<dyn pact_models::interaction::Interaction + Send + Sync + RefUnwindSafe>>
}

impl PactInteraction {
  /// Create a new FFI Pact interaction wrapper for the given Pact interaction model
  pub fn new(interaction: &Box<dyn pact_models::interaction::Interaction + Send + Sync + RefUnwindSafe>) -> Self {
    PactInteraction {
      inner: Mutex::new(interaction.boxed())
    }
  }
}

ffi_fn! {
  /// Frees the memory used by the Pact interaction model
  fn pactffi_pact_interaction_delete(interaction: *const PactInteraction) {
    ptr::drop_raw(interaction as *mut PactInteraction);
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;

  use crate::models::{
    pactffi_pact_model_delete,
    pactffi_parse_pact_json,
    pactffi_pact_spec_version,
    pactffi_pact_model_interaction_iterator
  };
  use crate::models::consumer::{
    pactffi_consumer_get_name,
    pactffi_pact_consumer_delete,
    pactffi_pact_get_consumer
  };
  use crate::models::http_interaction::pactffi_sync_http_delete;
  use crate::models::interactions::{
    pactffi_pact_interaction_as_asynchronous_message,
    pactffi_pact_interaction_as_message,
    pactffi_pact_interaction_as_synchronous_http,
    pactffi_pact_interaction_as_synchronous_message
  };
  use crate::models::iterators::{
    pactffi_pact_interaction_iter_delete,
    pactffi_pact_interaction_iter_next
  };
  use crate::models::pact_specification::PactSpecification;
  use crate::models::provider::{
    pactffi_pact_get_provider,
    pactffi_pact_provider_delete,
    pactffi_provider_get_name
  };

  #[test]
  fn load_pact_from_json() {
    let json = CString::new(r#"{
      "provider": {
        "name": "load_pact_from_json Provider"
      },
      "consumer": {
        "name": "load_pact_from_json Consumer"
      },
      "interactions": [
        {
          "description": "GET request to retrieve default values",
          "providerStates": [
            {
              "name": "This is a test"
            }
          ],
          "request": {
            "matchingRules": {
              "path": {
                "combine": "AND",
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "/api/test/\\d{1,8}"
                  }
                ]
              }
            },
            "method": "GET",
            "path": "/api/test/4"
          },
          "response": {
            "body": [
              {
                "id": 32432,
                "name": "testId254",
                "size": 1445211
              }
            ],
            "headers": {
              "Content-Type": "application/json"
            },
            "matchingRules": {
              "body": {
                "$": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "type",
                      "min": 1
                    }
                  ]
                },
                "$[*].id": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "number"
                    }
                  ]
                },
                "$[*].name": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "type"
                    }
                  ]
                },
                "$[*].size": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "number"
                    }
                  ]
                }
              }
            },
            "status": 200
          }
        }
      ]
    }"#).unwrap();
    let pact = pactffi_parse_pact_json(json.as_ptr());
    expect!(pact.is_null()).to(be_false());

    let spec_version = pactffi_pact_spec_version(pact);

    let consumer = pactffi_pact_get_consumer(pact);
    let consumer_name_ptr = pactffi_consumer_get_name(consumer);
    let consumer_name = unsafe { CString::from_raw(consumer_name_ptr as *mut c_char) };

    let provider = pactffi_pact_get_provider(pact);
    let provider_name_ptr = pactffi_provider_get_name(provider);
    let provider_name = unsafe { CString::from_raw(provider_name_ptr as *mut c_char) };

    pactffi_pact_consumer_delete(consumer);
    pactffi_pact_provider_delete(provider);

    let interactions = pactffi_pact_model_interaction_iterator(pact);
    expect!(interactions.is_null()).to(be_false());

    let first = pactffi_pact_interaction_iter_next(interactions);
    expect!(first.is_null()).to(be_false());
    let http = pactffi_pact_interaction_as_synchronous_http(first);
    expect!(http.is_null()).to(be_false());
    let message = pactffi_pact_interaction_as_message(first);
    expect!(message.is_null()).to(be_true());
    let as_message = pactffi_pact_interaction_as_asynchronous_message(first);
    expect!(as_message.is_null()).to(be_true());
    let s_message = pactffi_pact_interaction_as_synchronous_message(first);
    expect!(s_message.is_null()).to(be_true());

    pactffi_sync_http_delete(http);

    let second = pactffi_pact_interaction_iter_next(interactions);
    expect!(second.is_null()).to(be_true());

    pactffi_pact_interaction_iter_delete(interactions);
    pactffi_pact_model_delete(pact);

    expect!(consumer_name.to_string_lossy()).to(be_equal_to("load_pact_from_json Consumer"));
    expect!(provider_name.to_string_lossy()).to(be_equal_to("load_pact_from_json Provider"));
    expect!(spec_version).to(be_equal_to(PactSpecification::V3));
  }
}
