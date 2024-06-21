//! Interface to a standard HTTP mock server provided by Pact

use std::{env, thread};
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::anyhow;

use pact_matching::metrics::{MetricEvent, send_metrics};
use pact_mock_server::builder::MockServerBuilder;
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server;
use pact_mock_server::mock_server::{MockServerConfig, MockServerMetrics};
use pact_models::pact::Pact;
#[cfg(feature = "plugins")] use pact_models::plugins::PluginData;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_manager::{drop_plugin_access, increment_plugin_access};
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::{PluginDependency, PluginDependencyType};
use tokio::runtime::Runtime;
#[allow(unused_imports)] use tracing::{debug, trace, warn};
use url::Url;

use crate::mock_server::ValidatingMockServer;
use crate::util::panic_or_print_error;

/// A mock HTTP server that handles the requests described in a `Pact`, intended
/// for use in tests, and validates that the requests made to that server are
/// correct. This wraps the standard Pact HTTP mock server.
///
/// Because this is intended for use in tests, it will panic if something goes
/// wrong.
pub struct ValidatingHttpMockServer {
  // A description of our mock server, for use in error messages.
  description: String,
  // The URL of our mock server.
  url: Url,
  // The mock server instance
  mock_server: mock_server::MockServer,
  // Output directory to write pact files
  output_dir: Option<PathBuf>,
  // overwrite or merge Pact files
  overwrite: bool,
  // Tokio Runtime used to drive the mock server
  runtime: Option<Arc<Runtime>>
}

impl ValidatingHttpMockServer {
  /// Create a new mock server which handles requests as described in the
  /// pact, and runs in a background thread
  ///
  /// Panics:
  /// Will panic if the provided Pact can not be sent to the background thread.
  pub fn start(
    pact: Box<dyn Pact + Send + Sync>,
    output_dir: Option<PathBuf>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer> {
    debug!("Starting mock server from pact {:?}", pact);

    // Start a tokio runtime to drive the mock server
    let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .worker_threads(1)
      .build()
      .expect("Could not start a new Tokio runtime"));

    #[cfg(feature = "plugins")]
    Self::increment_plugin_access(&pact.plugin_data());

    // Start a background thread to run the mock server tasks on the runtime
    let tname = format!("test({})-pact-mock-server",
      thread::current().name().unwrap_or("<unknown>")
    );
    let rt = runtime.clone();
    let mock_server = thread::Builder::new()
      .name(tname)
      .spawn(move || {
        let mut builder = MockServerBuilder::new()
          .with_pact(pact);
        if let Some(config) = mock_server_config {
            builder = builder.with_config(config);
        }
        if !builder.address_assigned() {
          builder = builder.bind_to_ip4_port(0)
        };
        rt.block_on(builder.start())
      })
      .expect("INTERNAL ERROR: Could not spawn a thread to run the mock server")
      .join()
      .expect("INTERNAL ERROR: Failed to spawn the mock server task onto the runtime")
      .expect("Failed to start the mock server");

    let pact = &mock_server.pact;
    let description = format!("{}/{}", pact.consumer().name, pact.provider().name);
    let url_str = mock_server.url();

    Box::new(ValidatingHttpMockServer {
      description,
      url: url_str.parse().expect(format!("invalid mock server URL '{}'", url_str).as_str()),
      mock_server,
      output_dir,
      overwrite: false,
      runtime: Some(runtime)
    })
  }

  #[cfg(feature = "plugins")]
  fn decrement_plugin_access(plugins: &Vec<PluginData>) {
    for plugin in plugins {
      let dependency = PluginDependency {
        name: plugin.name.clone(),
        version: Some(plugin.version.clone()),
        dependency_type: PluginDependencyType::Plugin
      };
      drop_plugin_access(&dependency);
    }
  }

  #[cfg(feature = "plugins")]
  fn increment_plugin_access(plugins: &Vec<PluginData>) {
    for plugin in plugins {
      let dependency = PluginDependency {
        name: plugin.name.clone(),
        version: Some(plugin.version.clone()),
        dependency_type: PluginDependencyType::Plugin
      };
      increment_plugin_access(&dependency);
    }
  }

