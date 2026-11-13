//! V4 plugin features (`features/V4/plugins`), driven through pact_consumer (`PactBuilderAsync::using_plugin`,
//! `contents`/`contents_from` and the HTTP and plugin mock servers) and pact_verifier, which loads the plugins a pact
//! file lists.
//!
//! The plugins come from `~/.pact/plugins` (or `PACT_PLUGIN_DIR`). The suite root is `PACT_COMPAT_SUITE`, so the
//! features can be run from a suite checkout before they reach the `pact-compatibility-suite` subtree.

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail};
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bytes::Bytes;
use cucumber::gherkin::Step;
use cucumber::{World, given, then, when};
use maplit::hashmap;
use pact_consumer::builders::PactBuilderAsync;
use pact_consumer::mock_server::{StartMockServerAsync, ValidatingMockServer};
use pact_consumer::prelude::HttpPartBuilder;
use pact_models::PactSpecification;
use pact_models::content_types::ContentType;
use pact_models::pact::{Pact, write_pact};
use pact_models::provider_states::ProviderState;
use pact_plugin_driver::catalogue_manager::lookup_entry;
use pact_plugin_driver::mock_server::{MockServerConfig, MockServerDetails};
use pact_plugin_driver::plugin_manager::{get_mock_server_results, load_plugin, shutdown_mock_server, shutdown_plugins, start_mock_server_v2};
use pact_plugin_driver::plugin_models::PluginDependency;
use pact_verifier::callback_executors::{NullRequestFilterExecutor, ProviderStateExecutor};
use pact_verifier::verification_result::VerificationMismatchResult;
use pact_verifier::{FilterInfo, PactSource, ProviderInfo, ProviderTransport, VerificationOptions, verify_provider_async};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;

/// An interaction a step has described. They are added to a `PactBuilderAsync` once the mock server is started or
/// the pact file is generated, as each plugin call needs the plugin loaded for the builder first.
#[derive(Debug, Clone)]
enum Spec {
  Http {
    desc: String,
    method: String,
    path: String,
    req_plugin: Option<(String, Value)>,
    resp_plugin: Option<(String, Value)>,
    resp_body: Option<(String, String)>
  },
  Message {
    sync: bool,
    desc: String,
    contents: Option<(String, Value)>
  }
}

#[derive(Default, World)]
struct PluginWorld {
  scenario_id: String,
  /// Plugin versions from the "is installed" steps, used when a step loads a plugin without naming a version.
  versions: HashMap<String, String>,
  specs: Vec<Spec>,
  /// Plugins this test has loaded, with the version asked for.
  loaded: Vec<(String, Option<String>)>,
  plugin_error: Option<String>,
  builder: Option<PactBuilderAsync>,
  server: Option<Box<dyn ValidatingMockServer>>,
  /// A transport's mock server, started through the plugin driver, with the pact it was started for and the
  /// transport's catalogue key.
  transport_server: Option<(MockServerDetails, Box<dyn Pact + Send + Sync>, String)>,
  port: Option<u16>,
  matched: Option<bool>,
  mismatches: Vec<Value>,
  status: u16,
  body: Vec<u8>,
  pact_json: Value,
  provider: Option<Server>,
  pact_files: Vec<PathBuf>,
  verify_result: Option<bool>,
  output: String,
  error_types: Vec<String>,
  /// The line the matt mock server sent back.
  tcp_reply: Option<String>,
  /// The gRPC call's status (`grpc-status`, from the headers or trailers) and its message, without the frame prefix.
  grpc_reply: Option<(String, Vec<u8>)>,
  /// The port a TCP or gRPC provider listens on, or a closed one for "no provider is listening".
  provider_port: Option<u16>,
  tasks: Vec<JoinHandle<()>>,
}

impl fmt::Debug for PluginWorld {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("PluginWorld")
      .field("scenario_id", &self.scenario_id)
      .field("specs", &self.specs)
      .field("loaded", &self.loaded)
      .field("port", &self.port)
      .finish()
  }
}

impl PluginWorld {
  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite/plugins").join(&self.scenario_id)
  }

  fn spec(&mut self) -> anyhow::Result<&mut Spec> {
    self.specs.last_mut().ok_or_else(|| anyhow!("no interaction is being defined"))
  }

  /// Loads the plugin up front, as `using_plugin` panics when it can't.
  async fn load_plugin(&mut self, name: &str, version: Option<String>) -> anyhow::Result<()> {
    let version = version.or_else(|| self.versions.get(name).cloned());
    match check_plugin(name, version.clone()).await {
      Ok(()) => {
        self.loaded.push((name.to_string(), version));
        Ok(())
      }
      Err(e) => {
        self.plugin_error = Some(e.to_string());
        Err(e)
      }
    }
  }

  async fn build_builder(&mut self) -> anyhow::Result<&mut PactBuilderAsync> {
    let mut builder = PactBuilderAsync::new_v4("c", "p");
    builder.output_dir(self.pact_dir());
    for (name, version) in &self.loaded {
      builder.using_plugin(name, version.clone()).await;
    }
    for spec in self.specs.clone() {
      add_spec(&mut builder, spec).await?;
    }
    Ok(self.builder.insert(builder))
  }

  fn first_interaction(&self) -> &Value {
    &self.pact_json["interactions"][0]
  }

  async fn shutdown(&mut self) {
    for task in self.tasks.drain(..) {
      task.abort();
    }
    if let Some(server) = self.server.take() {
      let _ = drop_server(server);
    }
    if let Some((details, _, _)) = self.transport_server.take() {
      let _ = shutdown_mock_server(&details).await;
    }
    self.builder = None;
    shutdown_plugins();
  }
}

