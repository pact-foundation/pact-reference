//! Types for supporting building and executing plans for bodies

use std::fmt::Debug;
use std::sync::{Arc, LazyLock, RwLock};

use bytes::Bytes;

use pact_models::content_types::ContentType;
use pact_models::matchingrules::{MatchingRule, RuleList};
use pact_models::path_exp::{DocPath, PathToken};

use crate::engine::{ExecutionPlanNode, NodeValue, PlanMatchingContext};
use crate::engine::bodies::json::JsonPlanBuilder;
use crate::engine::bodies::xml::XMLPlanBuilder;

pub mod json;
#[cfg(feature = "xml")] pub mod xml;

/// Trait for implementations of builders for different types of bodies
pub trait PlanBodyBuilder: Debug {
  /// If this builder supports a namespace for nodes.
  fn namespace(&self) -> Option<String> {
    None
  }

  /// If this builder supports the given content type
  fn supports_type(&self, content_type: &ContentType) -> bool;

  /// Build the plan for the expected body
  fn build_plan(&self, content: &Bytes, context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode>;
}

static BODY_PLAN_BUILDERS: LazyLock<RwLock<Vec<Arc<dyn PlanBodyBuilder + Send + Sync>>>> = LazyLock::new(|| {
  let mut builders: Vec<Arc<dyn PlanBodyBuilder + Send + Sync>> = vec![];

  // TODO: Add default implementations here
  builders.push(Arc::new(JsonPlanBuilder::new()));
  #[cfg(feature = "xml")]
  builders.push(Arc::new(XMLPlanBuilder::new()));

  RwLock::new(builders)
});

pub(crate) fn get_body_plan_builder(content_type: &ContentType) -> Option<Arc<dyn PlanBodyBuilder + Send + Sync>> {
  let registered_builders = (*BODY_PLAN_BUILDERS).read().unwrap();
  registered_builders.iter().find(|builder| builder.supports_type(content_type))
    .cloned()
}

/// Plan builder for plain text. This just sets up an equality matcher
#[derive(Clone, Debug)]
pub struct PlainTextBuilder;

impl PlainTextBuilder {
  /// Create a new instance
  pub fn new() -> Self {
    PlainTextBuilder{}
  }
}

impl PlanBodyBuilder for PlainTextBuilder {
  fn supports_type(&self, content_type: &ContentType) -> bool {
    content_type.is_text()
  }

  fn build_plan(&self, content: &Bytes, _context: &PlanMatchingContext) -> anyhow::Result<ExecutionPlanNode> {
    let bytes = content.to_vec();
    let text_content = String::from_utf8_lossy(&bytes);
    let mut node = ExecutionPlanNode::action("match:equality");
    let mut child_node = ExecutionPlanNode::action("convert:UTF8");
    child_node.add(ExecutionPlanNode::resolve_value(DocPath::new_unwrap("$.body")));
    node.add(ExecutionPlanNode::value_node(text_content.to_string()));
    node.add(child_node);
    node.add(ExecutionPlanNode::value_node(NodeValue::NULL));
    Ok(node)
  }
}

fn should_apply_to_map_entries(rules: &RuleList) -> bool {
  rules.rules.iter().any(|rule| {
    match rule {
      MatchingRule::Values => true,
      MatchingRule::EachKey(_) => true,
      MatchingRule::EachValue(_) => true,
      _ => false
    }
  })
}

fn drop_indices(path: &DocPath) -> DocPath {
  DocPath::from_tokens(path.tokens()
    .iter()
    .filter(|token| match token {
      PathToken::Index(_) | PathToken::StarIndex => false,
      _ => true
    })
    .map(|token| {
      if let PathToken::Field(name) = token {
        if name.ends_with('*') {
          PathToken::Field(name.trim_end_matches('*').to_string())
        } else {
          token.clone()
        }
      } else {
        token.clone()
      }
    }))
}

fn remove_marker(path: &DocPath) -> DocPath {
  DocPath::from_tokens(path.tokens()
    .iter()
    .flat_map(|token| {
      if let PathToken::Field(name) = token {
        if name.ends_with('*') {
          vec![PathToken::Field(name.trim_end_matches('*').to_string()), PathToken::Index(0)]
        } else {
          vec![token.clone()]
        }
      } else {
        vec![token.clone()]
      }
    }))
}
