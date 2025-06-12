use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use cucumber::{given, then, when};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_mock_server::builder::MockServerBuilder;
use pact_mock_server::mock_server::MockServerConfig;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::pact::Pact;
use pact_models::prelude::ProviderState;
use pact_models::prelude::v4::V4Pact;
use pact_models::v4::interaction::V4Interaction;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use pact_matching::Mismatch;
use pact_verifier::{
  FilterInfo,
  PactSource,
  ProviderInfo,
  ProviderTransport,
  VerificationOptions,
  verify_provider_async
};
use pact_verifier::callback_executors::ProviderStateExecutor;
use pact_verifier::verification_result::VerificationMismatchResult;

use crate::shared_steps::{setup_body, setup_common_interactions};
use crate::shared_steps::provider::ProviderWorldRequestFilter;
use crate::v4_steps::V4World;

#[given("the following HTTP interactions have been defined:")]
fn the_following_http_interactions_have_been_setup(world: &mut V4World, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let interactions = setup_common_interactions(table);
    world.interactions.extend(interactions.iter().map(|i| i.as_v4().unwrap()));
  }
}

#[given(expr = "a provider is started that returns the response from interaction {int}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_response_from_interaction(world: &mut V4World, num: usize) -> anyhow::Result<()> {
  let pact = V4Pact {
    consumer: Consumer { name: "v4-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().boxed_v4() ],
    .. V4Pact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V4,
    .. MockServerConfig::default()
  };

  // let (mock_server, future) = MockServer::new(
  //   world.provider_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  // ).await.map_err(|err| anyhow!(err))?;
  // tokio::spawn(future);
  let mock_server = MockServerBuilder::new()
    .with_v4_pact(pact.as_v4_pact().unwrap())
    .with_id(world.provider_key.clone())
    .with_config(config)
    .bind_to("[::1]:0")
    .with_transport("http")?
    .start()
    .await?;

  // let ms = world.provider_server.lock().unwrap();
  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "[::1]".to_string(),
    port: Some(mock_server.port()),
    transports: vec![ProviderTransport {
      port: Some(mock_server.port()),
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };
  world.provider_server = mock_server;

  Ok(())
}

#[given(expr = "a provider is started that returns the response from interaction {int}, with the following changes:")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_response_from_interaction_with_the_following_changes(
  world: &mut V4World,
  step: &Step,
  num: usize
) -> anyhow::Result<()> {
  let mut interaction = world.interactions.get(num - 1).unwrap()
    .as_v4_http().unwrap();
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "status" | "response" => interaction.response.status = value.parse().unwrap(),
          "headers" | "response headers" => {
            let headers = interaction.response.headers_mut();
            let headers_to_add = value.split(",")
              .map(|header| {
                let key_value = header.strip_prefix("'").unwrap_or(header)
                  .strip_suffix("'").unwrap_or(header)
                  .splitn(2, ":")
                  .map(|v| v.trim())
                  .collect::<Vec<_>>();
                (key_value[0].to_string(), parse_header(key_value[0], key_value[1]))
              });
            for (k, v) in headers_to_add {
              match headers.entry(k) {
                Entry::Occupied(mut entry) => {
                  entry.get_mut().extend_from_slice(&v);
                }
                Entry::Vacant(entry) => {
                  entry.insert(v);
                }
              }
            }
          },
          "body" | "response body" => {
            setup_body(value, &mut interaction.response, None);
          },
          _ => {}
        }
      }
    }
  }

  let pact = V4Pact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction.boxed_v4()],
    .. V4Pact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V4,
    .. MockServerConfig::default()
  };

  // let (mock_server, future) = MockServer::new(
  //   world.provider_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  // ).await.map_err(|err| anyhow!(err))?;
  // tokio::spawn(future);
  let mock_server = MockServerBuilder::new()
    .with_v4_pact(pact.as_v4_pact().unwrap())
    .with_id(world.provider_key.clone())
    .with_config(config)
    .bind_to("[::1]:0")
    .with_transport("http")?
    .start()
    .await?;

  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "[::1]".to_string(),
    port: Some(mock_server.port()),
    transports: vec![ProviderTransport {
      port: Some(mock_server.port()),
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };

  world.provider_server = mock_server;

  Ok(())
}

