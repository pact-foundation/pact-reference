//! V1 to V4 HTTP consumer features, driven only through pact_ffi. Mirrors `v1_consumer.rs`, `v2_consumer.rs` and
//! `shared_steps/consumer.rs` in the pact-reference compatibility suite.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use serde_json::Value;

use pact_ffi_compat::ffi::{self, PactHandle, PactSpecification, PactSpecification_V1, PactSpecification_V2,
  PactSpecification_V3, PactSpecification_V4, InteractionHandle};
use pact_ffi_compat::model::{self, Interaction, FIXTURES};
use pact_ffi_compat::trace;

#[derive(Debug, Default)]
struct Response {
  status: u16,
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

#[derive(Debug, Default, World)]
struct ConsumerWorld {
  /// "v1" to "v4", from the feature's directory.
  version: String,
  scenario_id: String,
  interactions: Vec<Interaction>,
  pact: Option<PactHandle>,
  port: Option<i32>,
  /// Captured when the pact test is done, after which the mock server is gone.
  mismatches: Option<Vec<Value>>,
  response: Response,
  pact_json: Value,
  /// The single interaction the V3/V4 "being defined for a consumer test" scenarios build up.
  interaction: Option<InteractionHandle>,
}

impl ConsumerWorld {
  fn spec(&self) -> PactSpecification {
    match self.version.as_str() {
      "v4" => PactSpecification_V4,
      "v3" => PactSpecification_V3,
      "v2" => PactSpecification_V2,
      _ => PactSpecification_V1,
    }
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

  fn mismatches(&self) -> Vec<Value> {
    if let Some(m) = &self.mismatches {
      return m.clone();
    }
    let port = self.port.expect("mock server not started");
    ffi::mock_server_mismatches(port).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
  }

  fn request_mismatches(&self) -> Vec<Value> {
    self.mismatches().iter()
      .filter(|m| m["type"] == "request-mismatch")
      .flat_map(|m| m["mismatches"].as_array().cloned().unwrap_or_default())
      .collect()
  }

  fn shutdown(&mut self) {
    if let Some(port) = self.port.take() {
      ffi::cleanup_mock_server(port);
    }
    if let Some(pact) = self.pact.take() {
      ffi::free_pact_handle(pact);
    }
  }

  fn interaction(&self, n: usize) -> &Interaction {
    &self.interactions[n - 1]
  }

  fn pact_interaction(&self, nth: &str) -> anyhow::Result<&Value> {
    let i = match nth { "first" => 0, "second" => 1, "third" => 2, _ => bail!("bad ordinal {nth}") };
    self.pact_json["interactions"].get(i).ok_or_else(|| anyhow!("no {nth} interaction in the pact file"))
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

fn start(world: &mut ConsumerWorld, interactions: &[Interaction]) -> anyhow::Result<()> {
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  for i in interactions {
    model::register(pact, i);
  }
  world.pact = Some(pact);
  let port = ffi::create_mock_server_for_transport(pact, "::1", 0, "http");
  if port < 0 {
    bail!("pactffi_create_mock_server_for_transport failed with {port}");
  }
  world.port = Some(port);
  Ok(())
}

async fn send(world: &mut ConsumerWorld, method: &str, path: &str, query: &str, headers: &[(String, String)],
              body: Option<&[u8]>) -> anyhow::Result<()> {
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let mut url = format!("http://[::1]:{port}{path}");
  if !query.is_empty() {
    url.push('?');
    url.push_str(query);
  }
  let mut req = reqwest::Client::new().request(method.parse()?, &url);
  for (k, v) in headers {
    req = req.header(k, v);
  }
  if let Some(body) = body {
    req = req.body(body.to_vec());
  }
  let res = req.send().await?;
  let status = res.status().as_u16();
  let headers = res.headers().iter()
    .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
    .collect();
  let body = res.bytes().await?.to_vec();
  trace::note(format!("HTTP {method} {url} -> {status}"));
  world.response = Response { status, headers, body };
  Ok(())
}

#[given("the following HTTP interactions have been defined:")]
fn interactions_defined(world: &mut ConsumerWorld, step: &Step) {
  traced(step);
  world.interactions = model::setup_common_interactions(&table(step));
}

#[when(expr = "the mock server is started with interaction {int}")]
fn started_with_interaction(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let i = world.interaction(n).clone();
  start(world, &[i])
}

/// Applies a `| method | path | query | headers | body |` changes table. `raw headers` are
/// added on top of `headers` rather than replacing them, to send repeated headers.
fn apply_changes(i: &mut Interaction, rows: &[Vec<String>]) {
  let mut raw_headers = vec![];
  for (field, value) in rows[0].iter().zip(&rows[1]) {
    match field.as_str() {
      "method" => i.method = value.clone(),
      "path" => i.path = value.clone(),
      "query" => i.query = value.clone(),
      "headers" => i.request.headers = model::parse_headers(value),
      "body" => model::setup_body(value, &mut i.request),
      "raw headers" => raw_headers = model::parse_headers(value),
      "matching rules" => i.request.matching_rules = model::matching_rules(value),
      _ => {}
    }
  }
  i.request.headers.extend(raw_headers);
}

#[when(expr = "the mock server is started with interaction {int} but with the following changes:")]
fn started_with_changes(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let mut i = world.interaction(n).clone();
  apply_changes(&mut i, &table(step));
  start(world, &[i])
}

#[when(expr = "the mock server is started with interactions {string}")]
fn started_with_interactions(world: &mut ConsumerWorld, step: &Step, list: String) -> anyhow::Result<()> {
  traced(step);
  let numbers = list.split(',').map(|n| n.trim().parse()).collect::<Result<Vec<usize>, _>>()?;
  let interactions: Vec<Interaction> = numbers.iter().map(|n| world.interaction(*n).clone()).collect();
  start(world, &interactions)
}

#[when(expr = "request {int} is made to the mock server")]
async fn request_made(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let i = world.interaction(n).clone();
  send(world, &i.method, &i.path, &i.query, &i.request.headers, i.request.body.as_deref()).await
}

#[when(expr = "request {int} is made to the mock server with the following changes:")]
async fn request_made_with_changes(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let mut i = world.interaction(n).clone();
  apply_changes(&mut i, &table(step));
  send(world, &i.method, &i.path, &i.query, &i.request.headers, i.request.body.as_deref()).await
}

#[then(expr = "a {int} success response is returned")]
fn success_response(world: &mut ConsumerWorld, step: &Step, status: u16) -> anyhow::Result<()> {
  traced(step);
  if world.response.status != status {
    bail!("Expected a success response of {status} but got {:?}", world.response);
  }
  Ok(())
}

#[then(expr = "a {int} error response is returned")]
fn error_response(world: &mut ConsumerWorld, step: &Step, status: u16) -> anyhow::Result<()> {
  traced(step);
  if world.response.status != status {
    bail!("Expected an error response of {status} but got {:?}", world.response);
  }
  Ok(())
}

#[then(expr = "the payload will contain the {string} JSON document")]
fn payload_json(world: &mut ConsumerWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  // pactffi_with_body parses JSON bodies (to pull out matchers), so the mock server returns them
  // re-serialised. Through the FFI only the JSON value survives, not the fixture's formatting.
  let expected = fs::read(format!("{FIXTURES}/{name}.json"))?;
  let same = world.response.body == expected
    || serde_json::from_slice::<Value>(&world.response.body).ok() == serde_json::from_slice::<Value>(&expected).ok();
  if !same {
    bail!("Expected payload {} but got {}", String::from_utf8_lossy(&expected),
      String::from_utf8_lossy(&world.response.body));
  }
  Ok(())
}

#[then(expr = "the content type will be set as {string}")]
fn content_type_set(world: &mut ConsumerWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  match world.response.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
    Some((_, v)) if *v == expected => Ok(()),
    Some((_, v)) => bail!("Expected response content-type of '{expected}' but was '{v}'"),
    None => bail!("Response does not contain a content-type header"),
  }
}

#[when("the pact test is done")]
fn pact_test_done(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let mismatches = world.mismatches();
  ffi::mock_server_matched(port);
  if mismatches.is_empty() {
    let dir = world.pact_dir();
    fs::create_dir_all(&dir)?;
    let rc = ffi::write_pact_file(port, &dir.to_string_lossy(), true);
    if rc != 0 {
      bail!("pactffi_write_pact_file failed with {rc}");
    }
  }
  world.mismatches = Some(mismatches);
  world.shutdown();
  Ok(())
}

#[then(regex = r"^the mock server will write out a Pact file for the interactions? when done$")]
fn pact_written(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let file = world.pact_file();
  if !file.exists() {
    bail!("No pact file found: {}", file.display());
  }
  world.pact_json = serde_json::from_str(&fs::read_to_string(file)?)?;
  let expected = if world.version == "v2" { "2.0.0" } else { "1.0.0" };
  let version = &world.pact_json["metadata"]["pactSpecification"]["version"];
  if version != expected {
    bail!("Expected a {expected} pact file but metadata.pactSpecification.version was {version}");
  }
  Ok(())
}

#[then(regex = r"^the mock server will NOT write out a Pact file for the interactions? when done$")]
fn pact_not_written(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let file = world.pact_file();
  if file.exists() {
    bail!("Expected no pact file, but found {}", file.display());
  }
  Ok(())
}

#[then("the mock server status will be OK")]
fn status_ok(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let m = world.mismatches();
  if !m.is_empty() {
    bail!("Mock server has {} mismatches: {}", m.len(), Value::from(m));
  }
  Ok(())
}

#[then(regex = r"^the mock server status will (?:NOT be OK|be mismatches)$")]
fn status_not_ok(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.mismatches().is_empty() {
    bail!("Mock server has no mismatches");
  }
  Ok(())
}

#[then(regex = r"^the pact file will contain \{(\d+)\} interactions?$")]
fn pact_interaction_count(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let count = world.pact_json["interactions"].as_array().map(Vec::len).unwrap_or(0);
  if count != n {
    bail!("Expected the pact file to contain {n} interaction(s), but had {count}");
  }
  Ok(())
}

#[then(regex = r#"^the \{(first|second|third)\} interaction request will be for a "(\w+)"$"#)]
fn pact_request_method(world: &mut ConsumerWorld, step: &Step, nth: String, method: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &world.pact_interaction(&nth)?["request"]["method"];
  if !actual.as_str().is_some_and(|m| m.eq_ignore_ascii_case(&method)) {
    bail!("Expected request method {method} but was {actual}");
  }
  Ok(())
}

/// A V1 pact stores the query as a string; later formats use a map of name to values.
fn pact_query(query: &Value) -> HashMap<String, Vec<String>> {
  match query {
    Value::String(s) => model::query_map(&model::parse_query(s)),
    Value::Object(map) => map.iter().map(|(k, v)| {
      let values = match v {
        Value::Array(a) => a.iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect(),
        other => vec![other.as_str().unwrap_or_default().to_string()],
      };
      (k.clone(), values)
    }).collect(),
    _ => HashMap::new(),
  }
}

#[then(regex = r#"^the \{(first|second|third)\} interaction request query parameters will be "(.*)"$"#)]
fn pact_request_query(world: &mut ConsumerWorld, step: &Step, nth: String, query: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &world.pact_interaction(&nth)?["request"]["query"];
  if pact_query(actual) != model::query_map(&model::parse_query(&query)) {
    bail!("Expected query {query} but was {actual}");
  }
  Ok(())
}

fn pact_header<'a>(part: &'a Value, name: &str) -> Option<&'a str> {
  part["headers"].as_object()?.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).and_then(|(_, v)| v.as_str())
}

