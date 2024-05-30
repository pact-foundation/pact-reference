//! Builder for constructing synchronous message interactions

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use bytes::Bytes;
use maplit::hashmap;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::find_content_matcher;
#[cfg(feature = "plugins")] use pact_plugin_driver::content::ContentMatcher;
#[cfg(feature = "plugins")] use pact_plugin_driver::plugin_models::PactPluginManifest;
use serde_json::{json, Map, Value};
use tracing::debug;

use pact_models::content_types::ContentType;
use pact_models::generators::Generators;
use pact_models::json_utils::json_to_string;
use pact_models::path_exp::DocPath;
#[cfg(feature = "plugins")] use pact_models::plugins::PluginData;
use pact_models::prelude::{MatchingRuleCategory, MatchingRules, OptionalBody, ProviderState};
use pact_models::v4::interaction::InteractionMarkup;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;

use crate::builders::message_builder::{InteractionContents, PluginConfiguration};
#[cfg(not(feature = "plugins"))] use crate::builders::message_builder::PactPluginManifest;
use crate::prelude::{JsonPattern, Pattern};
#[cfg(feature = "plugins")] use crate::prelude::PluginInteractionBuilder;

#[derive(Clone, Debug)]
/// Synchronous message interaction builder. Normally created via PactBuilder::sync_message_interaction.
pub struct SyncMessageInteractionBuilder {
  description: String,
  provider_states: Vec<ProviderState>,
  comments: Vec<String>,
  test_name: Option<String>,
  key: Option<String>,
  pending: Option<bool>,
  /// Request contents of the message. This will include the payload as well as any metadata
  pub request_contents: InteractionContents,
  /// Response contents of the message. This will include the payloads as well as any metadata
  pub response_contents: Vec<InteractionContents>,
  #[allow(dead_code)] contents_plugin: Option<PactPluginManifest>,
  #[allow(dead_code)] plugin_config: HashMap<String, PluginConfiguration>
}

