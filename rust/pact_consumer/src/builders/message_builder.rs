//! Builder for constructing Asynchronous message interactions

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use bytes::Bytes;
use maplit::hashmap;
use pact_models::content_types::ContentType;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::MatchingRuleCategory;
use pact_models::generators::Generators;
use pact_models::message::Message;
use pact_models::path_exp::DocPath;
#[cfg(feature = "plugins")] use pact_models::plugins::PluginData;
use pact_models::prelude::{MatchingRules, OptionalBody, ProviderState};
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::interaction::InteractionMarkup;
use pact_models::v4::message_parts::MessageContents;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::find_content_matcher;
#[cfg(feature = "plugins")] use pact_plugin_driver::content::ContentMatcher;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::PactPluginManifest;
use serde_json::{json, Map, Value};
use tracing::debug;

use crate::patterns::JsonPattern;
use crate::prelude::Pattern;
#[cfg(feature = "plugins")] use crate::prelude::PluginInteractionBuilder;

#[cfg(not(feature = "plugins"))]
#[derive(Clone, Debug, Default)]
/// Placeholder struct when plugins are disabled
pub struct PactPluginManifest {}

#[derive(Clone, Debug, Default)]
/// Struct to store any config received from plugins
pub struct PluginConfiguration {
  /// Data to persist on the interaction
  pub interaction_configuration: HashMap<String, Value>,
  /// Data to persist in the Pact metadata
  pub pact_configuration: HashMap<String, Value>
}

impl PluginConfiguration {
  /// Create a new struct from a plugin specific one
  #[cfg(feature = "plugins")]
  pub fn from(config: &pact_plugin_driver::content::PluginConfiguration) -> Self {
    PluginConfiguration {
      interaction_configuration: config.interaction_configuration.clone(),
      pact_configuration: config.pact_configuration.clone()
    }
  }
}

#[derive(Clone, Debug, Default)]
/// Struct that stores the interaction contents for the message
pub struct InteractionContents {
  /// Description of what part this interaction belongs to (in the case of there being more than
  /// one, for instance, request/response messages). For async messages, this will not be used.
  #[allow(dead_code)] pub part_name: String,

  /// Body/Contents of the interaction
  pub body: OptionalBody,

  /// Matching rules to apply
  pub rules: Option<MatchingRuleCategory>,

  /// Generators to apply
  pub generators: Option<Generators>,

  /// Message metadata
  pub metadata: Option<HashMap<String, Value>>,

  /// Matching rules to apply to message metadata
  pub metadata_rules: Option<MatchingRuleCategory>,

  /// Plugin configuration data to apply to the interaction
  pub plugin_config: PluginConfiguration,

  /// Markup for the interaction to display in any UI
  pub interaction_markup: String,

  /// The type of the markup (CommonMark or HTML)
  pub interaction_markup_type: String
}

impl InteractionContents {
  #[cfg(feature = "plugins")]
  /// Create a new struct from a plugin specific one
  pub fn from(contents: &pact_plugin_driver::content::InteractionContents) -> Self {
    InteractionContents {
      part_name: contents.part_name.clone(),
      body: contents.body.clone(),
      rules: contents.rules.clone(),
      generators: contents.generators.clone(),
      metadata: contents.metadata.clone(),
      metadata_rules: contents.metadata_rules.clone(),
      plugin_config: PluginConfiguration::from(&contents.plugin_config),
      interaction_markup: contents.interaction_markup.clone(),
      interaction_markup_type: contents.interaction_markup_type.clone()
    }
  }
}

#[derive(Clone, Debug)]
/// Asynchronous message interaction builder. Normally created via PactBuilder::message_interaction.
pub struct MessageInteractionBuilder {
  description: String,
  provider_states: Vec<ProviderState>,
  comments: Vec<String>,
  test_name: Option<String>,
  key: Option<String>,
  pending: Option<bool>,
  /// Contents of the message. This will include the payload as well as any metadata
  pub message_contents: InteractionContents,
  #[allow(dead_code)] contents_plugin: Option<PactPluginManifest>,
  #[allow(dead_code)] plugin_config: HashMap<String, PluginConfiguration>
}

