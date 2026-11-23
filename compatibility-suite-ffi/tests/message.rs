//! V3 and V4 message consumer features (async and sync), plus V4's mixed-interaction pact, driven only through
//! pact_ffi. Mirrors `v3_steps/message.rs`, `v4_steps/message_consumer.rs`,
//! `v4_steps/sync_message_consumer.rs` and `v4_steps/mod.rs` in the pact-reference compatibility suite.
//!
//! A client's message consumer test is: define the message, generate its contents (what the consumer under test
//! receives), hand them to the consumer code, and write the pact only if that succeeded. The runner has no
//! consumer code, so "successfully processed" generates and writes, and "NOT successfully processed" writes
//! nothing.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use regex::Regex;
use serde_json::Value;

use pact_ffi_compat::ffi::{self, Contents, InteractionHandle, InteractionPart, InteractionPart_Request,
  InteractionPart_Response, PactHandle, PactSpecification, PactSpecification_V3, PactSpecification_V4};
use pact_ffi_compat::model::{self, Part, FIXTURES};
use pact_ffi_compat::trace;

#[derive(Debug, Default, World)]
struct MessageWorld {
  /// "v3" or "v4", from the feature's directory.
  version: String,
  scenario_id: String,
  pact: Option<PactHandle>,
  interaction: Option<InteractionHandle>,
  sync: bool,
  /// The error the consumer test failed with.
  failed: Option<String>,
  received: Vec<Contents>,
  received_sync: Vec<(Contents, Vec<Contents>)>,
  pact_json: Value,
  /// Bodies and metadata as configured, to check that generators replaced them.
  request_body: Option<Value>,
  response_body: Option<Value>,
  request_metadata: HashMap<String, Value>,
  response_metadata: HashMap<String, Value>,
}

impl MessageWorld {
  fn spec(&self) -> PactSpecification {
    if self.version == "v4" { PactSpecification_V4 } else { PactSpecification_V3 }
  }

  fn consumer(&self) -> String {
    format!("{}-compatibility-suite-c", self.version)
  }

