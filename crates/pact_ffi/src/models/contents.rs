//! Module for functions to deal with request, response and message contents

use bytes::Bytes;
use libc::{c_char, c_uchar, size_t};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::models::generators::{GeneratorCategory, GeneratorCategoryIterator};
use crate::models::matching_rules::{MatchingRuleCategory, MatchingRuleCategoryIterator};
use crate::models::message::MessageMetadataIterator;
use crate::string::optional_str;
use crate::util::*;

ffi_fn! {
    /// Delete the message contents instance.
    ///
    /// # Safety
    ///
    /// This should only be called on a message contents that require deletion.
    /// The function creating the message contents should document whether it
    /// requires deletion.
    ///
    /// Deleting a message content which is associated with an interaction
    /// will result in undefined behaviour.
    fn pactffi_message_contents_delete(contents: *const MessageContents) {
        ptr::drop_raw(contents as *mut MessageContents);
    }
}

ffi_fn! {
    /// Get the message contents in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message contents is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_message_contents_get_contents_str(contents: *const MessageContents) -> *const c_char {
        let contents = as_ref!(contents);

        match contents.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(contents.contents.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the contents of the message as a string.
  ///
  /// * `contents` - the message contents to set the contents for
  /// * `contents_str` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The message contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents string is a NULL pointer, it will set the message contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_message_contents_set_contents_str(contents: *mut MessageContents, contents_str: *const c_char, content_type: *const c_char) {
    let contents = as_mut!(contents);

    if contents_str.is_null() {
      contents.contents = OptionalBody::Null;
    } else {
      let contents_str = safe_str!(contents_str);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      contents.contents = OptionalBody::Present(Bytes::from(contents_str), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the message contents.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0. If the body of the message
    /// is missing, then this function also returns 0.
    fn pactffi_message_contents_get_contents_length(contents: *const MessageContents) -> size_t {
        let contents = as_ref!(contents);

        match &contents.contents {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the contents of a message as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_message_contents_get_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_message_contents_get_contents_bin(contents: *const MessageContents) -> *const c_uchar {
        let contents = as_ref!(contents);

        match &contents.contents {
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
  /// * `message` - the message contents to set the contents for
  /// * `contents_bin` - pointer to contents to copy from
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
  fn pactffi_message_contents_set_contents_bin(
    contents: *mut MessageContents,
    contents_bin: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let contents = as_mut!(contents);

    if contents_bin.is_null() {
      contents.contents = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents_bin, len) };
      let contents_bytes = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      contents.contents = OptionalBody::Present(contents_bytes, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

ffi_fn! {
    /// Get an iterator over the metadata of a message.
    ///
    /// The returned pointer must be deleted with `pactffi_message_metadata_iter_delete` when done
    /// with it.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the message contents, and must
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
    fn pactffi_message_contents_get_metadata_iter(contents: *const MessageContents) -> *mut MessageMetadataIterator {
        let contents = as_ref!(contents);

        let iter = MessageMetadataIterator::new_from_contents(&contents);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the matching rules for a category of a message.
    ///
    /// The returned pointer must be deleted with `pactffi_matching_rules_iter_delete` when done
    /// with it.
    ///
    /// Note that there could be multiple matching rules for the same key, so this iterator will
    /// sequentially return each rule with the same key.
    ///
    /// For sample, given the following rules:
    ///    "$.a" => Type,
    ///    "$.b" => Regex("\\d+"), Number
    ///
    /// This iterator will return a sequence of 3 values: ("$.a", Type), ("$.b", Regex("\\d+")), ("$.b", Number)
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the message or message
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_message_contents_get_matching_rule_iter(
      contents: *const MessageContents,
      category: MatchingRuleCategory
    ) -> *mut MatchingRuleCategoryIterator {
        let contents = as_ref!(contents);

        let iter = MatchingRuleCategoryIterator::new_from_contents(&contents, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the matching rules for a category of an HTTP request.
    ///
    /// The returned pointer must be deleted with `pactffi_matching_rules_iter_delete` when done
    /// with it.
    ///
    /// For sample, given the following rules:
    ///    "$.a" => Type,
    ///    "$.b" => Regex("\\d+"), Number
    ///
    /// This iterator will return a sequence of 3 values: ("$.a", Type), ("$.b", Regex("\\d+")), ("$.b", Number)
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the interaction or request
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_request_contents_get_matching_rule_iter(
      request: *const HttpRequest,
      category: MatchingRuleCategory
    ) -> *mut MatchingRuleCategoryIterator {
        let request = as_ref!(request);

        let iter = MatchingRuleCategoryIterator::new_from_request(&request, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the matching rules for a category of an HTTP response.
    ///
    /// The returned pointer must be deleted with `pactffi_matching_rules_iter_delete` when done
    /// with it.
    ///
    /// For sample, given the following rules:
    ///    "$.a" => Type,
    ///    "$.b" => Regex("\\d+"), Number
    ///
    /// This iterator will return a sequence of 3 values: ("$.a", Type), ("$.b", Regex("\\d+")), ("$.b", Number)
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the interaction or response
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_response_contents_get_matching_rule_iter(
      response: *const HttpResponse,
      category: MatchingRuleCategory
    ) -> *mut MatchingRuleCategoryIterator {
        let response = as_ref!(response);

        let iter = MatchingRuleCategoryIterator::new_from_response(&response, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the generators for a category of a message.
    ///
    /// The returned pointer must be deleted with `pactffi_generators_iter_delete` when done
    /// with it.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the message or message
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_message_contents_get_generators_iter(
      contents: *const MessageContents,
      category: GeneratorCategory
    ) -> *mut GeneratorCategoryIterator {
        let contents = as_ref!(contents);
        let iter = GeneratorCategoryIterator::new_from_contents(&contents, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the generators for a category of an HTTP request.
    ///
    /// The returned pointer must be deleted with `pactffi_generators_iter_delete` when done
    /// with it.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the interaction or request
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_request_contents_get_generators_iter(
      request: *const HttpRequest,
      category: GeneratorCategory
    ) -> *mut GeneratorCategoryIterator {
        let request = as_ref!(request);
        let iter = GeneratorCategoryIterator::new_from_request(&request, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get an iterator over the generators for a category of an HTTP response.
    ///
    /// The returned pointer must be deleted with `pactffi_generators_iter_delete` when done
    /// with it.
    ///
    /// # Safety
    ///
    /// The iterator contains a copy of the data, so is safe to use when the interaction or response
    /// contents has been deleted.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    fn pactffi_response_contents_get_generators_iter(
      response: *const HttpResponse,
      category: GeneratorCategory
    ) -> *mut GeneratorCategoryIterator {
        let response = as_ref!(response);
        let iter = GeneratorCategoryIterator::new_from_response(&response, category);
        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::{CStr, CString};

  use bytes::Bytes;
  use expectest::prelude::*;
  use libc::c_char;
  use maplit::hashmap;
  use pact_models::{generators, matchingrules};
  use pact_models::bodies::OptionalBody;
  use pact_models::content_types::JSON;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::prelude::Generator;
  use pact_models::v4::message_parts::MessageContents;
  use serde_json::json;

  use crate::models::contents::{
    pactffi_message_contents_get_contents_str,
    pactffi_message_contents_get_generators_iter,
    pactffi_message_contents_get_matching_rule_iter,
    pactffi_message_contents_get_metadata_iter
  };
  use crate::models::generators::{GeneratorCategory, pactffi_generator_to_json, pactffi_generators_iter_delete, pactffi_generators_iter_next, pactffi_generators_iter_pair_delete};
  use crate::models::matching_rules::{
    MatchingRuleCategory,
    pactffi_matching_rule_to_json,
    pactffi_matching_rules_iter_delete,
    pactffi_matching_rules_iter_next,
    pactffi_matching_rules_iter_pair_delete
  };
  use crate::models::message::{
    pactffi_message_metadata_iter_delete,
    pactffi_message_metadata_iter_next,
    pactffi_message_metadata_pair_delete
  };

  #[test_log::test]
  fn message_contents_feature_test() {
    let json_contents = json!({
      "a": "b",
      "b": 100
    });
    let json_string = json_contents.to_string();
    let json_bytes = Bytes::from(json_string.clone());
    let message_contents = MessageContents {
      contents: OptionalBody::Present(json_bytes, Some(JSON.clone()), None),
      metadata: hashmap!{
        "a".to_string() => json!("A"),
        "b".to_string() => json!(100)
      },
      matching_rules: matchingrules! {
        "body" => {
          "$.a" => [ MatchingRule::Regex("\\w+".to_string()) ],
          "$.b" => [ MatchingRule::Regex("\\d+".to_string()), MatchingRule::Number ]
        }
      },
      generators: generators! {
        "BODY" => {
          "$.a" => Generator::RandomString(10),
          "$.b" => Generator::RandomHexadecimal(10)
        }
      }
    };
    let message_contents_ptr = &message_contents as *const MessageContents;

    let json_str_ptr = pactffi_message_contents_get_contents_str(message_contents_ptr);
    let json_str = unsafe { CString::from_raw(json_str_ptr as *mut c_char) };
    expect!(json_str.to_string_lossy()).to(be_equal_to(json_string.as_str()));

    let metadata_iter_ptr = pactffi_message_contents_get_metadata_iter(message_contents_ptr);
    expect!(metadata_iter_ptr.is_null()).to(be_false());

    let first_pair = pactffi_message_metadata_iter_next(metadata_iter_ptr);
    expect!(first_pair.is_null()).to(be_false());
    let key = unsafe { CStr::from_ptr((*first_pair).key) };
    let value = unsafe { CStr::from_ptr((*first_pair).value) };
    expect!(key.to_string_lossy()).to(be_equal_to("a"));
    expect!(value.to_string_lossy()).to(be_equal_to("A"));
    pactffi_message_metadata_pair_delete(first_pair);

    let second_pair = pactffi_message_metadata_iter_next(metadata_iter_ptr);
    expect!(second_pair.is_null()).to(be_false());
    let key = unsafe { CStr::from_ptr((*second_pair).key) };
    let value = unsafe { CStr::from_ptr((*second_pair).value) };
    expect!(key.to_string_lossy()).to(be_equal_to("b"));
    expect!(value.to_string_lossy()).to(be_equal_to("100"));
    pactffi_message_metadata_pair_delete(second_pair);

    let third_pair = pactffi_message_metadata_iter_next(metadata_iter_ptr);
    expect!(third_pair.is_null()).to(be_true());

    pactffi_message_metadata_iter_delete(metadata_iter_ptr);

    let matching_rule_iter_pointer = pactffi_message_contents_get_matching_rule_iter(message_contents_ptr, MatchingRuleCategory::BODY);
    expect!(matching_rule_iter_pointer.is_null()).to(be_false());

    let mr_first_pair = pactffi_matching_rules_iter_next(matching_rule_iter_pointer);
    expect!(mr_first_pair.is_null()).to(be_false());
    let path = unsafe { CStr::from_ptr((*mr_first_pair).path) };
    let rule = unsafe { CString::from_raw(pactffi_matching_rule_to_json((*mr_first_pair).rule) as *mut c_char) };
    expect!(path.to_string_lossy()).to(be_equal_to("$.a"));
    expect!(rule.to_string_lossy()).to(be_equal_to("{\"match\":\"regex\",\"regex\":\"\\\\w+\"}"));
    pactffi_matching_rules_iter_pair_delete(mr_first_pair);

    let mr_second_pair = pactffi_matching_rules_iter_next(matching_rule_iter_pointer);
    expect!(mr_second_pair.is_null()).to(be_false());
    let path = unsafe { CStr::from_ptr((*mr_second_pair).path) };
    let rule = unsafe { CString::from_raw(pactffi_matching_rule_to_json((*mr_second_pair).rule) as *mut c_char) };
    expect!(path.to_string_lossy()).to(be_equal_to("$.b"));
    expect!(rule.to_string_lossy()).to(be_equal_to("{\"match\":\"regex\",\"regex\":\"\\\\d+\"}"));
    pactffi_matching_rules_iter_pair_delete(mr_second_pair);

    let mr_third_pair = pactffi_matching_rules_iter_next(matching_rule_iter_pointer);
    expect!(mr_third_pair.is_null()).to(be_false());
    let path = unsafe { CStr::from_ptr((*mr_third_pair).path) };
    let rule = unsafe { CString::from_raw(pactffi_matching_rule_to_json((*mr_third_pair).rule) as *mut c_char) };
    expect!(path.to_string_lossy()).to(be_equal_to("$.b"));
    expect!(rule.to_string_lossy()).to(be_equal_to("{\"match\":\"number\"}"));
    pactffi_matching_rules_iter_pair_delete(mr_third_pair);

    let mr_fouth_pair = pactffi_matching_rules_iter_next(matching_rule_iter_pointer);
    expect!(mr_fouth_pair.is_null()).to(be_true());

    pactffi_matching_rules_iter_delete(matching_rule_iter_pointer);

    let generator_iter_pointer = pactffi_message_contents_get_generators_iter(message_contents_ptr, GeneratorCategory::BODY);
    expect!(generator_iter_pointer.is_null()).to(be_false());

    let gen_first_pair = pactffi_generators_iter_next(generator_iter_pointer);
    expect!(gen_first_pair.is_null()).to(be_false());
    let path = unsafe { CStr::from_ptr((*gen_first_pair).path) };
    let generator = unsafe { CString::from_raw(pactffi_generator_to_json((*gen_first_pair).generator) as *mut c_char) };
    expect!(path.to_string_lossy()).to(be_equal_to("$.a"));
    expect!(generator.to_string_lossy()).to(be_equal_to("{\"size\":10,\"type\":\"RandomString\"}"));
    pactffi_generators_iter_pair_delete(gen_first_pair);

    let gen_second_pair = pactffi_generators_iter_next(generator_iter_pointer);
    expect!(gen_second_pair.is_null()).to(be_false());
    let path = unsafe { CStr::from_ptr((*gen_second_pair).path) };
    let generator = unsafe { CString::from_raw(pactffi_generator_to_json((*gen_second_pair).generator) as *mut c_char) };
    expect!(path.to_string_lossy()).to(be_equal_to("$.b"));
    expect!(generator.to_string_lossy()).to(be_equal_to("{\"digits\":10,\"type\":\"RandomHexadecimal\"}"));
    pactffi_generators_iter_pair_delete(gen_second_pair);

    let mr_third_pair = pactffi_generators_iter_next(generator_iter_pointer);
    expect!(mr_third_pair.is_null()).to(be_true());

    pactffi_generators_iter_delete(generator_iter_pointer);
  }
}
