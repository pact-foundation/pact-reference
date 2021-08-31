//! `generators` module includes all the classes to deal with V3 format generators

use std::collections::HashMap;

use log::*;
use serde_json::{self, Value};
use sxd_document::dom::Document;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{ContentTypeHandler, GenerateValue, Generator, GeneratorTestMode, JsonHandler, VariantMatcher};
use pact_models::matchingrules::MatchingRuleCategory;
use pact_models::path_exp::DocPath;
use pact_models::xml_utils::parse_bytes;

use crate::{DiffConfig, MatchingContext};
use crate::json::compare;

/// Implementation of a content type handler for XML (currently unimplemented).
pub struct XmlHandler<'a> {
  /// XML document to apply the generators to.
  pub value: Document<'a>
}

impl <'a> ContentTypeHandler<Document<'a>> for XmlHandler<'a> {
  fn process_body(
    &mut self,
    _generators: &HashMap<DocPath, Generator>,
    _mode: &GeneratorTestMode,
    _context: &HashMap<&str, Value>,
    _matcher: &Box<dyn VariantMatcher>
  ) -> Result<OptionalBody, String> {
    error!("UNIMPLEMENTED: Generators are not currently supported with XML");
    Err("Generators are not supported with XML".to_string())
  }

  fn apply_key(
    &mut self,
    _key: &DocPath,
    _generator: &dyn GenerateValue<Document<'a>>,
    _context: &HashMap<&str, Value>,
    _matcher: &Box<dyn VariantMatcher>
  ) {
    error!("UNIMPLEMENTED: Generators are not currently supported with XML");
  }
}

/// Apply the generators to the body, returning a new body
pub fn generators_process_body(
  mode: &GeneratorTestMode,
  body: &OptionalBody,
  content_type: Option<ContentType>,
  context: &HashMap<&str, Value>,
  generators: &HashMap<DocPath, Generator>,
  matcher: &Box<dyn VariantMatcher>
) -> OptionalBody {
  match content_type {
    Some(content_type) => if content_type.is_json() {
      trace!("apply_body_generators: JSON content type");
      let result: Result<Value, serde_json::Error> = serde_json::from_slice(&body.value().unwrap_or_default());
      match result {
        Ok(val) => {
          let mut handler = JsonHandler { value: val };
          handler.process_body(&generators, mode, context, matcher).unwrap_or_else(|err| {
            error!("Failed to generate the body: {}", err);
            body.clone()
          })
        },
        Err(err) => {
          error!("Failed to parse the body, so not applying any generators: {}", err);
          body.clone()
        }
      }
    } else if content_type.is_xml() {
      trace!("apply_body_generators: XML content type");
      match parse_bytes(&body.value().unwrap_or_default()) {
        Ok(val) => {
          let mut handler = XmlHandler { value: val.as_document() };
          handler.process_body(&generators, mode, context, matcher).unwrap_or_else(|err| {
            error!("Failed to generate the body: {}", err);
            body.clone()
          })
        },
        Err(err) => {
          error!("Failed to parse the body, so not applying any generators: {}", err);
          body.clone()
        }
      }
    } else {
      warn!("Unsupported content type {} - Generators only support JSON and XML", content_type);
      body.clone()
    },
    _ => body.clone()
  }
}

pub(crate) fn find_matching_variant<T>(
  value: &T,
  variants: &Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>,
  callback: &dyn Fn(&Vec<&str>, &T, &MatchingContext) -> bool
) -> Option<(usize, HashMap<DocPath, Generator>)>
  where T: Clone + std::fmt::Debug {
  let result = variants.iter()
    .find(|(index, rules, _)| {
      trace!("find_matching_variant: Comparing variant {} with value '{:?}'", index, value);
      let context = MatchingContext::new(DiffConfig::NoUnexpectedKeys, rules);
      let matches = callback(&vec!["$"], value, &context);
      trace!("find_matching_variant: Comparing variant {} => {}", index, matches);
      matches
    });
  trace!("find_matching_variant: result = {:?}", result);
  result.map(|(index, _, generators)| (*index, generators.clone()))
}

#[derive(Debug, Clone)]
pub(crate) struct DefaultVariantMatcher;

impl VariantMatcher for DefaultVariantMatcher {
  fn find_matching_variant(
    &self,
    value: &Value,
    variants: &Vec<(usize, MatchingRuleCategory, HashMap<DocPath, Generator>)>
  ) -> Option<(usize, HashMap<DocPath, Generator>)> {
    let callback = |path: &Vec<&str>, value: &Value, context: &MatchingContext| {
      compare(path, value, value, context).is_ok()
    };
    find_matching_variant(value, variants, &callback)
  }

  fn boxed(&self) -> Box<dyn VariantMatcher> {
    Box::new(self.clone())
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;
  use pretty_assertions::assert_eq;

  use pact_models::generators::{GenerateValue, Generator, VariantMatcher};
  use pact_models::matchingrules::MatchingRule;
  use pact_models::matchingrules_list;
  use pact_models::path_exp::DocPath;

  use crate::generators::DefaultVariantMatcher;

  #[test_env_log::test]
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
        "href": "https://somewhere.else:1234/subpath"
      })
    };
    let generated = generator.generate_value(&value, &context, &DefaultVariantMatcher.boxed());
    expect!(generated.as_ref()).to(be_ok());
    let generated_value = generated.unwrap();
    assert_eq!(generated_value, json!([
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
    ]));
  }
}
