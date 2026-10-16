//! Builder for application/x-www-form-urlencoded bodies

use bytes::Bytes;
use itertools::Itertools;
use pact_models::content_types::ContentType;
use pact_models::path_exp::DocPath;

use crate::engine::{build_matching_rule_node, ExecutionPlanNode, NodeValue};
use crate::engine::bodies::PlanBodyBuilder;
use crate::engine::context::PlanMatchingContext;

/// Plan builder for application/x-www-form-urlencoded bodies
#[derive(Clone, Debug)]
pub struct FormUrlencodedPlanBuilder;

impl FormUrlencodedPlanBuilder {
  /// Create a new instance
  pub fn new() -> Self {
    FormUrlencodedPlanBuilder {}
  }
}

impl PlanBodyBuilder for FormUrlencodedPlanBuilder {
  fn supports_type(&self, content_type: &ContentType) -> bool {
    content_type.base_type() == "application/x-www-form-urlencoded"
  }

  fn build_plan(&self, content: &Bytes, context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode> {
    let expected_form: Vec<(String, String)> = serde_urlencoded::from_bytes(content)
      .map_err(|e| anyhow::anyhow!("Failed to parse form-urlencoded body: {}", e))?;

    let mut params: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (key, value) in expected_form {
      params.entry(key).or_default().push(value);
    }

    let mut body_node = ExecutionPlanNode::action("tee");
    body_node.add(
      ExecutionPlanNode::action("form:parse")
        .add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.body")))
    );

    let root_path = DocPath::root();
    let mut root_node = ExecutionPlanNode::container(&root_path);

    let keys = params.keys().cloned().sorted().collect_vec();

    if !params.is_empty() {
      for key in &keys {
        let values = params.get(key).unwrap();
        let key_path = root_path.join(key);

        let expected_value = if values.len() == 1 {
          NodeValue::STRING(values[0].clone())
        } else {
          NodeValue::SLIST(values.clone())
        };

        let mut item_node = ExecutionPlanNode::container(&key_path);
        let mut presence_check = ExecutionPlanNode::action("if");
        presence_check.add(
          ExecutionPlanNode::action("check:exists")
            .add(ExecutionPlanNode::resolve_current_value(&key_path))
        );

        if context.matcher_is_defined(&key_path) {
          let matchers = context.select_best_matcher(&key_path);
          item_node.add(ExecutionPlanNode::annotation(
            format!("{} {}", key, matchers.generate_description(true))
          ));
          presence_check.add(build_matching_rule_node(
            &ExecutionPlanNode::value_node(expected_value),
            &ExecutionPlanNode::resolve_current_value(&key_path),
            &matchers,
            true,
            context.config.show_types_in_errors
          ));
        } else {
          item_node.add(ExecutionPlanNode::annotation(format!("{}={}", key, expected_value)));
          presence_check.add(
            ExecutionPlanNode::action("match:equality")
              .add(ExecutionPlanNode::value_node(expected_value))
              .add(ExecutionPlanNode::resolve_current_value(&key_path))
              .add(ExecutionPlanNode::value_node(NodeValue::NULL))
              .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors))
          );
        }

        item_node.add(presence_check);
        root_node.add(item_node);
      }

      root_node.add(
        ExecutionPlanNode::action("expect:entries")
          .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone())))
          .add(ExecutionPlanNode::resolve_current_value(&root_path))
          .add(
            ExecutionPlanNode::action("join")
              .add(ExecutionPlanNode::value_node("The following expected form parameters were missing: "))
              .add(
                ExecutionPlanNode::action("join-with")
                  .add(ExecutionPlanNode::value_node(", "))
                  .add(
                    ExecutionPlanNode::splat()
                      .add(ExecutionPlanNode::action("apply"))
                  )
              )
          )
      );

      if !context.config.allow_unexpected_entries {
        root_node.add(
          ExecutionPlanNode::action("expect:only-entries")
            .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone())))
            .add(ExecutionPlanNode::resolve_current_value(&root_path))
            .add(
              ExecutionPlanNode::action("join")
                .add(ExecutionPlanNode::value_node("The following form parameters were not expected: "))
                .add(
                  ExecutionPlanNode::action("join-with")
                    .add(ExecutionPlanNode::value_node(", "))
                    .add(
                      ExecutionPlanNode::splat()
                        .add(ExecutionPlanNode::action("apply"))
                    )
                )
            )
        );
      }
    }

    body_node.add(root_node);
    Ok(body_node)
  }
}
