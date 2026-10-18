//! Executor abstraction for executing callbacks to user code (request filters, provider state change callbacks)

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use bytes::Bytes;
use itertools::Either;
use maplit::*;
use serde_json::{json, Value};

static DEFAULT_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
  reqwest::Client::builder()
    .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
    .build()
    .expect("Failed to build default HTTP client")
});

use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::provider_states::ProviderState;
use pact_models::v4::http_parts::HttpRequest;
use tracing::warn;

use crate::provider_client::make_state_change_request;

/// Trait for executors that call request filters
pub trait RequestFilterExecutor: Debug {
  /// Mutates HTTP requests based on some criteria.
  fn call(self: Arc<Self>, request: &HttpRequest) -> HttpRequest;

  /// Callback to mutate request data. This form is used by plugins.
  fn call_non_http(
    &self,
    request_body: &OptionalBody,
    metadata: &HashMap<String, Either<Value, Bytes>>
  ) -> (OptionalBody, HashMap<String, Either<Value, Bytes>>);
}

/// A "null" request filter executor, which does nothing, but permits
/// bypassing of typechecking issues where no filter should be applied.
#[derive(Debug, Clone)]
pub struct NullRequestFilterExecutor {
  // This field is added (and is private) to guarantee that this struct
  // is never instantiated accidentally, and is instead only able to be
  // used for type-level programming.
  _private_field: (),
}

impl RequestFilterExecutor for NullRequestFilterExecutor {
  fn call(self: Arc<Self>, _request: &HttpRequest) -> HttpRequest {
    unimplemented!("NullRequestFilterExecutor should never be called")
  }

  fn call_non_http(
    &self,
    _body: &OptionalBody,
    _metadata: &HashMap<String, Either<Value, Bytes>>
  ) -> (OptionalBody, HashMap<String, Either<Value, Bytes>>) {
    unimplemented!("NullRequestFilterExecutor should never be called")
  }
}

/// Struct for returning errors from executing a provider state
#[derive(Debug, Clone)]
pub struct ProviderStateError {
  /// Description of the error
  pub description: String,
  /// Interaction ID of the interaction that the error occurred
  pub interaction_id: Option<String>
}

impl Display for ProviderStateError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Provider state failed: {}{}", self.interaction_id.as_ref()
      .map(|id| format!("(interaction_id: {}) ", id)).unwrap_or_default(), self.description)
  }
}

impl std::error::Error for ProviderStateError {}

/// Trait for executors that call provider state callbacks
#[async_trait]
pub trait ProviderStateExecutor: Debug {
  /// Invoke the callback for the given provider state, returning an optional Map of values
  async fn call(
    self: Arc<Self>,
    interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    client: Option<&reqwest::Client>
  ) -> anyhow::Result<HashMap<String, Value>>;

  /// If a teardown call for the Executor should be performed
  fn teardown(self: &Self)-> bool;
}

/// Default provider state callback executor, which executes an HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequestProviderStateExecutor {
  /// URL to post state change requests to
  pub state_change_url: Option<String>,
  /// If teardown state change requests should be made (default is false)
  pub state_change_teardown: bool,
  /// If state change request data should be sent in the body (true) or as query parameters (false)
  pub state_change_body: bool,
  /// Number of times to retry the provider state request, zero means none
  pub retries: u8
}

/// Environment variable that overrides how many times a failed provider state change request is
/// retried. Retries apply to 5xx, 408 and 429 responses, with an exponential back-off starting at
/// 500 ms. Zero disables them.
pub const STATE_CHANGE_RETRIES_ENV: &str = "PACT_PROVIDER_STATE_CHANGE_RETRIES";

/// How many times a failed provider state change request is retried when nothing overrides it.
pub const DEFAULT_STATE_CHANGE_RETRIES: u8 = 8;