impl MessageInteractionBuilder {
  /// Create a new message interaction builder, Description is the interaction description
  /// and interaction_type is the type of message (leave empty for the default type).
  pub fn new<D: Into<String>>(description: D) -> MessageInteractionBuilder {
    MessageInteractionBuilder {
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      key: None,
      pending: None,
      message_contents: Default::default(),
      contents_plugin: None,
      plugin_config: Default::default()
    }
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

  /// Adds a key/value pair to the message metadata. The key can be anything that is convertible
  /// into a string, and the value must be conveyable into a JSON value.
  pub fn metadata<S: Into<String>, J: Into<Value>>(&mut self, key: S, value: J) -> &mut Self {
    let metadata = self.message_contents.metadata
      .get_or_insert_with(|| hashmap!{});
    metadata.insert(key.into(), value.into());
    self
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

  /// The interaction we've built (in V4 format).
  pub fn build(&self) -> AsynchronousMessage {
    debug!("Building V4 AsynchronousMessage interaction: {:?}", self);

    let mut rules = MatchingRules::default();
    rules.add_category("body")
      .add_rules(self.message_contents.rules.as_ref().cloned().unwrap_or_default());

    #[allow(unused_mut, unused_assignments)] let mut plugin_config = hashmap!{};
    #[cfg(feature = "plugins")]
    {
      plugin_config = self.contents_plugin.as_ref().map(|plugin| {
        hashmap! {
          plugin.name.clone() => self.message_contents.plugin_config.interaction_configuration.clone()
        }
      }).unwrap_or_default();
    }

    #[allow(unused_mut, unused_assignments)] let mut interaction_markup = InteractionMarkup::default();
    #[cfg(feature = "plugins")]
    {
      interaction_markup = InteractionMarkup {
        markup: self.message_contents.interaction_markup.clone(),
        markup_type: self.message_contents.interaction_markup_type.clone()
      };
    }

    AsynchronousMessage {
      id: None,
      key: self.key.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: MessageContents {
        contents: self.message_contents.body.clone(),
        metadata: self.message_contents.metadata.as_ref().cloned().unwrap_or_default(),
        matching_rules: rules,
        generators: self.message_contents.generators.as_ref().cloned().unwrap_or_default()
      },
      comments: hashmap!{
        "text".to_string() => json!(self.comments),
        "testname".to_string() => json!(self.test_name)
      },
      pending: self.pending.unwrap_or(false),
      plugin_config,
      interaction_markup,
      transport: None
    }
  }

  /// The interaction we've built (in V3 format).
  pub fn build_v3(&self) -> Message {
    debug!("Building V3 Message interaction: {:?}", self);

    let mut rules = MatchingRules::default();
    rules.add_category("body")
      .add_rules(self.message_contents.rules.as_ref().cloned().unwrap_or_default());

        Message {
      id: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: self.message_contents.body.clone(),
          metadata: self.message_contents.metadata.as_ref().cloned().unwrap_or_default(),
          matching_rules: rules,
          generators: self.message_contents.generators.as_ref().cloned().unwrap_or_default()
        }
  }

  /// Configure the interaction contents from a map
  pub async fn contents_from(&mut self, contents: Value) -> &mut Self {
    debug!("Configuring interaction from {:?}", contents);

    let contents_map = contents.as_object().cloned().unwrap_or(Map::default());
    let contents_hashmap: HashMap<String, Value> = contents_map.iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    if let Some(content_type) = contents_map.get("pact:content-type") {
      let ct = ContentType::parse(json_to_string(content_type).as_str()).unwrap();

      #[cfg(feature = "plugins")]
      {
        if let Some(content_matcher) = find_content_matcher(&ct) {
          debug!("Found a matcher for '{}': {:?}", ct, content_matcher);
          if content_matcher.is_core() {
            debug!("Content matcher is a core matcher, will use the internal implementation");
            self.setup_core_matcher(&ct, &contents_hashmap, Some(content_matcher));
          } else {
            debug!("Plugin matcher, will get the plugin to provide the interaction contents");
            match content_matcher.configure_interaction(&ct, contents_hashmap).await {
              Ok((contents, plugin_config)) => {
                if let Some(contents) = contents.first() {
                  self.message_contents = InteractionContents::from(contents);
                  if !contents.plugin_config.is_empty() {
                    self.plugin_config.insert(content_matcher.plugin_name(), PluginConfiguration::from(&contents.plugin_config));
                  }
                }
                self.contents_plugin = content_matcher.plugin();

                if let Some(plugin_config) = plugin_config {
                  let plugin_name = content_matcher.plugin_name();
                  if self.plugin_config.contains_key(&*plugin_name) {
                    let entry = self.plugin_config.get_mut(&*plugin_name).unwrap();
                    for (k, v) in plugin_config.pact_configuration {
                      entry.pact_configuration.insert(k.clone(), v.clone());
                    }
                  } else {
                    self.plugin_config.insert(plugin_name.to_string(), PluginConfiguration::from(&plugin_config));
                  }
                }
              }
              Err(err) => panic!("Failed to call out to plugin - {}", err)
            }
          }
        } else {
          debug!("No content matcher found, will use the internal implementation");
          self.setup_core_matcher(&ct, &contents_hashmap, None);
        }
      }

      #[cfg(not(feature = "plugins"))]
      {
        self.message_contents = InteractionContents {
          body: if let Some(contents) = contents_hashmap.get("contents") {
            OptionalBody::Present(
              Bytes::from(contents.to_string()),
              Some(ct.clone()),
              None
            )
          } else {
            OptionalBody::Missing
          },
          .. InteractionContents::default()
        };
      }
    } else {
      self.message_contents = InteractionContents {
        body : OptionalBody::from(Value::Object(contents_map.clone())),
        .. InteractionContents::default()
      };
    }

    self
  }

  /// Configure the interaction contents from a plugin builder
  #[cfg(feature = "plugins")]
  pub async fn contents_for_plugin<B: PluginInteractionBuilder>(&mut self, builder: B) -> &mut Self {
    self.contents_from(builder.build()).await
  }

  #[cfg(feature = "plugins")]
  fn setup_core_matcher(
    &mut self,
    content_type: &ContentType,
    config: &HashMap<String, Value>,
    content_matcher: Option<ContentMatcher>
  ) {
    self.message_contents = InteractionContents {
      body: if let Some(contents) = config.get("contents") {
        OptionalBody::Present(
          Bytes::from(contents.to_string()),
          Some(content_type.clone()),
          None
        )
      } else {
        OptionalBody::Missing
      },
      .. InteractionContents::default()
    };

    if let Some(_content_matcher) = content_matcher {
      // TODO: get the content matcher to apply the matching rules and generators
      //     val (body, rules, generators, _, _) = contentMatcher.setupBodyFromConfig(bodyConfig)
      //     val matchingRules = MatchingRulesImpl()
      //     if (rules != null) {
      //       matchingRules.addCategory(rules)
      //     }
      //     MessageContents(body, mapOf(), matchingRules, generators ?: Generators())
    }
  }

  /// Any global plugin config required to add to the Pact
  #[cfg(feature = "plugins")]
  pub fn plugin_config(&self) -> Option<PluginData> {
    self.contents_plugin.as_ref().map(|plugin| {
      let config = if let Some(config) = self.plugin_config.get(plugin.name.as_str()) {
        config.pact_configuration.clone()
      } else {
        hashmap!{}
      };
      PluginData {
        name: plugin.name.clone(),
        version: plugin.version.clone(),
        configuration: config
      }
    })
  }

  /// Specify the body as `JsonPattern`, possibly including special matching
  /// rules.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::*;
  /// use pact_consumer::builders::MessageInteractionBuilder;
  ///
  /// MessageInteractionBuilder::new("hello message").json_body(json_pattern!({
  ///     "message": like!("Hello"),
  /// }));
  /// ```
  pub fn json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
    let body = body.into();
    {
      let message_body = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
      let mut rules = MatchingRuleCategory::empty("content");
      body.extract_matching_rules(DocPath::root(), &mut rules);
      self.message_contents.body = message_body;
      if rules.is_not_empty() {
        match &mut self.message_contents.rules {
          None => self.message_contents.rules = Some(rules.clone()),
          Some(mr) => mr.add_rules(rules.clone())
        }
      }
    }
    self
  }

  /// Specify the message payload and content type
  pub fn body<B:  Into<Bytes>>(&mut self, body: B, content_type: Option<String>) -> &mut Self {
    let message_body = OptionalBody::Present(
      body.into(),
      content_type.as_ref().map(|ct| ct.into()),
      None
    );
    self.message_contents.body = message_body;
    let metadata = self.message_contents.metadata
      .get_or_insert_with(|| hashmap!{});
    if let Some(content_type) = content_type {
      match metadata.entry("contentType".to_string()) {
        Entry::Occupied(_) => {}
        Entry::Vacant(entry) => {
          entry.insert(Value::String(content_type.clone()));
        }
      }
    }
    self
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::builders::MessageInteractionBuilder;

  #[test]
  fn supports_setting_metadata_values() {
    let message = MessageInteractionBuilder::new("test")
      .metadata("a", "a")
      .metadata("b", json!("b"))
      .metadata("c", vec![1, 2, 3])
      .build();
    expect!(message.contents.metadata).to(be_equal_to(hashmap! {
      "a".to_string() => json!("a"),
      "b".to_string() => json!("b"),
      "c".to_string() => json!([1, 2, 3])
    }));
  }
}
