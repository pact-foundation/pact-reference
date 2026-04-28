use std::collections::{BTreeMap, HashMap};

use maplit::{btreemap, hashmap};
use serde_json::{json, Value};
use tracing::debug;
use pact_models::json_utils::json_deep_merge;
use pact_models::provider_states::ProviderState;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::v4::synch_http::SynchronousHttp;

use super::request_builder::RequestBuilder;
use super::response_builder::ResponseBuilder;

/// Builder for `Interaction` objects. Normally created via
/// `PactBuilder::interaction`.
#[derive(Clone, Debug)]
pub struct InteractionBuilder {
    description: String,
    provider_states: Vec<ProviderState>,
    comments: Vec<String>,
    test_name: Option<String>,
    key: Option<String>,
    pending: Option<bool>,

    /// Protocol transport for this interaction
    transport: Option<String>,

    /// A builder for this interaction's `Request`.
    pub request: RequestBuilder,

    /// A builder for this interaction's `Response`.
    pub response: ResponseBuilder,

    /// The interaction type (as stored in the plugin catalogue)
    pub interaction_type: String,

    /// Any configuration provided by plugins that needs to be persisted to the Pact metadata
    pub plugin_configuration: HashMap<String, Value>,

    references: Option<BTreeMap<String, BTreeMap<String, Value>>>
}

impl InteractionBuilder {
  /// Create a new interaction.
  pub fn new<D: Into<String>>(description: D, interaction_type: D) -> Self {
    InteractionBuilder {
      interaction_type: interaction_type.into(),
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      key: None,
      pending: None,
      transport: None,
      request: RequestBuilder::default(),
      response: ResponseBuilder::default(),
      plugin_configuration: Default::default(),
      references: None
    }
  }

  /// Specify a unique key for this interaction. This key will be used to determine equality of
  /// the interaction, so must be unique.
  pub fn with_key<G: Into<String>>(&mut self, key: G) -> &mut Self {
    self.key = Some(key.into());
          self
  }

  /// Sets this interaction as pending. This will permantly mark the interaction as pending in the
  /// Pact file, and it will not cause a verification failure.
  pub fn pending(&mut self, pending: bool) -> &mut Self {
    self.pending = Some(pending);
    self
  }

    /// Specify a "provider state" for this interaction. This is normally use to
    /// set up database fixtures when using a pact to test a provider.
    pub fn given<G: Into<String>>(&mut self, given: G) -> &mut Self {
        self.provider_states.push(ProviderState::default(&given.into()));
        self
    }

  /// Specify a "provider state" for this interaction with some defined parameters. This is
  /// normally use to set up database fixtures when using a pact to test a provider.
  ///
  /// The paramaters must be provided as a serde_json::Value Object.
  pub fn given_with_params<G: Into<String>>(&mut self, given: G, params: &Value) -> &mut Self {
    let params = if let Some(params) = params.as_object() {
      params.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
    } else {
      HashMap::default()
    };

    self.provider_states.push(ProviderState {
      name: given.into(),
      params
    });
    self
  }

  /// Adds a text comment to this interaction. This allows to specify just a bit more information
  /// about the interaction. It has no functional impact, but can be displayed in the broker HTML
  /// page, and potentially in the test output.
  pub fn comment<G: Into<String>>(&mut self, comment: G) -> &mut Self {
    self.comments.push(comment.into());
    self
  }

  /// Sets the test name for this interaction. This allows to specify just a bit more information
  /// about the interaction. It has no functional impact, but can be displayed in the broker HTML
  /// page, and potentially in the test output.
  pub fn test_name<G: Into<String>>(&mut self, name: G) -> &mut Self {
    self.test_name = Some(name.into());
    self
  }

