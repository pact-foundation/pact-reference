//! Builder for JSON bodies

use bytes::Bytes;
use itertools::Itertools;
use serde_json::{Map, Value};
use tracing::trace;

use pact_models::content_types::ContentType;
use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleList, RuleLogic};
use pact_models::path_exp::{DocPath, PathToken};

use crate::engine::bodies::{PlanBodyBuilder, remove_marker, should_apply_to_map_entries};
use crate::engine::context::PlanMatchingContext;
use crate::engine::{build_matching_rule_node, ExecutionPlanNode, NodeValue};

/// Plan builder for JSON bodies
#[derive(Clone, Debug)]
pub struct JsonPlanBuilder;

impl JsonPlanBuilder {
  /// Create a new instance
  pub fn new() -> Self {
    JsonPlanBuilder{}
  }

  pub(crate) fn process_body_node(
    context: &PlanMatchingContext,
    json: &Value,
    path: &DocPath,
    root_node: &mut ExecutionPlanNode
  ) {
    trace!(%json, %path, ">>> process_body_node");

    let rewritten_path = remove_marker(&path);

    match &json {
      Value::Array(items) => {
        Self::process_array(context, json, path, root_node, &rewritten_path, items);
      }
      Value::Object(entries) => {
        Self::process_object(context, json, path, root_node, &rewritten_path, entries);
      }
      _ => {
        let matchers = Self::select_matchers(context, path, &rewritten_path);
        if !matchers.is_empty() {
          root_node.add(ExecutionPlanNode::annotation(format!("{} {}", path.last_field().unwrap_or_default(), matchers.generate_description(false))));
          root_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(json),
            &ExecutionPlanNode::resolve_current_value(path), &matchers, false,
            context.config.show_types_in_errors));
        } else {
          let mut match_node = ExecutionPlanNode::action("match:equality");
          match_node
            .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), json.to_string())))
            .add(ExecutionPlanNode::resolve_current_value(path))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL))
            .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors));
          root_node.add(match_node);
        }
      }
    }
  }

  /// Selects the rules for the path and the path with the markers removed. An `EachValue` rule
  /// from a parent path applies to the values of that collection, not to the values below them, so
  /// they are ignored if they have cascaded down from a parent path.
  fn select_matchers(
    context: &PlanMatchingContext,
    path: &DocPath,
    rewritten_path: &DocPath
  ) -> RuleList {
    let without_cascaded_each_value = |mut rules: RuleList| {
      if rules.cascaded {
        rules.rules.retain(|rule| !matches!(rule, MatchingRule::EachValue(_)));
      }
      rules
    };
    without_cascaded_each_value(context.select_best_matcher(path))
      .and_rules(&without_cascaded_each_value(context.select_best_matcher(rewritten_path)))
      .remove_duplicates()
  }

  fn process_object(
    context: &PlanMatchingContext,
    json: &Value,
    path: &DocPath,
    root_node: &mut ExecutionPlanNode,
    rewritten_path: &DocPath,
    entries: &Map<String, Value>
  ) {
    let rules = Self::select_matchers(context, path, rewritten_path);
    if !rules.is_empty() && should_apply_to_map_entries(&rules) {
      root_node.add(ExecutionPlanNode::annotation(rules.generate_description(true)));
      root_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(json.clone()),
        &ExecutionPlanNode::resolve_current_value(path), &rules, true,
        context.config.show_types_in_errors));
      if let Some(template) = entries.values().next() &&
        rules.rules.iter().any(|rule| matches!(rule, MatchingRule::EachValue(_))) &&
        (template.is_object() || template.is_array()) {
        let each_value_context = Self::each_value_context(context, path, &rules, false);
        root_node.add(Self::build_for_each_value_node(&each_value_context, path, template));
      }
    } else if entries.is_empty() {
      root_node.add(
        ExecutionPlanNode::action("json:expect:empty")
          .add(ExecutionPlanNode::value_node("OBJECT"))
          .add(ExecutionPlanNode::resolve_current_value(path))
      );
    } else {
      let keys = NodeValue::SLIST(entries.keys().map(|key| key.clone()).collect());
      root_node.add(
        ExecutionPlanNode::action("json:expect:entries")
          .add(ExecutionPlanNode::value_node("OBJECT"))
          .add(ExecutionPlanNode::value_node(keys.clone()))
          .add(ExecutionPlanNode::resolve_current_value(path))
      );
      if !context.config.allow_unexpected_entries {
        root_node.add(
          ExecutionPlanNode::action("expect:only-entries")
            .add(ExecutionPlanNode::value_node(keys.clone()))
            .add(ExecutionPlanNode::resolve_current_value(path))
        );
      } else {
        root_node.add(
          ExecutionPlanNode::action("json:expect:not-empty")
            .add(ExecutionPlanNode::value_node("OBJECT"))
            .add(ExecutionPlanNode::resolve_current_value(path))
        );
      }

      for (key, value) in entries {
        let mut item_path = path.clone();
        item_path.push_field(key);
        let mut item_node = ExecutionPlanNode::container(&item_path);
        Self::process_body_node(context, value, &item_path, &mut item_node);
        root_node.add(item_node);
      }
    }
  }

  /// Returns the context to use for the values of a collection with an `EachValue` rule. The rule's
  /// rules are added for the values (at `path.*`), so that they apply to the values below them that
  /// have no rules of their own.
  ///
  /// If there are already rules for the values (at `path.*` or `path[*]`), the rule's rules are
  /// merged into them. Adding a separate group would not work, as only one of the groups would be
  /// selected. A group with OR logic is replaced with the rule's rules instead, as merging them
  /// would make the rule's rules optional.
  ///
  /// If there are no rules for the values yet, they are added at `path[*]` for a list and `path.*`
  /// for a map. A rule at `path.*` would also be selected for the paths with a marker (like `$*`),
  /// which a rule at `path[*]` is not.
  fn each_value_context(
    context: &PlanMatchingContext,
    path: &DocPath,
    rules: &RuleList,
    is_list: bool
  ) -> PlanMatchingContext {
    let each_value_rules = rules.rules.iter()
      .filter_map(|rule| match rule {
        MatchingRule::EachValue(definition) => Some(definition),
        _ => None
      })
      .flat_map(|definition| definition.rules.iter())
      .filter_map(|rule| rule.as_ref().left().cloned())
      .collect_vec();

    let collection_path = remove_marker(path);
    let collection_fragments = collection_path.to_vec();
    let collection_slice = collection_fragments.iter().map(|p| p.as_str()).collect_vec();
    let mut matching_rules = context.matching_rules.rules.clone();
    let mut value_paths = matching_rules.keys()
      .filter(|rule_path| rule_path.len() == collection_path.len() + 1 &&
        matches!(rule_path.tokens().last(), Some(PathToken::Star | PathToken::StarIndex)) &&
        DocPath::from_tokens(rule_path.tokens()[..collection_path.len()].to_vec())
          .matches_path_exactly(collection_slice.as_slice()))
      .cloned()
      .collect_vec();
    if value_paths.is_empty() {
      value_paths.push(collection_path.join(if is_list { "[*]" } else { "*" }));
    }

    for value_path in value_paths {
      let mut value_rules = RuleList {
        rules: each_value_rules.clone(),
        rule_logic: RuleLogic::And,
        cascaded: false
      };
      if let Some(existing) = matching_rules.get(&value_path)
        .filter(|existing| existing.rule_logic == RuleLogic::And) {
        for rule in &existing.rules {
          if !value_rules.rules.contains(rule) {
            value_rules.rules.push(rule.clone());
          }
        }
      }
      matching_rules.insert(value_path, value_rules);
    }

    PlanMatchingContext {
      matching_rules: MatchingRuleCategory {
        name: context.matching_rules.name.clone(),
        rules: matching_rules
      },
      .. context.clone()
    }
  }

  /// Builds the node to match each value of a map with an `EachValue` rule against the first
  /// expected value, so that the rules defined below the values (like `$.chips.*.amount`) are
  /// applied.
  fn build_for_each_value_node(
    context: &PlanMatchingContext,
    path: &DocPath,
    template: &Value
  ) -> ExecutionPlanNode {
    let mut for_each_node = ExecutionPlanNode::action("for-each");
    let marker = if path.is_root() {
      "$*".to_string()
    } else {
      format!("{}*", path.last_field().unwrap_or_default())
    };
    for_each_node.add(ExecutionPlanNode::value_node(marker.as_str()));
    for_each_node.add(ExecutionPlanNode::resolve_current_value(path));

    let item_path = path.parent()
      .unwrap_or_else(|| path.clone())
      .join_field(marker);
    let mut item_node = ExecutionPlanNode::container(&item_path);
    Self::process_body_node(context, template, &item_path, &mut item_node);
    for_each_node.add(item_node);
    for_each_node
  }

  fn process_array(
    context: &PlanMatchingContext,
    json: &Value,
    path: &DocPath,
    root_node: &mut ExecutionPlanNode,
    rewritten_path: &DocPath,
    items: &Vec<Value>
  ) {
    let filtered_path = remove_marker(path);
    if context.matcher_is_defined(&filtered_path) {
      let matchers = context.select_best_matcher(&filtered_path);
      root_node.add(ExecutionPlanNode::annotation(format!("{} {}",
        path.last_field().unwrap_or_default(),
        matchers.generate_description(true))));
      root_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(json.clone()),
        &ExecutionPlanNode::resolve_current_value(path), &matchers, !matchers.cascaded,
        context.config.show_types_in_errors));

      if let Some(template) = items.first() {
        let mut for_each_node = ExecutionPlanNode::action("for-each");
        let marker = if path.is_root() {
          "$*".to_string()
        } else {
          format!("{}*", path.last_field().unwrap_or_default())
        };
        for_each_node.add(ExecutionPlanNode::value_node(marker.as_str()));

        let item_path = path.parent()
          .unwrap_or_else(|| path.clone())
          .join_field(marker);
        for_each_node.add(ExecutionPlanNode::resolve_current_value(path));
        let mut item_node = ExecutionPlanNode::container(&item_path);
        let item_context = if !matchers.cascaded &&
          matchers.rules.iter().any(|rule| matches!(rule, MatchingRule::EachValue(_))) {
          Self::each_value_context(context, path, &matchers, true)
        } else {
          context.clone()
        };
        match template {
          Value::Array(_) => Self::process_body_node(&item_context, template, &item_path, &mut item_node),
          Value::Object(_) => Self::process_body_node(&item_context, template, &item_path, &mut item_node),
          _ => {
            let mut presence_check = ExecutionPlanNode::action("if");
            presence_check
              .add(
                ExecutionPlanNode::action("check:exists")
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
              );

            let matchers = context.select_best_matcher(&item_path)
              .and_rules(&context.select_best_matcher(&rewritten_path))
              .remove_duplicates();
            if !matchers.is_empty() {
              let matchers = context.select_best_matcher(&item_path);
              presence_check.add(ExecutionPlanNode::annotation(format!("[*] {}", matchers.generate_description(false))));
              presence_check.add(build_matching_rule_node(&ExecutionPlanNode::value_node(template),
                &ExecutionPlanNode::resolve_current_value(&item_path), &matchers, false,
                context.config.show_types_in_errors));
            } else {
              presence_check.add(
                ExecutionPlanNode::action("match:equality")
                  .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), template.to_string())))
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
                  .add(ExecutionPlanNode::value_node(NodeValue::NULL))
                  .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors))
              );
            }
            item_node.add(presence_check);
          }
        }
        for_each_node.add(item_node);
        root_node.add(for_each_node);
      }
    } else if items.is_empty() {
      root_node.add(
        ExecutionPlanNode::action("json:expect:empty")
          .add(ExecutionPlanNode::value_node("ARRAY"))
          .add(ExecutionPlanNode::resolve_current_value(path))
      );
    } else {
      root_node.add(
        ExecutionPlanNode::action("json:match:length")
          .add(ExecutionPlanNode::value_node("ARRAY"))
          .add(ExecutionPlanNode::value_node(items.len()))
          .add(ExecutionPlanNode::resolve_current_value(path))
      );

      for (index, item) in items.iter().enumerate() {
        let item_path = path.join_index(index);
        let mut item_node = ExecutionPlanNode::container(&item_path);
        match item {
          Value::Array(_) => Self::process_body_node(context, item, &item_path, &mut item_node),
          Value::Object(_) => Self::process_body_node(context, item, &item_path, &mut item_node),
          _ => {
            let mut presence_check = ExecutionPlanNode::action("if");
            presence_check
              .add(
                ExecutionPlanNode::action("check:exists")
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
              );
            let matchers = context.select_best_matcher(&item_path)
              .and_rules(&context.select_best_matcher(&rewritten_path))
              .remove_duplicates();
            if !matchers.is_empty() {
              let matchers = context.select_best_matcher(&item_path);
              presence_check.add(ExecutionPlanNode::annotation(format!("[{}] {}", index, matchers.generate_description(false))));
              presence_check.add(build_matching_rule_node(&ExecutionPlanNode::value_node(item),
                &ExecutionPlanNode::resolve_current_value(&item_path), &matchers, false, false));
            } else {
              presence_check.add(
                ExecutionPlanNode::action("match:equality")
                  .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), item.to_string())))
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
                  .add(ExecutionPlanNode::value_node(NodeValue::NULL))
                  .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors))
              );
            }
            presence_check.add(
              ExecutionPlanNode::action("error")
                .add(ExecutionPlanNode::value_node(format!("Expected a value for '{}' but it was missing",
                                                           item_path.as_json_pointer().unwrap())))
            );
            item_node.add(presence_check);
          }
        }

        root_node.add(item_node);
      }
    }
  }
}

