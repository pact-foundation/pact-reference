//! V4 plugin features (`features/V4/plugins`), driven only through pact_ffi: `pactffi_using_plugin`,
//! `pactffi_interaction_contents` and `pactffi_cleanup_plugins` for the consumer side, and the verifier, which loads
//! the plugins a pact file lists, for the provider side.
//!
//! The plugins come from `~/.pact/plugins` (or `PACT_PLUGIN_DIR`). The suite root is `PACT_COMPAT_SUITE`, so the
//! features can be run from a suite checkout before they reach the copy `compatibility-suite` uses.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bytes::Bytes;
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use pact_ffi_compat::ffi::{self, InteractionHandle, InteractionPart, PactHandle};
use pact_ffi_compat::ffi::{InteractionPart_Request, InteractionPart_Response, PactSpecification_V4};
use pact_ffi_compat::model;
use pact_ffi_compat::server::{Request, Response, Server};
use pact_ffi_compat::trace;

#[derive(Debug, Default, World)]
struct PluginWorld {
  scenario_id: String,
  /// Plugin versions from the "is installed" steps, used when a step loads a plugin without naming a version.
  versions: HashMap<String, String>,
  pact: Option<PactHandle>,
  /// Plugins this pact has loaded.
  loaded: Vec<String>,
  /// The interactions defined so far; the last is the one the steps add to.
  interactions: Vec<InteractionHandle>,
  plugin_result: Option<u32>,
  plugin_error: Option<String>,
  port: Option<i32>,
  matched: Option<bool>,
  mismatches: Vec<Value>,
  status: u16,
  body: Vec<u8>,
  pact_json: Value,
  provider: Option<Server>,
  pact_files: Vec<PathBuf>,
  exit: Option<i32>,
  results: Value,
  output: String,
  /// The line the matt mock server sent back.
  tcp_reply: Option<String>,
  /// The gRPC call's status (`grpc-status`, from the headers or trailers) and its message, without the frame prefix.
  grpc_reply: Option<(String, Vec<u8>)>,
  /// The port a TCP or gRPC provider listens on, or a closed one for "no provider is listening".
  provider_port: Option<u16>,
  tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl PluginWorld {
  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite/plugins").join(&self.scenario_id)
  }

  fn pact(&self) -> anyhow::Result<PactHandle> {
    self.pact.ok_or_else(|| anyhow!("no consumer test is being defined"))
  }

  fn interaction(&self) -> anyhow::Result<InteractionHandle> {
    self.interactions.last().copied().ok_or_else(|| anyhow!("no interaction is being defined"))
  }

  fn load_plugin(&mut self, name: &str, version: Option<&str>) -> anyhow::Result<u32> {
    let rc = ffi::using_plugin(self.pact()?, name, version);
    self.plugin_result = Some(rc);
    if rc == 0 {
      self.loaded.push(name.to_string());
    } else {
      self.plugin_error = ffi::last_error();
    }
    Ok(rc)
  }

  fn first_interaction(&self) -> &Value {
    &self.pact_json["interactions"][0]
  }

  fn shutdown(&mut self) {
    for task in self.tasks.drain(..) {
      task.abort();
    }
    if let Some(port) = self.port.take() {
      ffi::cleanup_mock_server(port);
    }
    if let Some(pact) = self.pact.take() {
      ffi::cleanup_plugins(pact);
      ffi::free_pact_handle(pact);
    }
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

/// The suite checkout: `PACT_COMPAT_SUITE`, or the copy `compatibility-suite` runs.
fn suite_root() -> PathBuf {
  env::var("PACT_COMPAT_SUITE").map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("../compatibility-suite/pact-compatibility-suite"))
}

fn fixtures() -> PathBuf {
  suite_root().join("fixtures")
}

fn fixture(name: &str) -> anyhow::Result<Vec<u8>> {
  let path = fixtures().join(name);
  fs::read(&path).map_err(|e| anyhow!("fixture {}: {e}", path.display()))
}

/// A body cell: `file: <fixture>` or the literal body.
fn body_cell(cell: &str) -> anyhow::Result<Vec<u8>> {
  match cell.strip_prefix("file:") {
    Some(file) => fixture(file.trim()),
    None => Ok(cell.as_bytes().to_vec()),
  }
}

/// Plugin configuration refers to fixtures as `fixture:<path>`; the plugin needs the absolute path.
fn plugin_contents(text: &str) -> String {
  text.replace("\"fixture:", &format!("\"{}/", fixtures().display()))
}

fn plugin_dir() -> PathBuf {
  env::var("PACT_PLUGIN_DIR").map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from(env::var("HOME").unwrap_or_default()).join(".pact/plugins"))
}

fn installed(name: &str) -> Vec<String> {
  fs::read_dir(plugin_dir()).map(|dir| dir.filter_map(|e| e.ok())
    .filter_map(|e| e.file_name().to_str().map(str::to_string))
    .filter(|f| f.strip_prefix(name).is_some_and(|v| v.starts_with('-')))
    .collect()).unwrap_or_default()
}

fn plugin_running(name: &str) -> bool {
  let pattern = format!("{}/{name}-", plugin_dir().display());
  Command::new("pgrep").args(["-f", &pattern]).output().map(|o| o.status.success()).unwrap_or(false)
}

