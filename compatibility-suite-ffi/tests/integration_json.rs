//! FFI-only scenarios: matching rules defined with integration JSON (the format FFI consumers pass to
//! `pactffi_with_body`), which the shared compatibility suite cannot express since it defines matching rules in the
//! Pact file format.

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail};
use cucumber::gherkin::Step;
use cucumber::{given, then, when, World};
use serde_json::Value;

use pact_ffi_compat::ffi::{self, InteractionHandle, InteractionPart_Request, PactHandle, PactSpecification,
  PactSpecification_V3, PactSpecification_V4};
use pact_ffi_compat::{model, trace};

const FEATURES: &str = "features/ffi";

#[derive(Debug, Default, World)]
struct IntegrationJsonWorld {
  scenario_id: String,
  spec: String,
  pact: Option<PactHandle>,
  interaction: Option<InteractionHandle>,
  port: Option<i32>,
  pact_json: Value,
}

impl IntegrationJsonWorld {
  fn specification(&self) -> anyhow::Result<PactSpecification> {
    match self.spec.as_str() {
      "V3" => Ok(PactSpecification_V3),
      "V4" => Ok(PactSpecification_V4),
      s => bail!("unsupported specification {s}"),
    }
  }

  fn pact_dir(&self) -> PathBuf {
    env::current_dir().unwrap().join("target/compatibility-suite/ffi").join(&self.scenario_id)
  }

  fn pact(&self) -> anyhow::Result<PactHandle> {
    self.pact.ok_or_else(|| anyhow!("no pact is being defined"))
  }

  fn request(&self) -> &Value {
    &self.pact_json["interactions"][0]["request"]
  }

  /// The V3 Pact file has the body as JSON, the V4 one as `{"content": ...}`.
  fn request_body(&self) -> &Value {
    let body = &self.request()["body"];
    if self.spec == "V4" { &body["content"] } else { body }
  }

  fn shutdown(&mut self) {
    if let Some(port) = self.port.take() {
      ffi::cleanup_mock_server(port);
    }
    if let Some(pact) = self.pact.take() {
      ffi::free_pact_handle(pact);
    }
  }
}

fn traced(step: &Step) {
  trace::step(&step.keyword, &step.value);
}

fn docstring(step: &Step) -> anyhow::Result<&str> {
  step.docstring.as_deref().ok_or_else(|| anyhow!("step has no doc string"))
}

#[given(regex = r"^a (V3|V4) HTTP interaction with a POST request with the following JSON body:$")]
fn interaction_with_body(world: &mut IntegrationJsonWorld, step: &Step, spec: String) -> anyhow::Result<()> {
  traced(step);
  world.spec = spec;
  let pact = ffi::new_pact(&format!("{}-integration-json-c", world.spec.to_lowercase()), "p");
  ffi::with_specification(pact, world.specification()?);
  let i = ffi::new_interaction(pact, "a request with integration JSON");
  ffi::with_request(i, "POST", "/path");
  if !ffi::with_body(i, InteractionPart_Request, "application/json", docstring(step)?) {
    bail!("pactffi_with_body failed");
  }
  ffi::response_status(i, 200);
  world.pact = Some(pact);
  world.interaction = Some(i);
  Ok(())
}

#[when("the Pact file for the test is generated")]
fn pact_generated(world: &mut IntegrationJsonWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let dir = world.pact_dir();
  fs::create_dir_all(&dir)?;
  let rc = ffi::pact_handle_write_file(world.pact()?, &dir.to_string_lossy(), true);
  if rc != 0 {
    bail!("pactffi_pact_handle_write_file failed with {rc}");
  }
  let file = fs::read_dir(&dir)?.next().ok_or_else(|| anyhow!("no pact file written"))??.path();
  world.pact_json = serde_json::from_str(&fs::read_to_string(file)?)?;
  Ok(())
}

