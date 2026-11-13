//! V3 generators, V3 HTTP generators and V4 generators features, driven only through pact_ffi. Mirrors
//! `v3_steps/generators.rs` and `v4_steps/generators.rs` in the pact-reference compatibility suite.
//!
//! pact_ffi has no call that generates a request or response. A client sees generated values by running them
//! through the parts of the library that apply generators: a request is prepared by verifying a pact against a
//! provider that records what it receives (provider mode, with a state change server for a provider state
//! context), and a response by sending a request to a mock server (consumer mode). The mock server URL generator
//! only runs with a mock server context in consumer mode, which an HTTP client can't supply, so that scenario
//! walks a V4 message's body generators with `pactffi_generator_generate_string`.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use regex::Regex;
use serde_json::Value;

use pact_ffi_compat::ffi::{self, InteractionPart_Request, InteractionPart_Response, PactSpecification,
  PactSpecification_V3, PactSpecification_V4};
use pact_ffi_compat::model::{self, Part, FIXTURES};
use pact_ffi_compat::server::{Request as ServerRequest, Response as ServerResponse, Server};
use pact_ffi_compat::trace;

#[derive(Debug, Clone, Default)]
struct Generated {
  status: u16,
  /// Path and query as sent.
  path: String,
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

impl Generated {
  fn header(&self, name: &str) -> Option<&str> {
    self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
  }
}

#[derive(Debug, Default, World)]
struct GeneratorsWorld {
  /// "v3" or "v4", from the feature's directory.
  version: String,
  scenario_id: String,
  request: Part,
  request_generators: Option<String>,
  response: Part,
  response_generators: Option<String>,
  mode: String,
  generated_request: Option<Generated>,
  generated_response: Option<Generated>,
}

impl GeneratorsWorld {
  fn spec(&self) -> PactSpecification {
    if self.version == "v4" { PactSpecification_V4 } else { PactSpecification_V3 }
  }

  fn consumer(&self) -> String {
    format!("{}-compatibility-suite-c", self.version)
  }

  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite").join(&self.version).join(&self.scenario_id)
  }

  /// The body that was generated, from whichever side was prepared.
  fn generated_body(&self) -> anyhow::Result<Value> {
    let g = self.generated_request.as_ref().or(self.generated_response.as_ref())
      .ok_or_else(|| anyhow!("nothing was prepared"))?;
    serde_json::from_slice(&g.body).map_err(|e| anyhow!("generated body is not JSON ({e}): {:?}",
      String::from_utf8_lossy(&g.body)))
  }

