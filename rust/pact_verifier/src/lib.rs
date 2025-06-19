//! The `pact_verifier` crate provides the core logic to performing verification of providers.
//! It implements the V3 (`https://github.com/pact-foundation/pact-specification/tree/version-3`)
//! and V4 Pact specification (`https://github.com/pact-foundation/pact-specification/tree/version-4`).
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use std::fs;
use std::future::Future;
use std::panic::RefUnwindSafe;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ansi_term::*;
use ansi_term::Colour::*;
use anyhow::anyhow;
use futures::stream::StreamExt;
use http::{header, HeaderMap};
use http::header::HeaderName;
use humantime::format_duration;
use itertools::Itertools;
#[cfg(feature = "plugins")] use itertools::Either;
use maplit::*;
use pact_models::generators::GeneratorTestMode;
use pact_models::http_utils::HttpAuth;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::pact::{load_pact_from_json, Pact, read_pact};
use pact_models::prelude::v4::SynchronousHttp;
use pact_models::provider_states::*;
use pact_models::v4::interaction::V4Interaction;
#[cfg(feature = "plugins")] use pact_plugin_driver::{catalogue_manager, plugin_manager};
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::{CatalogueEntry, CatalogueEntryProviderType};
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_manager::{load_plugin, shutdown_plugins};
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::{PluginDependency, PluginDependencyType};
#[cfg(feature = "plugins")] use pact_plugin_driver::verification::{InteractionVerificationData, InteractionVerificationDetails};
use regex::Regex;
use reqwest::Client;
use serde_json::{Map, Value};
#[cfg(feature = "plugins")] use serde_json::json;
use tracing::{debug, debug_span, error, info, Instrument, instrument, trace, warn};

pub use callback_executors::NullRequestFilterExecutor;
use callback_executors::RequestFilterExecutor;
use pact_matching::{match_response, Mismatch};
use pact_matching::metrics::{MetricEvent, send_metrics_async};

use crate::callback_executors::{ProviderStateError, ProviderStateExecutor};
use crate::messages::{process_message_result, process_sync_message_result, verify_message_from_provider, verify_sync_message_from_provider};
use crate::metrics::VerificationMetrics;
use crate::pact_broker::{
  Link,
  PactBrokerError,
  PactVerificationContext,
  publish_verification_results,
  TestResult
};
pub use crate::pact_broker::{ConsumerVersionSelector, PactsForVerificationRequest};
use crate::provider_client::make_provider_request;
use crate::request_response::process_request_response_result;
use crate::utils::as_safe_ref;
use crate::verification_result::{
  VerificationExecutionResult,
  VerificationInteractionResult,
  VerificationResult
};

pub mod provider_client;
pub mod pact_broker;
pub mod callback_executors;
mod request_response;
mod messages;
pub mod selectors;
pub mod metrics;
pub mod verification_result;
mod utils;

const VERIFIER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Source for loading pacts
#[derive(Debug, Clone)]
pub enum PactSource {
    /// Unknown pact source
    Unknown,
    /// Load the pact from a pact file
    File(String),
    /// Load all the pacts from a Directory
    Dir(String),
    /// Load the pact from a URL
    URL(String, Option<HttpAuth>),
    /// Load all pacts with the provider name from the pact broker url
    BrokerUrl(String, String, Option<HttpAuth>, Vec<Link>),
    /// Load pacts with the newer pacts for verification API
    BrokerWithDynamicConfiguration {
      /// Name of the provider as named in the Pact Broker
      provider_name: String,
      /// Base URL of the Pact Broker from which to retrieve the pacts
      broker_url: String,
      /// Allow pacts which are in pending state to be verified without causing the overall task to fail. For more information, see https://pact.io/pending
      enable_pending: bool,
      /// Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip
      include_wip_pacts_since: Option<String>,
      /// Provider tags to use in determining pending status for return pacts
      provider_tags: Vec<String>,
      /// Provider branch to use when publishing verification results
      provider_branch: Option<String>,
      /// The set of selectors that identifies which pacts to verify
      selectors: Vec<ConsumerVersionSelector>,
      /// HTTP authentication details for accessing the Pact Broker
      auth: Option<HttpAuth>,
      /// Links to the specific Pact resources. Internal field
      links: Vec<Link>
    },
    /// Load the Pact from some JSON (used for testing purposed)
    String(String),
    /// Load the given pact with the URL via a webhook callback
    WebhookCallbackUrl {
      /// URL to the Pact to verify
      pact_url: String,
      /// Base URL of the Pact Broker
      broker_url: String,
      /// HTTP authentication details for accessing the Pact Broker
      auth: Option<HttpAuth>
    }
}

impl Display for PactSource {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PactSource::File(file) => write!(f, "File({})", file),
      PactSource::Dir(dir) => write!(f, "Dir({})", dir),
      PactSource::URL(url, _) => write!(f, "URL({})", url),
      PactSource::BrokerUrl(provider_name, broker_url, _, _) => {
          write!(f, "PactBroker({}, provider_name='{}')", broker_url, provider_name)
      }
      PactSource::BrokerWithDynamicConfiguration {
        provider_name,
        broker_url,
        enable_pending,
        include_wip_pacts_since,
        provider_branch,
        provider_tags,
        selectors,
        auth, .. } => {
        if let Some(auth) = auth {
          write!(f, "PactBrokerWithDynamicConfiguration({}, provider_name='{}', enable_pending={}, include_wip_since={:?}, provider_tags={:?}, provider_branch={:?}, consumer_version_selectors='{:?}, auth={}')", broker_url, provider_name, enable_pending, include_wip_pacts_since, provider_tags, provider_branch, selectors, auth)
        } else {
          write!(f, "PactBrokerWithDynamicConfiguration({}, provider_name='{}', enable_pending={}, include_wip_since={:?}, provider_tags={:?}, provider_branch={:?}, consumer_version_selectors='{:?}, auth=None')", broker_url, provider_name, enable_pending, include_wip_pacts_since, provider_tags, provider_branch, selectors)
        }
      }
      PactSource::WebhookCallbackUrl { pact_url, auth, .. } => {
        if let Some(auth) = auth {
          write!(f, "WebhookCallbackUrl({}, auth={}')", pact_url, auth)
        } else {
          write!(f, "WebhookCallbackUrl({}, auth=None')", pact_url)
        }
      }
      _ => write!(f, "Unknown")
    }
  }
}

/// Information about the Provider to verify
#[derive(Debug, Clone)]
pub struct ProviderTransport {
  /// Protocol Transport
  pub transport: String,
  /// Port to use for the transport
  pub port: Option<u16>,
  /// Base path to use for the transport (for protocols that support paths)
  pub path: Option<String>,
  /// Transport scheme to use. Will default to HTTP
  pub scheme: Option<String>
}

impl ProviderTransport {
  /// Calculate a base URL for the transport
  pub fn base_url(&self, hostname: &str) -> String {
    let scheme = self.scheme.clone().unwrap_or("http".to_string());
    match self.port {
      Some(port) => format!("{}://{}:{}{}", scheme, hostname, port, self.path.clone().unwrap_or_default()),
      None => format!("{}://{}{}", scheme, hostname, self.path.clone().unwrap_or_default())
    }
  }
}

