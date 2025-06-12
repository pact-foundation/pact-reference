use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use cucumber::{given, then, when, World};
use cucumber::gherkin::Step;
use itertools::{Either, Itertools};
use maplit::hashmap;
use pact_mock_server::builder::MockServerBuilder;
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::{MockServer, MockServerConfig};
use pact_models::{Consumer, generators, matchingrules, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, JSON, XML};
use pact_models::generators::Generator;
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::MatchingRule;
use pact_models::pact::{Pact, read_pact};
use pact_models::provider_states::ProviderState;
use pact_models::query_strings::parse_query_string;
use pact_models::request::Request;
use pact_models::response::Response;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::http_parts::HttpRequest;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use pact_matching::Mismatch;
use pact_verifier::{
  FilterInfo,
  PactSource,
  ProviderInfo,
  ProviderTransport,
  PublishOptions,
  VerificationOptions,
  verify_provider_async
};
use pact_verifier::callback_executors::{ProviderStateExecutor, RequestFilterExecutor};
use pact_verifier::verification_result::{VerificationExecutionResult, VerificationMismatchResult};

use crate::shared_steps::{setup_body, setup_common_interactions};

#[derive(Debug, World)]
pub struct ProviderWorld {
  pub spec_version: PactSpecification,
  pub interactions: Vec<RequestResponseInteraction>,
  pub provider_key: String,
  pub provider_server: MockServer,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub publish_options: Option<PublishOptions>,
  pub verification_results: VerificationExecutionResult,
  pub mock_brokers: Vec<MockServer>,
  pub provider_state_executor: Arc<MockProviderStateExecutor>,
  pub request_filter_data: HashMap<String, String>
}

impl ProviderWorld {
  pub(crate) fn verification_options(&self) -> VerificationOptions<ProviderWorldRequestFilter> {
    VerificationOptions {
      request_filter: if self.request_filter_data.is_empty() {
        None
      } else {
        Some(Arc::new(ProviderWorldRequestFilter {
          request_filter_data: self.request_filter_data.clone()
        }))
      },
      .. VerificationOptions::default()
    }
  }
}

impl Default for ProviderWorld {
  fn default() -> Self {
    ProviderWorld {
      spec_version: PactSpecification::V1,
      interactions: vec![],
      provider_key: "".to_string(),
      provider_server: Default::default(),
      provider_info: ProviderInfo::default(),
      sources: vec![],
      publish_options: None,
      verification_results: VerificationExecutionResult {
        result: false,
        .. VerificationExecutionResult::new()
      },
      mock_brokers: vec![],
      provider_state_executor: Default::default(),
      request_filter_data: Default::default()
    }
  }
}

#[derive(Debug, Default)]
pub struct MockProviderStateExecutor {
  pub params: Arc<Mutex<Vec<(ProviderState, bool)>>>,
  pub fail_mode: AtomicBool
}

impl MockProviderStateExecutor {
  pub fn set_fail_mode(&self, mode: bool) {
    self.fail_mode.store(mode, Ordering::Relaxed);
  }

  pub fn was_called(&self, is_setup: bool) -> bool {
    let params = self.params.lock().unwrap();
    params.iter().find(|(_, setup)| *setup == is_setup).is_some()
  }

  pub fn was_called_for_state(&self, state_name: &str, is_setup: bool) -> bool {
    let params = self.params.lock().unwrap();
    params.iter().find(|(state, setup)| {
      state.name == state_name && *setup == is_setup
    }).is_some()
  }

  pub fn was_called_for_state_with_params(
    &self,
    state_name: &str,
    state_params: &HashMap<String, Value>,
    is_setup: bool
  ) -> bool {
    let params = self.params.lock().unwrap();
    params.iter().find(|(state, setup)| {
      state.name == state_name &&
        state.params == *state_params &&
        *setup == is_setup
    }).is_some()
  }
}

