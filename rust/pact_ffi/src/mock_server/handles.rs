//! Handle interface wrapping Rust models for use via FFI calls.
//!
//! Example of setting up a Pact, starting a mock server and then sending requests to the mock
//! server:
//! ```
//! use std::ffi::{CStr, CString};
//! use expectest::prelude::*;
//! use reqwest::blocking::Client;
//! use pact_ffi::mock_server::handles::{
//!   InteractionPart,
//!   pactffi_new_interaction,
//!   pactffi_new_pact,
//!   pactffi_response_status,
//!   pactffi_upon_receiving,
//!   pactffi_with_body,
//!   pactffi_with_header,
//!   pactffi_with_query_parameter_v2,
//!   pactffi_with_request
//! };
//! use pact_ffi::mock_server::{
//!   pactffi_cleanup_mock_server,
//!   pactffi_create_mock_server_for_pact,
//!   pactffi_mock_server_mismatches,
//!   pactffi_write_pact_file
//! };
//!
//! let consumer_name = CString::new("http-consumer").unwrap();
//! let provider_name = CString::new("http-provider").unwrap();
//! let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
//!
//! let description = CString::new("request_with_matchers").unwrap();
//! let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());
//!
//! let special_header = CString::new("My-Special-Content-Type").unwrap();
//! let content_type = CString::new("Content-Type").unwrap();
//! let authorization = CString::new("Authorization").unwrap();
//! let path_matcher = CString::new("{\"value\":\"/request/1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"\\/request\\/[0-9]+\"}").unwrap();
//! let value_header_with_matcher = CString::new("{\"value\":\"application/json\",\"pact:matcher:type\":\"dummy\"}").unwrap();
//! let auth_header_with_matcher = CString::new("{\"value\":\"Bearer 1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"Bearer [0-9]+\"}").unwrap();
//! let query_param_matcher = CString::new("{\"value\":\"bar\",\"pact:matcher:type\":\"regex\", \"regex\":\"(bar|baz|bat)\"}").unwrap();
//! let request_body_with_matchers = CString::new("{\"id\": {\"value\":1,\"pact:matcher:type\":\"type\"}}").unwrap();
//! let response_body_with_matchers = CString::new("{\"created\": {\"value\":\"maybe\",\"pact:matcher:type\":\"regex\", \"regex\":\"(yes|no|maybe)\"}}").unwrap();
//! let address = CString::new("127.0.0.1:0").unwrap();
//! let file_path = CString::new("/tmp/pact").unwrap();
//! let description = CString::new("a request to test the FFI interface").unwrap();
//! let method = CString::new("POST").unwrap();
//! let query =  CString::new("foo").unwrap();
//! let header = CString::new("application/json").unwrap();
//!
//! // Setup the request
//! pactffi_upon_receiving(interaction.clone(), description.as_ptr());
//! pactffi_with_request(interaction.clone(), method  .as_ptr(), path_matcher.as_ptr());
//! pactffi_with_header(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
//! pactffi_with_header(interaction.clone(), InteractionPart::Request, authorization.as_ptr(), 0, auth_header_with_matcher.as_ptr());
//! pactffi_with_query_parameter_v2(interaction.clone(), query.as_ptr(), 0, query_param_matcher.as_ptr());
//! pactffi_with_body(interaction.clone(), InteractionPart::Request, header.as_ptr(), request_body_with_matchers.as_ptr());
//!
//! // will respond with...
//! pactffi_with_header(interaction.clone(), InteractionPart::Response, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
//! pactffi_with_header(interaction.clone(), InteractionPart::Response, special_header.as_ptr(), 0, value_header_with_matcher.as_ptr());
//! pactffi_with_body(interaction.clone(), InteractionPart::Response, header.as_ptr(), response_body_with_matchers.as_ptr());
//! pactffi_response_status(interaction.clone(), 200);
//!
//! // Start the mock server
//! let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);
//!
//! expect!(port).to(be_greater_than(0));
//!
//! // Mock server has started, we can't now modify the pact
//! expect!(pactffi_upon_receiving(interaction.clone(), description.as_ptr())).to(be_false());
//!
//! // Interact with the mock server
//! let client = Client::default();
//! let result = client.post(format!("http://127.0.0.1:{}/request/9999?foo=baz", port).as_str())
//!   .header("Content-Type", "application/json")
//!   .header("Authorization", "Bearer 9999")
//!   .body(r#"{"id": 7}"#)
//!   .send();
//!
//! match result {
//!   Ok(res) => {
//!     expect!(res.status()).to(be_eq(200));
//!     expect!(res.headers().get("My-Special-Content-Type").unwrap()).to(be_eq("application/json"));
//!     let json: serde_json::Value = res.json().unwrap_or_default();
//!     expect!(json.get("created").unwrap().as_str().unwrap()).to(be_eq("maybe"));
//!   }
//!   Err(_) => {
//!     panic!("expected 200 response but request failed");
//!   }
//! };
//!
//! let mismatches = unsafe {
//!   CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
//! };
//!
//! // Write out the pact file, then clean up the mock server
//! pactffi_write_pact_file(port, file_path.as_ptr(), true);
//! pactffi_cleanup_mock_server(port);
//!
//! // Should be no mismatches
//! expect!(mismatches).to(be_equal_to("[]"));
//! ```

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::ptr::null_mut;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use bytes::Bytes;
use either::Either;
use itertools::Itertools;
use lazy_static::*;
use libc::{c_char, c_int, c_uint, c_ushort, EXIT_FAILURE, EXIT_SUCCESS, size_t};
use maplit::*;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, detect_content_type_from_string, JSON, TEXT, XML};
use pact_models::generators::{Generator, GeneratorCategory, Generators};
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::{matchers_from_json, Category, MatchingRule, MatchingRuleCategory, MatchingRules, RuleLogic};
use pact_models::pact::{ReadWritePact, write_pact};
use pact_models::path_exp::DocPath;
use pact_models::prelude::Pact;
use pact_models::prelude::v4::V4Pact;
use pact_models::provider_states::ProviderState;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::synch_http::SynchronousHttp;
use serde_json::{json, Value};
use tracing::*;

use pact_matching::generators::generate_message;
use pact_models::generators::GeneratorTestMode;
use futures::executor::block_on;

use crate::{convert_cstr, ffi_fn, safe_str};
use crate::error::set_error_msg;
use crate::mock_server::{StringResult, xml};
#[allow(deprecated)]
use crate::mock_server::bodies::{
  empty_multipart_body,
  file_as_multipart_body,
  matcher_from_integration_json,
  MultipartBody,
  process_array,
  process_json,
  process_object,
  request_multipart,
  response_multipart,
  get_content_type_hint,
  part_body_replace_marker
};
use crate::models::iterators::{PactMessageIterator, PactSyncHttpIterator, PactSyncMessageIterator};
use crate::ptr;

#[derive(Debug, Clone)]
/// Pact handle inner struct
/// cbindgen:ignore
pub struct PactHandleInner {
  pub(crate) pact: V4Pact,
  pub(crate) mock_server_started: bool,
  pub(crate) specification_version: PactSpecification
}

lazy_static! {
  static ref PACT_HANDLES: Arc<Mutex<HashMap<u16, RefCell<PactHandleInner>>>> = Arc::new(Mutex::new(hashmap![]));
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct PactHandle {
  /// Pact reference
  pact_ref: u16
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct InteractionHandle {
  /// Interaction reference
  interaction_ref: u32
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Request or Response enum
pub enum InteractionPart {
  /// Request part
  Request = 0,
  /// Response part
  Response = 1
}

impl PactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();

    let keys: HashSet<&u16> = handles.keys().collect();
    let mut id: u16 = 1;
    while keys.contains(&id) {
      id = id + 1;
    }

    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    PactHandle {
      pact_ref: id
    }
  }

  /// Invokes the closure with the inner Pact model
  ///
  /// # Errors
  /// This function acquires a lock on the PACT_HANDLES mutex. If the closure panics, this mutex
  /// will be poisoned. So panics must be avoided.
  pub(crate) fn with_pact<R>(&self, f: &dyn Fn(u16, &mut PactHandleInner) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    trace!("with_pact - ref = {}, keys = {:?}", self.pact_ref, handles.keys());
    handles.get_mut(&self.pact_ref).map(|inner| {
      trace!("with_pact before - ref = {}, inner = {:?}", self.pact_ref, inner);
      let result = f(self.pact_ref - 1, &mut inner.borrow_mut());
      trace!("with_pact after - ref = {}, inner = {:?}", self.pact_ref, inner);
      result
    })
  }
}

impl InteractionHandle {
  /// Creates a new handle to an Interaction
  pub fn new(pact: PactHandle, interaction: u16) -> InteractionHandle {
    let mut index = pact.pact_ref as u32;
    index = index << 16;
    index = index + interaction as u32;
    InteractionHandle {
      interaction_ref: index
    }
  }

  /// Invokes the closure with the inner Pact model
  ///
  /// # Errors
  /// This function acquires a lock on the PACT_HANDLES mutex. If the closure panics, this mutex
  /// will be poisoned. So panics must be avoided.
  pub fn with_pact<R>(&self, f: &dyn Fn(u16, &mut PactHandleInner) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let index = (self.interaction_ref >> 16) as u16;
    handles.get_mut(&index).map(|inner| f(index - 1, &mut inner.borrow_mut()))
  }

  /// Invokes the closure with the inner Interaction model
  ///
  /// # Errors
  /// This function acquires a lock on the PACT_HANDLES mutex. If the closure panics, this mutex
  /// will be poisoned. So panics must be avoided.
  pub fn with_interaction<R>(&self, f: &dyn Fn(u16, bool, &mut dyn V4Interaction) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let index = (self.interaction_ref >> 16) as u16;
    let interaction = (self.interaction_ref & 0x0000FFFF) as u16;

    trace!("with_interaction - index = {}, interaction = {}", index, interaction);
    trace!("with_interaction - keys = {:?}", handles.keys());

    handles.get_mut(&index).map(|inner| {
      let inner_mut = &mut *inner.borrow_mut();
      trace!("with_interaction - inner = {:?}", inner_mut);
      let interactions = &mut inner_mut.pact.interactions;
      match interactions.get_mut((interaction - 1) as usize) {
        Some(inner_i) => {
          Some(f(interaction - 1, inner_mut.mock_server_started, inner_i.as_mut()))
        },
        None => {
          debug!("Did not find interaction for index = {}, interaction = {}, pact has {} interactions",
            index, interaction, interactions.len());
          None
        }
      }
    }).flatten()
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessagePactHandle {
  /// Pact reference
  pact_ref: u16
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessageHandle {
  /// Interaction reference
  interaction_ref: u32
}

impl MessagePactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let id = (handles.len() + 1) as u16;
    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    MessagePactHandle {
      pact_ref: id
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(u16, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact_ref).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(self.pact_ref - 1, &mut ref_mut.pact, specification)
    })
  }
}

impl MessageHandle {
  /// Creates a new handle to a message
  pub fn new(pact: MessagePactHandle, message: u16) -> MessageHandle {
    let mut index = pact.pact_ref as u32;
    index = index << 16;
    index = index + message as u32;
    MessageHandle {
      interaction_ref: index
    }
  }

  /// Creates a new handle to a message
  pub fn new_v4(pact: PactHandle, message: usize) -> MessageHandle {
    let mut index = pact.pact_ref as u32;
    index = index << 16;
    index = index + message as u32;
    MessageHandle {
      interaction_ref: index
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(u16, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let index = self.interaction_ref as u16;
    handles.get_mut(&index).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(index - 1, & mut ref_mut.pact, specification)
    })
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_message<R>(&self, f: &dyn Fn(u16, &mut dyn V4Interaction, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let index = (self.interaction_ref >> 16) as u16;
    let interaction = self.interaction_ref as u16;
    handles.get_mut(&index).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      ref_mut.pact.interactions.get_mut((interaction - 1) as usize)
        .map(|inner_i| {
          if inner_i.is_message() {
            Some(f(interaction - 1, inner_i.as_mut(), specification))
          } else {
            error!("Interaction {:#x} is not a message interaction, it is {}", self.interaction_ref, inner_i.type_of());
            None
          }
        }).flatten()
    }).flatten()
  }
}


/// Creates a new Pact model and returns a handle to it.
///
/// * `consumer_name` - The name of the consumer for the pact.
/// * `provider_name` - The name of the provider for the pact.
///
/// Returns a new `PactHandle`. The handle will need to be freed with the `pactffi_free_pact_handle`
/// method to release its resources.
#[no_mangle]
pub extern fn pactffi_new_pact(consumer_name: *const c_char, provider_name: *const c_char) -> PactHandle {
  let consumer = convert_cstr("consumer_name", consumer_name).unwrap_or("Consumer");
  let provider = convert_cstr("provider_name", provider_name).unwrap_or("Provider");
  PactHandle::new(consumer, provider)
}

ffi_fn! {
  /// Returns a mutable pointer to a Pact model which has been cloned from the Pact handle's inner
  /// Pact model. The returned Pact model must be freed with the `pactffi_pact_model_delete`
  /// function when no longer needed.
  fn pactffi_pact_handle_to_pointer(pact: PactHandle) -> *mut crate::models::Pact {
    pact.with_pact(&|_, inner| {
      ptr::raw_to(crate::models::Pact::new(inner.pact.boxed()))
    }).unwrap_or(std::ptr::null_mut())
  } {
    std::ptr::null_mut()
  }
}

fn find_interaction_with_description(pact: &V4Pact, description: &str) -> Option<usize> {
  pact.interactions.iter().find_position(|i| {
    i.description() == description
  }).map(|(index, _)| index)
}

/// Creates a new HTTP Interaction and returns a handle to it. Calling this function with the
/// same description as an existing interaction will result in that interaction being replaced
/// with the new one.
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
///
/// Returns a new `InteractionHandle`.
#[no_mangle]
pub extern fn pactffi_new_interaction(pact: PactHandle, description: *const c_char) -> InteractionHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner| {
      let interaction = SynchronousHttp {
        description: description.to_string(),
        ..SynchronousHttp::default()
      };
      if let Some(index) = find_interaction_with_description(&inner.pact, description) {
        warn!("There is an existing interaction with description '{}', it will be replaced", description);
        inner.pact.interactions[index] = interaction.boxed_v4();
        InteractionHandle::new(pact, (index + 1) as u16)
      } else {
        inner.pact.interactions.push(interaction.boxed_v4());
        InteractionHandle::new(pact, inner.pact.interactions.len() as u16)
      }
    }).unwrap_or_else(|| InteractionHandle::new(pact, 0))
  } else {
    InteractionHandle::new(pact, 0)
  }
}

