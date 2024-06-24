//! V4 Synchronous request/response messages

use std::collections::HashMap;

use anyhow::anyhow;
use bytes::Bytes;
use futures::executor::block_on;
use libc::{c_char, c_int, c_uchar, c_uint, EXIT_FAILURE, EXIT_SUCCESS, size_t};
use pact_matching::generators::apply_generators_to_sync_message;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::generators::GeneratorTestMode;
use pact_models::provider_states::ProviderState;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::models::message::ProviderStateIterator;
use crate::ptr;
use crate::util::*;
use crate::util::string::optional_str;

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
    fn pactffi_sync_message_new() -> *mut SynchronousMessage {
        let message = SynchronousMessage::default();
        ptr::raw_to(message)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Destroy the `Message` being pointed to.
    fn pactffi_sync_message_delete(message: *mut SynchronousMessage) {
        ptr::drop_raw(message);
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousMessage` in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the request message
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_sync_message_get_request_contents_str(message: *const SynchronousMessage) -> *const c_char {
        let message = as_ref!(message);

        match message.request.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(message.request.contents.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the request contents of the message.
  ///
  /// * `message` - the message to set the request contents for
  /// * `contents` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
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
  fn pactffi_sync_message_set_request_contents_str(message: *mut SynchronousMessage, contents: *const c_char, content_type: *const c_char) {
    let message = as_mut!(message);

    if contents.is_null() {
      message.request.contents = OptionalBody::Null;
    } else {
      let contents = safe_str!(contents);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      message.request.contents = OptionalBody::Present(Bytes::from(contents), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the request contents of a `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0. If the body of the request
    /// is missing, then this function also returns 0.
    fn pactffi_sync_message_get_request_contents_length(message: *const SynchronousMessage) -> size_t {
        let message = as_ref!(message);

        match &message.request.contents {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousMessage` as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_message_get_request_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_message_get_request_contents_bin(message: *const SynchronousMessage) -> *const c_uchar {
        let message = as_ref!(message);

        match &message.request.contents {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the request contents of the message as an array of bytes.
  ///
  /// * `message` - the message to set the request contents for
  /// * `contents` - pointer to contents to copy from
  /// * `len` - number of bytes to copy from the contents pointer
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
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
  fn pactffi_sync_message_set_request_contents_bin(
    message: *mut SynchronousMessage,
    contents: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let message = as_mut!(message);

    if contents.is_null() {
      message.request.contents = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents, len) };
      let contents = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      message.request.contents = OptionalBody::Present(contents, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

ffi_fn! {
    /// Get the request contents of an `SynchronousMessage` as a `MessageContents` pointer.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer this function returns will be deleted when the message
    /// is deleted. Trying to use if after the message is deleted will result in undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL.
    fn pactffi_sync_message_get_request_contents(message: *const SynchronousMessage) -> *const MessageContents {
        let message = as_ref!(message);
        &message.request as *const MessageContents
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Generate the request contents of a `SynchronousMessage` as a
    /// `MessageContents` pointer.
    ///
    /// This function differs from [`pactffi_sync_message_get_request_contents`]
    /// in that it will process the message contents for any generators or
    /// matchers that are present in the message in order to generate the actual
    /// message contents as would be received by the consumer.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer must be deleted with
    /// [`pactffi_message_contents_delete`][crate::models::contents::pactffi_message_contents_delete]
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL.
    fn pactffi_sync_message_generate_request_contents(message: *const SynchronousMessage) -> *const MessageContents {
        let message = as_ref!(message);
        let context = HashMap::new();
        let plugin_data = Vec::new();
        let interaction_data = HashMap::new();
        let (contents, _) = block_on(apply_generators_to_sync_message(
            &message,
            &GeneratorTestMode::Consumer,
            &context,
            &plugin_data,
            &interaction_data,
        ));
        ptr::raw_to(contents) as *const MessageContents
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get the number of response messages in the `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// The message pointer must point to a valid SynchronousMessage.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0.
    fn pactffi_sync_message_get_number_responses(message: *const SynchronousMessage) -> size_t {
        let message = as_ref!(message);
        message.response.len() as size_t
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousMessage` in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns NULL.
    ///
    /// If the body of the response message is missing, then this function also returns NULL.
    /// This means there's no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_sync_message_get_response_contents_str(message: *const SynchronousMessage, index: size_t) -> *const c_char {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match response.contents {
                // If it's missing, return a null pointer.
                OptionalBody::Missing => std::ptr::null(),
                // If empty or null, return an empty string on the heap.
                OptionalBody::Empty | OptionalBody::Null => {
                    let content = string::to_c("")?;
                    content as *const c_char
                }
                // Otherwise, get the contents, possibly still empty.
                _ => {
                    let content = string::to_c(response.contents.value_as_string().unwrap_or_default().as_str())?;
                    content as *const c_char
                }
            }
            None => std::ptr::null()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the response contents of the message as a string. If index is greater than the number of responses
  /// in the message, the responses will be padded with default values.
  ///
  /// * `message` - the message to set the response contents for
  /// * `index` - index of the response to set. 0 is the first response.
  /// * `contents` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The message contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the response contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_sync_message_set_response_contents_str(
    message: *mut SynchronousMessage,
    index: size_t,
    contents: *const c_char,
    content_type: *const c_char
  ) {
    let message = as_mut!(message);

    let response = match message.response.get_mut(index) {
      Some(response) => response,
      None => {
        message.response.resize(index + 1, MessageContents::default());
        message.response.get_mut(index).unwrap()
      }
    };

    if contents.is_null() {
      response.contents = OptionalBody::Null;
    } else {
      let contents = safe_str!(contents);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      response.contents = OptionalBody::Present(Bytes::from(contents), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the response contents of a `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns 0. If the body of the request
    /// is missing, then this function also returns 0.
    fn pactffi_sync_message_get_response_contents_length(message: *const SynchronousMessage, index: size_t) -> size_t {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match &response.contents {
                OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
                OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
            }
            None => 0 as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousMessage` as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_message_get_response_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_message_get_response_contents_bin(message: *const SynchronousMessage, index: size_t) -> *const c_uchar {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match &response.contents {
                OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
                OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
            }
            None => std::ptr::null()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the response contents of the message at the given index as an array of bytes. If index
  /// is greater than the number of responses in the message, the responses will be padded with
  /// default values.
  ///
  /// * `message` - the message to set the response contents for
  /// * `index` - index of the response to set. 0 is the first response
  /// * `contents` - pointer to contents to copy from
  /// * `len` - number of bytes to copy
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
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
  fn pactffi_sync_message_set_response_contents_bin(
    message: *mut SynchronousMessage,
    index: size_t,
    contents: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let message = as_mut!(message);

    let response = match message.response.get_mut(index) {
      Some(response) => response,
      None => {
        message.response.resize(index + 1, MessageContents::default());
        message.response.get_mut(index).unwrap()
      }
    };

    if contents.is_null() {
      response.contents = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents, len) };
      let contents = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      response.contents = OptionalBody::Present(contents, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

ffi_fn! {
    /// Get the response contents of an `SynchronousMessage` as a `MessageContents` pointer.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer this function returns will be deleted when the message
    /// is deleted. Trying to use if after the message is deleted will result in undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns NULL.
    fn pactffi_sync_message_get_response_contents(message: *const SynchronousMessage, index: size_t) -> *const MessageContents {
        let message = as_ref!(message);
        if let Some(response) = message.response.get(index) {
          response as *const MessageContents
        } else {
          std::ptr::null()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Generate the response contents of a `SynchronousMessage` as a
    /// `MessageContents` pointer.
    ///
    /// This function differs from
    /// [`pactffi_sync_message_get_response_contents`] in that it will process
    /// the message contents for any generators or matchers that are present in
    /// the message in order to generate the actual message contents as would be
    /// received by the consumer.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer must be deleted with
    /// [`pactffi_message_contents_delete`][crate::models::contents::pactffi_message_contents_delete]
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL.
    fn pactffi_sync_message_generate_response_contents(message: *const SynchronousMessage, index: size_t) -> *const MessageContents {
        let message = as_ref!(message);
        if index >= message.response.len() {
            return Ok(std::ptr::null());
        }

        let context = HashMap::new();
        let plugin_data = Vec::new();
        let interaction_data = HashMap::new();
        let (_, mut responses) = block_on(apply_generators_to_sync_message(
            &message,
            &GeneratorTestMode::Consumer,
            &context,
            &plugin_data,
            &interaction_data,
        ));
        ptr::raw_to(responses.swap_remove(index)) as *const MessageContents
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get a copy of the description.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `SynchronousMessage`.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the Rust string contains embedded
    /// null ('\0') bytes.
    fn pactffi_sync_message_get_description(message: *const SynchronousMessage) -> *const c_char {
        let message = as_ref!(message);
        let description = string::to_c(&message.description)?;
        description as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Write the `description` field on the `SynchronousMessage`.
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
    fn pactffi_sync_message_set_description(message: *mut SynchronousMessage, description: *const c_char) -> c_int {
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


ffi_fn! {
    /// Get a copy of the provider state at the given index from this message.
    ///
    /// # Safety
    ///
    /// The returned structure must be deleted with `provider_state_delete`.
    ///
    /// Since it is a copy, the returned structure may safely outlive
    /// the `SynchronousMessage`.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a variant other than Success.
    ///
    /// This function may fail if the index requested is out of bounds,
    /// or if any of the Rust strings contain embedded null ('\0') bytes.
    fn pactffi_sync_message_get_provider_state(message: *const SynchronousMessage, index: c_uint) -> *const ProviderState {
        let message = as_ref!(message);
        let index = index as usize;

        // Get a raw pointer directly, rather than boxing it, as its owned by the `SynchronousMessage`
        // and will be cleaned up when the `SynchronousMessage` is cleaned up.
        let provider_state = message
            .provider_states
            .get(index)
            .ok_or(anyhow!("index is out of bounds"))?;

        provider_state as *const ProviderState
    } {
        std::ptr::null_mut()
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
    fn pactffi_sync_message_get_provider_state_iter(message: *mut SynchronousMessage) -> *mut ProviderStateIterator {
        let message = as_mut!(message);
        let iter = ProviderStateIterator::new(message);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;

  use pact_models::generators;
  use pact_models::generators::Generator;

  use super::{
    pactffi_sync_message_delete,
    pactffi_sync_message_generate_request_contents,
    pactffi_sync_message_get_request_contents_length,
    pactffi_sync_message_get_request_contents_str,
    pactffi_sync_message_get_response_contents_length,
    pactffi_sync_message_get_response_contents_str,
    pactffi_sync_message_new,
    pactffi_sync_message_set_request_contents_str,
    pactffi_sync_message_set_response_contents_str,
  };

    #[test]
    fn get_and_set_message_contents() {
      let message = pactffi_sync_message_new();
      let message_contents = CString::new("This is a string").unwrap();
      let message_contents2 = CString::new("This is another string").unwrap();
      let content_type = CString::new("text/plain").unwrap();

      pactffi_sync_message_set_request_contents_str(message, message_contents.as_ptr(), std::ptr::null());
      let contents = pactffi_sync_message_get_request_contents_str(message) as *mut c_char;
      let len = pactffi_sync_message_get_request_contents_length(message);
      let str = unsafe { CString::from_raw(contents) };

      pactffi_sync_message_set_response_contents_str(message, 2, message_contents2.as_ptr(),
        content_type.as_ptr());
      let response_contents = pactffi_sync_message_get_response_contents_str(message, 0) as *mut c_char;
      let response_len = pactffi_sync_message_get_response_contents_length(message, 0);
      let response_contents1 = pactffi_sync_message_get_response_contents_str(message, 1) as *mut c_char;
      let response_len1 = pactffi_sync_message_get_response_contents_length(message, 1);
      let contents2 = pactffi_sync_message_get_response_contents_str(message, 2) as *mut c_char;
      let response_len2 = pactffi_sync_message_get_response_contents_length(message, 2);
      let response_str2 = unsafe { CString::from_raw(contents2) };

      pactffi_sync_message_delete(message);

      expect!(str.to_str().unwrap()).to(be_equal_to("This is a string"));
      expect!(len).to(be_equal_to(16));

      expect!(response_contents.is_null()).to(be_true());
      expect!(response_len).to(be_equal_to(0));
      expect!(response_contents1.is_null()).to(be_true());
      expect!(response_len1).to(be_equal_to(0));
      expect!(response_str2.to_str().unwrap()).to(be_equal_to("This is another string"));
      expect!(response_len2).to(be_equal_to(22));
    }

    #[test]
    fn test_generate_contents() {
        let message = pactffi_sync_message_new();
        let message_contents = CString::new(r#"{ "id": 1 }"#).unwrap();
        let content_type = CString::new("application/json").unwrap();
        pactffi_sync_message_set_request_contents_str(message, message_contents.as_ptr(), content_type.as_ptr());

        unsafe { &mut *message }.request.generators.add_generators(generators!{
            "body" => {
                "$.id" => Generator::RandomInt(1000, 1000)
            }
        });

        let contents = pactffi_sync_message_generate_request_contents(message);

        assert_eq!(
            r#"{"id":1000}"#,
            unsafe { &*contents }.contents.value_as_string().unwrap()
        );
    }
}