#[derive(Debug, Default, Clone)]
pub struct ProviderWorldRequestFilter {
  pub request_filter_data: HashMap<String, String>
}

impl RequestFilterExecutor for ProviderWorldRequestFilter {
  fn call(self: Arc<Self>, request: &HttpRequest) -> HttpRequest {
    let mut request = request.clone();

    if let Some(path) = self.request_filter_data.get("path") {
      request.path = path.clone();
    }

    if let Some(query) = self.request_filter_data.get("query") {
      request.query = parse_query_string(query);
    }

    if let Some(headers) = self.request_filter_data.get("headers") {
      if !headers.is_empty() {
        let headers = headers.split(",")
          .map(|header| {
            let key_value = header.strip_prefix("'").unwrap_or(header)
              .strip_suffix("'").unwrap_or(header)
              .splitn(2, ":")
              .map(|v| v.trim())
              .collect::<Vec<_>>();
            (key_value[0].to_string(), parse_header(key_value[0], key_value[1]))
          }).collect();
        request.headers = Some(headers);
      }
    }

    if let Some(body) = self.request_filter_data.get("body") {
      if !body.is_empty() {
        if body.starts_with("JSON:") {
          request.add_header("content-type", vec!["application/json"]);
          request.body = OptionalBody::Present(Bytes::from(body.strip_prefix("JSON:").unwrap_or(body).to_string()),
            Some(JSON.clone()), None);
        } else if body.starts_with("XML:") {
          request.add_header("content-type", vec!["application/xml"]);
          request.body = OptionalBody::Present(Bytes::from(body.strip_prefix("XML:").unwrap_or(body).to_string()),
            Some(XML.clone()), None);
        } else {
          let ct = if body.ends_with(".json") {
            "application/json"
          } else if body.ends_with(".xml") {
            "application/xml"
          } else {
            "text/plain"
          };
          request.headers_mut().insert("content-type".to_string(), vec![ct.to_string()]);

          let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", body))
            .expect(format!("could not load fixture '{}'", body).as_str());
          let mut buffer = Vec::new();
          f.read_to_end(&mut buffer)
            .expect(format!("could not read fixture '{}'", body).as_str());
          request.body = OptionalBody::Present(Bytes::from(buffer),
            ContentType::parse(ct).ok(), None);
        }
      }
    }

    request
  }

  fn call_non_http(
    &self,
    _request_body: &OptionalBody,
    _metadata: &HashMap<String, Either<Value, Bytes>>
  ) -> (OptionalBody, HashMap<String, Either<Value, Bytes>>) {
    unimplemented!()
  }
}

#[async_trait]
impl ProviderStateExecutor for MockProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    _client: Option<&Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    let mut lock = self.params.try_lock();
    if let Ok(ref mut params) = lock {
      params.push((provider_state.clone(), setup));
    }

    if self.fail_mode.load(Ordering::Relaxed) {
      Err(anyhow!("ProviderStateExecutor is in fail mode"))
    } else {
      Ok(hashmap! {})
    }
  }

  fn teardown(self: &Self) -> bool {
    return true
  }
}

#[given("the following HTTP interactions have been defined:")]
fn the_following_http_interactions_have_been_setup(world: &mut ProviderWorld, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let interactions = setup_common_interactions(table);
    world.interactions.extend(interactions);
  }
}

#[given(expr = "a provider is started that returns the response from interaction {int}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_response_from_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: world.spec_version,
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
  world: &mut ProviderWorld,
  step: &Step,
  num: usize
) -> anyhow::Result<()> {
  let mut interaction = world.interactions.get(num - 1).unwrap().clone();
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "response" => interaction.response.status = value.parse().unwrap(),
          "response headers" => {
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
          "response body" => {
            setup_body(value, &mut interaction.response, None);
          },
          _ => {}
        }
      }
    }
  }

  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: world.spec_version,
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

