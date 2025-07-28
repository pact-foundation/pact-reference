//! Handle interface to creating a verifier

use std::sync::Arc;

use itertools::Itertools;
use pact_mock_server::LOG_ID;
use serde_json::Value;
use tracing::{debug, error};
use pact_models::prelude::HttpAuth;
use pact_verifier::{ConsumerVersionSelector, FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, ProviderTransport, PublishOptions, VerificationOptions, verify_provider_async};
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;
use pact_verifier::metrics::VerificationMetrics;
use pact_verifier::verification_result::VerificationExecutionResult;

use crate::RUNTIME;

#[derive(Debug, Clone)]
/// Wraps a Pact verifier
pub struct VerifierHandle {
  provider: ProviderInfo,
  sources: Vec<PactSource>,
  filter: FilterInfo,
  state_change: Arc<HttpRequestProviderStateExecutor>,
  verification_options: VerificationOptions<NullRequestFilterExecutor>,
  publish_options: Option<PublishOptions>,
  consumers: Vec<String>,
  /// Calling application name and version
  calling_app: Option<(String, String)>,
  /// Output captured from the verifier
  verifier_output: VerificationExecutionResult
}

impl VerifierHandle {
  /// Create a new verifier and return the handle to it (deprecated, use new_for_application)
  #[deprecated(since = "0.1.4", note = "Use new_for_application instead")]
  pub fn new() -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new(),
      filter: FilterInfo::None,
      state_change: Arc::new(HttpRequestProviderStateExecutor::default()),
      verification_options: VerificationOptions::default(),
      publish_options: None,
      consumers: vec![],
      calling_app: None,
      verifier_output: VerificationExecutionResult::new()
    }
  }

  /// Create a new verifier and return the handle to it
  pub fn new_for_application(calling_app_name: &str, calling_app_version: &str) -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new(),
      filter: FilterInfo::None,
      state_change: Arc::new(HttpRequestProviderStateExecutor::default()),
      verification_options: VerificationOptions::default(),
      publish_options: None,
      consumers: vec![],
      calling_app: Some((calling_app_name.to_string(), calling_app_version.to_string())),
      verifier_output: VerificationExecutionResult::new()
    }
  }

  /// Retrieve the provider info from the handle
  pub fn provider_info(&self) -> ProviderInfo {
    self.provider.clone()
  }

  /// Add a new transport to the verification process
  pub fn add_transport(
    &mut self,
    protocol: String,
    port: u16,
    path: String,
    scheme: Option<String>
  ) {

    let transport = ProviderTransport {
      transport: protocol,
      port: Some(port),
      path: if path.is_empty() { None } else { Some(path) },
      scheme: scheme
    };

    self.provider.transports.push(transport);
  }

  /// Update the provider info
  #[allow(deprecated)]
  pub fn update_provider_info(
    &mut self,
    name: String,
    scheme: String,
    host: String,
    port: u16,
    path: String
  ) {
    let port = if port == 0 { None } else { Some(port) };
    self.provider = ProviderInfo {
      name,
      protocol: scheme.clone(),
      host,
      port: port.clone(),
      path: path.clone(),
      transports: vec![ ProviderTransport {
        transport: scheme.clone(),
        port,
        path: if path.is_empty() { None } else { Some(path) },
        scheme: if scheme.is_empty() { None } else { Some(scheme.clone()) }
      } ]
    }
  }

  /// Update the filter info
  pub fn update_filter_info(
    &mut self,
    filter_description: String,
    filter_state: String,
    filter_no_state: bool
  ) {
    self.filter = if !filter_description.is_empty() && (!filter_state.is_empty() || filter_no_state) {
        if !filter_state.is_empty() {
            FilterInfo::DescriptionAndState(filter_description, filter_state)
        } else {
            FilterInfo::DescriptionAndState(filter_description, String::new())
        }
    } else if !filter_description.is_empty() {
        FilterInfo::Description(filter_description)
    } else if !filter_state.is_empty() {
        FilterInfo::State(filter_state)
    } else if filter_no_state {
        FilterInfo::State(String::new())
    } else {
        FilterInfo::None
    }
  }

  /// Add a file source to be verified
  pub fn add_file_source(&mut self, file: &str) {
    self.sources.push(PactSource::File(file.to_string()));
  }

  /// Add a directory source to be verified. This will verify all pact files in the directory.
  pub fn add_directory_source(&mut self, dir: &str) {
    self.sources.push(PactSource::Dir(dir.to_string()));
  }

  /// Add a URL source to be verified. This will fetch the pact file from the URL. If a username
  /// and password is given, then basic authentication will be used when fetching the pact file.
  /// If a token is provided, then bearer token authentication will be used.
  pub fn add_url_source(&mut self, url: &str, auth: &HttpAuth) {
    if !auth.is_none() {
      self.sources.push(PactSource::URL(url.to_string(), Some(auth.clone())));
    } else {
      self.sources.push(PactSource::URL(url.to_string(), None));
    }
  }

  /// Add a Pact broker source to be verified. This will fetch all the pact files from the broker
  /// that match the provider name. If a username
  /// and password is given, the basic authentication will be used when fetching the pact file.
  /// If a token is provided, then bearer token authentication will be used.
  #[allow(clippy::too_many_arguments)]
  pub fn add_pact_broker_source(
    &mut self,
    url: &str,
    enable_pending: bool,
    include_wip_pacts_since: Option<String>,
    provider_tags: Vec<String>,
    provider_branch: Option<String>,
    selectors: Vec<ConsumerVersionSelector>,
    auth: &HttpAuth
  ) {
    if !auth.is_none() {
      self.sources.push(PactSource::BrokerWithDynamicConfiguration {
        provider_name: self.provider.name.clone(),
        broker_url: url.to_string(),
        enable_pending,
        include_wip_pacts_since,
        provider_tags,
        provider_branch,
        selectors,
        auth: Some(auth.clone()),
        links: vec![]
      });
    } else {
      self.sources.push(PactSource::BrokerWithDynamicConfiguration {
        provider_name: self.provider.name.clone(),
        broker_url: url.to_string(),
        enable_pending,
        include_wip_pacts_since,
        provider_tags,
        provider_branch,
        selectors,
        auth: None,
        links: vec![]
      });
    }
  }

  /// Update the provider state
  pub fn update_provider_state(
    &mut self,
    state_change_url: Option<String>,
    state_change_teardown: bool,
    state_change_body: bool
  ) {
    self.state_change = Arc::new(HttpRequestProviderStateExecutor {
      state_change_url,
      state_change_teardown,
      state_change_body,
      .. HttpRequestProviderStateExecutor::default()
    })
  }

  /// Update options used when running a verification
  /// 
  /// # Args
  /// 
  /// - disable_ssl_verification - Disable SSL verification on all HTTPS requests
  /// - request_timeout - Timeout for all requests to the provider
  pub fn update_verification_options(
    &mut self,
    disable_ssl_verification: bool,
    request_timeout: u64
  ) {
    self.verification_options.disable_ssl_verification = disable_ssl_verification;
    self.verification_options.request_timeout = request_timeout;
  }

  /// Enables or disables use of ANSI escape codes with the verifier output
  pub fn set_use_coloured_output(
    &mut self,
    coloured_output: bool
  ) {
    self.verification_options.coloured_output = coloured_output;
  }

  /// Enables or disables erroring if no pacts are found to verify
  pub fn set_no_pacts_is_error(&mut self, is_error: bool) {
    self.verification_options.no_pacts_is_error = is_error;
  }

  /// Update the details used when publishing results
  /// 
  /// # Args
  /// 
  /// - `provider_version` - Version of the provider
  pub fn update_publish_options(
    &mut self,
    provider_version: &str,
    build_url: Option<String>,
    provider_tags: Vec<String>,
    provider_branch: Option<String>
  ) {
    self.publish_options = Some(PublishOptions {
      provider_version: Some(provider_version.to_string()),
      build_url,
      provider_tags,
      provider_branch,
    })
  }

  /// Update the consumer filter
  pub fn update_consumers(
    &mut self,
    consumers: Vec<String>
  ) {
    self.consumers = consumers
  }

  /// Execute the verifier
  ///
  /// This will return an integer value based on the status of the verification:
  /// * 0 - verification was successful
  /// * 1 - verification was not successful
  /// * 2 - failed to run the verification
  ///
  /// Anu captured output from the verification will be stored against this handle
  pub fn execute(&mut self) -> i32 {
    for s in &self.sources {
      debug!("Pact source to verify = {s}");
    };

    let (calling_app_name, calling_app_version) = self.calling_app.clone().unwrap_or_else(|| {
      ("pact_ffi".to_string(), env!("CARGO_PKG_VERSION").to_string())
    });
    match RUNTIME.block_on(LOG_ID.scope(format!("verify:{}", self.provider.name), async {
      verify_provider_async(
        self.provider.clone(),
        self.sources.clone(),
        self.filter.clone(),
        self.consumers.clone(),
        &self.verification_options,
        self.publish_options.as_ref(),
        &self.state_change.clone(),
        Some(VerificationMetrics {
          test_framework: "pact_ffi".to_string(),
          app_name: calling_app_name.clone(),
          app_version: calling_app_version.clone()
        })
      ).await
    })) {
      Ok(result) => {
        self.verifier_output = result.clone();
        if result.result { 0 } else { 1 }
      }
      Err(err) => {
        error!("Verification execution failed: {}", err);
        self.verifier_output.output.push(format!("Verification execution failed: {}", err));
        2
      }
    }
  }

  /// Return the captured standard output from the verification execution
  pub fn output(&self) -> String {
    self.verifier_output.output.iter().join("\n")
  }

  /// Return the verification results as a JSON document
  pub fn json(&self) -> String {
    let json: Value = (&self.verifier_output).into();
    json.to_string()
  }

  #[cfg(test)]
  pub fn set_output(&mut self, out: &str) {
    self.verifier_output.output = out.split('\n').map(|s| s.to_string()).collect();
  }

  /// Add a custom header to be included in the call to the provider
  pub fn add_custom_header(&mut self, header_name: &str, header_value: &str) {
    self.verification_options.custom_headers.insert(header_name.to_string(), header_value.to_string());
  }
}

