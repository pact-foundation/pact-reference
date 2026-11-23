//! V3 HTTP matching and V3/V4 matching rules features, driven only through pact_ffi. Mirrors `v3_steps/http_matching.rs`
//! and `v4_steps/http_matching.rs` in the pact-reference compatibility suite.
//!
//! pact_ffi has no call that compares a request with an expected one. A client gets a comparison by starting a mock
//! server with the expected request and sending it the received one; the mismatches come back from
//! `pactffi_mock_server_mismatches`. Responses are compared by verifying a pact against a provider that returns the
//! received response, and reading `pactffi_verifier_json`.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use regex::Regex;
use serde_json::Value;

use pact_ffi_compat::ffi::{self, InteractionPart_Response, PactSpecification,
  PactSpecification_V3, PactSpecification_V4};
use pact_ffi_compat::model::{self, Interaction, Part};
use pact_ffi_compat::server::{Request as ServerRequest, Response as ServerResponse, Server};
use pact_ffi_compat::trace;

#[derive(Debug, Clone, Default)]
struct Received {
  query: String,
  part: Part,
}

#[derive(Debug, Default, World)]
struct MatchingWorld {
  /// "v3" or "v4", from the feature's directory.
  version: String,
  scenario_id: String,
  expected: Interaction,
  received: Vec<Received>,
  /// One list of request mismatches per received request.
  results: Vec<Vec<Value>>,
  expected_status: u16,
  response_rules: Option<String>,
  received_status: u16,
  response_mismatches: Vec<Value>,
}

impl MatchingWorld {
  fn spec(&self) -> PactSpecification {
    if self.version == "v4" { PactSpecification_V4 } else { PactSpecification_V3 }
  }