impl Default for ProviderTransport {
  fn default() -> Self {
    ProviderTransport {
      transport: "http".to_string(),
      port: Some(8080),
      path: None,
      scheme: Some("http".to_string())
    }
  }
}

/// Information about the Provider to verify
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider Name
    pub name: String,
    /// Provider protocol, defaults to HTTP
    #[deprecated(note = "Use transports instead")]
    pub protocol: String,
    /// Hostname of the provider
    pub host: String,
    /// Port the provider is running on, defaults to 8080
    #[deprecated(note = "Use transports instead")]
    pub port: Option<u16>,
    /// Base path for the provider, defaults to /
    #[deprecated(note = "Use transports instead")]
    pub path: String,
    /// Transports configured for the provider
    pub transports: Vec<ProviderTransport>
}

impl Default for ProviderInfo {
  /// Create a default provider info
  #[allow(deprecated)]
  fn default() -> ProviderInfo {
    ProviderInfo {
      name: "provider".to_string(),
      protocol: "http".to_string(),
      host: "localhost".to_string(),
      port: Some(8080),
      path: "/".to_string(),
      transports: vec![]
    }
  }
}

/// Result of performing a match
pub enum MismatchResult {
    /// Response mismatches
    Mismatches {
      /// Mismatches that occurred
      mismatches: Vec<Mismatch>,
      /// Expected Response/Message
      expected: Box<dyn Interaction + RefUnwindSafe>,
      /// Actual Response/Message
      actual: Box<dyn Interaction + RefUnwindSafe>,
      /// Interaction ID if fetched from a pact broker
      interaction_id: Option<String>
    },
    /// Error occurred
    Error(String, Option<String>)
}

impl MismatchResult {
  /// Return the interaction ID associated with the error, if any
  pub fn interaction_id(&self) -> Option<String> {
    match *self {
      MismatchResult::Mismatches { ref interaction_id, .. } => interaction_id.clone(),
      MismatchResult::Error(_, ref interaction_id) => interaction_id.clone()
    }
  }
}

impl From<ProviderStateError> for MismatchResult {
  fn from(error: ProviderStateError) -> Self {
    MismatchResult::Error(error.description, error.interaction_id)
  }
}

impl Debug for MismatchResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      MismatchResult::Mismatches { mismatches, expected, actual, interaction_id } => {
        if let Some(ref expected_reqres) = expected.as_request_response() {
          f.debug_struct("MismatchResult::Mismatches")
            .field("mismatches", mismatches)
            .field("expected", expected_reqres)
            .field("actual", &actual.as_request_response().unwrap())
            .field("interaction_id", interaction_id)
            .finish()
        } else if let Some(ref expected_message) = expected.as_message() {
          f.debug_struct("MismatchResult::Mismatches")
            .field("mismatches", mismatches)
            .field("expected", expected_message)
            .field("actual", &actual.as_message().unwrap())
            .field("interaction_id", interaction_id)
            .finish()
        } else {
          f.debug_struct("MismatchResult::Mismatches")
            .field("mismatches", mismatches)
            .field("expected", &"<UKNOWN TYPE>".to_string())
            .field("actual", &"<UKNOWN TYPE>".to_string())
            .field("interaction_id", interaction_id)
            .finish()
        }
      },
      MismatchResult::Error(error, opt) => {
        f.debug_tuple("MismatchResult::Error").field(error).field(opt).finish()
      }
    }
  }
}

impl Clone for MismatchResult {
  fn clone(&self) -> Self {
    match self {
      MismatchResult::Mismatches { mismatches, expected, actual, interaction_id } => {
        if expected.is_v4() {
          MismatchResult::Mismatches {
            mismatches: mismatches.clone(),
            expected: as_safe_ref(expected.as_ref()),
            actual: as_safe_ref(actual.as_ref()),
            interaction_id: interaction_id.clone()
          }
        } else if let Some(ref expected_reqres) = expected.as_request_response() {
          MismatchResult::Mismatches {
            mismatches: mismatches.clone(),
            expected: Box::new(expected_reqres.clone()),
            actual: Box::new(actual.as_request_response().unwrap().clone()),
            interaction_id: interaction_id.clone()
          }
        } else if let Some(ref expected_message) = expected.as_message() {
          MismatchResult::Mismatches {
            mismatches: mismatches.clone(),
            expected: Box::new(expected_message.clone()),
            actual: Box::new(actual.as_message().unwrap().clone()),
            interaction_id: interaction_id.clone()
          }
        } else {
          panic!("Cannot clone this MismatchResult::Mismatches as the expected and actual values are an unknown type")
        }
      },
      MismatchResult::Error(error, opt) => {
        MismatchResult::Error(error.clone(), opt.clone())
      }
    }
  }
}

async fn verify_response_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &SynchronousHttp,
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  options: &VerificationOptions<F>,
  client: &Client,
  verification_context: &HashMap<&str, Value>
) -> Result<Option<String>, MismatchResult> {
  let expected_response = &interaction.response;
  let request = pact_matching::generate_request(&interaction.request,
    &GeneratorTestMode::Provider, &verification_context).await;
  let transport = if let Some(transport) = &interaction.transport {
    provider.transports
      .iter()
      .find(|t| &t.transport == transport)
      .cloned()
  } else {
    provider.transports
      .iter()
      .find(|t| t.transport == "http")
      .cloned()
  }.map(|t| {
    if t.scheme.is_none() {
      ProviderTransport {
        scheme: Some("http".to_string()),
        .. t
      }
    } else {
      t
    }
  });
  match make_provider_request(provider, &request, options, client, transport).await {
    Ok(ref actual_response) => {
      let mismatches = match_response(expected_response.clone(), actual_response.clone(), pact, &interaction.boxed()).await;
      if mismatches.is_empty() {
        Ok(interaction.id.clone())
      } else {
        Err(MismatchResult::Mismatches {
          mismatches,
          expected: Box::new(interaction.clone()),
          actual: Box::new(SynchronousHttp { response: actual_response.clone(), .. SynchronousHttp::default() }),
          interaction_id: interaction.id.clone()
        })
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(err.to_string(), interaction.id.clone()))
    }
  }
}

async fn execute_state_change<S: ProviderStateExecutor>(
  provider_state: &ProviderState,
  setup: bool,
  interaction_id: Option<String>,
  client: &Client,
  provider_state_executor: Arc<S>
) -> Result<HashMap<String, Value>, MismatchResult> {
    let result = provider_state_executor.call(interaction_id, provider_state, setup, Some(client)).await;
    debug!("State Change: \"{:?}\" -> {:?}", provider_state, result);
    result.map_err(|err| {
      if let Some(err) = err.downcast_ref::<ProviderStateError>() {
        MismatchResult::Error(err.description.clone(), err.interaction_id.clone())
      } else {
        MismatchResult::Error(err.to_string(), None)
      }
    })
}