  fn pact_file(&self) -> PathBuf {
    self.pact_dir().join(format!("{}-p.json", self.consumer()))
  }

  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite").join(&self.version).join(&self.scenario_id)
  }

  fn pact(&mut self) -> PactHandle {
    if let Some(pact) = self.pact {
      return pact;
    }
    let pact = ffi::new_pact(&self.consumer(), "p");
    ffi::with_specification(pact, self.spec());
    self.pact = Some(pact);
    pact
  }

  fn defined(&self) -> anyhow::Result<InteractionHandle> {
    self.interaction.ok_or_else(|| anyhow!("no message is being defined"))
  }

  fn shutdown(&mut self) {
    if let Some(pact) = self.pact.take() {
      ffi::free_pact_handle(pact);
    }
  }

  fn write_pact(&mut self) -> anyhow::Result<()> {
    let pact = self.pact.ok_or_else(|| anyhow!("no pact is being defined"))?;
    let dir = self.pact_dir();
    fs::create_dir_all(&dir)?;
    let rc = ffi::pact_handle_write_file(pact, &dir.to_string_lossy(), true);
    if rc != 0 {
      bail!("pactffi_pact_handle_write_file failed with {rc}");
    }
    self.pact_json = serde_json::from_str(&fs::read_to_string(self.pact_file())?)?;
    Ok(())
  }

  /// The received async message, or the received sync message's request.
  fn received(&self, which: &str) -> anyhow::Result<&Contents> {
    let found = match which {
      "" if !self.sync => self.received.first(),
      "" | "response" => self.received_sync.first().and_then(|(_, r)| r.first()),
      "request" => self.received_sync.first().map(|(r, _)| r),
      _ => None,
    };
    found.ok_or_else(|| anyhow!("no {which} message was received"))
  }

  /// The first message in the pact file: V3 `messages[0]`, V4 `interactions[0]`.
  fn first_message(&self) -> &Value {
    match self.pact_json.get("messages") {
      Some(m) => &m[0],
      None => &self.pact_json["interactions"][0],
    }
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

fn part_of(which: &str) -> InteractionPart {
  if which == "response" { InteractionPart_Response } else { InteractionPart_Request }
}

fn json_to_string(v: &Value) -> String {
  match v {
    Value::String(s) => s.clone(),
    other => other.to_string(),
  }
}

/// Metadata values come back from the FFI as `json_to_string` text: strings bare, everything else as JSON.
fn metadata_value(c: &Contents, key: &str) -> anyhow::Result<Value> {
  let (_, v) = c.metadata.iter().find(|(k, _)| k == key)
    .ok_or_else(|| anyhow!("no metadata {key} in {:?}", c.metadata))?;
  Ok(serde_json::from_str(v).unwrap_or_else(|_| Value::String(v.clone())))
}

/// `JSON: {...}` is a JSON value (the feature escapes its quotes); anything else is a string.
fn expected_value(s: &str) -> anyhow::Result<Value> {
  match s.strip_prefix("JSON:") {
    Some(json) => Ok(serde_json::from_str(&json.trim().replace("\\\"", "\""))?),
    None => Ok(Value::String(s.to_string())),
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
    "boolean" => r"^(true|false)$",
    other => bail!("unknown value type {other}"),
  };
  if !Regex::new(pattern)?.is_match(&s) {
    bail!("expected a(n) {kind}, got {v}");
  }
  Ok(())
}

/// Sets a message body the way a client would: text through `pactffi_with_body`, binary through
/// `pactffi_with_binary_body`. Returns the body as a JSON value when it is one.
fn add_body(i: InteractionHandle, p: InteractionPart, cell: &str) -> anyhow::Result<Option<Value>> {
  let mut part = Part::default();
  model::setup_body(cell, &mut part);
  let body = part.body.clone().unwrap_or_default();
  let ct = part.header("content-type").unwrap_or("text/plain").to_string();
  let ok = if model::is_binary(&ct, &body) {
    ffi::with_binary_body(i, p, &ct, &body)
  } else {
    ffi::with_body(i, p, &ct, std::str::from_utf8(&body)?)
  };
  if !ok {
    bail!("pactffi_with_body failed");
  }
  Ok(serde_json::from_slice(&body).ok())
}

/// The fixture's contents as the pact file holds them: a JSON value for JSON, otherwise text.
fn expected_document(cell: &str) -> anyhow::Result<(Value, String)> {
  let cell = if cell.contains(':') { cell.to_string() } else { format!("file: {cell}") };
  let mut part = Part::default();
  model::setup_body(&cell, &mut part);
  let ct = part.header("content-type").unwrap_or("text/plain").to_string();
  let body = part.body.unwrap_or_default();
  let value = serde_json::from_slice(&body).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).into()));
  Ok((value, ct))
}

/// Compares a document, read either from generated contents (bytes) or a pact file (a JSON value or text).
fn compare_document(actual: &Value, cell: &str) -> anyhow::Result<()> {
  let (expected, _) = expected_document(cell)?;
  let actual = match (actual, &expected) {
    (Value::String(s), e) if !e.is_string() => serde_json::from_str(s).unwrap_or(actual.clone()),
    _ => actual.clone(),
  };
  if actual != expected {
    bail!("Expected document {expected} but was {actual}");
  }
  Ok(())
}

fn body_as_value(c: &Contents) -> Value {
  let bytes = c.body.clone().unwrap_or_default();
  serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
}

// ---- Defining the interaction ----

#[given(regex = r"^a message (?:integration|interaction) is being defined for a consumer test$")]
fn message_defined(world: &mut MessageWorld, step: &Step) {
  traced(step);
  let pact = world.pact();
  world.interaction = Some(ffi::new_message_interaction(pact, "a message"));
  world.sync = false;
}

#[given("a synchronous message interaction is being defined for a consumer test")]
fn sync_message_defined(world: &mut MessageWorld, step: &Step) {
  traced(step);
  let pact = world.pact();
  world.interaction = Some(ffi::new_sync_message_interaction(pact, "a synchronous message"));
  world.sync = true;
}

/// v4.feature puts an HTTP interaction and a message in one pact.
#[given("an HTTP interaction is being defined for a consumer test")]
fn http_defined(world: &mut MessageWorld, step: &Step) {
  traced(step);
  let pact = world.pact();
  let i = ffi::new_interaction(pact, "an HTTP interaction");
  ffi::with_request(i, "GET", "/");
}

#[given("a message is defined")]
fn a_message_is_defined(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  world.defined().map(|_| ())
}

