//! Adapts this crate's own native content matching/generation to the host-provided ("core")
//! capability shape (pact-plugins proposal 009), so plugins can delegate whole-content-type
//! matching and generation back to this framework instead of reimplementing it.
//!
//! Registration happens alongside the catalogue entries in
//! [`crate::matchingrules::configure_core_catalogue`], so an entry and its handler never drift
//! apart.
//!
//! It also registers this crate as `pact_models`' [`PluginSupport`] handler. That is the other
//! direction - `pact_models` reaching *out* to a plugin to apply a field-level rule or generator -
//! and it lives here because this is the crate that has both the plugin catalogue and the
//! bootstrap that runs before any matching happens.

use std::collections::HashMap;

use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use maplit::hashmap;
use tracing::debug;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::generators::{Generator, GeneratorTestMode};
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::plugins::{PluginSupport, set_plugin_support};
use pact_models::v4::http_parts::HttpResponse;
use pact_plugin_driver::core_capabilities::{
  CoreContentGenerator,
  CoreContentMatcher,
  register_core_content_generator,
  register_core_content_matcher
};
use pact_plugin_driver::field::{
  FieldContext,
  FieldValue,
  find_field_generator,
  find_field_matcher,
  TestMode as FieldTestMode
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

/// Resolves plugin-provided matching rules and generators on behalf of `pact_models`, which has no
/// visibility of the plugin catalogue - the driver depends on it, not the other way around. See
/// [`pact_models::plugins::PluginSupport`].
#[derive(Debug)]
struct DriverPluginSupport;

impl PluginSupport for DriverPluginSupport {
  fn config_key(&self, rule_name: &str) -> Option<String> {
    find_field_matcher(rule_name).ok()
      .and_then(|matcher| matcher.catalogue_entry.values.get("config-key").cloned())
  }

  fn generate(
    &self,
    name: &str,
    values: &serde_json::Value,
    example: &serde_json::Value,
    mode: Option<GeneratorTestMode>,
    path: &DocPath,
    context: &HashMap<&str, serde_json::Value>
  ) -> anyhow::Result<serde_json::Value> {
    let field_generator = find_field_generator(name)
      .map_err(|err| anyhow!("Could not apply the '{}' generator - {}", name, err))?;
    let generator = Generator::Plugin { name: name.to_string(), values: values.clone() };
    // The category only affects how a mismatch is reported, which generation has no equivalent of
    let field_context = FieldContext::new(path, "body")
      .with_test_context(context.iter().map(|(k, v)| (k.to_string(), v.clone())).collect());
    let test_mode = match mode {
      Some(GeneratorTestMode::Consumer) => FieldTestMode::Consumer,
      Some(GeneratorTestMode::Provider) => FieldTestMode::Provider,
      None => FieldTestMode::Unknown
    };

    debug!(%path, "Applying the '{}' generator provided by {}", name, field_generator.plugin_name());
    let generated = field_generator.generate_field_blocking(&generator,
      &FieldValue::Json(example.clone()), test_mode, &field_context)?;
    match generated {
      FieldValue::Json(value) => Ok(value),
      // A generator applied to a value in a document has to produce something the document can
      // hold, so binary is only usable here if it is text
      FieldValue::Binary(bytes) => match std::str::from_utf8(bytes.as_ref()) {
        Ok(text) => Ok(serde_json::Value::String(text.to_string())),
        Err(err) => Err(anyhow!("The '{}' generator returned {} bytes of binary data, which can \
          not be used as the value at {} - {}", name, bytes.len(), path, err))
      }
    }
  }
}

/// Registers this crate's native content matching/generation as host-provided ("core") capability
/// handlers, keyed to match the catalogue entries [`crate::matchingrules::configure_core_catalogue`]
/// registers, and registers this crate as `pact_models`' plugin support handler.
pub(crate) fn register_core_capabilities() {
  use std::sync::Arc;

  set_plugin_support(Arc::new(DriverPluginSupport));

  register_core_content_matcher("xml", Arc::new(XmlCoreContentMatcher));
  register_core_content_matcher("json", Arc::new(JsonCoreContentMatcher));
  register_core_content_matcher("text", Arc::new(TextCoreContentMatcher));
  register_core_content_matcher("multipart-form-data", Arc::new(MultipartCoreContentMatcher));

  register_core_content_generator("json", Arc::new(JsonCoreContentGenerator));
  register_core_content_generator("binary", Arc::new(BinaryCoreContentGenerator));
}