/// Main implementation for verifying an interaction. Will return a tuple containing the
/// result of the verification and any output collected plus the time taken to execute
#[tracing::instrument(level = "trace", skip_all)]
async fn verify_interaction<'a, F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider: &ProviderInfo,
  interaction: &(dyn Interaction + Send + Sync + RefUnwindSafe),
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  options: &VerificationOptions<F>,
  provider_state_executor: &Arc<S>
) -> Result<(Option<String>, Vec<String>, Duration), (MismatchResult, Vec<String>, Duration)> {
  let start = Instant::now();
  debug!("Verifying interaction {} {} ({:?})", interaction.type_of(), interaction.description(), interaction.id());
  let client = Arc::new(configure_http_client(options)
    .map_err(|err| (
      MismatchResult::Error(err.to_string(), interaction.id()),
      vec![],
      start.elapsed()
    ))?);

  debug!("Executing provider states");
  let context = execute_provider_states(interaction, provider_state_executor, &client, true)
    .await
    .map_err(|e| (e, vec![], start.elapsed()))?;
  let mut provider_states_context = hashmap!{};
  for provider_state in interaction.provider_states() {
    for (k, v) in provider_state.params {
      provider_states_context.insert(k, v);
    }
  }
  for (k, v) in context {
    provider_states_context.insert(k, v);
  }
  let provider_states_context = provider_states_context
    .iter()
    .map(|(k, v)| (k.as_str(), v.clone()))
    .collect();

  info!("Running provider verification for '{}'", interaction.description());
  trace!(?provider_states_context, "Interaction to verify: {:?}", interaction);

  #[allow(unused_assignments)] let mut result = Ok((None, vec![]));
  #[cfg(feature = "plugins")]
  {
    let transport = if interaction.is_v4() {
      interaction.as_v4()
        .and_then(|i| i.transport())
        .and_then(|t| catalogue_manager::lookup_entry(&*format!("transport/{}", t)))
    } else {
      None
    };

    result = if let Some(transport) = &transport {
      trace!("Verifying interaction via {}", transport.key);
      verify_interaction_using_transport(transport, provider, interaction, pact, options, &client, &provider_states_context).await
    } else {
      verify_v3_interaction(provider, interaction, &pact, options, &client, &provider_states_context)
        .await
        .map(|r| (r, vec![]))
        .map_err(|e| (e, vec![]))
    };
  }

  #[cfg(not(feature = "plugins"))]
  {
    result = verify_v3_interaction(provider, interaction, &pact, options, &client, &provider_states_context)
      .await
      .map(|r| (r, vec![]))
      .map_err(|e| (e, vec![]));
  }

  if provider_state_executor.teardown() {
    execute_provider_states(interaction, provider_state_executor, &client, false)
      .await
      .map_err(|e| (e, vec![], start.elapsed()))?;
  }

  result
    .map(|(id, output)| (id, output, start.elapsed()))
    .map_err(|(result, output)| (result, output, start.elapsed()))
}

/// Verify an interaction using the provided transport
#[cfg(feature = "plugins")]
async fn verify_interaction_using_transport<'a, F: RequestFilterExecutor>(
  transport_entry: &CatalogueEntry,
  provider: &ProviderInfo,
  interaction: &(dyn Interaction + Send + Sync + RefUnwindSafe),
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  options: &VerificationOptions<F>,
  client: &Arc<Client>,
  config: &HashMap<&str, Value>
) -> Result<(Option<String>, Vec<String>), (MismatchResult, Vec<String>)> {
  if transport_entry.provider_type == CatalogueEntryProviderType::PLUGIN {
    match pact.as_v4_pact() {
      Ok(pact) => {
        let mut context = hashmap!{
          "host".to_string() => Value::String(provider.host.clone())
        };

        #[allow(deprecated)]
        let port = provider.transports.iter()
          .find_map(|transport| {
            if transport_entry.key.ends_with(&transport.transport) {
              transport.port
            } else {
              None
            }
          })
          .or_else(|| provider.port);
        if let Some(port) = port {
          context.insert("port".to_string(), json!(port));
        }

        let mut psc = Map::new();
        for (k, v) in config {
          psc.insert(k.to_string(), v.clone());
        }
        context.insert("providerState".to_string(), Value::Object(psc));

        // Get plugin to prepare the request data
        let v4_interaction = interaction.as_v4().unwrap();
        let InteractionVerificationData { request_data, mut metadata } = plugin_manager::prepare_validation_for_interaction(transport_entry, &pact,
          v4_interaction.as_ref(), &context)
          .await
          .map_err(|err| {
            (MismatchResult::Error(format!("Failed to prepare interaction for verification - {err}"), interaction.id()), vec![])
          })?;

        // If any custom headers have been setup, add them to the metadata
        if !options.custom_headers.is_empty() {
          for (key, val) in &options.custom_headers {
            metadata.insert(key.clone(), Either::Left(Value::String(val.to_string())));
          }
        }

        // Invoke any callback to mutate the data
        let (request_body, request_metadata) = if let Some(filter) = &options.request_filter {
          info!("Invoking request filter for request data");
          filter.call_non_http(&request_data, &metadata)
        } else {
          (request_data.clone(), metadata.clone())
        };

        // Get the plugin to verify the request
        match plugin_manager::verify_interaction(
          transport_entry,
          &InteractionVerificationData::new(request_body, request_metadata),
          &context,
          &pact,
          v4_interaction.as_ref()
        ).await {
          Ok(result) => if result.ok {
            Ok((interaction.id(), result.output))
          } else {
            Err((MismatchResult::Mismatches {
              mismatches: result.details.iter().filter_map(|mismatch| match mismatch {
                InteractionVerificationDetails::Error(err) => {
                  error!("Individual mismatch is an error: {err}");
                  None // TODO: matching crate does not support storing an error against an item
                }
                InteractionVerificationDetails::Mismatch { expected, actual, mismatch, path } => {
                  Some(Mismatch::BodyMismatch {
                    path: path.clone(),
                    expected: Some(expected.clone()),
                    actual: Some(actual.clone()),
                    mismatch: mismatch.clone()
                  })
                }
              }).collect(),
              expected: as_safe_ref(interaction),
              actual: as_safe_ref(interaction),
              interaction_id: interaction.id()
            }, result.output))
          }
          Err(err) => {
            Err((MismatchResult::Error(format!("Verification failed with an error - {err}"), interaction.id()), vec![]))
          }
        }
      },
      Err(err) => {
        Err((MismatchResult::Error(format!("Pacts must be V4 format to work with plugins - {err}"), interaction.id()), vec![]))
      }
    }
  } else {
    verify_v3_interaction(provider, interaction, pact, options, client, config)
      .await
      .map(|r| (r, vec![]))
      .map_err(|e| (e, vec![]))
  }
}

/// Previous implementation (V3) of verification
async fn verify_v3_interaction<'a, F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &(dyn Interaction + Send + Sync + RefUnwindSafe),
  pact: &Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  options: &VerificationOptions<F>,
  client: &Arc<Client>,
  provider_states_context: &HashMap<&str, Value>
) -> Result<Option<String>, MismatchResult> {
  let mut result = Err(MismatchResult::Error("No interaction was verified".into(), interaction.id().clone()));

  // Verify an HTTP interaction
  if let Some(interaction) = interaction.as_v4_http() {
    debug!("Verifying a HTTP interaction");
    result = verify_response_from_provider(provider, &interaction, &pact.boxed(), options,
                                           &client, &provider_states_context).await;
  }
  // Verify an asynchronous message (single shot)
  if interaction.is_message() {
    debug!("Verifying an asynchronous message (single shot)");
    result = verify_message_from_provider(provider, pact, &interaction.boxed(), options,
                                          &client, &provider_states_context).await;
  }
  // Verify a synchronous message (request/response)
  if let Some(message) = interaction.as_v4_sync_message() {
    debug!("Verifying a synchronous message (request/response)");
    result = verify_sync_message_from_provider(provider, pact, message, options, &client,
                                               &provider_states_context).await;
  }

  result
}

