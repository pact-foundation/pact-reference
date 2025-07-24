//! Builder for JSON bodies

use bytes::Bytes;
use serde_json::{Map, Value};
use tracing::trace;

use pact_models::content_types::ContentType;
use pact_models::path_exp::DocPath;

use crate::engine::bodies::{drop_indices, PlanBodyBuilder, remove_marker, should_apply_to_map_entries};
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

  fn process_body_node(
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
        let matchers = context.select_best_matcher(&path)
          .and_rules(&context.select_best_matcher(&rewritten_path))
          .remove_duplicates();
        if !matchers.is_empty() {
          root_node.add(ExecutionPlanNode::annotation(format!("{} {}", path.last_field().unwrap_or_default(), matchers.generate_description(false))));
          root_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(json),
                                                 &ExecutionPlanNode::resolve_current_value(path), &matchers, false));
        } else {
          let mut match_node = ExecutionPlanNode::action("match:equality");
          match_node
            .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), json.to_string())))
            .add(ExecutionPlanNode::resolve_current_value(path))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL));
          root_node.add(match_node);
        }
      }
    }
  }

  fn process_object(
    context: &PlanMatchingContext,
    json: &Value,
    path: &DocPath,
    root_node: &mut ExecutionPlanNode,
    rewritten_path: &DocPath,
    entries: &Map<String, Value>
  ) {
    let rules = context.select_best_matcher(&path)
      .and_rules(&context.select_best_matcher(&rewritten_path))
      .remove_duplicates();
    if !rules.is_empty() && should_apply_to_map_entries(&rules) {
      root_node.add(ExecutionPlanNode::annotation(rules.generate_description(true)));
      root_node.add(build_matching_rule_node(&ExecutionPlanNode::value_node(json.clone()),
        &ExecutionPlanNode::resolve_current_value(path), &rules, true));
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
    }

    for (key, value) in entries {
      let mut item_path = path.clone();
      item_path.push_field(key);
      let mut item_node = ExecutionPlanNode::container(&item_path);
      Self::process_body_node(context, value, &item_path, &mut item_node);
      root_node.add(item_node);
    }
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
        &ExecutionPlanNode::resolve_current_value(path), &matchers, true));

      if let Some(template) = items.first() {
        let mut for_each_node = ExecutionPlanNode::action("for-each");
        let marker = format!("{}*", path.last_field().unwrap_or_default());
        for_each_node.add(ExecutionPlanNode::value_node(marker.as_str()));
        let item_path = path.parent()
          .unwrap_or_else(|| path.clone())
          .join_field(marker);
        for_each_node.add(ExecutionPlanNode::resolve_current_value(path));
        let mut item_node = ExecutionPlanNode::container(&item_path);
        match template {
          Value::Array(_) => Self::process_body_node(context, template, &item_path, &mut item_node),
          Value::Object(_) => Self::process_body_node(context, template, &item_path, &mut item_node),
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
                                                          &ExecutionPlanNode::resolve_current_value(&item_path), &matchers, false));
            } else {
              presence_check.add(
                ExecutionPlanNode::action("match:equality")
                  .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), template.to_string())))
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
                  .add(ExecutionPlanNode::value_node(NodeValue::NULL))
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
                                                          &ExecutionPlanNode::resolve_current_value(&item_path), &matchers, false));
            } else {
              presence_check.add(
                ExecutionPlanNode::action("match:equality")
                  .add(ExecutionPlanNode::value_node(NodeValue::NAMESPACED("json".to_string(), item.to_string())))
                  .add(ExecutionPlanNode::resolve_current_value(&item_path))
                  .add(ExecutionPlanNode::value_node(NodeValue::NULL))
              );
            }
            presence_check.add(
              ExecutionPlanNode::action("error")
                .add(ExecutionPlanNode::value_node(format!("Expected a value for '{}' but it was missing",
                                                           item_path.as_json_pointer().unwrap())))
            );
            item_node.add(presence_check);
            root_node.add(item_node);
          }
        }
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
      NULL
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
      NULL
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
      NULL
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
      NULL
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
      NULL
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
          NULL
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
          NULL
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
          NULL
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
        NULL
      )
    ),
    :$.b (
      %match:equality (
        json:200,
        ~>$.b,
        NULL
      )
    ),
    :$.c (
      %match:equality (
        json:300,
        ~>$.c,
        NULL
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
        json:{"regex":"^[0-9]+$"}
      )
    ),
    :$.b (
      %match:equality (
        json:200,
        ~>$.b,
        NULL
      )
    ),
    :$.c (
      %match:equality (
        json:300,
        ~>$.c,
        NULL
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
        json:{"min":2}
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
              json:{}
            )
          )
        )
      )
    )
  )
)"#, buffer);
  }

}