#[given(expr = "a Pact file for interaction {int} is to be verified, but is marked pending")]
fn a_pact_file_for_interaction_is_to_be_verified_but_is_marked_pending(
  world: &mut V4World,
  num: usize
) {
  let mut interaction = world.interactions.get(num - 1).unwrap()
    .as_v4_http().unwrap();
  interaction.pending = true;
  let pact = V4Pact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ interaction.boxed_v4() ],
    .. V4Pact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V4).unwrap().to_string()));
}

#[given(expr = "a Pact file for interaction {int} is to be verified with the following comments:")]
fn a_pact_file_for_interaction_is_to_be_verified_with_the_following_comments(
  world: &mut V4World,
  step: &Step,
  num: usize
) {
  let mut interaction = world.interactions.get(num - 1).unwrap()
    .as_v4_http().unwrap();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for row in table.rows.iter().skip(1) {
      let data: HashMap<String, String> = row.iter().enumerate().map(|(i, v)| (headers[i].clone(), v.clone())).collect();
      match data["type"].as_str() {
        "text" => {
           match interaction.comments.entry("text".to_string()) {
             Entry::Occupied(mut entry) => {
               let array = entry.get_mut().as_array_mut().unwrap();
               array.push(json!(data["comment"]));
             }
             Entry::Vacant(entry) => {
               entry.insert(json!([ data["comment"] ]));
             }
           }
        }
        "testname" => {
          interaction.comments.insert("testname".to_string(), json!(data["comment"]));
        },
        _ => {}
      }
    }
  }

  let pact = V4Pact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ interaction.boxed_v4() ],
    .. V4Pact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V4).unwrap().to_string()));
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
    _client: Option<&Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    Ok(hashmap!{})
  }

  fn teardown(self: &Self) -> bool {
    return false
  }
}

#[when("the verification is run")]
async fn the_verification_is_run(world: &mut V4World) -> anyhow::Result<()> {
  let options = VerificationOptions::<ProviderWorldRequestFilter>::default();
  world.verification_results = verify_provider_async(
    world.provider_info.clone(),
    world.sources.clone(),
    FilterInfo::None,
    vec![],
    &options,
    None,
    &Arc::new(DummyProviderStateExecutor {}),
    None
  ).await?;
  Ok(())
}

#[then("the verification will be successful")]
fn the_verification_will_be_successful(world: &mut V4World) -> anyhow::Result<()> {
  if world.verification_results.result {
    Ok(())
  } else {
    Err(anyhow!("Verification failed"))
  }
}

#[then(expr = "there will be a pending {string} error")]
fn there_will_be_a_pending_error(world: &mut V4World, err: String) -> anyhow::Result<()> {
  if let Some(_) = world.verification_results.pending_errors.iter().find(|(_, result)| {
    match result {
      VerificationMismatchResult::Mismatches { mismatches, .. } => {
        mismatches.iter().any(|mismatch| {
          match mismatch {
            Mismatch::MethodMismatch { .. } => false,
            Mismatch::PathMismatch { .. } => false,
            Mismatch::StatusMismatch { .. } => err == "Response status did not match",
            Mismatch::QueryMismatch { .. } => false,
            Mismatch::HeaderMismatch { .. } => err == "Headers had differences",
            Mismatch::BodyTypeMismatch { .. } => false,
            Mismatch::BodyMismatch { .. } => err == "Body had differences",
            Mismatch::MetadataMismatch { .. } => false
          }
        })
      }
      VerificationMismatchResult::Error { error, .. } => err == *error
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find {} in the pending errors", err))
  }
}

#[then(expr = "the comment {string} will have been printed to the console")]
fn the_comment_will_have_been_printed_to_the_console(world: &mut V4World, comment: String) -> anyhow::Result<()> {
  let comment = comment.as_str();
  if world.verification_results.output.iter().find(|o| o.contains(comment)).is_some() {
    Ok(())
  } else {
    Err(anyhow!("Did not find '{}' in the output", comment))
  }
}

#[then(expr = "the {string} will displayed as the original test name")]
fn the_will_displayed_as_the_original_test_name(world: &mut V4World, name: String) -> anyhow::Result<()> {
  let comment = format!("Test Name: {}", name);
  if world.verification_results.output.iter().find(|o| o.contains(comment.as_str())).is_some() {
    Ok(())
  } else {
    Err(anyhow!("Did not find '{}' in the output", comment))
  }
}