/// Creates a new message interaction and returns a handle to it. Calling this function with the
/// same description as an existing interaction will result in that interaction being replaced
/// with the new one.
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
///
/// Returns a new `InteractionHandle`.
#[no_mangle]
pub extern fn pactffi_new_message_interaction(pact: PactHandle, description: *const c_char) -> InteractionHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner| {
      let interaction = AsynchronousMessage {
        description: description.to_string(),
        ..AsynchronousMessage::default()
      };
      if let Some(index) = find_interaction_with_description(&inner.pact, description) {
        warn!("There is an existing interaction with description '{}', it will be replaced", description);
        inner.pact.interactions[index] = interaction.boxed_v4();
        InteractionHandle::new(pact, (index + 1) as u16)
      } else {
        inner.pact.interactions.push(interaction.boxed_v4());
        InteractionHandle::new(pact, inner.pact.interactions.len() as u16)
      }
    }).unwrap_or_else(|| InteractionHandle::new(pact, 0))
  } else {
    InteractionHandle::new(pact, 0)
  }
}

/// Creates a new synchronous message interaction (request/response) and returns a handle to it.
/// Calling this function with the same description as an existing interaction will result in
/// that interaction being replaced with the new one.
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
///
/// Returns a new `InteractionHandle`.
#[no_mangle]
pub extern fn pactffi_new_sync_message_interaction(pact: PactHandle, description: *const c_char) -> InteractionHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner| {
      let interaction = SynchronousMessage {
        description: description.to_string(),
        ..SynchronousMessage::default()
      };
      if let Some(index) = find_interaction_with_description(&inner.pact, description) {
        warn!("There is an existing interaction with description '{}', it will be replaced", description);
        inner.pact.interactions[index] = interaction.boxed_v4();
        InteractionHandle::new(pact, (index + 1) as u16)
      } else {
        inner.pact.interactions.push(interaction.boxed_v4());
        InteractionHandle::new(pact, inner.pact.interactions.len() as u16)
      }
    }).unwrap_or_else(|| InteractionHandle::new(pact, 0))
  } else {
    InteractionHandle::new(pact, 0)
  }
}

/// Sets the description for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
#[no_mangle]
pub extern fn pactffi_upon_receiving(interaction: InteractionHandle, description: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    interaction.with_interaction(&|_, mock_server_started, inner| {
      inner.set_description(description);
      !mock_server_started
    }).unwrap_or(false)
  } else {
    false
  }
}

/// Adds a provider state to the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `description` - The provider state description. It needs to be unique.
#[no_mangle]
pub extern fn pactffi_given(interaction: InteractionHandle, description: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    interaction.with_interaction(&|_, mock_server_started, inner| {
      inner.provider_states_mut().push(ProviderState::default(&description.to_string()));
      !mock_server_started
    }).unwrap_or(false)
  } else {
    false
  }
}

ffi_fn! {
    /// Sets the test name annotation for the interaction. This allows capturing the name of
    /// the test as metadata. This can only be used with V4 interactions.
    ///
    /// # Safety
    ///
    /// The test name parameter must be a valid pointer to a NULL terminated string.
    ///
    /// # Error Handling
    ///
    /// If the test name can not be set, this will return a positive value.
    ///
    /// * `1` - Function panicked. Error message will be available by calling `pactffi_get_error_message`.
    /// * `2` - Handle was not valid.
    /// * `3` - Mock server was already started and the integration can not be modified.
    /// * `4` - Not a V4 interaction.
    fn pactffi_interaction_test_name(interaction: InteractionHandle, test_name: *const c_char) -> c_uint {
      let test_name = safe_str!(test_name);
      interaction.with_interaction(&|_, started, inner| {
        if !started {
          if let Some(i) = inner.as_v4_mut() {
            i.comments_mut().insert("testname".to_string(), json!(test_name));
            0
          } else {
            4
          }
        } else {
          3
        }
      }).unwrap_or(2)
    } {
      1
    }
}

/// Adds a parameter key and value to a provider state to the Interaction. If the provider state
/// does not exist, a new one will be created, otherwise the parameter will be merged into the
/// existing one. The parameter value will be parsed as JSON.
///
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started).
///
/// # Parameters
/// * `description` - The provider state description. It needs to be unique.
/// * `name` - Parameter name.
/// * `value` - Parameter value as JSON.
#[no_mangle]
pub extern fn pactffi_given_with_param(interaction: InteractionHandle, description: *const c_char,
                                       name: *const c_char, value: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    if let Some(name) = convert_cstr("name", name) {
      let value = convert_cstr("value", value).unwrap_or_default();
      interaction.with_interaction(&|_, mock_server_started, inner| {
        let value = match serde_json::from_str(value) {
          Ok(json) => json,
          Err(_) => json!(value)
        };
        match inner.provider_states().iter().find_position(|state| state.name == description) {
          Some((index, _)) => {
            inner.provider_states_mut().get_mut(index).unwrap().params.insert(name.to_string(), value);
          },
          None => inner.provider_states_mut().push(ProviderState {
            name: description.to_string(),
            params: hashmap!{ name.to_string() => value }
          })
        };
        !mock_server_started
      }).unwrap_or(false)
    } else {
      false
    }
  } else {
    false
  }
}

/// Adds a provider state to the Interaction with a set of parameter key and value pairs in JSON
/// form. If the params is not an JSON object, it will add it as a single parameter with a `value`
/// key.
///
/// # Parameters
/// * `description` - The provider state description.
/// * `params` - Parameter values as a JSON fragment.
///
/// # Errors
/// Returns EXIT_FAILURE (1) if the interaction or Pact can't be modified (i.e. the mock server
/// for it has already started).
/// Returns 2 and sets the error message (which can be retrieved with `pactffi_get_error_message`)
/// if the parameter values con't be parsed as JSON.
/// Returns 3 if any of the C strings are not valid.
///
#[no_mangle]
pub extern fn pactffi_given_with_params(
  interaction: InteractionHandle,
  description: *const c_char,
  params: *const c_char
) -> c_int {
  if let Some(description) = convert_cstr("description", description) {
    if let Some(params) = convert_cstr("params", params) {
      let params_value = match serde_json::from_str(params) {
        Ok(json) => json,
        Err(err) => {
          error!("Parameters are not valid JSON: {}", err);
          set_error_msg(err.to_string());
          return 2;
        }
      };
      let params_map = match params_value {
        Value::Object(map) => map.iter()
          .map(|(k, v)| (k.clone(), v.clone()))
          .collect(),
        _ => hashmap! { "value".to_string() => params_value }
      };
      interaction.with_interaction(&|_, mock_server_started, inner| {
        inner.provider_states_mut().push(ProviderState {
          name: description.to_string(),
          params: params_map.clone()
        });
        if mock_server_started { EXIT_FAILURE } else { EXIT_SUCCESS }
      }).unwrap_or(EXIT_FAILURE)
    } else {
      3
    }
  } else {
    3
  }
}

/// Configures the request for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `method` - The request method. Defaults to GET.
/// * `path` - The request path. Defaults to `/`.
///
/// To include matching rules for the path (only regex really makes sense to use), include the
/// matching rule JSON format with the value as a single JSON document. I.e.
///
/// ```c
/// const char* value = "{\"value\":\"/path/to/100\", \"pact:matcher:type\":\"regex\", \"regex\":\"\\/path\\/to\\/\\\\d+\"}";
/// pactffi_with_request(handle, "GET", value);
/// ```
/// See [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
#[no_mangle]
pub extern fn pactffi_with_request(
  interaction: InteractionHandle,
  method: *const c_char,
  path: *const c_char
) -> bool {
  let method = convert_cstr("method", method).unwrap_or("GET");
  let path = convert_cstr("path", path).unwrap_or("/");

  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      let path = from_integration_json_v2(&mut reqres.request.matching_rules,
        &mut reqres.request.generators, &path.to_string(), DocPath::empty(), "path", 0);
      reqres.request.method = method.to_string();
      reqres.request.path = match path {
        Either::Left(value) => value,
        Either::Right(values) => {
          warn!("Received multiple values for the path ({:?}), will only use the first one", values);
          values.first().cloned().unwrap_or_default()
        }
      };
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Configures a query parameter for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `name` - the query parameter name.
/// * `value` - the query parameter value.
/// * `index` - the index of the value (starts at 0). You can use this to create a query parameter with multiple values
///
/// **DEPRECATED:** Use `pactffi_with_query_parameter_v2`, which deals with multiple values correctly
#[no_mangle]
#[deprecated]
pub extern fn pactffi_with_query_parameter(
  interaction: InteractionHandle,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        reqres.request.query = reqres.request.query.clone().map(|mut q| {
          let mut path = DocPath::root();
          path.push_field(name).push_index(index);
          #[allow(deprecated)]
          let value = from_integration_json(&mut reqres.request.matching_rules, &mut reqres.request.generators, &value.to_string(), path, "query");
          if q.contains_key(name) {
            let values = q.get_mut(name).unwrap();
            if index >= values.len() {
              values.resize_with(index + 1, Default::default);
            }
            values[index] = value;
          } else {
            let mut values: Vec<String> = Vec::new();
            values.resize_with(index + 1, Default::default);
            values[index] = value;
            q.insert(name.to_string(), values);
          };
          q
        }).or_else(|| {
          let mut path = DocPath::root();
          path.push_field(name).push_index(index);
          #[allow(deprecated)]
          let value = from_integration_json(&mut reqres.request.matching_rules, &mut reqres.request.generators, &value.to_string(), path, "query");
          let mut values: Vec<String> = Vec::new();
          values.resize_with(index + 1, Default::default);
          values[index] = value;
          Some(hashmap! { name.to_string() => values })
        });
        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring query parameter with empty or null name");
    false
  }
}

/// Configures a query parameter for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `name` - the query parameter name.
/// * `value` - the query parameter value. Either a simple string or a JSON document.
/// * `index` - the index of the value (starts at 0). You can use this to create a query parameter with multiple values
///
/// To setup a query parameter with multiple values, you can either call this function multiple times
/// with a different index value, i.e. to create `id=2&id=3`
///
/// ```c
/// pactffi_with_query_parameter_v2(handle, "id", 0, "2");
/// pactffi_with_query_parameter_v2(handle, "id", 1, "3");
/// ```
///
/// Or you can call it once with a JSON value that contains multiple values:
///
/// ```c
/// const char* value = "{\"value\": [\"2\",\"3\"]}";
/// pactffi_with_query_parameter_v2(handle, "id", 0, value);
/// ```
///
/// To include matching rules for the query parameter, include the matching rule JSON format with
/// the value as a single JSON document. I.e.
///
/// ```c
/// const char* value = "{\"value\":\"2\", \"pact:matcher:type\":\"regex\", \"regex\":\"\\\\d+\"}";
/// pactffi_with_query_parameter_v2(handle, "id", 0, value);
/// ```
/// See [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
///
/// If you want the matching rules to apply to all values (and not just the one with the given
/// index), make sure to set the value to be an array.
///
/// ```c
/// const char* value = "{\"value\":[\"2\"], \"pact:matcher:type\":\"regex\", \"regex\":\"\\\\d+\"}";
/// pactffi_with_query_parameter_v2(handle, "id", 0, value);
/// ```
/// # Safety
/// The name and value parameters must be valid pointers to NULL terminated strings.
/// ```
#[no_mangle]
pub extern fn pactffi_with_query_parameter_v2(
  interaction: InteractionHandle,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    trace!(?interaction, name, index, value, "pactffi_with_query_parameter_v2 called");
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        let mut path = DocPath::root();
        path.push_field(name);
        if index > 0 {
          path.push_index(index);
        }

        let value = from_integration_json_v2(
          &mut reqres.request.matching_rules,
          &mut reqres.request.generators,
          value,
          path,
          "query",
          index
        );
        match value {
          Either::Left(value) => {
            reqres.request.query = update_query_map(index, name, reqres, &value);
          }
          Either::Right(values) => if index == 0 {
            reqres.request.query = reqres.request.query.clone().map(|mut q| {
              if q.contains_key(name) {
                let vec = q.get_mut(name).unwrap();
                vec.extend_from_slice(&values);
              } else {
                q.insert(name.to_string(), values.clone());
              };
              q
            }).or_else(|| Some(hashmap! { name.to_string() => values }))
          } else {
            reqres.request.query = update_query_map(index, name, reqres, &values.first().cloned().unwrap_or_default());
          }
        }
        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring query parameter with empty or null name");
    false
  }
}

fn update_query_map(index: size_t, name: &str, reqres: &mut SynchronousHttp, value: &String) -> Option<HashMap<String, Vec<String>>> {
  reqres.request.query.clone().map(|mut q| {
    if q.contains_key(name) {
      let values = q.get_mut(name).unwrap();
      if index >= values.len() {
        values.resize_with(index + 1, Default::default);
      }
      values[index] = value.clone();
    } else {
      let mut values: Vec<String> = Vec::new();
      values.resize_with(index + 1, Default::default);
      values[index] = value.clone();
      q.insert(name.to_string(), values);
    };
    q
  }).or_else(|| {
    let mut values: Vec<String> = Vec::new();
    values.resize_with(index + 1, Default::default);
    values[index] = value.clone();
    Some(hashmap! { name.to_string() => values })
  })
}

/// Convert JSON matching rule structures into their internal representation (excl. bodies)
///
/// For non-body values (headers, query, path etc.) extract out the value from any matchers
/// and apply the matchers/generators to the model.
///
/// Will either return a single value, or a vector if the JSON represents multiple values.
#[deprecated]
fn from_integration_json(
  rules: &mut MatchingRules,
  generators: &mut Generators,
  value: &str,
  path: DocPath,
  category: &str,
) -> String {
  let category = rules.add_category(category);

  match serde_json::from_str(value) {
    Ok(json) => match json {
      Value::Object(ref map) => {
        let json: Value = process_object(map, category, generators, path, false);
        // These are simple JSON primitives (strings), so we must unescape them
        json_to_string(&json)
      },
      _ => value.to_string()
    },
    Err(_) => value.to_string()
  }
}

