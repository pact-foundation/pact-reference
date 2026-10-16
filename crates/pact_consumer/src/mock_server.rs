//! Support for mock HTTP servers that verify pacts.

use async_trait::async_trait;
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use url::Url;

use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::{MockServerConfig, MockServerMetrics};

use crate::mock_server::http_mock_server::ValidatingHttpMockServer;

pub(crate) mod http_mock_server;
#[cfg(feature = "plugins")] pub(crate) mod plugin_mock_server;

/// A mock server that handles the requests described in a `Pact`, intended
/// for use in tests, and validates that the requests made to that server are
/// correct.
///
/// Because this is intended for use in tests, it will panic if something goes
/// wrong.
pub trait ValidatingMockServer {
  /// The base URL of the mock server. You can make normal HTTP requests using this
  /// as the base URL (if it is a HTTP-based mock server).
  fn url(&self) -> Url;

  /// Given a path string, return a URL pointing to that path on the mock
  /// server. If the `path` cannot be parsed as URL, **this function will
  /// panic**. For a non-panicking version, call `.url()` instead and build
  /// this path yourself.
  fn path(&self, path: &str) -> Url;

  /// Returns the current status of the mock server. Note that with some mock server
  /// implementations, the status will only be available once the mock server has shutdown.
  fn status(&self) -> Vec<MatchResult>;

  /// Returns the metrics collected by the mock server
  fn metrics(&self) -> MockServerMetrics;
}

/// This trait is implemented by types which allow us to start a mock server.
pub trait StartMockServer {
  /// Start a mock server running in a background thread. If the catalog entry is omitted,
  /// then a standard HTTP mock server will be started.
  fn start_mock_server(
    &self,
    catalog_entry: Option<&str>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer>;
}

/// This trait is implemented by types which allow us to start a mock server (async version).
#[async_trait]
pub trait StartMockServerAsync {
  /// Start a mock server running in a task (requires a Tokio runtime to be already setup)
  async fn start_mock_server_async(
    &self,
    catalog_entry: Option<&str>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer>;
}

impl StartMockServer for RequestResponsePact {
  fn start_mock_server(
    &self,
    _catalog_entry: Option<&str>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer> {
    ValidatingHttpMockServer::start(self.boxed(), None, mock_server_config)
  }
}

#[async_trait]
impl StartMockServerAsync for RequestResponsePact {
  async fn start_mock_server_async(
    &self,
    _catalog_entry: Option<&str>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer> {
    ValidatingHttpMockServer::start_async(self.boxed(), None, mock_server_config).await
  }
}
