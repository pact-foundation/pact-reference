//! The Pact `Message` type, including associated matching rules and provider states.

/*===============================================================================================
 * # Imports
 *---------------------------------------------------------------------------------------------*/

use std::collections::HashMap;
use std::ops::Drop;
use std::sync::Mutex;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use either::Either;
use itertools::Itertools;
use libc::{c_char, c_int, c_uchar, c_uint, EXIT_FAILURE, EXIT_SUCCESS, size_t};
use serde_json::from_str as from_json_str;
use serde_json::Value as JsonValue;
use tracing::trace;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;

use crate::{as_mut, as_ref, cstr, ffi_fn, safe_str};
use crate::models::pact_specification::PactSpecification;
use crate::util::*;
use crate::util::string::optional_str;

use pact_matching::generators::generate_message;
use pact_models::generators::GeneratorTestMode;
use maplit::hashmap;
use futures::executor::block_on;

/*===============================================================================================
 * # Re-Exports
 *---------------------------------------------------------------------------------------------*/

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_models::message::Message;
pub use pact_models::provider_states::ProviderState;
use pact_models::v4::message_parts::MessageContents;

/*===============================================================================================
 * # Message
 *---------------------------------------------------------------------------------------------*/

/*-----------------------------------------------------------------------------------------------
 * ## Constructors / Destructor
 */