async fn check_plugin(name: &str, version: Option<String>) -> anyhow::Result<()> {
  load_plugin(&PluginDependency { name: name.to_string(), version, dependency_type: Default::default() }).await
    .map(|_| ())
}

fn content_type(ct: &str) -> anyhow::Result<ContentType> {
  ContentType::parse(ct).map_err(|e| anyhow!("content type {ct}: {e}"))
}

async fn add_spec(builder: &mut PactBuilderAsync, spec: Spec) -> anyhow::Result<()> {
  match spec {
    Spec::Http { desc, method, path, req_plugin, resp_plugin, resp_body } => {
      let req_plugin = req_plugin.map(|(ct, v)| content_type(&ct).map(|ct| (ct, v))).transpose()?;
      let resp_plugin = resp_plugin.map(|(ct, v)| content_type(&ct).map(|ct| (ct, v))).transpose()?;
      builder.interaction(desc, String::new(), move |mut i| async move {
        i.request.method(method).path(path);
        if let Some((ct, v)) = req_plugin {
          i.request.contents(ct, v).await;
        }
        if let Some((ct, v)) = resp_plugin {
          i.response.contents(ct, v).await;
        }
        if let Some((body, ct)) = resp_body {
          i.response.body2(body, ct);
        }
        i
      }).await;
    }
    Spec::Message { sync, desc, contents } => {
      let contents = contents.map(|(ct, mut v)| {
        if let Some(o) = v.as_object_mut() {
          o.insert("pact:content-type".into(), Value::String(ct));
        }
        v
      });
      if sync {
        builder.synchronous_message_interaction(desc, move |mut i| async move {
          if let Some(v) = contents {
            i.contents_from(v).await;
          }
          i
        }).await;
      } else {
        builder.message_interaction(desc, move |mut i| async move {
          if let Some(v) = contents {
            i.contents_from(v).await;
          }
          i
        }).await;
      }
    }
  }
  Ok(())
}

/// Mock servers verify and write the pact when dropped, and panic if they did not match.
fn drop_server(server: Box<dyn ValidatingMockServer>) -> Result<(), String> {
  tokio::task::block_in_place(|| std::panic::catch_unwind(AssertUnwindSafe(|| drop(server))))
    .map_err(|payload| payload.downcast_ref::<String>().cloned()
      .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
      .unwrap_or_else(|| "mock server panicked".to_string()))
}

fn table(step: &Step) -> Vec<Vec<String>> {
  step.table.as_ref().map(|t| t.rows.clone()).unwrap_or_default()
}

/// The suite checkout: `PACT_COMPAT_SUITE`, or the subtree.
fn suite_root() -> PathBuf {
  env::var("PACT_COMPAT_SUITE").map(PathBuf::from)
    .unwrap_or_else(|_| PathBuf::from("pact-compatibility-suite"))
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
fn plugin_contents(text: &str) -> anyhow::Result<Value> {
  let fixtures = fs::canonicalize(fixtures())?;
  let text = text.replace("\"fixture:", &format!("\"{}/", fixtures.display()));
  serde_json::from_str(&text).map_err(|e| anyhow!("plugin contents are not JSON: {e}"))
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
  let file = world.pact_dir().join("c-p.json");
  world.pact_json = serde_json::from_str(&fs::read_to_string(&file).map_err(|e| anyhow!("{}: {e}", file.display()))?)?;
  Ok(())
}

// ---- A plain HTTP provider ----

struct Request {
  body: Vec<u8>,
}

struct Response {
  status: u16,
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

type Handler = Arc<dyn Fn(&Request) -> Response + Send + Sync>;

/// An HTTP/1.1 server that answers each connection's one request with the handler, then closes it.
struct Server {
  port: u16,
  task: JoinHandle<()>,
}

impl Server {
  async fn start(handler: Handler) -> anyhow::Result<Server> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let task = tokio::spawn(async move {
      while let Ok((stream, _)) = listener.accept().await {
        let handler = handler.clone();
        tokio::spawn(async move { let _ = Server::serve(stream, handler).await; });
      }
    });
    Ok(Server { port, task })
  }

  async fn serve(mut stream: TcpStream, handler: Handler) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
      if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
        break i + 4;
      }
      let n = stream.read(&mut chunk).await?;
      if n == 0 {
        bail!("connection closed before the request head");
      }
      buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let length = head.lines().skip(1)
      .filter_map(|l| l.split_once(':'))
      .find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
      .and_then(|(_, v)| v.trim().parse::<usize>().ok())
      .unwrap_or(0);
    let mut body = buf[head_end..].to_vec();
    while body.len() < length {
      let n = stream.read(&mut chunk).await?;
      if n == 0 {
        break;
      }
      body.extend_from_slice(&chunk[..n]);
    }
    let res = handler(&Request { body });
    let mut out = format!("HTTP/1.1 {} X\r\n", res.status);
    for (k, v) in &res.headers {
      out.push_str(&format!("{k}: {v}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\nConnection: close\r\n\r\n", res.body.len()));
    stream.write_all(out.as_bytes()).await?;
    stream.write_all(&res.body).await?;
    stream.shutdown().await?;
    Ok(())
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    self.task.abort();
  }
}