#[then(regex = r#"^the \{(first|second|third)\} interaction request will contain the header "(.*)" with value "(.*)"$"#)]
fn pact_request_header(world: &mut ConsumerWorld, step: &Step, nth: String, name: String, value: String)
  -> anyhow::Result<()> {
  traced(step);
  let request = &world.pact_interaction(&nth)?["request"];
  match pact_header(request, &name) {
    Some(v) if v == value => Ok(()),
    other => bail!("Expected header {name}: {value} but was {other:?}"),
  }
}

#[then(regex = r#"^the \{(first|second|third)\} interaction request content type will be "(.*)"$"#)]
fn pact_request_content_type(world: &mut ConsumerWorld, step: &Step, nth: String, ct: String) -> anyhow::Result<()> {
  traced(step);
  let request = &world.pact_interaction(&nth)?["request"];
  let actual = pact_header(request, "content-type").or_else(|| match request["body"] {
    Value::Object(_) | Value::Array(_) => Some("application/json"),
    _ => None,
  });
  if actual != Some(ct.as_str()) {
    bail!("Expected request content type {ct} but was {actual:?}");
  }
  Ok(())
}

fn compare_body(body: &Value, fixture: &str) -> anyhow::Result<()> {
  let path = Path::new(FIXTURES).join(fixture);
  if fixture.ends_with(".json") {
    let expected: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    if *body != expected {
      bail!("Expected body {expected} but was {body}");
    }
  } else {
    let expected = fs::read_to_string(&path)?;
    if body.as_str() != Some(expected.as_str()) {
      bail!("Expected body {expected:?} but was {body}");
    }
  }
  Ok(())
}

