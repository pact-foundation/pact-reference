use std::fmt::{Debug, Formatter};
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use pact_mock_server::mock_server::MockServerConfig;

use pact_models::{Consumer, Provider};
use pact_models::interaction::Interaction;
use pact_models::message::Message;
use pact_models::pact::Pact;
use pact_models::prelude::MessagePact;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::sync_message::SynchronousMessage;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::CatalogueEntryType;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_manager::load_plugin;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::PluginDependency;
use tracing::trace;

use pact_matching::metrics::{MetricEvent, send_metrics};

use crate::builders::message_builder::MessageInteractionBuilder;
use crate::builders::message_iter::{
  asynchronous_messages_iter,
  messages_iter,
  MessageIterator,
  synchronous_messages_iter
};
#[cfg(feature = "plugins")] use crate::builders::pact_builder_async::PactBuilderAsync;
use crate::builders::sync_message_builder::SyncMessageInteractionBuilder;
use crate::mock_server::http_mock_server::ValidatingHttpMockServer;
#[cfg(feature = "plugins")] use crate::mock_server::plugin_mock_server::PluginMockServer;
use crate::PACT_CONSUMER_VERSION;
use crate::prelude::*;

use super::interaction_builder::InteractionBuilder;

/// Builder for `Pact` objects.
///
/// ```
/// use pact_consumer::prelude::*;
/// use pact_consumer::*;
///
/// let pact = PactBuilder::new("Greeting Client", "Greeting Server")
///     .interaction("asks for a greeting", "", |mut i| {
///         i.request.path("/greeting/hello");
///         i.response
///             .header("Content-Type", "application/json")
///             .json_body(json_pattern!({ "message": "hello" }));
///         i
///     })
///     .build();
///
/// // The request method and response status default as follows.
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().request.method, "GET");
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().response.status, 200);
/// ```
pub struct PactBuilder {
  pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  output_dir: Option<PathBuf>
}

impl PactBuilder {
    /// Create a new `PactBuilder`, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new<C, P>(consumer: C, provider: P) -> Self
    where
        C: Into<String>,
        P: Into<String>,
    {
        pact_matching::matchingrules::configure_core_catalogue();
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

        PactBuilder { pact: pact.boxed(), output_dir: None }
    }

  /// Create a new `PactBuilder`, specifying the names of the service
  /// consuming the API and the service providing it. This creates a V3 message pact, and only
  /// message interactions can be added.
  pub fn new_v3_message<C, P>(consumer: C, provider: P) -> Self
    where
      C: Into<String>,
      P: Into<String>,
  {
    pact_matching::matchingrules::configure_core_catalogue();
    pact_mock_server::configure_core_catalogue();

    let mut pact = MessagePact::default();
    pact.consumer = Consumer {
      name: consumer.into(),
    };
    pact.provider = Provider {
      name: provider.into(),
    };

    if let Some(version) = PACT_CONSUMER_VERSION {
      pact.add_md_version("consumer", version);
    }

    PactBuilder { pact: pact.boxed(), output_dir: None }
  }

    /// Create a new `PactBuilder` for a V4 specification Pact, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new_v4<C, P>(consumer: C, provider: P) -> Self
      where
        C: Into<String>,
        P: Into<String>
    {
      pact_matching::matchingrules::configure_core_catalogue();
      pact_mock_server::configure_core_catalogue();

      let mut pact = V4Pact {
        consumer: Consumer { name: consumer.into() },
        provider: Provider { name: provider.into() },
        .. V4Pact::default()
      };

      if let Some(version) = PACT_CONSUMER_VERSION {
        pact.add_md_version("consumer", version);
      }

      PactBuilder { pact: pact.boxed(), output_dir: None }
    }