/// Convert JSON matching rule structures into their internal representation (excl. bodies)
///
/// For non-body values (headers, query, path etc.) extract out the value from any matchers
/// and apply the matchers/generators to the model.
///
/// Will either return a single value, or a vector if the JSON represents multiple values.
fn from_integration_json_v2(
  rules: &mut MatchingRules,
  generators: &mut Generators,
  value: &str,
  path: DocPath,
  category: &str,
  index: usize
) -> Either<String, Vec<String>> {
  trace!(value, %path, category, index, "from_integration_json_v2 called");
  let matching_rules = rules.add_category(category);
  let path_or_status = [Category::PATH, Category::STATUS].contains(&matching_rules.name);
  let query_or_header = [Category::QUERY, Category::HEADER].contains(&matching_rules.name);

  match serde_json::from_str(value) {
    Ok(json) => match json {
      Value::Object(ref map) => {
        let result = if map.contains_key("pact:matcher:type") {
          debug!("detected pact:matcher:type, will configure a matcher");
          #[allow(deprecated)]
          let matching_rule = matcher_from_integration_json(map);
          trace!("matching_rule = {matching_rule:?}");

          let (path, result_value) = match map.get("value") {
            Some(val) => match val {
              Value::Array(array) => {
                let array = process_array(&array, matching_rules, generators, path.clone(), true, false);
                (path.clone(), array)
              },
              _ => (path.clone(), val.clone())
            },
            None => (path.clone(), Value::Null)
          };

          if let Some(rule) = &matching_rule {
            let path = if path_or_status {
              path.parent().unwrap_or(DocPath::root())
            } else {
              path.clone()
            };
            if query_or_header {
              if index > 0 {
                // If the index > 0, and there is an existing entry with the base name, we need
                // to re-key that with an index of 0
                let mut parent = path.parent().unwrap_or(DocPath::root());
                if let Entry::Occupied(rule) = matching_rules.rules.entry(parent.clone()) {
                  let rules = rule.remove();
                  matching_rules.rules.insert(parent.push_index(0).clone(), rules);
                }
              }
              matching_rules.add_rule(path, rule.clone(), RuleLogic::And);
            } else {
              matching_rules.add_rule(path, rule.clone(), RuleLogic::And);
            }
          }
          if let Some(gen) = map.get("pact:generator:type") {
            debug!("detected pact:generator:type, will configure a generators");
            if let Some(generator) = Generator::from_map(&json_to_string(gen), map) {
              let category = match matching_rules.name {
                Category::BODY => &GeneratorCategory::BODY,
                Category::HEADER => &GeneratorCategory::HEADER,
                Category::PATH => &GeneratorCategory::PATH,
                Category::QUERY => &GeneratorCategory::QUERY,
                Category::STATUS => &GeneratorCategory::STATUS,
                _ => {
                  warn!("invalid generator category {} provided, defaulting to body", matching_rules.name);
                  &GeneratorCategory::BODY
                }
              };
              let path = if path_or_status {
                path.parent().unwrap_or(DocPath::root())
              } else {
                path.clone()
              };
              generators.add_generator_with_subcategory(category, path.clone(), generator);
            }
          }

          result_value
        } else {
          debug!("Configuring a normal value using the 'value' attribute");
          map.get("value").cloned().unwrap_or_default()
        };
        match result {
          Value::Array(values) => Either::Right(values.iter().map(|v| json_to_string(v)).collect()),
          _ => {
            // These are simple JSON primitives (strings), so we must unescape them
            Either::Left(json_to_string(&result))
          }
        }
      },
      _ => Either::Left(value.to_string())
    },
    Err(err) => {
      warn!("Failed to parse the value, treating it as a plain string: {}", err);
      Either::Left(value.to_string())
    }
  }
}

pub(crate) fn process_xml(body: String, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> Result<Vec<u8>, String> {
  trace!("process_xml");
  match serde_json::from_str(&body) {
    Ok(json) => match json {
      Value::Object(ref map) => xml::generate_xml_body(map, matching_rules, generators),
      _ => Err(format!("JSON document is invalid (expected an Object), have {}", json))
    },
    Err(err) => Err(format!("Failed to parse XML builder document: {}", err))
  }
}

/// Sets the specification version for a given Pact model. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started) or the version is invalid.
///
/// * `pact` - Handle to a Pact model
/// * `version` - the spec version to use
#[no_mangle]
pub extern fn pactffi_with_specification(pact: PactHandle, version: PactSpecification) -> bool {
  pact.with_pact(&|_, inner| {
    inner.specification_version = version.into();
    !inner.mock_server_started
  }).unwrap_or(false)
}

ffi_fn! {
  /// Returns the Pact specification enum that the Pact is for.
  fn pactffi_handle_get_pact_spec_version(pact: PactHandle) -> PactSpecification {
    pact.with_pact(&|_, inner| {
      inner.specification_version
    }).unwrap_or(PactSpecification::Unknown)
  } {
    PactSpecification::Unknown
  }
}

/// Sets the additional metadata on the Pact file. Common uses are to add the client library details such as the name and version
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has already started)
///
/// * `pact` - Handle to a Pact model
/// * `namespace` - the top level metadat key to set any key values on
/// * `name` - the key to set
/// * `value` - the value to set
#[no_mangle]
pub extern fn pactffi_with_pact_metadata(
  pact: PactHandle,
  namespace: *const c_char,
  name: *const c_char,
  value: *const c_char
) -> bool {
  pact.with_pact(&|_, inner| {
    let namespace = convert_cstr("namespace", namespace).unwrap_or_default();
    let name = convert_cstr("name", name).unwrap_or_default();
    let value = convert_cstr("value", value).unwrap_or_default();

    if !namespace.is_empty() {
      inner.pact.metadata.insert(namespace.to_string(), json!({ name: value }));
    } else {
      warn!("no namespace provided for metadata {:?} => {:?}. Ignoring", name, value);
    }
    !inner.mock_server_started
  }).unwrap_or(false)
}

/// Configures a header for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the header to (Request or Response).
/// * `name` - the header name.
/// * `value` - the header value.
/// * `index` - the index of the value (starts at 0). You can use this to create a header with multiple values
///
/// **DEPRECATED:** Use `pactffi_with_header_v2`, which deals with multiple values correctly
#[deprecated]
#[no_mangle]
pub extern fn pactffi_with_header(
  interaction: InteractionHandle,
  part: InteractionPart,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  trace!(">>> pactffi_with_header({:?}, {:?}, {:?}, {:?}, {:?})", interaction, part, name, index, value);
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    trace!(?name, ?value, "pactffi_with_header");
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        let headers = match part {
          InteractionPart::Request => reqres.request.headers.clone(),
          InteractionPart::Response => reqres.response.headers.clone()
        };

        let mut path = DocPath::root();
        path.push_field(name);

        #[allow(deprecated)]
        let value = match part {
          InteractionPart::Request => from_integration_json(
            &mut reqres.request.matching_rules,
            &mut reqres.request.generators,
            &value.to_string(),
            path,
            "header"),
          InteractionPart::Response => from_integration_json(
            &mut reqres.response.matching_rules,
            &mut reqres.response.generators,
            &value.to_string(),
            path,
            "header")
        };

        let updated_headers = headers.map(|mut h| {
          if h.contains_key(name) {
            let values = h.get_mut(name).unwrap();
            if index >= values.len() {
              values.resize_with(index + 1, Default::default);
            }
            values[index] = value.to_string();
          } else {
            let mut values: Vec<String> = Vec::new();
            values.resize_with(index + 1, Default::default);
            values[index] = value.to_string();
            h.insert(name.to_string(), values);
          };
          h
        }).or_else(|| {
          let mut values: Vec<String> = Vec::new();
          values.resize_with(index + 1, Default::default);
          values[index] = value.to_string();
          Some(hashmap! { name.to_string() => values })
        });
        match part {
          InteractionPart::Request => reqres.request.headers = updated_headers,
          InteractionPart::Response => reqres.response.headers = updated_headers
        };
        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring header with empty or null name");
    false
  }
}

/// Configures a header for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the header to (Request or Response).
/// * `name` - the header name.
/// * `value` - the header value.
/// * `index` - the index of the value (starts at 0). You can use this to create a header with multiple values
///
/// To setup a header with multiple values, you can either call this function multiple times
/// with a different index value, i.e. to create `x-id=2, 3`
///
/// ```c
/// pactffi_with_header_v2(handle, InteractionPart::Request, "x-id", 0, "2");
/// pactffi_with_header_v2(handle, InteractionPart::Request, "x-id", 1, "3");
/// ```
///
/// Or you can call it once with a JSON value that contains multiple values:
///
/// ```c
/// const char* value = "{\"value\": [\"2\",\"3\"]}";
/// pactffi_with_header_v2(handle, InteractionPart::Request, "x-id", 0, value);
/// ```
///
/// To include matching rules for the header, include the matching rule JSON format with
/// the value as a single JSON document. I.e.
///
/// ```c
/// const char* value = "{\"value\":\"2\", \"pact:matcher:type\":\"regex\", \"regex\":\"\\\\d+\"}";
/// pactffi_with_header_v2(handle, InteractionPart::Request, "id", 0, value);
/// ```
/// See [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
///
/// NOTE: If you pass in a form with multiple values, the index will be ignored.
///
/// # Safety
/// The name and value parameters must be valid pointers to NULL terminated strings.
#[no_mangle]
pub extern fn pactffi_with_header_v2(
  interaction: InteractionHandle,
  part: InteractionPart,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        let mut path = DocPath::root();
        path.push_field(name);
        if index > 0 {
          path.push_index(index);
        }

        let value = match part {
          InteractionPart::Request => from_integration_json_v2(
            &mut reqres.request.matching_rules,
            &mut reqres.request.generators,
            &value.to_string(),
            path,
            "header",
            index
          ),
          InteractionPart::Response => from_integration_json_v2(
            &mut reqres.response.matching_rules,
            &mut reqres.response.generators,
            &value.to_string(),
            path,
            "header",
            index
          )
        };

        debug!("parsed header value: {:?}", value);
        match &value {
          Either::Left(value) => {
            // Single value, either as a single JSON string or a simple String
            trace!("Single header value received");
            let headers = match part {
              InteractionPart::Request => reqres.request.headers_mut(),
              InteractionPart::Response => reqres.response.headers_mut()
            };
            // Lookup any exiting key in the map. May have a different case
            trace!("Existing header keys = {:?}", headers.keys());
            let name_lookup = name.to_lowercase();
            let header_key = headers.iter()
              .find(|(k, _v)| k.to_lowercase() == name_lookup)
              .map(|(k, _v)| k.clone());
            if index > 0 {
              // Index is set, so we set that value to the string provided
              trace!("Index {} is set >0, so we set that value to the string provided", index);
              match &header_key {
                Some(key) => {
                  trace!("Existing key found '{}'", key);
                  let values = headers.get_mut(key).unwrap();
                  if index >= values.len() {
                    values.resize_with(index + 1, Default::default);
                  }
                  values[index] = value.clone();
                },
                None => {
                  // No existing key found, user may have called the API out of order
                  trace!("No existing key found");
                  let mut values = Vec::new();
                  values.resize_with(index + 1, Default::default);
                  values[index] = value.clone();
                  headers.insert(name.to_string(), values);
                }
              };
            } else {
              // Index is zero, so we need to try parse the header value into an array (See #300)
              trace!("Index is 0");
              match &header_key {
                Some(key) => {
                  // Exiting key, we need to merge with what is there
                  trace!("Existing key found '{}'", key);
                  let header_values = parse_header(key.as_str(), value.as_str());
                  let values = headers.get_mut(key).unwrap();
                  if values.is_empty() {
                    // No existing values, easy case, we just put what we have
                    values.extend_from_slice(header_values.as_slice());
                  } else {
                    if header_values.is_empty() {
                      // User passed in an empty value, so we just set an empty value.
                      trace!("Passed in value is empty");
                      values[0] = String::default();
                    } else if header_values.len() == 1 {
                      trace!("Single passed in value '{}'", value);
                      // User passed in a value that resolved to a single value, we just replace
                      // the value at index 0. This is probably the most common case.
                      values[0] = value.clone();
                    } else {
                      // User passed in a value that resolved to multiple values.
                      // This is the confusing case, there are existing values, and more values
                      // have been provided. Do we merge or replace them?
                      // Assuming the user meant to replace them.
                      trace!("Multiple passed in values {:?}", header_values);
                      values.clear();
                      values.extend_from_slice(header_values.as_slice());
                    }
                  };
                },
                None => {
                  // No existing key, easy case, we just put what we have
                  let header_values = parse_header(name, value.as_str());
                  trace!("No existing key found, setting header '{}' = {:?}", name, header_values);
                  headers.insert(name.to_string(), header_values);
                }
              };
            }
          }
          Either::Right(values) => {
            // Multiple values passed via a JSON array, so we just replace the values.
            trace!("Multiple header values received");
            let values: Vec<_> = values.iter().map(|v| v.as_str()).collect();
            match part {
              InteractionPart::Request => reqres.request.set_header(name, values.as_slice()),
              InteractionPart::Response => reqres.response.set_header(name, values.as_slice())
            };
          }
        };

        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring header with empty or null name");
    false
  }
}

ffi_fn! {
  /// Sets a header for the Interaction. Returns false if the interaction or Pact can't be
  /// modified (i.e. the mock server for it has already started). Note that this function will
  /// overwrite any previously set header values. Also, this function will not process the value in
  /// any way, so matching rules and generators can not be configured with it.
  ///
  /// If matching rules are required to be set, use `pactffi_with_header_v2`.
  ///
  /// * `part` - The part of the interaction to add the header to (Request or Response).
  /// * `name` - the header name.
  /// * `value` - the header value.
  ///
  /// # Safety
  /// The name and value parameters must be valid pointers to NULL terminated strings.
  fn pactffi_set_header(
    interaction: InteractionHandle,
    part: InteractionPart,
    name: *const c_char,
    value: *const c_char
  ) -> bool {
    if let Some(name) = convert_cstr("name", name) {
      let value = convert_cstr("value", value).unwrap_or_default();
      interaction.with_interaction(&|_, mock_server_started, inner| {
        if let Some(reqres) = inner.as_v4_http_mut() {
          match part {
            InteractionPart::Request => reqres.request.set_header(name, &[value]),
            InteractionPart::Response => reqres.response.set_header(name, &[value])
          };

          !mock_server_started
        } else {
          error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
          false
        }
      }).unwrap_or(false)
    } else {
      warn!("Ignoring header with empty or null name");
      false
    }
  } {
    false
  }
}

