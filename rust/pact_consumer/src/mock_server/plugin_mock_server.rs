//! Interface to a mock server provided by a plugin

use std::{env, thread};
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use itertools::Itertools;
use maplit::hashmap;
use pact_models::pact::{Pact, write_pact};
use pact_models::PactSpecification;
use pact_plugin_driver::catalogue_manager::CatalogueEntry;
use pact_plugin_driver::mock_server::{MockServerConfig, MockServerDetails};
use pact_plugin_driver::plugin_manager::{shutdown_mock_server, start_mock_server_v2};
use tokio::runtime::Handle;
use tracing::{debug, info};
use url::Url;

use pact_matching::metrics::{MetricEvent, send_metrics_async};
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::MockServerMetrics;

use crate::mock_server::ValidatingMockServer;
use crate::util::panic_or_print_error;

/// Mock server that has been provided by a plugin
pub struct PluginMockServer {
  /// Details of the running mock server
  pub mock_server_details: MockServerDetails,
  /// Pact that is used to configure the mock server
  pub pact: Box<dyn Pact + Send + Sync>,
  /// Path to write any pact files to
  pub output_path: Option<PathBuf>,
  /// Catalogue entry for the transport
  pub catalogue_entry: CatalogueEntry,
}

impl PluginMockServer {
  /// Start a new plugin mock server. This will send the start mock server request to the plugin
  /// that provides the mock server. A new Tokio reactor will be started.
  pub fn start(
    pact: Box<dyn Pact + Send + Sync>,
    output_path: Option<PathBuf>,
    catalogue_entry: &CatalogueEntry
  ) -> anyhow::Result<Box<dyn ValidatingMockServer>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()?;
    runtime.block_on(async { PluginMockServer::start_async(pact, output_path, catalogue_entry).await })
  }

  /// Start a new plugin mock server (async version). This will send the start mock server request
  /// to the plugin that provides the mock server.
  pub async fn start_async(
    pact: Box<dyn Pact + Send + Sync>,
    output_path: Option<PathBuf>,
    catalogue_entry: &CatalogueEntry
  ) -> anyhow::Result<Box<dyn ValidatingMockServer>> {
    let test_context = hashmap!{};
    let result = start_mock_server_v2(catalogue_entry, pact.boxed(), MockServerConfig {
      output_path: output_path.clone(),
      host_interface: None,
      port: 0,
      tls: false
    }, test_context).await?;
    Ok(Box::new(PluginMockServer {
      mock_server_details: result,
      pact: pact.boxed(),
      output_path: output_path.clone(),
      catalogue_entry: catalogue_entry.clone()
    }))
  }

  /// Helper to shutdown the mock server and get the results
  pub(crate) fn drop_helper(&self) -> anyhow::Result<()> {
    let handle = Handle::try_current()
      .or_else(|_| tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .map(|runtime| runtime.handle().clone()))?;

    let interactions = self.pact.interactions().len();
    let mock_server_details = self.mock_server_details.clone();
    let result = thread::spawn(move || {
      handle.block_on(async {
        send_metrics_async(MetricEvent::ConsumerTestRun {
          interactions,
          test_framework: "pact_consumer".to_string(),
          app_name: "pact_consumer".to_string(),
          app_version: env!("CARGO_PKG_VERSION").to_string()
        }).await;
        shutdown_mock_server(&mock_server_details).await
      })
    }).join();
    match result {
      Err(_) => Err(anyhow!("Failed to shutdown the mock server: could not start a new thread")),
      Ok(result) => match result {
        Ok(results) => {
          let results = results.iter()
            .filter(|r| !(r.error.is_empty() && r.mismatches.is_empty()))
            .collect_vec();
          if results.is_empty() {
            self.write_pact()
          } else {
            let mut message = format!("plugin mock server failed verification:\n");
            for (index, result) in results.iter().enumerate() {
              if result.error.is_empty() {
                let _ = writeln!(&mut message, "    {}) {} - the following mismatches occurred:", index + 1, result.path);
                for (mismatch, details) in result.mismatches.iter().enumerate() {
                  let _ = writeln!(&mut message, "        {}.{}) [{}] {}", index + 1, mismatch + 1, details.path, details.mismatch);
                }
              } else {
                let _ = writeln!(&mut message, "    {}) {}: {}", index + 1, result.path, result.error);
              }
            }
            Err(anyhow!(message))
          }
        }
        Err(err) => Err(anyhow!(err))
      }
    }
  }

  fn write_pact(&self) -> anyhow::Result<()> {
    let output_dir = self.output_path.as_ref().map(|dir| dir.to_string_lossy().to_string())
      .unwrap_or_else(|| {
        let val = env::var("PACT_OUTPUT_DIR");
        debug!("env:PACT_OUTPUT_DIR = {:?}", val);
        val.unwrap_or_else(|_| "target/pacts".to_owned())
      });
    let overwrite = env::var("PACT_OVERWRITE")
      .unwrap_or_else(|_| "false".to_owned()) == "true";
    debug!("env:PACT_OVERWRITE = {:?}", overwrite);

    let mut v4_pact = self.pact.as_v4_pact();
    let pact = if let Ok(ref mut pact) = v4_pact {
      for interaction in &mut pact.interactions {
        let catalogue_entry = &self.catalogue_entry;
        interaction.set_transport(Some(catalogue_entry.key.clone()));
      }
      pact.boxed()
    } else {
      self.pact.boxed()
    };

    let pact_file_name = pact.default_file_name();
    let mut filename = PathBuf::from(output_dir);
    filename.push(pact_file_name);

    info!("Writing pact out to '{}'", filename.display());
    write_pact(pact, filename.as_path(), PactSpecification::V4, overwrite)
  }
}

impl ValidatingMockServer for PluginMockServer {
  fn url(&self) -> Url {
    self.mock_server_details.base_url.parse()
      .expect("Failed to parse the URL from the plugin mock server")
  }

  fn path(&self, path: &str) -> Url {
    self.url().join(path).expect("Could not join the path to the base URL")
  }

  // TODO: need a mechanism for plugin mock servers to provide the current status
  fn status(&self) -> Vec<MatchResult> {
    vec![]
  }

  // TODO: need a mechanism for plugin mock servers to provide metrics
  fn metrics(&self) -> MockServerMetrics {
    MockServerMetrics::default()
  }
}

impl Drop for PluginMockServer {
  fn drop(&mut self) {
    let result = self.drop_helper();
    if let Err(msg) = result {
      panic_or_print_error(&msg);
    }
  }
}