fn any_string_contains(value: &Value, text: &str) -> bool {
  match value {
    Value::String(s) => s.contains(text),
    Value::Array(a) => a.iter().any(|v| any_string_contains(v, text)),
    Value::Object(o) => o.values().any(|v| any_string_contains(v, text)),
    _ => false,
  }
}

fn read_pact(world: &mut PluginWorld) -> anyhow::Result<()> {
  let dir = world.pact_dir();
  let file = dir.join("c-p.json");
  world.pact_json = serde_json::from_str(&fs::read_to_string(&file).map_err(|e| anyhow!("{}: {e}", file.display()))?)?;
  Ok(())
}

// ---- Consumer ----

#[given(regex = r#"^the "(.*)" plugin is not installed$"#)]
fn not_installed(_world: &mut PluginWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  let found = installed(&name);
  if !found.is_empty() {
    bail!("plugin {name} is installed: {found:?}");
  }
  Ok(())
}

#[given(regex = r#"^the "(.*)" plugin version "(.*)" is installed$"#)]
fn installed_version(world: &mut PluginWorld, step: &Step, name: String, version: String) -> anyhow::Result<()> {
  traced(step);
  let dir = format!("{name}-{version}");
  if !installed(&name).contains(&dir) {
    bail!("plugin {dir} is not installed in {}", plugin_dir().display());
  }
  world.versions.insert(name, version);
  Ok(())
}

#[given("an HTTP interaction is being defined for a consumer test")]
fn interaction_defined(world: &mut PluginWorld, step: &Step) {
  traced(step);
  let pact = new_v4_pact(world);
  world.interactions.push(ffi::new_interaction(pact, "plugin interaction 1"));
}

#[given("another HTTP interaction is being defined for the same consumer test")]
fn another_interaction(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let n = world.interactions.len() + 1;
  world.interactions.push(ffi::new_interaction(world.pact()?, &format!("plugin interaction {n}")));
  Ok(())
}

#[when(regex = r#"^the "(.*)" plugin is loaded for the test$"#)]
fn plugin_loaded(world: &mut PluginWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  world.load_plugin(&name, None).map(|_| ())
}

#[when(regex = r#"^the "(.*)" plugin version "(.*)" is loaded for the test$"#)]
fn plugin_version_loaded(world: &mut PluginWorld, step: &Step, name: String, version: String) -> anyhow::Result<()> {
  traced(step);
  world.load_plugin(&name, Some(&version)).map(|_| ())
}

#[then(regex = r#"^loading the plugin will fail with an error that mentions "(.*)"$"#)]
fn plugin_failed(world: &mut PluginWorld, step: &Step, text: String) -> anyhow::Result<()> {
  traced(step);
  match (world.plugin_result, &world.plugin_error) {
    (Some(0), _) | (None, _) => bail!("the plugin loaded"),
    (Some(_), Some(e)) if e.contains(&text) => Ok(()),
    (Some(rc), e) => bail!("pactffi_using_plugin returned {rc}, but the error does not mention \"{text}\": {e:?}"),
  }
}

#[then("the test will not have panicked")]
fn not_panicked(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.plugin_result == Some(1) {
    bail!("pactffi_using_plugin panicked: {:?}", world.plugin_error);
  }
  Ok(())
}

#[given(regex = r#"^the HTTP request is "(.*)" "(.*)"$"#)]
fn request_is(world: &mut PluginWorld, step: &Step, method: String, path: String) -> anyhow::Result<()> {
  traced(step);
  ffi::with_request(world.interaction()?, &method, &path);
  Ok(())
}

fn set_plugin_body(world: &mut PluginWorld, step: &Step, p: InteractionPart, name: &str, content_type: &str)
  -> anyhow::Result<()> {
  if !world.loaded.iter().any(|l| l == name) {
    let version = world.versions.get(name).cloned();
    let rc = world.load_plugin(name, version.as_deref())?;
    if rc != 0 {
      bail!("pactffi_using_plugin returned {rc}: {:?}", world.plugin_error);
    }
  }
  let contents = plugin_contents(step.docstring.as_deref().ok_or_else(|| anyhow!("no docstring"))?);
  let rc = ffi::interaction_contents(world.interaction()?, p, content_type, &contents);
  if rc != 0 {
    bail!("pactffi_interaction_contents returned {rc}");
  }
  Ok(())
}

#[given(regex = r#"^the HTTP request body is set with the "(.*)" plugin for "(.*)" with:$"#)]
fn request_plugin_body(world: &mut PluginWorld, step: &Step, name: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  set_plugin_body(world, step, InteractionPart_Request, &name, &content_type)
}

#[given(regex = r#"^the HTTP response body is set with the "(.*)" plugin for "(.*)" with:$"#)]
fn response_plugin_body(world: &mut PluginWorld, step: &Step, name: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  set_plugin_body(world, step, InteractionPart_Response, &name, &content_type)
}

#[given(regex = r#"^the HTTP response body is "(.*)" with content type "(.*)"$"#)]
fn response_body(world: &mut PluginWorld, step: &Step, body: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  let body = body_cell(&body)?;
  ffi::with_body(world.interaction()?, InteractionPart_Response, &content_type, std::str::from_utf8(&body)?);
  Ok(())
}