// ---- Consumer ----

#[given(regex = r#"^the "(.*)" plugin is not installed$"#)]
fn not_installed(_world: &mut PluginWorld, name: String) -> anyhow::Result<()> {
  let found = installed(&name);
  if !found.is_empty() {
    bail!("plugin {name} is installed: {found:?}");
  }
  Ok(())
}

#[given(regex = r#"^the "(.*)" plugin version "(.*)" is installed$"#)]
fn installed_version(world: &mut PluginWorld, name: String, version: String) -> anyhow::Result<()> {
  let dir = format!("{name}-{version}");
  if !installed(&name).contains(&dir) {
    bail!("plugin {dir} is not installed in {}", plugin_dir().display());
  }
  world.versions.insert(name, version);
  Ok(())
}

fn http_spec(desc: String) -> Spec {
  Spec::Http { desc, method: "GET".into(), path: "/".into(), req_plugin: None, resp_plugin: None, resp_body: None }
}

#[given("an HTTP interaction is being defined for a consumer test")]
fn interaction_defined(world: &mut PluginWorld) {
  world.specs.push(http_spec("plugin interaction 1".into()));
}

#[given("another HTTP interaction is being defined for the same consumer test")]
fn another_interaction(world: &mut PluginWorld) {
  let n = world.specs.len() + 1;
  world.specs.push(http_spec(format!("plugin interaction {n}")));
}

/// A failure is kept for the "loading the plugin will fail" step.
#[when(regex = r#"^the "(.*)" plugin is loaded for the test$"#)]
async fn plugin_loaded(world: &mut PluginWorld, name: String) {
  let _ = world.load_plugin(&name, None).await;
}

#[when(regex = r#"^the "(.*)" plugin version "(.*)" is loaded for the test$"#)]
async fn plugin_version_loaded(world: &mut PluginWorld, name: String, version: String) {
  let _ = world.load_plugin(&name, Some(version)).await;
}

#[then(regex = r#"^loading the plugin will fail with an error that mentions "(.*)"$"#)]
fn plugin_failed(world: &mut PluginWorld, text: String) -> anyhow::Result<()> {
  match &world.plugin_error {
    None => bail!("the plugin loaded"),
    Some(e) if e.contains(&text) => Ok(()),
    Some(e) => bail!("the error does not mention \"{text}\": {e}"),
  }
}

/// `load_plugin` returns the error, so nothing panics.
#[then("the test will not have panicked")]
fn not_panicked(_world: &mut PluginWorld) {}

#[given(regex = r#"^the HTTP request is "(.*)" "(.*)"$"#)]
fn request_is(world: &mut PluginWorld, method: String, path: String) -> anyhow::Result<()> {
  match world.spec()? {
    Spec::Http { method: m, path: p, .. } => {
      *m = method;
      *p = path;
      Ok(())
    }
    _ => bail!("the interaction is not HTTP"),
  }
}

async fn plugin_body(world: &mut PluginWorld, step: &Step, name: &str, ct: &str) -> anyhow::Result<(String, Value)> {
  if !world.loaded.iter().any(|(l, _)| l == name) {
    world.load_plugin(name, None).await?;
  }
  let contents = plugin_contents(step.docstring.as_deref().ok_or_else(|| anyhow!("no docstring"))?)?;
  Ok((ct.to_string(), contents))
}

#[given(regex = r#"^the HTTP request body is set with the "(.*)" plugin for "(.*)" with:$"#)]
async fn request_plugin_body(world: &mut PluginWorld, step: &Step, name: String, ct: String) -> anyhow::Result<()> {
  let body = plugin_body(world, step, &name, &ct).await?;
  match world.spec()? {
    Spec::Http { req_plugin, .. } => *req_plugin = Some(body),
    _ => bail!("the interaction is not HTTP"),
  }
  Ok(())
}

#[given(regex = r#"^the HTTP response body is set with the "(.*)" plugin for "(.*)" with:$"#)]
async fn response_plugin_body(world: &mut PluginWorld, step: &Step, name: String, ct: String) -> anyhow::Result<()> {
  let body = plugin_body(world, step, &name, &ct).await?;
  match world.spec()? {
    Spec::Http { resp_plugin, .. } => *resp_plugin = Some(body),
    _ => bail!("the interaction is not HTTP"),
  }
  Ok(())
}

#[given(regex = r#"^the HTTP response body is "(.*)" with content type "(.*)"$"#)]
fn response_body(world: &mut PluginWorld, body: String, ct: String) -> anyhow::Result<()> {
  let body = String::from_utf8(body_cell(&body)?)?;
  match world.spec()? {
    Spec::Http { resp_body, .. } => *resp_body = Some((body, ct)),
    _ => bail!("the interaction is not HTTP"),
  }
  Ok(())
}