#[given(regex = r#"^the message (request |)payload contains the "([^"]+)" JSON document$"#)]
fn payload_json(world: &mut MessageWorld, step: &Step, _request: String, name: String) -> anyhow::Result<()> {
  traced(step);
  world.request_body = add_body(world.defined()?, InteractionPart_Request, &format!("file: {name}.json"))?;
  Ok(())
}

#[given(regex = r#"^the message response payload contains the "([^"]+)" document$"#)]
fn response_payload(world: &mut MessageWorld, step: &Step, cell: String) -> anyhow::Result<()> {
  traced(step);
  world.response_body = add_body(world.defined()?, InteractionPart_Response, &cell)?;
  Ok(())
}

fn add_metadata(world: &mut MessageWorld, which: &str, key: &str, value: Value) -> anyhow::Result<()> {
  if !ffi::with_metadata(world.defined()?, key, &value.to_string(), part_of(which)) {
    bail!("pactffi_with_metadata failed for {key}");
  }
  let originals = if which == "response" { &mut world.response_metadata } else { &mut world.request_metadata };
  originals.insert(key.to_string(), value);
  Ok(())
}

/// `| key | value |`, where a `JSON:` value is a JSON value and anything else a string.
#[given(regex = r"^the message (request |)contains the following metadata:$")]
fn metadata_table(world: &mut MessageWorld, step: &Step, _request: String) -> anyhow::Result<()> {
  traced(step);
  for row in table(step).iter().skip(1) {
    add_metadata(world, "request", &row[0], expected_value(&row[1])?)?;
  }
  Ok(())
}

/// `| body | generators | metadata |`, any of them. Generators are inline `JSON:` or a fixture file.
#[given(regex = r"^the message (request |response |)is configured with the following:$")]
fn configured(world: &mut MessageWorld, step: &Step, which: String) -> anyhow::Result<()> {
  traced(step);
  let which = which.trim().to_string();
  let i = world.defined()?;
  let p = part_of(&which);
  let rows = table(step);
  let cells: HashMap<&str, &str> = rows[0].iter().map(String::as_str).zip(rows[1].iter().map(String::as_str)).collect();
  match cells.get("body") {
    Some(body) => {
      let value = add_body(i, p, body)?;
      if which == "response" { world.response_body = value } else { world.request_body = value }
    }
    // Sync response metadata and generators only apply to responses that exist, and only a body adds one.
    None if world.sync && which == "response" => {
      if !ffi::with_body(i, p, "text/plain", "") {
        bail!("pactffi_with_body failed");
      }
    }
    None => {}
  }
  if let Some(metadata) = cells.get("metadata") {
    let metadata: Value = serde_json::from_str(metadata)?;
    for (k, v) in metadata.as_object().ok_or_else(|| anyhow!("metadata is not an object"))? {
      add_metadata(world, &which, k, v.clone())?;
    }
  }
  if let Some(generators) = cells.get("generators") {
    let json = match generators.strip_prefix("JSON:") {
      Some(json) => json.trim().to_string(),
      None => fs::read_to_string(format!("{FIXTURES}/{generators}"))?,
    };
    if !ffi::with_generators(i, p, &json) {
      bail!("pactffi_with_generators failed");
    }
  }
  Ok(())
}

#[given(regex = r#"^a provider state "([^"]+)" for the (?:synchronous )?message is specified$"#)]
fn provider_state(world: &mut MessageWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::given(world.defined()?, &state) {
    bail!("pactffi_given failed");
  }
  Ok(())
}

/// The table's first row names the parameters; the second holds their values as JSON.
#[given(regex = r#"^a provider state "([^"]+)" for the (?:synchronous )?message is specified with the following data:$"#)]
fn provider_state_with_data(world: &mut MessageWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let params: serde_json::Map<String, Value> = rows[0].iter().zip(&rows[1])
    .map(|(k, v)| Ok((k.clone(), serde_json::from_str(v)?)))
    .collect::<anyhow::Result<_>>()?;
  let rc = ffi::given_with_params(world.defined()?, &state, &Value::Object(params).to_string());
  if rc != 0 {
    bail!("pactffi_given_with_params failed with {rc}");
  }
  Ok(())
}