/// Configures the response for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `status` - the response status. Defaults to 200.
#[no_mangle]
pub extern fn pactffi_response_status(interaction: InteractionHandle, status: c_ushort) -> bool {
  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      reqres.response.status = status;
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Configures the response for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `status` - the response status. Defaults to 200.
///
/// To include matching rules for the status (only statusCode or integer really makes sense to use), include the
/// matching rule JSON format with the value as a single JSON document. I.e.
///
/// ```c
/// const char* status = "{ \"pact:generator:type\": \"RandomInt\", \"min\": 100, \"max\": 399, \"pact:matcher:type\":\"statusCode\", \"status\": \"nonError\"}";
/// pactffi_response_status_v2(handle, status);
/// ```
/// See [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
///
/// # Safety
/// The status parameter must be valid pointers to NULL terminated strings.
#[no_mangle]
pub extern fn pactffi_response_status_v2(interaction: InteractionHandle, status: *const c_char) -> bool {
  let status = convert_cstr("status", status).unwrap_or("200");
  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      let status = from_integration_json_v2(&mut reqres.response.matching_rules,
        &mut reqres.response.generators, &status.to_string(), DocPath::empty(), "status", 0);
      reqres.response.status = match status {
        Either::Left(value) => value.parse().unwrap_or_default(),
        Either::Right(values) => {
          warn!("Received multiple values for the status ({:?}), will only use the first one", values);
          values.first().cloned().unwrap_or_default().parse().unwrap_or_default()
        }
      };
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Adds the body for the interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the body to (Request or Response).
/// * `content_type` - The content type of the body. Defaults to `text/plain`. Will be ignored if a content type
///   header is already set.
/// * `body` - The body contents. For JSON payloads, matching rules can be embedded in the body. See
/// [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
///
/// For HTTP and async message interactions, this will overwrite the body. With asynchronous messages, the
/// part parameter will be ignored. With synchronous messages, the request contents will be overwritten,
/// while a new response will be appended to the message.
///
/// # Safety
///
/// The interaction contents and content type must either be NULL pointers, or point to valid
/// UTF-8 encoded NULL-terminated strings. Otherwise, behaviour is undefined.
///
/// # Error Handling
///
/// If the contents is a NULL pointer, it will set the body contents as null. If the content
/// type is a null pointer, or can't be parsed, it will set the content type as TEXT.
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started) or an error has occurred.
#[no_mangle]
pub extern fn pactffi_with_body(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  body: *const c_char
) -> bool {
  trace!(">>> pactffi_with_body({:?}, {:?}, {:?}, {:?})", interaction, part, content_type, body);
  let content_type = convert_cstr("content_type", content_type).unwrap_or("text/plain");
  let body = convert_cstr("body", body).unwrap_or_default();
  let content_type_header = "Content-Type".to_string();

  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      match part {
        InteractionPart::Request => {
          trace!("Setting up the request body");
          if !reqres.request.has_header(&content_type_header) {
            match reqres.request.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.request.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
              }
            }
          }
          let body = if reqres.request.content_type().unwrap_or_default().is_json() {
            let category = reqres.request.matching_rules.add_category("body");
            OptionalBody::Present(Bytes::from(process_json(body.to_string(), category, &mut reqres.request.generators)),
                                  Some(ContentType::parse(content_type).unwrap()), None)
          } else if reqres.request.content_type().unwrap_or_default().is_xml() {
            // Try detect the intermediate JSON format
            trace!("Content type is XML, try sniff the provided body format");
            if let Some(ct) = detect_content_type_from_string(body) {
              trace!("Detected body body format is {}", ct);
              if ct.is_json() {
                // Process the intermediate JSON into XML
                trace!("Body is in JSON format, processing the intermediate JSON into XML");
                let category = reqres.request.matching_rules.add_category("body");
                OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut reqres.request.generators).unwrap_or(vec![])),
                                      Some(XML.clone()), None)
              } else {
                // Assume raw XML
                OptionalBody::from(body)
              }
            } else {
              // Assume raw XML
              OptionalBody::from(body)
            }
          } else {
            OptionalBody::from(body)
          };
          reqres.request.body = body;
        },
        InteractionPart::Response => {
          trace!("Setting up the response body");
          if !reqres.response.has_header(&content_type_header) {
            match reqres.response.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.response.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
              }
            }
          }
          let body = if reqres.response.content_type().unwrap_or_default().is_json() {
            let category = reqres.response.matching_rules.add_category("body");
            OptionalBody::Present(Bytes::from(process_json(body.to_string(), category, &mut reqres.response.generators)),
                                  Some(JSON.clone()), None)
          } else if reqres.response.content_type().unwrap_or_default().is_xml() {
            trace!("Content type is XML, try sniff the provided body format");
            // Try detect the intermediate JSON format
            if let Some(ct) = detect_content_type_from_string(body) {
              trace!("Detected body body format is {}", ct);
              if ct.is_json() {
                // Process the intermediate XML into JSON
                trace!("Body is in JSON format, processing the intermediate JSON into XML");
                let category = reqres.response.matching_rules.add_category("body");
                OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut reqres.response.generators).unwrap_or(vec![])),
                                      Some(XML.clone()), None)
              } else {
                // Assume raw XML
                OptionalBody::from(body)
              }
            } else {
              // Assume raw XML
              OptionalBody::from(body)
            }
          } else {
            OptionalBody::from(body)
          };
          reqres.response.body = body;
        }
      };
      !mock_server_started
    } else if let Some(message) = inner.as_v4_async_message_mut() {
      let ct = ContentType::parse(content_type).unwrap_or_else(|_| TEXT.clone());
      let body = if ct.is_json() {
        let category = message.contents.matching_rules.add_category("body");
        OptionalBody::Present(Bytes::from(process_json(body.to_string(), category, &mut message.contents.generators)),
                              Some(JSON.clone()), None)
      } else if ct.is_xml() {
        let category = message.contents.matching_rules.add_category("body");
        OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut message.contents.generators).unwrap_or(vec![])),
                              Some(XML.clone()), None)
      } else {
        OptionalBody::from(body)
      };
      message.contents.contents = body;
      message.contents.metadata.insert("contentType".to_string(), json!(content_type));
      true
    } else if let Some(message) = inner.as_v4_sync_message_mut() {
      let ct = ContentType::parse(content_type).unwrap_or_else(|_| TEXT.clone());
      match part {
        InteractionPart::Request => {
          let category = message.request.matching_rules.add_category("body");
          let body = if ct.is_json() {
            OptionalBody::Present(Bytes::from(process_json(body.to_string(), category, &mut message.request.generators)),
                                  Some(JSON.clone()), None)
          } else if ct.is_xml() {
            OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut message.request.generators).unwrap_or(vec![])),
                                  Some(XML.clone()), None)
          } else {
            OptionalBody::from(body)
          };
          message.request.contents = body;
          message.request.metadata.insert("contentType".to_string(), json!(content_type));
        }
        InteractionPart::Response => {
          let mut response = MessageContents::default();
          let category = response.matching_rules.add_category("body");
          let body = if ct.is_json() {
            OptionalBody::Present(Bytes::from(process_json(body.to_string(), category, &mut response.generators)),
                                  Some(JSON.clone()), None)
          } else if ct.is_xml() {
            OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut response.generators).unwrap_or(vec![])),
                                  Some(XML.clone()), None)
          } else {
            OptionalBody::from(body)
          };
          response.contents = body;
          response.metadata.insert("contentType".to_string(), json!(content_type));
          message.response.push(response);
        }
      }
      true
    } else {
      error!("Interaction is an unknown type, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Adds the body for the interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the body to (Request or Response).
/// * `content_type` - The content type of the body. Defaults to `application/octet-stream` if it
///   is NULL. Will be ignored if a content type header is already set.
/// * `body` - Body contents as a pointer to a byte array
/// * `size` - Number of bytes in the body
///
/// For HTTP and async message interactions, this will overwrite the body. With asynchronous messages, the
/// part parameter will be ignored. With synchronous messages, the request contents will be overwritten,
/// while a new response will be appended to the message.
///
/// # Safety
///
/// This function is safe to use as long as the following conditions are true:
/// The content type must either be a NULL pointer, or point to valid UTF-8 encoded NULL-terminated
/// string. The body pointer must be valid for reads of `size` bytes, and it must be properly
/// aligned and consecutive (that just means it must point a continuous array of at least `size`
/// bytes that can be read in a single operation and not to non-continuous structures like linked
/// lists, etc.).
///
/// # Error Handling
///
/// If the body is a NULL pointer, it will set the body contents as empty. If the content
/// type is a null pointer, it will set the content type as `application/octet-stream`.
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started) or an error has occurred.
#[no_mangle]
pub extern fn pactffi_with_binary_body(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  body: *const u8,
  size: size_t
) -> bool {
  trace!(">>> pactffi_with_binary_body({:?}, {:?}, {:?}, {:?}, {})", interaction, part, content_type, body, size);
  let content_type = convert_cstr("content_type", content_type)
    .unwrap_or("application/octet-stream");
  let content_type_header = "Content-Type".to_string();

  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      match part {
        InteractionPart::Request => {
          reqres.request.body = convert_ptr_to_body(body, size, ContentType::parse(content_type).ok());
          if !reqres.request.has_header(&content_type_header) {
            match reqres.request.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.request.headers = Some(hashmap! { content_type_header.clone() => vec![content_type.to_string()]});
              }
            }
          };
        },
        InteractionPart::Response => {
          reqres.response.body = convert_ptr_to_body(body, size, ContentType::parse(content_type).ok());
          if !reqres.response.has_header(&content_type_header) {
            match reqres.response.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.response.headers = Some(hashmap! { content_type_header.clone() => vec![content_type.to_string()]});
              }
            }
          }
        }
      };
      !mock_server_started
    } else if let Some(message) = inner.as_v4_async_message_mut() {
      message.contents.contents = convert_ptr_to_body(body, size, ContentType::parse(content_type).ok());
      message.contents.metadata.insert("contentType".to_string(), json!(content_type));
      true
    } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
      match part {
        InteractionPart::Request => {
          sync_message.request.contents = convert_ptr_to_body(body, size, ContentType::parse(content_type).ok());
          sync_message.request.metadata.insert("contentType".to_string(), json!(content_type));
        },
        InteractionPart::Response => {
          let mut response = MessageContents::default();
          response.contents = convert_ptr_to_body(body, size, ContentType::parse(content_type).ok());
          response.metadata.insert("contentType".to_string(), json!(content_type));
          sync_message.response.push(response);
        }
      };
      true
    } else {
      error!("Interaction is an unknown type, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Adds a binary file as the body with the expected content type and example contents. Will use
/// a mime type matcher to match the body. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `interaction` - Interaction handle to set the body for.
/// * `part` - Request or response part.
/// * `content_type` - Expected content type.
/// * `body` - example body contents in bytes
/// * `size` - number of bytes in the body
///
/// For HTTP and async message interactions, this will overwrite the body. With asynchronous messages, the
/// part parameter will be ignored. With synchronous messages, the request contents will be overwritten,
/// while a new response will be appended to the message.
///
/// # Safety
///
/// The content type must be a valid UTF-8 encoded NULL-terminated string. The body pointer must
/// be valid for reads of `size` bytes, and it must be properly aligned and consecutive.
///
/// # Error Handling
///
/// If the body is a NULL pointer, it will set the body contents as null. If the content
/// type is a null pointer, or can't be parsed, it will return false.
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started) or an error has occurred.
#[no_mangle]
pub extern fn pactffi_with_binary_file(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  body: *const u8,
  size: size_t
) -> bool {
  trace!(">>> pactffi_with_binary_file({:?}, {:?}, {:?}, {:?}, {})", interaction, part, content_type, body, size);
  let content_type_header = "Content-Type".to_string();
  let support_content_type_matching_rule = interaction.with_pact(
    &|_, pact| pact.specification_version >= PactSpecification::V3
  ).unwrap_or(false);
  match convert_cstr("content_type", content_type) {
    Some(content_type) => {
      interaction.with_interaction(&|_, mock_server_started, inner| {
        if let Some(reqres) = inner.as_v4_http_mut() {
          match part {
            InteractionPart::Request => {
              reqres.request.body = convert_ptr_to_body(body, size, None);
              if !reqres.request.has_header(&content_type_header) {
                match reqres.request.headers {
                  Some(ref mut headers) => {
                    headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
                  },
                  None => {
                    reqres.request.headers = Some(hashmap! { content_type_header.clone() => vec![content_type.to_string()]});
                  }
                }
              };
              add_content_type_matching_rule_to_body(support_content_type_matching_rule, &mut reqres.request.matching_rules, content_type);
            },
            InteractionPart::Response => {
              reqres.response.body = convert_ptr_to_body(body, size, None);
              if !reqres.response.has_header(&content_type_header) {
                match reqres.response.headers {
                  Some(ref mut headers) => {
                    headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
                  },
                  None => {
                    reqres.response.headers = Some(hashmap! { content_type_header.clone() => vec![content_type.to_string()]});
                  }
                }
              }
              add_content_type_matching_rule_to_body(support_content_type_matching_rule, &mut reqres.response.matching_rules, content_type);
            }
          };
          !mock_server_started
        } else if let Some(message) = inner.as_v4_async_message_mut() {
          message.contents.contents = convert_ptr_to_body(body, size, None);
          add_content_type_matching_rule_to_body(support_content_type_matching_rule, &mut message.contents.matching_rules, content_type);
          message.contents.metadata.insert("contentType".to_string(), json!(content_type));
          true
        } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
          match part {
            InteractionPart::Request => {
              sync_message.request.contents = convert_ptr_to_body(body, size, None);
              add_content_type_matching_rule_to_body(support_content_type_matching_rule, &mut sync_message.request.matching_rules, content_type);
              sync_message.request.metadata.insert("contentType".to_string(), json!(content_type));
            },
            InteractionPart::Response => {
              let mut response = MessageContents::default();
              response.contents = convert_ptr_to_body(body, size, None);
              add_content_type_matching_rule_to_body(support_content_type_matching_rule, &mut response.matching_rules, content_type);
              response.metadata.insert("contentType".to_string(), json!(content_type));
              sync_message.response.push(response);
            }
          };
          true
        } else {
          error!("Interaction is an unknown type, is {}", inner.type_of());
          false
        }
      }).unwrap_or(false)
    },
    None => {
      error!("with_binary_file: Content type value is not valid (NULL or non-UTF-8)");
      false
    }
  }
}

ffi_fn!{
  /// Add matching rules to the interaction.
  ///
  /// * `interaction` - Interaction handle to set the matching rules for.
  /// * `part` - Request or response part (if applicable).
  /// * `rules` - JSON string of the matching rules to add to the interaction.
  ///
  /// This function can be called multiple times, in which case the matching
  /// rules will be merged. The function will return `true` if the rules were
  /// successfully added, and `false` if an error occurred.
  ///
  /// For synchronous messages which allow multiple responses, the matching
  /// rules will be added to all the responses.
  ///
  /// # Safety
  ///
  /// The rules parameter must be a valid pointer to a NULL terminated UTF-8
  /// string.
  fn pactffi_with_matching_rules(
    interaction: InteractionHandle,
  part: InteractionPart,
    rules: *const c_char
  ) -> bool {
    let rules = match convert_cstr("rules", rules) {
      Some(rules) => rules,
      None => {
        error!("with_matching_rules: Rules value is not valid (NULL or non-UTF-8)");
        return Ok(false);
      }
    };

    let rules = match serde_json::from_str::<Value>(rules) {
      Ok(Value::Object(rules)) => rules,
      Ok(_) => {
        error!("with_matching_rules: Rules value is not a JSON object");
        return Ok(false);
      },
      Err(err) => {
        error!("with_matching_rules: Failed to parse the matching rules: {}", err);
        return Ok(false);
      }
    };

    // Wrap the rules in a object with a "matchingRules" key if it is not
    // already, as this is required for `matchers_from_json`.
    let rules = if rules.contains_key("matchingRules") {
      Value::Object(rules)
    } else {
      json!({ "matchingRules": rules })
    };
    let rules = match matchers_from_json(&rules, &None) {
      Ok(rules) => rules,
      Err(err) => {
        error!("with_matching_rules: Failed to load the matching rules: {}", err);
        return Ok(false);
      }
    };

    interaction.with_interaction(&move |_, _, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        match part {
          InteractionPart::Request => reqres.request.matching_rules_mut().merge(&rules),
          InteractionPart::Response => reqres.response.matching_rules_mut().merge(&rules)
        };
        Ok(())
      } else if let Some(message) = inner.as_v4_async_message_mut() {
        message.matching_rules_mut().merge(&rules);
        Ok(())
      } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
        match part {
          InteractionPart::Request => sync_message.request.matching_rules_mut().merge(&rules),
          InteractionPart::Response => sync_message.response.iter_mut().for_each(|response| response.matching_rules_mut().merge(&rules))
        };
        Ok(())
      } else {
        error!("Interaction is an unknown type, is {}", inner.type_of());
        Err(())
      }
    }).unwrap_or(Err(())).is_ok()
  }
  // Failure block
  {
    false
  }
}