  /// Create a new mock server which handles requests as described in the
  /// pact, and runs in a background task in the current Tokio runtime.
  ///
  /// Panics:
  /// Will panic if unable to get the URL to the spawned mock server
  pub async fn start_async(
    pact: Box<dyn Pact + Send + Sync>,
    output_dir: Option<PathBuf>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer> {
    debug!("Starting mock server from pact {:?}", pact);

    #[cfg(feature = "plugins")] Self::increment_plugin_access(&pact.plugin_data());

    let mut builder = MockServerBuilder::new()
      .with_pact(pact);
    if let Some(config) = mock_server_config {
      builder = builder.with_config(config);
    }
    if !builder.address_assigned() {
      builder = builder.bind_to_ip4_port(0)
    };
    let mock_server = builder
      .start()
      .await
      .expect("Could not start the mock server");

    let pact = &mock_server.pact;
    let description = format!("{}/{}", pact.consumer().name, pact.provider().name);
    let url_str = mock_server.url();
    Box::new(ValidatingHttpMockServer {
      description,
      url: url_str.parse().expect("invalid mock server URL"),
      mock_server,
      output_dir,
      overwrite: false,
      runtime: None
    })
  }

  /// Helper function called by our `drop` implementation. This basically exists
  /// so that it can return `Err(message)` whenever needed without making the
  /// flow control in `drop` ultra-complex.
  fn drop_helper(&mut self) -> anyhow::Result<()> {
    // Kill the mock server
    self.mock_server.shutdown()?;

    #[cfg(feature = "plugins")] Self::decrement_plugin_access(&self.mock_server.pact.plugin_data());

    // If there is a Tokio runtime for the mock server, try shut that down
    if let Some(runtime) = self.runtime.take() {
      if let Some(runtime) = Arc::into_inner(runtime) {
        runtime.shutdown_background();
      }
    }

    // Send any metrics in another thread as this thread could be panicking due to an assertion.
    let interactions = self.mock_server.pact.interactions().len();
    thread::spawn(move || {
      send_metrics(MetricEvent::ConsumerTestRun {
        interactions,
        test_framework: "pact_consumer".to_string(),
        app_name: "pact_consumer".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string()
      });
    });

    // Look up any mismatches which occurred with the mock server.
    let mismatches = self.mock_server.mismatches();
    if mismatches.is_empty() {
      // Success! Write out the generated pact file.
      let output_dir = self.output_dir.as_ref()
        .map(|dir| {
          let dir = dir.to_string_lossy().to_string();
          if dir.is_empty() { None } else { Some(dir) }
        })
        .flatten()
        .unwrap_or_else(|| {
          let val = env::var("PACT_OUTPUT_DIR");
          debug!("env:PACT_OUTPUT_DIR = {:?}", val);
          val.unwrap_or_else(|_| "target/pacts".to_owned())
        });
      debug!("Pact output_dir = '{}'", output_dir);
      let overwrite = env::var("PACT_OVERWRITE")
        .map(|v| {
          debug!("env:PACT_OVERWRITE = {:?}", v);
          v == "true"
        })
        .ok()
        .unwrap_or(self.overwrite);
      self.mock_server.write_pact(&Some(output_dir), overwrite)
        .map_err(|err| anyhow!("error writing pact: {}", err))?;
      Ok(())
    } else {
      // Failure. Format our errors.
      let mut msg = format!("mock server {} failed verification:\n", self.description,);
      for mismatch in mismatches {
        match mismatch {
          MatchResult::RequestMatch(..) => {
            unreachable!("list of mismatches contains a match");
          }
          MatchResult::RequestMismatch(request, _, mismatches) => {
            let _ = writeln!(&mut msg, "- request {}:", request);
            for m in mismatches {
              let _ = writeln!(&mut msg, "  - {}", m.description());
            }
          }
          MatchResult::RequestNotFound(request) => {
            let _ = writeln!(&mut msg, "- received unexpected request:");
            let _ = writeln!(&mut msg, "{:#?}", request);
          }
          MatchResult::MissingRequest(request) => {
            let _ = writeln!(
              &mut msg,
              "- request {} expected, but never occurred", request,
            );
            let _ = writeln!(&mut msg, "{:#?}", request);
          }
        }
      }
      Err(anyhow!(msg))
    }
  }
}

impl ValidatingMockServer for ValidatingHttpMockServer {
  fn url(&self) -> Url {
    self.url.clone()
  }

  fn path(&self, path: &str) -> Url {
    // We panic here because this a _test_ library, the `?` operator is
    // useless in tests, and filling up our test code with piles of `unwrap`
    // calls is ugly.
    self.url.join(path.as_ref()).expect("could not parse URL")
  }

  fn status(&self) -> Vec<MatchResult> {
    self.mock_server.mismatches()
  }

  fn metrics(&self) -> MockServerMetrics {
    self.mock_server.metrics.lock().unwrap().clone()
  }
}

impl Drop for ValidatingHttpMockServer {
  fn drop(&mut self) {
    let result = self.drop_helper();
    if let Err(msg) = result {
      panic_or_print_error(&msg);
    }
  }
}