/// Executes the provider states, returning a map of the results
#[instrument(ret, skip_all, fields(?interaction, is_setup), level = "trace")]
async fn execute_provider_states<S: ProviderStateExecutor>(
  interaction: &(dyn Interaction + Send + Sync + RefUnwindSafe),
  provider_state_executor: &Arc<S>,
  client: &Arc<Client>,
  is_setup: bool
) -> Result<HashMap<String, Value>, MismatchResult> {
  let mut provider_states_results = hashmap!{};

  let sc_type = if is_setup { "setup" } else { "teardown" };
  let mut sc_results = vec![];

  if interaction.provider_states().is_empty() {
    info!("Running {} provider state change handler with empty state for '{}'", sc_type, interaction.description());
    match execute_state_change(&ProviderState::default(""), is_setup, interaction.id(), client,
                               provider_state_executor.clone()).await {
      Ok(data) => {
        sc_results.push(Ok(data));
      }
      Err(err) => {
        error!("Provider {} state change for has failed - {:?}", sc_type, err);
        sc_results.push(Err(err));
      }
    }
  } else {
    for state in &interaction.provider_states() {
      info!("Running {} provider state change handler '{}' for '{}'", sc_type, state.name, interaction.description());
      match execute_state_change(state, is_setup, interaction.id(), client,
                                 provider_state_executor.clone()).await {
        Ok(data) => {
          sc_results.push(Ok(data));
        }
        Err(err) => {
          error!("Provider {} state change for '{}' has failed - {:?}", sc_type, state.name, err);
          sc_results.push(Err(err));
        }
      }
    }
  }

  if sc_results.iter().any(|result| result.is_err()) {
    return Err(MismatchResult::Error(
      format!("One or more of the {} state change handlers has failed", sc_type), interaction.id()))
  } else {
    for result in sc_results {
      if let Ok(data) = result {
        for (k, v) in data {
          provider_states_results.insert(k, v);
        }
      }
    }
  };

  Ok(provider_states_results)
}

/// Configure the HTTP client to use for requests to the provider
pub(crate) fn configure_http_client<F: RequestFilterExecutor>(
  options: &VerificationOptions<F>
) -> anyhow::Result<Client> {
  let mut client_builder = reqwest::Client::builder()
    .danger_accept_invalid_certs(options.disable_ssl_verification)
    .timeout(Duration::from_millis(options.request_timeout));

  if !options.custom_headers.is_empty() {
    let headers = setup_custom_headers(&options.custom_headers)?;
    client_builder = client_builder.default_headers(headers);
  }

  client_builder.build().map_err(|err| anyhow!(err))
}

fn setup_custom_headers(custom_headers: &HashMap<String, String>) -> anyhow::Result<HeaderMap> {
  let mut headers = header::HeaderMap::new();
  for (key, value) in custom_headers {
    let header_name = match HeaderName::try_from(key) {
      Ok(name) => name,
      Err(err) => {
        return Err(anyhow!("Custom header '{key}' is invalid. Only ASCII characters (32-127) are permitted - {err}"));
      }
    };
    let header_value = match header::HeaderValue::from_str(value.as_str()) {
      Ok(value) => value,
      Err(err) => {
        return Err(anyhow!("Custom header '{key}' has an invalid value '{value}'. Only ASCII characters (32-127) are permitted - {err}"));
      }
    };
    headers.append(header_name, header_value);
  }
  Ok(headers)
}

fn generate_display_for_result(
  status: u16,
  status_result: ANSIGenericString<str>,
  header_results: Option<Vec<(String, String, ANSIGenericString<str>)>>,
  body_result: ANSIGenericString<str>,
  output: &mut Vec<String>,
  coloured: bool
) {
  output.push("    returns a response which".to_string());
  let style = if coloured { Style::new().bold() } else { Style::new() };
  output.push(format!("      has status code {} ({})", style.paint(format!("{}", status)),
      status_result));
  if let Some(header_results) = header_results {
    output.push("      includes headers".to_string());
    for (key, value, result) in header_results {
      output.push(format!("        \"{}\" with value \"{}\" ({})", style.paint(key),
               style.paint(value), result));
    }
  }
  output.push(format!("      has a matching body ({})", body_result));
}

fn walkdir(
  dir: &Path,
  provider: &ProviderInfo
) -> anyhow::Result<Vec<anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Duration)>>> {
    let mut pacts = vec![];
    debug!("Scanning {:?}", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walkdir(&path, provider)?;
        } else {
          match timeit(|| read_pact(&path)) {
            Ok((pact, tm)) => {
              if pact.provider().name == provider.name {
                pacts.push(Ok((pact, tm)));
              }
            }
            Err(err) => pacts.push(Err(err))
          };
        }
    }
    Ok(pacts)
}

fn display_body_mismatch(
  expected: &dyn Interaction,
  actual: &dyn Interaction,
  path: &str,
  output: &mut Vec<String>
) {
  if expected.contents_for_verification().content_type().unwrap_or_default().is_json() {
    output.push(pact_matching::json::display_diff(
      &expected.contents_for_verification().display_string().to_string(),
      &actual.contents_for_verification().display_string().to_string(),
      path, "    "));
  }
}

/// Filter information used to filter the interactions that are verified
#[derive(Debug, Clone)]
pub enum FilterInfo {
    /// No filter, all interactions will be verified
    None,
    /// Filter on the interaction description
    Description(String),
    /// Filter on the interaction provider state
    State(String),
    /// Filter on both the interaction description and provider state
    DescriptionAndState(String, String)
}

impl FilterInfo {

