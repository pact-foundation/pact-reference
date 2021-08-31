//! The `pact_verifier` crate provides the core logic to performing verification of providers.
//! It implements the V3 (https://github.com/pact-foundation/pact-specification/tree/version-3)
//! and V4 Pact specification (https://github.com/pact-foundation/pact-specification/tree/version-4).
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ansi_term::*;
use ansi_term::Colour::*;
use futures::prelude::*;
use futures::stream::StreamExt;
use itertools::Itertools;
use log::*;
use maplit::*;
use regex::Regex;
use serde_json::Value;

pub use callback_executors::NullRequestFilterExecutor;
use callback_executors::RequestFilterExecutor;
use pact_matching::*;
use pact_models::generators::GeneratorTestMode;
use pact_models::http_utils::HttpAuth;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::pact::{load_pact_from_url, Pact, read_pact};
use pact_models::provider_states::*;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::v4::interaction::V4Interaction;

use crate::callback_executors::{ProviderStateError, ProviderStateExecutor};
use crate::messages::{display_message_result, verify_message_from_provider};
use crate::pact_broker::{Link, PactVerificationContext, publish_verification_results, TestResult};
pub use crate::pact_broker::{ConsumerVersionSelector, PactsForVerificationRequest};
use crate::provider_client::{make_provider_request, provider_client_error_to_string};
use crate::request_response::display_request_response_result;

mod provider_client;
pub mod pact_broker;
pub mod callback_executors;
mod request_response;
mod messages;

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
      ///Base URL of the Pact Broker from which to retrieve the pacts
      broker_url: String,
      /// Allow pacts which are in pending state to be verified without causing the overall task to fail. For more information, see https://pact.io/pending
      enable_pending: bool,
      /// Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip
      include_wip_pacts_since: Option<String>,
      /// Provider tags to use in determining pending status for return pacts
      provider_tags: Vec<String>,
      /// The set of selectors that identifies which pacts to verify
      selectors: Vec<ConsumerVersionSelector>,
      /// HTTP authentication details for accessing the Pact Broker
      auth: Option<HttpAuth>,
      /// Links to the specific Pact resources. Internal field
      links: Vec<Link>
    }
}

impl Display for PactSource {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match *self {
      PactSource::File(ref file) => write!(f, "File({})", file),
      PactSource::Dir(ref dir) => write!(f, "Dir({})", dir),
      PactSource::URL(ref url, _) => write!(f, "URL({})", url),
      PactSource::BrokerUrl(ref provider_name, ref broker_url, _, _) => {
          write!(f, "PactBroker({}, provider_name='{}')", broker_url, provider_name)
      }
      PactSource::BrokerWithDynamicConfiguration { ref provider_name, ref broker_url,ref enable_pending, ref include_wip_pacts_since, ref provider_tags, ref selectors, ref auth, links: _ } => {
        if let Some(auth) = auth {
          write!(f, "PactBrokerWithDynamicConfiguration({}, provider_name='{}', enable_ending={}, include_wip_since={:?}, provider_tagcs={:?}, consumer_version_selectors='{:?}, auth={}')", broker_url, provider_name, enable_pending, include_wip_pacts_since, provider_tags, selectors, auth)
        } else {
          write!(f, "PactBrokerWithDynamicConfiguration({}, provider_name='{}', enable_ending={}, include_wip_since={:?}, provider_tagcs={:?}, consumer_version_selectors='{:?}, auth=None')", broker_url, provider_name, enable_pending, include_wip_pacts_since, provider_tags, selectors)

        }
      }
      _ => write!(f, "Unknown")
    }
  }
}

/// Information about the Provider to verify
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider Name
    pub name: String,
    /// Provider protocol, defaults to HTTP
    pub protocol: String,
    /// Hostname of the provider
    pub host: String,
    /// Port the provider is running on, defaults to 8080
    pub port: Option<u16>,
    /// Base path for the provider, defaults to /
    pub path: String
}

