use std::collections::HashMap;
use std::panic::RefUnwindSafe;
use anyhow::anyhow;

use cucumber::{then, when, World};
use pact_models::bodies::OptionalBody;
use pact_models::generators::GeneratorTestMode;
use pact_models::json_utils::json_to_string;
use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::sync_message::SynchronousMessage;
use serde_json::Value;

use pact_consumer::builders::{
  InteractionBuilder,
  MessageInteractionBuilder,
  PactBuilder,
  SyncMessageInteractionBuilder
};
use pact_matching::{Mismatch, RequestMatchResult};
use pact_mock_server::mock_server::MockServer;
use pact_verifier::{PactSource, ProviderInfo};
use pact_verifier::verification_result::VerificationExecutionResult;
use crate::shared_steps::IndexType;

mod http_consumer;
mod http_provider;
mod generators;
mod http_matching;
mod message_consumer;
pub(crate) mod message_provider;
mod sync_message_consumer;

#[derive(Debug, World)]
pub struct V4World {
  pub scenario_id: String,
  pub request: HttpRequest,
  pub generated_request: HttpRequest,
  pub original_body: OptionalBody,
  pub generated_body: OptionalBody,
  pub generator_test_mode: GeneratorTestMode,
  pub generator_context: HashMap<String, Value>,
  pub builder: PactBuilder,
  pub integration_builder: Option<InteractionBuilder>,
  pub message_builder: Option<MessageInteractionBuilder>,
  pub sync_message_builder: Option<SyncMessageInteractionBuilder>,
  pub pact: V4Pact,
  pub pact_json: Value,
  pub interactions: Vec<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>>,
  pub provider_key: String,
  pub provider_server: MockServer,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub verification_results: VerificationExecutionResult,
  pub expected_response: HttpResponse,
  pub received_responses: Vec<HttpResponse>,
  pub response_results: Vec<Mismatch>,
  pub expected_request: HttpRequest,
  pub received_requests: Vec<HttpRequest>,
  pub request_results: Vec<RequestMatchResult>,
  pub message_proxy_port: u16,
  pub received_sync_messages: Vec<SynchronousMessage>
}

impl Default for V4World {
  fn default() -> Self {
    V4World {
      scenario_id: "".to_string(),
      request: Default::default(),
      generated_request: Default::default(),
      original_body: Default::default(),
      generated_body: Default::default(),
      generator_test_mode: GeneratorTestMode::Consumer,
      generator_context: Default::default(),
      builder: PactBuilder::new_v4("C", "P"),
      integration_builder: None,
      message_builder: None,
      sync_message_builder: None,
      pact: Default::default(),
      pact_json: Default::default(),
      interactions: vec![],
      provider_key: "".to_string(),
      provider_server: Default::default(),
      provider_info: Default::default(),
      sources: vec![],
      verification_results: VerificationExecutionResult::new(),
      expected_response: Default::default(),
      received_responses: vec![],
      response_results: vec![],
      expected_request: Default::default(),
      received_requests: vec![],
      request_results: vec![],
      message_proxy_port: 0,
      received_sync_messages: vec![],
    }
  }
}

#[when("the Pact file for the test is generated")]
fn the_pact_file_for_the_test_is_generated(world: &mut V4World) {
  if let Some(integration_builder) = world.integration_builder.as_ref() {
    world.builder.push_interaction(&integration_builder.build_v4());
  }
  if let Some(message_builder) = world.message_builder.as_ref() {
    world.builder.push_interaction(&message_builder.build());
  }
  if let Some(message_builder) = world.sync_message_builder.as_ref() {
    world.builder.push_interaction(&message_builder.build());
  }
  world.pact = world.builder.build().as_v4_pact().unwrap();
  world.pact_json = world.pact.to_json(PactSpecification::V4).unwrap();
}

#[then(expr = "there will be an interaction in the Pact file with a type of {string}")]
fn there_will_be_an_interaction_in_the_pact_file_with_a_type_of(
  world: &mut V4World,
  i_type: String
) -> anyhow::Result<()> {
  let interactions = world.pact_json["interactions"].as_array().unwrap();
  let interaction = interactions.iter().find(|i| {
    json_to_string(i.get("type").unwrap_or(&Value::Null)) == i_type
  });
  if let Some(_) = interaction {
    Ok(())
  } else {
    Err(anyhow!("Did not find interaction in Pact JSON with type attribute {}", i_type))
  }
}

#[then(expr = "the {numType} interaction in the Pact file will have a type of {string}")]
fn the_interaction_in_the_pact_file_will_have_a_type_of(
  world: &mut V4World,
  index: IndexType,
  i_type: String
) -> anyhow::Result<()> {
  let interactions = world.pact_json["interactions"].as_array().unwrap();
  let interaction = interactions[index.val()].as_object().unwrap();
  if let Some(interaction_type) = interaction.get("type") {
    if json_to_string(interaction_type) == i_type {
      Ok(())
    } else {
      Err(anyhow!("Expected interaction type attribute {} but got {}", i_type, interaction_type))
    }
  } else {
    Err(anyhow!("Interaction in Pact JSON has no type attribute"))
  }
}

#[then(expr = "the {numType} interaction in the Pact file will have {string} = {string}")]
fn the_first_interaction_in_the_pact_file_will_have(
  world: &mut V4World,
  index: IndexType,
  name: String,
  value: String
) -> anyhow::Result<()> {
  let interactions = world.pact_json["interactions"].as_array().unwrap();
  let interaction = interactions[index.val()].as_object().unwrap();
  let json: Value = serde_json::from_str(value.as_str()).unwrap();
  if let Some(actual_value) = interaction.get(name.as_str()) {
    if json == *actual_value {
      Ok(())
    } else {
      Err(anyhow!("Expected interaction {} attribute {} but got {}", name, value, actual_value))
    }
  } else {
    Err(anyhow!("Interaction in Pact JSON has no {} attribute", name))
  }
}