#[given(expr = "a Pact file for interaction {int} is to be verified")]
fn a_pact_file_for_interaction_is_to_be_verified(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(world.spec_version)?.to_string()));
  Ok(())
}

#[given(expr = "a Pact file for interaction {int} is to be verified with a provider state {string} defined")]
fn a_pact_file_for_interaction_is_to_be_verified_with_a_provider_state(
  world: &mut ProviderWorld,
  num: usize,
  state: String
) -> anyhow::Result<()> {
  let mut interaction = world.interactions.get(num - 1).unwrap().clone();
  interaction.provider_states.push(ProviderState {
    name: state,
    params: Default::default(),
  });
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(world.spec_version)?.to_string()));
  Ok(())
}

#[given(expr = "a Pact file for interaction {int} is to be verified with the following provider states defined:")]
fn a_pact_file_for_interaction_is_to_be_verified_with_the_following_provider_states_defined(
  world: &mut ProviderWorld,
  step: &Step,
  num: usize
) -> anyhow::Result<()> {
  let mut interaction = world.interactions.get(num - 1).unwrap().clone();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    for values in table.rows.iter().skip(1) {
      let data = values.iter().enumerate()
        .map(|(index, v)| (headers.get(&index).unwrap().as_str(), v.clone()))
        .collect::<HashMap<_, _>>();
      if let Some(parameters) = data.get("Parameters") {
        let json: Value = serde_json::from_str(parameters.as_str()).unwrap();
        interaction.provider_states.push(ProviderState {
          name: data.get("State Name").unwrap().clone(),
          params: json.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        });
      } else {
        interaction.provider_states.push(ProviderState {
          name: data.get("State Name").unwrap().clone(),
          params: Default::default(),
        });
      }
    }
  } else {
    return Err(anyhow!("No data table defined"));
  }

  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(world.spec_version)?.to_string()));
  Ok(())
}

#[when("the verification is run")]
async fn the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let options = world.verification_options();
  world.verification_results = verify_provider_async(
    world.provider_info.clone(),
    world.sources.clone(),
    FilterInfo::None,
    vec![],
    &options,
    world.publish_options.as_ref(),
    &world.provider_state_executor,
    None
  ).await?;
  Ok(())
}

#[then("the verification will be successful")]
fn the_verification_will_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Ok(())
  } else {
    Err(anyhow!("Verification failed"))
  }
}

#[given(expr = "a provider is started that returns the responses from interactions {string}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_responses_from_interactions(
  world: &mut ProviderWorld,
  ids: String
) -> anyhow::Result<()> {
  let interactions = ids.split(",")
    .map(|id| id.trim().parse::<usize>().unwrap())
    .map(|index| world.interactions.get(index - 1).unwrap().clone())
    .collect();
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions,
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: world.spec_version,
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

#[then("the verification will NOT be successful")]
fn the_verification_will_not_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Err(anyhow!("Was expecting the verification to fail"))
  } else {
    Ok(())
  }
}