fn add_content_type_matching_rule_to_body(is_supported: bool, matching_rules: &mut MatchingRules, content_type: &str) {
  if is_supported {
    matching_rules.add_category("body").add_rule(
      DocPath::root(), MatchingRule::ContentType(content_type.into()), RuleLogic::And);
  }
}

/// Adds a binary file as the body as a MIME multipart with the expected content type and example contents. Will use
/// a mime type matcher to match the body. Returns an error if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started) or an error occurs.
///
/// * `interaction` - Interaction handle to set the body for.
/// * `part` - Request or response part.
/// * `content_type` - Expected content type of the file.
/// * `file` - path to the example file
/// * `part_name` - name for the mime part
/// * `boundary` - boundary for the multipart separation
///
/// This function can be called multiple times. In that case, each subsequent call will be
/// appended to the existing multipart body as a new part.
///
/// # Safety
///
/// The content type, file path and part name must be valid pointers to UTF-8 encoded NULL-terminated strings.
/// Passing invalid pointers or pointers to strings that are not NULL terminated will lead to undefined
/// behaviour.
///
/// # Error Handling
///
/// If the boundary is a NULL pointer, a random string will be used.
/// If the file path is a NULL pointer, it will set the body contents as as an empty mime-part.
/// If the file path does not point to a valid file, or is not able to be read, it will return an
/// error result. If the content type is a null pointer, or can't be parsed, it will return an error result.
/// Returns an error if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started), the interaction is not an HTTP interaction or some other error occurs.
#[no_mangle]
pub extern fn pactffi_with_multipart_file_v2(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  file: *const c_char,
  part_name: *const c_char,
  boundary: *const c_char
) -> StringResult {
  let part_name = convert_cstr("part_name", part_name).unwrap_or("file");
  match convert_cstr("content_type", content_type) {
    Some(content_type) => {
      let result = interaction.with_interaction(&|_, mock_server_started, inner| {
        match convert_ptr_to_mime_part_body(file, part_name) {
          Ok(body) => {
            if let Some(reqres) = inner.as_v4_http_mut() {
              let (body, boundary) = match convert_cstr("boundary", boundary) {
                Some(boundary) => {
                  let part = part_body_replace_marker(&body.body, body.boundary.as_str(), boundary);
                  let body = OptionalBody::Present(part, body.body.content_type(), get_content_type_hint(&body.body));
                  (body, boundary)
                },
                None => {
                  (body.body, body.boundary.as_str())
                }
              };
              match part {
                InteractionPart::Request => request_multipart(&mut reqres.request, boundary, body, content_type, part_name),
                InteractionPart::Response => response_multipart(&mut reqres.response, boundary, body, content_type, part_name)
              };
              if mock_server_started {
                Err("with_multipart_file: This Pact can not be modified, as the mock server has already started".to_string())
              } else {
                Ok(())
              }
            } else {
              error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
              Err(format!("with_multipart_file: Interaction is not an HTTP interaction, is {}", inner.type_of()))
            }
          },
          Err(err) => Err(format!("with_multipart_file: failed to generate multipart body - {}", err))
        }
      });
      match result {
        Some(inner_result) => match inner_result {
          Ok(_) => StringResult::Ok(null_mut()),
          Err(err) => {
            let error = CString::new(err).unwrap();
            StringResult::Failed(error.into_raw())
          }
        },
        None => {
          let error = CString::new("with_multipart_file: Interaction handle is invalid").unwrap();
          StringResult::Failed(error.into_raw())
        }
      }
    },
    None => {
      error!("with_multipart_file: Content type value is not valid (NULL or non-UTF-8)");
      let error = CString::new("with_multipart_file: Content type value is not valid (NULL or non-UTF-8)").unwrap();
      StringResult::Failed(error.into_raw())
    }
  }
}

/// Adds a binary file as the body as a MIME multipart with the expected content type and example contents. Will use
/// a mime type matcher to match the body. Returns an error if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started) or an error occurs.
///
/// * `interaction` - Interaction handle to set the body for.
/// * `part` - Request or response part.
/// * `content_type` - Expected content type of the file.
/// * `file` - path to the example file
/// * `part_name` - name for the mime part
///
/// This function can be called multiple times. In that case, each subsequent call will be
/// appended to the existing multipart body as a new part.
///
/// # Safety
///
/// The content type, file path and part name must be valid pointers to UTF-8 encoded NULL-terminated strings.
/// Passing invalid pointers or pointers to strings that are not NULL terminated will lead to undefined
/// behaviour.
///
/// # Error Handling
///
/// If the file path is a NULL pointer, it will set the body contents as as an empty mime-part.
/// If the file path does not point to a valid file, or is not able to be read, it will return an
/// error result. If the content type is a null pointer, or can't be parsed, it will return an error result.
/// Returns an error if the interaction or Pact can't be modified (i.e. the mock server for it has
/// already started), the interaction is not an HTTP interaction or some other error occurs.
#[no_mangle]
pub extern fn pactffi_with_multipart_file(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  file: *const c_char,
  part_name: *const c_char
) -> StringResult {
  pactffi_with_multipart_file_v2(interaction, part, content_type, file, part_name, std::ptr::null())
}

ffi_fn!{
  /// Sets the key attribute for the interaction.
  ///
  /// * `interaction` - Interaction handle to modify.
  /// * `value` - Key value. This must be a valid UTF-8 null-terminated string,
  ///   or NULL to clear the key.
  ///
  /// This function will return `true` if the key was successfully updated.
  ///
  /// # Safety
  ///
  /// The key parameter must be a valid pointer to a NULL terminated UTF-8, or
  /// NULL if the key is to be cleared.
  fn pactffi_set_key(interaction: InteractionHandle, value: *const c_char) -> bool {
    let value = if value.is_null() {
      None
    } else {
      match convert_cstr("value", value) {
        Some(value) => Some(value.to_string()),
        None => {
          error!("set_key: Value is not valid (NULL or non-UTF-8)");
          return Err(anyhow!("Value is not valid (NULL or non-UTF-8)"));
        }
      }
    };

    interaction.with_interaction(&|_, _, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        reqres.key = value.clone();
        Ok(())
      } else if let Some(message) = inner.as_v4_async_message_mut() {
        message.key = value.clone();
        Ok(())
      } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
        sync_message.key = value.clone();
        Ok(())
      } else {
        error!("Interaction is an unknown type, is {}", inner.type_of());
        Err(anyhow!("Interaction is an unknown type, is {}", inner.type_of()))
      }
    }).unwrap_or(Err(anyhow!("Not value to unwrap"))).is_ok()
  } {
    false
  }
}

ffi_fn!{
  /// Mark the interaction as pending.
  ///
  /// * `interaction` - Interaction handle to modify.
  /// * `pending` - Boolean value to toggle the pending state of the interaction.
  ///
  /// This function will return `true` if the key was successfully updated.
  fn pactffi_set_pending(interaction: InteractionHandle, pending: bool) -> bool {
    interaction.with_interaction(&|_, _, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        reqres.pending = pending;
        Ok(())
      } else if let Some(message) = inner.as_v4_async_message_mut() {
        message.pending = pending;
        Ok(())
      } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
        sync_message.pending = pending;
        Ok(())
      } else {
        error!("Interaction is an unknown type, is {}", inner.type_of());
        Err(anyhow!("Interaction is an unknown type, is {}", inner.type_of()))
      }
    }).unwrap_or(Err(anyhow!("Not value to unwrap"))).is_ok()
  } {
    false
  }
}

ffi_fn!{
  /// Add a comment to the interaction.
  ///
  /// * `interaction` - Interaction handle to set the comments for.
  /// * `key` - Key value
  /// * `value` - Comment value. This may be any valid JSON value, or a NULL to
  ///   clear the comment.
  ///
  /// This function will return `true` if the comments were successfully
  /// updated. Both `key` and `value` must be valid UTF-8 null-terminated
  /// strings; or in the case of `value`, it may also be a NULL pointer in which
  /// case the comment will be cleared.
  ///
  /// Note that a `value` that deserialize to a JSON null will result in a
  /// comment being added, with the value being the JSON null.
  ///
  /// # Safety
  ///
  /// The comments parameter must be a valid pointer to a NULL terminated UTF-8,
  /// or NULL if the comment is to be cleared.
  fn pactffi_set_comment(interaction: InteractionHandle, key: *const c_char, value: *const c_char) -> bool {
    let key = match convert_cstr("key", key) {
      Some(key) => key,
      None => {
        error!("set_comments: Key value is not valid (NULL or non-UTF-8)");
        return Err(anyhow!("Key value is not valid (NULL or non-UTF-8)"));
      }
    };
    let value: Option<serde_json::Value> = if value.is_null() {
      None
    } else {
      match convert_cstr("value", value) {
        Some(value) =>{ match serde_json::from_str(value) {
          Ok(value) => Some(value),
          Err(_) => Some(serde_json::Value::String(value.to_string())),
        }},
        None => {
          error!("set_comments: Value is not valid (non-UTF-8)");
          return Err(anyhow!("Value is not valid (non-UTF-8)"));
        }
      }
    };

    interaction.with_interaction(&|_, _, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        match &value {
          Some(value) => reqres.comments.insert(key.to_string(), value.clone()),
          None => reqres.comments.remove(key)
        };
        Ok(())
      } else if let Some(message) = inner.as_v4_async_message_mut() {
        match &value {
          Some(value) => message.comments.insert(key.to_string(), value.clone()),
          None => message.comments.remove(key)
        };
        Ok(())
      } else if let Some(sync_message) = inner.as_v4_sync_message_mut() {
        match &value {
          Some(value) => sync_message.comments.insert(key.to_string(), value.clone()),
          None => sync_message.comments.remove(key)
        };
        Ok(())
      } else {
        error!("Interaction is an unknown type, is {}", inner.type_of());
        Err(anyhow!("Interaction is an unknown type, is {}", inner.type_of()))
      }
    }).unwrap_or(Err(anyhow!("Not value to unwrap"))).is_ok()
  } {
    false
  }
}

fn convert_ptr_to_body(body: *const u8, size: size_t, content_type: Option<ContentType>) -> OptionalBody {
  if body.is_null() {
    OptionalBody::Null
  } else if size == 0 {
    OptionalBody::Empty
  } else {
    OptionalBody::Present(Bytes::from(unsafe { std::slice::from_raw_parts(body, size) }), content_type, None)
  }
}

fn convert_ptr_to_mime_part_body(file: *const c_char, part_name: &str) -> Result<MultipartBody, String> {
  if file.is_null() {
    empty_multipart_body()
  } else {
    let c_str = unsafe { CStr::from_ptr(file) };
    let file = match c_str.to_str() {
      Ok(str) => Ok(str),
      Err(err) => {
        error!("convert_ptr_to_mime_part_body: Failed to parse file name as a UTF-8 string: {}", err);
        Err(format!("convert_ptr_to_mime_part_body: Failed to parse file name as a UTF-8 string: {}", err))
      }
    }?;
    file_as_multipart_body(file, part_name)
  }
}