  /// Sets an external reference for the interaction.  The reference will be stored in the Pact
  /// file comments under the group. For instance, you could store the OpenAPI operation ID that
  /// the interaction corresponds to as an external reference.
  /// ```
  /// # let mut builder = pact_consumer::builders::InteractionBuilder::new("test", "");
  /// builder.reference("asyncapi", "operationId", "createUser");
  ///
  pub fn reference<G: Into<String>, J: Into<Value>>(&mut self, group: G, name: G, value: J) -> &mut Self {
    if let Some(references) = self.references.as_mut() {
      match references.entry(group.into()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
          entry.insert(btreemap! { name.into() => value.into() });
        }
        std::collections::btree_map::Entry::Occupied(mut entry) => {
          entry.get_mut().insert(name.into(), value.into());
        }
      }
    } else {
      self.references = Some(btreemap!{
        group.into() => btreemap!{
          name.into() => value.into()
        }
      });
    }
    self
  }

  /// Sets the protocol transport for this interaction. This would be required when there are
  /// different types of interactions in the Pact file (i.e. HTTP and messages).
  pub fn transport<G: Into<String>>(&mut self, name: G) -> &mut Self {
    self.transport = Some(name.into());
    self
  }

  /// The interaction we've built.
  pub fn build(&self) -> RequestResponseInteraction {
    RequestResponseInteraction {
      id: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.build(),
      response: self.response.build(),
    }
  }

  /// The interaction we've built (in V4 format).
  pub fn build_v4(&self) -> SynchronousHttp {
    debug!("Building V4 HTTP interaction: {:?}", self);

    let markup = self.request.interaction_markup().merge(self.response.interaction_markup());

    let mut comments = hashmap! {
      "text".to_string() => json!(self.comments),
      "testname".to_string() => json!(self.test_name)
    };
    if let Some(references) = &self.references {
      comments.insert("references".to_string(), references.iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect());
    }

    SynchronousHttp {
      id: None,
      key: self.key.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.build_v4(),
      response: self.response.build_v4(),
      comments,
      pending: self.pending.unwrap_or(false),
      plugin_config: self.plugin_config(),
      interaction_markup: markup,
      transport: self.transport.clone()
    }
  }

  /// Any plugin configuration returned from plugins to add to the interaction
  pub fn plugin_config(&self) -> HashMap<String, HashMap<String, Value>> {
    #[allow(unused_mut)] let mut config = hashmap!{};

    #[cfg(feature = "plugins")]
    {
      let request_config = self.request.plugin_config();
      if !request_config.is_empty() {
        for (key, value) in request_config {
          config.insert(key, value.interaction_configuration);
        }
      }
      let response_config = self.response.plugin_config();
      if !response_config.is_empty() {
        for (key, value) in response_config {
          let value_config = value.interaction_configuration.clone();
          config.entry(key)
            .and_modify(|entry| {
              for (k, v) in value_config {
                entry
                  .entry(k)
                  .and_modify(|e| *e = json_deep_merge(e, &v))
                  .or_insert(v);
              }
            })
            .or_insert(value.interaction_configuration);
        }
      }
    }

    config
  }

  /// Any plugin configuration returned from plugins to add to the Pact metadata
  #[cfg(feature = "plugins")]
  pub fn pact_plugin_config(&self) -> HashMap<String, HashMap<String, Value>> {
    let mut config = hashmap!{};
    let request_config = self.request.plugin_config();
    if !request_config.is_empty() {
      for (key, value) in request_config {
        config.insert(key.clone(), value.pact_configuration.clone());
      }
    }
    let response_config = self.response.plugin_config();
    if !response_config.is_empty() {
      for (key, value) in response_config {
        config.insert(key.clone(), value.pact_configuration.clone());
      }
    }
    config
  }
}

#[cfg(all(test, feature = "plugins"))]
mod plugin_tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_plugin_driver::content::PluginConfiguration;
  use proclaim_it::assert_that;
  use serde_json::{json, Value};

  use crate::builders::InteractionBuilder;

  #[test]
  fn plugin_config_merges_config_from_request_and_response_parts() {
    let mut builder = InteractionBuilder::new("test", "");
    builder.request.plugin_config = hashmap!{
      "plugin1".to_string() => PluginConfiguration {
        interaction_configuration: hashmap!{
          "other".to_string() => json!(100),
          "request".to_string() => json!({
            "descriptorKey": "d58838959e37498cddf51805bedf4dca",
            "message": ".area_calculator.ShapeMessage"
          })
        },
        pact_configuration: Default::default()
      }
    };
    builder.response.plugin_config = hashmap!{
      "plugin1".to_string() => PluginConfiguration {
        interaction_configuration: hashmap!{
          "other".to_string() => json!(200),
          "response".to_string() => json!({
            "descriptorKey": "d58838959e37498cddf51805bedf4dca",
            "message": ".area_calculator.AreaResponse"
          })
        },
        pact_configuration: Default::default()
      }
    };

    let config = builder.plugin_config();
    expect!(config).to(be_equal_to(hashmap!{
      "plugin1".to_string() => hashmap!{
        "other".to_string() => json!(200),
        "request".to_string() => json!({
          "descriptorKey": "d58838959e37498cddf51805bedf4dca",
          "message": ".area_calculator.ShapeMessage"
        }),
        "response".to_string() => json!({
          "descriptorKey": "d58838959e37498cddf51805bedf4dca",
          "message": ".area_calculator.AreaResponse"
        })
      }
    }));
  }

  #[test]
  fn supports_setting_external_references() {
    let interaction = InteractionBuilder::new("test", "")
      .reference("asyncapi", "operationId", "test")
      .reference("openapi", "operationId", "test2")
      .build_v4();

    assert_that! {
      interaction.comments == hashmap! {
        "references".to_string() => json!({
            "asyncapi": {
              "operationId": "test"
            },
            "openapi": {
              "operationId": "test2"
            }
        }),
        "text".to_string() => json!([]),
        "testname".to_string() => Value::Null
      }
    }
  }
}