impl Default for ProviderInfo {
    /// Create a default provider info
  fn default() -> ProviderInfo {
        ProviderInfo {
            name: s!("provider"),
            protocol: s!("http"),
            host: s!("localhost"),
            port: Some(8080),
            path: s!("/")
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
      expected: Box<dyn Interaction>,
      /// Actual Response/Message
      actual: Box<dyn Interaction>,
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
        if let Some(ref expected_reqres) = expected.as_request_response() {
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
  interaction: &RequestResponseInteraction,
  options: &VerificationOptions<F>,
  client: &reqwest::Client,
  verification_context: &HashMap<&str, Value>
) -> Result<Option<String>, MismatchResult> {
  let expected_response = &interaction.response;
  match make_provider_request(provider, &pact_matching::generate_request(&interaction.request, &GeneratorTestMode::Provider, &verification_context), options, client).await {
    Ok(ref actual_response) => {
      let mismatches = match_response(expected_response.clone(), actual_response.clone());
      if mismatches.is_empty() {
        Ok(interaction.id.clone())
      } else {
        Err(MismatchResult::Mismatches {
          mismatches,
          expected: Box::new(interaction.clone()),
          actual: Box::new(RequestResponseInteraction { response: actual_response.clone(), .. RequestResponseInteraction::default() }),
          interaction_id: interaction.id.clone()
        })
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(provider_client_error_to_string(err), interaction.id.clone()))
    }
  }
}

async fn execute_state_change<S: ProviderStateExecutor>(
  provider_state: &ProviderState,
  setup: bool,
  interaction_id: Option<String>,
  client: &reqwest::Client,
  provider_state_executor: Arc<S>
) -> Result<HashMap<String, Value>, MismatchResult> {
    if setup {
        println!("  Given {}", Style::new().bold().paint(provider_state.name.clone()));
    }
    let result = provider_state_executor.call(interaction_id, provider_state, setup, Some(client)).await;
    log::trace!("State Change: \"{:?}\" -> {:?}", provider_state, result);
    result.map_err(|err| MismatchResult::Error(err.description, err.interaction_id))
}

async fn verify_interaction<F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider: &ProviderInfo,
  interaction: &dyn Interaction,
  options: &VerificationOptions<F>,
  provider_state_executor: &Arc<S>
) -> Result<Option<String>, MismatchResult> {
  let client = Arc::new(reqwest::Client::builder()
  .danger_accept_invalid_certs(options.disable_ssl_verification)
  .timeout(Duration::from_millis(options.request_timeout))
  .build()
  .unwrap_or(reqwest::Client::new()));

  let mut provider_states_results = hashmap!{};
  let sc_results = futures::stream::iter(
    interaction.provider_states().iter().map(|state| (state, client.clone())))
    .then(|(state, client)| {
      let state_name = state.name.clone();
      log::debug!("Running provider state change handler '{}' for '{}'", state_name, interaction.description());
      async move {
        execute_state_change(&state, true, interaction.id(), &client,
                             provider_state_executor.clone())
          .map_err(|err| {
            error!("Provider state change for '{}' has failed - {:?}", state_name, err);
            err
          }).await
      }
    }).collect::<Vec<Result<HashMap<String, Value>, MismatchResult>>>().await;
  if sc_results.iter().any(|result| result.is_err()) {
    return Err(MismatchResult::Error("One or more of the state change handlers has failed".to_string(), interaction.id()))
  } else {
    for result in sc_results {
      if result.is_ok() {
        for (k, v) in result.unwrap() {
          provider_states_results.insert(k, v);
        }
      }
    }
  };

  info!("Running provider verification for '{}'", interaction.description());
  let result = futures::future::ready((provider_states_results.iter()
    .map(|(k, v)| (k.as_str(), v.clone())).collect(), client.clone()))
    .then(|(context, client)| async move {
    let mut result = Err(MismatchResult::Error("No interaction was verified".into(), interaction.id().clone()));
    if let Some(interaction) = interaction.as_request_response() {
      result = verify_response_from_provider(provider, &interaction, options, &client, &context).await;
    }
    if interaction.is_message() {
      result = verify_message_from_provider(provider, &interaction.boxed(), options, &client, &context).await;
    }
    result
  }).await;

  if !interaction.provider_states().is_empty() {
    let sc_teardown_result = futures::stream::iter(
      interaction.provider_states().iter().map(|state| (state, client.clone())))
      .then(|(state, client)| async move {
        let state_name = state.name.clone();
        debug!("Running provider state change handler '{}' for '{}'", state_name, interaction.description());
        execute_state_change(&state, false, interaction.id(), &client,
                             provider_state_executor.clone())
          .map_err(|err| {
            error!("Provider state change teardown for '{}' has failed - {:?}", state.name, err);
            err
          }).await
      }).collect::<Vec<Result<HashMap<String, Value>, MismatchResult>>>().await;

    if sc_teardown_result.iter().any(|result| result.is_err()) {
      return Err(MismatchResult::Error("One or more of the state change handlers has failed during teardown phase".to_string(), interaction.id()))
    }
  }

  result
}

fn display_result(
  status: u16,
  status_result: ANSIGenericString<str>,
  header_results: Option<Vec<(String, String, ANSIGenericString<str>)>>,
  body_result: ANSIGenericString<str>
) {
  println!("    returns a response which");
  println!("      has status code {} ({})", Style::new().bold().paint(format!("{}", status)),
      status_result);
  if let Some(header_results) = header_results {
    println!("      includes headers");
    for (key, value, result) in header_results {
      println!("        \"{}\" with value \"{}\" ({})", Style::new().bold().paint(key),
               Style::new().bold().paint(value), result);
    }
  }
  println!("      has a matching body ({})", body_result);
}

fn walkdir(dir: &Path) -> anyhow::Result<Vec<anyhow::Result<Box<dyn Pact>>>> {
    let mut pacts = vec![];
    log::trace!("Scanning {:?}", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walkdir(&path)?;
        } else {
            pacts.push(read_pact(&path))
        }
    }
    Ok(pacts)
}

fn display_body_mismatch(expected: &Box<dyn Interaction>, actual: &Box<dyn Interaction>, path: &str) {
  if expected.contents_for_verification().content_type().unwrap_or_default().is_json() {
    println!("{}", pact_matching::json::display_diff(
      &expected.contents_for_verification().str_value().to_string(),
      &actual.contents_for_verification().str_value().to_string(),
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
            _ => s!("")
        }
    }

    /// Value of the description to filter
    pub fn description(&self) -> String {
        match *self {
            FilterInfo::Description(ref s) => s.clone(),
            FilterInfo::DescriptionAndState(ref s, _) => s.clone(),
            _ => s!("")
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

fn filter_consumers(consumers: &[String], res: &Result<(Box<dyn Pact>, Option<PactVerificationContext>, PactSource), String>) -> bool {
  consumers.is_empty() || res.is_err() || consumers.contains(&res.as_ref().unwrap().0.consumer().name)
}

/// Options to use when running the verification
#[derive(Debug, Clone)]
pub struct VerificationOptions<F> where F: RequestFilterExecutor {
  /// If results should be published back to the broker
  pub publish: bool,
  /// Provider version being published
  pub provider_version: Option<String>,
  /// Build URL to associate with the published results
  pub build_url: Option<String>,
  /// Request filter callback
  pub request_filter: Option<Arc<F>>,
  /// Tags to use when publishing results
  pub provider_tags: Vec<String>,
  /// Ignore invalid/self-signed SSL certificates
  pub disable_ssl_verification: bool,
  /// Timeout in ms for verification requests and state callbacks
  pub request_timeout: u64
}

impl <F: RequestFilterExecutor> Default for VerificationOptions<F> {
  fn default() -> Self {
    VerificationOptions {
      publish: false,
      provider_version: None,
      build_url: None,
      request_filter: None,
      provider_tags: vec![],
      disable_ssl_verification: false,
      request_timeout: 5000
    }
  }
}

const VERIFICATION_NOTICE_BEFORE: &str = "before_verification";
const VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_PUBLISH: &str = "after_verification:success_true_published_true";
const VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_NO_PUBLISH: &str = "after_verification:success_true_published_false";
const VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_PUBLISH: &str = "after_verification:success_false_published_true";
const VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_NO_PUBLISH: &str = "after_verification:success_false_published_false";

fn display_notices(context: &Option<PactVerificationContext>, stage: &str) {
  if let Some(c) = context {
    for notice in &c.verification_properties.notices {
      if let Some(when) = notice.get("when") {
        if when.as_str() == stage {
          println!("{}", notice.get("text").unwrap_or(&"".to_string()));
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
  options: VerificationOptions<F>,
  provider_state_executor: &Arc<S>
) -> bool {
  match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
    Ok(runtime) => runtime.block_on(
      verify_provider_async(provider_info, source, filter, consumers, options, provider_state_executor)),
    Err(err) => {
      error!("Verify provider process failed to start the tokio runtime: {}", err);
      false
    }
  }
}

/// Verify the provider with the given pact sources (async version)
pub async fn verify_provider_async<F: RequestFilterExecutor, S: ProviderStateExecutor>(
    provider_info: ProviderInfo,
    source: Vec<PactSource>,
    filter: FilterInfo,
    consumers: Vec<String>,
    options: VerificationOptions<F>,
    provider_state_executor: &Arc<S>
) -> bool {
    let pact_results = fetch_pacts(source, consumers).await;

    let mut results: Vec<(Option<String>, Result<(), MismatchResult>)> = vec![];
    let mut pending_errors: Vec<(String, MismatchResult)> = vec![];
    let mut errors: Vec<(String, MismatchResult)> = vec![];
    for pact_result in pact_results {
      match pact_result {
        Ok((pact, context, pact_source)) => {
          display_notices(&context, VERIFICATION_NOTICE_BEFORE);
          println!("\nVerifying a pact between {} and {}",
          Style::new().bold().paint(pact.consumer().name.clone()),
          Style::new().bold().paint(pact.provider().name.clone()));

          if pact.interactions().is_empty() {
            println!("         {}", Yellow.paint("WARNING: Pact file has no interactions"));
          } else {
            let pending = match &context {
              Some(context) => context.verification_properties.pending,
              None => false
            };
            let result = verify_pact_internal(&provider_info, &filter, pact, &options,
                                              &provider_state_executor.clone(), pending).await;
            for result in &result.results {
              results.push((result.interaction_id.clone(), result.result.clone()));
              if let Err(error) = &result.result {
                if result.pending {
                  pending_errors.push((result.description.clone(), error.clone()));
                } else {
                  errors.push((result.description.clone(), error.clone()));
                }
              }
            }

            if options.publish {
              publish_result(&results, &pact_source, &options).await;

              if !errors.is_empty() || !pending_errors.is_empty() {
                display_notices(&context, VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_PUBLISH);
              } else {
                display_notices(&context, VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_PUBLISH);
              }
            } else {
              if !errors.is_empty() || pending_errors.is_empty() {
                display_notices(&context, VERIFICATION_NOTICE_AFTER_ERROR_RESULT_AND_NO_PUBLISH);
              } else {
                display_notices(&context, VERIFICATION_NOTICE_AFTER_SUCCESSFUL_RESULT_AND_NO_PUBLISH);
              }
            }
          }
        },
        Err(err) => {
          error!("Failed to load pact - {}", Red.paint(err.to_string()));
          errors.push(("Failed to load pact".to_string(), MismatchResult::Error(err.to_string(), None)));
        }
      }
    };

    if !pending_errors.is_empty() {
      println!("\nPending Failures:\n");
      print_errors(&pending_errors);
      println!("\nThere were {} non-fatal pact failures on pending pacts or interactions (see docs.pact.io/pending for more information)\n", pending_errors.len());
    }
    if !errors.is_empty() {
      println!("\nFailures:\n");
      print_errors(&errors);
      println!("\nThere were {} pact failures\n", errors.len());
      false
    } else {
      println!();
      true
    }
}

fn print_errors(errors: &Vec<(String, MismatchResult)>) {
  for (i, &(ref description, ref mismatch)) in errors.iter().enumerate() {
    match *mismatch {
        MismatchResult::Error(ref err, _) => println!("{}) {} - {}\n", i + 1, description, err),
        MismatchResult::Mismatches { ref mismatches, ref expected, ref actual, .. } => {
          println!("{}) {}", i + 1, description);

          let mut j = 1;
          for (_, mut mismatches) in &mismatches.into_iter().group_by(|m| m.mismatch_type()) {
            let mismatch = mismatches.next().unwrap();
            println!("    {}.{}) {}", i + 1, j, mismatch.summary());
            println!("           {}", mismatch.ansi_description());
            for mismatch in mismatches {
              println!("           {}", mismatch.ansi_description());
            }

            if let Mismatch::BodyMismatch{ref path, ..} = mismatch {
              display_body_mismatch(expected, actual, path);
            }

            j += 1;
          }
        }
    }
  }
}

async fn fetch_pact(source: PactSource) -> Vec<Result<(Box<dyn Pact>, Option<PactVerificationContext>, PactSource), String>> {
  trace!("fetch_pact(source={})", source);

  match source {
    PactSource::File(ref file) => vec![read_pact(Path::new(&file))
      .map_err(|err| format!("Failed to load pact '{}' - {}", file, err))
      .map(|pact| (pact, None, source))],
    PactSource::Dir(ref dir) => match walkdir(Path::new(dir)) {
      Ok(pact_results) => pact_results.into_iter().map(|pact_result| {
          match pact_result {
              Ok(pact) => Ok((pact, None, source.clone())),
              Err(err) => Err(format!("Failed to load pact from '{}' - {}", dir, err))
          }
      }).collect(),
      Err(err) => vec![Err(format!("Could not load pacts from directory '{}' - {}", dir, err))]
    },
    PactSource::URL(ref url, ref auth) => vec![load_pact_from_url(url, auth)
      .map_err(|err| format!("Failed to load pact '{}' - {}", url, err))
      .map(|pact| (pact, None, source))],
    PactSource::BrokerUrl(ref provider_name, ref broker_url, ref auth, _) => {
      let result = pact_broker::fetch_pacts_from_broker(
        broker_url.as_str(),
        provider_name.as_str(),
        auth.clone()
      ).await;

      match result {
        Ok(ref pacts) => {
          let mut buffer = vec![];
          for result in pacts.iter() {
            match result {
              Ok((pact, _, links)) => {
                log::trace!("Got pact with links {:?}", links);
                if let Ok(pact) = pact.as_request_response_pact() {
                  buffer.push(Ok((Box::new(pact) as Box<dyn Pact>, None, PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()))))
                }
                if let Ok(pact) = pact.as_message_pact() {
                  buffer.push(Ok((Box::new(pact) as Box<dyn Pact>, None, PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()))))
                }
              },
              &Err(ref err) => buffer.push(Err(format!("Failed to load pact from '{}' - {:?}", broker_url, err)))
            }
          }
          buffer
        },
        Err(err) => vec![Err(format!("Could not load pacts from the pact broker '{}' - {:?}", broker_url, err))]
      }
    },
    PactSource::BrokerWithDynamicConfiguration { provider_name, broker_url, enable_pending, include_wip_pacts_since, provider_tags, selectors, auth, links: _ } => {
      let result = pact_broker::fetch_pacts_dynamically_from_broker(
        broker_url.as_str(),
        provider_name.clone(),
        enable_pending,
        include_wip_pacts_since,
        provider_tags,
        selectors,
        auth.clone()
      ).await;

      match result {
        Ok(ref pacts) => {
          let mut buffer = vec![];
          for result in pacts.iter() {
            match result {
              Ok((pact, context, links)) => {
                log::trace!("Got pact with links {:?}", links);
                if let Ok(pact) = pact.as_request_response_pact() {

                  buffer.push(Ok((Box::new(pact) as Box<dyn Pact>, context.clone(), PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()))))
                }
                if let Ok(pact) = pact.as_message_pact() {
                  buffer.push(Ok((Box::new(pact) as Box<dyn Pact>, context.clone(), PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone()))))
                }
              },
              &Err(ref err) => buffer.push(Err(format!("Failed to load pact from '{}' - {:?}", broker_url, err)))
            }
          }
          buffer
        },
        Err(err) => vec![Err(format!("Could not load pacts from the pact broker '{}' - {:?}", broker_url, err))]
      }
    },
    _ => vec![Err("Could not load pacts, unknown pact source".to_string())]
  }
}

async fn fetch_pacts(source: Vec<PactSource>, consumers: Vec<String>)
  -> Vec<Result<(Box<dyn Pact>, Option<PactVerificationContext>, PactSource), String>> {
  trace!("fetch_pacts(source={:?}, consumers={:?})", source, consumers);

  futures::stream::iter(source)
    .then(|pact_source| async {
      futures::stream::iter(fetch_pact(pact_source).await)
    })
    .flatten()
    .filter(|res| futures::future::ready(filter_consumers(&consumers, res)))
    .collect()
    .await
}

/// /// Result of verifying a Pact interaction
pub struct VerificationInteractionResult {
  /// Interaction ID
  pub interaction_id: Option<String>,
  /// Description
  pub description: String,
  /// Result of the verification
  pub result: Result<(), MismatchResult>,
  /// If the Pact or interaction is pending
  pub pending: bool
}

/// Result of verifying a Pact
pub struct VerificationResult {
  /// Results that occurred
  pub results: Vec<VerificationInteractionResult>
}

/// Internal function, public for testing purposes
pub async fn verify_pact_internal<'a, F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider_info: &ProviderInfo,
  filter: &FilterInfo,
  pact: Box<dyn Pact + 'a>,
  options: &VerificationOptions<F>,
  provider_state_executor: &Arc<S>,
  pending: bool
) -> VerificationResult {
  let results: Vec<(&dyn Interaction, Result<Option<String>, MismatchResult>)> = futures::stream::iter(
    pact.interactions().iter().cloned()
  )
    .filter(|interaction| futures::future::ready(filter_interaction(*interaction, filter)))
    .then( |interaction| async move {
      verify_interaction(provider_info, interaction, options, provider_state_executor)
        .then(|result| futures::future::ready((interaction, result)))
        .await
    })
    .collect()
    .await;

  let mut errors: Vec<VerificationInteractionResult> = vec![];
  for (interaction, match_result) in results {
    let mut description = format!("Verifying a pact between {} and {}",
      pact.consumer().name.clone(), pact.provider().name.clone());
    if let Some((first, elements)) = interaction.provider_states().split_first() {
      description.push_str(&format!(" Given {}", first.name));
      for state in elements {
        description.push_str(&format!(" And {}", state.name));
      }
    }
    description.push_str(" - ");
    description.push_str(&interaction.description());
    if interaction.pending() {
      println!("  {} {}", interaction.description(), Yellow.paint("[PENDING]"));
    } else {
      println!("  {}", interaction.description());
    };

    if interaction.is_v4() {
      if let Some(interaction) = interaction.as_v4() {
        display_comments(interaction)
      }
    }

    if let Some(interaction) = interaction.as_request_response() {
      display_request_response_result(&interaction, &match_result)
    }
    if let Some(interaction) = interaction.as_message() {
      display_message_result(&interaction, &match_result)
    }

    match match_result {
      Ok(_) => {
        errors.push(VerificationInteractionResult {
          interaction_id: interaction.id(),
          description: description.clone(),
          result: Ok(()),
          pending: pending || interaction.pending()
        });
      },
      Err(err) => {
        errors.push(VerificationInteractionResult {
          interaction_id: interaction.id(),
          description: description.clone(),
          result: Err(err.clone()),
          pending: pending || interaction.pending()
        });
      }
    }
  }

  println!();

  VerificationResult { results: errors }
}

fn display_comments(interaction: Box<dyn V4Interaction>) {
  let comments = interaction.comments();
  if !comments.is_empty() {
    if let Some(testname) = comments.get("testname") {
      println!("\n  Test Name: {}", json_to_string(testname));
    }
    if let Some(comment_text) = comments.get("text") {
      println!("\n  Comments:");
      match comment_text {
        Value::Array(comment_text) => for value in comment_text {
          println!("    {}", json_to_string(value));
        }
        _ => println!("    {}", comment_text)
      }
    }
    println!();
  }
}

async fn publish_result<F: RequestFilterExecutor>(
  results: &[(Option<String>, Result<(), MismatchResult>)],
  source: &PactSource,
  options: &VerificationOptions<F>
) {
  if let PactSource::BrokerUrl(_, broker_url, auth, links) = source.clone() {
    info!("Publishing verification results back to the Pact Broker");
    let result = if results.iter().all(|(_, result)| result.is_ok()) {
      debug!("Publishing a successful result to {}", source);
      TestResult::Ok(results.iter().map(|(id, _)| id.clone()).collect())
    } else {
      debug!("Publishing a failure result to {}", source);
      TestResult::Failed(
        results.iter()
        .map(|(id, result)| (id.clone(), result.as_ref().err().cloned()))
        .collect()
      )
    };
    let provider_version = options.provider_version.clone().unwrap();
    let publish_result = publish_verification_results(
      links,
      broker_url.as_str(),
      auth.clone(),
      result,
      provider_version,
      options.build_url.clone(),
      options.provider_tags.clone()
    ).await;

    match &publish_result {
      Ok(_) => info!("Results published to Pact Broker"),
      Err(err) => error!("Publishing of verification results failed with an error: {}", err)
    };
  }
}

#[cfg(test)]
mod tests;