impl SyncMessageInteractionBuilder {
  /// Create a new message interaction builder
  pub fn new<D: Into<String>>(description: D) -> SyncMessageInteractionBuilder {
    SyncMessageInteractionBuilder {
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      key: None,
      pending: None,
      request_contents: Default::default(),
      response_contents: vec![],
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

  /// Adds a key/value pair to the message request metadata. The key can be anything that is
  /// convertible into a string, and the value must be conveyable into a JSON value.
  pub fn request_metadata<S: Into<String>, J: Into<Value>>(&mut self, key: S, value: J) -> &mut Self {
    let metadata = self.request_contents.metadata
      .get_or_insert_with(|| hashmap!{});
    metadata.insert(key.into(), value.into());
    self
  }

  /// The interaction we've built (in V4 format).
  pub fn build(&self) -> SynchronousMessage {
    debug!("Building V4 SynchronousMessages interaction: {:?}", self);

    let mut rules = MatchingRules::default();
    rules.add_category("body")
      .add_rules(self.request_contents.rules.as_ref().cloned().unwrap_or_default());
    rules.add_category("metadata")
      .add_rules(self.request_contents.metadata_rules.as_ref().cloned().unwrap_or_default());

    #[allow(unused_mut, unused_assignments)] let mut plugin_config = hashmap!{};
    #[cfg(feature = "plugins")]
    {
      plugin_config = self.contents_plugin.as_ref().map(|plugin| {
        hashmap! {
          plugin.name.clone() => self.request_contents.plugin_config.interaction_configuration.clone()
        }
      }).unwrap_or_default();
    }

    #[allow(unused_mut, unused_assignments)] let mut interaction_markup = InteractionMarkup::default();
    #[cfg(feature = "plugins")]
    {
      interaction_markup = InteractionMarkup {
        markup: self.interaction_markup(),
        markup_type: self.request_contents.interaction_markup_type.clone()
      };
    }

    SynchronousMessage {
      id: None,
      key: self.key.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: MessageContents {
        contents: self.request_contents.body.clone(),
        metadata: self.request_contents.metadata.as_ref().cloned().unwrap_or_default(),
        matching_rules: rules,
        generators: self.request_contents.generators.as_ref().cloned().unwrap_or_default()
      },
      response: self.response_contents.iter().map(|contents| {
        let mut rules = MatchingRules::default();
        rules.add_category("body")
          .add_rules(contents.rules.as_ref().cloned().unwrap_or_default());
        rules.add_category("metadata")
          .add_rules(contents.metadata_rules.as_ref().cloned().unwrap_or_default());
        MessageContents {
          contents: contents.body.clone(),
          metadata: contents.metadata.as_ref().cloned().unwrap_or_default(),
          matching_rules: rules,
          generators: self.request_contents.generators.as_ref().cloned().unwrap_or_default()
        }
      }).collect(),
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

  #[cfg(feature = "plugins")]
  fn interaction_markup(&self) -> String {
    let mut markup = self.request_contents.interaction_markup.clone();

    for interaction in &self.response_contents {
      if !interaction.interaction_markup.is_empty() {
        markup.push_str(interaction.interaction_markup.as_str());
      }
    }

    markup
  }

  /// Configure the interaction contents from a map of values
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
            self.setup_core_matcher(Some(ct.clone()), &contents_hashmap, Some(content_matcher));
          } else {
            match content_matcher.configure_interaction(&ct, contents_hashmap).await {
              Ok((contents, plugin_config)) => {
                if let Some(interaction) = contents.iter().find(|i| i.part_name == "request") {
                  self.request_contents = InteractionContents::from(&interaction);
                }

                for interaction in contents.iter().filter(|i| i.part_name == "response") {
                  self.response_contents.push(InteractionContents::from(&interaction));
                }

                self.contents_plugin = content_matcher.plugin();

                if let Some(plugin_config) = plugin_config {
                  let plugin_name = content_matcher.plugin_name();
                  self.add_plugin_config(PluginConfiguration::from(&plugin_config), plugin_name)
                }
              }
              Err(err) => panic!("Failed to call out to plugin - {}", err)
            }
          }
        } else {
          debug!("No content matcher found, will use the internal implementation");
          self.setup_core_matcher(Some(ct.clone()), &contents_hashmap, None);
        }
      }

      #[cfg(not(feature = "plugins"))]
      {
        self.setup_core_matcher(Some(ct.clone()), &contents_hashmap);
      }
    } else {
      debug!("No content type provided, will use the internal implementation");

      #[cfg(feature = "plugins")]
      {
        self.setup_core_matcher(None, &contents_hashmap, None);
      }

      #[cfg(not(feature = "plugins"))]
      {
        self.setup_core_matcher(None, &contents_hashmap);
      }
    }