fn new_v4_pact(world: &mut PluginWorld) -> PactHandle {
  let pact = ffi::new_pact("c", "p");
  ffi::with_specification(pact, PactSpecification_V4);
  world.pact = Some(pact);
  pact
}

#[given("a message interaction is being defined for a consumer test")]
fn message_defined(world: &mut PluginWorld, step: &Step) {
  traced(step);
  let pact = new_v4_pact(world);
  world.interactions.push(ffi::new_message_interaction(pact, "plugin message 1"));
}

#[given("a synchronous message interaction is being defined for a consumer test")]
fn sync_message_defined(world: &mut PluginWorld, step: &Step) {
  traced(step);
  let pact = new_v4_pact(world);
  world.interactions.push(ffi::new_sync_message_interaction(pact, "plugin message 1"));
}

/// The plugin's configuration gives the request and, for a synchronous message, the response, so this is set once,
/// on the request part.
#[given(regex = r#"^the (?:synchronous )?message contents are set with the "(.*)" plugin for "(.*)" with:$"#)]
fn message_plugin_contents(world: &mut PluginWorld, step: &Step, name: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  set_plugin_body(world, step, InteractionPart_Request, &name, &content_type)
}

#[when(regex = r"^the mock server is started with (all the interactions|the plugin interaction)$")]
fn mock_server_started(world: &mut PluginWorld, step: &Step, _which: String) -> anyhow::Result<()> {
  traced(step);
  let port = ffi::create_mock_server_for_transport(world.pact()?, "127.0.0.1", 0, "http");
  if port < 0 {
    bail!("pactffi_create_mock_server_for_transport failed with {port}");
  }
  world.port = Some(port);
  Ok(())
}

async fn send(world: &mut PluginWorld, method: &str, path: &str, body: Option<(&str, &str)>) -> anyhow::Result<()> {
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let url = format!("http://127.0.0.1:{port}{path}");
  let mut req = reqwest::Client::new().request(method.parse()?, &url);
  if let Some((content_type, body)) = body {
    req = req.header("content-type", content_type).body(body.to_string());
  }
  let res = req.send().await?;
  world.status = res.status().as_u16();
  world.body = res.bytes().await?.to_vec();
  trace::note(format!("HTTP {method} {url} -> {}", world.status));
  Ok(())
}

#[when(regex = r#"^a "(.*)" request is made to "(.*)" with content type "(.*)" and body "(.*)"$"#)]
async fn request_with_body(world: &mut PluginWorld, step: &Step, method: String, path: String, content_type: String,
                           body: String) -> anyhow::Result<()> {
  traced(step);
  send(world, &method, &path, Some((&content_type, &body))).await
}

#[when(regex = r#"^a "([A-Z]+)" request is made to "([^"]*)"$"#)]
async fn request_made(world: &mut PluginWorld, step: &Step, method: String, path: String) -> anyhow::Result<()> {
  traced(step);
  send(world, &method, &path, None).await
}

#[then("a 200 success response is returned")]
fn success_response(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.status != 200 {
    bail!("expected a 200 response, got {}: {}", world.status, String::from_utf8_lossy(&world.body));
  }
  Ok(())
}

#[then(regex = r#"^the response body will be "(.*)"$"#)]
fn response_body_is(world: &mut PluginWorld, step: &Step, body: String) -> anyhow::Result<()> {
  traced(step);
  let expected = body_cell(&body)?;
  // The mock server re-serialises JSON bodies, so JSON is compared as values
  let same_json = serde_json::from_slice::<Value>(&expected).ok()
    .zip(serde_json::from_slice::<Value>(&world.body).ok()).is_some_and(|(a, b)| a == b);
  if world.body != expected && !same_json {
    bail!("expected body {:?}, got {:?}", String::from_utf8_lossy(&expected), String::from_utf8_lossy(&world.body));
  }
  Ok(())
}

#[when("the pact test is done")]
fn pact_test_done(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.take().ok_or_else(|| anyhow!("mock server not started"))?;
  world.matched = Some(ffi::mock_server_matched(port));
  world.mismatches = ffi::mock_server_mismatches(port).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
  if world.matched == Some(true) {
    let dir = world.pact_dir();
    let rc = ffi::write_pact_file(port, dir.to_str().unwrap(), false);
    if rc != 0 {
      ffi::cleanup_mock_server(port);
      bail!("pactffi_write_pact_file failed with {rc}");
    }
    read_pact(world)?;
  }
  ffi::cleanup_mock_server(port);
  Ok(())
}

#[then("the mock server status will be OK")]
fn status_ok(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.matched != Some(true) {
    bail!("mock server did not match: {:?}", world.mismatches);
  }
  Ok(())
}

#[then("the mock server status will be mismatches")]
fn status_mismatches(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.matched != Some(false) {
    bail!("mock server had no mismatches");
  }
  Ok(())
}

#[then(regex = r#"^the mismatches will contain a mismatch with error "(.*)"$"#)]
fn mismatch_with_error(world: &mut PluginWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  if !world.mismatches.iter().any(|m| any_string_contains(m, &error)) {
    bail!("no mismatch with \"{error}\": {:?}", world.mismatches);
  }
  Ok(())
}