#[given(regex = r#"^a key of "([^"]+)" is specified for the (?:synchronous )?message interaction$"#)]
fn key_specified(world: &mut MessageWorld, step: &Step, key: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::set_key(world.defined()?, &key) {
    bail!("pactffi_set_key failed");
  }
  Ok(())
}

#[given(regex = r"^the (?:synchronous )?message interaction is marked as pending$")]
fn marked_pending(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !ffi::set_pending(world.defined()?, true) {
    bail!("pactffi_set_pending failed");
  }
  Ok(())
}

#[given(regex = r#"^a comment "([^"]+)" is added to the (?:synchronous )?message interaction$"#)]
fn comment_added(world: &mut MessageWorld, step: &Step, comment: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::add_text_comment(world.defined()?, &comment) {
    bail!("pactffi_add_text_comment failed");
  }
  Ok(())
}

// ---- Running the consumer test ----

#[when("the message is successfully processed")]
fn processed(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let pact = world.pact.ok_or_else(|| anyhow!("no pact is being defined"))?;
  if world.sync {
    world.received_sync = ffi::sync_messages(pact);
  } else {
    world.received = ffi::async_messages(pact);
  }
  world.write_pact()
}

#[when(expr = "the message is NOT successfully processed with a {string} exception")]
fn not_processed(world: &mut MessageWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  let pact = world.pact.ok_or_else(|| anyhow!("no pact is being defined"))?;
  world.received = ffi::async_messages(pact);
  world.failed = Some(error);
  Ok(())
}

#[when("the Pact file for the test is generated")]
fn pact_generated(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  world.write_pact()
}

#[then("the consumer test will have passed")]
fn passed(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  match &world.failed {
    Some(e) => bail!("the consumer test failed with {e}"),
    None => Ok(()),
  }
}

#[then("the consumer test will have failed")]
fn failed(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  world.failed.as_ref().map(|_| ()).ok_or_else(|| anyhow!("the consumer test passed"))
}

#[then(expr = "the consumer test error will be {string}")]
fn failed_with(world: &mut MessageWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  if world.failed.as_deref() != Some(error.as_str()) {
    bail!("Expected error {error:?} but was {:?}", world.failed);
  }
  Ok(())
}

// ---- What the consumer received ----

#[then(regex = r#"^the received message payload will contain the "([^"]+)" (?:JSON )?document$"#)]
fn received_payload(world: &mut MessageWorld, step: &Step, cell: String) -> anyhow::Result<()> {
  traced(step);
  let cell = if cell.contains(':') { cell } else { format!("{cell}.json") };
  compare_document(&body_as_value(world.received("")?), &cell)
}

#[then(expr = "the received message content type will be {string}")]
fn received_content_type(world: &mut MessageWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let actual = metadata_value(world.received("")?, "contentType")?;
  if actual != expected.as_str() {
    bail!("Expected content type {expected} but was {actual}");
  }
  Ok(())
}

#[then(regex = r#"^the received message (request |response |)metadata will contain "([^"]+)" == "(.*)"$"#)]
fn received_metadata(world: &mut MessageWorld, step: &Step, which: String, key: String, value: String)
  -> anyhow::Result<()> {
  traced(step);
  let actual = metadata_value(world.received(which.trim())?, &key)?;
  let expected = expected_value(&value)?;
  if actual != expected {
    bail!("Expected metadata {key} = {expected} but was {actual}");
  }
  Ok(())
}

#[then(regex = r#"^the received message (request |response |)metadata will contain "([^"]+)" replaced with an? "([^"]+)"$"#)]
fn received_metadata_replaced(world: &mut MessageWorld, step: &Step, which: String, key: String, kind: String)
  -> anyhow::Result<()> {
  traced(step);
  let which = which.trim();
  let actual = metadata_value(world.received(which)?, &key)?;
  let originals = if which == "response" { &world.response_metadata } else { &world.request_metadata };
  if originals.get(&key) == Some(&actual) {
    bail!("metadata {key} was not replaced: {actual}");
  }
  assert_value_type(&kind, &actual)
}

#[then(regex = r#"^the message (request |response |)contents for "([^"]+)" will have been replaced with an? "([^"]+)"$"#)]
fn contents_replaced(world: &mut MessageWorld, step: &Step, which: String, path: String, kind: String)
  -> anyhow::Result<()> {
  traced(step);
  let which = which.trim();
  let body = body_as_value(world.received(which)?);
  let original = if which == "response" { &world.response_body } else { &world.request_body };
  let value = body_value(&body, &path)?;
  if original.as_ref().and_then(|o| body_value(o, &path).ok()) == Some(value) {
    bail!("{path} was not replaced: {value}");
  }
  assert_value_type(&kind, value)
}