#[then(expr = "the verification results will contain a {string} error")]
fn the_verification_results_will_contain_a_error(world: &mut ProviderWorld, err: String) -> anyhow::Result<()> {
  if world.verification_results.errors.iter().any(|(_, r)| {
    match r {
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
      VerificationMismatchResult::Error { error, .. } => match err.as_str() {
        "State change request failed" => error == "One or more of the setup state change handlers has failed",
        _ => error.as_str() == err
      }
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find error message in verification results"))
  }
}

#[given(expr = "a Pact file for interaction {int} is to be verified from a Pact broker")]
async fn a_pact_file_for_interaction_is_to_be_verified_from_a_pact_broker(
  world: &mut ProviderWorld,
  num: usize
) -> anyhow::Result<()> {
  let interaction = world.interactions.get(num - 1).unwrap().clone();
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction.clone()],
    specification_version: world.spec_version,
    .. RequestResponsePact::default()
  };
  let mut pact_json = pact.to_json(world.spec_version)?;
  let pact_json_inner = pact_json.as_object_mut().unwrap();
  pact_json_inner.insert("_links".to_string(), json!({
    "pb:publish-verification-results": {
      "title": "Publish verification results",
      "href": format!("http://localhost:1234/pacts/provider/p/consumer/c_{}/verification-results", num)
    }
  }));
  let interactions_json = pact_json_inner.get_mut("interactions").unwrap().as_array_mut().unwrap();
  let interaction_json = interactions_json.get_mut(0).unwrap().as_object_mut().unwrap();
  interaction_json.insert("_id".to_string(), json!(interaction.id.unwrap()));

  let f = PathBuf::from(format!("pact-compatibility-suite/fixtures/pact-broker_c{}.json", num));
  let mut broker_pact = read_pact(&*f)
    .expect(format!("could not load fixture 'pact-broker_c{}.json'", num).as_str())
    .as_request_response_pact().unwrap();

  // AAARGH! My head. Adding a Pact Interaction to a Pact file for fetching a Pact file for verification
  let matching_rules = matchingrules! {
    "body" => { "$._links.pb:publish-verification-results.href" => [
      MatchingRule::Regex(format!(".*(\\/pacts\\/provider\\/p\\/consumer\\/c_{}\\/verification-results)", num))
    ] }
  };
  let generators = generators! {
    "BODY" => {
      "$._links.pb:publish-verification-results.href" => Generator::MockServerURL(
        format!("http://localhost:1234/pacts/provider/p/consumer/c_{}/verification-results", num),
        format!(".*(\\/pacts\\/provider\\/p\\/consumer\\/c_{}\\/verification-results)", num)
      )
    }
  };
  let interaction = RequestResponseInteraction {
    request: Request {
      path: format!("/pacts/provider/p/consumer/c_{}", num),
      .. Request::default()
    },
    response: Response {
      headers: Some(hashmap!{
        "content-type".to_string() => vec![ "application/json".to_string() ]
      }),
      body: OptionalBody::Present(Bytes::from(pact_json.to_string()),
                                  Some(JSON.clone()), None),
      matching_rules,
      generators,
      .. Response::default()
    },
    .. RequestResponseInteraction::default()
  };
  broker_pact.interactions.push(interaction);

  let config = MockServerConfig {
    .. MockServerConfig::default()
  };

  // let (mock_server, future) = MockServer::new(
  //   Uuid::new_v4().to_string(), broker_pact.boxed(), "127.0.0.1:0".parse()?, config
  // ).await.map_err(|err| anyhow!(err))?;
  // tokio::spawn(future);
  // let broker_port = {
  //   let ms = mock_server.lock().unwrap();
  //   ms.port
  // };
  let mock_server = MockServerBuilder::new()
    .with_v4_pact(broker_pact.as_v4_pact().unwrap())
    .with_id(Uuid::new_v4().to_string())
    .with_config(config)
    .bind_to("127.0.0.1:0")
    .with_transport("http")?
    .start()
    .await?;
  let broker_port = mock_server.port();
  world.mock_brokers.push(mock_server);

  world.sources.push(PactSource::BrokerWithDynamicConfiguration {
    provider_name: "p".to_string(),
    broker_url: format!("http://localhost:{}", broker_port),
    enable_pending: false,
    include_wip_pacts_since: None,
    provider_tags: vec![],
    provider_branch: None,
    selectors: vec![],
    auth: None,
    links: vec![],
  });
  Ok(())
}

#[then("a verification result will NOT be published back")]
fn a_verification_result_will_not_be_published_back(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let metrics = broker.metrics.lock().unwrap();
    let verification_requests = metrics.requests_by_path.iter()
      .find(|(path, _)| {
        path.ends_with("/verification-results")
      })
      .map(|(_, count)| *count)
      .unwrap_or(0);
    verification_requests > 0
  });
  if verification_results {
    Err(anyhow!("Was expecting no verification results"))
  } else {
    Ok(())
  }
}

