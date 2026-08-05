
use std::collections::HashMap;

use serde_json::Value;
use tracing::debug;
use anyhow::{anyhow, Result};

use crate::generators::{ContentTypeHandler, Generator, GeneratorTestMode, VariantMatcher, GenerateValue};
use crate::path_exp::DocPath;
use crate::bodies::OptionalBody;

pub type QueryParams = Vec<(String, String)>;

/// Implementation of a content type handler for FORM URLENCODED
pub struct FormUrlEncodedHandler {
  /// Query params to apply the generators to.
  pub params: QueryParams
}

impl ContentTypeHandler<String> for FormUrlEncodedHandler {
  fn process_body(
    &mut self,
    generators: &HashMap<DocPath, Generator>,
    mode: &GeneratorTestMode,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> Result<OptionalBody, String> {
    for (key, generator) in generators {
      if generator.corresponds_to_mode(mode) {
        debug!("Applying generator {:?} to key {}", generator, key);
        self.apply_key(key, generator, context, matcher);
      }
    };
    debug!("Query Params {:?}", self.params);
    match serde_urlencoded::to_string(self.params.clone()) {
      Ok(query_string) => Ok(OptionalBody::Present(query_string.into(), Some("application/x-www-form-urlencoded".into()), None)),
      Err(err) => Err(anyhow!("Failed to convert query params to query string: {}", err).to_string())
    }
  }

  fn apply_key(
    &mut self,
    key: &DocPath,
    generator: &Generator,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>,
  ) {
    let mut map: HashMap<String, usize> = HashMap::new();
    for (param_key, param_value) in self.params.iter_mut() {
      let index = map.entry(param_key.clone()).or_insert(0);
      if key.eq(&DocPath::root().join(param_key.clone())) || key.eq(&DocPath::root().join(param_key.clone()).join_index(*index)) {
        return match generator.generate_value(&*param_value, context, matcher) {
          Ok(new_value) => *param_value = new_value,
          Err(_) => ()
        }
      }
      *index += 1;
    }
  }
}


#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use test_log::test;
  use maplit::hashmap;

  use crate::generators::NoopVariantMatcher;

  use super::*;
  use super::Generator;

  #[test]
  fn applies_the_generator_to_a_valid_param() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.b"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&form_urlencoded_handler.params[1].1).to_not(be_equal_to("B"));
  }

  #[test]
  fn does_not_apply_the_generator_to_invalid_param() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.d"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn applies_the_generator_to_a_list_item() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B1".to_string()), ("b".to_string(), "B2".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.b[1]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&form_urlencoded_handler.params[2].1).to_not(be_equal_to("B2"));
  }

  #[test]
  fn does_not_apply_the_generator_when_index_is_not_in_list() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.b[3]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn does_not_apply_the_generator_when_not_a_list() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.a[0]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(&form_urlencoded_handler.params[0].1).to_not(be_equal_to("100"));
  }

  #[test]
  fn applies_the_generator_to_the_root() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::root(), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn does_not_apply_the_generator_to_long_path() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.a[1].b['2']"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn applies_the_generator_to_all_map_entries() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.*"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn applies_the_generator_to_all_list_items() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$[*]"), &Generator::RandomInt(0, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }

  #[test]
  fn applies_the_generator_to_long_path_with_wildcard() {
    let params = vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()));
    let mut form_urlencoded_handler = FormUrlEncodedHandler { params };

    form_urlencoded_handler.apply_key(&DocPath::new_unwrap("$.*[1].b[*]"), &Generator::RandomInt(3, 10), &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(form_urlencoded_handler.params).to(be_equal_to(vec!(("a".to_string(), "100".to_string()), ("b".to_string(), "B".to_string()), ("c".to_string(), "C".to_string()))));
  }
}