#[then(regex = r#"^the \{(first|second|third)\} interaction (request|response) will contain the "(.*)" document$"#)]
fn pact_body(world: &mut ConsumerWorld, step: &Step, nth: String, part: String, fixture: String)
  -> anyhow::Result<()> {
  traced(step);
  compare_body(&world.pact_interaction(&nth)?[part.as_str()]["body"], &fixture)
}

fn mismatch_matches(m: &Value, kind: &str, error: &str) -> bool {
  let t = m["type"].as_str().unwrap_or_default();
  let correct_type = if t == "BodyTypeMismatch" { kind == "body-content-type" } else { t.to_lowercase().starts_with(kind) };
  correct_type && m["mismatch"].as_str().is_some_and(|d| d.contains(error))
}

#[then(expr = "the mismatches will contain a {string} mismatch with error {string}")]
fn mismatch_with_error(world: &mut ConsumerWorld, step: &Step, kind: String, error: String) -> anyhow::Result<()> {
  traced(step);
  let all = world.request_mismatches();
  if !all.iter().any(|m| mismatch_matches(m, &kind, &error)) {
    bail!("Did not find a {kind} mismatch with error {error:?} in {}", Value::from(all));
  }
  Ok(())
}

#[then(expr = "the mismatches will contain a {string} mismatch with path {string} with error {string}")]
fn mismatch_with_path(world: &mut ConsumerWorld, step: &Step, kind: String, path: String, error: String)
  -> anyhow::Result<()> {
  traced(step);
  let all = world.request_mismatches();
  let found = all.iter().any(|m| {
    let actual_path = match m["type"].as_str().unwrap_or_default() {
      "QueryMismatch" => &m["parameter"],
      "HeaderMismatch" => &m["key"],
      "BodyMismatch" => &m["path"],
      _ => &Value::Null,
    };
    mismatch_matches(m, &kind, &error) && actual_path.as_str() == Some(path.as_str())
  });
  if !found {
    bail!("Did not find a {kind} mismatch at {path} with error {error:?} in {}", Value::from(all));
  }
  Ok(())
}