/// The number of state change retries an override asks for, or the default when there is no
/// override or it is not a number from 0 to 255.
fn state_change_retries(value: Option<String>) -> u8 {
  match value {
    None => DEFAULT_STATE_CHANGE_RETRIES,
    Some(value) => value.trim().parse().unwrap_or_else(|_| {
      warn!("Ignoring {}='{}' as it is not a number from 0 to 255, using the default of {}",
        STATE_CHANGE_RETRIES_ENV, value, DEFAULT_STATE_CHANGE_RETRIES);
      DEFAULT_STATE_CHANGE_RETRIES
    })
  }
}

impl Default for HttpRequestProviderStateExecutor {
  /// Create a default executor. The number of retries is read from the
  /// `PACT_PROVIDER_STATE_CHANGE_RETRIES` environment variable, defaulting to 8.
  fn default() -> HttpRequestProviderStateExecutor {
    HttpRequestProviderStateExecutor {
      state_change_url: None,
      state_change_teardown: false,
      state_change_body: true,
      retries: state_change_retries(std::env::var(STATE_CHANGE_RETRIES_ENV).ok())
    }
  }
}

#[async_trait]
impl ProviderStateExecutor for HttpRequestProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    client: Option<&reqwest::Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    match &self.state_change_url {
      Some(state_change_url) => {
        let mut state_change_request = HttpRequest { method: "POST".to_string(), .. HttpRequest::default() };
        if self.state_change_body {
          let json_body = json!({
                    "state".to_string() : provider_state.name.clone(),
                    "params".to_string() : provider_state.params.clone(),
                    "action".to_string() : if setup {
                        "setup".to_string()
                    } else {
                        "teardown".to_string()
                    }
                });
          state_change_request.body = OptionalBody::Present(json_body.to_string().into(), Some(JSON.clone()), None);
          state_change_request.headers = Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] });
        } else {
          let mut query = hashmap!{ "state".to_string() => vec![Some(provider_state.name.clone())] };
          if setup {
            query.insert("action".to_string(), vec![Some("setup".to_string())]);
          } else {
            query.insert("action".to_string(), vec![Some("teardown".to_string())]);
          }
          for (k, v) in provider_state.params.clone() {
            query.insert(k, vec![match v {
              Value::String(ref s) => Some(s.clone()),
              _ => Some(v.to_string())
            }]);
          }
          state_change_request.query = Some(query);
        }
        make_state_change_request(client.unwrap_or(&DEFAULT_CLIENT), &state_change_url, &state_change_request, self.retries).await
          .map_err(|err| ProviderStateError { description: err.to_string(), interaction_id }.into())
      },
      None => {
        if !provider_state.name.is_empty() && setup {
          warn!("State Change ignored as there is no state change URL provided for interaction {}", interaction_id.unwrap_or_default());
        }
        Ok(hashmap!{})
      }
    }
  }

  fn teardown(
    self: &Self,
  ) -> bool {
    return self.state_change_teardown;
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use super::{DEFAULT_STATE_CHANGE_RETRIES, state_change_retries};

  #[test]
  fn state_change_retries_defaults_to_eight() {
    expect!(state_change_retries(None)).to(be_equal_to(DEFAULT_STATE_CHANGE_RETRIES));
    expect!(DEFAULT_STATE_CHANGE_RETRIES).to(be_equal_to(8));
  }

  #[test]
  fn state_change_retries_can_be_overridden() {
    expect!(state_change_retries(Some("3".to_string()))).to(be_equal_to(3));
    expect!(state_change_retries(Some(" 0 ".to_string()))).to(be_equal_to(0));
  }

  #[test]
  fn state_change_retries_ignores_an_override_that_is_not_a_count() {
    expect!(state_change_retries(Some("lots".to_string()))).to(be_equal_to(DEFAULT_STATE_CHANGE_RETRIES));
    expect!(state_change_retries(Some("-1".to_string()))).to(be_equal_to(DEFAULT_STATE_CHANGE_RETRIES));
    expect!(state_change_retries(Some("256".to_string()))).to(be_equal_to(DEFAULT_STATE_CHANGE_RETRIES));
  }
}