#[when("the mock server is started with the interaction")]
fn mock_server_started(world: &mut IntegrationJsonWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let port = ffi::create_mock_server_for_transport(world.pact()?, "::1", 0, "http");
  if port < 0 {
    bail!("pactffi_create_mock_server_for_transport failed with {port}");
  }
  world.port = Some(port);
  Ok(())
}

#[when(regex = r"^a POST request is made to the mock server with the JSON body '(.*)'$")]
async fn request_made(world: &mut IntegrationJsonWorld, step: &Step, body: String) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let url = format!("http://[::1]:{port}/path");
  let res = reqwest::Client::new().post(&url).header("content-type", "application/json").body(body).send().await?;
  trace::note(format!("HTTP POST {url} -> {}", res.status()));
  Ok(())
}

#[then(regex = r"^the mock server status will be (OK|NOT OK)$")]
fn mock_server_status(world: &mut IntegrationJsonWorld, step: &Step, status: String) -> anyhow::Result<()> {
  traced(step);
  let port = world.port.ok_or_else(|| anyhow!("mock server not started"))?;
  let matched = ffi::mock_server_matched(port);
  let mismatches = ffi::mock_server_mismatches(port).unwrap_or_default();
  match (status.as_str(), matched) {
    ("OK", false) => bail!("Expected the mock server to match, but it had mismatches: {mismatches}"),
    ("NOT OK", true) => bail!("Expected the mock server to have mismatches, but it matched"),
    _ => Ok(()),
  }
}

#[then(regex = r"^the request body in the Pact file will be '(.*)'$")]
fn request_body(world: &mut IntegrationJsonWorld, step: &Step, expected: String) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(&expected)?;
  let actual = world.request_body();
  if *actual != expected {
    bail!("Expected the request body to be {expected} but was {actual}");
  }
  Ok(())
}

#[then("the request body matching rules in the Pact file will be:")]
fn request_body_rules(world: &mut IntegrationJsonWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let expected: Value = serde_json::from_str(docstring(step)?)?;
  let actual = &world.request()["matchingRules"]["body"];
  if *actual != expected {
    bail!("Expected the request body matching rules to be {expected} but were {actual}");
  }
  Ok(())
}

#[then("the request body in the Pact file will have no matching rules")]
fn no_request_body_rules(world: &mut IntegrationJsonWorld, step: &Step) -> anyhow::Result<()> {
  traced(step);
  let rules = &world.request()["matchingRules"]["body"];
  if !(rules.is_null() || rules.as_object().is_some_and(|o| o.is_empty())) {
    bail!("Expected no request body matching rules but found {rules}");
  }
  Ok(())
}

#[tokio::main]
async fn main() {
  let lib = model::ffi_lib();
  ffi::load(&lib).unwrap_or_else(|e| panic!("could not load {lib}: {e}"));

  IntegrationJsonWorld::cucumber()
    .max_concurrent_scenarios(1)
    .fail_on_skipped()
    .before(|_feature, _rule, scenario, world| Box::pin(async move {
      trace::reset();
      world.scenario_id = scenario.name.replace(['/', '\\', ':'], "_");
      let _ = fs::remove_dir_all(world.pact_dir());
    }))
    .after(|feature, _rule, scenario, _ev, world| Box::pin(async move {
      if let Some(world) = world {
        world.shutdown();
      }
      let dir = env::var("FFI_TRACE_DIR").unwrap_or_else(|_| "target/ffi-trace".into());
      let path = PathBuf::from(dir).join(format!("FFI {}", feature.name))
        .join(format!("{}.md", scenario.name.replace(['/', '\\', ':'], "_")));
      if let Err(e) = trace::write(&path, &feature.name, &scenario.name) {
        eprintln!("could not write trace {}: {e}", path.display());
      }
    }))
    .filter_run_and_exit(FEATURES, |_feature, _rule, scenario| !scenario.tags.iter().any(|t| t == "wip"))
    .await;
}