    /// If this filter is filtering on description
    pub fn has_description(&self) -> bool {
        match *self {
            FilterInfo::Description(_) => true,
            FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// If this filter is filtering on provider state
    pub fn has_state(&self) -> bool {
        match *self {
            FilterInfo::State(_) => true,
            FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// Value of the state to filter
    pub fn state(&self) -> String {
        match *self {
            FilterInfo::State(ref s) => s.clone(),
            FilterInfo::DescriptionAndState(_, ref s) => s.clone(),
            _ => String::default()
        }
    }

    /// Value of the description to filter
    pub fn description(&self) -> String {
        match *self {
            FilterInfo::Description(ref s) => s.clone(),
            FilterInfo::DescriptionAndState(ref s, _) => s.clone(),
            _ => String::default()
        }
    }

    /// If the filter matches the interaction provider state using a regular expression. If the
    /// filter value is the empty string, then it will match interactions with no provider state.
    ///
    /// # Panics
    /// If the state filter value can't be parsed as a regular expression
    pub fn match_state(&self, interaction: &dyn Interaction) -> bool {
      if !interaction.provider_states().is_empty() {
        if self.state().is_empty() {
          false
        } else {
          let re = Regex::new(&self.state()).unwrap();
          interaction.provider_states().iter().any(|state| re.is_match(&state.name))
        }
      } else {
        self.has_state() && self.state().is_empty()
      }
    }

    /// If the filter matches the interaction description using a regular expression
    ///
    /// # Panics
    /// If the description filter value can't be parsed as a regular expression
    pub fn match_description(&self, interaction: &dyn Interaction) -> bool {
      let re = Regex::new(&self.description()).unwrap();
      re.is_match(&interaction.description())
    }
}

fn filter_interaction(interaction: &dyn Interaction, filter: &FilterInfo) -> bool {
  if filter.has_description() && filter.has_state() {
    filter.match_description(interaction) && filter.match_state(interaction)
  } else if filter.has_description() {
    filter.match_description(interaction)
  } else if filter.has_state() {
    filter.match_state(interaction)
  } else {
    true
  }
}

fn filter_consumers(
  consumers: &[String],
  res: &anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Option<PactVerificationContext>, PactSource, Duration)>
) -> bool {
  consumers.is_empty() || res.is_err() || consumers.contains(&res.as_ref().unwrap().0.consumer().name)
}

/// Options for publishing results to the Pact Broker
#[derive(Debug, Clone)]
pub struct PublishOptions {
  /// Provider version being published
  pub provider_version: Option<String>,
  /// Build URL to associate with the published results
  pub build_url: Option<String>,
  /// Tags to use when publishing results
  pub provider_tags: Vec<String>,
  /// Provider branch used when publishing results
  pub provider_branch: Option<String>,
}

impl Default for PublishOptions {
  fn default() -> Self {
    PublishOptions {
      provider_version: None,
      build_url: None,
      provider_tags: vec![],
      provider_branch: None,
    }
  }
}

/// Options to use when running the verification
#[derive(Debug, Clone)]
pub struct VerificationOptions<F> where F: RequestFilterExecutor {
  /// Request filter callback
  pub request_filter: Option<Arc<F>>,
  /// Ignore invalid/self-signed SSL certificates
  pub disable_ssl_verification: bool,
  /// Timeout in ms for verification requests and state callbacks
  pub request_timeout: u64,
  /// Custom headers to be added to the requests to the provider
  pub custom_headers: HashMap<String, String>,
  /// If coloured output should be used (using ANSI escape codes)
  pub coloured_output: bool,
  /// If no pacts are found to verify, then this should be an error
  pub no_pacts_is_error: bool,
  /// Exit verification run on first failure
  pub exit_on_first_failure: bool,
  /// Only execute the interactions that failed on the previous verifier run
  pub run_last_failed_only: bool
}

impl <F: RequestFilterExecutor> Default for VerificationOptions<F> {
  fn default() -> Self {
    VerificationOptions {
      request_filter: None,
      disable_ssl_verification: false,
      request_timeout: 5000,
      custom_headers: Default::default(),
      coloured_output: true,
      no_pacts_is_error: true,
      exit_on_first_failure: false,
      run_last_failed_only: false
    }
  }
}

const VERIFICATION_NOTICE_BEFORE: &str = "before_verification";
const VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_PUBLISH: &str = "after_verification:success_true_published_true";
const VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_NO_PUBLISH: &str = "after_verification:success_true_published_false";
const VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_PUBLISH: &str = "after_verification:success_false_published_true";
const VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_NO_PUBLISH: &str = "after_verification:success_false_published_false";

fn process_notices(context: &Option<PactVerificationContext>, stage: &str, result: &mut VerificationExecutionResult) {
  if let Some(c) = context {
    result.notices = c.verification_properties.notices.clone();
    for notice in &c.verification_properties.notices {
      if let Some(when) = notice.get("when") {
        if when.as_str() == stage {
          result.output.push(notice.get("text").unwrap_or(&"".to_string()).clone());
        }
      }
    }
  }
}

/// Verify the provider with the given pact sources.
pub fn verify_provider<F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider_info: ProviderInfo,
  source: Vec<PactSource>,
  filter: FilterInfo,
  consumers: Vec<String>,
  verification_options: &VerificationOptions<F>,
  publish_options: Option<&PublishOptions>,
  provider_state_executor: &Arc<S>,
  metrics_data: Option<VerificationMetrics>
) -> anyhow::Result<bool> {
  match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
    Ok(runtime) => runtime.block_on(
      verify_provider_async(provider_info, source, filter, consumers, verification_options, publish_options, provider_state_executor, metrics_data)
    ).map(|result| result.result),
    Err(err) => {
      error!("Verify provider process failed to start the tokio runtime: {}", err);
      Ok(false)
    }
  }
}

/// Verify the provider with the given pact sources (async version)
pub async fn verify_provider_async<F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider_info: ProviderInfo,
  source: Vec<PactSource>,
  filter: FilterInfo,
  consumers: Vec<String>,
  verification_options: &VerificationOptions<F>,
  publish_options: Option<&PublishOptions>,
  provider_state_executor: &Arc<S>,
  metrics_data: Option<VerificationMetrics>
) -> anyhow::Result<VerificationExecutionResult> {
  pact_matching::matchers::configure_core_catalogue();
  async {
    let pact_results = fetch_pacts(source, consumers, &provider_info).await;

    let mut total_results = 0;
    let mut pending_errors: Vec<(String, MismatchResult)> = vec![];
    let mut errors: Vec<(String, MismatchResult)> = vec![];

    let mut verification_result = VerificationExecutionResult::new();

    for pact_result in pact_results {
      match pact_result {
        Ok((pact, context, pact_source, pact_source_duration)) => {
          trace!("Pact file took {} to load", format_duration(pact_source_duration));

          #[cfg(feature = "plugins")]
          if pact.requires_plugins() {
            info!("Pact file requires plugins, will load those now");
            for plugin_details in pact.plugin_data() {
              let version = plugin_details.version.split('.')
                .take(3)
                .join(".");
              load_plugin(&PluginDependency {
                name: plugin_details.name.clone(),
                version: Some(version),
                dependency_type: PluginDependencyType::Plugin
              }).await?;
            }
          }

          process_notices(&context, VERIFICATION_NOTICE_BEFORE, &mut verification_result);

          if verification_options.coloured_output {
            verification_result.output.push(format!("\nVerifying a pact between {} and {}",
              Style::new().bold().paint(pact.consumer().name.clone()),
              Style::new().bold().paint(pact.provider().name.clone())));
          } else {
            verification_result.output.push(format!("\nVerifying a pact between {} and {}",
              pact.consumer().name, pact.provider().name));
          }

          if pact.interactions().is_empty() {
            if verification_options.coloured_output {
              verification_result.output.push(
                Yellow.paint("WARNING: Pact file has no interactions").to_string()
              );
            } else {
              verification_result.output.push("WARNING: Pact file has no interactions".to_string());
            }
          } else {
            let pending = match &context {
              Some(context) => context.verification_properties.pending,
              None => false
            };
            let verify_result = verify_pact_internal(
              &provider_info,
              &filter,
              pact,
              &verification_options,
              &provider_state_executor.clone(),
              pending,
              pact_source_duration
            ).await;

            let mut results = vec![];
            match &verify_result {
              Ok(result) => {
                for interaction_result in &result.results {
                  results.push(interaction_result.clone());
                  if let Err(error) = &interaction_result.result {
                    if interaction_result.pending {
                      pending_errors.push((interaction_result.description.clone(), error.clone()));
                    } else {
                      errors.push((interaction_result.description.clone(), error.clone()));
                    }
                  }
                }

                for output in &result.output {
                  verification_result.output.push(output.clone());
                }
              }
              Err(err) => {
                if pending {
                  pending_errors.push(("Could not verify the provided pact".to_string(),
                                       MismatchResult::Error(err.to_string(), None)));
                } else {
                  errors.push(("Could not verify the provided pact".to_string(),
                               MismatchResult::Error(err.to_string(), None)));
                }
              }
            }

            total_results += results.len();
            verification_result.interaction_results.extend_from_slice(results.as_slice());

            if verification_options.exit_on_first_failure && !errors.is_empty() {
              warn!("Executing verification on first failure (--exit-first is set)");
              break;
            }

            if let Some(publish) = publish_options {
              publish_result(results.as_slice(), &pact_source, &publish, metrics_data.as_ref()).await;

              if !errors.is_empty() || !pending_errors.is_empty() {
                process_notices(&context, VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_PUBLISH, &mut verification_result);
              } else {
                process_notices(&context, VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_PUBLISH, &mut verification_result);
              }
            } else {
              if !errors.is_empty() || pending_errors.is_empty() {
                process_notices(&context, VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_NO_PUBLISH, &mut verification_result);
              } else {
                process_notices(&context, VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_NO_PUBLISH, &mut verification_result);
              }
            }
          }
        },
        Err(err) => {
          if let Some(PactBrokerError::NotFound(_)) = err.downcast_ref() {
            if verification_options.no_pacts_is_error {
              error!("Failed to load pact - {}", Red.paint(err.to_string()));
              errors.push(("Failed to load pact".to_string(), MismatchResult::Error(err.to_string(), None)));
            } else {
              warn!("Ignoring no pacts error - {}", Yellow.paint(err.to_string()));
            }
          } else {
            let error = format!("{:#}", err);
            error!("Failed to load pact - {}", Red.paint(error.clone()));
            errors.push(("Failed to load pact".to_string(), MismatchResult::Error(error, None)));
          }
        }
      }
    };

    let metrics_data = metrics_data.unwrap_or_else(|| VerificationMetrics {
      test_framework: "pact-rust".to_string(),
      app_name: "pact_verifier".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    });
    send_metrics_async(MetricEvent::ProviderVerificationRan {
      tests_run: total_results,
      test_framework: metrics_data.test_framework,
      app_name: metrics_data.app_name,
      app_version: metrics_data.app_version
    }).await;

    for (error, result) in &errors {
      verification_result.errors.push((error.clone(), result.into()));
    }
    for (error, result) in &pending_errors {
      verification_result.pending_errors.push((error.clone(), result.into()));
    }

    if !pending_errors.is_empty() {
      verification_result.output.push("\nPending Failures:\n".to_string());
      process_errors(&pending_errors, &mut verification_result.output, verification_options.coloured_output);
      verification_result.output.push(format!("\nThere were {} non-fatal pact failures on pending pacts or interactions (see docs.pact.io/pending for more information)\n", pending_errors.len()));
    }

    if !errors.is_empty() {
      verification_result.output.push("\nFailures:\n".to_string());
      process_errors(&errors, &mut verification_result.output, verification_options.coloured_output);
      verification_result.output.push(format!("\nThere were {} pact failures\n", errors.len()));
      verification_result.result = false;
    } else {
      verification_result.output.push(String::default());
      verification_result.result = true;
    };

    for line in &verification_result.output {
      println!("{line}");
    }

    #[cfg(feature = "plugins")] shutdown_plugins();

    Ok(verification_result)
  }.instrument(tracing::trace_span!("verify_provider_async")).await
}