#[given("a message interaction is being defined for a consumer test")]
fn message_defined(world: &mut PluginWorld) {
  world.specs.push(Spec::Message { sync: false, desc: "plugin message 1".into(), contents: None });
}

#[given("a synchronous message interaction is being defined for a consumer test")]
fn sync_message_defined(world: &mut PluginWorld) {
  world.specs.push(Spec::Message { sync: true, desc: "plugin message 1".into(), contents: None });
}

#[given(regex = r#"^the (?:synchronous )?message contents are set with the "(.*)" plugin for "(.*)" with:$"#)]
async fn message_plugin_contents(world: &mut PluginWorld, step: &Step, name: String, ct: String) -> anyhow::Result<()> {
  let body = plugin_body(world, step, &name, &ct).await?;
  match world.spec()? {
    Spec::Message { contents, .. } => *contents = Some(body),
    _ => bail!("the interaction is not a message"),
  }
  Ok(())
}

async fn start_mock_server(world: &mut PluginWorld) -> anyhow::Result<()> {
  let builder = world.build_builder().await?;
  let server = builder.start_mock_server_async(None, None).await;
  world.port = server.url().port();
  world.server = Some(server);
  Ok(())
}

/// Transport mock servers are started through the plugin driver, not `start_mock_server_async`: pact_consumer's
/// `PluginMockServer::url()` panics when the plugin's address is not a URL (matt reports a bare host), and the
/// port is all the steps need.
async fn start_transport_mock_server(world: &mut PluginWorld, transport: String) -> anyhow::Result<()> {
  let pact = world.build_builder().await?.build();
  let entry = lookup_entry(&format!("transport/{transport}"))
    .ok_or_else(|| anyhow!("no catalogue entry for the {transport} transport"))?;
  let config = MockServerConfig { output_path: Some(world.pact_dir()), host_interface: None, port: 0, tls: false };
  let details = start_mock_server_v2(&entry, pact.boxed(), config, hashmap!{}).await?;
  world.port = Some(details.port as u16);
  world.transport_server = Some((details, pact, entry.key));
  Ok(())
}

/// Shuts a transport mock server down, and writes the pact (each interaction's transport set to the catalogue key,
/// as pact_consumer does) when it matched. Mismatches are reported in pact_consumer's format.
async fn finish_transport_mock_server(world: &mut PluginWorld, details: MockServerDetails,
  pact: Box<dyn Pact + Send + Sync>, key: String) -> anyhow::Result<Option<String>> {
  // The results are asked for before shutting down, as the FFI does: matt only reports mismatches here
  let results = get_mock_server_results(&details).await;
  shutdown_mock_server(&details).await?;
  let results = results?;
  let failed: Vec<_> = results.iter().filter(|r| !(r.error.is_empty() && r.mismatches.is_empty())).collect();
  if !failed.is_empty() {
    let mut message = "plugin mock server failed verification:\n".to_string();
    for (index, result) in failed.iter().enumerate() {
      if result.error.is_empty() {
        message += &format!("    {}) {} - the following mismatches occurred:\n", index + 1, result.path);
        for (n, m) in result.mismatches.iter().enumerate() {
          message += &format!("        {}.{}) [{}] {}\n", index + 1, n + 1, m.path, m.mismatch);
        }
      } else {
        message += &format!("    {}) {}: {}\n", index + 1, result.path, result.error);
      }
    }
    return Ok(Some(message));
  }
  let mut pact = pact.as_v4_pact()?;
  for interaction in &mut pact.interactions {
    interaction.set_transport(Some(key.clone()));
  }
  let pact = pact.boxed();
  let file = world.pact_dir().join(pact.default_file_name());
  write_pact(pact, &file, PactSpecification::V4, true)?;
  Ok(None)
}

#[when(regex = r"^the mock server is started with (all the interactions|the plugin interaction)$")]
async fn mock_server_started(world: &mut PluginWorld, _which: String) -> anyhow::Result<()> {
  start_mock_server(world).await
}

async fn send(world: &mut PluginWorld, method: &str, path: &str, body: Option<(&str, &str)>) -> anyhow::Result<()> {
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let url = format!("http://127.0.0.1:{port}{path}");
  let mut req = reqwest::Client::new().request(method.parse()?, &url);
  if let Some((ct, body)) = body {
    req = req.header("content-type", ct).body(body.to_string());
  }
  let res = req.send().await?;
  world.status = res.status().as_u16();
  world.body = res.bytes().await?.to_vec();
  Ok(())
}

#[when(regex = r#"^a "(.*)" request is made to "(.*)" with content type "(.*)" and body "(.*)"$"#)]
async fn request_with_body(world: &mut PluginWorld, method: String, path: String, ct: String, body: String)
  -> anyhow::Result<()> {
  send(world, &method, &path, Some((&ct, &body))).await
}

#[when(regex = r#"^a "([A-Z]+)" request is made to "([^"]*)"$"#)]
async fn request_made(world: &mut PluginWorld, method: String, path: String) -> anyhow::Result<()> {
  send(world, &method, &path, None).await
}

#[then("a 200 success response is returned")]
fn success_response(world: &mut PluginWorld) -> anyhow::Result<()> {
  if world.status != 200 {
    bail!("expected a 200 response, got {}: {}", world.status, String::from_utf8_lossy(&world.body));
  }
  Ok(())
}