// ---- The pact file ----

#[then(regex = r"^a Pact file for the message interaction will have been written$")]
fn pact_written(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !world.pact_file().exists() {
    bail!("{} was not written", world.pact_file().display());
  }
  Ok(())
}

#[then(regex = r"^a Pact file for the message interaction will NOT have been written$")]
fn pact_not_written(world: &mut MessageWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.pact_file().exists() {
    bail!("{} was written", world.pact_file().display());
  }
  Ok(())
}

#[then(regex = r"^the pact file will contain (\d+) (message )?interactions?$")]
fn interaction_count(world: &mut MessageWorld, step: &Step, n: usize, message: String) -> anyhow::Result<()> {
  traced(step);
  let key = if message.is_empty() || world.version == "v4" { "interactions" } else { "messages" };
  let actual = world.pact_json[key].as_array().map(Vec::len).unwrap_or(0);
  if actual != n {
    bail!("Expected {n} {key} but found {actual}");
  }
  Ok(())
}

#[then(expr = "the first message in the pact file will contain the {string} document")]
fn first_message_document(world: &mut MessageWorld, step: &Step, fixture: String) -> anyhow::Result<()> {
  traced(step);
  compare_document(&world.first_message()["contents"], &fixture)
}

#[then(expr = "the first message in the pact file content type will be {string}")]
fn first_message_content_type(world: &mut MessageWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &world.first_message()["metadata"]["contentType"];
  if actual != expected.as_str() {
    bail!("Expected content type {expected} but was {actual}");
  }
  Ok(())
}

#[then(regex = r#"^the first message in the pact file will contain the (request |)message metadata "([^"]+)" == "(.*)"$"#)]
fn first_message_metadata(world: &mut MessageWorld, step: &Step, request: String, key: String, value: String)
  -> anyhow::Result<()> {
  traced(step);
  let message = world.first_message();
  let metadata = if request.is_empty() { &message["metadata"] } else { &message["request"]["metadata"] };
  let expected = expected_value(&value)?;
  if metadata[key.as_str()] != expected {
    bail!("Expected metadata {key} = {expected} but was {}", metadata[key.as_str()]);
  }
  Ok(())
}

fn provider_states(world: &MessageWorld) -> Vec<Value> {
  world.first_message()["providerStates"].as_array().cloned().unwrap_or_default()
}

#[then(regex = r"^the first message in the pact file will contain (\d+) provider states?$")]
fn provider_state_count(world: &mut MessageWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let states = provider_states(world);
  if states.len() != n {
    bail!("Expected {n} provider states but found {}", Value::from(states));
  }
  Ok(())
}

#[then(expr = "the first message in the Pact file will contain provider state {string}")]
fn provider_state_present(world: &mut MessageWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let states = provider_states(world);
  if !states.iter().any(|s| s["name"] == state.as_str()) {
    bail!("No provider state '{state}' in {}", Value::from(states));
  }
  Ok(())
}

#[then(regex = r#"^the provider state "([^"]+)" for the message will contain the following parameters:$"#)]
fn provider_state_params(world: &mut MessageWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(&table(step)[1][0])?;
  let states = provider_states(world);
  let s = states.iter().find(|s| s["name"] == state.as_str())
    .ok_or_else(|| anyhow!("No provider state '{state}' in {}", Value::from(states.clone())))?;
  if s["params"] != expected {
    bail!("Expected parameters {expected} but were {}", s["params"]);
  }
  Ok(())
}

fn first_interaction(world: &MessageWorld) -> anyhow::Result<&Value> {
  world.pact_json["interactions"].get(0).ok_or_else(|| anyhow!("no interactions in the pact file"))
}

/// A V4 message part's `contents` is `{content, contentType, encoded}`.
fn v4_content(contents: &Value) -> Value {
  let content = &contents["content"];
  match contents["encoded"].as_str() {
    Some("base64") | Some("BASE64") => content.as_str()
      .and_then(|s| base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s).ok())
      .map(|b| Value::String(String::from_utf8_lossy(&b).into()))
      .unwrap_or(content.clone()),
    _ => content.clone(),
  }
}

