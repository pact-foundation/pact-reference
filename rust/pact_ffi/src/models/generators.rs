//! FFI functions to deal with generators and generated values

use anyhow::{anyhow, Context};
use libc::{c_char, c_ushort};
use pact_models::generators::{GenerateValue, Generator, NoopVariantMatcher, VariantMatcher};
use serde_json::Value;
use tracing::{error, warn};

use crate::{as_ref, ffi_fn, safe_str};
use crate::util::{ptr, string};

ffi_fn! {
  /// Get the JSON form of the generator.
  ///
  /// The returned string must be deleted with `pactffi_string_delete`.
  ///
  /// # Safety
  ///
  /// This function will fail if it is passed a NULL pointer, or the owner of the generator has
  /// been deleted.
  fn pactffi_generator_to_json(generator: *const Generator) -> *const c_char {
    let generator = as_ref!(generator);
    let json = generator.to_json().unwrap_or_default().to_string();
    string::to_c(&json)? as *const c_char
  } {
    ptr::null_to::<c_char>()
  }
 }

ffi_fn! {
  /// Get a Generator from its JSON representation.
  ///
  /// Will return a NULL pointer if the generator was invalid.
  ///
  /// # Safety
  ///
  /// This function will fail if it is passed a NULL pointer, or the owner of the generator has
  /// been deleted.
  fn pactffi_generator_from_json(generator: *const c_char) -> *const Generator {
    let generator = safe_str!(generator);
    let value: serde_json::Value = serde_json::from_str(generator).context("error parsing generator as JSON")?;
    let result = Generator::from_json(&value);

    match result {
      Ok(rule) => ptr::raw_to(rule) as *const Generator,
      _ => ptr::null_to::<Generator>()
    }
  } {
      ptr::null_to::<Generator>()
  }
}

ffi_fn! {
  /// Generate a string value using the provided generator and an optional JSON payload containing
  /// any generator context. The context value is used for generators like `MockServerURL` (which
  /// should contain details about the running mock server) and `ProviderStateGenerator` (which
  /// should be the values returned from the Provider State callback function).
  ///
  /// If anything goes wrong, it will return a NULL pointer.
  fn pactffi_generator_generate_string(
    generator: *const Generator,
    context_json: *const c_char
  ) -> *const c_char {
    let generator = as_ref!(generator);
    let context = string::optional_str(context_json);

    let context_entries = context_map(context)?;
    let map = context_entries.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    match generator.generate_value(&"".to_string(), &map, &NoopVariantMatcher.boxed()) {
      Ok(value) => string::to_c(value.as_str())? as *const c_char,
      Err(err) => {
        error!("Failed to generate value - {}", err);
        ptr::null_to::<c_char>()
      }
    }
  } {
    ptr::null_to::<c_char>()
  }
}

fn context_map<'a>(context: Option<String>) -> anyhow::Result<serde_json::Map<String, Value>> {
  if let Some(context) = context {
    match serde_json::from_str::<Value>(context.as_str()) {
      Ok(json) => match json {
        Value::Object(entries) => Ok(entries.clone()),
        _ => {
          warn!("'{}' is not a JSON object, ignoring it", json);
          Ok(serde_json::Map::default())
        }
      },
      Err(err) => {
        error!("Failed to parse the context value as JSON - {}", err);
        Err(anyhow!("Failed to parse the context value as JSON - {}", err))
      }
    }
  } else {
    Ok(serde_json::Map::default())
  }
}

ffi_fn! {
  /// Generate an integer value using the provided generator and an optional JSON payload containing
  /// any generator context. The context value is used for generators like `ProviderStateGenerator`
  /// (which should be the values returned from the Provider State callback function).
  ///
  /// If anything goes wrong or the generator is not a type that can generate an integer value, it
  /// will return a zero value.
  fn pactffi_generator_generate_integer(
    generator: *const Generator,
    context_json: *const c_char
  ) -> c_ushort {
    let generator = as_ref!(generator);
    let context = string::optional_str(context_json);

    let context_entries = context_map(context)?;
    let map = context_entries.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    match generator.generate_value(&0, &map, &NoopVariantMatcher.boxed()) {
      Ok(value) => value,
      Err(err) => {
        error!("Failed to generate value - {}", err);
        0
      }
    }
  } {
    0
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;
  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::generators::{Generator};
  use pact_models::prelude::Generator::{RandomInt, RandomString};

  use crate::models::generators::{
    pactffi_generator_generate_integer,
    pactffi_generator_generate_string,
    pactffi_generator_to_json,
    pactffi_generator_from_json,
  };
  use crate::util::ptr::null_to;
  use crate::util::string;

  #[test]
  fn generate_string_test() {
    let generator = RandomString(4);

    let value = pactffi_generator_generate_string(&generator, null_to());
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy().len()).to(be_equal_to(4));
  }

  #[test]
  fn generate_string_test_with_invalid_context() {
    let generator = RandomString(4);
    let context = "{not valid";

    let context_json = string::to_c(context).unwrap();
    let value = pactffi_generator_generate_string(&generator, context_json);
    expect!(value.is_null()).to(be_true());
  }

  #[test]
  fn generate_integer_test() {
    let generator = RandomInt(10, 100);

    let value = pactffi_generator_generate_integer(&generator, null_to());
    expect!(value).to(be_greater_or_equal_to(10));
    expect!(value).to(be_less_or_equal_to(100));
  }

  #[test]
  fn generator_json() {
    let generator = RandomInt(10, 100);
    let generator_ptr = &generator as *const Generator;
    let json_ptr = pactffi_generator_to_json(generator_ptr);
    let json = unsafe { CString::from_raw(json_ptr as *mut c_char) };
    expect!(json.to_string_lossy()).to(be_equal_to("{\"max\":100,\"min\":10,\"type\":\"RandomInt\"}"));
  }

  #[test]
  fn generator_from_json() {
    let json_string = CString::new("{\"max\":100,\"min\":10,\"type\":\"RandomInt\"}").unwrap();
    let rule_ptr = pactffi_generator_from_json(json_string.as_ptr());

    unsafe {
      expect!(rule_ptr.as_ref().unwrap().name()).to(be_eq("RandomInt"));
    }
  }
}