impl PlanBodyBuilder for JsonPlanBuilder {
  fn namespace(&self) -> Option<String> {
    Some("json".to_string())
  }

  fn supports_type(&self, content_type: &ContentType) -> bool {
    content_type.is_json()
  }

  fn build_plan(&self, content: &Bytes, context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode> {
    let expected_json: Value = serde_json::from_slice(&content)?;
    let mut body_node = ExecutionPlanNode::action("tee");
    body_node
      .add(ExecutionPlanNode::action("json:parse")
        .add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.body"))));

    let path = DocPath::root();
    let mut root_node = ExecutionPlanNode::container(&path);
    Self::process_body_node(context, &expected_json, &path, &mut root_node);
    body_node.add(root_node);

    Ok(body_node)
  }
}

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use serde_json::{json, Value};
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use crate::engine::bodies::json::JsonPlanBuilder;
  use crate::engine::bodies::PlanBodyBuilder;
  use crate::engine::context::PlanMatchingContext;

  #[test]
  fn sets_the_show_types_value_from_config() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default().for_body();
    let content = Bytes::copy_from_slice(Value::Null.to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:null,
      ~>$,
      NULL,
      BOOL(true)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_null() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(Value::Null.to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:null,
      ~>$,
      NULL,
      BOOL(false)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_boolean() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(Value::Bool(true).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:true,
      ~>$,
      NULL,
      BOOL(false)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_string() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(Value::String("I am a string!".to_string()).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:"I am a string!",
      ~>$,
      NULL,
      BOOL(false)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_int() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!(1000).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:1000,
      ~>$,
      NULL,
      BOOL(false)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_float() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!(1000.3).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %match:equality (
      json:1000.3,
      ~>$,
      NULL,
      BOOL(false)
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_empty_array() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!([]).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:expect:empty (
      'ARRAY',
      ~>$
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_array() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!([100, 200, 300]).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:match:length (
      'ARRAY',
      UINT(3),
      ~>$
    ),
    :$[0] (
      %if (
        %check:exists (
          ~>$[0]
        ),
        %match:equality (
          json:100,
          ~>$[0],
          NULL,
          BOOL(false)
        ),
        %error (
          'Expected a value for \'/0\' but it was missing'
        )
      )
    ),
    :$[1] (
      %if (
        %check:exists (
          ~>$[1]
        ),
        %match:equality (
          json:200,
          ~>$[1],
          NULL,
          BOOL(false)
        ),
        %error (
          'Expected a value for \'/1\' but it was missing'
        )
      )
    ),
    :$[2] (
      %if (
        %check:exists (
          ~>$[2]
        ),
        %match:equality (
          json:300,
          ~>$[2],
          NULL,
          BOOL(false)
        ),
        %error (
          'Expected a value for \'/2\' but it was missing'
        )
      )
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_empty_object() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!({}).to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:expect:empty (
      'OBJECT',
      ~>$
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_object() {
    let builder = JsonPlanBuilder::new();
    let context = PlanMatchingContext::default();
    let content = Bytes::copy_from_slice(json!({"a": 100, "b": 200, "c": 300})
      .to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:expect:entries (
      'OBJECT',
      ['a', 'b', 'c'],
      ~>$
    ),
    %expect:only-entries (
      ['a', 'b', 'c'],
      ~>$
    ),
    :$.a (
      %match:equality (
        json:100,
        ~>$.a,
        NULL,
        BOOL(false)
      )
    ),
    :$.b (
      %match:equality (
        json:200,
        ~>$.b,
        NULL,
        BOOL(false)
      )
    ),
    :$.c (
      %match:equality (
        json:300,
        ~>$.c,
        NULL,
        BOOL(false)
      )
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_object_with_matching_rule() {
    let builder = JsonPlanBuilder::new();
    let matching_rules = matchingrules! {
      "body" => { "$.a" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
    };
    let context = PlanMatchingContext {
      matching_rules: matching_rules.rules_for_category("body").unwrap_or_default(),
      .. PlanMatchingContext::default()
    };
    let content = Bytes::copy_from_slice(json!({"a": 100, "b": 200, "c": 300})
      .to_string().as_bytes());
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:expect:entries (
      'OBJECT',
      ['a', 'b', 'c'],
      ~>$
    ),
    %expect:only-entries (
      ['a', 'b', 'c'],
      ~>$
    ),
    :$.a (
      #{'a must match the regular expression /^[0-9]+$/'},
      %match:regex (
        json:100,
        ~>$.a,
        json:{"regex":"^[0-9]+$"},
        BOOL(false)
      )
    ),
    :$.b (
      %match:equality (
        json:200,
        ~>$.b,
        NULL,
        BOOL(false)
      )
    ),
    :$.c (
      %match:equality (
        json:300,
        ~>$.c,
        NULL,
        BOOL(false)
      )
    )
  )
)"#, buffer);
  }

  #[test]
  fn json_plan_builder_with_array_and_type_matcher() {
    let builder = JsonPlanBuilder::new();
    let matching_rules = matchingrules! {
      "body" => { "$.item" => [ MatchingRule::MinType(2) ] }
    };
    let context = PlanMatchingContext {
      matching_rules: matching_rules.rules_for_category("body").unwrap_or_default(),
      .. PlanMatchingContext::default()
    };
    let content = Bytes::copy_from_slice(
      json!({
        "item": [
          { "a": 100 },
          { "a": 200 },
          { "a": 300 }
        ]
      }).to_string().as_bytes()
    );
    let node = builder.build_plan(&content, &context).unwrap();
    let mut buffer = String::new();
    node.pretty_form(&mut buffer, 0);
    pretty_assertions::assert_eq!(r#"%tee (
  %json:parse (
    $.body
  ),
  :$ (
    %json:expect:entries (
      'OBJECT',
      ['item'],
      ~>$
    ),
    %expect:only-entries (
      ['item'],
      ~>$
    ),
    :$.item (
      #{'item must match by type and have at least 2 items'},
      %match:min-type (
        json:[{"a":100},{"a":200},{"a":300}],
        ~>$.item,
        json:{"min":2},
        BOOL(false)
      ),
      %for-each (
        'item*',
        ~>$.item,
        :$['item*'] (
          %json:expect:entries (
            'OBJECT',
            ['a'],
            ~>$['item*']
          ),
          %expect:only-entries (
            ['a'],
            ~>$['item*']
          ),
          :$['item*'].a (
            #{'a must match by type'},
            %match:type (
              json:100,
              ~>$['item*'].a,
              json:{},
              BOOL(false)
            )
          )
        )
      )
    )
  )
)"#, buffer);
  }

}
