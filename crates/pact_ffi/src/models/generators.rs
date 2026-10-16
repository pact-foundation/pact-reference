//! FFI functions to deal with generators and generated values

use std::collections::HashMap;
use anyhow::anyhow;
use itertools::Itertools;
use libc::{c_char, c_ushort};
use maplit::hashmap;
use pact_models::generators::{
  GeneratorCategory as CoreGeneratorCategory,
  GenerateValue,
  Generator,
  NoopVariantMatcher,
  VariantMatcher
};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;
use serde_json::Value;
use tracing::{error, warn, trace};

use crate::{as_mut, as_ref, ffi_fn};
use crate::util::{ptr, string};
use crate::util::ptr::{drop_raw, raw_to};

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
    std::ptr::null()
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
        std::ptr::null()
      }
    }
  } {
    std::ptr::null()
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

/// Enum defining the categories that generators can be applied to
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GeneratorCategory {
  /// Request Method
  METHOD,
  /// Request Path
  PATH,
  /// Request/Response Header
  HEADER,
  /// Request Query Parameter
  QUERY,
  /// Body
  BODY,
  /// Response Status
  STATUS,
  /// Message metadata
  METADATA
}

impl From<CoreGeneratorCategory> for GeneratorCategory {
  #[inline]
  fn from(category: CoreGeneratorCategory) -> GeneratorCategory {
    match category {
      CoreGeneratorCategory::METHOD => GeneratorCategory::METHOD,
      CoreGeneratorCategory::PATH => GeneratorCategory::PATH,
      CoreGeneratorCategory::HEADER => GeneratorCategory::HEADER,
      CoreGeneratorCategory::QUERY => GeneratorCategory::QUERY,
      CoreGeneratorCategory::BODY => GeneratorCategory::BODY,
      CoreGeneratorCategory::STATUS => GeneratorCategory::STATUS,
      CoreGeneratorCategory::METADATA => GeneratorCategory::METADATA
    }
  }
}

impl From<GeneratorCategory> for CoreGeneratorCategory {
  #[inline]
  fn from(category: GeneratorCategory) -> CoreGeneratorCategory {
    match category {
      GeneratorCategory::METHOD => CoreGeneratorCategory::METHOD,
      GeneratorCategory::PATH => CoreGeneratorCategory::PATH,
      GeneratorCategory::HEADER => CoreGeneratorCategory::HEADER,
      GeneratorCategory::QUERY => CoreGeneratorCategory::QUERY,
      GeneratorCategory::BODY => CoreGeneratorCategory::BODY,
      GeneratorCategory::STATUS => CoreGeneratorCategory::STATUS,
      GeneratorCategory::METADATA => CoreGeneratorCategory::METADATA
    }
  }
}

/// An iterator that enables FFI iteration over the generators for a particular generator
/// category.
#[derive(Debug)]
pub struct GeneratorCategoryIterator {
  generators: Vec<(DocPath, Generator)>,
  current_idx: usize
}

impl GeneratorCategoryIterator {
  /// Creates a new iterator over generators of a particular category
  fn new(generators: &HashMap<DocPath, Generator>) -> GeneratorCategoryIterator {
    let generators = generators.iter()
      .sorted_by(|(a, _), (b, _)| Ord::cmp(a.to_string().as_str(), b.to_string().as_str()))
      .map(|(k, v)| (k.clone(), v.clone()));
    GeneratorCategoryIterator {
      generators: generators.collect(),
      current_idx: 0
    }
  }

  /// Create a new iterator for the generators from a message contents
  pub fn new_from_contents(contents: &MessageContents, category: GeneratorCategory) -> Self {
    let category: CoreGeneratorCategory = category.into();
    let empty = hashmap!{};
    GeneratorCategoryIterator::new(contents.generators.categories.get(&category).unwrap_or(&empty))
  }

  /// Create a new iterator for the generators from a request
  pub fn new_from_request(request: &HttpRequest, category: GeneratorCategory) -> Self {
    let category: CoreGeneratorCategory = category.into();
    let empty = hashmap!{};
    GeneratorCategoryIterator::new(request.generators.categories.get(&category).unwrap_or(&empty))
  }

  /// Create a new iterator for the generators from a response
  pub fn new_from_response(response: &HttpResponse, category: GeneratorCategory) -> Self {
    let category: CoreGeneratorCategory = category.into();
    let empty = hashmap!{};
    GeneratorCategoryIterator::new(response.generators.categories.get(&category).unwrap_or(&empty))
  }

  fn next(&mut self) -> Option<&(DocPath, Generator)> {
    let value = self.generators.get(self.current_idx);
    self.current_idx += 1;
    value
  }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_generators_iter_delete(iter: *mut GeneratorCategoryIterator) {
        ptr::drop_raw(iter);
    }
}

/// A single key-value pair of a path and generator exported to the C-side.
#[derive(Debug)]
#[repr(C)]
pub struct GeneratorKeyValuePair {
  /// The generator path
  pub path: *const c_char,
  /// The generator
  pub generator: *const Generator
}

impl GeneratorKeyValuePair {
  fn new(
    key: &str,
    value: &Generator
  ) -> anyhow::Result<GeneratorKeyValuePair> {
    Ok(GeneratorKeyValuePair {
      path: string::to_c(key)? as *const c_char,
      generator: raw_to(value.clone()) as *const Generator
    })
  }
}

// Ensure that the owned values are freed when the pair is dropped.
impl Drop for GeneratorKeyValuePair {
  fn drop(&mut self) {
    string::pactffi_string_delete(self.path as *mut c_char);
    drop_raw(self.generator as *mut Generator);
  }
}

ffi_fn! {
    /// Get the next path and generator out of the iterator, if possible.
    ///
    /// The returned pointer must be deleted with `pactffi_generator_iter_pair_delete`.
    ///
    /// # Safety
    ///
    /// The underlying data is owned by the `GeneratorKeyValuePair`, so is always safe to use.
    ///
    /// # Error Handling
    ///
    /// If no further data is present, returns NULL.
    fn pactffi_generators_iter_next(iter: *mut GeneratorCategoryIterator) -> *const GeneratorKeyValuePair {
        let iter = as_mut!(iter);

        match iter.next() {
          Some((path, generator)) => {
            let pair = GeneratorKeyValuePair::new(&path.to_string(), generator)?;
            ptr::raw_to(pair)
          }
          None => {
            trace!("iter past the end of the generators");
            std::ptr::null_mut()
          }
        }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free a pair of key and value returned from `pactffi_generators_iter_next`.
    fn pactffi_generators_iter_pair_delete(pair: *const GeneratorKeyValuePair) {
        ptr::drop_raw(pair as *mut GeneratorKeyValuePair);
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;
  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::generators::Generator;
  use pact_models::prelude::Generator::{RandomInt, RandomString};

  use crate::models::generators::{
    pactffi_generator_generate_integer,
    pactffi_generator_generate_string,
    pactffi_generator_to_json
  };
  use crate::util::string;

  #[test]
  fn generate_string_test() {
    let generator = RandomString(4);

    let value = pactffi_generator_generate_string(&generator, std::ptr::null());
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

    let value = pactffi_generator_generate_integer(&generator, std::ptr::null());
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
}
