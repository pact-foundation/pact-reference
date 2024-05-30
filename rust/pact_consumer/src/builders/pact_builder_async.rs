use std::future::Future;
use std::path::PathBuf;

use async_trait::async_trait;
use pact_models::{Consumer, Provider};
use pact_models::interaction::Interaction;
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::sync_message::SynchronousMessage;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::CatalogueEntryType;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_manager::{drop_plugin_access, load_plugin};
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::{PluginDependency, PluginDependencyType};
use tracing::trace;

use pact_matching::metrics::{MetricEvent, send_metrics};

use crate::builders::message_builder::MessageInteractionBuilder;
use crate::builders::message_iter::{asynchronous_messages_iter, MessageIterator, synchronous_messages_iter};
use crate::builders::sync_message_builder::SyncMessageInteractionBuilder;
use crate::mock_server::http_mock_server::ValidatingHttpMockServer;
#[cfg(feature = "plugins")] use crate::mock_server::plugin_mock_server::PluginMockServer;
use crate::mock_server::StartMockServerAsync;
use crate::PACT_CONSUMER_VERSION;
use crate::prelude::*;

use super::interaction_builder::InteractionBuilder;

/// Builder for `Pact` objects (async version).
///
/// ```
/// use pact_consumer::prelude::*;
/// use pact_consumer::*;
///
/// # tokio_test::block_on(async {
/// let pact = PactBuilderAsync::new("Greeting Client", "Greeting Server")
///     .interaction("asks for a greeting", "", |mut i| async move {
///         i.request.path("/greeting/hello");
///         i.response
///             .header("Content-Type", "application/json")
///             .json_body(json_pattern!({ "message": "hello" }));
///         i
///     })
///     .await
///     .build();
///
/// // The request method and response status default as follows.
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().request.method, "GET");
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().response.status, 200);
/// # });
/// ```
#[derive(Debug)]
pub struct PactBuilderAsync {
  pact: Box<dyn Pact + Send + Sync>,
  output_dir: Option<PathBuf>
}

impl PactBuilderAsync {
    /// Create a new `PactBuilder`, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new<C, P>(consumer: C, provider: P) -> Self
    where
        C: Into<String>,
        P: Into<String>,
    {
        pact_matching::matchers::configure_core_catalogue();
        pact_mock_server::configure_core_catalogue();

        let mut pact = RequestResponsePact::default();
        pact.consumer = Consumer {
            name: consumer.into(),
        };
        pact.provider = Provider {
            name: provider.into(),
        };

        if let Some(version) = PACT_CONSUMER_VERSION {
          pact.add_md_version("consumer", version);
        }

      PactBuilderAsync { pact: pact.boxed(), output_dir: None }
    }

    /// Create a new `PactBuilder` for a V4 specification Pact, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new_v4<C, P>(consumer: C, provider: P) -> Self
      where
        C: Into<String>,
        P: Into<String>
    {
      pact_matching::matchers::configure_core_catalogue();
      pact_mock_server::configure_core_catalogue();

      let mut pact = V4Pact {
        consumer: Consumer { name: consumer.into() },
        provider: Provider { name: provider.into() },
        .. V4Pact::default()
      };

      if let Some(version) = PACT_CONSUMER_VERSION {
        pact.add_md_version("consumer", version);
      }

      PactBuilderAsync { pact: pact.boxed(), output_dir: None }
    }

    #[cfg(feature = "plugins")]
    pub(crate) fn from_builder(pact: Box<dyn Pact + Send + Sync>, output_dir: Option<PathBuf>) -> Self {
      PactBuilderAsync {
        pact,
        output_dir
      }
    }