#[then(regex = r#"^the mock server status will be an unexpected "(\w+)" request received error for interaction \{(\d+)\}$"#)]
fn unexpected_request_for_interaction(world: &mut ConsumerWorld, step: &Step, method: String, n: usize)
  -> anyhow::Result<()> {
  traced(step);
  let i = world.interaction(n).clone();
  let expected_query = model::query_map(&model::parse_query(&i.query));
  let all = world.mismatches();
  let found = all.iter().any(|m| {
    m["type"] == "request-not-found" && m["method"] == method.as_str() && m["path"] == i.path.as_str()
      && pact_query(&m["request"]["query"]) == expected_query
  });
  if !found {
    bail!("Did not find an unexpected {method} request for interaction {n} in {}", Value::from(all));
  }
  Ok(())
}

#[then(regex = r#"^the mock server status will be an unexpected "(\w+)" request received error for path "(.*)"$"#)]
fn unexpected_request_for_path(world: &mut ConsumerWorld, step: &Step, method: String, path: String)
  -> anyhow::Result<()> {
  traced(step);
  let all = world.mismatches();
  if !all.iter().any(|m| m["type"] == "request-not-found" && m["method"] == method.as_str() && m["path"] == path.as_str()) {
    bail!("Did not find an unexpected {method} request for {path} in {}", Value::from(all));
  }
  Ok(())
}

#[then(regex = r"^the mock server status will be an expected but not received error for interaction \{(\d+)\}$")]
fn missing_request(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let i = world.interaction(n).clone();
  let all = world.mismatches();
  if !all.iter().any(|m| m["type"] == "missing-request" && m["method"] == i.method.as_str() && m["path"] == i.path.as_str()) {
    bail!("Did not find a missing request for interaction {n} in {}", Value::from(all));
  }
  Ok(())
}

// V3 and V4: interactions defined without a mock server, then written straight from the pact handle.

#[given(regex = r"^an (integration|HTTP interaction) is being defined for a consumer test$")]
fn interaction_being_defined(world: &mut ConsumerWorld, step: &Step, _kind: String) {
  traced(step);
  let pact = ffi::new_pact(&world.consumer(), "p");
  ffi::with_specification(pact, world.spec());
  let i = ffi::new_interaction(pact, "interaction for a consumer test");
  ffi::with_request(i, "GET", "/");
  world.pact = Some(pact);
  world.interaction = Some(i);
}

fn defined(world: &ConsumerWorld) -> anyhow::Result<InteractionHandle> {
  world.interaction.ok_or_else(|| anyhow!("no interaction is being defined"))
}

