//! Recording which plugins a Pact file needs because of the matching rules and generators in it.
//!
//! A plugin-provided field-level matching rule or generator never goes through
//! `configure_interaction`, which is where the `plugins` entry in the Pact metadata is written for
//! a plugin that owns a content type or a transport. Without that entry, provider verification has
//! no way to know it has to load the plugin before it can interpret the file.
//!
//! So before a Pact is written, every `MatchingRule::Plugin` and `Generator::Plugin` in it is
//! resolved against the catalogue and the plugin that provides it recorded. See proposal 006
//! section 4.

use std::collections::HashSet;

use pact_models::generators::{Generator, Generators};
use pact_models::matchingrules::{MatchingRule, MatchingRules};
use pact_models::pact::Pact;
use pact_plugin_driver::field::{find_field_generator, find_field_matcher};
use tracing::warn;

/// Records the plugin providing every plugin-supplied matching rule and generator in the Pact, in
/// the Pact's `plugins` metadata.
///
/// A name that resolves to nothing is left alone rather than failing the build: it will fail with
/// a better message when the rule is applied, and a Pact that cannot be verified is still more
/// useful than no Pact at all.
pub(crate) fn record_plugins_for_rules(pact: &mut (dyn Pact + Send + Sync)) {
  let mut names: HashSet<String> = HashSet::new();

  for interaction in pact.interactions() {
    if let Some(http) = interaction.as_v4_http() {
      collect_from_rules(&http.request.matching_rules, &mut names);
      collect_from_generators(&http.request.generators, &mut names);
      collect_from_rules(&http.response.matching_rules, &mut names);
      collect_from_generators(&http.response.generators, &mut names);
    } else if let Some(message) = interaction.as_v4_async_message() {
      collect_from_rules(&message.contents.matching_rules, &mut names);
      collect_from_generators(&message.contents.generators, &mut names);
    } else if let Some(message) = interaction.as_v4_sync_message() {
      collect_from_rules(&message.request.matching_rules, &mut names);
      collect_from_generators(&message.request.generators, &mut names);
      for response in &message.response {
        collect_from_rules(&response.matching_rules, &mut names);
        collect_from_generators(&response.generators, &mut names);
      }
    }
  }

  for name in names {
    if let Some(manifest) = plugin_for(name.as_str()) {
      if let Err(err) = pact.add_plugin(manifest.0.as_str(), manifest.1.as_str(), None) {
        warn!("Could not record the plugin providing '{}' in the Pact metadata - {}", name, err);
      }
    }
  }
}

fn collect_from_rules(rules: &MatchingRules, names: &mut HashSet<String>) {
  for category in rules.rules.values() {
    for rule_list in category.rules.values() {
      for rule in &rule_list.rules {
        if let MatchingRule::Plugin { name, .. } = rule {
          names.insert(name.clone());
        }
      }
    }
  }
}

fn collect_from_generators(generators: &Generators, names: &mut HashSet<String>) {
  for category in generators.categories.values() {
    for generator in category.values() {
      if let Generator::Plugin { name, .. } = generator {
        names.insert(name.clone());
      }
    }
  }
}

/// The name and version of the plugin providing the rule or generator, if the catalogue knows one
fn plugin_for(name: &str) -> Option<(String, String)> {
  find_field_matcher(name).ok()
    .and_then(|matcher| matcher.plugin())
    .or_else(|| find_field_generator(name).ok().and_then(|generator| generator.plugin()))
    .map(|manifest| (manifest.name.clone(), manifest.version.clone()))
}