fn process_errors(errors: &Vec<(String, MismatchResult)>, output: &mut Vec<String>, coloured_output: bool) {
  for (i, &(ref description, ref mismatch)) in errors.iter().enumerate() {
    match *mismatch {
        MismatchResult::Error(ref err, _) => output.push(format!("{}) {} - {}\n", i + 1, description, err)),
        MismatchResult::Mismatches { ref mismatches, ref expected, ref actual, .. } => {
          interaction_mismatch_output(output, coloured_output, i, description, mismatches, expected.as_ref(), actual.as_ref())
        }
    }
  }
}

/// Generate the output for an interaction verification
pub fn interaction_mismatch_output(
  output: &mut Vec<String>,
  coloured_output: bool,
  i: usize,
  description: &String,
  mismatches: &Vec<Mismatch>,
  expected: &dyn Interaction,
  actual: &dyn Interaction
) {
  output.push(format!("{}) {}", i + 1, description));

  let mut j = 1;
  for (_, mut mismatches) in &mismatches.into_iter().chunk_by(|m| m.mismatch_type()) {
    let mismatch = mismatches.next().unwrap();
    output.push(format!("    {}.{}) {}", i + 1, j, mismatch.summary()));
    output.push(format!("           {}", if coloured_output { mismatch.ansi_description() } else { mismatch.description() }));
    for mismatch in mismatches.sorted_by(|m1, m2| {
      match (m1, m2) {
        (Mismatch::QueryMismatch { parameter: p1, .. }, Mismatch::QueryMismatch { parameter: p2, .. }) => Ord::cmp(&p1, &p2),
        (Mismatch::HeaderMismatch { key: p1, .. }, Mismatch::HeaderMismatch { key: p2, .. }) => Ord::cmp(&p1, &p2),
        (Mismatch::BodyMismatch { path: p1, .. }, Mismatch::BodyMismatch { path: p2, .. }) => Ord::cmp(&p1, &p2),
        (Mismatch::MetadataMismatch { key: p1, .. }, Mismatch::MetadataMismatch { key: p2, .. }) => Ord::cmp(&p1, &p2),
        _ => Ord::cmp(m1, m2)
      }
    }) {
      output.push(format!("           {}", if coloured_output { mismatch.ansi_description() } else { mismatch.description() }));
    }

    if let Mismatch::BodyMismatch { path, .. } = mismatch {
      display_body_mismatch(expected, actual, path, output);
    }

    j += 1;
  }
}

