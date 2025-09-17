//! `generators` module includes all the classes to deal with V3 format generators

use std::collections::HashMap;

use maplit::hashmap;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{
  apply_generators,
  GenerateValue,
  Generator,
  GeneratorCategory,
  GeneratorTestMode,
  NoopVariantMatcher,
  VariantMatcher
};
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::MatchingRuleCategory;
use pact_models::message::Message;
use pact_models::path_exp::DocPath;
use pact_models::plugins::PluginData;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;
use serde_json::{self, Value};
use tracing::{debug, error, trace};

use crate::{CoreMatchingContext, DiffConfig, MatchingContext};
use crate::json::compare_json;

pub mod bodies;


/// Apply the generators to the body, returning a new body
#[deprecated(note = "moved to the generators::bodies module", since = "0.12.16")]
pub async fn generators_process_body(
  mode: &GeneratorTestMode,
  body: &OptionalBody,
  content_type: Option<ContentType>,
  context: &HashMap<&str, Value>,
  generators: &HashMap<DocPath, Generator>,
  matcher: &(dyn VariantMatcher + Send + Sync)
) -> anyhow::Result<OptionalBody> {
  bodies::generators_process_body(mode, body, content_type, context, generators, matcher, &vec![], &hashmap!{}).await
}

pub(crate) fn find_matching_variant<T>(
  value: &T,
  variants: &[(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)],
  callback: &dyn Fn(&DocPath, &T, &(dyn MatchingContext + Send + Sync)) -> bool
) -> Option<(usize, HashMap<DocPath, Generator>)>
  where T: Clone + std::fmt::Debug {
  let result = variants.iter()
    .find(|(index, rules, _)| {
      debug!("find_matching_variant: Comparing variant {} with value '{:?}'", index, value);

      #[cfg(feature = "plugins")] #[cfg(not(target_family = "wasm"))]
      {
        let context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
          rules, &hashmap! {});
        let matches = callback(&DocPath::root(), value, &context);
        debug!("find_matching_variant: Comparing variant {} => {}", index, matches);
        matches
      }

      #[cfg(any(not(feature = "plugins"), target_family = "wasm"))]
      {
        let context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys, rules);
        let matches = callback(&DocPath::root(), value, &context);
        debug!("find_matching_variant: Comparing variant {} => {}", index, matches);
        matches
      }
    });
  debug!("find_matching_variant: result = {:?}", result);
  result.map(|(index, _, generators)| (*index, generators.clone()))
}

/// Default implementation of a VariantMatcher
#[derive(Debug, Clone)]
pub struct DefaultVariantMatcher;

impl VariantMatcher for DefaultVariantMatcher {
  fn find_matching_variant(
    &self,
    value: &Value,
    variants: &Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>
  ) -> Option<(usize, HashMap<DocPath, Generator>)> {
    let callback = |path: &DocPath, value: &Value, context: &(dyn MatchingContext + Send + Sync)| {
      compare_json(path, value, value, context).is_ok()
    };
    find_matching_variant(value, variants, &callback)
  }

  fn boxed(&self) -> Box<dyn VariantMatcher + Send + Sync> {
    Box::new(self.clone())
  }
}

/// Apply any generators to the synchronous message contents and then return a copy of the
/// request and response contents
pub async fn apply_generators_to_sync_message(
  message: &SynchronousMessage,
  mode: &GeneratorTestMode,
  context: &HashMap<&str, Value>,
  plugin_data: &Vec<PluginData>,
  interaction_data: &HashMap<String, HashMap<String, Value>>
) -> (MessageContents, Vec<MessageContents>) {
  let mut request = message.request.clone();
  let variant_matcher = NoopVariantMatcher {};
  let vm_boxed = variant_matcher.boxed();

  let generators = request.build_generators(&GeneratorCategory::METADATA);
  if !generators.is_empty() {
    debug!("Applying request metadata generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(k) = key.first_field() {
        let value = request.metadata.get(k).cloned().unwrap_or_default();
        if let Ok(v) = generator.generate_value(&value, context, &vm_boxed) {
          request.metadata.insert(k.to_string(), v);
        }
      }
    });
  }

  let generators = request.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && request.contents.is_present() {
    debug!("Applying request content generators...");
    match bodies::generators_process_body(mode, &request.contents, request.content_type(),
                                  context, &generators, &variant_matcher, plugin_data, interaction_data).await {
      Ok(contents) => request.contents = contents,
      Err(err) => error!("Failed to generate the message contents, will use the original: {}", err)
    }
  }

  let mut responses = message.response.clone();
  for response in responses.iter_mut() {
    let generators = response.build_generators(&GeneratorCategory::METADATA);
    if !generators.is_empty() {
      debug!("Applying response metadata generators...");
      apply_generators(mode, &generators, &mut |key, generator| {
        if let Some(k) = key.first_field() {
          let value = response.metadata.get(k).cloned().unwrap_or_default();
          if let Ok(v) = generator.generate_value(&value, context, &vm_boxed) {
            response.metadata.insert(k.to_string(), v);
          }
        }
      });
    }

    let generators = response.build_generators(&GeneratorCategory::BODY);
    if !generators.is_empty() && response.contents.is_present() {
      debug!("Applying response content generators...");
      match bodies::generators_process_body(mode, &response.contents, response.content_type(),
                                    context, &generators, &variant_matcher, plugin_data, interaction_data).await {
        Ok(contents) => response.contents = contents,
        Err(err) => error!("Failed to generate the message contents, will use the original: {}", err)
      }
    }
  }

  (request, responses)
}