#[given("publishing of verification results is enabled")]
fn publishing_of_verification_results_is_enabled(world: &mut ProviderWorld) {
  world.publish_options = Some(PublishOptions {
    provider_version: Some("1.2.3".to_string()),
    build_url: None,
    provider_tags: vec![],
    provider_branch: None,
  });
}

#[then(expr = "a successful verification result will be published back for interaction \\{{int}}")]
fn a_successful_verification_result_will_be_published_back_for_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()>  {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let vec = broker.matches();
    let verification_request = vec.iter()
      .find(|result| {
        let expected_path = format!("/pacts/provider/p/consumer/c_{}/verification-results", num);
        match result {
          MatchResult::RequestMatch(req, _, _) => req.path == expected_path,
          MatchResult::RequestMismatch(req, _, _) => req.path == expected_path,
          MatchResult::RequestNotFound(req) => req.path == expected_path,
          MatchResult::MissingRequest(req) => req.path == expected_path
        }
      });
    if let Some(result) = verification_request {
      match result {
        MatchResult::RequestMatch(req, _, _) => if let Some(body) = req.body.value() {
          if let Ok(json) = serde_json::from_slice::<Value>(body.as_ref()) {
            if let Some(success) = json.get("success") {
              match success {
                Value::Bool(b) => *b,
                _ => false
              }
            } else {
              false
            }
          } else {
            false
          }
        } else {
          false
        },
        _ => false
      }
    } else {
      false
    }
  });
  if verification_results {
    Ok(())
  } else {
    Err(anyhow!("Either no verification results was published, or it was incorrect"))
  }
}

#[then(expr = "a failed verification result will be published back for the interaction \\{{int}}")]
fn a_failed_verification_result_will_be_published_back_for_the_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()>  {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let vec = broker.matches();
    let verification_request = vec.iter()
      .find(|result| {
        let expected_path = format!("/pacts/provider/p/consumer/c_{}/verification-results", num);
        match result {
          MatchResult::RequestMatch(req, _, _) => req.path == expected_path,
          MatchResult::RequestMismatch(req, _, _) => req.path == expected_path,
          MatchResult::RequestNotFound(req) => req.path == expected_path,
          MatchResult::MissingRequest(req) => req.path == expected_path
        }
      });
    if let Some(result) = verification_request {
      match result {
        MatchResult::RequestMatch(req, _, _) => if let Some(body) = req.body.value() {
          if let Ok(json) = serde_json::from_slice::<Value>(body.as_ref()) {
            if let Some(success) = json.get("success") {
              match success {
                Value::Bool(b) => !*b,
                _ => false
              }
            } else {
              false
            }
          } else {
            false
          }
        } else {
          false
        },
        _ => false
      }
    } else {
      false
    }
  });
  if verification_results {
    Ok(())
  } else {
    Err(anyhow!("Either no verification results was published, or it was incorrect"))
  }
}

#[given("a provider state callback is configured")]
fn a_provider_state_callback_is_configured(world: &mut ProviderWorld) -> anyhow::Result<()> {
  world.provider_state_executor.set_fail_mode(false);
  Ok(())
}

#[given("a provider state callback is configured, but will return a failure")]
fn a_provider_state_callback_is_configured_but_will_return_a_failure(world: &mut ProviderWorld) -> anyhow::Result<()> {
  world.provider_state_executor.set_fail_mode(true);
  Ok(())
}

#[then("the provider state callback will be called before the verification is run")]
fn the_provider_state_callback_will_be_called_before_the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called"))
  }
}

#[then(expr = "the provider state callback will receive a setup call with {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_setup_call_with_as_the_provider_state_parameter(
  world: &mut ProviderWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called for state '{}'", state))
  }
}

#[then(expr = "the provider state callback will receive a setup call with {string} and the following parameters:")]
fn the_provider_state_callback_will_receive_a_setup_call_with_and_the_following_parameters(
  world: &mut ProviderWorld,
  step: &Step,
  state: String
) -> anyhow::Result<()> {
  validate_state_call(world, step, state, true)
}