ffi_fn! {
    /// Get an iterator over all the messages of the Pact. The returned iterator needs to be
    /// freed with `pactffi_pact_message_iter_delete`.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the Pact, so it is always safe to use.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain embedded
    /// null ('\0') bytes.
    fn pactffi_pact_handle_get_message_iter(pact: PactHandle) -> *mut PactMessageIterator {
        let message_pact = pact.with_pact(&|_, inner| {
          // Ok to unwrap this, as the worse case given an HTTP Pact it will return a new message
          // pact with no messages
          inner.pact.as_message_pact().unwrap()
        }).ok_or_else(|| anyhow!("Pact handle is not valid"))?;
        let iter = PactMessageIterator::new(message_pact);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over all the synchronous request/response messages of the Pact.
    /// The returned iterator needs to be freed with `pactffi_pact_sync_message_iter_delete`.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the Pact, so it is always safe to use.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain embedded
    /// null ('\0') bytes.
    fn pactffi_pact_handle_get_sync_message_iter(pact: PactHandle) -> *mut PactSyncMessageIterator {
        let v4_pact = pact.with_pact(&|_, inner| {
          // Ok to unwrap this, as any non-v4 pact will be upgraded
          inner.pact.as_v4_pact().unwrap()
        }).ok_or_else(|| anyhow!("Pact handle is not valid"))?;
        let iter = PactSyncMessageIterator::new(v4_pact);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over all the synchronous HTTP request/response interactions of the Pact.
    /// The returned iterator needs to be freed with `pactffi_pact_sync_http_iter_delete`.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the Pact, so it is always safe to use.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain embedded
    /// null ('\0') bytes.
    fn pactffi_pact_handle_get_sync_http_iter(pact: PactHandle) -> *mut PactSyncHttpIterator {
        let v4_pact = pact.with_pact(&|_, inner| {
          // Ok to unwrap this, as any non-v4 pact will be upgraded
          inner.pact.as_v4_pact().unwrap()
        }).ok_or_else(|| anyhow!("Pact handle is not valid"))?;
        let iter = PactSyncHttpIterator::new(v4_pact);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

/// Creates a new Pact Message model and returns a handle to it.
///
/// * `consumer_name` - The name of the consumer for the pact.
/// * `provider_name` - The name of the provider for the pact.
///
/// Returns a new `MessagePactHandle`. The handle will need to be freed with the `pactffi_free_message_pact_handle`
/// function to release its resources.
#[no_mangle]
pub extern fn pactffi_new_message_pact(consumer_name: *const c_char, provider_name: *const c_char) -> MessagePactHandle {
  let consumer = convert_cstr("consumer_name", consumer_name).unwrap_or("Consumer");
  let provider = convert_cstr("provider_name", provider_name).unwrap_or("Provider");
  MessagePactHandle::new(consumer, provider)
}

/// Creates a new Message and returns a handle to it.
///
/// * `description` - The message description. It needs to be unique for each Message.
///
/// Returns a new `MessageHandle`.
#[no_mangle]
pub extern fn pactffi_new_message(pact: MessagePactHandle, description: *const c_char) -> MessageHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner, _| {
      let message = AsynchronousMessage {
        description: description.to_string(),
        ..AsynchronousMessage::default()
      };
      inner.interactions.push(message.boxed_v4());
      MessageHandle::new(pact, inner.interactions.len() as u16)
    }).unwrap_or_else(|| MessageHandle::new(pact, 0))
  } else {
    MessageHandle::new(pact, 0)
  }
}

/// Sets the description for the Message.
///
/// * `description` - The message description. It needs to be unique for each message.
#[no_mangle]
pub extern fn pactffi_message_expects_to_receive(message: MessageHandle, description: *const c_char) {
  if let Some(description) = convert_cstr("description", description) {
    message.with_message(&|_, inner, _| {
      inner.set_description(description);
    });
  }
}

/// Adds a provider state to the Interaction.
///
/// * `description` - The provider state description. It needs to be unique for each message
#[no_mangle]
pub extern fn pactffi_message_given(message: MessageHandle, description: *const c_char) {
  if let Some(description) = convert_cstr("description", description) {
    message.with_message(&|_, inner, _| {
      inner.provider_states_mut().push(ProviderState::default(&description.to_string()));
    });
  }
}

/// Adds a provider state to the Message with a parameter key and value.
///
/// * `description` - The provider state description. It needs to be unique.
/// * `name` - Parameter name.
/// * `value` - Parameter value.
#[no_mangle]
pub extern fn pactffi_message_given_with_param(message: MessageHandle, description: *const c_char,
                                               name: *const c_char, value: *const c_char) {
  if let Some(description) = convert_cstr("description", description) {
    if let Some(name) = convert_cstr("name", name) {
      let value = convert_cstr("value", value).unwrap_or_default();
      message.with_message(&|_, inner, _| {
        let value = match serde_json::from_str(value) {
          Ok(json) => json,
          Err(_) => json!(value)
        };
        match inner.provider_states().iter().find_position(|state| state.name == description) {
          Some((index, _)) => {
            inner.provider_states_mut().get_mut(index).unwrap().params.insert(name.to_string(), value);
          },
          None => inner.provider_states_mut().push(ProviderState {
            name: description.to_string(),
            params: hashmap!{ name.to_string() => value }
          })
        };
      });
    }
  }
}

/// Adds the contents of the Message.
///
/// Accepts JSON, binary and other payload types. Binary data will be base64 encoded when serialised.
///
/// Note: For text bodies (plain text, JSON or XML), you can pass in a C string (NULL terminated)
/// and the size of the body is not required (it will be ignored). For binary bodies, you need to
/// specify the number of bytes in the body.
///
/// * `content_type` - The content type of the body. Defaults to `text/plain`, supports JSON structures with matchers and binary data.
/// * `body` - The body contents as bytes. For text payloads (JSON, XML, etc.), a C string can be used and matching rules can be embedded in the body.
/// * `content_type` - Expected content type (e.g. application/json, application/octet-stream)
/// * `size` - number of bytes in the message body to read. This is not required for text bodies (JSON, XML, etc.).
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern fn pactffi_message_with_contents(message_handle: MessageHandle, content_type: *const c_char, body: *const u8, size: size_t) {
  let content_type = convert_cstr("content_type", content_type).unwrap_or("text/plain");
  trace!("pactffi_message_with_contents(message_handle: {:?}, content_type: {:?}, body: {:?}, size: {})", message_handle, content_type, body, size);

  message_handle.with_message(&|_, inner, _| {
    let content_type = ContentType::parse(content_type).ok();

    if let Some(message) = inner.as_v4_async_message_mut() {
      let body = if let Some(content_type) = content_type {
        let category = message.contents.matching_rules.add_category("body");
        let body_str = convert_cstr("body", body as *const c_char).unwrap_or_default();

        if content_type.is_xml() {
          OptionalBody::Present(Bytes::from(process_xml(body_str.to_string(), category, &mut message.contents.generators).unwrap_or(vec![])), Some(content_type), None)
        } else if content_type.is_text() || content_type.is_json() {
          OptionalBody::Present(Bytes::from(process_json(body_str.to_string(), category, &mut message.contents.generators)), Some(content_type), None)
        } else {
          OptionalBody::Present(Bytes::from(unsafe { std::slice::from_raw_parts(body, size) }), Some(content_type), None)
        }
      } else {
        OptionalBody::Present(Bytes::from(unsafe { std::slice::from_raw_parts(body, size) }), None, None)
      };

      message.contents.contents = body;
    }
  });
}

/// Adds expected metadata to the Message
///
/// * `key` - metadata key
/// * `value` - metadata value.
#[no_mangle]
pub extern fn pactffi_message_with_metadata(message_handle: MessageHandle, key: *const c_char, value: *const c_char) {
  if let Some(key) = convert_cstr("key", key) {
    let value = convert_cstr("value", value).unwrap_or_default();
    message_handle.with_message(&|_, inner, _| {
      if let Some(message) = inner.as_v4_async_message_mut() {
        message.contents.metadata.insert(key.to_string(), Value::String(value.to_string()));
      }
    });
  }
}

/// Adds expected metadata to the Message
///
/// * `key` - metadata key
/// * `value` - metadata value, supports JSON structures with matchers and generators
///
/// To include matching rules for the value, include the
/// matching rule JSON format with the value as a single JSON document. I.e.
///
/// ```c
/// const char* value = "{\"value\": { \"ID\": \"sjhdjkshsdjh\", \"weight\": 100.5 }, \"pact:matcher:type\":\"type\"}";
/// pactffi_message_with_metadata_v2(handle, "TagData", value);
/// ```
/// See [IntegrationJson.md](https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/IntegrationJson.md)
///
/// # Safety
/// The key and value parameters must be valid pointers to NULL terminated strings.
#[no_mangle]
pub extern fn pactffi_message_with_metadata_v2(message_handle: MessageHandle, key: *const c_char, value: *const c_char) {
  if let Some(key) = convert_cstr("key", key) {
    let value = convert_cstr("value", value).unwrap_or_default();
    trace!("pactffi_message_with_metadata_v2(message_handle: {:?}, key: {:?}, value: {})", message_handle, key, value);

    message_handle.with_message(&|_, inner, _| {
      if let Some(message) = inner.as_v4_async_message_mut() {
        let matching_rules = message.contents.matching_rules.add_category(Category::METADATA);
        let generators = &mut message.contents.generators;
        let value = match serde_json::from_str(value) {
          Ok(json) => match json {
            Value::Object(ref map) => process_object(map, matching_rules, generators, DocPath::new(key).unwrap(), false),
            Value::Array(ref array) => process_array(array, matching_rules, generators, DocPath::new(key).unwrap(), false, false),
            Value::Null => Value::Null,
            Value::String(string) => Value::String(string),
            Value::Bool(bool) => Value::Bool(bool),
            Value::Number(number) => Value::Number(number),
          },
          Err(err) => {
            warn!("Failed to parse metadata value '{}' as JSON - {}. Will treat it as string", value, err);
            Value::String(value.to_string())
          }
        };
        message.contents.metadata.insert(key.to_string(), value);
      }
    });
  }
}

/// Reifies the given message
///
/// Reification is the process of stripping away any matchers, and returning the original contents.
///
/// # Safety
///
/// The returned string needs to be deallocated with the `free_string` function.
/// This function must only ever be called from a foreign language. Calling it from a Rust function
/// that has a Tokio runtime in its call stack can result in a deadlock.
#[no_mangle]
pub extern fn pactffi_message_reify(message_handle: MessageHandle) -> *const c_char {
  let res = message_handle.with_message(&|_, inner, spec_version| {
    trace!("pactffi_message_reify(message: {:?}, spec_version: {})", inner, spec_version);
    if let Some(message) = inner.as_v4_async_message() {
      match message.contents.contents {
        OptionalBody::Null => "null".to_string(),
        OptionalBody::Present(_, _, _) => if spec_version <= pact_models::PactSpecification::V3 {
          let message = message.as_message().unwrap_or_default();
          let message = block_on(generate_message(&message, &GeneratorTestMode::Consumer, &hashmap!{}, &vec![], &hashmap!{}));
          message.to_json(&spec_version).to_string()
        } else {
          message.to_json().to_string()
        },
        _ => "".to_string()
      }
    } else {
      "".to_string()
    }
  });

  match res {
    Some(res) => {
      let string = CString::new(res).unwrap();
      string.into_raw() as *const c_char
    },
    None => CString::default().into_raw() as *const c_char
  }
}

/// External interface to write out the message pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If a NULL pointer is passed, the current working directory is used.
///
/// If overwrite is true, the file will be overwritten with the contents of the current pact.
/// Otherwise, it will be merged with any existing pact file.
///
/// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
/// not be written, or there is no mock server running on that port or the function panics.
///
/// # Errors
///
/// Errors are returned as positive values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | The pact file was not able to be written |
/// | 2 | The message pact for the given handle was not found |
#[no_mangle]
pub extern fn pactffi_write_message_pact_file(pact: MessagePactHandle, directory: *const c_char, overwrite: bool) -> i32 {
  let result = pact.with_pact(&|_, inner, spec_version| {
    let filename = path_from_dir(directory, Some(inner.default_file_name().as_str()));
    write_pact(inner.boxed(), &filename.unwrap_or_else(|| PathBuf::from(inner.default_file_name().as_str())), spec_version, overwrite)
  });

  match result {
    Some(write_result) => match write_result {
      Ok(_) => 0,
      Err(e) => {
        error!("unable to write the pact file: {:}", e);
        1
      }
    },
    None => {
      error!("unable to write the pact file, message pact for handle {:?} not found", &pact);
      2
    }
  }
}

/// Sets the additional metadata on the Pact file. Common uses are to add the client library details such as the name and version
///
/// * `pact` - Handle to a Pact model
/// * `namespace` - the top level metadat key to set any key values on
/// * `name` - the key to set
/// * `value` - the value to set
#[no_mangle]
pub extern fn pactffi_with_message_pact_metadata(pact: MessagePactHandle, namespace: *const c_char, name: *const c_char, value: *const c_char) {
  pact.with_pact(&|_, inner, _| {
    let namespace = convert_cstr("namespace", namespace).unwrap_or_default();
    let name = convert_cstr("name", name).unwrap_or_default();
    let value = convert_cstr("value", value).unwrap_or_default();

    if !namespace.is_empty() {
      inner.metadata.insert(namespace.to_string(), json!({ name: value }));
    } else {
      warn!("no namespace provided for metadata {:?} => {:?}. Ignoring", name, value);
    }
  });
}

/// Given a c string for the output directory, and an optional filename
/// return a fully qualified directory or file path name for the output pact file
pub(crate) fn path_from_dir(directory: *const c_char, file_name: Option<&str>) -> Option<PathBuf> {
  let dir = unsafe {
    if directory.is_null() {
      warn!("Directory to write to is NULL, defaulting to the current working directory");
      None
    } else {
      let c_str = CStr::from_ptr(directory);
      let dir_str = from_utf8(c_str.to_bytes()).unwrap();
      if dir_str.is_empty() {
        None
      } else {
        Some(dir_str.to_string())
      }
    }
  };

  dir.map(|path| {
    let mut full_path = PathBuf::from(path);
    if let Some(pact_file_name) = file_name {
      full_path.push(pact_file_name);
    }
    full_path
  })
}

ffi_fn! {
  /// External interface to write out the pact file. This function should
  /// be called if all the consumer tests have passed. The directory to write the file to is passed
  /// as the second parameter. If a NULL pointer is passed, the current working directory is used.
  ///
  /// If overwrite is true, the file will be overwritten with the contents of the current pact.
  /// Otherwise, it will be merged with any existing pact file.
  ///
  /// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
  /// not be written or the function panics.
  ///
  /// # Safety
  ///
  /// The directory parameter must either be NULL or point to a valid NULL terminated string.
  ///
  /// # Errors
  ///
  /// Errors are returned as positive values.
  ///
  /// | Error | Description |
  /// |-------|-------------|
  /// | 1 | The function panicked. |
  /// | 2 | The pact file was not able to be written. |
  /// | 3 | The pact for the given handle was not found. |
  fn pactffi_pact_handle_write_file(pact: PactHandle, directory: *const c_char, overwrite: bool) -> i32 {
    let result = pact.with_pact(&|_, inner| {
      let pact_file = inner.pact.default_file_name();
      let filename = path_from_dir(directory, Some(pact_file.as_str()));
      write_pact(inner.pact.boxed(), &filename.unwrap_or_else(|| PathBuf::from(pact_file.as_str())), inner.specification_version, overwrite)
    });

    match result {
      Some(write_result) => match write_result {
        Ok(_) => 0,
        Err(e) => {
          error!("unable to write the pact file: {:}", e);
          2
        }
      },
      None => {
        error!("unable to write the pact file, message pact for handle {:?} not found", &pact);
        3
      }
    }
  } {
    1
  }
}

/// Creates a new V4 asynchronous message and returns a handle to it.
///
/// * `description` - The message description. It needs to be unique for each Message.
///
/// Returns a new `MessageHandle`.
///
/// Note: This function is deprecated in favour of `new_message_interaction` which returns an
/// InteractionHandle that can be used for both HTTP and message interactions.
#[no_mangle]
#[deprecated(note = "Replaced with new_message_interaction")]
pub extern fn pactffi_new_async_message(pact: PactHandle, description: *const c_char) -> MessageHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner| {
      let message = AsynchronousMessage {
        description: description.to_string(),
        ..AsynchronousMessage::default()
      };
      inner.pact.interactions.push(message.boxed_v4());
      MessageHandle::new_v4(pact, inner.pact.interactions.len())
    }).unwrap_or_else(|| MessageHandle::new_v4(pact, 0))
  } else {
    MessageHandle::new_v4(pact, 0)
  }
}

/// Delete a Pact handle and free the resources used by it.
///
/// # Error Handling
///
/// On failure, this function will return a positive integer value.
///
/// * `1` - The handle is not valid or does not refer to a valid Pact. Could be that it was previously deleted.
///
#[no_mangle]
pub extern fn pactffi_free_pact_handle(pact: PactHandle) -> c_uint {
  let mut handles = PACT_HANDLES.lock().unwrap();
  trace!("pactffi_free_pact_handle - removing pact with index {}", pact.pact_ref);
  handles.remove(&pact.pact_ref).map(|_| 0).unwrap_or(1)
}

/// Delete a Pact handle and free the resources used by it.
///
/// # Error Handling
///
/// On failure, this function will return a positive integer value.
///
/// * `1` - The handle is not valid or does not refer to a valid Pact. Could be that it was previously deleted.
///
#[no_mangle]
pub extern fn pactffi_free_message_pact_handle(pact: MessagePactHandle) -> c_uint {
  let mut handles = PACT_HANDLES.lock().unwrap();
  handles.remove(&pact.pact_ref).map(|_| 0).unwrap_or(1)
}