#[then(regex = r#"^the first interaction in the pact file will contain the "([^"]+)" document as (the request|a response|the first response message|the second response message)$"#)]
fn interaction_document(world: &mut MessageWorld, step: &Step, cell: String, which: String) -> anyhow::Result<()> {
  traced(step);
  let i = first_interaction(world)?;
  let contents = match which.as_str() {
    "the request" => &i["request"]["contents"],
    "the second response message" => &i["response"][1]["contents"],
    _ => &i["response"][0]["contents"],
  };
  compare_document(&v4_content(contents), &cell)
}

#[then(regex = r#"^the first interaction in the pact file (request|response) content type will be "([^"]+)"$"#)]
fn interaction_content_type(world: &mut MessageWorld, step: &Step, which: String, expected: String)
  -> anyhow::Result<()> {
  traced(step);
  let i = first_interaction(world)?;
  let part = if which == "request" { &i["request"] } else { &i["response"][0] };
  let actual = &part["contents"]["contentType"];
  if actual != expected.as_str() {
    bail!("Expected {which} content type {expected} but was {actual} in {part}");
  }
  Ok(())
}

#[then(regex = r"^the first interaction in the pact file will contain (\d+) response messages?$")]
fn response_count(world: &mut MessageWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let actual = first_interaction(world)?["response"].as_array().map(Vec::len).unwrap_or(0);
  if actual != n {
    bail!("Expected {n} response messages but found {actual}");
  }
  Ok(())
}

#[then(expr = "the first interaction in the Pact file will have a type of {string}")]
fn interaction_type(world: &mut MessageWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &first_interaction(world)?["type"];
  if actual != expected.as_str() {
    bail!("Expected type '{expected}' but was {actual}");
  }
  Ok(())
}

#[then(expr = "there will be an interaction in the Pact file with a type of {string}")]
fn any_interaction_type(world: &mut MessageWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let interactions = world.pact_json["interactions"].as_array().cloned().unwrap_or_default();
  if !interactions.iter().any(|i| i["type"] == expected.as_str()) {
    bail!("No interaction of type '{expected}' in {}", Value::from(interactions));
  }
  Ok(())
}

/// The expected value is JSON in single quotes.
#[then(regex = r#"^the first interaction in the Pact file will have "([^"]+)" = '(.*)'$"#)]
fn interaction_attribute(world: &mut MessageWorld, step: &Step, name: String, value: String) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(&value)?;
  let actual = &first_interaction(world)?[name.as_str()];
  if *actual != expected {
    bail!("Expected {name} = {expected} but was {actual}");
  }
  Ok(())
}

fn trace_path(feature: &str, scenario: &str) -> PathBuf {
  let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
  let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
  PathBuf::from(dir).join(safe(feature)).join(format!("{}.md", safe(scenario)))
}

/// `features/V3/message_consumer.feature` -> "v3".
fn feature_version(feature: &cucumber::gherkin::Feature) -> String {
  feature.path.as_ref().and_then(|p| p.parent()?.file_name()?.to_str().map(str::to_lowercase)).unwrap_or_default()
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  MessageWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|feature, _rule, scenario, world| Box::pin(async move {
      trace::reset();
      world.version = feature_version(feature);
      world.scenario_id = scenario.name.clone();
      let _ = fs::remove_dir_all(world.pact_dir());
    }))
    .after(|feature, _rule, scenario, _ev, world| Box::pin(async move {
      if let Some(world) = world {
        world.shutdown();
      }
      let path = trace_path(&format!("{} {}", feature_version(feature).to_uppercase(), feature.name), &scenario.name);
      if let Err(e) = trace::write(&path, &feature.name, &scenario.name) {
        eprintln!("could not write trace {}: {e}", path.display());
      }
    }))
    .filter_run_and_exit(model::FEATURES, |feature, _rule, scenario| {
      let file = feature.path.as_ref().and_then(|p| p.file_name()?.to_str().map(str::to_string)).unwrap_or_default();
      matches!((feature_version(feature).as_str(), file.as_str()),
        ("v3", "message_consumer.feature")
        | ("v4", "message_consumer.feature" | "synchronous_message_consumer.feature" | "v4.feature"))
        && !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
