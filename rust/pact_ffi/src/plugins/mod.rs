//! The `plugins` module provides exported functions using C bindings for using plugins with
//! Pact tests.

use std::time::Duration;

use anyhow::anyhow;
use bytes::Bytes;
use itertools::Itertools;
use libc::{c_char, c_uint};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::http_parts::HttpPart;
use pact_models::json_utils::body_from_json;
use pact_models::pact::Pact;
use pact_models::prelude::{Generators, MatchingRules};
use pact_models::v4::interaction::{InteractionMarkup, V4Interaction};
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::V4InteractionType;
use pact_plugin_driver::catalogue_manager::find_content_matcher;
use pact_plugin_driver::content::{InteractionContents, PluginConfiguration};
use pact_plugin_driver::plugin_manager::{drop_plugin_access, load_plugin, lookup_plugin};
use pact_plugin_driver::plugin_models::{PluginDependency, PluginDependencyType};
use serde_json::Value;
use tokio::runtime::Builder;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::{ffi_fn, safe_str, RUNTIME};
use crate::error::{catch_panic, set_error_msg};
use crate::mock_server::handles::{InteractionHandle, InteractionPart, PactHandle};
use crate::string::if_null;

ffi_fn! {
  /// Add a plugin to be used by the test. The plugin needs to be installed correctly for this
  /// function to work.
  ///
  /// * `plugin_name` is the name of the plugin to load.
  /// * `plugin_version` is the version of the plugin to load. It is optional, and can be NULL.
  /// * `completion_delay` is an arbitrary delay specified in milliseconds to add before the 
  ///    function returns to allow asynchronous tasks to complete.
  ///
  /// Returns zero on success, and a positive integer value on failure.
  ///
  /// Note that plugins run as separate processes, so will need to be cleaned up afterwards by
  /// calling `pactffi_cleanup_plugins` otherwise you will have plugin processes left running.
  ///
  /// # Safety
  ///
  /// `plugin_name` must be a valid pointer to a NULL terminated string. `plugin_version` may be null,
  /// and if not NULL must also be a valid pointer to a NULL terminated string. Invalid
  /// pointers will result in undefined behaviour.
  ///
  /// # Errors
  ///
  /// * `1` - A general panic was caught.
  /// * `2` - Failed to load the plugin.
  /// * `3` - Pact Handle is not valid.
  ///
  /// When an error errors, LAST_ERROR will contain the error message.
  fn pactffi_using_plugin_with_delay(pact: PactHandle, plugin_name: *const c_char, plugin_version: *const c_char, completion_delay: u64) -> c_uint {
    let plugin_name = safe_str!(plugin_name);
    let plugin_version = if_null(plugin_version, "");

    let dependency = PluginDependency {
      name: plugin_name.to_string(),
      version: if plugin_version.is_empty() { None } else { Some(plugin_version) },
      dependency_type: Default::default()
    };
    let result = lookup_plugin(&dependency)
      .and_then(|mut plugin| {
        plugin.update_access();
        Some(plugin)
      })
      .ok_or(())
      .or_else(|_| {
        pact_mock_server::configure_core_catalogue();
        pact_matching::matchingrules::configure_core_catalogue();

        let result = RUNTIME.block_on(async {
          let result = load_plugin(&dependency).await;

          // Add a small delay to let asynchronous tasks to complete
          sleep(Duration::from_millis(completion_delay)).await;

          result
        });

        result
      });

    match result {
      Ok(plugin) => pact.with_pact(&|_, inner| {
        inner.pact.add_plugin(plugin.manifest.name.as_str(), plugin.manifest.version.as_str(), None)
          .expect("Could not add plugin to pact");
        0
      }).unwrap_or(3),
      Err(err) => {
        error!("Could not load plugin - {}", err);
        set_error_msg(format!("Could not load plugin - {}", err));
        2
      }
    }
  } {
    1
  }
}

/// Add a plugin to be used by the test. The plugin needs to be installed correctly for this
/// function to work.
///
/// * `plugin_name` is the name of the plugin to load.
/// * `plugin_version` is the version of the plugin to load. It is optional, and can be NULL.
///
/// Returns zero on success, and a positive integer value on failure.
///
/// Note that plugins run as separate processes, so will need to be cleaned up afterwards by
/// calling `pactffi_cleanup_plugins` otherwise you will have plugin processes left running.
///
/// # Safety
///
/// `plugin_name` must be a valid pointer to a NULL terminated string. `plugin_version` may be null,
/// and if not NULL must also be a valid pointer to a NULL terminated string. Invalid
/// pointers will result in undefined behaviour.
///
/// # Errors
///
/// * `1` - A general panic was caught.
/// * `2` - Failed to load the plugin.
/// * `3` - Pact Handle is not valid.
///
/// When an error errors, LAST_ERROR will contain the error message.
#[no_mangle]
pub extern "C" fn pactffi_using_plugin(pact: PactHandle, plugin_name: *const c_char, plugin_version: *const c_char) -> c_uint {
  let result = pactffi_using_plugin_with_delay(pact, plugin_name, plugin_version, 500);
  result
}