#[given(expr = "a provider state {string} is specified")]
fn provider_state(world: &mut ConsumerWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::given(defined(world)?, &state) {
    bail!("pactffi_given failed");
  }
  Ok(())
}

/// The table's first row names the parameters; the second holds their values as JSON.
#[given(expr = "a provider state {string} is specified with the following data:")]
fn provider_state_with_data(world: &mut ConsumerWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let params: serde_json::Map<String, Value> = rows[0].iter().zip(&rows[1])
    .map(|(k, v)| Ok((k.clone(), serde_json::from_str(v)?)))
    .collect::<anyhow::Result<_>>()?;
  let rc = ffi::given_with_params(defined(world)?, &state, &Value::Object(params).to_string());
  if rc != 0 {
    bail!("pactffi_given_with_params failed with {rc}");
  }
  Ok(())
}

#[given(expr = "a key of {string} is specified for the HTTP interaction")]
fn key_specified(world: &mut ConsumerWorld, step: &Step, key: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::set_key(defined(world)?, &key) {
    bail!("pactffi_set_key failed");
  }
  Ok(())
}

#[given("the HTTP interaction is marked as pending")]
fn marked_pending(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if !ffi::set_pending(defined(world)?, true) {
    bail!("pactffi_set_pending failed");
  }
  Ok(())
}

#[given(expr = "a comment {string} is added to the HTTP interaction")]
fn comment_added(world: &mut ConsumerWorld, step: &Step, comment: String) -> anyhow::Result<()> {
  traced(step);
  if !ffi::add_text_comment(defined(world)?, &comment) {
    bail!("pactffi_add_text_comment failed");
  }
  Ok(())
}

#[when("the Pact file for the test is generated")]
fn pact_generated(world: &mut ConsumerWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let pact = world.pact.ok_or_else(|| anyhow!("no pact is being defined"))?;
  let dir = world.pact_dir();
  fs::create_dir_all(&dir)?;
  let rc = ffi::pact_handle_write_file(pact, &dir.to_string_lossy(), true);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  world.pact_json = serde_json::from_str(&fs::read_to_string(world.pact_file())?)?;
  Ok(())
}

fn provider_states(world: &ConsumerWorld) -> Vec<Value> {
  world.pact_json["interactions"][0]["providerStates"].as_array().cloned().unwrap_or_default()
}

#[then(regex = r"^the interaction in the Pact file will contain (\d+) provider states?$")]
fn provider_state_count(world: &mut ConsumerWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let states = provider_states(world);
  if states.len() != n {
    bail!("Expected {n} provider states but found {}", Value::from(states));
  }
  Ok(())
}

#[then(expr = "the interaction in the Pact file will contain provider state {string}")]
fn provider_state_present(world: &mut ConsumerWorld, step: &Step, state: String) -> anyhow::Result<()> {
  traced(step);
  let states = provider_states(world);
  if !states.iter().any(|s| s["name"] == state.as_str()) {
    bail!("No provider state '{state}' in {}", Value::from(states));
  }
  Ok(())
}

#[then(expr = "the provider state {string} in the Pact file will contain the following parameters:")]
fn provider_state_params(world: &mut ConsumerWorld, step: &Step, state: String) -> anyhow::Result<()> {
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

#[then(expr = "the first interaction in the Pact file will have a type of {string}")]
fn interaction_type(world: &mut ConsumerWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &world.pact_interaction("first")?["type"];
  if actual != expected.as_str() {
    bail!("Expected type '{expected}' but was {actual}");
  }
  Ok(())
}

/// The expected value is JSON in single quotes.
#[then(regex = r#"^the first interaction in the Pact file will have "([^"]+)" = '(.*)'$"#)]
fn interaction_attribute(world: &mut ConsumerWorld, step: &Step, name: String, value: String) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(&value)?;
  let actual = &world.pact_interaction("first")?[name.as_str()];
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

/// `features/V2/http_consumer.feature` -> "v2".
fn feature_version(feature: &cucumber::gherkin::Feature) -> String {
  feature.path.as_ref().and_then(|p| p.parent()?.file_name()?.to_str().map(str::to_lowercase)).unwrap_or_default()
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  ConsumerWorld::cucumber()
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
      matches!(feature_version(feature).as_str(), "v1" | "v2" | "v3" | "v4")
        && feature.path.as_ref().is_some_and(|p| p.ends_with("http_consumer.feature"))
        && feature.tags.iter().any(|t| t == "consumer") && !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