  fn original_body(&self) -> Value {
    let part = if self.generated_request.is_some() { &self.request } else { &self.response };
    part.body.as_ref().and_then(|b| serde_json::from_slice(b).ok()).unwrap_or(Value::Null)
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

/// A `| body | generators |` table: the part's body, and the generators as JSON text.
fn configure(step: &Step) -> (Part, Option<String>) {
  let rows = table(step);
  let mut part = Part::default();
  let mut generators = None;
  for (field, value) in rows[0].iter().zip(&rows[1]) {
    match field.as_str() {
      "body" => model::setup_body(value, &mut part),
      "generators" => {
        let json = match value.strip_prefix("JSON:") {
          Some(json) => json.trim().to_string(),
          None => fs::read_to_string(format!("{FIXTURES}/{value}"))
            .unwrap_or_else(|e| panic!("fixture {value}: {e}")),
        };
        generators = Some(json);
      }
      _ => {}
    }
  }
  (part, generators)
}

#[given("a request configured with the following generators:")]
fn request_generators(world: &mut GeneratorsWorld, step: &Step) {
  traced(step);
  let (part, generators) = configure(step);
  world.request = part;
  world.request_generators = generators;
}

#[given("a response configured with the following generators:")]
fn response_generators(world: &mut GeneratorsWorld, step: &Step) {
  traced(step);
  let (part, generators) = configure(step);
  world.response = part;
  world.response_generators = generators;
}

#[given(expr = "the generator test mode is set as {string}")]
fn test_mode(world: &mut GeneratorsWorld, step: &Step, mode: String) {
  traced(step);
  world.mode = mode;
}

/// Verifies a pact holding the configured request against a provider that records it, and returns the first
/// request the provider received. With a provider state context, a state change server returns it as JSON, which
/// the verifier adds to the generator context.
async fn prepare_request(world: &GeneratorsWorld, state: Option<Value>) -> anyhow::Result<Generated> {
  let dir = world.pact_dir();
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  let i = ffi::new_interaction(pact, "a request with generators");
  if state.is_some() {
    ffi::given(i, "a state");
  }
  ffi::with_request(i, "GET", "/path/one");
  model::register_part(i, InteractionPart_Request, &world.request);
  if let Some(generators) = &world.request_generators {
    ffi::with_generators(i, InteractionPart_Request, generators);
  }
  ffi::response_status(i, 200);
  fs::create_dir_all(&dir)?;
  let rc = ffi::pact_handle_write_file(pact, dir.to_str().unwrap(), true);
  ffi::free_pact_handle(pact);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  let file = dir.join(format!("{}-p.json", world.consumer()));

  let provider = Server::start(Arc::new(|_req: &ServerRequest, _port| ServerResponse { status: 200,
    ..ServerResponse::default() })).await?;
  let state_server = match &state {
    Some(context) => {
      let context = context.clone();
      Some(Server::start(Arc::new(move |_req: &ServerRequest, _port| ServerResponse::json(200, &context))).await?)
    }
    None => None,
  };
  let v = ffi::verifier_new_for_application("pact-ffi-compat", env!("CARGO_PKG_VERSION"));
  ffi::verifier_set_provider_info(v, "p", "http", "127.0.0.1", provider.port, "/");
  if let Some(s) = &state_server {
    ffi::verifier_set_provider_state(v, &s.url(), false, true);
  }
  ffi::verifier_add_file_source(v, file.to_str().unwrap());
  tokio::task::spawn_blocking(move || ffi::verifier_execute(v)).await?;
  ffi::verifier_shutdown(v);

  let req = provider.requests().into_iter().next().ok_or_else(|| anyhow!("the provider received no request"))?;
  trace::note(format!("provider received {} {}", req.method, req.path));
  Ok(Generated { status: 0, path: req.path.clone(), headers: req.headers.clone(), body: req.body.clone() })
}

/// Builds a V4 message whose contents are the configured request body, runs its body generators with `context`,
/// and writes each generated value into the body.
fn prepare_message_body(world: &GeneratorsWorld, context: &Value) -> anyhow::Result<Generated> {
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, PactSpecification_V4);
  let i = ffi::new_message_interaction(pact, "a message with generators");
  let ct = world.request.header("content-type").unwrap_or("application/json").to_string();
  let body = world.request.body.clone().unwrap_or_default();
  ffi::with_body(i, InteractionPart_Request, &ct, std::str::from_utf8(&body)?);
  if let Some(generators) = &world.request_generators {
    ffi::with_generators(i, InteractionPart_Request, generators);
  }
  let generated = ffi::async_body_generators_with_context(pact, &context.to_string());
  ffi::free_pact_handle(pact);

  let mut json: Value = serde_json::from_slice(&body)?;
  for (path, value) in generated {
    let value = value.ok_or_else(|| anyhow!("no value generated for {path}"))?;
    let pointer = path.strip_prefix('$').unwrap_or(&path).replace('.', "/");
    let slot = json.pointer_mut(&pointer).ok_or_else(|| anyhow!("{path} is not in the body"))?;
    *slot = Value::String(value);
  }
  Ok(Generated { body: serde_json::to_vec(&json)?, ..Generated::default() })
}

#[when("the request is prepared for use")]
async fn request_prepared(world: &mut GeneratorsWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  world.generated_request = Some(prepare_request(world, None).await?);
  Ok(())
}

#[when(expr = "the request is prepared for use with a {string} context:")]
async fn request_prepared_with_context(world: &mut GeneratorsWorld, step: &Step, kind: String)
  -> anyhow::Result<()> {
  traced(step);
  let context: Value = serde_json::from_str(&table(step)[0][0])?;
  let generated = match kind.as_str() {
    "providerState" => prepare_request(world, Some(context)).await?,
    "mockServer" => {
      let href = context["href"].clone();
      prepare_message_body(world, &serde_json::json!({ "mockServer": { "url": href, "href": href } }))?
    }
    other => bail!("unknown context {other}"),
  };
  world.generated_request = Some(generated);
  Ok(())
}

#[when("the response is prepared for use")]
async fn response_prepared(world: &mut GeneratorsWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  let i = ffi::new_interaction(pact, "a response with generators");
  ffi::with_request(i, "GET", "/path/one");
  ffi::response_status(i, 200);
  model::register_part(i, InteractionPart_Response, &world.response);
  if let Some(generators) = &world.response_generators {
    ffi::with_generators(i, InteractionPart_Response, generators);
  }
  let port = ffi::create_mock_server_for_transport(pact, "127.0.0.1", 0, "http");
  if port < 0 {
    ffi::free_pact_handle(pact);
    bail!("pactffi_create_mock_server_for_transport failed with {port}");
  }
  let url = format!("http://127.0.0.1:{port}/path/one");
  let sent = reqwest::get(&url).await;
  ffi::cleanup_mock_server(port);
  ffi::free_pact_handle(pact);
  let res = sent?;
  let status = res.status().as_u16();
  trace::note(format!("HTTP GET {url} -> {status}"));
  let headers = res.headers().iter()
    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string())).collect();
  let body = res.bytes().await?.to_vec();
  world.generated_response = Some(Generated { status, path: String::new(), headers, body });
  Ok(())
}

