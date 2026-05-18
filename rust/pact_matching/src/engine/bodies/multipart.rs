//! Plan builder for multipart/form-data bodies

use bytes::Bytes;
use serde_json::Value;
use tracing::trace;

use pact_models::content_types::ContentType;
use pact_models::path_exp::DocPath;

use crate::binary_utils::parse_multipart_body;
use crate::engine::bodies::json::JsonPlanBuilder;
use crate::engine::bodies::PlanBodyBuilder;
use crate::engine::context::PlanMatchingContext;
use crate::engine::{build_matching_rule_node, ExecutionPlanNode, NodeValue};

/// Plan builder for multipart/form-data bodies
#[derive(Clone, Debug)]
pub struct MultipartFormDataPlanBuilder;

impl MultipartFormDataPlanBuilder {
  /// Create a new instance
  pub fn new() -> Self {
    MultipartFormDataPlanBuilder {}
  }
}

impl PlanBodyBuilder for MultipartFormDataPlanBuilder {
  fn supports_type(&self, content_type: &ContentType) -> bool {
    content_type.main_type == "multipart"
  }

  fn build_plan(&self, content: &Bytes, context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode> {
    // Get the expected content-type (with boundary) from the interaction
    let expected_ct = context.interaction.as_v4_http()
      .and_then(|r| r.request.content_type())
      .map(|ct| ct.to_string())
      .unwrap_or_else(|| "multipart/form-data".to_string());

    trace!(%expected_ct, "Building multipart plan");

    // Parse expected parts at build time to know their names and content
    let expected_parts = parse_multipart_body(content.clone(), &expected_ct)?;

    let mut body_node = ExecutionPlanNode::action("tee");
    body_node.add(
      ExecutionPlanNode::action("multipart:parse")
        .add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.body")))
        .add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.content-type")))
    );

    let mut root_node = ExecutionPlanNode::container("$");
    let mut part_names: Vec<String> = vec![];

    for (name, (data, mime_ct)) in &expected_parts {
      part_names.push(name.clone());
      let part_path = DocPath::new(&format!("$.{}", name))?;
      let mut part_node = ExecutionPlanNode::container(part_path.to_string());

      let mut presence = ExecutionPlanNode::action("if");
      presence.add(
        ExecutionPlanNode::action("check:exists")
          .add(ExecutionPlanNode::resolve_current_value(part_path.clone()))
      );

      let is_json = mime_ct.as_ref()
        .map(|ct| ct.type_() == mime::APPLICATION && ct.subtype() == mime::JSON)
        .unwrap_or(false);

      if is_json {
        // Build JSON sub-plan starting at part_path so RESOLVE_CURRENT paths are
        // $.metadata, $.metadata.name etc., matching the body rules at those paths.
        if let Ok(json) = serde_json::from_slice::<Value>(data.as_ref()) {
          let mut sub_node = ExecutionPlanNode::container(part_path.to_string());
          JsonPlanBuilder::process_body_node(context, &json, &part_path, &mut sub_node);
          presence.add(sub_node);
        }
      } else {
        // For binary/text parts, apply any matching rules defined at the part path
        if context.matcher_is_defined(&part_path) {
          let matchers = context.select_best_matcher(&part_path);
          presence.add(build_matching_rule_node(
            &ExecutionPlanNode::value_node(NodeValue::NULL),
            &ExecutionPlanNode::resolve_current_value(part_path.clone()),
            &matchers,
            false,
            context.config.show_types_in_errors,
          ));
        }
      }

      // else error branch: part was expected but missing
      presence.add(
        ExecutionPlanNode::action("error")
          .add(ExecutionPlanNode::value_node(NodeValue::STRING(
            format!("Expected multipart part '{}' but was missing", name)
          )))
      );

      part_node.add(presence);
      root_node.add(part_node);
    }

    // Verify all expected parts are present in the actual
    root_node.add(
      ExecutionPlanNode::action("expect:entries")
        .add(ExecutionPlanNode::value_node(NodeValue::SLIST(part_names)))
        .add(ExecutionPlanNode::resolve_current_value(DocPath::root()))
    );

    body_node.add(root_node);
    Ok(body_node)
  }
}
