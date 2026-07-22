//! Adapts this crate's own native content matching/generation to the host-provided ("core")
//! capability shape (pact-plugins proposal 009), so plugins can delegate whole-content-type
//! matching and generation back to this framework instead of reimplementing it.
//!
//! Registration happens alongside the catalogue entries in
//! [`crate::matchingrules::configure_core_catalogue`], so an entry and its handler never drift
//! apart. There is no field-level equivalent yet: that needs proposal 006's field-level operation
//! shape, which the plugin driver does not implement yet.

use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use maplit::hashmap;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::generators::GeneratorTestMode;
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::HttpResponse;
use pact_plugin_driver::core_capabilities::{
  CoreContentGenerator,
  CoreContentMatcher,
  register_core_content_generator,
  register_core_content_matcher
};
use pact_plugin_driver::proto::{
  body,
  Body,
  CompareContentsRequest,
  CompareContentsResponse,
  ContentMismatch,
  ContentMismatches,
  GenerateContentRequest,
  GenerateContentResponse,
  generate_content_request::TestMode
};
use pact_plugin_driver::utils::proto_struct_to_json;

use crate::{CoreMatchingContext, DiffConfig, Mismatch};
use crate::generators::DefaultVariantMatcher;
use crate::generators::bodies::generators_process_body;

fn to_http_part(body: &Option<Body>) -> HttpResponse {
  let body = match body {
    Some(body) => {
      let contents = body.content.as_ref().cloned().unwrap_or_default();
      if contents.is_empty() {
        OptionalBody::Empty
      } else {
        let content_type = ContentType::parse(body.content_type.as_str()).ok();
        OptionalBody::Present(Bytes::from(contents), content_type, Some(match body.content_type_hint() {
          body::ContentTypeHint::Text => ContentTypeHint::TEXT,
          body::ContentTypeHint::Binary => ContentTypeHint::BINARY,
          body::ContentTypeHint::Default => ContentTypeHint::DEFAULT
        }))
      }
    },
    None => OptionalBody::Missing
  };
  HttpResponse { body, ..HttpResponse::default() }
}

fn to_matching_context(request: &CompareContentsRequest) -> CoreMatchingContext {
  let mut category = MatchingRuleCategory::empty(Category::BODY);
  for (path, rules) in &request.rules {
    if let Ok(doc_path) = DocPath::new(path.as_str()) {
      for rule in &rules.rule {
        let values = rule.values.as_ref().map(proto_struct_to_json).unwrap_or_default();
        if let Ok(matching_rule) = MatchingRule::create(rule.r#type.as_str(), &values) {
          category.add_rule(doc_path.clone(), matching_rule, RuleLogic::And);
        }
      }
    }
  }
  let diff_config = if request.allow_unexpected_keys {
    DiffConfig::AllowUnexpectedKeys
  } else {
    DiffConfig::NoUnexpectedKeys
  };
  // The plugin-configuration map is keyed by plugin name so a core matcher can recurse into
  // another plugin's own matcher for embedded content; the request only carries a single (unnamed)
  // config, so there is no name to key it under here. Nested plugin delegation from within core
  // matching is out of scope until proposal 006's field-level shape exists.
  CoreMatchingContext::new(diff_config, &category, &hashmap!{})
}

fn to_content_mismatch(mismatch: &Mismatch) -> Option<ContentMismatch> {
  match mismatch {
    Mismatch::BodyMismatch { path, expected, actual, mismatch } => Some(ContentMismatch {
      expected: Some(expected.clone().map(|b| b.to_vec()).unwrap_or_default()),
      actual: Some(actual.clone().map(|b| b.to_vec()).unwrap_or_default()),
      mismatch: mismatch.clone(),
      path: path.clone(),
      diff: String::default(),
      mismatch_type: "body".to_string()
    }),
    _ => None
  }
}

fn to_response(mismatches: Vec<Mismatch>) -> CompareContentsResponse {
  let grouped = crate::group_by(mismatches, |m| match m {
    Mismatch::BodyMismatch { path, .. } => path.clone(),
    _ => String::default()
  });
  CompareContentsResponse {
    error: String::default(),
    type_mismatch: None,
    results: grouped.into_iter()
      .map(|(path, mismatches)| (path, ContentMismatches {
        mismatches: mismatches.iter().filter_map(to_content_mismatch).collect()
      }))
      .collect()
  }
}

macro_rules! core_content_matcher {
  ($name:ident, $match_fn:expr) => {
    #[derive(Debug)]
    struct $name;

    #[async_trait]
    impl CoreContentMatcher for $name {
      async fn compare_contents(&self, request: CompareContentsRequest) -> anyhow::Result<CompareContentsResponse> {
        let expected = to_http_part(&request.expected);
        let actual = to_http_part(&request.actual);
        let context = to_matching_context(&request);
        let mismatches = match $match_fn(&expected, &actual, &context) {
          Ok(()) => vec![],
          Err(m) => m
        };
        Ok(to_response(mismatches))
      }
    }
  }
}