/// Apply any generators to the asynchronous message contents and then return a copy of the contents
pub async fn apply_generators_to_async_message(
  message: &AsynchronousMessage,
  mode: &GeneratorTestMode,
  context: &HashMap<&str, Value>,
  plugin_data: &Vec<PluginData>,
  interaction_data: &HashMap<String, HashMap<String, Value>>
) -> MessageContents {
  let mut copy = message.contents.clone();
  let variant_matcher = NoopVariantMatcher {};
  let vm_boxed = variant_matcher.boxed();

  let generators = message.build_generators(&GeneratorCategory::METADATA);
  if !generators.is_empty() {
    debug!("Applying metadata generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(k) = key.first_field() {
        let value = message.contents.metadata.get(k).cloned().unwrap_or_default();
        if let Ok(v) = generator.generate_value(&value, context, &vm_boxed) {
          copy.metadata.insert(k.to_string(), v);
        }
      }
    });
  }

  let generators = message.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && message.contents.contents.is_present() {
    debug!("Applying content generators...");
    match bodies::generators_process_body(mode, &message.contents.contents, message.contents.content_type(),
                                  context, &generators, &variant_matcher, plugin_data, interaction_data).await {
      Ok(contents) => copy.contents = contents,
      Err(err) => error!("Failed to generate the message contents, will use the original: {}", err)
    }
  }

  copy
}

/// Generates the message by applying any defined generators to the contents and metadata
pub async fn generate_message(
  message: &Message,
  mode: &GeneratorTestMode,
  context: &HashMap<&str, Value>,
  plugin_data: &Vec<PluginData>,
  interaction_data: &HashMap<String, HashMap<String, Value>>
) -> Message {
  trace!(?message, ?mode, ?context, "generate_message");
  let mut message = message.clone();

  let generators = message.build_generators(&GeneratorCategory::METADATA);
  if !generators.is_empty() {
    debug!("Applying metadata generators...");
    apply_generators(mode, &generators, &mut |key, generator| {
      if let Some(header) = key.first_field() {
        if message.metadata.contains_key(header) {
          if let Ok(v) = generator.generate_value(&message.metadata.get(header).unwrap().clone(), context, &DefaultVariantMatcher.boxed()) {
            message.metadata.insert(header.to_string(), v);
          }
        } else {
          if let Ok(v) = generator.generate_value(&Value::Null, context, &DefaultVariantMatcher.boxed()) {
            message.metadata.insert(header.to_string(), v);
          }
        }
      }
    });
  }

  let generators = message.build_generators(&GeneratorCategory::BODY);
  if !generators.is_empty() && message.contents.is_present() {
    debug!("Applying body generators...");
    match  bodies::generators_process_body(mode, &message.contents, message.content_type(),
      context, &generators, &DefaultVariantMatcher{}, plugin_data, interaction_data).await {
      Ok(body) => message.contents = body,
      Err(err) => error!("Failed to generate the body, will use the original: {}", err)
    }
  }

  message
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::generators::{GenerateValue, Generator, VariantMatcher};
  use pact_models::matchingrules::MatchingRule;
  use pact_models::matchingrules_list;
  use pact_models::path_exp::DocPath;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::generators::DefaultVariantMatcher;

  #[test_log::test]
  fn array_contains_generator_test() {
    let generator = Generator::ArrayContains(vec![
      (0, matchingrules_list! {
        "body"; "$.href" => [ MatchingRule::Regex(".*(\\/orders\\/\\d+)$".into()) ]
      }, hashmap! {
        DocPath::new_unwrap("$.href") =>
          Generator::MockServerURL(
            "http://localhost:8080/orders/1234".into(),
            ".*(\\/orders\\/\\d+)$".into(),
          )
      }),
      (1, matchingrules_list! {
        "body"; "$.href" => [ MatchingRule::Regex(".*(\\/orders\\/\\d+)$".into()) ]
      }, hashmap! {
        DocPath::new_unwrap("$.href") =>
          Generator::MockServerURL(
            "http://localhost:8080/orders/1234".into(),
            ".*(\\/orders\\/\\d+)$".into(),
          )
      })
    ]);
    let value = json!([
      {
        "href": "http://localhost:9000/orders/1234",
        "method": "PUT",
        "name": "update"
      },
      {
        "href": "http://localhost:9000/orders/1234",
        "method": "DELETE",
        "name": "delete"
      }
    ]);
    let context = hashmap! {
      "mockServer" => json!({
        "url": "https://somewhere.else:1234/subpath"
      })
    };
    let generated = generator.generate_value(&value, &context, &DefaultVariantMatcher.boxed());
    expect!(generated.as_ref()).to(be_ok());
    let generated_value = generated.unwrap();
    assert_eq!(json!([
      {
        "href": "https://somewhere.else:1234/subpath/orders/1234",
        "method": "PUT",
        "name": "update"
      },
      {
        "href": "https://somewhere.else:1234/subpath/orders/1234",
        "method": "DELETE",
        "name": "delete"
      }
    ]), generated_value);
  }
}
