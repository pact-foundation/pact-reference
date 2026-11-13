//! V1 to V4 HTTP and message provider features, verified only through the pact_ffi verifier functions. Mirrors
//! `v1_provider.rs`, `v2_provider.rs`, `shared_steps/provider.rs`, the provider halves of `v3_steps/message.rs`
//! and `v4_steps/message_provider.rs`, and the V3/V4 HTTP provider steps in the pact-reference compatibility suite.
//!
//! The pact files are written through the FFI (`pactffi_pact_handle_write_file`), and the provider, the provider
//! state callback and the Pact broker are small local HTTP servers (`server.rs`) that record what they receive.
//! A message provider is the same kind of server, answering the verifier's POST with the message the way the
//! reference suite's message proxy does: the body, its content type, and the metadata as base64 JSON in the
//! `pact-message-metadata` header.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use serde_json::{json, Value};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use pact_ffi_compat::ffi::{self, InteractionHandle, InteractionPart_Request, PactSpecification};
use pact_ffi_compat::ffi::{PactSpecification_V1, PactSpecification_V2, PactSpecification_V3, PactSpecification_V4};
use pact_ffi_compat::model::{self, Interaction, Part};
use pact_ffi_compat::server::{Request, Response, Server};
use pact_ffi_compat::trace;

#[derive(Debug, Default, World)]
struct ProviderWorld {
  /// "v1" to "v4", from the feature's directory.
  version: String,
  scenario_id: String,
  interactions: Vec<Interaction>,
  /// Messages the message provider can generate, by description: the body and the metadata.
  messages: Vec<(String, Part, serde_json::Map<String, Value>)>,
  provider: Option<Server>,
  state_callback: Option<Server>,
  /// Pact files written for the scenario, each a file source.
  pact_files: Vec<PathBuf>,
  /// One stand-in broker per broker-sourced pact, keyed by interaction number.
  brokers: Vec<(usize, Server)>,
  publish: bool,
  custom_headers: Vec<(String, String)>,
  exit: Option<i32>,
  results: Value,
  output: String,
}