    self
  }

  /// Configure the interaction contents from a plugin builder
  #[cfg(feature = "plugins")]
  pub async fn contents_for_plugin<B: PluginInteractionBuilder>(&mut self, builder: B) -> &mut Self {
    self.contents_from(builder.build()).await
  }

  #[cfg(feature = "plugins")]
  fn add_plugin_config(&mut self, plugin_config: PluginConfiguration, plugin_name: String) {
    if self.plugin_config.contains_key(&*plugin_name) {
      let entry = self.plugin_config.get_mut(&*plugin_name).unwrap();
      for (k, v) in plugin_config.pact_configuration {
        entry.pact_configuration.insert(k.clone(), v.clone());
      }
    } else {
      self.plugin_config.insert(plugin_name.to_string(), plugin_config.clone());
    }
  }

  #[cfg(feature = "plugins")]
  fn setup_core_matcher(
    &mut self,
    content_type: Option<ContentType>,
    config: &HashMap<String, Value>,
    content_matcher: Option<ContentMatcher>
  ) {
    if let Some(request) = config.get("request") {
      let mut body = OptionalBody::from(request);
      if let Some(ct) = content_type.as_ref() {
        body.set_content_type(ct);
      }

      self.request_contents = InteractionContents {
        body, .. InteractionContents::default()
      };
    }
    if let Some(responses) = config.get("response") {
      match responses {
        Value::Array(responses) => {
          for response in responses {
            let mut body = OptionalBody::from(response);
            if let Some(ct) = content_type.as_ref() {
              body.set_content_type(ct);
            }

            self.response_contents.push(InteractionContents {
              body, .. InteractionContents::default()
            });
          }
        }
        _ => {
          let mut body = OptionalBody::from(responses);
          if let Some(ct) = content_type.as_ref() {
            body.set_content_type(ct);
          }

          self.response_contents.push(InteractionContents {
            body, .. InteractionContents::default()
          });
        }
      }
    }

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

  #[cfg(not(feature = "plugins"))]
  fn setup_core_matcher(
    &mut self,
    content_type: Option<ContentType>,
    config: &HashMap<String, Value>
  ) {
    if let Some(request) = config.get("request") {
      let mut body = OptionalBody::from(request);
      if let Some(ct) = content_type.as_ref() {
        body.set_content_type(ct);
      }

      self.request_contents = InteractionContents {
        body, .. InteractionContents::default()
      };
    }
    if let Some(responses) = config.get("response") {
      match responses {
        Value::Array(responses) => {
          for response in responses {
            let mut body = OptionalBody::from(response);
            if let Some(ct) = content_type.as_ref() {
              body.set_content_type(ct);
            }

            self.response_contents.push(InteractionContents {
              body, .. InteractionContents::default()
            });
          }
        }
        _ => {
          let mut body = OptionalBody::from(responses);
          if let Some(ct) = content_type.as_ref() {
            body.set_content_type(ct);
          }

          self.response_contents.push(InteractionContents {
            body, .. InteractionContents::default()
          });
        }
      }
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
  /// use pact_consumer::builders::SyncMessageInteractionBuilder;
  ///
  /// SyncMessageInteractionBuilder::new("hello message").request_json_body(json_pattern!({
  ///     "message": like!("Hello"),
  /// }));
  /// ```
  pub fn request_json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
    let body = body.into();
    {
      let message_body = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
      let mut rules = MatchingRuleCategory::empty("content");
      body.extract_matching_rules(DocPath::root(), &mut rules);
      self.request_contents.body = message_body;
      if rules.is_not_empty() {
        match &mut self.request_contents.rules {
          None => self.request_contents.rules = Some(rules.clone()),
          Some(mr) => mr.add_rules(rules.clone())
        }
      }
    }
    self
  }

  /// Specify the message request payload and content type
  pub fn request_body<B:  Into<Bytes>>(&mut self, body: B, content_type: Option<String>) -> &mut Self {
    let message_body = OptionalBody::Present(
      body.into(),
      content_type.as_ref().map(|ct| ct.into()),
      None
    );
    self.request_contents.body = message_body;
    let metadata = self.request_contents.metadata
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

  ///  Sets the request message contents
  pub fn request_contents(&mut self, contents: &MessageContents) -> &mut Self {
    self.request_contents.body = contents.contents.clone();
    let metadata = self.request_contents.metadata.get_or_insert_with(|| hashmap! {});
    metadata.extend(contents.metadata.iter().map(|(k, v)| (k.clone(), v.clone())));

    if let Some(category) = contents.matching_rules.rules_for_category("body") {
      let rules = self.request_contents.rules.get_or_insert_with(|| MatchingRuleCategory::empty("body"));
      rules.add_rules(category);
    }

    if let Some(category) = contents.matching_rules.rules_for_category("metadata") {
      let rules = self.request_contents.metadata_rules.get_or_insert_with(|| MatchingRuleCategory::empty("metadata"));
      rules.add_rules(category);
    }

    let generators = self.request_contents.generators.get_or_insert_with(|| Generators::default());
    generators.add_generators(contents.generators.clone());
    self
  }

  /// Specify the body as `JsonPattern`, possibly including special matching
  /// rules. You can call this method multiple times, each will add a new response message to the
  /// interaction.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::*;
  /// use pact_consumer::builders::SyncMessageInteractionBuilder;
  ///
  /// SyncMessageInteractionBuilder::new("hello message").response_json_body(json_pattern!({
  ///     "message": like!("Hello"),
  /// }));
  /// ```
  pub fn response_json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
    let body = body.into();
    {
      let message_body = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
      let mut rules = MatchingRuleCategory::empty("content");
      body.extract_matching_rules(DocPath::root(), &mut rules);
      self.response_contents.push(InteractionContents {
        part_name: "response".to_string(),
        body: message_body.clone(),
        rules: if rules.is_not_empty() { Some(rules) } else { None },
        .. InteractionContents::default()
      });
    }
    self
  }

  /// Specify the message response payload and content type. You can call this method multiple
  /// times, each will add a new response message to the interaction.
  pub fn response_body<B:  Into<Bytes>>(&mut self, body: B, content_type: Option<String>) -> &mut Self {
    let message_body = OptionalBody::Present(
      body.into(),
      content_type.as_ref().map(|ct| ct.into()),
      None
    );
    let mut metadata = hashmap!{};
    if let Some(content_type) = content_type {
      metadata.insert(" contentType".to_string(), Value::String(content_type.clone()));
    }
    let response = InteractionContents {
      part_name: "response".to_string(),
      body: message_body,
      rules: None,
      generators: None,
      metadata: Some(metadata),
      metadata_rules: None,
      plugin_config: Default::default(),
      interaction_markup: "".to_string(),
      interaction_markup_type: "".to_string(),
    };

        self.response_contents.push(response);
    self
  }

  ///  Sets the response message contents. You can call this method multiple
  /// times, each will add a new response message to the interaction.
  pub fn response_contents(&mut self, contents: &MessageContents) -> &mut Self {
    let response = InteractionContents {
      part_name: "response".to_string(),
      body: contents.contents.clone(),
      rules: contents.matching_rules.rules_for_category("body"),
      generators: Some(contents.generators.clone()),
      metadata: Some(contents.metadata.clone()),
      metadata_rules: contents.matching_rules.rules_for_category("metadata"),
      plugin_config: Default::default(),
      interaction_markup: "".to_string(),
      interaction_markup_type: "".to_string(),
    };

    self.response_contents.push(response);
    self
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use pact_models::matchingrules;
  use pact_models::matchingrules::{Category, MatchingRule, MatchingRules, RuleLogic};
  use pact_models::path_exp::DocPath;
  use pact_models::v4::message_parts::MessageContents;

  use crate::builders::SyncMessageInteractionBuilder;

  #[test]
  fn supports_setting_metadata_values() {
    let message = SyncMessageInteractionBuilder::new("test")
      .request_metadata("a", "a")
      .request_metadata("b", json!("b"))
      .request_metadata("c", vec![1, 2, 3])
      .build();
    expect!(message.request.metadata).to(be_equal_to(hashmap! {
      "a".to_string() => json!("a"),
      "b".to_string() => json!("b"),
      "c".to_string() => json!([1, 2, 3])
    }));
  }

  #[test]
  fn supports_matching_rules_on_metadata_values() {
    let message = SyncMessageInteractionBuilder::new("test")
      .request_contents(&MessageContents {
        contents: Default::default(),
        metadata: Default::default(),
        matching_rules: matchingrules! {
          "metadata" => { "R" => [  MatchingRule::Regex("1".to_string())  ] }
        },
        generators: Default::default()
      })
      .response_contents(&MessageContents {
        contents: Default::default(),
        metadata: Default::default(),
        matching_rules: matchingrules! {
          "metadata" => { "R" => [  MatchingRule::Regex("2".to_string())  ] }
        },
        generators: Default::default()
      })
      .build();
    expect!(message.request.matching_rules).to(be_equal_to(matchingrules! {
      "metadata" => { "R" => [  MatchingRule::Regex("1".to_string())  ] }
    }));
    expect!(message.response.first().cloned().unwrap().matching_rules).to(be_equal_to(matchingrules! {
      "metadata" => { "R" => [  MatchingRule::Regex("2".to_string())  ] }
    }));
  }

  fn meta_matching_rules(path_str: &'static str) -> MatchingRules {
    let mut rules: MatchingRules = MatchingRules::default();
    rules.add_category(Category::BODY)
      .add_rule(DocPath::new_unwrap(path_str), MatchingRule::Type, RuleLogic::And);
    rules
  }

  fn message_contents(matching_rules: &MatchingRules) -> MessageContents {
    MessageContents {
      contents: pact_models::bodies::OptionalBody::Missing,
      metadata: hashmap! {
        "a".to_string() => json!("a")
      },
      matching_rules: matching_rules.clone(),
      generators: Default::default()
    }
  }

  #[test]
  fn supports_request_metadata_rules() {
    let rules = meta_matching_rules("$.a");
    let message_contents: MessageContents = message_contents(&rules);
    let message = SyncMessageInteractionBuilder::new("test")
      .request_contents(&message_contents)
      .build();
    expect!(message.request.matching_rules).to(be_equal_to(rules));
  }

  #[test]
  fn supports_response_metadata_rules() {
    let rules = meta_matching_rules("$.a");
    let message_contents = message_contents(&rules);
    let message = SyncMessageInteractionBuilder::new("test")
      .response_contents(&message_contents)
      .build();
    expect!(message.response.len()).to(be_equal_to(1));
    expect!(message.response[0].clone().matching_rules).to(be_equal_to(rules));
  }

  #[test]
  fn supports_multiple_response_metadata_rules() {
    let mut rules = MatchingRules::default();
    let cat = rules.add_category(Category::METADATA);
    cat.add_rule(DocPath::new_unwrap("$.a"), MatchingRule::Type, RuleLogic::And);
    cat.add_rule(DocPath::new_unwrap("$.b"), MatchingRule::Type, RuleLogic::And);
    let message_contents = message_contents(&rules);
    let message = SyncMessageInteractionBuilder::new("test")
      .response_contents(&message_contents)
      .build();

    expect!(message.response.len()).to(be_equal_to(1));
    expect!(message.response[0].clone().matching_rules).to(be_equal_to(rules));
  }

  #[test]
  fn supports_mutliple_response_contents_with_metadata_rules() {
    let rules1 = meta_matching_rules("$.a");
    let contents1 = message_contents(&rules1);
    let rules2 = meta_matching_rules("$.b");
    let contents2 = message_contents(&rules2);
    let message = SyncMessageInteractionBuilder::new("test")
      .response_contents(&contents1)
      .response_contents(&contents2)
      .build();

    // expect message.response to have only two elements
    expect!(message.response.len()).to(be_equal_to(2));
    expect!(message.response[0].clone().matching_rules).to(be_equal_to(rules1));
    expect!(message.response[1].clone().matching_rules).to(be_equal_to(rules2));
  }

  #[test]
  fn supports_rules_for_body_and_metadata_in_request_and_response() {
    let mut rules = MatchingRules::default();
    let cat_body = rules.add_category(Category::BODY);
    cat_body.add_rule(DocPath::new_unwrap("$.a"), MatchingRule::Type, RuleLogic::And);
    let cat_meta = rules.add_category(Category::METADATA);
    cat_meta.add_rule(DocPath::new_unwrap("$.b"), MatchingRule::Type, RuleLogic::And);
    let message_contents = message_contents(&rules);
    let message = SyncMessageInteractionBuilder::new("test")
      .request_contents(&message_contents)
      .response_contents(&message_contents)
      .build();

    expect!(message.request.matching_rules).to(be_equal_to(rules.clone()));
    expect!(message.response.len()).to(be_equal_to(1));
    expect!(message.response[0].clone().matching_rules).to(be_equal_to(rules.clone()));
  }
}