#[then(expr = "the provider state callback will receive a teardown call {string} and the following parameters:")]
fn the_provider_state_callback_will_receive_a_teardown_call_with_and_the_following_parameters(
  world: &mut ProviderWorld,
  step: &Step,
  state: String
) -> anyhow::Result<()> {
  validate_state_call(world, step, state, false)
}

fn validate_state_call(world: &mut ProviderWorld, step: &Step, state: String, is_setup: bool) -> anyhow::Result<()> {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    if let Some(values) = table.rows.get(1) {
      let parameters = values.iter().enumerate()
        .map(|(index, v)| {
          let key = headers.get(&index).unwrap();
          let value = serde_json::from_str(v).unwrap();
          (key.clone(), value)
        })
        .collect::<HashMap<_, _>>();
      if world.provider_state_executor.was_called_for_state_with_params(state.as_str(), &parameters, is_setup) {
        Ok(())
      } else {
        Err(anyhow!("Provider state callback was not called for state '{}' with params {:?}", state, parameters))
      }
    } else {
      Err(anyhow!("No data table defined"))
    }
  } else {
    Err(anyhow!("No data table defined"))
  }
}

#[then("the provider state callback will be called after the verification is run")]
fn the_provider_state_callback_will_be_called_after_the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback teardown was not called"))
  }
}

#[then(expr = "the provider state callback will receive a teardown call {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_teardown_call_as_the_provider_state_parameter(
  world: &mut ProviderWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state teardown callback was not called for state '{}'", state))
  }
}

#[then("the provider state callback will NOT receive a teardown call")]
fn the_provider_state_callback_will_not_receive_a_teardown_call(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(false) {
    Err(anyhow!("Provider state callback teardown was called but was expecting no call"))
  } else {
    Ok(())
  }
}

#[then(expr = "a warning will be displayed that there was no provider state callback configured for provider state {string}")]
fn a_warning_will_be_displayed_that_there_was_no_provider_state_callback_configured(
  _world: &mut ProviderWorld,
  _state: String
) -> anyhow::Result<()> {
  // Unable to verify this, as the default provider state callback handler displays this message,
  // and this has been overwritten for the test suite. The verifier will not display it.
  Ok(())
}

#[given("a request filter is configured to make the following changes:")]
fn a_request_filter_is_configured_to_make_the_following_changes(
  world: &mut ProviderWorld,
  step: &Step
) -> anyhow::Result<()> {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    if let Some(values) = table.rows.get(1) {
      world.request_filter_data = values.iter().enumerate()
        .map(|(index, v)| (headers.get(&index).cloned(), v.clone()))
        .filter_map(|(k, v)| k.map(|k| (k.clone(), v.clone())))
        .collect();
      Ok(())
    } else {
      Err(anyhow!("No data table defined"))
    }
  } else {
    Err(anyhow!("No data table defined"))
  }
}

#[then(expr = "the request to the provider will contain the header {string}")]
fn the_request_to_the_provider_will_contain_the_header(
  world: &mut ProviderWorld,
  header: String
) -> anyhow::Result<()> {
  let header = header.splitn(2, ':')
    .map(|s| s.trim())
    .collect_vec();
  let matches = world.provider_server.matches();
  if matches.iter().all(|m| {
    let req = match m {
      MatchResult::RequestMatch(_, _, req) => req,
      MatchResult::RequestMismatch(_, req, _) => req,
      MatchResult::RequestNotFound(req) => req,
      MatchResult::MissingRequest(req) => req
    };
    if let Some(headers) = &req.headers {
      let key = header[0].to_lowercase();
      headers.contains_key(key.as_str()) && headers.get(key.as_str()).unwrap()[0] == header[1]
    } else {
      false
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Not all request to the provider contained the required header"))
  }
}