fn json_to_string(v: &Value) -> String {
  match v {
    Value::String(s) => s.clone(),
    other => other.to_string(),
  }
}

/// Body path `$.one` -> the value at `/one`.
fn body_value<'a>(body: &'a Value, path: &str) -> anyhow::Result<&'a Value> {
  let pointer = path.strip_prefix('$').unwrap_or(path).replace('.', "/");
  body.pointer(&pointer).ok_or_else(|| anyhow!("{path} is not in the body {body}"))
}

fn assert_value_type(kind: &str, v: &Value) -> anyhow::Result<()> {
  let s = json_to_string(v);
  let pattern = match kind {
    "integer" => r"^-?\d+$",
    "decimal number" => r"^-?\d+\.\d+$",
    "hexadecimal number" => r"^[a-fA-F0-9]+$",
    "random string" | "string from the regex" if !v.is_string() => bail!("expected a string, got {v}"),
    "random string" => return Ok(()),
    "string from the regex" => r"^\d{1,8}$",
    "date" => r"^\d{4}-\d{2}-\d{2}$",
    "time" => r"^\d{2}:\d{2}:\d{2}$",
    "date-time" => r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,9})?",
    "UUID" | "simple UUID" | "lower-case-hyphenated UUID" | "upper-case-hyphenated UUID" | "URN UUID" =>
      r"^(urn:uuid:)?[a-fA-F0-9]{8}-?[a-fA-F0-9]{4}-?[a-fA-F0-9]{4}-?[a-fA-F0-9]{4}-?[a-fA-F0-9]{12}$",
    "boolean" => r"^(true|false)$",
    other => bail!("unknown value type {other}"),
  };
  if !Regex::new(pattern)?.is_match(&s) {
    bail!("expected a(n) {kind}, got {v}");
  }
  Ok(())
}

#[then(expr = "the body value for {string} will have been replaced with a(n) {string}")]
fn body_replaced_with_type(world: &mut GeneratorsWorld, step: &Step, path: String, kind: String)
  -> anyhow::Result<()> {
  traced(step);
  let body = world.generated_body()?;
  let original = world.original_body();
  let value = body_value(&body, &path)?;
  if Some(value) == body_value(&original, &path).ok() {
    bail!("{path} was not replaced: {value}");
  }
  assert_value_type(&kind, value)
}

#[then(expr = "the body value for {string} will have been replaced with {string}")]
fn body_replaced_with_value(world: &mut GeneratorsWorld, step: &Step, path: String, expected: String)
  -> anyhow::Result<()> {
  traced(step);
  let body = world.generated_body()?;
  let original = world.original_body();
  let value = body_value(&body, &path)?;
  if Some(value) == body_value(&original, &path).ok() {
    bail!("{path} was not replaced: {value}");
  }
  if json_to_string(value) != expected {
    bail!("expected {path} to be {expected:?}, got {value}");
  }
  Ok(())
}

/// `header[X-TEST]` -> ("header", "X-TEST"); `path` -> ("path", "").
fn field(name: &str) -> (String, String) {
  let key = Regex::new(r"\[(.*)]").unwrap();
  match key.captures(name) {
    Some(c) => (name[..c.get(0).unwrap().start()].to_string(), c[1].to_string()),
    None => (name.to_string(), String::new()),
  }
}

