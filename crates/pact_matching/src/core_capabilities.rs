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
use pact_models::generators::{GenerateValue, Generator, GeneratorTestMode, VariantMatcher};
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::plugins::{PluginSupport, set_plugin_support};
use pact_models::v4::http_parts::HttpResponse;
use pact_plugin_driver::core_capabilities::{
  CoreContentGenerator,
  CoreContentMatcher,
  CoreFieldGenerator,
  CoreFieldMatcher,
  register_core_content_generator,
  register_core_content_matcher,
  register_core_field_generator,
  register_core_field_matcher
};
use pact_plugin_driver::field::{
  FieldContext,
  FieldValue,
  find_field_generator,
  find_field_matcher,
  TestMode as FieldTestMode
};
use pact_plugin_driver::proto_v2::{
  ContentMismatch as FieldContentMismatch,
  GenerateFieldRequest,
  GenerateFieldResponse,
  generate_content_request::TestMode as ProtoTestMode,
  MatchFieldRequest,
  MatchFieldResponse
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
use crate::matchingrules::DoMatch;
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

/// The matching rules this crate can apply to a single value, and so registers a field-level
/// handler for. Every other rule it implements is collection-wide (see [`COLLECTION_RULES`]).
const FIELD_RULES: [&str; 16] = ["equality", "regex", "type", "include", "number", "integer",
  "decimal", "boolean", "null", "date", "time", "datetime", "content-type", "not-empty", "semver",
  "status-code"];

/// The matching rules that decide *which* values they apply to, and so can not be handed one value
/// at a time - see proposal 006's non-goals. They get a handler anyway, so a plugin naming one is
/// told why it can not have it rather than being told nothing is registered.
const COLLECTION_RULES: [&str; 7] = ["min-type", "max-type", "min-max-type", "values",
  "array-contains", "each-key", "each-value"];

/// The generators this crate can apply to a single value. `ProviderState` and `MockServerURL` are
/// included: both read host state, but it reaches them through the request's test context, which
/// is what proposal 006 requires of any generator.
const FIELD_GENERATORS: [&str; 12] = ["RandomInt", "RandomDecimal", "RandomHexadecimal",
  "RandomString", "RandomBoolean", "Regex", "Uuid", "Date", "Time", "DateTime", "ProviderState",
  "MockServerURL"];

/// Generators that build a value inside a structure their caller owns, so there is no single value
/// to generate. See [`COLLECTION_RULES`].
const COLLECTION_GENERATORS: [&str; 1] = ["ArrayContains"];

/// Rule and generator configuration crosses the plugin interface as a `google.protobuf.Struct`,
/// which has one number type - a double - so a `min` of 2 arrives as `2.0`. `pact_models` reads
/// those attributes with `as_i64`/`as_u64`, which reject a float, and silently falls back to the
/// attribute's default: a `RandomInt(5, 5)` from a plugin would generate from `0..10` instead.
/// Whole floats are put back to integers here, on the way in, so a configuration value means what
/// the plugin sent.
///
/// This is the configuration-value counterpart of what `FieldValue`'s per-type arms do for the
/// value being matched, which does not go through a `Struct` for exactly this reason.
fn whole_floats_to_integers(value: serde_json::Value) -> serde_json::Value {
  match value {
    serde_json::Value::Number(number) => match number.as_f64() {
      Some(float) if float.fract() == 0.0 && !number.is_i64() && !number.is_u64() =>
        serde_json::Value::Number((float as i64).into()),
      _ => serde_json::Value::Number(number)
    },
    serde_json::Value::Array(values) => serde_json::Value::Array(
      values.into_iter().map(whole_floats_to_integers).collect()
    ),
    serde_json::Value::Object(values) => serde_json::Value::Object(
      values.into_iter().map(|(k, v)| (k, whole_floats_to_integers(v))).collect()
    ),
    value => value
  }
}

/// The rule named by a field-level request: the rule it carries, or - if it carries none - the
/// catalogue key it was dispatched under, so `host_match_field("not-empty", ...)` works without
/// having to restate the rule.
fn rule_from_request(request: &MatchFieldRequest) -> anyhow::Result<MatchingRule> {
  let name = request.rule.as_ref()
    .map(|rule| rule.r#type.clone())
    .filter(|name| !name.is_empty())
    .unwrap_or_else(|| request.key.clone());
  let values = request.rule.as_ref()
    .and_then(|rule| rule.values.as_ref())
    .map(proto_struct_to_json)
    .map(whole_floats_to_integers)
    .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

  let rule = MatchingRule::create(name.as_str(), &values)
    .map_err(|err| anyhow!("'{}' is not a valid matching rule - {}", name, err))?;
  // An unrecognised name parses into a plugin rule, which would send the call straight back out to
  // a plugin. A core handler answers for core rules only.
  if let MatchingRule::Plugin { .. } = rule {
    return Err(anyhow!("'{}' is not one of the matching rules provided by this framework", name));
  }
  Ok(rule)
}

/// The generator named by a field-level request. See [`rule_from_request`].
fn generator_from_request(request: &GenerateFieldRequest) -> anyhow::Result<Generator> {
  let name = request.generator.as_ref()
    .map(|generator| generator.r#type.clone())
    .filter(|name| !name.is_empty())
    .unwrap_or_else(|| request.key.clone());
  let values = request.generator.as_ref()
    .and_then(|generator| generator.values.as_ref())
    .map(proto_struct_to_json)
    .map(whole_floats_to_integers)
    .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
  let values = values.as_object().cloned().unwrap_or_default();

  match Generator::from_map(name.as_str(), &values) {
    Some(Generator::Plugin { .. }) | None =>
      Err(anyhow!("'{}' is not one of the generators provided by this framework", name)),
    Some(generator) => Ok(generator)
  }
}

/// Apply a matching rule to one value, picking the comparison for the types the two sides actually
/// have. Binary values are compared as bytes rather than being stringified, which is the whole
/// reason `FieldValue` has a binary arm.
fn apply_field_rule(
  rule: &MatchingRule,
  expected: &FieldValue,
  actual: &FieldValue
) -> anyhow::Result<()> {
  match (expected, actual) {
    (FieldValue::Json(expected), FieldValue::Json(actual)) =>
      rule.match_value(expected, actual, false, true),
    (FieldValue::Binary(expected), FieldValue::Binary(actual)) =>
      rule.match_value(expected, actual, false, true),
    // Text on one side and bytes on the other is a legitimate pairing (a plugin that decoded one
    // of them), so compare as bytes rather than refusing
    (FieldValue::Json(serde_json::Value::String(expected)), FieldValue::Binary(actual)) =>
      rule.match_value(&Bytes::from(expected.clone().into_bytes()), actual, false, true),
    (FieldValue::Binary(expected), FieldValue::Json(serde_json::Value::String(actual))) =>
      rule.match_value(expected, &Bytes::from(actual.clone().into_bytes()), false, true),
    (expected, actual) => Err(anyhow!("Can not apply the '{}' matching rule to {} against {}",
      rule.name(), describe_field_value(expected), describe_field_value(actual)))
  }
}

fn describe_field_value(value: &FieldValue) -> String {
  match value {
    FieldValue::Json(value) => format!("a JSON value ({})", value),
    FieldValue::Binary(bytes) => format!("{} bytes of binary data", bytes.len())
  }
}

/// The bytes a mismatch reports for a value. `ContentMismatch` carries bytes so that a binary value
/// survives being reported, which means a JSON value has to be rendered into some form here.
fn field_value_bytes(value: &FieldValue) -> Vec<u8> {
  match value {
    FieldValue::Json(serde_json::Value::String(value)) => value.clone().into_bytes(),
    FieldValue::Json(value) => value.to_string().into_bytes(),
    FieldValue::Binary(bytes) => bytes.to_vec()
  }
}

/// Applies one of this crate's standard matching rules to a single value, on behalf of a plugin
/// that does not want to reimplement it (proposal 009). Registered against every key in
/// [`FIELD_RULES`] - the rule to apply comes from the request, so one handler serves them all.
#[derive(Debug)]
struct CoreFieldRuleMatcher;

#[async_trait]
impl CoreFieldMatcher for CoreFieldRuleMatcher {
  async fn match_field(&self, request: MatchFieldRequest) -> anyhow::Result<MatchFieldResponse> {
    let rule = match rule_from_request(&request) {
      Ok(rule) => rule,
      Err(err) => return Ok(MatchFieldResponse { error: err.to_string(), mismatches: vec![] })
    };
    let expected = request.expected.as_ref()
      .map(FieldValue::from_proto)
      .unwrap_or(FieldValue::Json(serde_json::Value::Null));
    let actual = request.actual.as_ref()
      .map(FieldValue::from_proto)
      .unwrap_or(FieldValue::Json(serde_json::Value::Null));

    debug!(path = request.path.as_str(), "Applying the core '{}' matching rule to a single value",
      rule.name());
    match apply_field_rule(&rule, &expected, &actual) {
      Ok(()) => Ok(MatchFieldResponse::default()),
      Err(err) => Ok(MatchFieldResponse {
        error: String::default(),
        mismatches: vec![FieldContentMismatch {
          expected: Some(field_value_bytes(&expected)),
          actual: Some(field_value_bytes(&actual)),
          mismatch: err.to_string(),
          path: request.path.clone(),
          diff: String::default(),
          mismatch_type: request.mismatch_type.clone()
        }]
      })
    }
  }
}

/// Answers for the collection-wide rules: they are real capabilities this framework has, so they
/// are in the catalogue, but the matching engine has to interpret them itself to know which values
/// they cover. See [`COLLECTION_RULES`].
#[derive(Debug)]
struct CollectionRuleMatcher;

#[async_trait]
impl CoreFieldMatcher for CollectionRuleMatcher {
  async fn match_field(&self, request: MatchFieldRequest) -> anyhow::Result<MatchFieldResponse> {
    Ok(MatchFieldResponse {
      error: format!("The '{}' matching rule applies to a collection as a whole, not to a single \
        value, so it can not be applied through MatchField. Match the collection with a content \
        matcher instead.", request.key),
      mismatches: vec![]
    })
  }
}

/// Applies one of this crate's standard generators to a single value, on behalf of a plugin
/// (proposal 009). Registered against every key in [`FIELD_GENERATORS`].
#[derive(Debug)]
struct CoreFieldValueGenerator;

#[async_trait]
impl CoreFieldGenerator for CoreFieldValueGenerator {
  async fn generate_field(&self, request: GenerateFieldRequest) -> anyhow::Result<GenerateFieldResponse> {
    let generator = match generator_from_request(&request) {
      Ok(generator) => generator,
      Err(err) => return Ok(GenerateFieldResponse { error: err.to_string(), value: None })
    };
    let example = request.example_value.as_ref()
      .map(FieldValue::from_proto)
      .unwrap_or(FieldValue::Json(serde_json::Value::Null));
    // A generator that does not apply on this side of the test leaves the example value alone,
    // the same as it would when applied to a body. An unknown mode applies it rather than guessing
    // a side: guessing wrong turns `MockServerURL` into a silent no-op in a consumer test.
    let mode = match ProtoTestMode::try_from(request.test_mode) {
      Ok(ProtoTestMode::Consumer) => Some(GeneratorTestMode::Consumer),
      Ok(ProtoTestMode::Provider) => Some(GeneratorTestMode::Provider),
      _ => None
    };
    if let Some(mode) = &mode {
      if !generator.corresponds_to_mode(mode) {
        return Ok(GenerateFieldResponse { error: String::default(), value: Some(example.to_proto()) });
      }
    }

    let context_values = request.test_context.as_ref()
      .map(proto_struct_to_json)
      .and_then(|value| value.as_object().cloned())
      .unwrap_or_default();
    let context: HashMap<&str, serde_json::Value> = context_values.iter()
      .map(|(key, value)| (key.as_str(), value.clone()))
      .collect();

    debug!(path = request.path.as_str(), "Applying the core '{}' generator to a single value",
      generator.name());
    let generated = match &example {
      FieldValue::Json(value) => generator
        .generate_value(value, &context, &DefaultVariantMatcher.boxed())
        .map(FieldValue::Json),
      // A generator produces a JSON-ish value, so binary is only usable here if it is text
      FieldValue::Binary(bytes) => match std::str::from_utf8(bytes.as_ref()) {
        Ok(text) => generator
          .generate_value(&text.to_string(), &context, &DefaultVariantMatcher.boxed())
          .map(|generated| FieldValue::Binary(Bytes::from(generated.into_bytes()))),
        Err(err) => Err(anyhow!("Can not apply the '{}' generator to {} bytes of binary data - {}",
          generator.name(), bytes.len(), err))
      }
    };

    match generated {
      Ok(value) => Ok(GenerateFieldResponse { error: String::default(), value: Some(value.to_proto()) }),
      Err(err) => Ok(GenerateFieldResponse { error: err.to_string(), value: None })
    }
  }
}

/// Answers for generators that build values inside a structure their caller owns. See
/// [`CollectionRuleMatcher`].
#[derive(Debug)]
struct CollectionValueGenerator;

#[async_trait]
impl CoreFieldGenerator for CollectionValueGenerator {
  async fn generate_field(&self, request: GenerateFieldRequest) -> anyhow::Result<GenerateFieldResponse> {
    Ok(GenerateFieldResponse {
      error: format!("The '{}' generator generates values within a collection, not a single value, \
        so it can not be applied through GenerateField.", request.key),
      value: None
    })
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

  for rule in FIELD_RULES {
    register_core_field_matcher(rule, Arc::new(CoreFieldRuleMatcher));
  }
  for rule in COLLECTION_RULES {
    register_core_field_matcher(rule, Arc::new(CollectionRuleMatcher));
  }

  for generator in FIELD_GENERATORS {
    register_core_field_generator(generator, Arc::new(CoreFieldValueGenerator));
  }
  for generator in COLLECTION_GENERATORS {
    register_core_field_generator(generator, Arc::new(CollectionValueGenerator));
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use maplit::hashmap;

  use expectest::prelude::*;
  use pact_plugin_driver::proto_v2::{Generator as ProtoGenerator, MatchingRule as ProtoMatchingRule};
  use pact_plugin_driver::utils::to_proto_struct;
  use serde_json::json;

  use super::*;

  fn match_request(key: &str, rule: Option<ProtoMatchingRule>, expected: FieldValue, actual: FieldValue) -> MatchFieldRequest {
    MatchFieldRequest {
      key: key.to_string(),
      rule,
      path: "$.one".to_string(),
      mismatch_type: "body".to_string(),
      expected: Some(expected.to_proto()),
      actual: Some(actual.to_proto()),
      .. MatchFieldRequest::default()
    }
  }

  fn proto_rule(name: &str, values: serde_json::Value) -> ProtoMatchingRule {
    ProtoMatchingRule {
      r#type: name.to_string(),
      values: Some(to_proto_struct(&values.as_object().unwrap().iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()))
    }
  }

  #[tokio::test]
  async fn applies_a_core_rule_to_a_single_value() {
    let request = match_request("regex", Some(proto_rule("regex", json!({ "regex": "\\d+" }))),
      FieldValue::Json(json!("100")), FieldValue::Json(json!("200")));

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(response.mismatches.iter()).to(be_empty());
  }

  #[tokio::test]
  async fn reports_a_mismatch_against_the_path_from_the_request() {
    let request = match_request("regex", Some(proto_rule("regex", json!({ "regex": "\\d+" }))),
      FieldValue::Json(json!("100")), FieldValue::Json(json!("not a number")));

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(response.mismatches.len()).to(be_equal_to(1));
    let mismatch = &response.mismatches[0];
    expect!(mismatch.path.as_str()).to(be_equal_to("$.one"));
    expect!(mismatch.mismatch_type.as_str()).to(be_equal_to("body"));
    expect!(mismatch.actual.clone().unwrap()).to(be_equal_to("not a number".as_bytes().to_vec()));
    expect!(mismatch.mismatch.contains("to match")).to(be_true());
  }

  /// The distinction FieldValue's per-type arms exist for: an integer actual against a decimal
  /// expected is a type mismatch, and would not be if both arrived as a JSON number
  #[tokio::test]
  async fn keeps_the_numeric_type_of_a_value() {
    let integer = CoreFieldRuleMatcher.match_field(match_request("integer",
      Some(proto_rule("integer", json!({}))), FieldValue::Json(json!(100)), FieldValue::Json(json!(200))))
      .await.unwrap();
    let decimal = CoreFieldRuleMatcher.match_field(match_request("decimal",
      Some(proto_rule("decimal", json!({}))), FieldValue::Json(json!(100.0)), FieldValue::Json(json!(200))))
      .await.unwrap();

    expect!(integer.mismatches.iter()).to(be_empty());
    expect!(decimal.mismatches.len()).to(be_equal_to(1));
  }

  #[tokio::test]
  async fn falls_back_to_the_catalogue_key_when_the_request_carries_no_rule() {
    let request = match_request("not-empty", None,
      FieldValue::Json(json!("a")), FieldValue::Json(json!("")));

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(response.mismatches.len()).to(be_equal_to(1));
  }

  #[tokio::test]
  async fn compares_a_binary_value_as_bytes() {
    let expected = FieldValue::Binary(Bytes::from_static(&[0x00, 0xfe, 0x01]));
    let actual = FieldValue::Binary(Bytes::from_static(&[0x00, 0xfe, 0x02]));
    let request = match_request("equality", Some(proto_rule("equality", json!({}))),
      expected, actual);

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.mismatches.len()).to(be_equal_to(1));
    expect!(response.mismatches[0].actual.clone().unwrap()).to(be_equal_to(vec![0x00, 0xfe, 0x02]));
  }

  #[tokio::test]
  async fn a_rule_this_framework_does_not_provide_is_an_error_not_a_call_back_out_to_a_plugin() {
    let request = match_request("type", Some(proto_rule("creditcard", json!({ "brand": "visa" }))),
      FieldValue::Json(json!("4111111111111111")), FieldValue::Json(json!("4111111111111111")));

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error.contains("'creditcard' is not one of the matching rules provided by this framework"))
      .to(be_true());
    expect!(response.mismatches.iter()).to(be_empty());
  }

  #[tokio::test]
  async fn a_malformed_rule_is_reported_as_an_error() {
    let request = match_request("regex", Some(proto_rule("regex", json!({}))),
      FieldValue::Json(json!("100")), FieldValue::Json(json!("200")));

    let response = CoreFieldRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error.contains("is not a valid matching rule")).to(be_true());
  }

  #[tokio::test]
  async fn a_collection_wide_rule_says_why_it_can_not_be_applied() {
    let request = match_request("each-value", None,
      FieldValue::Json(json!(["a"])), FieldValue::Json(json!(["b"])));

    let response = CollectionRuleMatcher.match_field(request).await.unwrap();

    expect!(response.error.contains("applies to a collection as a whole")).to(be_true());
    expect!(response.mismatches.iter()).to(be_empty());
  }

  fn generate_request(key: &str, generator: Option<ProtoGenerator>, example: FieldValue, mode: ProtoTestMode) -> GenerateFieldRequest {
    GenerateFieldRequest {
      key: key.to_string(),
      generator,
      path: "$.one".to_string(),
      example_value: Some(example.to_proto()),
      test_mode: mode as i32,
      .. GenerateFieldRequest::default()
    }
  }

  fn proto_generator(name: &str, values: serde_json::Value) -> ProtoGenerator {
    ProtoGenerator {
      r#type: name.to_string(),
      values: Some(to_proto_struct(&values.as_object().unwrap().iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()))
    }
  }

  #[tokio::test]
  async fn generates_a_single_value() {
    let request = generate_request("RandomInt",
      Some(proto_generator("RandomInt", json!({ "min": 5, "max": 5 }))),
      FieldValue::Json(json!(0)), ProtoTestMode::Consumer);

    let response = CoreFieldValueGenerator.generate_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(FieldValue::from_proto(&response.value.unwrap())).to(be_equal_to(FieldValue::Json(json!(5))));
  }

  #[tokio::test]
  async fn a_generator_for_the_other_side_of_the_test_leaves_the_example_value_alone() {
    // MockServerURL only applies on the consumer side
    let request = generate_request("MockServerURL",
      Some(proto_generator("MockServerURL", json!({ "example": "http://localhost:1234/a", "regex": ".*(/a)" }))),
      FieldValue::Json(json!("http://localhost:1234/a")), ProtoTestMode::Provider);

    let response = CoreFieldValueGenerator.generate_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(FieldValue::from_proto(&response.value.unwrap()))
      .to(be_equal_to(FieldValue::Json(json!("http://localhost:1234/a"))));
  }

  /// An unknown mode is not a guess at a side: a consumer-only generator still applies, because
  /// guessing "provider" would turn it into a silent no-op. The value it needs reaches it through
  /// the request's test context, which is the only way proposal 006 lets a generator see host
  /// state.
  #[tokio::test]
  async fn a_generator_runs_when_the_side_of_the_test_is_not_known() {
    let mut request = generate_request("MockServerURL",
      Some(proto_generator("MockServerURL", json!({ "example": "http://localhost:1234/a", "regex": ".*(/a)" }))),
      FieldValue::Json(json!("http://localhost:1234/a")), ProtoTestMode::Unknown);
    request.test_context = Some(to_proto_struct(&hashmap!{
      "mockServer".to_string() => json!({ "url": "http://127.0.0.1:9876" })
    }));

    let response = CoreFieldValueGenerator.generate_field(request).await.unwrap();

    expect!(response.error).to(be_equal_to(String::default()));
    expect!(FieldValue::from_proto(&response.value.unwrap()))
      .to(be_equal_to(FieldValue::Json(json!("http://127.0.0.1:9876/a"))));
  }

  #[tokio::test]
  async fn a_generator_this_framework_does_not_provide_is_an_error() {
    let request = generate_request("Uuid", Some(proto_generator("creditcard", json!({ "brand": "visa" }))),
      FieldValue::Json(json!("4111111111111111")), ProtoTestMode::Consumer);

    let response = CoreFieldValueGenerator.generate_field(request).await.unwrap();

    expect!(response.error.contains("'creditcard' is not one of the generators provided by this framework"))
      .to(be_true());
    expect!(response.value).to(be_none());
  }

  #[tokio::test]
  async fn a_collection_generator_says_why_it_can_not_be_applied() {
    let request = generate_request("ArrayContains", None, FieldValue::Json(json!([])), ProtoTestMode::Consumer);

    let response = CollectionValueGenerator.generate_field(request).await.unwrap();

    expect!(response.error.contains("generates values within a collection")).to(be_true());
  }

  /// A `Struct` has one number type, so every whole number a plugin sends as configuration arrives
  /// as a float. Fractional values are left alone - a `2.5` was meant to be a decimal.
  #[test]
  fn puts_whole_numbers_from_a_proto_struct_back_to_integers() {
    let normalised = whole_floats_to_integers(json!({
      "min": 2.0,
      "max": -3.0,
      "ratio": 2.5,
      "nested": { "size": 8.0 },
      "list": [1.0, 1.5],
      "text": "2.0"
    }));

    expect!(normalised).to(be_equal_to(json!({
      "min": 2,
      "max": -3,
      "ratio": 2.5,
      "nested": { "size": 8 },
      "list": [1, 1.5],
      "text": "2.0"
    })));
  }

  /// End to end through the driver: the catalogue entry `configure_core_catalogue` registers, the
  /// resolver a plugin's callback goes through, and the handler registered under that key. A key
  /// registered on one side but not the other would only show up here.
  #[tokio::test]
  async fn a_plugin_callback_for_a_core_rule_reaches_the_registered_handler() {
    crate::matchingrules::configure_core_catalogue();

    let matcher = pact_plugin_driver::field::find_field_matcher("type")
      .expect("expected the core 'type' rule to resolve");
    let context = FieldContext::new(&DocPath::new_unwrap("$.one"), "body");
    let matched = matcher.match_field(&MatchingRule::Type, &FieldValue::Json(json!("a")),
      &FieldValue::Json(json!("b")), &context).await;
    let mismatched = matcher.match_field(&MatchingRule::Type, &FieldValue::Json(json!("a")),
      &FieldValue::Json(json!(100)), &context).await;

    expect!(matcher.is_core()).to(be_true());
    expect!(matched).to(be_ok());
    let mismatches = mismatched.expect_err("expected a string against a number to be a type mismatch");
    expect!(mismatches.len()).to(be_equal_to(1));
    expect!(mismatches[0].path.as_str()).to(be_equal_to("$.one"));
  }

  /// The generator half of [`a_plugin_callback_for_a_core_rule_reaches_the_registered_handler`].
  #[tokio::test]
  async fn a_plugin_callback_for_a_core_generator_reaches_the_registered_handler() {
    crate::matchingrules::configure_core_catalogue();

    let generator = pact_plugin_driver::field::find_field_generator("Uuid")
      .expect("expected the core 'Uuid' generator to resolve");
    let context = FieldContext::new(&DocPath::new_unwrap("$.one"), "body");
    let generated = generator.generate_field(&Generator::Uuid(None), &FieldValue::Json(json!("")),
      FieldTestMode::Consumer, &context).await;

    expect!(generator.is_core()).to(be_true());
    match generated.expect("expected a generated value") {
      FieldValue::Json(serde_json::Value::String(value)) => {
        expect!(value.len()).to(be_equal_to(36));
      },
      other => panic!("expected a UUID string, got {:?}", other)
    }
  }

  /// Every key the catalogue advertises for this crate has a handler behind it, and no key has one
  /// that is not advertised - the two lists are what proposal 009 step 3 is
  #[test]
  fn every_registered_field_handler_matches_a_catalogue_entry() {
    let rules: HashSet<&str> = FIELD_RULES.iter().chain(COLLECTION_RULES.iter()).cloned().collect();
    let generators: HashSet<&str> = FIELD_GENERATORS.iter().chain(COLLECTION_GENERATORS.iter()).cloned().collect();
    let rule_entries: HashSet<&str> = crate::matchingrules::MATCHER_CATALOGUE_ENTRIES.iter()
      .map(|entry| entry.key.as_str())
      .collect();
    let generator_entries: HashSet<&str> = crate::matchingrules::GENERATOR_CATALOGUE_ENTRIES.iter()
      .map(|entry| entry.key.as_str())
      .collect();

    expect!(rules.symmetric_difference(&rule_entries).collect::<Vec<_>>()).to(be_equal_to(Vec::<&&str>::new()));
    expect!(generators.symmetric_difference(&generator_entries).collect::<Vec<_>>()).to(be_equal_to(Vec::<&&str>::new()));
  }
}