impl Default for VerifierHandle {
   fn default() -> Self {
     #[allow(deprecated)]
     Self::new()
   }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use hamcrest2::prelude::*;
  use serde_json::Value;

  use pact_models::pact::Pact;
  use pact_models::PactSpecification;
  use pact_models::v4::interaction::V4Interaction;
  use pact_models::v4::pact::V4Pact;
  use pact_models::v4::synch_http::SynchronousHttp;
  use pact_verifier::PactSource;

  use crate::verifier::handle::VerifierHandle;

  #[test]
  fn update_provider_info_sets_scheme_correctly() {
    let mut handle = VerifierHandle::new_for_application("test", "0.0.0");
    handle.update_provider_info("Test".to_string(), "https".to_string(), "localhost".to_string(), 1234, "".to_string());

    let interaction = SynchronousHttp {
      transport: Some("https".to_string()),
      .. SynchronousHttp::default()
    };
    let pact = V4Pact {
      interactions: vec![ interaction.boxed_v4() ],
      .. V4Pact::default()
    };
    handle.sources.push(PactSource::String(pact.to_json(PactSpecification::V4).unwrap().to_string()));
    let status = handle.execute();

    expect!(status).to(be_equal_to(1));
    expect!(handle.verifier_output.result).to(be_false());
    let error: Value = handle.verifier_output.errors[0].clone().1.into();
    let message = error.as_object().unwrap()["message"].as_str().unwrap();
    assert_that!(message, matches_regex(r"^error sending request"));
  }
}