#[then(regex = r#"^the response body will be "(.*)"$"#)]
fn response_body_is(world: &mut PluginWorld, body: String) -> anyhow::Result<()> {
  let expected = body_cell(&body)?;
  // The mock server re-serialises JSON bodies, so JSON is compared as values
  let same_json = serde_json::from_slice::<Value>(&expected).ok()
    .zip(serde_json::from_slice::<Value>(&world.body).ok()).is_some_and(|(a, b)| a == b);
  if world.body != expected && !same_json {
    bail!("expected body {:?}, got {:?}", String::from_utf8_lossy(&expected), String::from_utf8_lossy(&world.body));
  }
  Ok(())
}

/// Dropping the mock server writes the pact file when everything matched, and otherwise panics with the
/// mismatches (the only place a plugin mock server reports them).
#[when("the pact test is done")]
async fn pact_test_done(world: &mut PluginWorld) -> anyhow::Result<()> {
  world.port = None;
  let mut matched = true;
  world.mismatches = vec![];
  if let Some((details, pact, key)) = world.transport_server.take() {
    if let Some(message) = finish_transport_mock_server(world, details, pact, key).await? {
      matched = false;
      world.mismatches.push(Value::String(message));
    }
  } else {
    let server = world.server.take().ok_or_else(|| anyhow!("mock server not started"))?;
    let status = server.status();
    world.mismatches = status.iter().map(|m| m.to_json()).collect();
    matched = status.iter().all(|m| m.matched());
    if let Err(message) = drop_server(server) {
      matched = false;
      world.mismatches.push(Value::String(message));
    }
  }
  world.matched = Some(matched);
  if matched {
    read_pact(world)?;
    // A plugin mock server records the transport by its catalogue key ("transport/matt")
    if let Some(interactions) = world.pact_json["interactions"].as_array_mut() {
      for i in interactions {
        if let Some(t) = i["transport"].as_str().and_then(|t| t.rsplit_once('/')).map(|(_, t)| t.to_string()) {
          i["transport"] = Value::String(t);
        }
      }
    }
  }
  Ok(())
}

#[then("the mock server status will be OK")]
fn status_ok(world: &mut PluginWorld) -> anyhow::Result<()> {
  if world.matched != Some(true) {
    bail!("mock server did not match: {:?}", world.mismatches);
  }
  Ok(())
}

#[then("the mock server status will be mismatches")]
fn status_mismatches(world: &mut PluginWorld) -> anyhow::Result<()> {
  if world.matched != Some(false) {
    bail!("mock server had no mismatches");
  }
  Ok(())
}

#[then(regex = r#"^the mismatches will contain a mismatch with error "(.*)"$"#)]
fn mismatch_with_error(world: &mut PluginWorld, error: String) -> anyhow::Result<()> {
  if !world.mismatches.iter().any(|m| any_string_contains(m, &error)) {
    bail!("no mismatch with \"{error}\": {:?}", world.mismatches);
  }
  Ok(())
}

#[when("the Pact file for the test is generated")]
async fn pact_generated(world: &mut PluginWorld) -> anyhow::Result<()> {
  let dir = world.pact_dir();
  let pact = world.build_builder().await?.build();
  fs::create_dir_all(&dir)?;
  write_pact(pact, &dir.join("c-p.json"), PactSpecification::V4, false)?;
  read_pact(world)
}

#[when("the plugins are cleaned up")]
fn plugins_cleaned_up(world: &mut PluginWorld) {
  world.builder = None;
  shutdown_plugins();
}