impl ProviderWorld {
  fn spec(&self) -> PactSpecification {
    match self.version.as_str() {
      "v4" => PactSpecification_V4,
      "v3" => PactSpecification_V3,
      "v2" => PactSpecification_V2,
      _ => PactSpecification_V1,
    }
  }

  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite/provider").join(&self.version).join(&self.scenario_id)
  }

  fn interaction(&self, n: usize) -> anyhow::Result<Interaction> {
    self.interactions.get(n.wrapping_sub(1)).cloned().ok_or_else(|| anyhow!("no interaction {n}"))
  }

  fn state_calls(&self) -> Vec<Value> {
    self.state_callback.as_ref().map(|s| s.requests()).unwrap_or_default().iter()
      .filter_map(|r| serde_json::from_slice(&r.body).ok())
      .collect()
  }

  fn state_called(&self, action: &str, state: Option<&str>) -> bool {
    self.state_called_with(action, state, &[])
  }

  /// The callback body is `{"state", "params", "action"}`; each expected parameter must be in `params`.
  fn state_called_with(&self, action: &str, state: Option<&str>, params: &[(String, Value)]) -> bool {
    self.state_calls().iter().any(|c| c["action"] == action && state.map_or(true, |s| c["state"] == s)
      && params.iter().all(|(k, v)| &c["params"][k] == v))
  }

  fn published(&self, n: usize) -> Vec<Value> {
    self.brokers.iter().filter(|(i, _)| *i == n)
      .flat_map(|(_, b)| b.requests())
      .filter(|r| r.method == "POST" && r.path_only().ends_with("/verification-results"))
      .filter_map(|r| serde_json::from_slice(&r.body).ok())
      .collect()
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

/// Serves each interaction's response to a request with the same method and path; anything else is a 500.
async fn start_provider(world: &mut ProviderWorld, interactions: Vec<Interaction>) -> anyhow::Result<()> {
  let server = Server::start(Arc::new(move |req: &Request, _port| {
    interactions.iter()
      .find(|i| i.method.eq_ignore_ascii_case(&req.method) && i.path == req.path_only())
      .map(|i| Response {
        status: i.status,
        headers: i.response.headers.clone(),
        body: i.response.body.clone().unwrap_or_default(),
      })
      .unwrap_or(Response { status: 500, headers: vec![], body: b"no matching interaction".to_vec() })
  })).await?;
  trace::note(format!("provider listening on {}", server.url()));
  world.provider = Some(server);
  Ok(())
}

/// Builds a one-interaction pact through the FFI and writes it, as a consumer test would have. `setup` adds
/// whatever else the scenario needs (provider states, pending, comments) to the interaction.
fn write_pact(world: &ProviderWorld, n: usize, setup: impl FnOnce(InteractionHandle) -> anyhow::Result<()>) -> anyhow::Result<PathBuf> {
  let interaction = world.interaction(n)?;
  let consumer = format!("c_{n}");
  let pact = ffi::new_pact(&consumer, "p");
  ffi::with_specification(pact, world.spec());
  let i = model::register(pact, &interaction);
  setup(i)?;
  write_file(world, pact, &consumer)
}

/// Builds a one-message pact through the FFI: the message's description is its name, which is what the message
/// provider looks it up by. The consumer is `c_{name}`, so each message gets its own pact file.
fn write_message_pact(world: &ProviderWorld, name: &str, setup: impl FnOnce(InteractionHandle) -> anyhow::Result<()>) -> anyhow::Result<PathBuf> {
  let consumer = format!("c_{name}");
  let pact = ffi::new_pact(&consumer, "p");
  ffi::with_specification(pact, world.spec());
  let i = ffi::new_message_interaction(pact, name);
  setup(i)?;
  write_file(world, pact, &consumer)
}

fn write_file(world: &ProviderWorld, pact: ffi::PactHandle, consumer: &str) -> anyhow::Result<PathBuf> {
  let dir = world.pact_dir();
  let rc = ffi::pact_handle_write_file(pact, dir.to_str().unwrap(), false);
  ffi::free_pact_handle(pact);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  Ok(dir.join(format!("{consumer}-p.json")))
}

#[given("the following HTTP interactions have been defined:")]
fn interactions_defined(world: &mut ProviderWorld, step: &Step) {
  traced(step);
  world.interactions = model::setup_common_interactions(&table(step));
}

#[given(regex = r"^a provider is started that returns the response from interaction (\d+)$")]
async fn provider_for_interaction(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let i = world.interaction(n)?;
  start_provider(world, vec![i]).await
}

#[given(regex = r"^a provider is started that returns the response from interaction (\d+), with the following changes:$")]
async fn provider_for_changed_interaction(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let mut i = world.interaction(n)?;
  let rows = table(step);
  for (col, value) in rows[0].iter().zip(&rows[1]) {
    match col.as_str() {
      "response" => i.status = value.parse()?,
      "response headers" => i.response.headers.extend(model::parse_headers(value)),
      "response body" => model::setup_body(value, &mut i.response),
      other => bail!("unsupported change column {other}"),
    }
  }
  start_provider(world, vec![i]).await
}

#[given(regex = r#"^a provider is started that returns the responses from interactions "(.*)"$"#)]
async fn provider_for_interactions(world: &mut ProviderWorld, step: &Step, ids: String) -> anyhow::Result<()> {
  traced(step);
  let interactions = ids.split(',').map(|id| world.interaction(id.trim().parse()?)).collect::<anyhow::Result<_>>()?;
  start_provider(world, interactions).await
}

#[given(regex = r"^a Pact file for interaction (\d+) is to be verified$")]
fn pact_file(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let file = write_pact(world, n, |_| Ok(()))?;
  world.pact_files.push(file);
  Ok(())
}

#[given(regex = r#"^a Pact file for interaction (\d+) is to be verified with a provider state "(.*)" defined$"#)]
fn pact_file_with_state(world: &mut ProviderWorld, step: &Step, n: usize, state: String) -> anyhow::Result<()> {
  traced(step);
  let file = write_pact(world, n, |i| { ffi::given(i, &state); Ok(()) })?;
  world.pact_files.push(file);
  Ok(())
}

/// One provider state per row; a `Parameters` column holds the state's parameters as a JSON object.
#[given(regex = r"^a Pact file for interaction (\d+) is to be verified with the following provider states defined:$")]
fn pact_file_with_states(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let file = write_pact(world, n, |i| {
    let params_col = rows[0].iter().position(|c| c == "Parameters");
    for row in &rows[1..] {
      match params_col.map(|c| row[c].trim()).filter(|p| !p.is_empty()) {
        Some(params) => {
          let rc = ffi::given_with_params(i, &row[0], params);
          if rc != 0 {
            bail!("pactffi_given_with_params failed with {rc}");
          }
        }
        None => { ffi::given(i, &row[0]); }
      }
    }
    Ok(())
  })?;
  world.pact_files.push(file);
  Ok(())
}

#[given(regex = r"^a Pact file for interaction (\d+) is to be verified, but is marked pending$")]
fn pact_file_pending(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let file = write_pact(world, n, |i| { ffi::set_pending(i, true); Ok(()) })?;
  world.pact_files.push(file);
  Ok(())
}

/// `text` rows are added as text comments, a `testname` row as the `testname` comment.
fn add_comments(i: InteractionHandle, rows: &[Vec<String>]) -> anyhow::Result<()> {
  for row in &rows[1..] {
    match row[1].as_str() {
      "text" => { ffi::add_text_comment(i, &row[0]); }
      "testname" => { ffi::set_comment(i, "testname", &Value::String(row[0].clone()).to_string()); }
      other => bail!("unknown comment type {other}"),
    }
  }
  Ok(())
}

#[given(regex = r"^a Pact file for interaction (\d+) is to be verified with the following comments:$")]
fn pact_file_with_comments(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let file = write_pact(world, n, |i| add_comments(i, &rows))?;
  world.pact_files.push(file);
  Ok(())
}

/// Cucumber's `{string}` unescapes `\"`; these steps match with a regex, so do the same.
fn unescape(s: &str) -> String {
  s.replace("\\\"", "\"")
}

/// `JSON: ...` is a JSON value, anything else a string.
fn metadata_value(value: &str) -> anyhow::Result<Value> {
  Ok(match value.trim().strip_prefix("JSON:") {
    Some(json) => serde_json::from_str(json.trim())?,
    None => Value::String(value.to_string()),
  })
}

fn message_part(cell: &str) -> Part {
  let mut part = Part::default();
  model::setup_body(cell, &mut part);
  part
}

fn add_message(world: &mut ProviderWorld, name: String, body: &str, metadata: serde_json::Map<String, Value>) {
  world.messages.push((name, message_part(&unescape(body)), metadata));
}

#[given(regex = r#"^a provider is started that can generate the "(.*)" message with "(.*)"$"#)]
fn message_provider(world: &mut ProviderWorld, step: &Step, name: String, body: String) {
  traced(step);
  add_message(world, name, &body, Default::default());
}

#[given(regex = r#"^a provider is started that can generate the "(.*)" message with "(.*)" and the following metadata:$"#)]
fn message_provider_with_metadata(world: &mut ProviderWorld, step: &Step, name: String, body: String) -> anyhow::Result<()> {
  traced(step);
  let metadata = table(step)[1..].iter()
    .map(|row| Ok((row[0].clone(), metadata_value(&row[1])?)))
    .collect::<anyhow::Result<_>>()?;
  add_message(world, name, &body, metadata);
  Ok(())
}

/// Serves every message the scenario defined. The verifier POSTs `{"description": ...}` for each message.
async fn start_message_provider(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let messages = world.messages.clone();
  let server = Server::start(Arc::new(move |req: &Request, _port| {
    let description = serde_json::from_slice::<Value>(&req.body).ok()
      .and_then(|v| v["description"].as_str().map(str::to_string)).unwrap_or_default();
    match messages.iter().find(|(name, _, _)| *name == description) {
      Some((_, part, metadata)) => {
        let ct = part.header("content-type").unwrap_or("text/plain").to_string();
        let mut metadata = metadata.clone();
        metadata.entry("contentType").or_insert(Value::String(ct.clone()));
        Response {
          status: 200,
          headers: vec![
            ("Content-Type".into(), ct),
            ("pact-message-metadata".into(), BASE64.encode(Value::Object(metadata).to_string())),
          ],
          body: part.body.clone().unwrap_or_default(),
        }
      }
      None => Response::json(404, &json!({"error": format!("no message '{description}'")})),
    }
  })).await?;
  trace::note(format!("message provider listening on {}", server.url()));
  world.provider = Some(server);
  Ok(())
}

/// The message's body from the fixture cell, through `pactffi_with_body` or `pactffi_with_binary_body`.
fn message_body(i: InteractionHandle, cell: &str, matching_rules: Option<String>) {
  let mut part = message_part(&unescape(cell));
  part.matching_rules = matching_rules;
  model::register_part(i, InteractionPart_Request, &part);
}

fn push_message_pact(world: &mut ProviderWorld, name: &str, setup: impl FnOnce(InteractionHandle) -> anyhow::Result<()>) -> anyhow::Result<()> {
  let file = write_message_pact(world, name, setup)?;
  world.pact_files.push(file);
  Ok(())
}

#[given(regex = r#"^a Pact file for "([^"]*)":"(.*)" is to be verified$"#)]
fn message_pact(world: &mut ProviderWorld, step: &Step, name: String, body: String) -> anyhow::Result<()> {
  traced(step);
  push_message_pact(world, &name, |i| { message_body(i, &body, None); Ok(()) })
}

#[given(regex = r#"^a Pact file for "([^"]*)":"(.*)" is to be verified with provider state "(.*)"$"#)]
fn message_pact_with_state(world: &mut ProviderWorld, step: &Step, name: String, body: String, state: String) -> anyhow::Result<()> {
  traced(step);
  push_message_pact(world, &name, |i| { ffi::given(i, &state); message_body(i, &body, None); Ok(()) })
}

#[given(regex = r#"^a Pact file for "([^"]*)":"(.*)" is to be verified, but is marked pending$"#)]
fn message_pact_pending(world: &mut ProviderWorld, step: &Step, name: String, body: String) -> anyhow::Result<()> {
  traced(step);
  push_message_pact(world, &name, |i| { message_body(i, &body, None); ffi::set_pending(i, true); Ok(()) })
}

#[given(regex = r#"^a Pact file for "([^"]*)":"(.*)" is to be verified with the following comments:$"#)]
fn message_pact_with_comments(world: &mut ProviderWorld, step: &Step, name: String, body: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  push_message_pact(world, &name, |i| { message_body(i, &body, None); add_comments(i, &rows) })
}

/// `pactffi_with_metadata` parses the value as JSON and falls back to a string, so a string is passed as is.
fn add_metadata(i: InteractionHandle, key: &str, value: &str) -> anyhow::Result<()> {
  let value = match value.trim().strip_prefix("JSON:") {
    Some(json) => json.trim().to_string(),
    None => value.to_string(),
  };
  if !ffi::with_metadata(i, key, &value, InteractionPart_Request) {
    bail!("pactffi_with_metadata failed for {key}");
  }
  Ok(())
}

#[given(regex = r#"^a Pact file for "([^"]*)":"(.*)" is to be verified with the following metadata:$"#)]
fn message_pact_with_metadata(world: &mut ProviderWorld, step: &Step, name: String, body: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  push_message_pact(world, &name, |i| {
    message_body(i, &body, None);
    rows[1..].iter().try_for_each(|row| add_metadata(i, &row[0], &row[1]))
  })
}

/// Rows of `body`, `matching rules` (a fixture or `JSON:`) and `metadata` (`key=value; ...`).
#[given(regex = r#"^a Pact file for "([^"]*)" is to be verified with the following:$"#)]
fn message_pact_with_following(world: &mut ProviderWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let get = |key: &str| rows.iter().find(|r| r[0] == key).map(|r| r[1].clone());
  let rules = get("matching rules").and_then(|cell| model::matching_rules(&cell));
  push_message_pact(world, &name, |i| {
    message_body(i, &get("body").unwrap_or_default(), rules);
    for entry in get("metadata").unwrap_or_default().split(';').filter(|e| !e.trim().is_empty()) {
      let (key, value) = entry.trim().split_once('=').ok_or_else(|| anyhow!("bad metadata {entry}"))?;
      add_metadata(i, key, value)?;
    }
    Ok(())
  })
}

/// The broker serves the pacts-for-verification flow for one pact and accepts verification results.
#[given(regex = r"^a Pact file for interaction (\d+) is to be verified from a Pact broker$")]
async fn pact_from_broker(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let file = write_pact(world, n, |_| Ok(()))?;
  let pact: Value = serde_json::from_str(&fs::read_to_string(&file)?)?;
  let broker = Server::start(Arc::new(move |req: &Request, port| {
    let base = format!("http://127.0.0.1:{port}");
    let pact_url = format!("{base}/pacts/provider/p/consumer/c_{n}");
    match (req.method.as_str(), req.path_only()) {
      ("GET", "/") => Response::json(200, &json!({"_links": {
        "pb:provider-pacts-for-verification": {
          "href": format!("{base}/pacts/provider/{{provider}}/for-verification"), "templated": true
        }
      }})),
      // The verifier navigates here first, then POSTs the selectors to the resource's self link.
      ("GET", "/pacts/provider/p/for-verification") => Response::json(200, &json!({"_links": {
        "self": {"href": format!("{base}/pacts/provider/p/for-verification")}
      }})),
      ("POST", "/pacts/provider/p/for-verification") => Response::json(200, &json!({
        "_embedded": {"pacts": [{
          "_links": {"self": {"href": pact_url, "name": format!("Pact between c_{n} and p")}},
          "shortDescription": "latest"
        }]},
        "_links": {"self": {"href": format!("{base}/pacts/provider/p/for-verification")}}
      })),
      ("GET", path) if path == format!("/pacts/provider/p/consumer/c_{n}") => {
        let mut pact = pact.clone();
        pact["_links"] = json!({
          "self": {"href": pact_url},
          "pb:publish-verification-results": {"href": format!("{pact_url}/verification-results")}
        });
        Response::json(200, &pact)
      }
      ("POST", path) if path.ends_with("/verification-results") => Response::json(201, &json!({})),
      _ => Response::json(404, &json!({"error": "not found"})),
    }
  })).await?;
  trace::note(format!("Pact broker for c_{n} listening on {}", broker.url()));
  world.brokers.push((n, broker));
  Ok(())
}

#[given("publishing of verification results is enabled")]
fn publishing_enabled(world: &mut ProviderWorld, step: &Step) {
  traced(step);
  world.publish = true;
}

async fn state_callback(world: &mut ProviderWorld, fail: bool) -> anyhow::Result<()> {
  let server = Server::start(Arc::new(move |_req: &Request, _port| {
    if fail { Response::json(500, &json!({"error": "state change failed"})) } else { Response::json(200, &json!({})) }
  })).await?;
  trace::note(format!("provider state callback listening on {}", server.url()));
  world.state_callback = Some(server);
  Ok(())
}

#[given("a provider state callback is configured")]
async fn state_callback_configured(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  state_callback(world, false).await
}

#[given("a provider state callback is configured, but will return a failure")]
async fn failing_state_callback_configured(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  state_callback(world, true).await
}

/// The FFI has no request filter callback, only custom headers, so other kinds of change can't be expressed.
#[given("a request filter is configured to make the following changes:")]
fn request_filter(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  for (col, value) in rows[0].iter().zip(&rows[1]) {
    match col.as_str() {
      "headers" => world.custom_headers.extend(model::parse_headers(value)),
      other => bail!("pact_ffi has no request filter for {other}; only pactffi_verifier_add_custom_header exists"),
    }
  }
  Ok(())
}

#[when("the verification is run")]
async fn verification_run(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !world.messages.is_empty() {
    start_message_provider(world).await?;
  }
  let provider = world.provider.as_ref().ok_or_else(|| anyhow!("no provider started"))?;
  let v = ffi::verifier_new_for_application("pact-ffi-compat", env!("CARGO_PKG_VERSION"));
  ffi::verifier_set_provider_info(v, "p", "http", "127.0.0.1", provider.port, "/");
  if let Some(cb) = &world.state_callback {
    ffi::verifier_set_provider_state(v, &cb.url(), true, true);
  }
  if world.publish {
    ffi::verifier_set_publish_options(v, "1.2.3");
  }
  for (k, val) in &world.custom_headers {
    ffi::verifier_add_custom_header(v, k, val);
  }
  for file in &world.pact_files {
    ffi::verifier_add_file_source(v, file.to_str().unwrap());
  }
  for (_, broker) in &world.brokers {
    ffi::verifier_broker_source_with_selectors(v, &broker.url(), false, None);
  }
  // The verifier blocks on its own runtime; the servers keep running on this one.
  let exit = tokio::task::spawn_blocking(move || ffi::verifier_execute(v)).await?;
  world.exit = Some(exit);
  world.output = ffi::verifier_output(v, true).unwrap_or_default();
  world.results = ffi::verifier_json(v).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
  ffi::verifier_shutdown(v);
  Ok(())
}

#[then("the verification will be successful")]
fn successful(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.exit != Some(0) || world.results["result"] != true {
    bail!("verification failed (exit {:?}):\n{}", world.exit, world.output);
  }
  Ok(())
}

#[then("the verification will NOT be successful")]
fn not_successful(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.exit == Some(0) || world.results["result"] != false {
    bail!("verification passed (exit {:?})", world.exit);
  }
  Ok(())
}

/// Maps the suite's error wording onto the verifier's JSON report: mismatch types for response and message
/// mismatches, the error message for a failed state change. `entries` are `interactionResults` or
/// `pendingErrors`, both of which carry a `mismatch`.
fn has_error(entries: &Value, error: &str) -> anyhow::Result<bool> {
  let mismatch_types: Vec<String> = entries.as_array().cloned().unwrap_or_default().iter()
    .flat_map(|r| r["mismatch"]["mismatches"].as_array().cloned().unwrap_or_default())
    .filter_map(|m| m["type"].as_str().map(str::to_string))
    .collect();
  let has = |types: &[&str]| mismatch_types.iter().any(|t| types.contains(&t.as_str()));
  Ok(match error {
    "Response status did not match" => has(&["StatusMismatch"]),
    "Headers had differences" => has(&["HeaderMismatch"]),
    "Body had differences" => has(&["BodyMismatch", "BodyTypeMismatch"]),
    "Body type had differences" => has(&["BodyTypeMismatch"]),
    "Metadata had differences" => has(&["MetadataMismatch"]),
    "State change request failed" => entries.to_string().contains("state change handlers has failed"),
    other => bail!("unknown error {other}"),
  })
}

#[then(regex = r#"^the verification results will contain a "(.*)" error$"#)]
fn results_contain_error(world: &mut ProviderWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  if !has_error(&world.results["interactionResults"], &error)? {
    bail!("no \"{error}\" error in the verification results: {}", world.results);
  }
  Ok(())
}

#[then(regex = r#"^there will be a pending "(.*)" error$"#)]
fn pending_error(world: &mut ProviderWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  if !has_error(&world.results["pendingErrors"], &error)? {
    bail!("no pending \"{error}\" error in the verification results: {}", world.results);
  }
  Ok(())
}

#[then(regex = r#"^the comment "(.*)" will have been printed to the console$"#)]
fn comment_printed(world: &mut ProviderWorld, step: &Step, comment: String) -> anyhow::Result<()> {
  traced(step);
  if !world.output.contains(&comment) {
    bail!("comment '{comment}' not in the verifier output:\n{}", world.output);
  }
  Ok(())
}

#[then(regex = r#"^the "(.*)" will displayed as the original test name$"#)]
fn test_name_printed(world: &mut ProviderWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  if !world.output.contains(&format!("Test Name: {name}")) {
    bail!("test name '{name}' not in the verifier output:\n{}", world.output);
  }
  Ok(())
}

/// The table's first row names the parameters, the second holds their JSON values.
fn expected_params(step: &Step) -> anyhow::Result<Vec<(String, Value)>> {
  let rows = table(step);
  rows[0].iter().zip(&rows[1]).map(|(k, v)| Ok((k.clone(), serde_json::from_str(v)?))).collect()
}

#[then(regex = r#"^the provider state callback will receive a setup call with "(.*)" and the following parameters:$"#)]
fn state_setup_with_params(world: &mut ProviderWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let params = expected_params(step)?;
  if !world.state_called_with("setup", Some(&state), &params) {
    bail!("no setup call for state '{state}' with {params:?}: {:?}", world.state_calls());
  }
  Ok(())
}

#[then(regex = r#"^the provider state callback will receive a teardown call "(.*)" and the following parameters:$"#)]
fn state_teardown_with_params(world: &mut ProviderWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let params = expected_params(step)?;
  if !world.state_called_with("teardown", Some(&state), &params) {
    bail!("no teardown call for state '{state}' with {params:?}: {:?}", world.state_calls());
  }
  Ok(())
}

#[then("a verification result will NOT be published back")]
fn not_published(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let published: Vec<Value> = world.brokers.iter().flat_map(|(n, _)| world.published(*n)).collect();
  if !published.is_empty() {
    bail!("verification results were published: {published:?}");
  }
  Ok(())
}

fn published_with(world: &ProviderWorld, n: usize, success: bool) -> anyhow::Result<()> {
  let published = world.published(n);
  if !published.iter().any(|r| r["success"] == success) {
    bail!("no verification result with success = {success} published for interaction {n}: {published:?}");
  }
  Ok(())
}

#[then(regex = r"^a successful verification result will be published back for interaction \{(\d+)\}$")]
fn published_success(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  published_with(world, n, true)
}

#[then(regex = r"^a failed verification result will be published back for the interaction \{(\d+)\}$")]
fn published_failure(world: &mut ProviderWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  published_with(world, n, false)
}

#[then("the provider state callback will be called before the verification is run")]
fn state_setup_called(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !world.state_called("setup", None) {
    bail!("provider state callback was not called for setup: {:?}", world.state_calls());
  }
  Ok(())
}

#[then(regex = r#"^the provider state callback will receive a setup call with "(.*)" as the provider state parameter$"#)]
fn state_setup_with(world: &mut ProviderWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  if !world.state_called("setup", Some(&state)) {
    bail!("no setup call for state '{state}': {:?}", world.state_calls());
  }
  Ok(())
}

#[then("the provider state callback will be called after the verification is run")]
fn state_teardown_called(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !world.state_called("teardown", None) {
    bail!("provider state callback was not called for teardown: {:?}", world.state_calls());
  }
  Ok(())
}

#[then(regex = r#"^the provider state callback will receive a teardown call "(.*)" as the provider state parameter$"#)]
fn state_teardown_with(world: &mut ProviderWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  if !world.state_called("teardown", Some(&state)) {
    bail!("no teardown call for state '{state}': {:?}", world.state_calls());
  }
  Ok(())
}

#[then("the provider state callback will NOT receive a teardown call")]
fn state_no_teardown(world: &mut ProviderWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.state_called("teardown", None) {
    bail!("provider state callback received a teardown call: {:?}", world.state_calls());
  }
  Ok(())
}

/// The reference suite does not check this either: the warning only goes to the log.
#[then(regex = r#"^a warning will be displayed that there was no provider state callback configured for provider state "(.*)"$"#)]
fn no_state_callback_warning(_world: &mut ProviderWorld, step: &Step, _state: String) {
  traced(step);
}

#[then(regex = r#"^the request to the provider will contain the header "(.*)"$"#)]
fn provider_request_header(world: &mut ProviderWorld, step: &Step, header: String) -> anyhow::Result<()> {
  traced(step);
  let (name, value) = header.split_once(':').map(|(k, v)| (k.trim(), v.trim())).ok_or_else(|| anyhow!("bad header"))?;
  let requests = world.provider.as_ref().map(|p| p.requests()).unwrap_or_default();
  if requests.is_empty() || !requests.iter().all(|r| r.header(name) == Some(value)) {
    bail!("not every provider request had {name}: {value}: {requests:?}");
  }
  Ok(())
}

fn trace_path(feature: &str, scenario: &str) -> PathBuf {
  let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
  let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
  PathBuf::from(dir).join(safe(feature)).join(format!("{}.md", safe(scenario)))
}

/// `features/V2/http_provider.feature` -> "v2".
fn feature_version(feature: &cucumber::gherkin::Feature) -> String {
  feature.path.as_ref().and_then(|p| p.parent()?.file_name()?.to_str().map(str::to_lowercase)).unwrap_or_default()
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  ProviderWorld::cucumber()
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
      let file = feature.path.as_ref().and_then(|p| p.file_name()?.to_str().map(str::to_string)).unwrap_or_default();
      matches!(feature_version(feature).as_str(), "v1" | "v2" | "v3" | "v4")
        && file.ends_with("provider.feature") && !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