    /// Add a plugin to be used by the test. This requires the plugins feature.
    ///
    /// Panics:
    /// Plugins only work with V4 specification pacts. This method will panic if the pact
    /// being built is V3 format. Use `PactBuilder::new_v4` to create a builder with a V4 format
    /// pact.
    #[cfg(feature = "plugins")]
    pub async fn using_plugin(&mut self, name: &str, version: Option<String>) -> &mut Self {
      if !self.pact.is_v4() {
        panic!("Plugins require V4 specification pacts. Use PactBuilder::new_v4");
      }

      let result = load_plugin(&PluginDependency {
        name: name.to_string(),
        version,
        dependency_type: Default::default()
      }).await;
      match result {
        Ok(plugin) => self.pact.add_plugin(plugin.manifest().name.as_str(), plugin.manifest().version.as_str(), None)
          .expect("Could not add plugin to pact"),
        Err(err) => panic!("Could not load plugin - {}", err)
      }

      self
    }

    /// Add a new HTTP `Interaction` to the `Pact`. Needs to return a clone of the builder
    /// that is passed in.
    pub async fn interaction<D, F, O>(&mut self, description: D, interaction_type: D, build_fn: F) -> &mut Self
    where
        D: Into<String>,
        F: FnOnce(InteractionBuilder) -> O,
        O: Future<Output=InteractionBuilder> + Send
    {
        let interaction = InteractionBuilder::new(description.into(), interaction_type.into());
        let interaction = build_fn(interaction).await;

        if self.pact.is_v4() {
          #[cfg(feature = "plugins")]
          for (plugin_name, plugin_config) in interaction.pact_plugin_config() {
            // TODO: Update this when there is an update plugin data function added to Pact models
            if let Some(plugin_data) = self.pact.plugin_data().iter().find(|pl| pl.name == plugin_name) {
              self.pact.add_plugin(plugin_data.name.as_str(), plugin_data.version.as_str(),
                Some(plugin_config)).expect("Could not update Pact with plugin data");
            }
          }
          self.push_interaction(&interaction.build_v4())
        } else {
          self.push_interaction(&interaction.build())
        }
    }

    /// Directly add a pre-built `Interaction` to our `Pact`. Normally it's
    /// easier to use `interaction` instead of this function.
    pub fn push_interaction(&mut self, interaction: &dyn Interaction) -> &mut Self {
      trace!("Adding interaction {:?}", interaction);
      self.pact.add_interaction(interaction).unwrap();
      self
    }

  /// Return the `Pact` we've built.
  pub fn build(&self) -> Box<dyn Pact + Send + Sync> {
    trace!("Building Pact -> {:?}", self.pact);
    self.pact.boxed()
  }

  /// Sets the output directory to write pact files to
  pub fn output_dir<D: Into<PathBuf>>(&mut self, dir: D) -> &mut Self {
    self.output_dir = Some(dir.into());
    self
  }

  /// Add a new Asynchronous message `Interaction` to the `Pact`. Needs to return a clone of the builder
  /// that is passed in.
  pub async fn message_interaction<D, F, O>(&mut self, description: D, build_fn: F) -> &mut Self
    where
      D: Into<String>,
      F: FnOnce(MessageInteractionBuilder) -> O,
      O: Future<Output=MessageInteractionBuilder> + Send
  {
    let interaction = MessageInteractionBuilder::new(description.into());
    let interaction = build_fn(interaction).await;

    #[cfg(feature = "plugins")]
    if let Some(plugin_data) = interaction.plugin_config() {
      let _ = self.pact.add_plugin(plugin_data.name.as_str(), plugin_data.version.as_str(),
                           Some(plugin_data.configuration.clone()));
    }

    self.push_interaction(&interaction.build())
  }