  ///  Sets the output directory to write any pact files to. If this is not set, will default
  /// to the PACT_OUTPUT_DIR environment variable. If that is not set, will use the target
  /// directory.
    pub fn with_output_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
    self.output_dir = Some(dir.into());
    self
  }

    /// Add a plugin to be used by the test. Note this will return an async version of the Pact
    /// builder and requires the plugin crate feature.
    ///
    /// Panics:
    /// Plugins only work with V4 specification pacts. This method will panic if the pact
    /// being built is V3 format. Use `PactBuilder::new_v4` to create a builder with a V4 format
    /// pact.
    #[cfg(feature = "plugins")]
    pub async fn using_plugin(self, name: &str, version: Option<String>) -> PactBuilderAsync {
      if !self.pact.is_v4() {
        panic!("Plugins require V4 specification pacts. Use PactBuilder::new_v4");
      }

      let result = load_plugin(&PluginDependency {
        name: name.to_string(),
        version,
        dependency_type: Default::default()
      }).await;

      let mut pact = self.pact.boxed();
      match result {
        Ok(plugin) => pact.add_plugin(plugin.manifest.name.as_str(), plugin.manifest.version.as_str(), None)
          .expect("Could not add plugin to pact"),
        Err(err) => panic!("Could not load plugin - {}", err)
      }

      PactBuilderAsync::from_builder(pact, self.output_dir.clone())
    }

    /// Add a new HTTP `Interaction` to the `Pact`. Needs to return a clone of the builder
    /// that is passed in.
    pub fn interaction<D, F>(&mut self, description: D, interaction_type: D, build_fn: F) -> &mut Self
    where
        D: Into<String>,
        F: FnOnce(InteractionBuilder) -> InteractionBuilder
    {
        let interaction = InteractionBuilder::new(description.into(), interaction_type.into());
        let interaction = build_fn(interaction);

        if self.pact.is_v4() {
          self.push_interaction(&interaction.build_v4())
        } else {
          self.push_interaction(&interaction.build())
        }
    }

    /// Directly add a pre-built `Interaction` to our `Pact`. Normally it's
    /// easier to use `interaction` instead of this function.
    pub fn push_interaction(&mut self, interaction: &(dyn Interaction + Send + Sync + RefUnwindSafe)) -> &mut Self {
      trace!("Adding interaction {:?}", interaction);
      self.pact.add_interaction(interaction).unwrap();
      self
    }

  /// Return the `Pact` we've built.
  pub fn build(&self) -> Box<dyn Pact + Send + Sync + RefUnwindSafe> {
    trace!("Building Pact -> {:?}", self.pact);
    self.pact.boxed()
  }

  /// Sets the output directory to write pact files to
  #[deprecated(note = "Use with_output_dir")]
  pub fn output_dir<D: Into<PathBuf>>(&mut self, dir: D) -> &mut Self {
    self.output_dir = Some(dir.into());
    self
  }

  /// Add a new Asynchronous message `Interaction` to the `Pact`
  pub fn message_interaction<D, F>(&mut self, description: D, build_fn: F) -> &mut Self
    where
      D: Into<String>,
      F: FnOnce(MessageInteractionBuilder) -> MessageInteractionBuilder
  {
    let interaction = MessageInteractionBuilder::new(description.into());
    let interaction = build_fn(interaction);

    #[cfg(feature = "plugins")]
    if let Some(plugin_data) = interaction.plugin_config() {
      let _ = self.pact.add_plugin(plugin_data.name.as_str(), plugin_data.version.as_str(),
                           Some(plugin_data.configuration.clone()));
    }

    self.push_interaction(&interaction.build())
  }


  /// Add a new synchronous message `Interaction` to the `Pact`
  pub fn synchronous_message_interaction<D, F>(&mut self, description: D, build_fn: F) -> &mut Self
    where
      D: Into<String>,
      F: FnOnce(SyncMessageInteractionBuilder) -> SyncMessageInteractionBuilder
  {
    let interaction = SyncMessageInteractionBuilder::new(description.into());
    let interaction = build_fn(interaction);

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

  /// Returns an iterator over the asynchronous messages in a V3 Message Pact
  pub fn v3_messages(&self) -> MessageIterator<Message> {
    send_metrics(MetricEvent::ConsumerTestRun {
      interactions: self.pact.interactions().len(),
      test_framework: "pact_consumer".to_string(),
      app_name: "pact_consumer".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    });
    messages_iter(self.pact.as_message_pact().unwrap(), &self.output_dir)
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

impl StartMockServer for PactBuilder {
  fn start_mock_server(
    &self,
    _catalog_entry: Option<&str>,
    mock_server_config: Option<MockServerConfig>
  ) -> Box<dyn ValidatingMockServer> {
    #[cfg(feature = "plugins")]
    {
      match _catalog_entry {
        Some(entry_name) => match catalogue_manager::lookup_entry(entry_name) {
          Some(entry) => if entry.entry_type == CatalogueEntryType::TRANSPORT {
            PluginMockServer::start(self.build(), self.output_dir.clone(), &entry, mock_server_config)
              .expect("Could not start the plugin mock server")
          } else {
            panic!("Catalogue entry for key '{}' is not for a network transport", entry_name);
          }
          None => panic!("Did not find a catalogue entry for key '{}'", entry_name)
        }
        None => ValidatingHttpMockServer::start(self.build(), self.output_dir.clone(), mock_server_config)
      }
    }

    #[cfg(not(feature = "plugins"))]
    {
      ValidatingHttpMockServer::start(self.build(), self.output_dir.clone(), mock_server_config)
    }
  }
}

impl Debug for PactBuilder {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "PactBuilder({:?}, {:?})", self.pact, self.output_dir)
  }
}

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules::{Category, MatchingRuleCategory, MatchingRules};
  use pact_models::provider_states::ProviderState;
  use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
  use pact_models::v4::synch_http::SynchronousHttp;
  use serde_json::Value;

  use crate::builders::{HttpPartBuilder, PactBuilder};

  #[test]
  fn v4_calc_key_test() {
    let pact = PactBuilder::new_v4("Consumer", "Alice Service")
      // Start a new interaction. We can add as many interactions as we want.
      .interaction("a retrieve Mallory request", "", |mut i| {
        // Defines a provider state. It is optional.
        i.given("there is some good mallory");
        // Define the request, a GET (default) request to '/mallory'.
        i.request.path("/mallory");
        i.request.header("Content-Type", "application/json");
        // Define the response we want returned.
        i.response
          .ok()
          .content_type("text/plain")
          .body("That is some good Mallory.");

        // Return the interaction back to the pact framework
        i.clone()
      }).build();
    let interactions = pact.interactions();
    let interaction = interactions.first().unwrap();
    let synchronous_http = interaction.as_v4_http().unwrap();

    pretty_assertions::assert_eq!(SynchronousHttp {
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ] }),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::HEADER => MatchingRuleCategory::empty("HEADER"),
            Category::PATH => MatchingRuleCategory::empty("PATH")
          }
        },
        .. HttpRequest::default()
      },
      response: HttpResponse {
        headers: Some(hashmap!{ "content-type".to_string() => vec![ "text/plain".to_string() ] }),
        body: OptionalBody::Present(Bytes::from("That is some good Mallory."), None, None),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::HEADER => MatchingRuleCategory::empty("HEADER")
          }
        },
        .. HttpResponse::default()
      },
      comments: hashmap!{
        "testname".to_string() => Value::Null,
        "text".to_string() => Value::Array(vec![])
      },
      .. SynchronousHttp::default()
    }, synchronous_http);

    let v4interaction = synchronous_http.with_key();
    pretty_assertions::assert_eq!(SynchronousHttp {
      key: Some("93371e6e7ae2556".to_string()),
      description: "a retrieve Mallory request".to_string(),
      provider_states: vec![ProviderState::default("there is some good mallory")],
      request: HttpRequest {
        path: "/mallory".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec![ "application/json".to_string() ] }),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::HEADER => MatchingRuleCategory::empty("HEADER"),
            Category::PATH => MatchingRuleCategory::empty("PATH")
          }
        },
        .. HttpRequest::default()
      },
      response: HttpResponse {
        headers: Some(hashmap!{ "content-type".to_string() => vec![ "text/plain".to_string() ] }),
        body: OptionalBody::Present(Bytes::from("That is some good Mallory."), None, None),
        matching_rules: MatchingRules {
          rules: hashmap!{
            Category::HEADER => MatchingRuleCategory::empty("HEADER")
          }
        },
        .. HttpResponse::default()
      },
      comments: hashmap!{
        "testname".to_string() => Value::Null,
        "text".to_string() => Value::Array(vec![])
      },
      .. SynchronousHttp::default()
    }, v4interaction);
    expect!(v4interaction.key.as_ref().unwrap()).to(be_equal_to("93371e6e7ae2556"));
  }
}