ffi_fn! {
    /// Get a mutable pointer to a newly-created default message on the heap.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// Returns NULL on error.
    fn pactffi_message_new() -> *mut Message {
        let message = Message::default();
        ptr::raw_to(message)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Constructs a `Message` from the JSON string
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the JSON string is invalid or not UTF-8 encoded, returns a NULL.
    fn pactffi_message_new_from_json(
        index: c_uint,
        json_str: *const c_char,
        spec_version: PactSpecification
    ) -> *mut Message {
        let message = {
            let index = index as usize;
            let json_value: JsonValue = from_json_str(safe_str!(json_str))
                .context("error parsing json_str as JSON")?;
            let spec_version = spec_version.into();

            Message::from_json(index, &json_value, &spec_version)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        };

        ptr::raw_to(message)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Constructs a `Message` from a body with a given content-type.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the body or content type are invalid or not UTF-8 encoded, returns NULL.
    fn pactffi_message_new_from_body(body: *const c_char, content_type: *const c_char) -> *mut Message {
        // Get the body as a Vec<u8>.
        let body = cstr!(body)
            .to_bytes()
            .to_owned();

        // Parse the content type.
        let content_type = ContentType::parse(safe_str!(content_type))
            .map_err(|s| anyhow!("invalid content type '{}'", s))?;

        // Populate the Message metadata.
        let mut metadata = HashMap::new();
        metadata.insert(String::from("contentType"), JsonValue::String(content_type.to_string()));

        // Populate the OptionalBody with our content and content type.
        let contents = OptionalBody::Present(body.into(), Some(content_type), None);

        // Construct and return the message.
        let message = Message {
            contents,
            metadata,
            .. Message::default()
        };

        ptr::raw_to(message)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Destroy the `Message` being pointed to.
    fn pactffi_message_delete(message: *mut Message) {
        ptr::drop_raw(message);
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Contents
 */

ffi_fn! {
    /// Get the contents of a `Message` in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete` and can outlive the message.
    /// This function must only ever be called from a foreign language. Calling it from a Rust function
    /// that has a Tokio runtime in its call stack can result in a deadlock.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_message_get_contents(message: *const Message) -> *const c_char {
        let message = as_ref!(message);
        let message = block_on(generate_message(&message, &GeneratorTestMode::Consumer, &hashmap!{}, &vec![], &hashmap!{}));

        match message.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(message.contents.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the contents of the message.
  ///
  /// # Safety
  ///
  /// The message contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the message contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_message_set_contents(message: *mut Message, contents: *const c_char, content_type: *const c_char) {
    let message = as_mut!(message);

    if contents.is_null() {
      message.contents = OptionalBody::Null;
    } else {
      let contents = safe_str!(contents);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      message.contents = OptionalBody::Present(Bytes::from(contents), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the contents of a `Message`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0. If the body of the message
    /// is missing, then this function also returns 0.
    fn pactffi_message_get_contents_length(message: *const Message) -> size_t {
        let message = as_ref!(message);

        match &message.contents {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the contents of a `Message` as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_message_get_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_message_get_contents_bin(message: *const Message) -> *const c_uchar {
        let message = as_ref!(message);

        match &message.contents {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the contents of the message as an array of bytes.
  ///
  /// # Safety
  ///
  /// The contents pointer must be valid for reads of `len` bytes, and it must be properly aligned
  /// and consecutive. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the message contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_message_set_contents_bin(message: *mut Message, contents: *const c_uchar, len: size_t, content_type: *const c_char) {
    let message = as_mut!(message);

    if contents.is_null() {
      message.contents = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents, len) };
      let contents = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      message.contents = OptionalBody::Present(contents, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

/*-----------------------------------------------------------------------------------------------
 * ## Description
 */

ffi_fn! {
    /// Get a copy of the description.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `Message`.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the Rust string contains embedded
    /// null ('\0') bytes.
    fn pactffi_message_get_description(message: *const Message) -> *const c_char {
        let message = as_ref!(message);
        let description = string::to_c(&message.description)?;
        description as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Write the `description` field on the `Message`.
    ///
    /// # Safety
    ///
    /// `description` must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    /// This function will only reallocate if the new string
    /// does not fit in the existing buffer.
    ///
    /// # Error Handling
    ///
    /// Errors will be reported with a non-zero return value.
    fn pactffi_message_set_description(message: *mut Message, description: *const c_char) -> c_int {
        let message = as_mut!(message);
        let description = safe_str!(description);

        // Wipe out the previous contents of the string, without
        // deallocating, and set the new description.
        message.description.clear();
        message.description.push_str(description);

        EXIT_SUCCESS
    } {
        EXIT_FAILURE
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Provider States
 */

ffi_fn! {
    /// Get a copy of the provider state at the given index from this message.
    ///
    /// # Safety
    ///
    /// The returned structure must be deleted with `provider_state_delete`.
    ///
    /// Since it is a copy, the returned structure may safely outlive
    /// the `Message`.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a variant other than Success.
    ///
    /// This function may fail if the index requested is out of bounds,
    /// or if any of the Rust strings contain embedded null ('\0') bytes.
    fn pactffi_message_get_provider_state(message: *const Message, index: c_uint) -> *const ProviderState {
        let message = as_ref!(message);
        let index = index as usize;

        // Get a raw pointer directly, rather than boxing it, as its owned by the `Message`
        // and will be cleaned up when the `Message` is cleaned up.
        let provider_state = message
            .provider_states
            .get(index)
            .ok_or(anyhow!("index is out of bounds"))?;

        provider_state as *const ProviderState
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get an iterator over provider states.
    ///
    /// # Safety
    ///
    /// The underlying data must not change during iteration.
    ///
    /// # Error Handling
    ///
    /// Returns NULL if an error occurs.
    fn pactffi_message_get_provider_state_iter(message: *mut Message) -> *mut ProviderStateIterator {
        let message = as_mut!(message);
        let iter = ProviderStateIterator::new(message);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get the next value from the iterator.
    ///
    /// # Safety
    ///
    /// The underlying data must not change during iteration.
    ///
    /// If a previous call panicked, then the internal mutex will have been poisoned and this
    /// function will return NULL.
    ///
    /// # Error Handling
    ///
    /// Returns NULL if an error occurs.
    fn pactffi_provider_state_iter_next(iter: *mut ProviderStateIterator) -> *mut ProviderState {
        let iter = as_mut!(iter);
        let index = iter.next();
        let guard = iter.message.lock().unwrap();
        let message_ptr = unsafe { guard.as_mut() };
        match message_ptr {
            Some(message) => {
                match message.provider_states_mut().get_mut(index) {
                  Some(provider_state) => provider_state as *mut ProviderState,
                  None => {
                    trace!("iter past the end of provider states");
                    std::ptr::null_mut()
                  }
                }
            }
            None => std::ptr::null_mut()
        }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Delete the iterator.
    fn pactffi_provider_state_iter_delete(iter: *mut ProviderStateIterator) {
        ptr::drop_raw(iter);
    }
}

/// Iterator over individual provider states.
#[allow(missing_copy_implementations)]
#[allow(missing_debug_implementations)]
pub struct ProviderStateIterator {
    current: usize,
    message: Mutex<*mut dyn Interaction>
}

impl ProviderStateIterator {
    /// Create a new iterator
    pub fn new(interaction: *mut dyn Interaction) -> Self {
        ProviderStateIterator { current: 0, message: Mutex::new(interaction) }
    }

    fn next(&mut self) -> usize {
        let idx = self.current;
        self.current += 1;
        idx
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Metadata
 */

ffi_fn! {
    /// Get a copy of the metadata value indexed by `key`.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `Message`.
    ///
    /// The returned pointer will be NULL if the metadata does not contain
    /// the given key, or if an error occurred.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the provided `key` string contains
    /// invalid UTF-8, or if the Rust string contains embedded null ('\0')
    /// bytes.
    fn pactffi_message_find_metadata(message: *const Message, key: *const c_char) -> *const c_char {
        let message = as_ref!(message);
        let key = safe_str!(key);
        let value = message.metadata.get(key).ok_or(anyhow::anyhow!("invalid metadata key"))?;
        let value_ptr = string::to_c(value.as_str().unwrap_or_default())?;
        value_ptr as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Insert the (`key`, `value`) pair into this Message's
    /// `metadata` HashMap.
    ///
    /// # Safety
    ///
    /// This function returns an enum indicating the result;
    /// see the comments on HashMapInsertStatus for details.
    ///
    /// # Error Handling
    ///
    /// This function may fail if the provided `key` or `value` strings
    /// contain invalid UTF-8.
    fn pactffi_message_insert_metadata(
        message: *mut Message,
        key: *const c_char,
        value: *const c_char
    ) -> c_int {
        let message = as_mut!(message);
        let key = safe_str!(key);
        let value = safe_str!(value);

        match message.metadata.insert(key.to_string(), JsonValue::String(value.to_string())) {
            None => HashMapInsertStatus::SuccessNew as c_int,
            Some(_) => HashMapInsertStatus::SuccessOverwrite as c_int,
        }
    } {
        HashMapInsertStatus::Error as c_int
    }
}

ffi_fn! {
    /// Get the next key and value out of the iterator, if possible.
    ///
    /// The returned pointer must be deleted with `pactffi_message_metadata_pair_delete`.
    ///
    /// # Safety
    ///
    /// The underlying data must not change during iteration.
    /// This function must only ever be called from a foreign language. Calling it from a Rust function
    /// that has a Tokio runtime in its call stack can result in a deadlock.
    ///
    /// # Error Handling
    ///
    /// If no further data is present, returns NULL.
    fn pactffi_message_metadata_iter_next(iter: *mut MessageMetadataIterator) -> *mut MessageMetadataPair {
        let iter = as_mut!(iter);
        let generated_metadata;

        let metadata = match iter.message {
            Either::Left(message) => {
                let message = as_ref!(message);
                generated_metadata = block_on(generate_message(message, &GeneratorTestMode::Consumer, &hashmap!{}, &vec![], &hashmap!{})).metadata;
                &generated_metadata
            }
            Either::Right(contents) => {
                let contents = as_ref!(contents);
                &contents.metadata
            }
        };

        match iter.next() {
          Some(key) => {
            let (key, value) = metadata
                .get_key_value(key)
                .ok_or(anyhow::anyhow!("iter provided invalid metadata key"))?;

            let pair = MessageMetadataPair::new(key, json_to_string(&value).as_str())?;
            ptr::raw_to(pair)
          }
          None => {
            trace!("iter past the end of metadata");
            std::ptr::null_mut()
          }
        }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the metadata of a message.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the message, and must
    /// not outlive the message.
    ///
    /// The message metadata also must not be modified during iteration. If it is,
    /// the old iterator must be deleted and a new iterator created.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain
    /// embedded null ('\0') bytes.
    fn pactffi_message_get_metadata_iter(message: *mut Message) -> *mut MessageMetadataIterator {
        let message = as_mut!(message);

        let iter = MessageMetadataIterator {
            keys:  message.metadata.keys().sorted().cloned().collect(),
            current: 0,
            message: Either::Left(message as *const Message)
        };

        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free the metadata iterator when you're done using it.
    fn pactffi_message_metadata_iter_delete(iter: *mut MessageMetadataIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Free a pair of key and value returned from `message_metadata_iter_next`.
    fn pactffi_message_metadata_pair_delete(pair: *mut MessageMetadataPair) {
        ptr::drop_raw(pair);
    }
}

/// An iterator that enables FFI iteration over metadata by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying metadata happens while the iterator is live.
#[derive(Debug)]
pub struct MessageMetadataIterator {
    /// The metadata keys
    keys: Vec<String>,
    /// The current key
    current: usize,
    /// Pointer to either the V3 message or the V4 message contents.
    message: Either<*const Message, *const MessageContents>
}

impl MessageMetadataIterator {
    fn next(&mut self) -> Option<&String> {
        let idx = self.current;
        self.current += 1;
        self.keys.get(idx)
    }

    /// Construct a new iterator that wraps the message contents object
    pub fn new_from_contents(contents: &MessageContents) -> Self {
        MessageMetadataIterator {
            keys:  contents.metadata.keys().sorted().cloned().collect(),
            current: 0,
            message: Either::Right(contents as *const MessageContents)
        }
    }
}

/// A single key-value pair exported to the C-side.
#[derive(Debug)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct MessageMetadataPair {
    /// The metadata key.
    pub key: *const c_char,
    /// The metadata value.
    pub value: *const c_char,
}

impl MessageMetadataPair {
    fn new(
        key: &str,
        value: &str,
    ) -> anyhow::Result<MessageMetadataPair> {
        Ok(MessageMetadataPair {
            key: string::to_c(key)? as *const c_char,
            value: string::to_c(value)? as *const c_char,
        })
    }
}

// Ensure that the owned strings are freed when the pair is dropped.
//
// Notice that we're casting from a `*const c_char` to a `*mut c_char`.
// This may seem wrong, but is safe so long as it doesn't violate Rust's
// guarantees around immutable references, which this doesn't. In this case,
// the underlying data came from `CString::into_raw` which takes ownership
// of the `CString` and hands it off via a `*mut pointer`. We cast that pointer
// back to `*const` to limit the C-side from doing any shenanigans, since the
// pointed-to values live inside of the `Message` metadata `HashMap`, but
// cast back to `*mut` here so we can free the memory.
//
// The discussion here helps explain: https://github.com/rust-lang/rust-clippy/issues/4774
impl Drop for MessageMetadataPair {
    fn drop(&mut self) {
        string::pactffi_string_delete(self.key as *mut c_char);
        string::pactffi_string_delete(self.value as *mut c_char);
    }
}

/*===============================================================================================
 * # Status Types
 *---------------------------------------------------------------------------------------------*/

/// Result from an attempt to insert into a HashMap
enum HashMapInsertStatus {
    /// The value was inserted, and the key was unset
    SuccessNew = 0,
    /// The value was inserted, and the key was previously set
    SuccessOverwrite = -1,
    /// An error occured, and the value was not inserted
    Error = -2,
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;

  use crate::models::message::{
    pactffi_message_delete,
    pactffi_message_get_contents,
    pactffi_message_get_contents_length,
    pactffi_message_new,
    pactffi_message_set_contents
  };

  #[test]
  fn get_and_set_message_contents() {
    let message = pactffi_message_new();
    let message_contents = CString::new("This is a string").unwrap();

    pactffi_message_set_contents(message, message_contents.as_ptr(), std::ptr::null());
    let contents = pactffi_message_get_contents(message) as *mut c_char;
    let len = pactffi_message_get_contents_length(message);

    let str = unsafe { CString::from_raw(contents) };

    pactffi_message_delete(message);

    expect!(str.to_str().unwrap()).to(be_equal_to("This is a string"));
    expect!(len).to(be_equal_to(16));
  }
}