ffi_fn! {
  /// Decrement the access count on any plugins that are loaded for the Pact. This will shutdown
  /// any plugins that are no longer required (access count is zero).
  fn pactffi_cleanup_plugins(pact: PactHandle) {
    pact.with_pact(&|_, inner| {
      // decrement access to any plugin loaded for the Pact
      for plugin in inner.pact.plugin_data().iter().map(|plugin| {
        PluginDependency {
          name: plugin.name.clone(),
          version: Some(plugin.version.clone()),
          dependency_type: PluginDependencyType::Plugin
        }
      }).unique() {
        drop_plugin_access(&plugin);
      }
    });
  }
}

/// Setup the interaction part using a plugin. The contents is a JSON string that will be passed on to
/// the plugin to configure the interaction part. Refer to the plugin documentation on the format
/// of the JSON contents.
///
/// Returns zero on success, and a positive integer value on failure.
///
/// * `interaction` - Handle to the interaction to configure.
/// * `part` - The part of the interaction to configure (request or response). It is ignored for messages.
/// * `content_type` - NULL terminated C string of the content type of the part.
/// * `contents` - NULL terminated C string of the JSON contents that gets passed to the plugin.
///
/// # Safety
///
/// `content_type` and `contents` must be a valid pointers to NULL terminated strings. Invalid
/// pointers will result in undefined behaviour.
///
/// # Errors
///
/// * `1` - A general panic was caught.
/// * `2` - The mock server has already been started.
/// * `3` - The interaction handle is invalid.
/// * `4` - The content type is not valid.
/// * `5` - The contents JSON is not valid JSON.
/// * `6` - The plugin returned an error.
///
/// When an error errors, LAST_ERROR will contain the error message.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn pactffi_interaction_contents(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  contents: *const c_char
) -> c_uint {
  catch_panic(|| {
    let content_type_str = safe_str!(content_type);
    let content_type = match ContentType::parse(content_type_str) {
      Ok(ct) => ct,
      Err(err) => {
        error!("'{}' is not a valid content type - {}", content_type_str, err);
        set_error_msg(format!("'{}' is not a valid content type - {}", content_type_str, err));
        return Ok(4);
      }
    };

    let contents_str = safe_str!(contents);
    let contents = match serde_json::from_str(contents_str) {
      Ok(value) => value,
      Err(err) => {
        error!("Contents is not a valid JSON - {}", err);
        error!("contents='{}'", contents_str);
        set_error_msg(format!("Contents is not a valid JSON - {}", err));
        return Ok(5);
      }
    };

    let result = interaction.with_interaction(&|_, started, inner| {
      if !started {
        match inner.v4_type() {
          V4InteractionType::Synchronous_HTTP => setup_contents(inner, part, &content_type, &contents, &|interaction, contents, plugin_name, _| {
            let part = get_part(interaction, part);
            if let Some(contents) = contents.first() {
              *part.body_mut() = contents.body.clone();
              if !part.has_header("content-type") {
                part.add_header("content-type", vec![content_type.to_string().as_str()]);
              }
              if let Some(rules) = &contents.rules {
                part.matching_rules_mut().add_rules("body", rules.clone());
              }
              if let Some(generators) = &contents.generators {
                part.generators_mut().add_generators(generators.clone());
              }
              if !contents.plugin_config.is_empty() {
                interaction.plugin_config_mut().insert(plugin_name, contents.plugin_config.interaction_configuration.clone());
              }
              *interaction.interaction_markup_mut() = InteractionMarkup {
                markup: contents.interaction_markup.clone(),
                markup_type: contents.interaction_markup_type.clone()
              };
            }
          }),
          V4InteractionType::Asynchronous_Messages => setup_contents(inner, part, &content_type, &contents, &|interaction, contents, plugin_name, _| {
            let message = interaction.as_v4_async_message_mut().unwrap();
            if let Some(contents) = contents.first() {
              message.contents.contents = contents.body.clone();
              let metadata = contents.metadata.as_ref()
                .map(|md| md.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();
              message.contents.metadata = metadata;
              if let Some(rules) = &contents.rules {
                message.contents.matching_rules.add_rules("body", rules.clone());
              }
              if let Some(generators) = &contents.generators {
                message.contents.generators.add_generators(generators.clone());
              }
              if !contents.plugin_config.is_empty() {
                message.plugin_config.insert(plugin_name, contents.plugin_config.interaction_configuration.clone());
              }
              message.interaction_markup = InteractionMarkup {
                markup: contents.interaction_markup.clone(),
                markup_type: contents.interaction_markup_type.clone()
              };
            }
          }),
          V4InteractionType::Synchronous_Messages => setup_contents(inner, part, &content_type, &contents, &setup_sync_message_contents)
        }
      } else {
        Err(anyhow!("Mock server is already started"))
      }
    });

    match result {
      Some(value) => match value {
        Ok(plugin_config) => {
          if let Some((plugin, version, config)) = plugin_config {
            let add_plugin_result = interaction.with_pact(&|_, pact| {
              pact.pact.add_plugin(plugin.as_str(), version.as_str(), Some(config.pact_configuration.clone()))
            });
            if let Some(Err(err)) = add_plugin_result {
              error!("Failed to add plugin configuration to pact - {}", err);
            }
          }
          Ok(0)
        }
        Err(err) => {
          error!("{}", err);
          set_error_msg(err.to_string());
          Ok(6)
        }
      }
      None => Ok(3)
    }
  }).unwrap_or(1)
}

fn setup_sync_message_contents(
  interaction: &mut dyn V4Interaction,
  contents: Vec<InteractionContents>,
  plugin_name: String,
  _plugin_version: String
) {
  let message = interaction.as_v4_sync_message_mut().unwrap();

  if let Some(contents) = &contents.iter().find(|c| c.part_name == "request") {
    message.request.contents = contents.body.clone();
    let metadata = contents.metadata.as_ref()
      .map(|md| md.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
      .unwrap_or_default();
    message.request.metadata = metadata;
    if let Some(rules) = &contents.rules {
      message.request.matching_rules.add_rules("body", rules.clone());
    }
    if let Some(rules) = &contents.metadata_rules {
      message.request.matching_rules.add_rules("metadata", rules.clone());
    }
    if let Some(generators) = &contents.generators {
      message.request.generators.add_generators(generators.clone());
    }
    if !contents.plugin_config.interaction_configuration.is_empty() {
      message.plugin_config.insert(plugin_name.clone(), contents.plugin_config.interaction_configuration.clone());
    }
    message.interaction_markup = InteractionMarkup {
      markup: contents.interaction_markup.clone(),
      markup_type: contents.interaction_markup_type.clone()
    };
  }

  for c in contents.iter().filter(|c| c.part_name == "response") {
    let mut matching_rules = MatchingRules::default();
    matching_rules.add_rules("body", c.rules.as_ref().cloned().unwrap_or_default());
    if let Some(rules) = &c.metadata_rules {
      matching_rules.add_rules("metadata", rules.clone());
    }
    let mut generators = Generators::default();
    if let Some(g) = &c.generators {
      generators.add_generators(g.clone());
    }
    let metadata = c.metadata.as_ref()
      .map(|md| md.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
      .unwrap_or_default();
    message.response.push(MessageContents {
      contents: c.body.clone(),
      metadata,
      matching_rules,
      generators
    });

    if !c.plugin_config.is_empty() {
      message.plugin_config.insert(plugin_name.clone(), c.plugin_config.interaction_configuration.clone());
    }
    if !c.interaction_markup.is_empty() {
      message.interaction_markup = InteractionMarkup {
        markup: c.interaction_markup.clone(),
        markup_type: c.interaction_markup_type.clone()
      };
    }
  }
}

// TODO: This needs to setup rules/generators based on the content type
fn setup_core_matcher(interaction: &mut dyn V4Interaction, part: InteractionPart, content_type: &ContentType, definition: &Value) -> anyhow::Result<()> {
  let part = get_part(interaction, part);
  match definition {
    Value::String(s) => *part.body_mut() = OptionalBody::Present(Bytes::from(s.clone()), Some(content_type.clone()), None),
    Value::Object(ref o) => if o.contains_key("contents") {
      *part.body_mut() = body_from_json(&definition, "contents", &None);
    }
    _ => {}
  };
  Ok(())
}

fn get_part<'a>(interaction: &'a mut dyn V4Interaction, part: InteractionPart) -> &'a mut dyn HttpPart {
  if interaction.is_request_response() {
    let reqres = interaction.as_v4_http_mut().unwrap();
    match part {
      InteractionPart::Request => &mut reqres.request,
      InteractionPart::Response => &mut reqres.response
    }
  } else if interaction.is_v4_sync_message() {
    let message = interaction.as_v4_sync_message_mut().unwrap();
    match part {
      InteractionPart::Request => &mut message.request,
      InteractionPart::Response => message.response.get_mut(0).expect("Message did not have a response")
    }
  } else {
    interaction.as_v4_async_message_mut().unwrap()
  }
}

fn setup_contents(
  interaction: &mut dyn V4Interaction,
  part: InteractionPart,
  content_type: &ContentType,
  definition: &Value,
  callback: &dyn Fn(&mut dyn V4Interaction, Vec<InteractionContents>, String, String)
) -> anyhow::Result<Option<(String, String, PluginConfiguration)>> {
  match find_content_matcher(content_type) {
    Some(matcher) => {
      debug!("Found a matcher for '{}': {:?}", content_type, matcher);
      if matcher.is_core() {
        debug!("Matcher is from the core framework");
        setup_core_matcher(interaction, part, &content_type, definition).map(|_| None)
      } else {
        debug!("Plugin matcher, will get the plugin to provide the part contents");
        match definition {
          Value::Object(attributes) => {
            let map = attributes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let runtime = Builder::new_multi_thread()
              .enable_all()
              .worker_threads(2)
              .thread_name("ffi-setup_contents")
              .build()
              .expect("Could not start a Tokio runtime");
            let result = runtime.block_on(matcher.configure_interation(&content_type, map));
            match result {
              Ok((contents, plugin_config)) => {
                debug!("Interaction contents = {:?}", contents);
                debug!("Interaction plugin_config = {:?}", plugin_config);
                callback(interaction, contents, matcher.plugin_name(), matcher.plugin_version());
                Ok(plugin_config.map(|config| (matcher.plugin_name(), matcher.plugin_version(), config)))
              }
              Err(err) => Err(anyhow!("Failed to call out to plugin - {}", err))
            }
          }
          _ => Err(anyhow!("{} is not a valid value for contents", definition))
        }
      }
    }
    None => {
      debug!("No matcher was found, will default to the core framework");
      setup_core_matcher(interaction, part, &content_type, definition).map(|_| None)
    }
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;
  use std::ptr::null;

  use expectest::prelude::*;
  use pact_plugin_driver::content::InteractionContents;
  use pact_models::{matchingrules, matchingrules_list};
  use pact_models::matchingrules::MatchingRule;
  use pact_models::v4::sync_message::SynchronousMessage;

  use crate::mock_server::handles::{InteractionHandle, InteractionPart, PactHandle};

  use super::{pactffi_interaction_contents, setup_sync_message_contents};

  #[test]
  fn pactffi_interaction_contents_with_invalid_content_type() {
    let pact_handle = PactHandle::new("Test", "Test");
    let i_handle = InteractionHandle::new(pact_handle, 0);
    expect!(pactffi_interaction_contents(i_handle, InteractionPart::Request, null(), null())).to(be_equal_to(1));

    let content_type = CString::new("not valid").unwrap();
    expect!(pactffi_interaction_contents(i_handle, InteractionPart::Request, content_type.as_ptr(), null())).to(be_equal_to(4));
  }

  #[test]
  fn pactffi_interaction_contents_with_invalid_contents() {
    let pact_handle = PactHandle::new("Test", "Test");
    let i_handle = InteractionHandle::new(pact_handle, 0);
    let content_type = CString::new("application/json").unwrap();
    expect!(pactffi_interaction_contents(i_handle, InteractionPart::Request, null(), null())).to(be_equal_to(1));

    let contents = CString::new("not valid").unwrap();
    expect!(pactffi_interaction_contents(i_handle, InteractionPart::Request, content_type.as_ptr(), contents.as_ptr())).to(be_equal_to(5));
  }

  #[test]
  fn setup_sync_message_contents_handles_matching_rules_on_metadata() {
    let mut interaction = SynchronousMessage {
      .. SynchronousMessage::default()
    };
    let contents = vec![
      InteractionContents {
        part_name: "request".to_string(),
        body: Default::default(),
        rules: None,
        generators: None,
        metadata: None,
        metadata_rules: Some(matchingrules_list! {
          "metadata"; "R" => [  MatchingRule::Regex("1".to_string())  ]
        }),
        plugin_config: Default::default(),
        interaction_markup: "".to_string(),
        interaction_markup_type: "".to_string(),
      },
      InteractionContents {
        part_name: "response".to_string(),
        body: Default::default(),
        rules: None,
        generators: None,
        metadata: None,
        metadata_rules: Some(matchingrules_list! {
          "metadata"; "R" => [  MatchingRule::Regex("2".to_string())  ]
        }),
        plugin_config: Default::default(),
        interaction_markup: "".to_string(),
        interaction_markup_type: "".to_string(),
      }
    ];

    setup_sync_message_contents(&mut interaction, contents, "test".to_string(), "0".to_string());

    expect!(interaction.request.matching_rules).to(be_equal_to(matchingrules! {
      "metadata" => { "R" => [  MatchingRule::Regex("1".to_string())  ] }
    }));
    expect!(interaction.response[0].clone().matching_rules).to(be_equal_to(matchingrules! {
      "metadata" => { "R" => [  MatchingRule::Regex("2".to_string())  ] }
    }));
  }
}