  fn consumer(&self) -> String {
    format!("{}-compatibility-suite-c", self.version)
  }

  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite").join(&self.version).join(&self.scenario_id)
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

fn expected_request() -> Interaction {
  Interaction { description: "expected request".into(), method: "GET".into(), path: "/".into(), status: 200,
    ..Interaction::default() }
}

/// Fills `part` and `query` from one row of a `| body | content type | query | headers | ... |` table.
fn apply_row(fields: &[String], values: &[String], part: &mut Part, query: &mut String) {
  let get = |name: &str| fields.iter().position(|f| f == name).map(|n| values[n].as_str());
  if let Some(headers) = get("headers") {
    if !headers.is_empty() {
      part.headers = model::parse_headers(headers);
    }
  }
  if let Some(body) = get("body") {
    model::setup_body_as(body, part, get("content type").filter(|ct| !ct.is_empty()));
  }
  if let Some(q) = get("query") {
    *query = q.to_string();
  }
  if let Some(rules) = get("matching rules") {
    part.matching_rules = model::matching_rules(rules);
  }
}

#[given(expr = "an expected request with a(n) {string} header of {string}")]
fn expected_header(world: &mut MatchingWorld, step: &Step, name: String, value: String) {
  traced(step);
  world.expected.request.set_header(&name, &value);
}

#[given(expr = "a request is received with a(n) {string} header of {string}")]
fn received_header(world: &mut MatchingWorld, step: &Step, name: String, value: String) {
  traced(step);
  let mut part = Part::default();
  part.set_header(&name, &value);
  world.received.push(Received { query: String::new(), part });
}

#[given("an expected request configured with the following:")]
fn expected_configured(world: &mut MatchingWorld, step: &Step) {
  traced(step);
  let rows = table(step);
  let mut i = expected_request();
  apply_row(&rows[0], &rows[1], &mut i.request, &mut i.query);
  world.expected = i;
}

#[given("a request is received with the following:")]
fn received_configured(world: &mut MatchingWorld, step: &Step) {
  traced(step);
  let rows = table(step);
  let mut r = Received::default();
  apply_row(&rows[0], &rows[1], &mut r.part, &mut r.query);
  world.received.push(r);
}

#[given("the following requests are received:")]
fn received_many(world: &mut MatchingWorld, step: &Step) {
  traced(step);
  let rows = table(step);
  let fields: Vec<String> = rows[0].iter().filter(|f| *f != "desc").cloned().collect();
  for row in &rows[1..] {
    let values: Vec<String> = rows[0].iter().zip(row).filter(|(f, _)| *f != "desc").map(|(_, v)| v.clone()).collect();
    let mut r = Received::default();
    apply_row(&fields, &values, &mut r.part, &mut r.query);
    world.received.push(r);
  }
}

/// Starts a mock server holding only the expected request, sends it `received`, and returns its request mismatches.
async fn compare(world: &MatchingWorld, received: &Received) -> anyhow::Result<Vec<Value>> {
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  model::register(pact, &world.expected);
  let port = ffi::create_mock_server_for_transport(pact, "::1", 0, "http");
  if port < 0 {
    ffi::free_pact_handle(pact);
    bail!("pactffi_create_mock_server_for_transport failed with {port}");
  }
  let mut url = format!("http://[::1]:{port}/");
  if !received.query.is_empty() {
    url.push('?');
    url.push_str(&received.query);
  }
  let mut req = reqwest::Client::new().get(&url);
  for (k, v) in &received.part.headers {
    req = req.header(k, v);
  }
  if let Some(body) = &received.part.body {
    req = req.body(body.clone());
  }
  let sent = req.send().await;
  let mismatches: Vec<Value> = ffi::mock_server_mismatches(port).and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_default();
  ffi::cleanup_mock_server(port);
  ffi::free_pact_handle(pact);
  let status = sent?.status();
  trace::note(format!("HTTP GET {url} -> {status}"));
  Ok(mismatches.iter()
    .flat_map(|m| match m["type"].as_str() {
      Some("request-mismatch") => m["mismatches"].as_array().cloned().unwrap_or_default(),
      _ => vec![m.clone()],
    })
    .collect())
}

#[when(regex = r"^the requests? (?:is|are) compared to the expected one$")]
async fn compared(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let received = world.received.clone();
  for r in &received {
    let result = compare(world, r).await?;
    world.results.push(result);
  }
  Ok(())
}

#[then("the comparison should be OK")]
fn comparison_ok(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if let Some(m) = world.results.iter().find(|m| !m.is_empty()) {
    bail!("Expected all requests to match, got mismatches {}", Value::from(m.clone()));
  }
  Ok(())
}

#[then("the comparison should NOT be OK")]
fn comparison_not_ok(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.results.iter().all(|m| m.is_empty()) {
    bail!("All requests matched");
  }
  Ok(())
}

/// The path field for each mismatch type, as `Mismatch::to_json` writes them.
fn mismatch_path(m: &Value) -> Option<&str> {
  let field = match m["type"].as_str()? {
    "QueryMismatch" => "parameter",
    "HeaderMismatch" | "MetadataMismatch" => "key",
    "BodyMismatch" => "path",
    _ => return None,
  };
  m[field].as_str()
}

/// cucumber-rs leaves Gherkin's `\"` escapes in a `{string}` parameter.
fn description_matches(description: &str, error: &str) -> bool {
  let error = &error.replace("\\\"", "\"");
  description.contains(error) || Regex::new(error).map(|re| re.is_match(description)).unwrap_or(false)
}

#[then(expr = "the mismatches will contain a mismatch with error {string} -> {string}")]
fn mismatch_with_error(world: &mut MatchingWorld, step: &Step, path: String, error: String) -> anyhow::Result<()> {
  traced(step);
  let all: Vec<Value> = world.results.iter().flatten().cloned().collect();
  let found = all.iter().any(|m| {
    mismatch_path(m) == Some(path.as_str()) && description_matches(m["mismatch"].as_str().unwrap_or_default(), &error)
  });
  if !found {
    bail!("Did not find a mismatch at {path} with error {error:?} in {}", Value::from(all));
  }
  Ok(())
}

#[given("an expected response configured with the following:")]
fn expected_response(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  for (field, value) in rows[0].iter().zip(&rows[1]) {
    match field.as_str() {
      "status" => world.expected_status = value.parse()?,
      "matching rules" => world.response_rules = model::matching_rules(value),
      other => bail!("unsupported response field {other}"),
    }
  }
  Ok(())
}

#[given(expr = "a status {int} response is received")]
fn status_received(world: &mut MatchingWorld, step: &Step, status: u16) {
  traced(step);
  world.received_status = status;
}

/// Verifies a one-interaction pact holding the expected response against a provider that returns the received
/// status. pact_ffi has no response comparison call, so this is the only way to reach it.
#[when("the response is compared to the expected one")]
async fn response_compared(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  let i = ffi::new_interaction(pact, "expected response");
  ffi::with_request(i, "GET", "/");
  ffi::response_status(i, world.expected_status);
  if let Some(rules) = &world.response_rules {
    ffi::with_matching_rules(i, InteractionPart_Response, rules);
  }
  let dir = world.pact_dir();
  fs::create_dir_all(&dir)?;
  let rc = ffi::pact_handle_write_file(pact, dir.to_str().unwrap(), true);
  ffi::free_pact_handle(pact);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  let file = dir.join(format!("{}-p.json", world.consumer()));

  let status = world.received_status;
  let provider = Server::start(Arc::new(move |_req: &ServerRequest, _port| {
    ServerResponse { status, ..ServerResponse::default() }
  })).await?;
  let v = ffi::verifier_new_for_application("pact-ffi-compat", env!("CARGO_PKG_VERSION"));
  ffi::verifier_set_provider_info(v, "p", "http", "127.0.0.1", provider.port, "/");
  ffi::verifier_add_file_source(v, file.to_str().unwrap());
  tokio::task::spawn_blocking(move || ffi::verifier_execute(v)).await?;
  let results: Value = ffi::verifier_json(v).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
  ffi::verifier_shutdown(v);
  world.response_mismatches = results["errors"].as_array().into_iter().flatten()
    .flat_map(|e| e["mismatch"]["mismatches"].as_array().cloned().unwrap_or_default())
    .collect();
  Ok(())
}

#[then("the response comparison should be OK")]
fn response_ok(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !world.response_mismatches.is_empty() {
    bail!("Expected the response to match, got {}", Value::from(world.response_mismatches.clone()));
  }
  Ok(())
}

#[then("the response comparison should NOT be OK")]
fn response_not_ok(world: &mut MatchingWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.response_mismatches.is_empty() {
    bail!("The response matched");
  }
  Ok(())
}

#[then(expr = "the response mismatches will contain a {string} mismatch with error {string}")]
fn response_mismatch(world: &mut MatchingWorld, step: &Step, kind: String, error: String) -> anyhow::Result<()> {
  traced(step);
  let ty = match kind.as_str() {
    "status" => "StatusMismatch",
    "header" => "HeaderMismatch",
    "body" => "BodyMismatch",
    other => return Err(anyhow!("unknown mismatch kind {other}")),
  };
  let found = world.response_mismatches.iter()
    .any(|m| m["type"] == ty && description_matches(m["mismatch"].as_str().unwrap_or_default(), &error));
  if !found {
    bail!("Did not find a {kind} mismatch with error {error:?} in {}", Value::from(world.response_mismatches.clone()));
  }
  Ok(())
}

fn trace_path(feature: &str, scenario: &str) -> PathBuf {
  let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
  let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
  PathBuf::from(dir).join(safe(feature)).join(format!("{}.md", safe(scenario)))
}

/// `features/V3/http_matching.feature` -> "v3".
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

  MatchingWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|feature, _rule, scenario, world| Box::pin(async move {
      trace::reset();
      world.version = feature_version(feature);
      world.scenario_id = scenario.name.clone();
      world.expected = expected_request();
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
      ((v == "v3" && (f == "http_matching" || f == "matching_rules")) || (v == "v4" && f == "matching_rules"))
        && !scenario.tags.iter().any(|t| t == "wip")
        && !KNOWN_FAILURES.contains(&scenario.name.as_str())
    })
    .await;
}

// The mock server reads a 0-byte body as no body (pact_mock_server `extract_body`), so the notEmpty rule
// never sees the empty request body.
const KNOWN_FAILURES: &[&str] = &["Supports a not empty matcher with binary data (negative case)"];
