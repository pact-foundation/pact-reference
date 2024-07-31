//! Structs and traits to support a general matching engine

use std::panic::RefUnwindSafe;
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::synch_http::SynchronousHttp;

#[derive(Clone, Debug, Default)]
pub enum PlanNodeType {
  #[default]
  EMPTY,
  CONTAINER(String),
  ACTION(String),
  VALUE(NodeValue),
  RESOLVE(DocPath),
}

#[derive(Clone, Debug, Default)]
pub enum NodeValue {
  #[default]
  NULL,
  STRING(String),
}

impl NodeValue {
  pub fn str_form(&self) -> String {
    match self {
      NodeValue::NULL => "NULL".to_string(),
      NodeValue::STRING(str) => format!("\"{}\"", str)
    }
  }
}

impl From<String> for NodeValue {
  fn from(value: String) -> Self {
    NodeValue::STRING(value.clone())
  }
}

impl From<&str> for NodeValue {
  fn from(value: &str) -> Self {
    NodeValue::STRING(value.to_string())
  }
}

#[derive(Clone, Debug, Default)]
pub enum NodeResult {
  #[default]
  OK,
  VALUE(NodeValue),
  ERROR(String)
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionPlanNode {
  pub node_type: PlanNodeType,
  pub result: Option<NodeResult>,
  pub children: Vec<ExecutionPlanNode>
}

impl ExecutionPlanNode {
  pub fn pretty_form(&self, buffer: &mut String, indent: usize) {
    let pad = " ".repeat(indent);

    match &self.node_type {
      PlanNodeType::EMPTY => {}
      PlanNodeType::CONTAINER(label) => {
        buffer.push_str(pad.as_str());
        buffer.push(':');
        buffer.push_str(label.as_str());
        buffer.push_str(" (\n");
        self.pretty_form_children(buffer, indent);
        buffer.push_str(pad.as_str());
        buffer.push(')');
      }
      PlanNodeType::ACTION(value) => {
        buffer.push_str(pad.as_str());
        buffer.push('%');
        buffer.push_str(value.as_str());
        buffer.push_str(" (\n");
        self.pretty_form_children(buffer, indent);
        buffer.push_str(pad.as_str());
        buffer.push(')');
      }
      PlanNodeType::VALUE(value) => {
        buffer.push_str(pad.as_str());
        buffer.push_str(value.str_form().as_str());
      }
      PlanNodeType::RESOLVE(str) => {
        buffer.push_str(pad.as_str());
        buffer.push_str(str.to_string().as_str());
      }
    }
  }

  fn pretty_form_children(&self, buffer: &mut String, indent: usize) {
    let len = self.children.len();
    let pad = " ".repeat(indent);
    for (index, child) in self.children.iter().enumerate() {
      child.pretty_form(buffer, indent + 2);
      if index < len - 1 {
        buffer.push(',');
      }
      buffer.push('\n');
    }
  }

  pub fn str_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push('(');

    match &self.node_type {
      PlanNodeType::EMPTY => {}
      PlanNodeType::CONTAINER(label) => {
        buffer.push(':');
        buffer.push_str(label.as_str());
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');
      }
      PlanNodeType::ACTION(value) => {
        buffer.push('%');
        buffer.push_str(value.as_str());
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');
      }
      PlanNodeType::VALUE(value) => {
        buffer.push_str(value.str_form().as_str());
      }
      PlanNodeType::RESOLVE(str) => {
        buffer.push_str(str.to_string().as_str());
      }
    }

    buffer.push(')');
    buffer
  }

  fn str_form_children(&self, buffer: &mut String) {
    let len = self.children.len();
    for (index, child) in self.children.iter().enumerate() {
      buffer.push_str(child.str_form().as_str());
      if index < len - 1 {
        buffer.push(',');
      }
    }
  }

  pub fn container(label: &str) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::CONTAINER(label.to_string()),
      result: None,
      children: vec![],
    }
  }

  pub fn action(value: &str) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::ACTION(value.to_string()),
      result: None,
      children: vec![],
    }
  }

  pub fn value<T: Into<NodeValue>>(value: T) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::VALUE(value.into()),
      result: None,
      children: vec![],
    }
  }

  pub fn resolve_value<T: Into<DocPath>>(resolve_str: T) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::RESOLVE(resolve_str.into()),
      result: None,
      children: vec![],
    }
  }

  pub fn add<N>(&mut self, node: N) -> &mut Self where N: Into<ExecutionPlanNode> {
    self.children.push(node.into());
    self
  }
}