#[when("the Pact file for the test is generated")]
fn pact_generated(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let dir = world.pact_dir();
  let rc = ffi::pact_handle_write_file(world.pact()?, dir.to_str().unwrap(), false);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  read_pact(world)
}

#[when("the plugins are cleaned up")]
fn plugins_cleaned_up(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let pact = world.pact.take().ok_or_else(|| anyhow!("no consumer test is being defined"))?;
  ffi::cleanup_plugins(pact);
  ffi::free_pact_handle(pact);
  Ok(())
}

/// Plugin processes are stopped asynchronously, so this waits a little for them to go.
#[then(regex = r#"^no "(.*)" plugin process will still be running$"#)]
async fn no_plugin_process(_world: &mut PluginWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  let start = Instant::now();
  while plugin_running(&name) {
    if start.elapsed() > Duration::from_secs(10) {
      bail!("a {name} plugin process is still running");
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
  }
  Ok(())
}

#[then(regex = r"^the pact file will contain (\d+) interactions$")]
fn pact_interactions(world: &mut PluginWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let count = world.pact_json["interactions"].as_array().map(Vec::len).unwrap_or(0);
  if count != n {
    bail!("expected {n} interactions, the pact file has {count}");
  }
  Ok(())
}

#[then("the second interaction in the pact file will not have plugin configuration")]
fn no_plugin_configuration(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let config = &world.pact_json["interactions"][1]["pluginConfiguration"];
  if config.as_object().is_some_and(|o| !o.is_empty()) {
    bail!("the second interaction has plugin configuration: {config}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file will have plugin configuration for "(.*)"$"#)]
fn has_plugin_configuration(world: &mut PluginWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  if world.first_interaction()["pluginConfiguration"].get(&name).is_none() {
    bail!("no {name} plugin configuration: {}", world.first_interaction());
  }
  Ok(())
}

#[then(regex = r#"^the Pact file metadata will list the "(.*)" plugin with version "(.*)"$"#)]
fn metadata_plugin(world: &mut PluginWorld, step: &Step, name: String, version: String) -> anyhow::Result<()> {
  traced(step);
  let plugins = &world.pact_json["metadata"]["plugins"];
  let listed = plugins.as_array().is_some_and(|a| a.iter().any(|p| p["name"] == name && p["version"] == version));
  if !listed {
    bail!("{name} {version} is not in the metadata plugins: {plugins}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file (request|response|contents) content type will be "(.*)"$"#)]
fn content_type_is(world: &mut PluginWorld, step: &Step, part: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &part_contents(world.first_interaction(), &part)["contentType"];
  if actual != &content_type {
    bail!("expected the {part} content type to be {content_type}, got {actual}");
  }
  Ok(())
}

/// A part of the first interaction: an HTTP request or response, a synchronous message's request or first response,
/// or (`contents`) an asynchronous message.
fn part_of<'a>(interaction: &'a Value, part: &str) -> &'a Value {
  match part {
    "contents" => interaction,
    "response" if interaction["response"].is_array() => &interaction["response"][0],
    _ => &interaction[part],
  }
}

/// The body of a part: `body` for HTTP, `contents` for messages.
fn part_contents<'a>(interaction: &'a Value, part: &str) -> &'a Value {
  let p = part_of(interaction, part);
  if p.get("contents").is_some() { &p["contents"] } else { &p["body"] }
}

fn check_matching_rule(interaction: &Value, part: &str, key: &str, rule: &str) -> anyhow::Result<()> {
  let rules = &part_of(interaction, part)["matchingRules"]["body"];
  let found = rules[key]["matchers"].as_array().is_some_and(|m| m.iter().any(|m| m["match"] == rule));
  if !found {
    bail!("no {rule} matching rule for {key} in the {part}: {rules}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction (request|response) in the pact file will have a matching rule for "(.*)" of type "(.*)"$"#)]
fn part_matching_rule(world: &mut PluginWorld, step: &Step, part: String, key: String, rule: String) -> anyhow::Result<()> {
  traced(step);
  check_matching_rule(world.first_interaction(), &part, &key, &rule)
}

#[then(regex = r#"^the first interaction contents in the pact file will have a matching rule for "(.*)" of type "(.*)"$"#)]
fn contents_matching_rule(world: &mut PluginWorld, step: &Step, key: String, rule: String) -> anyhow::Result<()> {
  traced(step);
  check_matching_rule(world.first_interaction(), "contents", &key, &rule)
}

#[then(regex = r#"^the first interaction in the Pact file will have a type of "(.*)"$"#)]
fn interaction_type(world: &mut PluginWorld, step: &Step, kind: String) -> anyhow::Result<()> {
  traced(step);
  let actual = &world.first_interaction()["type"];
  if actual != &kind {
    bail!("expected the first interaction to be {kind}, got {actual}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file (contents|request contents|response contents) will be "(.*)"$"#)]
fn contents_are(world: &mut PluginWorld, step: &Step, part: String, expected: String) -> anyhow::Result<()> {
  traced(step);
  let part = part.strip_suffix(" contents").unwrap_or(&part);
  let contents = part_contents(world.first_interaction(), part);
  let actual = match (&contents["content"], &contents["encoded"]) {
    (Value::String(s), Value::String(e)) if e.eq_ignore_ascii_case("base64") => BASE64.decode(s)?,
    (Value::String(s), _) => s.as_bytes().to_vec(),
    (other, _) => other.to_string().into_bytes(),
  };
  if actual != expected.as_bytes() {
    bail!("expected the {part} contents to be {expected:?}, got {:?}", String::from_utf8_lossy(&actual));
  }
  Ok(())
}

#[then(regex = r"^the first interaction in the pact file will contain (\d+) response messages?$")]
fn response_messages(world: &mut PluginWorld, step: &Step, n: usize) -> anyhow::Result<()> {
  traced(step);
  let count = world.first_interaction()["response"].as_array().map(Vec::len).unwrap_or(0);
  if count != n {
    bail!("expected {n} response messages, got {count}");
  }
  Ok(())
}

// ---- Provider ----

/// Builds the pact a consumer test with the plugin would have written: `pactffi_using_plugin`, then
/// `pactffi_interaction_contents` for each part the table gives.
#[given(regex = r#"^a Pact file with an HTTP "(.*)" "(.*)" interaction with plugin contents:$"#)]
fn pact_with_plugin_contents(world: &mut PluginWorld, step: &Step, method: String, path: String) -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let cols: HashMap<&str, usize> = rows[0].iter().enumerate().map(|(i, h)| (h.as_str(), i)).collect();
  let get = |name: &str| cols.get(name).and_then(|i| rows[1].get(*i)).cloned().unwrap_or_default();
  let name = get("plugin");
  let content_type = get("content type");

  let pact = ffi::new_pact("c", "p");
  ffi::with_specification(pact, PactSpecification_V4);
  let result = (|| {
    let rc = ffi::using_plugin(pact, &name, world.versions.get(&name).map(String::as_str));
    if rc != 0 {
      bail!("pactffi_using_plugin returned {rc}: {:?}", ffi::last_error());
    }
    let i = ffi::new_interaction(pact, "plugin interaction");
    ffi::with_request(i, &method, &path);
    for (p, col) in [(InteractionPart_Request, "request"), (InteractionPart_Response, "response")] {
      let contents = get(col);
      if !contents.is_empty() {
        let rc = ffi::interaction_contents(i, p, &content_type, &plugin_contents(&contents));
        if rc != 0 {
          bail!("pactffi_interaction_contents returned {rc} for the {col}");
        }
      }
    }
    let dir = world.pact_dir();
    let rc = ffi::pact_handle_write_file(pact, dir.to_str().unwrap(), false);
    if rc != 0 {
      bail!("pactffi_pact_handle_write_file failed with {rc}");
    }
    Ok(dir.join("c-p.json"))
  })();
  ffi::cleanup_plugins(pact);
  ffi::free_pact_handle(pact);
  world.pact_files.push(result?);
  Ok(())
}

/// As `pact_with_plugin_contents`, for a message or synchronous message named `description`.
#[given(regex = r#"^a Pact file with a "(.*)" (message|synchronous message) interaction with plugin contents:$"#)]
fn pact_with_message_contents(world: &mut PluginWorld, step: &Step, description: String, kind: String)
  -> anyhow::Result<()> {
  traced(step);
  let rows = table(step);
  let cols: HashMap<&str, usize> = rows[0].iter().enumerate().map(|(i, h)| (h.as_str(), i)).collect();
  let get = |name: &str| cols.get(name).and_then(|i| rows[1].get(*i)).cloned().unwrap_or_default();
  let name = get("plugin");

  let pact = ffi::new_pact("c", "p");
  ffi::with_specification(pact, PactSpecification_V4);
  let result = (|| {
    let rc = ffi::using_plugin(pact, &name, world.versions.get(&name).map(String::as_str));
    if rc != 0 {
      bail!("pactffi_using_plugin returned {rc}: {:?}", ffi::last_error());
    }
    let i = if kind == "message" {
      ffi::new_message_interaction(pact, &description)
    } else {
      ffi::new_sync_message_interaction(pact, &description)
    };
    let rc = ffi::interaction_contents(i, InteractionPart_Request, &get("content type"), &plugin_contents(&get("contents")));
    if rc != 0 {
      bail!("pactffi_interaction_contents returned {rc}");
    }
    let dir = world.pact_dir();
    let rc = ffi::pact_handle_write_file(pact, dir.to_str().unwrap(), false);
    if rc != 0 {
      bail!("pactffi_pact_handle_write_file failed with {rc}");
    }
    Ok(dir.join("c-p.json"))
  })();
  ffi::cleanup_plugins(pact);
  ffi::free_pact_handle(pact);
  world.pact_files.push(result?);
  Ok(())
}

/// A message provider as the verifier calls it: a POST of `{"description": ...}`, answered with the message body,
/// its content type, and the metadata as base64 JSON in `pact-message-metadata`.
#[given(regex = r#"^a provider is started that can generate the "(.*)" message with "(.*)" and content type "(.*)"$"#)]
async fn message_provider_started(world: &mut PluginWorld, step: &Step, description: String, body: String,
                                  content_type: String) -> anyhow::Result<()> {
  traced(step);
  let body = body_cell(&body)?;
  let metadata = BASE64.encode(serde_json::json!({"contentType": content_type}).to_string());
  let server = Server::start(Arc::new(move |req: &Request, _port| {
    let asked = serde_json::from_slice::<Value>(&req.body).ok()
      .and_then(|v| v["description"].as_str().map(str::to_string)).unwrap_or_default();
    if asked == description {
      Response {
        status: 200,
        headers: vec![("content-type".into(), content_type.clone()), ("pact-message-metadata".into(), metadata.clone())],
        body: body.clone(),
      }
    } else {
      Response { status: 404, headers: vec![], body: format!("no message {asked:?}").into_bytes() }
    }
  })).await?;
  trace::note(format!("message provider listening on {}", server.url()));
  world.provider = Some(server);
  Ok(())
}

#[given(regex = r#"^a provider is started that returns "(.*)" with content type "(.*)"$"#)]
async fn provider_started(world: &mut PluginWorld, step: &Step, body: String, content_type: String) -> anyhow::Result<()> {
  traced(step);
  let body = body_cell(&body)?;
  let server = Server::start(Arc::new(move |_req: &Request, _port| Response {
    status: 200,
    headers: vec![("content-type".into(), content_type.clone())],
    body: body.clone(),
  })).await?;
  trace::note(format!("provider listening on {}", server.url()));
  world.provider = Some(server);
  Ok(())
}

#[when("the verification is run")]
async fn verification_run(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let provider = world.provider.as_ref().ok_or_else(|| anyhow!("no provider started"))?;
  let v = ffi::verifier_new_for_application("pact-ffi-compat", env!("CARGO_PKG_VERSION"));
  ffi::verifier_set_provider_info(v, "p", "http", "127.0.0.1", provider.port, "/");
  for file in &world.pact_files {
    ffi::verifier_add_file_source(v, file.to_str().unwrap());
  }
  let exit = tokio::task::spawn_blocking(move || ffi::verifier_execute(v)).await?;
  world.exit = Some(exit);
  world.output = ffi::verifier_output(v, true).unwrap_or_default();
  world.results = ffi::verifier_json(v).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
  ffi::verifier_shutdown(v);
  Ok(())
}

#[then("the verification will be successful")]
fn successful(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.exit != Some(0) || world.results["result"] != true {
    bail!("verification failed (exit {:?}):\n{}", world.exit, world.output);
  }
  Ok(())
}

#[then("the verification will NOT be successful")]
fn not_successful(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  if world.exit == Some(0) || world.results["result"] != false {
    bail!("verification passed (exit {:?})", world.exit);
  }
  Ok(())
}

#[then(regex = r#"^the verification results will contain a "(.*)" error$"#)]
fn results_contain_error(world: &mut PluginWorld, step: &Step, error: String) -> anyhow::Result<()> {
  traced(step);
  let types: &[&str] = match error.as_str() {
    "Body had differences" => &["BodyMismatch", "BodyTypeMismatch"],
    "Metadata had differences" => &["MetadataMismatch"],
    other => bail!("unknown error {other}"),
  };
  let found = world.results["interactionResults"].as_array().cloned().unwrap_or_default().iter()
    .flat_map(|r| r["mismatch"]["mismatches"].as_array().cloned().unwrap_or_default())
    .any(|m| m["type"].as_str().is_some_and(|t| types.contains(&t)));
  if !found {
    bail!("no \"{error}\" error in the verification results: {}", world.results);
  }
  Ok(())
}

#[then(regex = r#"^the verification output will contain "(.*)"$"#)]
fn output_contains(world: &mut PluginWorld, step: &Step, text: String) -> anyhow::Result<()> {
  traced(step);
  if !world.output.contains(&text) {
    bail!("the verification output does not contain \"{text}\":\n{}", world.output);
  }
  Ok(())
}

// ---- Plugin transports ----

#[when(regex = r#"^a mock server is started for the "(.*)" transport$"#)]
fn transport_mock_server_started(world: &mut PluginWorld, step: &Step, transport: String) -> anyhow::Result<()> {
  traced(step);
  let port = ffi::create_mock_server_for_transport(world.pact()?, "127.0.0.1", 0, &transport);
  if port < 0 {
    bail!("pactffi_create_mock_server_for_transport failed with {port}: {:?}", ffi::last_error());
  }
  world.port = Some(port);
  Ok(())
}

/// Writes `line` and a newline, then reads one line back (or whatever arrives before the peer closes).
async fn tcp_exchange(port: u16, line: &str) -> anyhow::Result<String> {
  let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
  stream.write_all(format!("{line}\n").as_bytes()).await?;
  let mut reply = String::new();
  tokio::time::timeout(Duration::from_secs(10), BufReader::new(stream).read_line(&mut reply)).await??;
  Ok(reply.trim_end_matches(['\r', '\n']).to_string())
}

#[when(regex = r#"^the line "(.*)" is sent to the mock server over TCP$"#)]
async fn line_sent(world: &mut PluginWorld, step: &Step, line: String) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  // A mismatch can leave the mock server with nothing to reply, so a failed read is kept as no reply
  let reply = tcp_exchange(port as u16, &line).await;
  trace::note(format!("TCP {line:?} -> {reply:?}"));
  world.tcp_reply = reply.ok();
  Ok(())
}

#[then(regex = r#"^the line "(.*)" will be received from the mock server$"#)]
fn line_received(world: &mut PluginWorld, step: &Step, line: String) -> anyhow::Result<()> {
  traced(step);
  if world.tcp_reply.as_deref() != Some(line.as_str()) {
    bail!("expected {line:?}, got {:?}", world.tcp_reply);
  }
  Ok(())
}

/// gRPC frames a message with a 0 compression flag and its length as a big-endian u32.
fn grpc_frame(message: &[u8]) -> Bytes {
  let mut frame = vec![0u8];
  frame.extend_from_slice(&(message.len() as u32).to_be_bytes());
  frame.extend_from_slice(message);
  Bytes::from(frame)
}

fn grpc_status(name: &str) -> anyhow::Result<&'static str> {
  Ok(match name {
    "OK" => "0",
    "INVALID_ARGUMENT" => "3",
    "NOT_FOUND" => "5",
    "FAILED_PRECONDITION" => "9",
    "UNIMPLEMENTED" => "12",
    "INTERNAL" => "13",
    other => bail!("unknown gRPC status {other}"),
  })
}

/// A unary gRPC call over HTTP/2 without TLS.
async fn grpc_call(port: u16, method: &str, message: &[u8]) -> anyhow::Result<(String, Vec<u8>)> {
  let tcp = TcpStream::connect(("127.0.0.1", port)).await?;
  let (client, conn) = h2::client::handshake(tcp).await?;
  let conn = tokio::spawn(async move { let _ = conn.await; });
  let mut client = client.ready().await?;
  let req = http::Request::post(format!("http://127.0.0.1:{port}{method}"))
    .header("content-type", "application/grpc")
    .header("te", "trailers")
    .body(())?;
  let (res, mut send) = client.send_request(req, false)?;
  send.send_data(grpc_frame(message), true)?;
  let (parts, mut body) = res.await?.into_parts();
  let mut data = Vec::new();
  while let Some(chunk) = body.data().await {
    let chunk = chunk?;
    let _ = body.flow_control().release_capacity(chunk.len());
    data.extend_from_slice(&chunk);
  }
  let trailers = body.trailers().await?;
  conn.abort();
  // A call that fails before any message sends its status in the headers ("trailers-only")
  let status = trailers.as_ref().and_then(|t| t.get("grpc-status")).or_else(|| parts.headers.get("grpc-status"))
    .and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
  Ok((status, data.get(5..).map(<[u8]>::to_vec).unwrap_or_default()))
}

#[when(regex = r#"^the gRPC method "(.*)" is called on the mock server with (.*)$"#)]
async fn grpc_called(world: &mut PluginWorld, step: &Step, method: String, message: String) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  // Either a quoted body cell or "an empty message"
  let message = match message.strip_prefix('"').and_then(|m| m.strip_suffix('"')) {
    Some(cell) => body_cell(cell)?,
    None => vec![],
  };
  let reply = grpc_call(port as u16, &method, &message).await?;
  trace::note(format!("gRPC {method} -> grpc-status {}", reply.0));
  world.grpc_reply = Some(reply);
  Ok(())
}

#[then(regex = r#"^the gRPC call will return "(.*)"$"#)]
fn grpc_returned(world: &mut PluginWorld, step: &Step, message: String) -> anyhow::Result<()> {
  traced(step);
  let (status, data) = world.grpc_reply.as_ref().ok_or_else(|| anyhow!("no gRPC call made"))?;
  if status != "0" {
    bail!("the gRPC call failed with grpc-status {status}");
  }
  let expected = body_cell(&message)?;
  if *data != expected {
    bail!("expected message {expected:?}, got {data:?}");
  }
  Ok(())
}

#[then(regex = r#"^the gRPC call will fail with status "(.*)"$"#)]
fn grpc_failed(world: &mut PluginWorld, step: &Step, name: String) -> anyhow::Result<()> {
  traced(step);
  let (status, _) = world.grpc_reply.as_ref().ok_or_else(|| anyhow!("no gRPC call made"))?;
  if status != grpc_status(&name)? {
    bail!("expected grpc-status {name}, got {status}");
  }
  Ok(())
}

#[then("the mock server will write out a Pact file for the interaction when done")]
fn transport_pact_written(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  // "the pact test is done" writes the file once the mock server matched
  if world.first_interaction().is_null() {
    bail!("no Pact file was written (mismatches: {:?})", world.mismatches);
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the Pact file will have "(.*)" = '(.*)'$"#)]
fn interaction_field(world: &mut PluginWorld, step: &Step, field: String, value: String) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(&value)?;
  let actual = &world.first_interaction()[field.as_str()];
  if *actual != expected {
    bail!("expected {field} = {expected}, got {actual}");
  }
  Ok(())
}

/// As `pact_with_message_contents`, with the interaction's transport. The FFI has no call to set it outside a mock
/// server, so it is added to the written file, as the consumer's mock server would have recorded it.
#[given(regex = r#"^a Pact file with a "(.*)" synchronous message interaction over the "(.*)" transport with plugin contents:$"#)]
fn pact_with_transport(world: &mut PluginWorld, step: &Step, description: String, transport: String)
  -> anyhow::Result<()> {
  pact_with_message_contents(world, step, description, "synchronous message".into())?;
  let file = world.pact_files.last().unwrap().clone();
  let mut pact: Value = serde_json::from_slice(&fs::read(&file)?)?;
  pact["interactions"][0]["transport"] = Value::String(transport.clone());
  fs::write(&file, serde_json::to_vec_pretty(&pact)?)?;
  trace::note(format!("set interactions[0].transport = {transport:?} in {}", file.display()));
  Ok(())
}

#[given(regex = r#"^a TCP provider is started that replies to "(.*)" with "(.*)"$"#)]
async fn tcp_provider_started(world: &mut PluginWorld, step: &Step, line: String, reply: String) -> anyhow::Result<()> {
  traced(step);
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  world.provider_port = Some(listener.local_addr()?.port());
  world.tasks.push(tokio::spawn(async move {
    while let Ok((stream, _)) = listener.accept().await {
      let (line, reply) = (line.clone(), reply.clone());
      tokio::spawn(async move {
        let (read, mut write) = stream.into_split();
        let mut lines = BufReader::new(read).lines();
        while let Ok(Some(got)) = lines.next_line().await {
          let answer = if got.trim_end() == line { &reply } else { "MATTerrorMATT" };
          if write.write_all(format!("{answer}\n").as_bytes()).await.is_err() {
            break;
          }
        }
      });
    }
  }));
  Ok(())
}

#[given(regex = r#"^a gRPC provider is started that answers "(.*)" with "(.*)"$"#)]
async fn grpc_provider_started(world: &mut PluginWorld, step: &Step, method: String, message: String)
  -> anyhow::Result<()> {
  traced(step);
  let message = body_cell(&message)?;
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  world.provider_port = Some(listener.local_addr()?.port());
  world.tasks.push(tokio::spawn(async move {
    while let Ok((stream, _)) = listener.accept().await {
      let (method, message) = (method.clone(), message.clone());
      tokio::spawn(async move {
        let Ok(mut conn) = h2::server::handshake(stream).await else { return };
        // Each request is answered in its own task: the connection must keep being polled to receive its body
        while let Some(Ok((req, mut respond))) = conn.accept().await {
          let found = req.uri().path() == method;
          let message = message.clone();
          tokio::spawn(async move {
            let mut body = req.into_body();
            while let Some(Ok(chunk)) = body.data().await {
              let _ = body.flow_control().release_capacity(chunk.len());
            }
            let mut trailers = http::HeaderMap::new();
            trailers.insert("grpc-status", if found { "0" } else { "12" }.parse().unwrap());
            let res = http::Response::builder().header("content-type", "application/grpc").body(()).unwrap();
            let Ok(mut send) = respond.send_response(res, false) else { return };
            if found {
              let _ = send.send_data(grpc_frame(&message), false);
            }
            let _ = send.send_trailers(trailers);
          });
        }
      });
    }
  }));
  Ok(())
}

#[given("no provider is listening")]
async fn no_provider(world: &mut PluginWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  // A port that was free a moment ago
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  world.provider_port = Some(listener.local_addr()?.port());
  Ok(())
}

#[when(regex = r#"^the verification is run with the "(.*)" transport$"#)]
async fn transport_verification_run(world: &mut PluginWorld, step: &Step, transport: String) -> anyhow::Result<()> {
  traced(step);
  let port = world.provider_port.ok_or_else(|| anyhow!("no provider started"))?;
  let scheme = if transport == "grpc" { "http" } else { "tcp" };
  let v = ffi::verifier_new_for_application("pact-ffi-compat", env!("CARGO_PKG_VERSION"));
  ffi::verifier_set_provider_info(v, "p", "http", "127.0.0.1", port, "/");
  ffi::verifier_add_provider_transport(v, &transport, port, "", scheme);
  for file in &world.pact_files {
    ffi::verifier_add_file_source(v, file.to_str().unwrap());
  }
  let exit = tokio::task::spawn_blocking(move || ffi::verifier_execute(v)).await?;
  world.exit = Some(exit);
  world.output = ffi::verifier_output(v, true).unwrap_or_default();
  world.results = ffi::verifier_json(v).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
  ffi::verifier_shutdown(v);
  Ok(())
}

fn trace_path(feature: &str, scenario: &str) -> PathBuf {
  let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
  let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
  PathBuf::from(dir).join(safe(feature)).join(format!("{}.md", safe(scenario)))
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  PluginWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|_feature, _rule, scenario, world| Box::pin(async move {
      trace::reset();
      world.scenario_id = scenario.name.clone();
      let _ = fs::remove_dir_all(world.pact_dir());
    }))
    .after(|feature, _rule, scenario, _ev, world| Box::pin(async move {
      if let Some(world) = world {
        world.shutdown();
      }
      let path = trace_path(&format!("V4 {}", feature.name), &scenario.name);
      if let Err(e) = trace::write(&path, &feature.name, &scenario.name) {
        eprintln!("could not write trace {}: {e}", path.display());
      }
    }))
    .filter_run_and_exit(suite_root().join("features/V4/plugins"), |_feature, _rule, scenario| {
      !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