/// Plugin processes are stopped asynchronously, so this waits a little for them to go.
#[then(regex = r#"^no "(.*)" plugin process will still be running$"#)]
async fn no_plugin_process(_world: &mut PluginWorld, name: String) -> anyhow::Result<()> {
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
fn pact_interactions(world: &mut PluginWorld, n: usize) -> anyhow::Result<()> {
  let count = world.pact_json["interactions"].as_array().map(Vec::len).unwrap_or(0);
  if count != n {
    bail!("expected {n} interactions, the pact file has {count}");
  }
  Ok(())
}

#[then("the second interaction in the pact file will not have plugin configuration")]
fn no_plugin_configuration(world: &mut PluginWorld) -> anyhow::Result<()> {
  let config = &world.pact_json["interactions"][1]["pluginConfiguration"];
  if config.as_object().is_some_and(|o| !o.is_empty()) {
    bail!("the second interaction has plugin configuration: {config}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file will have plugin configuration for "(.*)"$"#)]
fn has_plugin_configuration(world: &mut PluginWorld, name: String) -> anyhow::Result<()> {
  if world.first_interaction()["pluginConfiguration"].get(&name).is_none() {
    bail!("no {name} plugin configuration: {}", world.first_interaction());
  }
  Ok(())
}

#[then(regex = r#"^the Pact file metadata will list the "(.*)" plugin with version "(.*)"$"#)]
fn metadata_plugin(world: &mut PluginWorld, name: String, version: String) -> anyhow::Result<()> {
  let plugins = &world.pact_json["metadata"]["plugins"];
  let listed = plugins.as_array().is_some_and(|a| a.iter().any(|p| p["name"] == name && p["version"] == version));
  if !listed {
    bail!("{name} {version} is not in the metadata plugins: {plugins}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file (request|response|contents) content type will be "(.*)"$"#)]
fn content_type_is(world: &mut PluginWorld, part: String, ct: String) -> anyhow::Result<()> {
  let actual = &part_contents(world.first_interaction(), &part)["contentType"];
  if actual != &ct {
    bail!("expected the {part} content type to be {ct}, got {actual}");
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
fn part_matching_rule(world: &mut PluginWorld, part: String, key: String, rule: String) -> anyhow::Result<()> {
  check_matching_rule(world.first_interaction(), &part, &key, &rule)
}

#[then(regex = r#"^the first interaction contents in the pact file will have a matching rule for "(.*)" of type "(.*)"$"#)]
fn contents_matching_rule(world: &mut PluginWorld, key: String, rule: String) -> anyhow::Result<()> {
  check_matching_rule(world.first_interaction(), "contents", &key, &rule)
}

#[then(regex = r#"^the first interaction in the Pact file will have a type of "(.*)"$"#)]
fn interaction_type(world: &mut PluginWorld, kind: String) -> anyhow::Result<()> {
  let actual = &world.first_interaction()["type"];
  if actual != &kind {
    bail!("expected the first interaction to be {kind}, got {actual}");
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the pact file (contents|request contents|response contents) will be "(.*)"$"#)]
fn contents_are(world: &mut PluginWorld, part: String, expected: String) -> anyhow::Result<()> {
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
fn response_messages(world: &mut PluginWorld, n: usize) -> anyhow::Result<()> {
  let count = world.first_interaction()["response"].as_array().map(Vec::len).unwrap_or(0);
  if count != n {
    bail!("expected {n} response messages, got {count}");
  }
  Ok(())
}

// ---- Provider ----

/// The first data row of a step's table, by column name.
fn table_row(step: &Step) -> HashMap<String, String> {
  let rows = table(step);
  rows.first().map(|h| h.iter().cloned().zip(rows.get(1).cloned().unwrap_or_default()).collect()).unwrap_or_default()
}

/// Writes the pact a consumer test with the plugin would have: `using_plugin`, then the interaction.
async fn write_plugin_pact(world: &mut PluginWorld, name: &str, spec: Spec) -> anyhow::Result<()> {
  let version = world.versions.get(name).cloned();
  check_plugin(name, version.clone()).await?;
  let mut builder = PactBuilderAsync::new_v4("c", "p");
  builder.using_plugin(name, version).await;
  add_spec(&mut builder, spec).await?;
  let dir = world.pact_dir();
  fs::create_dir_all(&dir)?;
  let file = dir.join("c-p.json");
  write_pact(builder.build(), &file, PactSpecification::V4, false)?;
  world.pact_files.push(file);
  Ok(())
}

#[given(regex = r#"^a Pact file with an HTTP "(.*)" "(.*)" interaction with plugin contents:$"#)]
async fn pact_with_plugin_contents(world: &mut PluginWorld, step: &Step, method: String, path: String)
  -> anyhow::Result<()> {
  let row = table_row(step);
  let get = |col: &str| row.get(col).cloned().unwrap_or_default();
  let name = get("plugin");
  let ct = get("content type");
  let part = |col: &str| -> anyhow::Result<Option<(String, Value)>> {
    let cell = get(col);
    Ok(if cell.is_empty() { None } else { Some((ct.clone(), plugin_contents(&cell)?)) })
  };
  let spec = Spec::Http {
    desc: "plugin interaction".into(), method, path,
    req_plugin: part("request")?, resp_plugin: part("response")?, resp_body: None
  };
  write_plugin_pact(world, &name, spec).await
}

#[given(regex = r#"^a Pact file with a "(.*)" (message|synchronous message) interaction with plugin contents:$"#)]
async fn pact_with_message_contents(world: &mut PluginWorld, step: &Step, desc: String, kind: String)
  -> anyhow::Result<()> {
  let row = table_row(step);
  let get = |col: &str| row.get(col).cloned().unwrap_or_default();
  let spec = Spec::Message {
    sync: kind != "message", desc,
    contents: Some((get("content type"), plugin_contents(&get("contents"))?))
  };
  write_plugin_pact(world, &get("plugin"), spec).await
}

/// A message provider as the verifier calls it: a POST of `{"description": ...}`, answered with the message body,
/// its content type, and the metadata as base64 JSON in `pact-message-metadata`.
#[given(regex = r#"^a provider is started that can generate the "(.*)" message with "(.*)" and content type "(.*)"$"#)]
async fn message_provider_started(world: &mut PluginWorld, description: String, body: String, ct: String)
  -> anyhow::Result<()> {
  let body = body_cell(&body)?;
  let metadata = BASE64.encode(serde_json::json!({"contentType": ct}).to_string());
  let server = Server::start(Arc::new(move |req: &Request| {
    let asked = serde_json::from_slice::<Value>(&req.body).ok()
      .and_then(|v| v["description"].as_str().map(str::to_string)).unwrap_or_default();
    if asked == description {
      Response {
        status: 200,
        headers: vec![("content-type".into(), ct.clone()), ("pact-message-metadata".into(), metadata.clone())],
        body: body.clone(),
      }
    } else {
      Response { status: 404, headers: vec![], body: format!("no message {asked:?}").into_bytes() }
    }
  })).await?;
  world.provider = Some(server);
  Ok(())
}

#[given(regex = r#"^a provider is started that returns "(.*)" with content type "(.*)"$"#)]
async fn provider_started(world: &mut PluginWorld, body: String, ct: String) -> anyhow::Result<()> {
  let body = body_cell(&body)?;
  let server = Server::start(Arc::new(move |_req: &Request| Response {
    status: 200,
    headers: vec![("content-type".into(), ct.clone())],
    body: body.clone(),
  })).await?;
  world.provider = Some(server);
  Ok(())
}

#[derive(Debug)]
struct DummyProviderStateExecutor;

#[async_trait]
impl ProviderStateExecutor for DummyProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    _provider_state: &ProviderState,
    _setup: bool,
    _client: Option<&reqwest::Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    Ok(hashmap!{})
  }

  fn teardown(self: &Self) -> bool {
    false
  }
}

// Message interactions are verified over the default protocol/port, which only the deprecated fields set
#[allow(deprecated)]
async fn verify(world: &mut PluginWorld, port: u16, transport: Option<ProviderTransport>) -> anyhow::Result<()> {
  let provider_info = ProviderInfo {
    name: "p".to_string(),
    protocol: "http".to_string(),
    host: "127.0.0.1".to_string(),
    port: Some(port),
    path: "/".to_string(),
    transports: transport.into_iter().collect(),
    .. ProviderInfo::default()
  };
  let sources = world.pact_files.iter().map(|f| PactSource::File(f.to_string_lossy().to_string())).collect();
  let result = verify_provider_async(
    provider_info, sources, FilterInfo::None, vec![],
    &VerificationOptions::<NullRequestFilterExecutor>::default(), None,
    &Arc::new(DummyProviderStateExecutor), None
  ).await?;
  let mut output = result.output.clone();
  for (description, error) in &result.errors {
    match error {
      VerificationMismatchResult::Mismatches { mismatches, .. } => {
        for m in mismatches {
          world.error_types.push(m.mismatch_type().to_string());
          output.push(format!("{description}: {}", m.description()));
        }
      }
      VerificationMismatchResult::Error { error, .. } => output.push(format!("{description}: {error}")),
    }
  }
  world.output = output.join("\n");
  world.verify_result = Some(result.result);
  Ok(())
}

#[when("the verification is run")]
async fn verification_run(world: &mut PluginWorld) -> anyhow::Result<()> {
  let port = world.provider.as_ref().ok_or_else(|| anyhow!("no provider started"))?.port;
  verify(world, port, None).await
}

#[then("the verification will be successful")]
fn successful(world: &mut PluginWorld) -> anyhow::Result<()> {
  if world.verify_result != Some(true) {
    bail!("verification failed:\n{}", world.output);
  }
  Ok(())
}

#[then("the verification will NOT be successful")]
fn not_successful(world: &mut PluginWorld) -> anyhow::Result<()> {
  if world.verify_result != Some(false) {
    bail!("verification passed:\n{}", world.output);
  }
  Ok(())
}

#[then(regex = r#"^the verification results will contain a "(.*)" error$"#)]
fn results_contain_error(world: &mut PluginWorld, error: String) -> anyhow::Result<()> {
  let types: &[&str] = match error.as_str() {
    "Body had differences" => &["BodyMismatch", "BodyTypeMismatch"],
    "Metadata had differences" => &["MetadataMismatch"],
    other => bail!("unknown error {other}"),
  };
  if !world.error_types.iter().any(|t| types.contains(&t.as_str())) {
    bail!("no \"{error}\" error in the verification results: {:?}\n{}", world.error_types, world.output);
  }
  Ok(())
}

#[then(regex = r#"^the verification output will contain "(.*)"$"#)]
fn output_contains(world: &mut PluginWorld, text: String) -> anyhow::Result<()> {
  if !world.output.contains(&text) {
    bail!("the verification output does not contain \"{text}\":\n{}", world.output);
  }
  Ok(())
}

// ---- Plugin transports ----

#[when(regex = r#"^a mock server is started for the "(.*)" transport$"#)]
async fn transport_mock_server_started(world: &mut PluginWorld, transport: String) -> anyhow::Result<()> {
  start_transport_mock_server(world, transport).await
}

/// Plugin mock servers bind localhost, which can be IPv6 only (the protobuf plugin listens on `::1`).
async fn connect_local(port: u16) -> std::io::Result<TcpStream> {
  match TcpStream::connect(("127.0.0.1", port)).await {
    Ok(stream) => Ok(stream),
    Err(_) => TcpStream::connect(("::1", port)).await,
  }
}

/// Writes `line` and a newline, then reads one line back (or whatever arrives before the peer closes).
async fn tcp_exchange(port: u16, line: &str) -> anyhow::Result<String> {
  let mut stream = connect_local(port).await?;
  stream.write_all(format!("{line}\n").as_bytes()).await?;
  let mut reply = String::new();
  tokio::time::timeout(Duration::from_secs(10), BufReader::new(stream).read_line(&mut reply)).await??;
  Ok(reply.trim_end_matches(['\r', '\n']).to_string())
}

#[when(regex = r#"^the line "(.*)" is sent to the mock server over TCP$"#)]
async fn line_sent(world: &mut PluginWorld, line: String) -> anyhow::Result<()> {
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  // A mismatch can leave the mock server with nothing to reply, so a failed read is kept as no reply
  world.tcp_reply = tcp_exchange(port, &line).await.ok();
  Ok(())
}

#[then(regex = r#"^the line "(.*)" will be received from the mock server$"#)]
fn line_received(world: &mut PluginWorld, line: String) -> anyhow::Result<()> {
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
  let tcp = connect_local(port).await?;
  let (client, conn) = h2::client::handshake(tcp).await?;
  let conn = tokio::spawn(async move { let _ = conn.await; });
  let mut client = client.ready().await?;
  let req = http::Request::post(format!("http://localhost:{port}{method}"))
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
async fn grpc_called(world: &mut PluginWorld, method: String, message: String) -> anyhow::Result<()> {
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  // Either a quoted body cell or "an empty message"
  let message = match message.strip_prefix('"').and_then(|m| m.strip_suffix('"')) {
    Some(cell) => body_cell(cell)?,
    None => vec![],
  };
  world.grpc_reply = Some(grpc_call(port, &method, &message).await?);
  Ok(())
}

#[then(regex = r#"^the gRPC call will return "(.*)"$"#)]
fn grpc_returned(world: &mut PluginWorld, message: String) -> anyhow::Result<()> {
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
fn grpc_failed(world: &mut PluginWorld, name: String) -> anyhow::Result<()> {
  let (status, _) = world.grpc_reply.as_ref().ok_or_else(|| anyhow!("no gRPC call made"))?;
  if status != grpc_status(&name)? {
    bail!("expected grpc-status {name}, got {status}");
  }
  Ok(())
}

#[then("the mock server will write out a Pact file for the interaction when done")]
fn transport_pact_written(world: &mut PluginWorld) -> anyhow::Result<()> {
  // "the pact test is done" reads the file once the mock server matched
  if world.first_interaction().is_null() {
    bail!("no Pact file was written (mismatches: {:?})", world.mismatches);
  }
  Ok(())
}

#[then(regex = r#"^the first interaction in the Pact file will have "(.*)" = '(.*)'$"#)]
fn interaction_field(world: &mut PluginWorld, field: String, value: String) -> anyhow::Result<()> {
  let expected: Value = serde_json::from_str(&value)?;
  let actual = &world.first_interaction()[field.as_str()];
  if *actual != expected {
    bail!("expected {field} = {expected}, got {actual}");
  }
  Ok(())
}

/// As `pact_with_message_contents`, with the interaction's transport added to the written file, as the consumer's
/// mock server would have recorded it.
#[given(regex = r#"^a Pact file with a "(.*)" synchronous message interaction over the "(.*)" transport with plugin contents:$"#)]
async fn pact_with_transport(world: &mut PluginWorld, step: &Step, desc: String, transport: String)
  -> anyhow::Result<()> {
  pact_with_message_contents(world, step, desc, "synchronous message".into()).await?;
  let file = world.pact_files.last().unwrap().clone();
  let mut pact: Value = serde_json::from_slice(&fs::read(&file)?)?;
  pact["interactions"][0]["transport"] = Value::String(transport);
  fs::write(&file, serde_json::to_vec_pretty(&pact)?)?;
  Ok(())
}

#[given(regex = r#"^a TCP provider is started that replies to "(.*)" with "(.*)"$"#)]
async fn tcp_provider_started(world: &mut PluginWorld, line: String, reply: String) -> anyhow::Result<()> {
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
async fn grpc_provider_started(world: &mut PluginWorld, method: String, message: String) -> anyhow::Result<()> {
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
async fn no_provider(world: &mut PluginWorld) -> anyhow::Result<()> {
  // A port that was free a moment ago
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  world.provider_port = Some(listener.local_addr()?.port());
  Ok(())
}

#[when(regex = r#"^the verification is run with the "(.*)" transport$"#)]
async fn transport_verification_run(world: &mut PluginWorld, transport: String) -> anyhow::Result<()> {
  let port = world.provider_port.ok_or_else(|| anyhow!("no provider started"))?;
  let scheme = if transport == "grpc" { "http" } else { "tcp" };
  let transport = ProviderTransport {
    transport,
    port: Some(port),
    path: None,
    scheme: Some(scheme.to_string())
  };
  verify(world, port, Some(transport)).await
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }

  // The plugin features are not in the suite subtree yet; point PACT_COMPAT_SUITE at a checkout that has them
  let features = suite_root().join("features/V4/plugins");
  if !features.is_dir() {
    eprintln!("Skipping the plugin features: {} does not exist", features.display());
    return;
  }

  PluginWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|_feature, _rule, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
      let _ = fs::remove_dir_all(world.pact_dir());
    }))
    .after(|_feature, _rule, _scenario, _ev, world| Box::pin(async move {
      if let Some(world) = world {
        world.shutdown().await;
      }
    }))
    .filter_run_and_exit(features, |_feature, _rule, scenario| {
      !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