core_content_matcher!(JsonCoreContentMatcher, crate::json::match_json);
core_content_matcher!(TextCoreContentMatcher, (|expected: &HttpResponse, actual: &HttpResponse, context: &CoreMatchingContext|
  crate::match_text(&expected.body.value(), &actual.body.value(), context)));
core_content_matcher!(MultipartCoreContentMatcher, crate::binary_utils::match_mime_multipart);

#[derive(Debug)]
struct XmlCoreContentMatcher;

#[async_trait]
impl CoreContentMatcher for XmlCoreContentMatcher {
  async fn compare_contents(&self, request: CompareContentsRequest) -> anyhow::Result<CompareContentsResponse> {
    let expected = to_http_part(&request.expected);
    let actual = to_http_part(&request.actual);
    let context = to_matching_context(&request);
    #[cfg(feature = "xml")]
    let result = crate::xml::match_xml(&expected, &actual, &context);
    #[cfg(not(feature = "xml"))]
    let result = {
      tracing::warn!("Matching XML bodies requires the xml feature to be enabled");
      crate::match_text(&expected.body.value(), &actual.body.value(), &context)
    };
    let mismatches = match result {
      Ok(()) => vec![],
      Err(m) => m
    };
    Ok(to_response(mismatches))
  }
}

#[derive(Debug)]
struct JsonCoreContentGenerator;

#[async_trait]
impl CoreContentGenerator for JsonCoreContentGenerator {
  async fn generate_content(&self, request: GenerateContentRequest) -> anyhow::Result<GenerateContentResponse> {
    let body = to_http_part(&request.contents).body;
    let content_type = body.content_type().unwrap_or_default();
    let mode = match request.test_mode() {
      TestMode::Consumer => GeneratorTestMode::Consumer,
      _ => GeneratorTestMode::Provider
    };
    let generators = request.generators.iter()
      .filter_map(|(path, generator)| {
        let values = generator.values.as_ref().map(proto_struct_to_json).unwrap_or_default();
        let g = pact_models::generators::Generator::from_map(generator.r#type.as_str(), values.as_object()?)?;
        let doc_path = DocPath::new(path.as_str()).unwrap_or_else(|_| DocPath::root());
        Some((doc_path, g))
      })
      .collect::<HashMap<_, _>>();
    let test_context_owned = request.test_context.as_ref().map(proto_struct_to_json)
      .and_then(|v| v.as_object().cloned())
      .unwrap_or_default();
    let test_context: HashMap<&str, serde_json::Value> = test_context_owned.iter()
      .map(|(k, v)| (k.as_str(), v.clone()))
      .collect();

    let generated = generators_process_body(&mode, &body, Some(content_type.clone()), &test_context,
      &generators, &DefaultVariantMatcher, &vec![], &hashmap!{}).await?;

    Ok(GenerateContentResponse {
      contents: Some(Body {
        content_type: content_type.to_string(),
        content: generated.value().map(|b| b.to_vec()),
        content_type_hint: body::ContentTypeHint::Default as i32
      })
    })
  }
}

/// No native generation logic exists for arbitrary binary content in this crate (the same is true
/// internally: [`generators_process_body`] does not apply generators to non-JSON/XML/form-urlencoded
/// bodies either), so this is a documented no-op passthrough rather than a fabricated behaviour.
#[derive(Debug)]
struct BinaryCoreContentGenerator;

#[async_trait]
impl CoreContentGenerator for BinaryCoreContentGenerator {
  async fn generate_content(&self, request: GenerateContentRequest) -> anyhow::Result<GenerateContentResponse> {
    Ok(GenerateContentResponse { contents: request.contents })
  }
}

/// Registers this crate's native content matching/generation as host-provided ("core") capability
/// handlers, keyed to match the catalogue entries [`crate::matchingrules::configure_core_catalogue`]
/// registers.
pub(crate) fn register_core_capabilities() {
  use std::sync::Arc;

  register_core_content_matcher("xml", Arc::new(XmlCoreContentMatcher));
  register_core_content_matcher("json", Arc::new(JsonCoreContentMatcher));
  register_core_content_matcher("text", Arc::new(TextCoreContentMatcher));
  register_core_content_matcher("multipart-form-data", Arc::new(MultipartCoreContentMatcher));

  register_core_content_generator("json", Arc::new(JsonCoreContentGenerator));
  register_core_content_generator("binary", Arc::new(BinaryCoreContentGenerator));
}