/// Returns the default file name for a Pact handle
pub fn pact_default_file_name(handle: &PactHandle) -> Option<String> {
  handle.with_pact(&|_, inner| {
    inner.pact.default_file_name()
  })
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use either::Either;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::content_types::JSON;
  use pact_models::{generators, matchingrules, HttpStatus};
  use pact_models::matchingrules::{Category, MatchingRule};
  use pact_models::path_exp::DocPath;
  use pact_models::prelude::{Generators, MatchingRules};
  use pretty_assertions::assert_eq;

  use crate::mock_server::handles::*;

  use super::from_integration_json_v2;

  #[test]
  fn pact_handles() {
    let pact_handle = PactHandle::new("TestHandlesC", "TestHandlesP");
    let description = CString::new("first interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let description2 = CString::new("second interaction").unwrap();
    #[allow(deprecated)]
    let i_handle2 = pactffi_new_async_message(pact_handle, description2.as_ptr());

    expect!(i_handle.interaction_ref).to(be_equal_to(((pact_handle.pact_ref as u32) << 16) + 1));
    expect!(i_handle2.interaction_ref).to(be_equal_to(((pact_handle.pact_ref as u32) << 16) + 2));

    pact_handle.with_pact(&|pact_ref, inner| {
      expect!(pact_ref).to(be_equal_to(pact_handle.pact_ref - 1));
      expect!(inner.pact.consumer.name.as_str()).to(be_equal_to("TestHandlesC"));
      expect!(inner.pact.provider.name.as_str()).to(be_equal_to("TestHandlesP"));
      expect!(inner.pact.interactions.len()).to(be_equal_to(2));
    });

    i_handle.with_interaction(&|i_ref, _, inner| {
      expect!(i_ref).to(be_equal_to(0));
      expect!(inner.description().as_str()).to(be_equal_to("first interaction"));
      expect!(inner.type_of().as_str()).to(be_equal_to("V4 Synchronous/HTTP"));
    });

    i_handle2.with_message(&|i_ref, inner, _| {
      expect!(i_ref).to(be_equal_to(1));
      expect!(inner.description().as_str()).to(be_equal_to("second interaction"));
      expect!(inner.type_of().as_str()).to(be_equal_to("V4 Asynchronous/Messages"));
    });

    pactffi_free_pact_handle(pact_handle);
  }

  #[test]
  fn simple_query_parameter() {
    let pact_handle = PactHandle::new("TestC1", "TestP");
    let description = CString::new("simple_query_parameter").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.query.clone()).to(be_some().value(hashmap!{
      "id".to_string() => vec!["100".to_string()]
    }));
    expect!(interaction.request.matching_rules.rules.get(&Category::QUERY).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn query_parameter_with_matcher() {
    let pact_handle = PactHandle::new("TestC2", "TestP");
    let description = CString::new("query_parameter_with_matcher").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("id").unwrap();
    let value = CString::new("{\"value\": \"100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\d+\"}").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.query.clone()).to(be_some().value(hashmap!{
      "id".to_string() => vec!["100".to_string()]
    }));
    expect!(&interaction.request.matching_rules).to(be_equal_to(&matchingrules! {
      "query" => { "$.id" => [ MatchingRule::Regex("\\d+".to_string()) ] }
    }));
  }

  #[test]
  fn query_parameter_with_multiple_values() {
    let pact_handle = PactHandle::new("TestC3", "TestP");
    let description = CString::new("query_parameter_with_multiple_values").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("id").unwrap();
    let value = CString::new("{\"value\": [\"1\", \"2\"]}").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.query.clone()).to(be_some().value(hashmap!{
      "id".to_string() => vec!["1".to_string(), "2".to_string()]
    }));
    expect!(interaction.request.matching_rules.rules.get(&Category::QUERY).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn query_parameter_with_multiple_values_with_matchers() {
    let pact_handle = PactHandle::new("TestC4", "TestP");
    let description = CString::new("query_parameter_with_multiple_values_with_matchers").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("id").unwrap();
    let value = CString::new("{\"value\": \"100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\d+\"}").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 0, value.as_ptr());
    let value = CString::new("{\"value\": \"abc\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\w+\"}").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 1, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.query.clone()).to(be_some().value(hashmap!{
      "id".to_string() => vec!["100".to_string(), "abc".to_string()]
    }));
    assert_eq!(&interaction.request.matching_rules, &matchingrules! {
      "query" => {
        "$.id[1]" => [ MatchingRule::Regex("\\w+".to_string()) ],
        "$.id[0]" => [ MatchingRule::Regex("\\d+".to_string()) ]
      }
    });
  }

  // Issue #205
  #[test]
  fn query_parameter_with_multiple_values_in_json() {
    let pact_handle = PactHandle::new("TestC5", "TestP");
    let description = CString::new("query_parameter_with_multiple_values").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("catId[]").unwrap();
    let value = CString::new("{\"value\": [\"1\"], \"pact:matcher:type\": \"type\", \"min\": 1}").unwrap();
    pactffi_with_query_parameter_v2(handle, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.query.clone()).to(be_some().value(hashmap!{
      "catId[]".to_string() => vec!["1".to_string()]
    }));
    expect!(&interaction.request.matching_rules).to(be_equal_to(&matchingrules! {
      "query" => { "$['catId[]']" => [ MatchingRule::MinType(1) ] }
    }));
  }

  #[test]
  fn from_integration_json_test() {
    let mut rules = MatchingRules::default();
    let mut generators = Generators::default();
    let path = DocPath::root();

    expect!(from_integration_json_v2(&mut rules, &mut generators, "100", path.clone(), "query", 0))
      .to(be_equal_to(Either::Left("100".to_string())));
    expect!(from_integration_json_v2(&mut rules, &mut generators, "kjhaksdhj", path.clone(), "query", 0))
      .to(be_equal_to(Either::Left("kjhaksdhj".to_string())));
    expect!(from_integration_json_v2(&mut rules, &mut generators, r#"{"value":"100"}"#, path.clone(), "query", 0))
      .to(be_equal_to(Either::Left("100".to_string())));
    expect!(from_integration_json_v2(&mut rules, &mut generators, r#"{"value":["100"]}"#, path.clone(), "query", 0))
      .to(be_equal_to(Either::Right(vec!["100".to_string()])));
    expect!(from_integration_json_v2(&mut rules, &mut generators, r#"{"value":["100","200"]}"#, path.clone(), "query", 0))
      .to(be_equal_to(Either::Right(vec!["100".to_string(), "200".to_string()])));
  }

  #[test]
  fn pactffi_with_header_v2_simple_header() {
    let pact_handle = PactHandle::new("TestHC1", "TestHP");
    let description = CString::new("simple_header").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["100".to_string()]
    }));
    expect!(interaction.request.matching_rules.rules.get(&Category::HEADER).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn pactffi_set_header_simple_header() {
    let pact_handle = PactHandle::new("TestHC1", "TestHP");
    let description = CString::new("simple_header").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_set_header(handle, InteractionPart::Request, name.as_ptr(), value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["100".to_string()]
    }));
  }

  #[test]
  fn pactffi_with_header_v2_different_case() {
    let pact_handle = PactHandle::new("TestHC1", "TestHP");
    let description = CString::new("simple_header").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let name = CString::new("X-Id").unwrap();
    let value = CString::new("200").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["200".to_string()]
    }));
    expect!(interaction.request.matching_rules.rules.get(&Category::HEADER).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn pactffi_set_header_different_case() {
    let pact_handle = PactHandle::new("TestHC1", "TestHP");
    let description = CString::new("simple_header").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_set_header(handle, InteractionPart::Request, name.as_ptr(), value.as_ptr());

    let name = CString::new("X-Id").unwrap();
    let value = CString::new("300").unwrap();
    pactffi_set_header(handle, InteractionPart::Request, name.as_ptr(), value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["300".to_string()]
    }));
  }

  #[test]
  fn header_with_matcher() {
    let pact_handle = PactHandle::new("TestHC2", "TestHP");
    let description = CString::new("header_with_matcher").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("{\"value\": \"100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\d+\"}").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["100".to_string()]
    }));
    expect!(&interaction.request.matching_rules).to(be_equal_to(&matchingrules! {
      "header" => { "$['x-id']" => [ MatchingRule::Regex("\\d+".to_string()) ] }
    }));
  }

  #[test]
  fn header_with_multiple_values() {
    let pact_handle = PactHandle::new("TestHC3", "TestHP");
    let description = CString::new("header_with_multiple_values").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("{\"value\": [\"1\", \"2\"]}").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["1".to_string(), "2".to_string()]
    }));
    expect!(interaction.request.matching_rules.rules.get(&Category::HEADER).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn header_with_multiple_values_with_matchers() {
    let pact_handle = PactHandle::new("TestHC4", "TestHP");
    let description = CString::new("header_with_multiple_values_with_matchers").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("{\"value\": \"100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\d+\"}").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());
    let value = CString::new("{\"value\": \"abc\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\w+\"}").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 1, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["100".to_string(), "abc".to_string()]
    }));
    assert_eq!(&interaction.request.matching_rules, &matchingrules! {
      "header" => {
        "$['x-id'][1]" => [ MatchingRule::Regex("\\w+".to_string()) ],
        "$['x-id'][0]" => [ MatchingRule::Regex("\\d+".to_string()) ]
      }
    });
  }

  // Issue #300
  #[test_log::test]
  fn header_with_multiple_values_as_a_string() {
    let pact_handle = PactHandle::new("TestHC3", "TestHP");
    let description = CString::new("header_with_multiple_values_as_a_string").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("accept").unwrap();
    let value = CString::new("application/problem+json, application/json, text/plain, */*").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "accept".to_string() => vec![
        "application/problem+json".to_string(),
        "application/json".to_string(),
        "text/plain".to_string(),
        "*/*".to_string()
      ]
    }));
  }

  #[test]
  fn pactffi_with_header_v2_incorrect_order() {
    let pact_handle = PactHandle::new("TestHC1", "TestHP");
    let description = CString::new("simple_header").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("200").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 1, value.as_ptr());

    let name = CString::new("X-Id").unwrap();
    let value = CString::new("300").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 2, value.as_ptr());

    let name = CString::new("x-id").unwrap();
    let value = CString::new("100").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "x-id".to_string() => vec!["100".to_string(), "200".to_string(), "300".to_string()]
    }));
  }

  // Issue #355
  #[test_log::test]
  fn header_with_provider_state_generator() {
    let pact_handle = PactHandle::new("TestHC5", "TestHP");
    let description = CString::new("header_with_provider_state_generator").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let name = CString::new("se-token").unwrap();
    let value = CString::new("{\"expression\":\"${seToken}\",\"pact:generator:type\":\"ProviderState\",\"pact:matcher:type\":\"type\",\"value\":\"ABC123\"}").unwrap();
    pactffi_with_header_v2(handle, InteractionPart::Request, name.as_ptr(), 0, value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.headers.clone()).to(be_some().value(hashmap!{
      "se-token".to_string() => vec!["ABC123".to_string()]
    }));
    expect!(&interaction.request.matching_rules).to(be_equal_to(&matchingrules! {
      "header" => {
        "$['se-token']" => [ MatchingRule::Type ]
      }
    }));
    expect!(&interaction.request.generators).to(be_equal_to(&generators! {
      "header" => {
        "$['se-token']" => Generator::ProviderStateGenerator("${seToken}".to_string(), None)
      }
    }));
    let json = interaction.to_json();
    assert_eq!(json, json!({
      "description": "header_with_provider_state_generator",
      "pending": false,
      "request": {
        "generators": {
          "header": {
            "se-token": {"expression": "${seToken}", "type": "ProviderState"}
          }
        },
        "headers": {
          "se-token": ["ABC123"]
        },
        "matchingRules": {
          "header": {
            "se-token": {
              "combine": "AND",
              "matchers": [
                {"match": "type"}
              ]
            }
          }
        },
        "method": "GET",
        "path": "/"
      },
      "response": {"status": 200}, "type": "Synchronous/HTTP"
    }));
  }

  #[test]
  fn simple_path() {
    let pact_handle = PactHandle::new("TestPC1", "TestPP");
    let description = CString::new("simple_path").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let method = CString::new("PUT").unwrap();
    let path = CString::new("/path/to/100").unwrap();
    pactffi_with_request(handle, method.as_ptr(), path.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.method).to(be_equal_to("PUT"));
    expect!(interaction.request.path).to(be_equal_to("/path/to/100"));
    expect!(interaction.request.matching_rules.rules.get(&Category::PATH).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn path_with_matcher() {
    let pact_handle = PactHandle::new("TestPC2", "TestPP");
    let description = CString::new("path_with_matcher").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let method = CString::new("PUT").unwrap();
    let path = CString::new("{\"value\": \"/path/to/100\", \"pact:matcher:type\": \"regex\", \"regex\": \"\\\\/path\\\\/to\\\\/\\\\d+\"}").unwrap();
    pactffi_with_request(handle, method.as_ptr(), path.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.request.method).to(be_equal_to("PUT"));
    expect!(interaction.request.path).to(be_equal_to("/path/to/100"));
    expect!(&interaction.request.matching_rules).to(be_equal_to(&matchingrules! {
      "path" => { "$" => [ MatchingRule::Regex("\\/path\\/to\\/\\d+".to_string()) ] }
    }));
  }

  #[test]
  fn pactffi_with_body_test() {
    let pact_handle = PactHandle::new("WithBodyC", "WithBodyP");
    let description = CString::new("first interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let json_ct = CString::new(JSON.to_string()).unwrap();
    let json = "{\"test\":true}";
    let body = CString::new(json).unwrap();
    let result = pactffi_with_body(i_handle, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let description2 = CString::new("second interaction").unwrap();
    let i_handle2 = pactffi_new_message_interaction(pact_handle, description2.as_ptr());
    let result2 = pactffi_with_body(i_handle2, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let description3 = CString::new("third interaction").unwrap();
    let i_handle3 = pactffi_new_sync_message_interaction(pact_handle, description3.as_ptr());
    let result3 = pactffi_with_body(i_handle3, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let interaction1 = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction2 = i_handle2.with_interaction(&|_, _, inner| {
      inner.as_v4_async_message().unwrap()
    }).unwrap();
    let interaction3 = i_handle3.with_interaction(&|_, _, inner| {
      inner.as_v4_sync_message().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(result).to(be_true());
    expect!(result2).to(be_true());
    expect!(result3).to(be_true());

    let body1 = interaction1.request.body.value().unwrap();
    expect!(body1.len()).to(be_equal_to(json.len()));
    let headers = interaction1.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to(&JSON.to_string()));

    let body2 = interaction2.contents.contents.value().unwrap();
    expect!(body2.len()).to(be_equal_to(json.len()));
    expect!(interaction2.contents.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/json\""));

    let body3 = interaction3.request.contents.value().unwrap();
    expect!(body3.len()).to(be_equal_to(json.len()));
    expect!(interaction3.request.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/json\""));
  }

  #[test]
  fn pactffi_with_body_for_non_default_json_test() {
    let pact_handle = PactHandle::new("WithBodyC", "WithBodyP");
    let description = CString::new("first interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let json_ct = CString::new("application/vnd.schemaregistry.v1+json").unwrap();
    let json = "{\"test\":true}";
    let body = CString::new(json).unwrap();
    let result = pactffi_with_body(i_handle, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let description2 = CString::new("second interaction").unwrap();
    let i_handle2 = pactffi_new_message_interaction(pact_handle, description2.as_ptr());
    let result2 = pactffi_with_body(i_handle2, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let description3 = CString::new("third interaction").unwrap();
    let i_handle3 = pactffi_new_sync_message_interaction(pact_handle, description3.as_ptr());
    let result3 = pactffi_with_body(i_handle3, InteractionPart::Request, json_ct.as_ptr(), body.as_ptr());

    let interaction1 = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction2 = i_handle2.with_interaction(&|_, _, inner| {
      inner.as_v4_async_message().unwrap()
    }).unwrap();
    let interaction3 = i_handle3.with_interaction(&|_, _, inner| {
      inner.as_v4_sync_message().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(result).to(be_true());
    expect!(result2).to(be_true());
    expect!(result3).to(be_true());

    let body1 = interaction1.request.body.value().unwrap();
    expect!(body1.len()).to(be_equal_to(json.len()));
    let headers = interaction1.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to("application/vnd.schemaregistry.v1+json"));

    let body2 = interaction2.contents.contents.value().unwrap();
    expect!(body2.len()).to(be_equal_to(json.len()));
    expect!(interaction2.contents.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/vnd.schemaregistry.v1+json\""));

    let body3 = interaction3.request.contents.value().unwrap();
    expect!(body3.len()).to(be_equal_to(json.len()));
    expect!(interaction3.request.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/vnd.schemaregistry.v1+json\""));
  }

  #[test]
  fn pactffi_with_binary_file_test() {
    let pact_handle = PactHandle::new("CBin", "PBin");
    let description = CString::new("first interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let json_ct = CString::new(JSON.to_string()).unwrap();
    let json = "{\"test\":true}";
    let result = pactffi_with_binary_file(i_handle, InteractionPart::Request,
      json_ct.as_ptr(), json.as_ptr(), json.len());

    let description2 = CString::new("second interaction").unwrap();
    let i_handle2 = pactffi_new_message_interaction(pact_handle, description2.as_ptr());
    let result2 = pactffi_with_binary_file(i_handle2, InteractionPart::Request,
      json_ct.as_ptr(), json.as_ptr(), json.len());

    let description3 = CString::new("third interaction").unwrap();
    let i_handle3 = pactffi_new_sync_message_interaction(pact_handle, description3.as_ptr());
    let result3 = pactffi_with_binary_file(i_handle3, InteractionPart::Request,
      json_ct.as_ptr(), json.as_ptr(), json.len());

    let interaction1 = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction2 = i_handle2.with_interaction(&|_, _, inner| {
      inner.as_v4_async_message().unwrap()
    }).unwrap();
    let interaction3 = i_handle3.with_interaction(&|_, _, inner| {
      inner.as_v4_sync_message().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(result).to(be_true());
    expect!(result2).to(be_true());
    expect!(result3).to(be_true());

    let body1 = interaction1.request.body.value().unwrap();
    expect!(body1.len()).to(be_equal_to(json.len()));
    let headers = interaction1.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to(&JSON.to_string()));

    let body2 = interaction2.contents.contents.value().unwrap();
    expect!(body2.len()).to(be_equal_to(json.len()));
    expect!(interaction2.contents.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/json\""));

    let body3 = interaction3.request.contents.value().unwrap();
    expect!(body3.len()).to(be_equal_to(json.len()));
    expect!(interaction3.request.metadata.get("contentType").unwrap().to_string()).to(be_equal_to("\"application/json\""));
  }

  const GIF_1PX: [u8; 35] = [
    0o107, 0o111, 0o106, 0o070, 0o067, 0o141, 0o001, 0o000, 0o001, 0o000, 0o200, 0o000, 0o000, 0o377, 0o377, 0o377,
    0o377, 0o377, 0o377, 0o054, 0o000, 0o000, 0o000, 0o000, 0o001, 0o000, 0o001, 0o000, 0o000, 0o002, 0o002, 0o104,
    0o001, 0o000, 0o073
  ];

  #[test]
  fn pactffi_with_binary_body_test() {
    let pact_handle = PactHandle::new("WithBodyC", "WithBodyP");
    let description = CString::new("binary interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let gif_ct = CString::new("image/gif".to_string()).unwrap();
    let value = GIF_1PX.as_ptr();
    let result = pactffi_with_binary_body(i_handle, InteractionPart::Request, gif_ct.as_ptr(), value, GIF_1PX.len());

    let description2 = CString::new("second binary interaction").unwrap();
    let i_handle2 = pactffi_new_interaction(pact_handle, description2.as_ptr());
    let result2 = pactffi_with_binary_body(i_handle2, InteractionPart::Request, std::ptr::null(), value, GIF_1PX.len());

    let description3 = CString::new("third binary interaction").unwrap();
    let i_handle3 = pactffi_new_interaction(pact_handle, description3.as_ptr());
    let result3 = pactffi_with_binary_body(i_handle3, InteractionPart::Request, std::ptr::null(), std::ptr::null(), GIF_1PX.len());

    let description4 = CString::new("message binary interaction").unwrap();
    let i_handle4 = pactffi_new_message_interaction(pact_handle, description4.as_ptr());
    let result4 = pactffi_with_binary_body(i_handle4, InteractionPart::Request, gif_ct.as_ptr(), value, GIF_1PX.len());

    let description5 = CString::new("sync message interaction").unwrap();
    let i_handle5 = pactffi_new_sync_message_interaction(pact_handle, description5.as_ptr());
    let result5 = pactffi_with_binary_body(i_handle5, InteractionPart::Request, gif_ct.as_ptr(), value, GIF_1PX.len());

    let interaction1 = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction2 = i_handle2.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction3 = i_handle3.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();
    let interaction4 = i_handle4.with_interaction(&|_, _, inner| {
      inner.as_v4_async_message().unwrap()
    }).unwrap();
    let interaction5 = i_handle5.with_interaction(&|_, _, inner| {
      inner.as_v4_sync_message().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(result).to(be_true());
    expect!(result2).to(be_true());
    expect!(result3).to(be_true());
    expect!(result4).to(be_true());
    expect!(result5).to(be_true());

    let body1 = interaction1.request.body.value().unwrap();
    expect!(body1.len()).to(be_equal_to(GIF_1PX.len()));
    let gif_ct = ContentType::parse("image/gif").unwrap();
    expect!(interaction1.request.body.content_type().unwrap()).to(be_equal_to(gif_ct.clone()));
    let headers = interaction1.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to(&gif_ct.to_string()));

    let body2 = interaction2.request.body.value().unwrap();
    expect!(body2.len()).to(be_equal_to(GIF_1PX.len()));
    let bin_ct = ContentType::parse("application/octet-stream").unwrap();
    expect!(interaction2.request.body.content_type().unwrap()).to(be_equal_to(bin_ct.clone()));
    let headers = interaction2.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to(&bin_ct.to_string()));

    expect!(interaction3.request.body).to(be_equal_to(OptionalBody::Null));

    let body4 = interaction4.contents.contents.value().unwrap();
    expect!(body4.len()).to(be_equal_to(GIF_1PX.len()));
    expect!(interaction4.contents.contents.content_type().unwrap()).to(be_equal_to(gif_ct.clone()));
    let headers = &interaction4.contents.metadata;
    expect!(headers.get("contentType").unwrap()).to(be_equal_to(&json!(gif_ct.to_string())));

    let body5 = interaction5.request.contents.value().unwrap();
    expect!(body5.len()).to(be_equal_to(GIF_1PX.len()));
    expect!(interaction5.request.contents.content_type().unwrap()).to(be_equal_to(gif_ct.clone()));
    let headers = &interaction5.request.metadata;
    expect!(headers.get("contentType").unwrap()).to(be_equal_to(&json!(gif_ct.to_string())));
  }

  #[test]
  fn process_json_with_nested_rules() {
    let mut rules = MatchingRules::default();
    let mut category = rules.add_category("body");
    let mut generators = Generators::default();
    let json = json!({
      "pact:matcher:type": "values",
      "value": {
        "some-string": {
          "pact:matcher:type": "values",
          "value": {
            "some-string": {
              "pact:matcher:type": "values",
              "value": {
                "some-string": {
                  "pact:matcher:type": "type",
                  "value": "some string"
                }
              }
            }
          }
        }
      }
    });

    let result = process_json(json.to_string(), &mut category, &mut generators);
    expect!(result).to(be_equal_to("{\"some-string\":{\"some-string\":{\"some-string\":\"some string\"}}}"));
    expect!(&rules).to(be_equal_to(&matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Values ],
        "$.*" => [ MatchingRule::Values ],
        "$.*.*" => [ MatchingRule::Values ],
        "$.*.*.*" => [ MatchingRule::Type ]
      }
    }));
  }

  #[test]
  fn pactffi_given_with_param_test() {
    let pact_handle = PactHandle::new("pactffi_given_with_param", "pactffi_given_with_param");
    let description = CString::new("pactffi_given_with_param").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let state_one = CString::new("state one").unwrap();
    let state_two = CString::new("state two").unwrap();
    let param_one = CString::new("one").unwrap();
    let param_two = CString::new("two").unwrap();
    let param_value = CString::new("100").unwrap();
    pactffi_given_with_param(handle, state_one.as_ptr(), param_one.as_ptr(), param_value.as_ptr());
    pactffi_given_with_param(handle, state_one.as_ptr(), param_two.as_ptr(), param_value.as_ptr());
    pactffi_given_with_param(handle, state_one.as_ptr(), param_one.as_ptr(), param_value.as_ptr());
    pactffi_given_with_param(handle, state_two.as_ptr(), param_one.as_ptr(), param_value.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_request_response().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.provider_states.len()).to(be_equal_to(2));
    let state_1 = interaction.provider_states.iter()
      .find(|state| state.name == "state one").unwrap();
    let state_2 = interaction.provider_states.iter()
      .find(|state| state.name == "state two").unwrap();
    let keys: Vec<&String> = state_1.params.keys().sorted().collect();
    expect!(keys).to(be_equal_to(vec!["one", "two"]));
    let keys: Vec<&String> = state_2.params.keys().collect();
    expect!(keys).to(be_equal_to(vec!["one"]));
  }

  #[test]
  fn pactffi_given_with_params_test() {
    let pact_handle = PactHandle::new("pactffi_given_with_params", "pactffi_given_with_params");
    let description = CString::new("pactffi_given_with_params").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let state_one = CString::new("state one").unwrap();
    let state_two = CString::new("state two").unwrap();
    let params_one = CString::new("{\"one\": 100}").unwrap();
    let params_two = CString::new("{\"two\": 200}").unwrap();
    pactffi_given_with_params(handle, state_one.as_ptr(), params_one.as_ptr());
    pactffi_given_with_params(handle, state_one.as_ptr(), params_two.as_ptr());
    pactffi_given_with_params(handle, state_two.as_ptr(), params_one.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_request_response().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.provider_states.len()).to(be_equal_to(3));
    let state_1 = interaction.provider_states.get(0).unwrap();
    let state_2 = interaction.provider_states.get(1).unwrap();
    let state_3 = interaction.provider_states.get(2).unwrap();
    let keys: Vec<&String> = state_1.params.keys().collect();
    expect!(keys).to(be_equal_to(vec!["one"]));
    let keys: Vec<&String> = state_2.params.keys().collect();
    expect!(keys).to(be_equal_to(vec!["two"]));
    let keys: Vec<&String> = state_3.params.keys().collect();
    expect!(keys).to(be_equal_to(vec!["one"]));
  }

  #[test]
  fn pactffi_with_xml_body_test() {
    let pact_handle = PactHandle::new("XMLC", "XMLP");
    let description = CString::new("XML interaction").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let ct = CString::new(XML.to_string()).unwrap();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <projects>
    <item>
    <id>1</id>
    <tasks>
        <item>
            <id>1</id>
            <name>Do the laundry</name>
            <done>true</done>
        </item>
        <item>
            <id>2</id>
            <name>Do the dishes</name>
            <done>false</done>
        </item>
        <item>
            <id>3</id>
            <name>Do the backyard</name>
            <done>false</done>
        </item>
        <item>
            <id>4</id>
            <name>Do nothing</name>
            <done>false</done>
        </item>
    </tasks>
    </item>
    </projects>"#;
    let body = CString::new(xml).unwrap();
    let result = pactffi_with_body(i_handle, InteractionPart::Request, ct.as_ptr(), body.as_ptr());

    let interaction = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(result).to(be_true());

    let body = interaction.request.body.value().unwrap();
    expect!(body.len()).to(be_equal_to(xml.len()));
    let headers = interaction.request.headers.unwrap();
    expect!(headers.get("Content-Type").unwrap().first().unwrap()).to(be_equal_to(&XML.to_string()));
  }

  #[test]
  fn simple_status() {
    let pact_handle = PactHandle::new("TestPC1", "TestPP");
    let description = CString::new("simple_status").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let status = CString::new("404").unwrap();
    pactffi_response_status_v2(handle, status.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.response.status).to(be_equal_to(404));
    expect!(interaction.response.matching_rules.rules.get(&Category::PATH).cloned().unwrap_or_default().is_empty()).to(be_true());
  }

  #[test]
  fn status_with_matcher() {
    let pact_handle = PactHandle::new("TestPC2", "TestPP");
    let description = CString::new("status_with_matcher").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let status = CString::new("{\"value\": 503, \"pact:matcher:type\":\"statusCode\", \"status\": \"error\"}").unwrap();
    pactffi_response_status_v2(handle, status.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.response.status).to(be_equal_to(503));
    expect!(&interaction.response.matching_rules).to(be_equal_to(&matchingrules! {
      "status" => { "$" => [ MatchingRule::StatusCode(HttpStatus::Error) ] }
    }));
  }

  #[test]
  fn status_with_matcher_and_generator() {
    let pact_handle = PactHandle::new("TestPC3", "TestPP");
    let description = CString::new("status_with_matcher_and_generator").unwrap();
    let handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let status = CString::new("{\"pact:generator:type\": \"RandomInt\", \"min\": 201, \"max\": 299, \"pact:matcher:type\": \"integer\"}").unwrap();
    pactffi_response_status_v2(handle, status.as_ptr());

    let interaction = handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);

    expect!(interaction.response.status).to(be_equal_to(0));
    expect!(&interaction.response.matching_rules).to(be_equal_to(&matchingrules! {
      "status" => { "$" => [ MatchingRule::Integer ] }
    }));
    expect!(&interaction.response.generators).to(be_equal_to(&generators! {
      "status" => { "$" => Generator::RandomInt(201, 299) }
    }));
  }

  #[test]
  fn pactffi_with_matching_rules_v2_test() {
    let pact_handle = PactHandle::new("Consumer", "Provider");
    let description = CString::new("Matching Rule Test").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let matching_rule = CString::new(r#"{
        "$.body.one": {
          "match": "regex",
          "regex": "\\w{3}\\d{3}"
        }
      }"#).unwrap();
    let result = pactffi_with_matching_rules(
      i_handle,
      InteractionPart::Request,
      matching_rule.as_ptr(),
    );
    expect!(result).to(be_true());

    let interaction = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);
    assert_eq!(interaction.request.matching_rules, matchingrules! {
      "body" => {
        "$.one" => [ MatchingRule::Regex("\\w{3}\\d{3}".to_string()) ]
      }
    });
  }

  #[test]
  fn pactffi_with_matching_rules_v4_test() {
    let pact_handle = PactHandle::new("Consumer", "Provider");
    let description = CString::new("Matching Rule Test").unwrap();
    let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

    let matching_rule = CString::new(r#"{
      "body": {
        "$.*": {
          "combine": "AND",
          "matchers": [
            {
              "match": "semver"
            }
          ]
        }
      }
    }"#).unwrap();
    let result = pactffi_with_matching_rules(
      i_handle,
      InteractionPart::Response,
      matching_rule.as_ptr(),
    );
    expect!(result).to(be_true());

    let interaction = i_handle.with_interaction(&|_, _, inner| {
      inner.as_v4_http().unwrap()
    }).unwrap();

    pactffi_free_pact_handle(pact_handle);
    assert_eq!(interaction.response.matching_rules, matchingrules! {
      "body" => {
        "$.*" => [ MatchingRule::Semver ]
      }
    });
  }
}