#[tracing::instrument(level = "trace")]
async fn fetch_pact(
  source: PactSource,
  provider: &ProviderInfo
) -> Vec<anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Option<PactVerificationContext>, PactSource, Duration)>> {
  trace!("fetch_pact(source={})", source);

  match &source {
    PactSource::File(file) => vec![
      timeit(|| read_pact(Path::new(&file)))
        .map_err(|err| anyhow!("Failed to load pact '{}' - {}", file, err))
        .map(|(pact, tm)| {
          trace!(%file, duration = ?tm, "Loaded pact from file");
          (pact, None, source.clone(), tm)
        })
    ],
    PactSource::Dir(dir) => match walkdir(Path::new(dir), provider) {
      Ok(pact_results) => pact_results.into_iter().map(|pact_result| {
          match pact_result {
              Ok((pact, tm)) => {
                trace!(%dir, duration = ?tm, "Loaded pact from directory");
                Ok((pact, None, source.clone(), tm))
              },
              Err(err) => Err(anyhow!("Failed to load pact from '{}' - {}", dir, err))
          }
      }).collect(),
      Err(err) => vec![Err(anyhow!("Could not load pacts from directory '{}' - {}", dir, err))]
    },
    PactSource::URL(url, auth) => vec![
      timeit_async(pact_broker::fetch_pact_from_url(url, auth)).await
        .map_err(|err| anyhow!("Failed to load pact '{}' - {}", url, err))
        .map(|((pact, links), tm)| {
          trace!(%url, duration = ?tm, "Loaded pact from url");
          if is_pact_broker_source(&links) {
            let provider = pact.provider();
            let base_url = url.parse::<reqwest::Url>()
              .map(|mut url| {
                url.set_path("/");
                url.to_string()
              }).ok().unwrap_or_else(||url.to_string());
            (pact, None, PactSource::BrokerUrl(provider.name.clone(), base_url,
                                               auth.clone(), links.clone()), tm)
          } else {
            (pact, None, source.clone(), tm)
          }
        })
    ],
    PactSource::BrokerUrl(provider_name, broker_url, auth, _) => {
      let result = timeit_async(pact_broker::fetch_pacts_from_broker(
        broker_url.as_str(),
        provider_name.as_str(),
        auth.clone()
      )).await;

      match result {
        Ok((ref pacts, tm)) => {
          trace!(%broker_url, duration = ?tm, "Loaded pacts from pact broker");
          let mut buffer = vec![];
          for result in pacts.iter() {
            match result {
              Ok((pact, context, links)) => {
                trace!("Got pact with links {:?}", pact);
                buffer.push(Ok((
                  pact.boxed(),
                  context.clone(),
                  PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()),
                  tm
                )));
              },
              &Err(ref err) => buffer.push(Err(anyhow!("Failed to load pact from '{}' - {:?}", broker_url, err)))
            }
          }
          buffer
        },
        Err(err) => {
          error!("No pacts found for provider '{}' in pact broker '{}': {}", provider_name, broker_url, err);
          vec![
            Err(anyhow!(err).context(format!("No pacts found for provider '{}' in pact broker '{}'", provider_name, broker_url)))
          ]
        }
      }
    },
    PactSource::BrokerWithDynamicConfiguration {
      provider_name, broker_url, enable_pending, include_wip_pacts_since,
      provider_tags, provider_branch, selectors,
      auth, links: _ } => {
      let result = timeit_async(pact_broker::fetch_pacts_dynamically_from_broker(
        broker_url.as_str(),
        provider_name.clone(),
        *enable_pending,
        include_wip_pacts_since.clone(),
        provider_tags.clone(),
        provider_branch.clone(),
        selectors.clone(),
        auth.clone()
      )).await;

      match result {
        Ok((pacts, tm)) => {
          trace!(%broker_url, duration = ?tm, "Loaded pacts from pact broker");
          let mut buffer = vec![];
          for result in pacts.iter() {
            match result {
              Ok((pact, context, links)) => {
                trace!(?links, ?pact, "Got pact with links");
                buffer.push(Ok((
                  pact.boxed(),
                  context.clone(),
                  PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()),
                  tm.clone()
                )));
              },
              Err(err) => buffer.push(Err(anyhow::Error::new(err.clone()).context(format!("Failed to load pact from '{}'", broker_url))))
            }
          }
          buffer
        },
        Err(err) => {
          error!("No pacts found for provider '{}' matching the given consumer version selectors in pact broker '{}': {}", provider_name, broker_url, err);
          vec![
            Err(err.context(format!("No pacts found for provider '{}' matching the given consumer version selectors in pact broker '{}'", provider_name, broker_url)))
          ]
        }
      }
    },
    PactSource::String(json) => vec![
      timeit(|| serde_json::from_str(json)
        .map_err(|err| anyhow!(err))
        .and_then(|json| load_pact_from_json("<json>", &json)))
        .map(|(pact, tm)| {
          (pact, None, source.clone(), tm)
        })
    ],
    PactSource::WebhookCallbackUrl { pact_url, broker_url, auth, .. } => vec![
      timeit_async(pact_broker::fetch_pact_from_url(pact_url, auth)).await
        .map_err(|err| anyhow!("Failed to load pact '{}' - {}", pact_url, err))
        .map(|((pact, links), tm)| {
          trace!(%pact_url, duration = ?tm, "Loaded pact from url");
          let provider = pact.provider();
          (pact, None, PactSource::BrokerUrl(provider.name.clone(), broker_url.clone(),
            auth.clone(), links.clone()), tm)
        })
    ],
    _ => vec![Err(anyhow!("Could not load pacts, unknown pact source {}", source))]
  }
}

fn timeit<T, FN: FnOnce() -> anyhow::Result<T>>(callback: FN) -> anyhow::Result<(T, Duration)> {
  let start = Instant::now();
  let result = callback()?;
  Ok((result, start.elapsed()))
}

async fn timeit_async<T, F>(future: F) -> anyhow::Result<(T, Duration)>
where F: Future<Output = anyhow::Result<T>> + Send
{
  let start = Instant::now();
  let result = future.await?;
  Ok((result, start.elapsed()))
}

// Checks if any of Pactbroker links exist. Actually looks for the pb:publish-verification-results
// link
fn is_pact_broker_source(links: &Vec<Link>) -> bool {
  links.iter().any(|link| link.name == "pb:publish-verification-results")
}

#[instrument(level = "trace")]
async fn fetch_pacts(
  source: Vec<PactSource>,
  consumers: Vec<String>,
  provider: &ProviderInfo
) -> Vec<anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Option<PactVerificationContext>, PactSource, Duration)>> {
  trace!("fetch_pacts(source={}, consumers={:?})", source.iter().map(|s| s.to_string()).join(", "), consumers);

  futures::stream::iter(source)
    .then(|pact_source| async {
      futures::stream::iter(fetch_pact(pact_source, provider).await)
    })
    .flatten()
    .filter(|res| futures::future::ready(filter_consumers(&consumers, res)))
    .collect()
    .await
}