fn request_field(g: &Generated, name: &str) -> anyhow::Result<String> {
  let (f, key) = field(name);
  match f.as_str() {
    "path" => Ok(g.path.split('?').next().unwrap_or_default().to_string()),
    "header" => g.header(&key).map(str::to_string).ok_or_else(|| anyhow!("no {key} header in {:?}", g.headers)),
    "queryParameter" => {
      let query = g.path.split_once('?').map(|(_, q)| q).unwrap_or_default();
      model::parse_query(query).into_iter().find(|(k, _)| *k == key).map(|(_, v)| v)
        .ok_or_else(|| anyhow!("no {key} query parameter in {:?}", g.path))
    }
    other => bail!("unknown request field {other}"),
  }
}

fn response_field(g: &Generated, name: &str) -> anyhow::Result<String> {
  let (f, key) = field(name);
  match f.as_str() {
    "status" => Ok(g.status.to_string()),
    "header" => g.header(&key).map(str::to_string).ok_or_else(|| anyhow!("no {key} header in {:?}", g.headers)),
    other => bail!("unknown response field {other}"),
  }
}

#[then(expr = "the request {string} will be set as {string}")]
fn request_set_as(world: &mut GeneratorsWorld, step: &Step, name: String, expected: String) -> anyhow::Result<()> {
  traced(step);
  let g = world.generated_request.as_ref().ok_or_else(|| anyhow!("the request was not prepared"))?;
  let actual = request_field(g, &name)?;
  if actual != expected {
    bail!("expected request {name} to be {expected:?}, got {actual:?}");
  }
  Ok(())
}

#[then(expr = "the request {string} will match {string}")]
fn request_matches(world: &mut GeneratorsWorld, step: &Step, name: String, pattern: String) -> anyhow::Result<()> {
  traced(step);
  let g = world.generated_request.as_ref().ok_or_else(|| anyhow!("the request was not prepared"))?;
  let actual = request_field(g, &name)?;
  if !Regex::new(&format!("^{pattern}$"))?.is_match(&actual) {
    bail!("expected request {name} to match {pattern:?}, got {actual:?}");
  }
  Ok(())
}

#[then(expr = "the response {string} will not be {string}")]
fn response_not(world: &mut GeneratorsWorld, step: &Step, name: String, value: String) -> anyhow::Result<()> {
  traced(step);
  let g = world.generated_response.as_ref().ok_or_else(|| anyhow!("the response was not prepared"))?;
  let actual = response_field(g, &name)?;
  if actual == value {
    bail!("expected response {name} not to be {value:?}");
  }
  Ok(())
}

#[then(expr = "the response {string} will match {string}")]
fn response_matches(world: &mut GeneratorsWorld, step: &Step, name: String, pattern: String) -> anyhow::Result<()> {
  traced(step);
  let g = world.generated_response.as_ref().ok_or_else(|| anyhow!("the response was not prepared"))?;
  let actual = response_field(g, &name)?;
  if !Regex::new(&format!("^{pattern}$"))?.is_match(&actual) {
    bail!("expected response {name} to match {pattern:?}, got {actual:?}");
  }
  Ok(())
}

fn trace_path(feature: &str, scenario: &str) -> PathBuf {
  let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
  let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
  PathBuf::from(dir).join(safe(feature)).join(format!("{}.md", safe(scenario)))
}

/// `features/V3/generators.feature` -> "v3".
fn feature_version(feature: &cucumber::gherkin::Feature) -> String {
  feature.path.as_ref().and_then(|p| p.parent()?.file_name()?.to_str().map(str::to_lowercase)).unwrap_or_default()
}

fn feature_file(feature: &cucumber::gherkin::Feature) -> String {
  feature.path.as_ref().and_then(|p| p.file_stem()?.to_str().map(str::to_string)).unwrap_or_default()
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  GeneratorsWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|feature, _rule, scenario, world| Box::pin(async move {
      trace::reset();
      world.version = feature_version(feature);
      world.scenario_id = scenario.name.clone();
      let _ = fs::remove_dir_all(world.pact_dir());
    }))
    .after(|feature, _rule, scenario, _ev, _world| Box::pin(async move {
      let path = trace_path(&format!("{} {}", feature_version(feature).to_uppercase(), feature.name), &scenario.name);
      if let Err(e) = trace::write(&path, &feature.name, &scenario.name) {
        eprintln!("could not write trace {}: {e}", path.display());
      }
    }))
    .filter_run_and_exit(model::FEATURES, |feature, _rule, scenario| {
      let v = feature_version(feature);
      let f = feature_file(feature);
      ((v == "v3" && (f == "generators" || f == "http_generators")) || (v == "v4" && f == "generators"))
        && !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