impl From<&mut ExecutionPlanNode> for ExecutionPlanNode {
  fn from(value: &mut ExecutionPlanNode) -> Self {
    value.clone()
  }
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionPlan {
  pub plan_root: ExecutionPlanNode
}

impl ExecutionPlan {
  fn new(label: &str) -> ExecutionPlan {
    ExecutionPlan {
      plan_root: ExecutionPlanNode::container(label)
    }
  }

  pub fn str_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push('(');
    buffer.push_str(self.plan_root.str_form().as_str());
    buffer.push(')');
    buffer
  }

  pub fn pretty_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push_str("(\n");
    self.plan_root.pretty_form(&mut buffer, 2);
    buffer.push_str("\n)\n");
    buffer
  }
}

#[derive(Clone, Debug)]
pub struct PlanMatchingContext {
  pub pact: V4Pact,
  pub interaction: Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>
}

impl Default for PlanMatchingContext {
  fn default() -> Self {
    PlanMatchingContext {
      pact: Default::default(),
      interaction: Box::new(SynchronousHttp::default())
    }
  }
}

pub fn build_request_plan(
  expected: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  let mut plan = ExecutionPlan::new("request");

  setup_method_plan(&mut plan.plan_root, expected, context)?;

  Ok(plan)
}

fn setup_method_plan(
  node: &mut ExecutionPlanNode,
  expected: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<()> {
  let mut method_container = ExecutionPlanNode::container("method");

  let mut match_method = ExecutionPlanNode::action("match:equality");
  match_method
    .add(ExecutionPlanNode::action("upper-case")
      .add(ExecutionPlanNode::resolve_value(DocPath::new("$.method")?)))
    .add(ExecutionPlanNode::value(expected.method.as_str()));

  // TODO: Look at the matching rules and generators here
  method_container.add(match_method);

  node.add(method_container);

  Ok(())
}

pub fn execute_request_plan(
  plan: &ExecutionPlan,
  actual: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  Ok(ExecutionPlan::default())
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use serde_json::json;
  use pact_models::bodies::OptionalBody;
  use pact_models::v4::http_parts::HttpRequest;
  use pretty_assertions::assert_eq;

  use crate::engine::{build_request_plan, execute_request_plan, ExecutionPlan, PlanMatchingContext};

  #[test]
  fn simple_match_request_test() -> anyhow::Result<()> {
    let request = HttpRequest {
      method: "POST".to_string(),
      path: "/test".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::from(&json!({
        "b": "22"
      })),
      matching_rules: Default::default(),
      generators: Default::default(),
    };
    let expected_request = HttpRequest {
      method: "POST".to_string(),
      path: "/test".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::from(&json!({
        "a": 100,
        "b": 200.1
      })),
      matching_rules: Default::default(),
      generators: Default::default(),
    };
    let context = PlanMatchingContext::default();
    let plan = build_request_plan(&expected_request, &context)?;

    assert_eq!(plan.pretty_form(),
r#"(
  :request (
    :method (
      %match:equality (
        %upper-case (
          $.method
        ),
        "POST"
      )
    ),
    :path (
      %match:equality (
        $.path,
        "/test"
      )
    ),
    :"query parameters" (
      %expect:empty (
        $.query
      )
    ),
    :body (
      %if (
        %match:equality (
          %content-type (),
          "application/json"
        ),
        :body:$ (
          :body:$:a (
            %if (
              %expect:present ($.body."$.a"),
              %match:equality ($.body."$.a", 100)
            )
          ),
          :body:$:b (
            %if (
              %expect:present ($.body."$.b"),
              %match:equality ($.body."$.b", 200.1)
            )
          )
        )
      )
    )
  )
)
"#);

    let executed_plan = execute_request_plan(&plan, &request, &context)?;
    assert_eq!(executed_plan.pretty_form(), r#"(
      :request (
        :method (
          %match:equality (
            %upper-case (
              $.method ~ "GET"
            ),
            "POST" ~ OK
          )
        ),
        :path (
          %match:equality ($.path ~ "/test", "/test") ~ OK
        ),
        :"query parameters" (
          %expect:empty ($.query ~ {}) ~ OK
        ),
        :body (
          %if (
            %match:equality (%content-type () ~ "application/json", "application/json") ~ OK,
            :body:$ (
              :body:$:a (
                %if (
                  %expect:present ($.body."$.a" ~ NULL) ~ ERROR(Expected attribute "$.a" but it was missing),
                  %match:equality ($.body."$.a", 100) ~ NULL
                )
              ),
              :body:$:b (
                %if (
                  %expect:present ($.body."$.b" ~ "22") ~ OK,
                  %match:equality ($.body."$.b" ~ "22", 200.1) ~ ERROR(Expected attribute "$.b" to equal "22" (String) but it was 200.1 (Double))
                )
              )
            )
          )
        )
      )
    )
    "#);

    Ok(())
  }
}