/// Internal function, public for testing purposes
#[tracing::instrument(level = "trace", skip(pact))]
pub async fn verify_pact_internal<'a, F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider_info: &ProviderInfo,
  filter: &FilterInfo,
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe + 'a>,
  options: &VerificationOptions<F>,
  provider_state_executor: &Arc<S>,
  pending: bool,
  pact_source_duration: Duration
) -> anyhow::Result<VerificationResult> {
  let mut output = vec![];
  let mut results = vec![];
  for interaction in pact.interactions() {
    if filter_interaction(interaction.as_ref(), filter) {
      let interaction_desc = interaction.description();
      let result = verify_interaction(provider_info, interaction.as_ref(), &pact.boxed(), options, provider_state_executor)
        .instrument(debug_span!("verify_interaction", interaction = interaction_desc.as_str())).await;

      let is_err = result.is_err();
      results.push((interaction, result));
      if options.exit_on_first_failure && is_err {
        break;
      }
    }
  }

  let mut errors: Vec<VerificationInteractionResult> = vec![];
  for (interaction, match_result) in results {
    let mut description = format!("Verifying a pact between {} and {}",
      pact.consumer().name.clone(), pact.provider().name.clone());

    output.push(String::default());
    let duration = match match_result {
      Ok((_, _, d)) => Duration::from_millis(d.as_millis() as u64),
      Err((_, _, d)) => Duration::from_millis(d.as_millis() as u64)
    };
    let pact_source_duration = Duration::from_millis(pact_source_duration.as_millis() as u64);
    if interaction.pending() {
      output.push(format!("  {} ({} loading, {} verification) {}", interaction.description(),
        format_duration(pact_source_duration),
        format_duration(duration),
        if options.coloured_output { Yellow.paint("[PENDING]") } else { Style::new().paint("[PENDING]") }));
    } else {
      output.push(format!("  {} ({} loading, {} verification)", interaction.description(),
        format_duration(pact_source_duration),
        format_duration(duration)));
    };

    if let Some((first, elements)) = interaction.provider_states().split_first() {
      let s = format!(" Given {}", first.name);
      description.push_str(&s);
      output.push(format!("    {}", s));
      for state in elements {
        let s = format!(" And {}", state.name);
        description.push_str(&s);
        output.push(format!("    {}", s));
      }
    }
    description.push_str(" - ");
    description.push_str(&interaction.description());

    let (interaction_key, verification_from_plugin) = if interaction.is_v4() {
      if let Some(interaction) = interaction.as_v4() {
        process_comments(interaction.as_ref(), &mut output);

        #[cfg(feature = "plugins")]
        {
          let verification_from_plugin = interaction.transport()
            .and_then(|t| catalogue_manager::lookup_entry(&*format!("transport/{}", t)))
            .and_then(|entry| Some(entry.provider_type == CatalogueEntryProviderType::PLUGIN))
            .unwrap_or(false);

          (interaction.key(), verification_from_plugin)
        }

        #[cfg(not(feature = "plugins"))]
        {
          (interaction.key(), false)
        }
      } else {
        (None, false)
      }
    } else {
      (None, false)
    };

    let match_result = match match_result {
      Ok((id, out, _)) => {
        if !out.is_empty() {
          output.push(String::default());
          output.extend(out.iter().map(|o| format!("  {}", o)));
        }
        Ok(id)
      }
      Err((err, out, _)) => {
        if !out.is_empty() {
          output.push(String::default());
          output.extend(out.iter().map(|o| format!("  {}", o)));
        }
        Err(err)
      }
    };

    if !verification_from_plugin {
      // Plugins should provide verification output, so we can skip this bit if a plugin was used

      // TODO: Update this to use V4 models
      if let Some(interaction) = interaction.as_request_response() {
        process_request_response_result(&interaction, &match_result, &mut output, options.coloured_output);
      }
      if let Some(interaction) = interaction.as_message() {
        process_message_result(&interaction, &match_result, &mut output, options.coloured_output);
      }
      if let Some(interaction) = interaction.as_v4_sync_message() {
        process_sync_message_result(&interaction, &match_result, &mut output, options.coloured_output);
      }
    }

    match match_result {
      Ok(_) => {
        errors.push(VerificationInteractionResult {
          interaction_id: interaction.id(),
          interaction_key,
          description: description.clone(),
          interaction_description: interaction.description(),
          result: Ok(()),
          pending: pending || interaction.pending(),
          duration
        });
      },
      Err(err) => {
        errors.push(VerificationInteractionResult {
          interaction_id: interaction.id(),
          interaction_key,
          description: description.clone(),
          interaction_description: interaction.description(),
          result: Err(err.clone()),
          pending: pending || interaction.pending(),
          duration
        });
      }
    }
  }

  output.push(String::default());

  Ok(VerificationResult { results: errors, output: output.clone() })
}

fn process_comments(interaction: &dyn V4Interaction, output: &mut Vec<String>) {
  let comments = interaction.comments();
  if !comments.is_empty() {
    if let Some(testname) = comments.get("testname") {
      let s = json_to_string(testname);
      if !s.is_empty() {
        output.push(format!("\n  Test Name: {}", s));
      }
    }
    if let Some(comment_text) = comments.get("text") {
      match comment_text {
        Value::Array(comment_text) => if !comment_text.is_empty() {
          output.push("\n  Comments:".to_string());
          for value in comment_text {
            output.push(json_to_string(value));
          }
          output.push(String::default());
        }
        Value::String(comment) => if !comment.is_empty() {
          output.push("\n  Comments:".to_string());
          output.push(comment.clone());
          output.push(String::default());
        }
        _ => {}
      }
    }
  }
}

async fn publish_result(
  results: &[VerificationInteractionResult],
  source: &PactSource,
  options: &PublishOptions,
  metrics_data: Option<&VerificationMetrics>
) {
  let publish_result = match source {
    PactSource::BrokerUrl(_, broker_url, auth, links) => {
      publish_to_broker(results, source, &options.build_url, &options.provider_tags,
        &options.provider_branch, &options.provider_version, links.clone(), broker_url.clone(),
        auth.clone(), metrics_data
      ).await
    }
    PactSource::BrokerWithDynamicConfiguration { broker_url, auth, links, provider_branch, provider_tags, .. } => {
      publish_to_broker(results, source, &options.build_url, &provider_tags, &provider_branch,
        &options.provider_version, links.clone(), broker_url.clone(), auth.clone(), metrics_data
      ).await
    }
    _ => {
      info!("Not publishing results as publishing for pact source {:?} is not possible or not yet implemented", source);
      return;
    }
  };
  match &publish_result {
    Ok(_) => info!("Results published to Pact Broker"),
    Err(err) => error!("Publishing of verification results failed with an error: {}", err)
  };
}

async fn publish_to_broker(
  results: &[VerificationInteractionResult],
  source: &PactSource,
  build_url: &Option<String>,
  provider_tags: &Vec<String>,
  provider_branch: &Option<String>,
  provider_version: &Option<String>,
  links: Vec<Link>,
  broker_url: String,
  auth: Option<HttpAuth>,
  metrics_data: Option<&VerificationMetrics>
) -> Result<Value, pact_broker::PactBrokerError> {
  info!("Publishing verification results back to the Pact Broker");
  let result = if results.iter().all(|r| r.result.is_ok()) {
    debug!("Publishing a successful result to {}", source);
    TestResult::Ok(results.iter().map(|r| r.interaction_id.clone()).collect())
  } else {
    debug!("Publishing a failure result to {}", source);
    TestResult::Failed(
      results.iter()
        .map(|r| (r.interaction_id.clone(), Some(r.interaction_description.clone()), r.result.as_ref().err().cloned()))
        .collect()
    )
  };
  publish_verification_results(
    links,
    broker_url.as_str(),
    auth.clone(),
    result,
    provider_version.clone().unwrap(),
    build_url.clone(),
    provider_tags.clone(),
    provider_branch.clone(),
    metrics_data
  ).await
}

#[cfg(test)]
mod tests;