  /// Add a new synchronous message `Interaction` to the `Pact`. Needs to return a clone of the builder
  /// that is passed in.
  pub async fn synchronous_message_interaction<D, F, O>(&mut self, description: D, build_fn: F) -> &mut Self
    where
      D: Into<String>,
      F: FnOnce(SyncMessageInteractionBuilder) -> O,
      O: Future<Output=SyncMessageInteractionBuilder> + Send
  {
    let interaction = SyncMessageInteractionBuilder::new(description.into());
    let interaction = build_fn(interaction).await;

    #[cfg(feature = "plugins")]
    if let Some(plugin_data) = interaction.plugin_config() {
      let _ = self.pact.add_plugin(plugin_data.name.as_str(), plugin_data.version.as_str(),
                                   Some(plugin_data.configuration.clone()));
    }

    self.push_interaction(&interaction.build())
  }

  /// Returns an iterator over the asynchronous messages in the Pact
  pub fn messages(&self) -> MessageIterator<AsynchronousMessage> {
    send_metrics(MetricEvent::ConsumerTestRun {
      interactions: self.pact.interactions().len(),
      test_framework: "pact_consumer".to_string(),
      app_name: "pact_consumer".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    });
    asynchronous_messages_iter(self.pact.as_v4_pact().unwrap(), &self.output_dir)
  }

  /// Returns an iterator over the synchronous req/res messages in the Pact
  pub fn synchronous_messages(&self) -> MessageIterator<SynchronousMessage> {
    send_metrics(MetricEvent::ConsumerTestRun {
      interactions: self.pact.interactions().len(),
      test_framework: "pact_consumer".to_string(),
      app_name: "pact_consumer".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    });
    synchronous_messages_iter(self.pact.as_v4_pact().unwrap(), &self.output_dir)
  }
}

impl StartMockServer for PactBuilderAsync {
  fn start_mock_server(&self, _catalog_entry: Option<&str>) -> Box<dyn ValidatingMockServer> {
    #[cfg(feature = "plugins")]
    {
      match _catalog_entry {
        Some(entry_name) => match catalogue_manager::lookup_entry(entry_name) {
          Some(entry) => if entry.entry_type == CatalogueEntryType::TRANSPORT {
            PluginMockServer::start(self.build(), self.output_dir.clone(), &entry)
              .expect("Could not start the plugin mock server")
          } else {
            panic!("Catalogue entry for key '{}' is not for a network transport", entry_name);
          }
          None => panic!("Did not find a catalogue entry for key '{}'", entry_name)
        }
        None => ValidatingHttpMockServer::start(self.build(), self.output_dir.clone())
      }
    }

    #[cfg(not(feature = "plugins"))]
    {
      ValidatingHttpMockServer::start(self.build(), self.output_dir.clone())
    }
  }
}

#[async_trait]
impl StartMockServerAsync for PactBuilderAsync {
  async fn start_mock_server_async(&self, _catalog_entry: Option<&str>) -> Box<dyn ValidatingMockServer> {
    #[cfg(feature = "plugins")]
    {
      match _catalog_entry {
        Some(entry_name) => match catalogue_manager::lookup_entry(entry_name) {
          Some(entry) => if entry.entry_type == CatalogueEntryType::TRANSPORT {
            PluginMockServer::start_async(self.build(), self.output_dir.clone(), &entry).await
              .expect("Could not start the plugin mock server")
          } else {
            panic!("Catalogue entry for key '{}' is not for a network transport", entry_name);
          }
          None => panic!("Did not find a catalogue entry for key '{}'", entry_name)
        }
        None => ValidatingHttpMockServer::start_async(self.build(), self.output_dir.clone()).await
      }
    }

    #[cfg(not(feature = "plugins"))]
    {
      ValidatingHttpMockServer::start_async(self.build(), self.output_dir.clone()).await
    }
  }
}

#[cfg(feature = "plugins")]
impl Drop for PactBuilderAsync {
  fn drop(&mut self) {
    // decrement access to any plugin loaded for the Pact
    for plugin in self.pact.plugin_data() {
      let dependency = PluginDependency {
        name: plugin.name,
        version: Some(plugin.version),
        dependency_type: PluginDependencyType::Plugin
      };
      drop_plugin_access(&dependency);
    }
  }
}
