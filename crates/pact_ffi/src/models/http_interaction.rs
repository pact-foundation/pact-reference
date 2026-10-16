//! Structs and functions to deal with HTTP Pact interactions

use anyhow::anyhow;
use bytes::Bytes;
use libc::{c_char, c_int, c_uchar, c_uint, EXIT_FAILURE, EXIT_SUCCESS, size_t};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::provider_states::ProviderState;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::synch_http::SynchronousHttp;

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::models::message::ProviderStateIterator;
use crate::ptr;
use crate::util::*;
use crate::util::string::optional_str;

ffi_fn! {
    /// Get a mutable pointer to a newly-created default interaction on the heap.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// Returns NULL on error.
    fn pactffi_sync_http_new() -> *mut SynchronousHttp {
        let interaction = SynchronousHttp::default();
        ptr::raw_to(interaction)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Destroy the `SynchronousHttp` interaction being pointed to.
    fn pactffi_sync_http_delete(interaction: *const SynchronousHttp) {
        ptr::drop_raw(interaction as *mut SynchronousHttp);
    }
}

ffi_fn! {
    /// Get the request of a `SynchronousHttp` interaction.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer this function returns will be deleted when the interaction
    /// is deleted. Trying to use if after the interaction is deleted will result in undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL.
    fn pactffi_sync_http_get_request(interaction: *const SynchronousHttp) -> *const HttpRequest {
        let interaction = as_ref!(interaction);
        &interaction.request as *const HttpRequest
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousHttp` interaction in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the interaction.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL. If the body of the request
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL body and a missing body.
    fn pactffi_sync_http_get_request_contents(interaction: *const SynchronousHttp) -> *const c_char {
        let interaction = as_ref!(interaction);

        match interaction.request.body {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(interaction.request.body.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the request contents of the interaction.
  ///
  /// * `interaction` - the interaction to set the request contents for
  /// * `contents` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The request contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the request contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_sync_http_set_request_contents(
    interaction: *mut SynchronousHttp,
    contents: *const c_char,
    content_type: *const c_char
  ) {
    let interaction = as_mut!(interaction);

    if contents.is_null() {
      interaction.request.body = OptionalBody::Null;
    } else {
      let contents = safe_str!(contents);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      interaction.request.body = OptionalBody::Present(Bytes::from(contents), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the request contents of a `SynchronousHttp` interaction.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns 0. If the body of the request
    /// is missing, then this function also returns 0.
    fn pactffi_sync_http_get_request_contents_length(interaction: *const SynchronousHttp) -> size_t {
        let interaction = as_ref!(interaction);

        match &interaction.request.body {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousHttp` interaction as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_http_get_request_contents_length`.
    /// It is safe to use the pointer while the interaction is not deleted or changed. Using the pointer
    /// after the interaction is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL. If the body of the request
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_http_get_request_contents_bin(interaction: *const SynchronousHttp) -> *const c_uchar {
        let interaction = as_ref!(interaction);

        match &interaction.request.body {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the request contents of the interaction as an array of bytes.
  ///
  /// * `interaction` - the interaction to set the request contents for
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
  /// If the contents is a NULL pointer, it will set the request contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_sync_http_set_request_contents_bin(
    interaction: *mut SynchronousHttp,
    contents: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let interaction = as_mut!(interaction);

    if contents.is_null() {
      interaction.request.body = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents, len) };
      let contents = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      interaction.request.body = OptionalBody::Present(contents, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

ffi_fn! {
    /// Get the response of a `SynchronousHttp` interaction.
    ///
    /// # Safety
    ///
    /// The data pointed to by the pointer this function returns will be deleted when the interaction
    /// is deleted. Trying to use if after the interaction is deleted will result in undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL.
    fn pactffi_sync_http_get_response(interaction: *const SynchronousHttp) -> *const HttpResponse {
        let interaction = as_ref!(interaction);
        &interaction.response as *const HttpResponse
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousHttp` interaction in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the interaction.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL.
    ///
    /// If the body of the response is missing, then this function also returns NULL.
    /// This means there's no mechanism to differentiate with this function call alone between
    /// a NULL body and a missing body.
    fn pactffi_sync_http_get_response_contents(interaction: *const SynchronousHttp) -> *const c_char {
        let interaction = as_ref!(interaction);

        match interaction.response.body {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(interaction.response.body.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the response contents of the interaction.
  ///
  /// * `interaction` - the interaction to set the response contents for
  /// * `contents` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The response contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the response contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_sync_http_set_response_contents(
    interaction: *mut SynchronousHttp,
    contents: *const c_char,
    content_type: *const c_char
  ) {
    let interaction = as_mut!(interaction);

    if contents.is_null() {
      interaction.response.body = OptionalBody::Null;
    } else {
      let contents = safe_str!(contents);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      interaction.response.body = OptionalBody::Present(Bytes::from(contents), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the response contents of a `SynchronousHttp` interaction.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL or the index is not valid, returns 0. If the body of the response
    /// is missing, then this function also returns 0.
    fn pactffi_sync_http_get_response_contents_length(interaction: *const SynchronousHttp) -> size_t {
        let interaction = as_ref!(interaction);

        match &interaction.response.body {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousHttp` interaction as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_http_get_response_contents_length`.
    /// It is safe to use the pointer while the interaction is not deleted or changed. Using the pointer
    /// after the interaction is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the interaction is NULL, returns NULL. If the body of the response
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_http_get_response_contents_bin(interaction: *const SynchronousHttp) -> *const c_uchar {
        let interaction = as_ref!(interaction);

        match &interaction.response.body {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the response contents of the `SynchronousHttp` interaction as an array of bytes.
  ///
  /// * `interaction` - the interaction to set the response contents for
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
  /// If the contents is a NULL pointer, it will set the response contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_sync_http_set_response_contents_bin(
    interaction: *mut SynchronousHttp,
    contents: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let interaction = as_mut!(interaction);

    if contents.is_null() {
      interaction.response.body = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents, len) };
      let contents = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      interaction.response.body = OptionalBody::Present(contents, content_type, Some(ContentTypeHint::BINARY));
    }
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
    /// the `SynchronousHttp` interaction.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the Rust string contains embedded
    /// null ('\0') bytes.
    fn pactffi_sync_http_get_description(interaction: *const SynchronousHttp) -> *const c_char {
        let interaction = as_ref!(interaction);
        let description = string::to_c(&interaction.description)?;
        description as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Write the `description` field on the `SynchronousHttp`.
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
    fn pactffi_sync_http_set_description(interaction: *mut SynchronousHttp, description: *const c_char) -> c_int {
        let interaction = as_mut!(interaction);
        let description = safe_str!(description);

        // Wipe out the previous contents of the string, without
        // deallocating, and set the new description.
        interaction.description.clear();
        interaction.description.push_str(description);

        EXIT_SUCCESS
    } {
        EXIT_FAILURE
    }
}


ffi_fn! {
    /// Get a copy of the provider state at the given index from this interaction.
    ///
    /// # Safety
    ///
    /// The returned structure must be deleted with `provider_state_delete`.
    ///
    /// Since it is a copy, the returned structure may safely outlive
    /// the `SynchronousHttp`.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a variant other than Success.
    ///
    /// This function may fail if the index requested is out of bounds,
    /// or if any of the Rust strings contain embedded null ('\0') bytes.
    fn pactffi_sync_http_get_provider_state(interaction: *const SynchronousHttp, index: c_uint) -> *const ProviderState {
        let interaction = as_ref!(interaction);
        let index = index as usize;

        // Get a raw pointer directly, rather than boxing it, as its owned by the `SynchronousHttp`
        // and will be cleaned up when the `SynchronousHttp` is cleaned up.
        let provider_state = interaction
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
    fn pactffi_sync_http_get_provider_state_iter(interaction: *mut SynchronousHttp) -> *mut ProviderStateIterator {
        let interaction = as_mut!(interaction);
        let iter = ProviderStateIterator::new(interaction);
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

  use crate::models::http_interaction::{
    pactffi_sync_http_delete,
    pactffi_sync_http_get_request_contents,
    pactffi_sync_http_get_request_contents_length,
    pactffi_sync_http_get_response_contents,
    pactffi_sync_http_get_response_contents_length,
    pactffi_sync_http_new,
    pactffi_sync_http_set_request_contents,
    pactffi_sync_http_set_response_contents
  };

  #[test]
  fn get_and_set_http_contents() {
    let http = pactffi_sync_http_new();
    let http_contents = CString::new("This is a string").unwrap();
    let http_contents2 = CString::new("This is another string").unwrap();
    let content_type = CString::new("text/plain").unwrap();

    pactffi_sync_http_set_request_contents(http, http_contents.as_ptr(), std::ptr::null());
    let contents = pactffi_sync_http_get_request_contents(http) as *mut c_char;
    let len = pactffi_sync_http_get_request_contents_length(http);
    let str = unsafe { CString::from_raw(contents) };

    pactffi_sync_http_set_response_contents(http, http_contents2.as_ptr(),
                                               content_type.as_ptr());
    let response_contents = pactffi_sync_http_get_response_contents(http) as *mut c_char;
    let response_len = pactffi_sync_http_get_response_contents_length(http);
    let response_str = unsafe { CString::from_raw(response_contents) };

    pactffi_sync_http_delete(http);

    expect!(str.to_str().unwrap()).to(be_equal_to("This is a string"));
    expect!(len).to(be_equal_to(16));

    expect!(response_str.to_str().unwrap()).to(be_equal_to("This is another string"));
    expect!(response_len).to(be_equal_to(22));
  }
}
