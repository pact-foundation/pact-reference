//! This module provides the interpreter that can execute a matching plan AST

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::iter::once;
use std::time::Instant;

use anyhow::anyhow;
use itertools::Itertools;
use maplit::hashset;
use serde_json::{json, Value};
use tracing::{debug, error, instrument, Level, trace};

use itertools::Either;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::{DocPath, PathToken};
#[cfg(feature = "xml")] use pact_models::xml_utils::resolve_matching_node;

use crate::engine::{ExecutionPlan, ExecutionPlanNode, NodeResult, NodeValue, PlanNodeType};
use crate::engine::context::PlanMatchingContext;
use crate::engine::value_resolvers::ValueResolver;
#[cfg(feature = "xml")] use crate::engine::xml::XmlValue;
use crate::headers::{parse_charset_parameters, strip_whitespace};
use crate::json::type_of;
use crate::matchingrules::DoMatch;
#[cfg(feature = "xml")] use crate::xml::resolve_attr_namespaces;

/// Main interpreter for the matching plan AST
#[derive(Debug)]
pub struct ExecutionPlanInterpreter {
  /// Stack of intermediate values (used by the pipeline operator and apply action)
  value_stack: Vec<Option<NodeResult>>,
  /// Context to use to execute the plan
  context: PlanMatchingContext
}

impl ExecutionPlanInterpreter {
  /// Creates a new interpreter
  #[allow(dead_code)]
  pub fn new() -> Self {
    ExecutionPlanInterpreter {
      value_stack: vec![],
      context: PlanMatchingContext::default()
    }
  }

  /// Creates a new interpreter with a given test context
  pub fn new_with_context(context: &PlanMatchingContext) -> Self {
    ExecutionPlanInterpreter {
      value_stack: vec![],
      context: context.clone()
    }
  }

  /// Executes the given plan using the provided value resolver
  pub fn execute_plan(
    &mut self,
    plan: &ExecutionPlan,
    value_resolver: &dyn ValueResolver
  ) -> anyhow::Result<ExecutionPlan> {
    let path = vec![];
    let start = Instant::now();
    let executed_tree = self.walk_tree(&path, plan.plan_root.clone(), value_resolver)?;
    Ok(ExecutionPlan {
      plan_root: executed_tree,
      execution_time: Some(start.elapsed())
    })
  }

  /// Walks the tree from a given node, executing all visited nodes
  pub fn walk_tree(
    &mut self,
    path: &[String],
    node: ExecutionPlanNode,
    value_resolver: &dyn ValueResolver
  ) -> anyhow::Result<ExecutionPlanNode> {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match node_type {
      PlanNodeType::EMPTY => {
        trace!(?path, "walk_tree ==> Empty node");
        Ok(ExecutionPlanNode { node_type: PlanNodeType::EMPTY, result: None, children })
      },
      PlanNodeType::CONTAINER(ref label) => {
        trace!(?path, label, "walk_tree ==> Container node");

        let mut result_children = vec![];
        let mut child_path = path.to_vec();
        child_path.push(label.clone());
        let mut status = NodeResult::OK;
        let mut loop_items: VecDeque<ExecutionPlanNode> = children.into_iter().collect();

        while let Some(child) = loop_items.pop_front() {
          let child_result = self.walk_tree(&child_path, child, value_resolver)?;
          status = status.and(child_result.result.as_ref().unwrap_or(&NodeResult::OK));
          let is_splat = child_result.is_splat();
          result_children.push(child_result);
          if is_splat {
            let last = result_children.last().unwrap();
            for item in last.children.iter().rev() {
              loop_items.push_front(item.clone());
            }
          }
        }

        Ok(ExecutionPlanNode {
          node_type,
          result: Some(status.truthy()),
          children: result_children
        })
      }
      PlanNodeType::ACTION(ref action) => {
        trace!(?path, action, "walk_tree ==> Action node");
        let action_str = action.clone();
        let node = ExecutionPlanNode { node_type, children, result: None };
        Ok(self.execute_action(&action_str, value_resolver, node, path))
      }
      PlanNodeType::VALUE(ref val) => {
        trace!(?path, ?val, "walk_tree ==> Value node");
        let value = match val {
          NodeValue::NAMESPACED(namespace, value) => match namespace.as_str() {
            "json" => serde_json::from_str(value.as_str())
              .map(NodeValue::JSON)
              .map_err(|err| anyhow!(err)),
            #[cfg(feature = "xml")]
            "xml" => kiss_xml::parse_str(value)
              .map(|doc| NodeValue::XML(XmlValue::Element(doc.root_element().clone())))
              .map_err(|err| anyhow!("Failed to parse XML value: {}", err)),
            _ => Err(anyhow!("'{}' is not a known namespace", namespace))
          }
          _ => Ok(val.clone())
        }?;
        Ok(ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::VALUE(value)),
          children: vec![]
        })
      }
      PlanNodeType::RESOLVE(ref resolve_path) => {
        trace!(?path, %resolve_path, "walk_tree ==> Resolve node");
        match value_resolver.resolve(resolve_path, &self.context) {
          Ok(val) => {
            Ok(ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::VALUE(val)),
              children: vec![]
            })
          }
          Err(err) => {
            trace!(?path, %resolve_path, %err, "Resolve node failed");
            Ok(ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![]
            })
          }
        }
      }
      PlanNodeType::PIPELINE => {
        trace!(?path, "walk_tree ==> Apply pipeline node");

        let child_path = path.to_vec();
        self.push_result(None);
        let mut child_results = vec![];
        let mut loop_items: VecDeque<ExecutionPlanNode> = children.into_iter().collect();

        // TODO: Need a short circuit here if any child results in an error
        while let Some(child) = loop_items.pop_front() {
          let child_result = self.walk_tree(&child_path, child, value_resolver)?;
          self.update_result(child_result.result.clone());
          let is_splat = child_result.is_splat();
          child_results.push(child_result);
          if is_splat {
            let last = child_results.last().unwrap();
            for item in last.children.iter().rev() {
              loop_items.push_front(item.clone());
            }
          }
        }

        let result = self.pop_result();
        let result = match result {
          Some(value) => value,
          None => {
            trace!(?path, "Value from stack is empty");
            NodeResult::ERROR("Value from stack is empty".to_string())
          }
        };
        Ok(ExecutionPlanNode {
          node_type,
          result: Some(result),
          children: child_results
        })
      }
      PlanNodeType::RESOLVE_CURRENT(ref expression) => {
        trace!(?path, %expression, "walk_tree ==> Resolve current node");
        match self.resolve_stack_value(expression) {
          Ok(val) => {
            Ok(ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::VALUE(val)),
              children: vec![]
            })
          }
          Err(err) => {
            debug!(?path, %expression, %err, "Resolve node failed");
            Ok(ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![]
            })
          }
        }
      }
      PlanNodeType::SPLAT => {
        trace!(?path, "walk_tree ==> Apply splat node");

        let child_path = path.to_vec();
        let mut child_results = vec![];

        // TODO: Need a short circuit here if any child results in an error
        for child in children {
          let child_result = self.walk_tree(&child_path, child, value_resolver)?;
          match &child_result.result {
            None => child_results.push(child_result),
            Some(result) => match result {
              NodeResult::OK => child_results.push(child_result),
              NodeResult::VALUE(value) => match value {
                NodeValue::MMAP(map) => {
                  for (key, value) in map.iter() {
                    child_results.push(child_result.clone_with_value(NodeValue::ENTRY(key.clone(), Box::new(NodeValue::SLIST(value.clone())))));
                  }
                }
                NodeValue::SLIST(list) => {
                  for item in list.iter() {
                    child_results.push(child_result.clone_with_value(NodeValue::STRING(item.clone())));
                  }
                }
                _ => child_results.push(child_result)
              }
              NodeResult::ERROR(_) => child_results.push(child_result)
            }
          }
        }

        Ok(ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::OK),
          children: child_results
        })
      }
      PlanNodeType::ANNOTATION(_) => {
        Ok(ExecutionPlanNode { node_type, result: None, children })
      }
    }
  }

  /// Execute the action
  #[instrument(ret, skip_all, level = Level::TRACE, fields(action, path, node))]
  pub fn execute_action(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    path: &[String]
  ) -> ExecutionPlanNode {
    trace!(%action, "Executing action");

    let mut action_path = path.to_vec();
    action_path.push(action.to_string());

    if action == "match:each-key" {
      self.execute_match_each_key(value_resolver, node, &action_path)
    } else if action == "match:each-value" {
      self.execute_match_each_value(value_resolver, node, &action_path)
    } else if action == "match:values" {
      self.execute_match_values(value_resolver, node, &action_path)
    } else if action.starts_with("match:") {
      match action.strip_prefix("match:") {
        None => {
          let ExecutionPlanNode { node_type, children, .. } = node;
          ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(format!("'{}' is not a valid action", action))),
            children
          }
        }
        Some(matcher) => self.execute_match(action, matcher, value_resolver, node, &action_path)
          .unwrap_or_else(|node| node)
      }
    } else {
      match action {
        "upper-case" => self.execute_change_case(action, value_resolver, node, &action_path, true),
        "lower-case" => self.execute_change_case(action, value_resolver, node, &action_path, false),
        "to-string" => self.execute_to_string(action, value_resolver, node, &action_path),
        "length" => self.execute_length(action, value_resolver, node, &action_path),
        "expect:empty" => self.execute_expect_empty(action, value_resolver, node, &action_path),
        "convert:UTF8" => self.execute_convert_utf8(action, value_resolver, node, &action_path),
        "if" => self.execute_if(value_resolver, node, &action_path),
        "and" => self.execute_and(value_resolver, node, &action_path),
        "or" => self.execute_or(value_resolver, node, &action_path),
        "tee" => self.execute_tee(value_resolver, node, &action_path),
        "apply" => self.execute_apply(node),
        "json:parse" => self.execute_json_parse(action, value_resolver, node, &action_path),
        "form:parse" => self.execute_form_parse(action, value_resolver, node, &action_path),
        #[cfg(feature = "xml")]
        "xml:parse" => self.execute_xml_parse(action, value_resolver, node, &action_path),
        #[cfg(feature = "xml")]
        "xml:value" => self.execute_xml_value(action, value_resolver, node, &action_path),
        #[cfg(feature = "xml")]
        "xml:attributes" => self.execute_xml_attributes(action, value_resolver, node, &action_path),
        "json:expect:empty" => self.execute_json_expect_empty(action, value_resolver, node, &action_path, true),
        "json:expect:not-empty" => self.execute_json_expect_empty(action, value_resolver, node, &action_path, false),
        "json:match:length" => self.execute_json_match_length(action, value_resolver, node, &action_path),
        "json:expect:entries" => self.execute_json_expect_entries(action, value_resolver, node, &action_path),
        "check:exists" => self.execute_check_exists(action, value_resolver, node, &action_path),
        "expect:entries" => self.execute_check_entries(action, value_resolver, node, &action_path),
        "expect:only-entries" => self.execute_check_entries(action, value_resolver, node, &action_path),
        "expect:count" => self.execute_expect_count(action, value_resolver, node, &action_path),
        "join" => self.execute_join(action, value_resolver, node, &action_path),
        "join-with" => self.execute_join(action, value_resolver, node, &action_path),
        "error" => self.execute_error(action, value_resolver, node, &action_path),
        "header:parse" => self.execute_header_parse(action, value_resolver, node, &action_path),
        "header:normalize-commas" => self.execute_normalize_comma_whitespace(action, value_resolver, node, &action_path),
        "for-each" => self.execute_for_each(value_resolver, node, &action_path),
        #[cfg(feature = "multipart")]
        "multipart:parse" => self.execute_multipart_parse(action, value_resolver, node, &action_path),
        _ => {
          let ExecutionPlanNode { node_type, children, .. } = node;
          ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(format!("'{}' is not a valid action", action))),
            children
          }
        }
      }
    }
  }

  fn execute_json_expect_entries(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_three_args(children, action, value_resolver, &action_path) {
      Ok((first_node, second_node, third_node)) => {
        let result1 = first_node.value().unwrap_or_default();
        let expected_json_type = match result1.as_string() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("'{}' is not a valid JSON type", result1))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(str) => str
        };
        let result2 = second_node.value().unwrap_or_default();
        let expected_keys = match result2.as_slist() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("'{}' is not a list of Strings", result2))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(list) => list.iter()
            .cloned()
            .collect::<HashSet<_>>()
        };
        let result3 = third_node.value().unwrap_or_default();
        let value = match result3.as_value() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {}", result3))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(value) => value
        };
        let json_value = match &value {
          NodeValue::JSON(json) => json,
          _ => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {:?}", value))),
              children: vec![first_node, second_node, third_node]
            }
          }
        };
        if let Err(err) = json_check_type(expected_json_type, json_value) {
          return ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![first_node, second_node, third_node]
          }
        }

        match json_value {
          Value::Object(o) => {
            let actual_keys = o.keys()
              .cloned()
              .collect::<HashSet<_>>();
            let diff = &expected_keys - &actual_keys;
            if diff.is_empty() {
              ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::VALUE(NodeValue::BOOL(true))),
                children: vec![first_node, second_node, third_node]
              }
            } else {
              ExecutionPlanNode {
                node_type,
                result: Some(
                  NodeResult::ERROR(
                    format!("The following expected entries were missing from the actual Object: {}",
                            diff.iter().join(", "))
                  )
                ),
                children: vec![first_node, second_node, third_node]
              }
            }
          }
          _ => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON Object, but got {:?}", json_value))),
              children: vec![first_node, second_node, third_node]
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_json_match_length(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_three_args(children, action, value_resolver, &action_path) {
      Ok((first_node, second_node, third_node)) => {
        let result1 = first_node.value().unwrap_or_default();
        let expected_json_type = match result1.as_string() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("'{}' is not a valid JSON type", result1))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(str) => str
        };
        let result2 = second_node.value().unwrap_or_default();
        let expected_length = match result2.as_number() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("'{}' is not a valid number", result2))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(length) => length
        };
        let result3 = third_node.value().unwrap_or_default();
        let value = match result3.as_value() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {}", result3))),
              children: vec![first_node, second_node, third_node]
            }
          }
          Some(value) => value
        };
        let json_value = match value {
          NodeValue::JSON(json) => json,
          _ => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {:?}", value))),
              children: vec![first_node, second_node, third_node]
            }
          }
        };
        if let Err(err) = json_check_type(expected_json_type, &json_value) {
          return ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![first_node, second_node, third_node]
          }
        }
        if let Err(err) = json_check_length(expected_length as usize, &json_value) {
          return ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![first_node, second_node, third_node]
          }
        }
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::VALUE(NodeValue::BOOL(true))),
          children: vec![first_node, second_node, third_node]
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_json_expect_empty(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>,
    is_empty: bool
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_two_args(children, action, value_resolver, &action_path) {
      Ok((first_node, second_node)) => {
        let result1 = first_node.value().unwrap_or_default();
        let expected_json_type = match result1.as_string() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("'{}' is not a valid JSON type", result1))),
              children: vec![first_node, second_node]
            }
          }
          Some(str) => str
        };
        let result2 = second_node.value().unwrap_or_default();
        let value = match result2.as_value() {
          None => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {}", result2))),
              children: vec![first_node, second_node]
            }
          }
          Some(value) => value
        };

        let json_value = match value {
          NodeValue::JSON(json) => json,
          _ => {
            return ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(format!("Was expecting a JSON value, but got {:?}", value))),
              children: vec![first_node, second_node]
            }
          }
        };

        if let Err(err) = json_check_type(expected_json_type, &json_value) {
          return ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![first_node, second_node]
          }
        };

        let result = if is_empty {
          match &json_value {
            Value::Null => Ok(NodeResult::VALUE(NodeValue::BOOL(true))),
            Value::String(s) => if s.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON String ({}) to be empty", json_value))
            }
            Value::Array(a) => if a.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON Array ({}) to be empty", json_value))
            }
            Value::Object(o) => if o.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON Object ({}) to be empty", json_value))
            }
            _ => Err(anyhow!("Expected json ({}) to be empty", json_value))
          }
        } else {
          match &json_value {
            Value::Null => Err(anyhow!("Expected JSON value to not be empty but was NULL")),
            Value::String(s) => if !s.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON String ({}) to not be empty", json_value))
            }
            Value::Array(a) => if !a.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON Array ({}) to not be empty", json_value))
            }
            Value::Object(o) => if !o.is_empty() {
              Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
            } else {
              Err(anyhow!("Expected JSON Object ({}) to not be empty", json_value))
            }
            _ => Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
          }
        };

        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![first_node, second_node]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![first_node, second_node]
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_json_parse(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::NULL => Ok(NodeResult::VALUE(NodeValue::NULL)),
            NodeValue::STRING(s) => serde_json::from_str(s.as_str())
              .map(|json| NodeResult::VALUE(NodeValue::JSON(json)))
              .map_err(|err| anyhow!("json parse error - {}", err)),
            NodeValue::BARRAY(b) => serde_json::from_slice(b.as_slice())
              .map(|json| NodeResult::VALUE(NodeValue::JSON(json)))
              .map_err(|err| anyhow!("json parse error - {}", err)),
            _ => Err(anyhow!("json:parse can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::NULL))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_form_parse(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::NULL => Ok(NodeResult::VALUE(NodeValue::NULL)),
            NodeValue::STRING(s) => {
              serde_urlencoded::from_str::<Vec<(String, String)>>(s.as_str())
                .map(|pairs| {
                  let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                  for (k, v) in pairs {
                    map.entry(k).or_default().push(v);
                  }
                  NodeResult::VALUE(NodeValue::MMAP(map))
                })
                .map_err(|err| anyhow!("form:parse error - {}", err))
            }
            NodeValue::BARRAY(b) => {
              serde_urlencoded::from_bytes::<Vec<(String, String)>>(b.as_slice())
                .map(|pairs| {
                  let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                  for (k, v) in pairs {
                    map.entry(k).or_default().push(v);
                  }
                  NodeResult::VALUE(NodeValue::MMAP(map))
                })
                .map_err(|err| anyhow!("form:parse error - {}", err))
            }
            _ => Err(anyhow!("form:parse can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::NULL))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  #[cfg(feature = "multipart")]
  fn execute_multipart_parse(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_two_args(children, action, value_resolver, action_path) {
      Ok((body_node, ct_node)) => {
        let body_value = body_node.value().unwrap_or_default().as_value().unwrap_or_default();
        let ct_value = ct_node.value().unwrap_or_default().as_value().unwrap_or_default();
        let result = match (body_value, ct_value) {
          (NodeValue::BARRAY(body_bytes), NodeValue::STRING(ct_str)) => {
            crate::binary_utils::parse_multipart_body(bytes::Bytes::from(body_bytes), &ct_str)
              .map(|parts| {
                let map: BTreeMap<String, NodeValue> = parts.into_iter()
                  .map(|(name, (data, _mime_ct))| {
                    // Try JSON first; fall back to raw bytes for binary parts
                    let value = serde_json::from_slice::<Value>(data.as_ref())
                      .map(NodeValue::JSON)
                      .unwrap_or_else(|_| NodeValue::BARRAY(data.to_vec()));
                    (name, value)
                  })
                  .collect();
                NodeResult::VALUE(NodeValue::MAP(map))
              })
              .map_err(|err| anyhow!("multipart:parse error - {}", err))
          }
          (body_val, ct_val) => Err(anyhow!(
            "multipart:parse requires BARRAY body and STRING content-type, got {} and {}",
            body_val.value_type(), ct_val.value_type()
          ))
        };
        match result {
          Ok(result) => ExecutionPlanNode {
            node_type,
            result: Some(result),
            children: vec![body_node, ct_node]
          },
          Err(err) => ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![body_node, ct_node]
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  #[cfg(feature = "xml")]
  fn execute_xml_parse(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::NULL => Ok(NodeResult::VALUE(NodeValue::NULL)),
            NodeValue::STRING(s) => {
              kiss_xml::parse_str(s)
                .map(|doc| NodeResult::VALUE(NodeValue::XML(XmlValue::Element(doc.root_element().clone()))))
                .map_err(|err| anyhow!("XML parse error - {}", err))
            }
            NodeValue::BARRAY(b) => {
              kiss_xml::parse_str(String::from_utf8_lossy(b.as_slice()))
                .map(|doc| NodeResult::VALUE(NodeValue::XML(XmlValue::Element(doc.root_element().clone()))))
                .map_err(|err| anyhow!("XML parse error - {}", err))
            }
            _ => Err(anyhow!("xml:parse can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::NULL))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  #[cfg(feature = "xml")]
  fn execute_xml_value(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::XML(xml) => match xml {
              XmlValue::Attribute(_, value) => Ok(NodeResult::VALUE(NodeValue::STRING(value.clone()))),
              _ => Err(anyhow!("xml:value can not be used with {}", xml))
            }
            _ => Err(anyhow!("xml:value can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::NULL))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  #[cfg(feature = "xml")]
  fn execute_xml_attributes(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::XML(xml) => match xml {
              XmlValue::Attribute(name, value) => Ok(NodeResult::VALUE(NodeValue::ENTRY(name.clone(), Box::new(NodeValue::STRING(value.clone()))))),
              XmlValue::Element(element) => {
                let attributes = resolve_attr_namespaces(element);
                Ok(NodeResult::VALUE(NodeValue::MMAP(attributes.iter()
                  .map(|(k, v)| (k.clone(), vec![v.clone()]))
                  .collect())))
              },
              _ => Err(anyhow!("xml:attributes can not be used with {}", xml))
            }
            _ => Err(anyhow!("xml:attributes can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::NULL))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  fn execute_apply(&mut self, node: ExecutionPlanNode) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    if let Some(value) = self.value_stack.last() {
      ExecutionPlanNode {
        node_type,
        result: value.clone(),
        children
      }
    } else {
      ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR("No value to apply (stack is empty)".to_string())),
        children
      }
    }
  }

  fn execute_if(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    if let Some(first_node) = children.first() {
      match self.walk_tree(action_path.as_slice(), first_node.clone(), value_resolver) {
        Ok(first) => {
          let node_result = first.value().unwrap_or_default();
          let mut result_children = children.clone();
          result_children[0] = first.clone();
          if !node_result.is_truthy() {
            if children.len() > 2 {
              match self.walk_tree(action_path.as_slice(), children[2].clone(), value_resolver) {
                Ok(else_node) => {
                  result_children[2] = else_node.clone();
                  ExecutionPlanNode {
                    node_type,
                    result: else_node.result.clone().map(|r| r.truthy()),
                    children: result_children
                  }
                }
                Err(err) => {
                  ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(err.to_string())),
                    children: result_children
                  }
                }
              }
            } else {
              ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::VALUE(NodeValue::BOOL(false))),
                children: result_children
              }
            }
          } else if let Some(second_node) = children.get(1) {
            match self.walk_tree(action_path.as_slice(), second_node.clone(), value_resolver) {
              Ok(second) => {
                let second_result = second.value().unwrap_or_default();
                ExecutionPlanNode {
                  node_type,
                  result: Some(second_result.truthy()),
                  children: vec![first, second].into_iter()
                    .chain(children.into_iter().skip(2))
                    .collect()
                }
              }
              Err(err) => {
                error!("Failed to evaluate the second child - {}", err);
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(false))),
                  children: vec![first, second_node.clone()].into_iter()
                    .chain(children.into_iter().skip(2))
                    .collect()
                }
              }
            }
          } else {
            ExecutionPlanNode {
              node_type,
              result: Some(node_result),
              children: vec![first].into_iter().chain(children.into_iter().skip(1))
                .collect()
            }
          }
        }
        Err(err) => {
          ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children
          }
        }
      }
    } else {
      ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR("'if' action requires at least one argument".to_string())),
        children
      }
    }
  }

  fn execute_tee(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    if let Some(first_node) = children.first() {
      match self.walk_tree(action_path.as_slice(), first_node.clone(), value_resolver) {
        Ok(first) => {
          let first_result = first.value().unwrap_or_default();
          if first_result.is_err() {
            ExecutionPlanNode {
              node_type,
              result: Some(first_result.clone()),
              children: once(first).chain(children.into_iter().skip(1)).collect()
            }
          } else {
            let mut result = NodeResult::OK;
            self.push_result(first.result.clone());
            let mut child_results = vec![first.clone()];
            for child in children.iter().skip(1) {
              match self.walk_tree(&action_path, child.clone(), value_resolver) {
                Ok(value) => {
                  result = result.and(&value.result.clone().unwrap_or_default());
                  child_results.push(value.clone());
                }
                Err(err) => {
                  let node_result = NodeResult::ERROR(err.to_string());
                  result = result.and(&node_result);
                  child_results.push(child.clone_with_result(node_result));
                }
              }
            }

            self.pop_result();
            ExecutionPlanNode {
              node_type,
              result: Some(result.truthy()),
              children: child_results
            }
          }
        }
        Err(err) => ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children
        }
      }
    } else {
      ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::OK),
        children
      }
    }
  }

  fn execute_convert_utf8(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value().unwrap_or_default().as_value();
        let result = if let Some(value) = &arg_value {
          match value {
            NodeValue::NULL => Ok(NodeResult::VALUE(NodeValue::STRING("".to_string()))),
            NodeValue::STRING(s) => Ok(NodeResult::VALUE(NodeValue::STRING(s.clone()))),
            NodeValue::BARRAY(b) => Ok(NodeResult::VALUE(NodeValue::STRING(String::from_utf8_lossy(b).to_string()))),
            _ => Err(anyhow!("convert:UTF8 can not be used with {}", value.value_type()))
          }
        } else {
          Ok(NodeResult::VALUE(NodeValue::STRING("".to_string())))
        };
        match result {
          Ok(result) => {
            ExecutionPlanNode {
              node_type,
              result: Some(result),
              children: vec![value]
            }
          }
          Err(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err.to_string())),
              children: vec![value]
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_expect_empty(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(1, 1, children, action, value_resolver, &action_path) {
      Ok((values, optional)) => {
        let first = values.first().unwrap().value().unwrap_or_default();
        if let NodeResult::ERROR(err) = first  {
          ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: values.iter().chain(optional.iter()).cloned().collect()
          }
        } else {
          let arg_value = first.as_value();
          let result = if let Some(value) = &arg_value {
            match value {
              NodeValue::NULL => Ok(NodeResult::VALUE(NodeValue::BOOL(true))),
              NodeValue::STRING(s) => if s.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {:?} to be empty", value))
              }
              NodeValue::BOOL(b) => Ok(NodeResult::VALUE(NodeValue::BOOL(*b))),
              NodeValue::MMAP(m) => if m.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {} to be empty", value))
              }
              NodeValue::MAP(m) => if m.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {} to be empty", value))
              }
              NodeValue::SLIST(l) => if l.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {} to be empty", value))
              },
              NodeValue::BARRAY(bytes) => if bytes.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected byte array ({} bytes) to be empty", bytes.len()))
              },
              NodeValue::NAMESPACED(_, _) => { todo!("Not Implemented: Need a way to resolve NodeValue::NAMESPACED") }
              NodeValue::UINT(ui) => if *ui == 0 {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {:?} to be empty", value))
              },
              NodeValue::JSON(json) => match json {
                Value::Null => Ok(NodeResult::VALUE(NodeValue::BOOL(true))),
                Value::String(s) => if s.is_empty() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected JSON String ({}) to be empty", json))
                }
                Value::Array(a) => if a.is_empty() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected JSON Array ({}) to be empty", json))
                }
                Value::Object(o) => if o.is_empty() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected JSON Object ({}) to be empty", json))
                }
                _ => Err(anyhow!("Expected json ({}) to be empty", json))
              },
              NodeValue::ENTRY(_, _) =>  Ok(NodeResult::VALUE(NodeValue::BOOL(false))),
              NodeValue::LIST(l) => if l.is_empty() {
                Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
              } else {
                Err(anyhow!("Expected {} to be empty", value))
              },
              #[cfg(feature = "xml")]
              NodeValue::XML(xml) => match xml {
                XmlValue::Element(element) => if element.child_elements().next().is_none() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected {} to be empty", element))
                }
                XmlValue::Text(text) => if text.is_empty() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected {:?} to be empty", value))
                }
                XmlValue::Attribute(name, value) => if value.is_empty() {
                  Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
                } else {
                  Err(anyhow!("Expected {}={} to be empty", name, value))
                }
              }
            }
          } else {
            Ok(NodeResult::VALUE(NodeValue::BOOL(true)))
          };
          match result {
            Ok(result) => {
              ExecutionPlanNode {
                node_type,
                result: Some(result),
                children: values.iter().chain(optional.iter()).cloned().collect()
              }
            }
            Err(err) => {
              debug!("expect:empty failed with an error: {}", err);
              if optional.len() > 0 {
                if let Ok(value) = self.walk_tree(action_path.as_slice(), optional[0].clone(), value_resolver) {
                  let message = value.value().unwrap_or_default().as_string().unwrap_or_default();
                  ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(message)),
                    children: values.iter().chain(once(&value)).cloned().collect()
                  }
                } else {
                  // There was an error generating the optional message, so just return the
                  // original error
                  ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(err.to_string())),
                    children: values.iter().chain(optional.iter()).cloned().collect()
                  }
                }
              } else {
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::ERROR(err.to_string())),
                  children: values.iter().chain(optional.iter()).cloned().collect()
                }
              }
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_match(
    &mut self,
    action: &str,
    matcher: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> Result<ExecutionPlanNode, ExecutionPlanNode> {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(4, 1, children, action, value_resolver, &action_path) {
      Ok((args, optional)) => {
        let first_node = &args[0];
        let second_node = &args[1];
        let third_node = &args[2];
        let fourth_node = &args[3];

        let exepected_value = first_node.value()
          .unwrap_or_default()
          .value_or_error()
          .map_err(|err| {
            ExecutionPlanNode {
              node_type: node_type.clone(),
              result: Some(NodeResult::ERROR(err.to_string())),
              children: args.iter()
                .chain(optional.iter())
                .cloned()
                .collect()
            }
          })?;

        let actual_value = second_node.value()
          .unwrap_or_default()
          .value_or_error()
          .map_err(|err| {
            ExecutionPlanNode {
              node_type: node_type.clone(),
              result: Some(NodeResult::ERROR(err.to_string())),
              children: args.iter()
                .chain(optional.iter())
                .cloned()
                .collect()
            }
          })?;

        let matcher_params = third_node.value()
          .unwrap_or_default()
          .value_or_error()
          .map_err(|err| {
            ExecutionPlanNode {
              node_type: node_type.clone(),
              result: Some(NodeResult::ERROR(err.to_string())),
              children: args.iter()
                .chain(optional.iter())
                .cloned()
                .collect()
            }
          })?
          .as_json()
          .unwrap_or_default();

        let show_types = fourth_node.value()
          .unwrap_or_default()
          .value_or_error()
            .map_err(|err| {
              ExecutionPlanNode {
                node_type: node_type.clone(),
                result: Some(NodeResult::ERROR(err.to_string())),
                children: args.iter()
                  .chain(optional.iter())
                  .cloned()
                  .collect()
              }
            })?
        .as_bool()
        .unwrap_or_default();

        match MatchingRule::create(matcher, &matcher_params) {
          Ok(rule) => {
            match rule.match_value(&exepected_value, &actual_value, false, show_types) {
              Ok(_) => {
                Ok(ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(true))),
                  children: args.iter()
                    .chain(optional.iter())
                    .cloned()
                    .collect()
                })
              }
              Err(err) => {
                if let Some(error_node) = optional.first() {
                  self.push_result(Some(NodeResult::VALUE(NodeValue::STRING(err.to_string()))));
                  match self.walk_tree(action_path.as_slice(), error_node.clone(), value_resolver) {
                    Ok(error_node) => {
                      let message = match error_node.value().unwrap_or_default() {
                        NodeResult::ERROR(e) => e,
                        other => other.as_string().unwrap_or_default()
                      };
                      Err(ExecutionPlanNode {
                        node_type,
                        result: Some(NodeResult::ERROR(message)),
                        children: args.iter().chain(vec![error_node].iter()).cloned().collect()
                      })
                    }
                    Err(_) => {
                      error!("Failed to generate error node - {}", err);
                      // There was an error generating the optional error node, so just return the
                      // original error
                      Err(ExecutionPlanNode {
                        node_type,
                        result: Some(NodeResult::ERROR(err.to_string())),
                        children: args.iter()
                          .chain(optional.iter())
                          .cloned()
                          .collect()
                      })
                    }
                  }
                } else {
                  Err(ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(err.to_string())),
                    children: args.iter()
                      .chain(optional.iter())
                      .cloned()
                      .collect()
                  })
                }
              }
            }
          }
          Err(err) => Err(ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR(err.to_string())),
            children: vec![]
          })
        }
      }
      Err(err) => Err(ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      })
    }
  }

  fn execute_match_values(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(4, 0, children, "match:values", value_resolver, action_path) {
      Ok((args, _)) => {
        let expected = args[0].value().unwrap_or_default().as_value();
        let actual = args[1].value().unwrap_or_default().as_value();
        let ok = match (&expected, &actual) {
          (Some(NodeValue::JSON(Value::Object(_))), Some(NodeValue::JSON(Value::Object(_)))) => true,
          (Some(NodeValue::JSON(Value::Array(_))), Some(NodeValue::JSON(Value::Array(_)))) => true,
          (Some(NodeValue::MMAP(_)), Some(NodeValue::MMAP(_))) => true,
          (Some(NodeValue::NULL), _) | (_, Some(NodeValue::NULL)) => true,
          _ => false
        };
        ExecutionPlanNode {
          node_type,
          result: Some(if ok {
            NodeResult::VALUE(NodeValue::BOOL(true))
          } else {
            NodeResult::ERROR(format!("Expected type {:?} but was {:?}", expected, actual))
          }),
          children: args
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  fn execute_match_each_key(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(4, 0, children, "match:each-key", value_resolver, action_path) {
      Ok((args, _)) => {
        let actual_result = args[1].value().unwrap_or_default();
        let matcher_params = args[2].value().unwrap_or_default()
          .as_value().and_then(|v| v.as_json()).unwrap_or_default();
        let show_types = args[3].value().unwrap_or_default()
          .as_value().and_then(|v| v.as_bool()).unwrap_or_default();

        match MatchingRule::create("each-key", &matcher_params) {
          Ok(MatchingRule::EachKey(def)) => {
            match actual_result.as_value() {
              Some(NodeValue::JSON(Value::Object(map))) => {
                let inner_rules: Vec<MatchingRule> = def.rules.iter()
                  .filter_map(|r| match r { Either::Left(rule) => Some(rule.clone()), _ => None })
                  .collect();
                let mut child_results = args.clone();
                let mut has_error = false;
                for key in map.keys() {
                  let key_node = NodeValue::STRING(key.clone());
                  for rule in &inner_rules {
                    if let Err(e) = rule.match_value(
                      &NodeValue::STRING(String::new()), &key_node, false, show_types
                    ) {
                      has_error = true;
                      child_results.push(ExecutionPlanNode {
                        node_type: PlanNodeType::ACTION(format!("each-key:{}", key)),
                        result: Some(NodeResult::ERROR(e.to_string())),
                        children: vec![]
                      });
                      break;
                    }
                  }
                }
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(!has_error))),
                  children: child_results
                }
              }
              _ => ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::VALUE(NodeValue::BOOL(true))),
                children: args
              }
            }
          }
          Ok(_) | Err(_) => ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR("match:each-key requires an each-key matching rule".to_string())),
            children: args
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  fn execute_match_each_value(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(4, 0, children, "match:each-value", value_resolver, action_path) {
      Ok((args, _)) => {
        let actual_result = args[1].value().unwrap_or_default();
        let matcher_params = args[2].value().unwrap_or_default()
          .as_value().and_then(|v| v.as_json()).unwrap_or_default();
        let show_types = args[3].value().unwrap_or_default()
          .as_value().and_then(|v| v.as_bool()).unwrap_or_default();

        match MatchingRule::create("each-value", &matcher_params) {
          Ok(MatchingRule::EachValue(def)) => {
            match actual_result.as_value() {
              Some(NodeValue::JSON(Value::Object(map))) => {
                let inner_rules: Vec<MatchingRule> = def.rules.iter()
                  .filter_map(|r| match r { Either::Left(rule) => Some(rule.clone()), _ => None })
                  .collect();
                let mut child_results = args.clone();
                let mut has_error = false;
                for (key, val) in map.iter() {
                  let val_node = NodeValue::JSON(val.clone());
                  for rule in &inner_rules {
                    if let Err(e) = rule.match_value(
                      &NodeValue::JSON(val.clone()), &val_node, false, show_types
                    ) {
                      has_error = true;
                      child_results.push(ExecutionPlanNode {
                        node_type: PlanNodeType::ACTION(format!("each-value:{}", key)),
                        result: Some(NodeResult::ERROR(e.to_string())),
                        children: vec![]
                      });
                      break;
                    }
                  }
                }
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(!has_error))),
                  children: child_results
                }
              }
              Some(NodeValue::JSON(Value::Array(items))) => {
                let inner_rules: Vec<MatchingRule> = def.rules.iter()
                  .filter_map(|r| match r { Either::Left(rule) => Some(rule.clone()), _ => None })
                  .collect();
                let mut child_results = args.clone();
                let mut has_error = false;
                for (index, val) in items.iter().enumerate() {
                  let val_node = NodeValue::JSON(val.clone());
                  for rule in &inner_rules {
                    if let Err(e) = rule.match_value(
                      &NodeValue::JSON(val.clone()), &val_node, false, show_types
                    ) {
                      has_error = true;
                      child_results.push(ExecutionPlanNode {
                        node_type: PlanNodeType::ACTION(format!("each-value:{}", index)),
                        result: Some(NodeResult::ERROR(e.to_string())),
                        children: vec![]
                      });
                      break;
                    }
                  }
                }
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(!has_error))),
                  children: child_results
                }
              }
              Some(NodeValue::SLIST(items)) => {
                let inner_rules: Vec<MatchingRule> = def.rules.iter()
                  .filter_map(|r| match r { Either::Left(rule) => Some(rule.clone()), _ => None })
                  .collect();
                let mut child_results = args.clone();
                let mut has_error = false;
                for (index, item) in items.iter().enumerate() {
                  let item_node = NodeValue::STRING(item.clone());
                  for rule in &inner_rules {
                    if let Err(e) = rule.match_value(
                      &NodeValue::STRING(item.clone()), &item_node, false, show_types
                    ) {
                      has_error = true;
                      child_results.push(ExecutionPlanNode {
                        node_type: PlanNodeType::ACTION(format!("each-value:{}", index)),
                        result: Some(NodeResult::ERROR(e.to_string())),
                        children: vec![]
                      });
                      break;
                    }
                  }
                }
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::VALUE(NodeValue::BOOL(!has_error))),
                  children: child_results
                }
              }
              _ => ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::VALUE(NodeValue::BOOL(true))),
                children: args
              }
            }
          }
          Ok(_) | Err(_) => ExecutionPlanNode {
            node_type,
            result: Some(NodeResult::ERROR("match:each-value requires an each-value matching rule".to_string())),
            children: args
          }
        }
      }
      Err(err) => ExecutionPlanNode {
        node_type,
        result: Some(NodeResult::ERROR(err.to_string())),
        children: vec![]
      }
    }
  }

  fn execute_change_case(
    &mut self,
    _action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>,
    upper_case: bool
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    let (result_children, values) = match self.evaluate_children(value_resolver, &node_type, children, action_path, true) {
      Ok(value) => value,
      Err(value) => return value
    };

    let results = values.iter()
      .map(|v| {
        let v = v.as_value().unwrap_or_default();
        if upper_case {
          match v {
            NodeValue::STRING(s) => NodeValue::STRING(s.to_uppercase()),
            NodeValue::SLIST(list) => NodeValue::SLIST(list.iter().map(|s| s.to_uppercase()).collect()),
            NodeValue::JSON(json) => match json {
              Value::String(s) => NodeValue::STRING(s.to_uppercase()),
              _ => NodeValue::STRING(json.to_string())
            }
            _ => v.clone()
          }
        } else {
          match v {
            NodeValue::STRING(s) => NodeValue::STRING(s.to_lowercase()),
            NodeValue::SLIST(list) => NodeValue::SLIST(list.iter().map(|s| s.to_lowercase()).collect()),
            NodeValue::JSON(json) => match json {
              Value::String(s) => NodeValue::STRING(s.to_lowercase()),
              _ => NodeValue::STRING(json.to_string())
            }
            _ => v.clone()
          }
        }
      })
      .collect_vec();
    let result = if results.len() == 1 {
      results[0].clone()
    } else {
      NodeValue::LIST(results)
    };
    ExecutionPlanNode {
      node_type,
      result: Some(NodeResult::VALUE(result)),
      children: result_children
    }
  }

  fn execute_to_string(
    &mut self,
    _action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    let (result_children, values) = match self.evaluate_children(value_resolver, &node_type, children, action_path, true) {
      Ok(value) => value,
      Err(value) => return value
    };

    let results = values.iter()
      .map(|v| {
        let node_value = v.as_value().unwrap_or_default();
        match node_value {
          NodeValue::NULL => NodeValue::STRING(String::default()),
          NodeValue::STRING(s) => NodeValue::STRING(s),
          NodeValue::SLIST(l) => NodeValue::SLIST(l),
          NodeValue::JSON(json) => match json {
            Value::String(s) => NodeValue::STRING(s.clone()),
            _ => NodeValue::STRING(json.to_string())
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(xml) => match xml {
            XmlValue::Element(element) => NodeValue::STRING(element.to_string()),
            XmlValue::Text(text) => NodeValue::STRING(text.clone()),
            XmlValue::Attribute(name, value) => NodeValue::STRING(format!("@{}='{}'", name, value))
          }
          _ => NodeValue::STRING(node_value.str_form())
        }
      })
      .collect_vec();
    let result = if results.len() == 1 {
      results[0].clone()
    } else {
      NodeValue::LIST(results)
    };
    ExecutionPlanNode {
      node_type,
      result: Some(NodeResult::VALUE(result)),
      children: result_children
    }
  }

  fn execute_length(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let result = value.value()
          .unwrap_or_default()
          .as_value()
          .unwrap_or_default();
        let result = match result {
          NodeValue::NULL => NodeResult::VALUE(NodeValue::UINT(0)),
          NodeValue::STRING(s) => NodeResult::VALUE(NodeValue::UINT(s.len() as u64)),
          NodeValue::MMAP(m) => NodeResult::VALUE(NodeValue::UINT(m.len() as u64)),
          NodeValue::SLIST(l) => NodeResult::VALUE(NodeValue::UINT(l.len() as u64)),
          NodeValue::BARRAY(a) => NodeResult::VALUE(NodeValue::UINT(a.len() as u64)),
          NodeValue::JSON(json) => match json {
            Value::String(s) => NodeResult::VALUE(NodeValue::UINT(s.len() as u64)),
            Value::Array(a) => NodeResult::VALUE(NodeValue::UINT(a.len() as u64)),
            Value::Object(m) => NodeResult::VALUE(NodeValue::UINT(m.len() as u64)),
            _ => NodeResult::ERROR(format!("'length' can't be used with a {:?} node", value))
          }
          NodeValue::LIST(l) => NodeResult::VALUE(NodeValue::UINT(l.len() as u64)),
          #[cfg(feature = "xml")]
          NodeValue::XML(xml) => match xml {
            XmlValue::Element(_) => NodeResult::VALUE(NodeValue::UINT(1)),
            XmlValue::Text(text) => NodeResult::VALUE(NodeValue::UINT(text.len() as u64)),
            XmlValue::Attribute(_, _) => NodeResult::VALUE(NodeValue::UINT(1))
          }
          _ => NodeResult::ERROR(format!("'length' can't be used with a {:?} node", value))
        };
        ExecutionPlanNode {
          node_type,
          result: Some(result),
          children: vec![ value.clone() ]
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_check_exists(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let result = if let NodeResult::VALUE(value) = value.value().unwrap_or_default() {
          match value {
            NodeValue::NULL => NodeResult::VALUE(NodeValue::BOOL(false)),
            _ => NodeResult::VALUE(NodeValue::BOOL(true))
          }
        } else {
          NodeResult::VALUE(NodeValue::BOOL(false))
        };
        ExecutionPlanNode {
          node_type,
          result: Some(result),
          children: vec![value]
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  /// Push a result value onto the value stack
  fn push_result(&mut self, value: Option<NodeResult>) {
    self.value_stack.push(value);
  }

  /// Replace the top value of the stack with the new value
  fn update_result(&mut self, value: Option<NodeResult>) {
    if let Some(current) = self.value_stack.last_mut() {
      *current = value;
    } else {
      self.value_stack.push(value);
    }
  }

  /// Return the value on the top if the stack
  fn pop_result(&mut self) -> Option<NodeResult> {
    self.value_stack.pop().flatten()
  }

  /// Return the current stack value
  fn stack_value(&self) -> Option<NodeResult> {
    self.value_stack.last().cloned().flatten()
  }

  fn validate_one_arg(
    &mut self,
    children: Vec<ExecutionPlanNode>,
    action: &str,
    value_resolver: &dyn ValueResolver,
    path: &Vec<String>
  ) -> anyhow::Result<ExecutionPlanNode> {
    if children.len() > 1 {
      Err(anyhow!("{} takes only one argument, got {}", action, children.len()))
    } else {
      let mut iter = children.into_iter();
      if let Some(argument) = iter.next() {
        self.walk_tree(path.as_slice(), argument, value_resolver)
      } else {
        Err(anyhow!("{} requires one argument, got none", action))
      }
    }
  }

  fn validate_two_args(
    &mut self,
    children: Vec<ExecutionPlanNode>,
    action: &str,
    value_resolver: &dyn ValueResolver,
    path: &Vec<String>
  ) -> anyhow::Result<(ExecutionPlanNode, ExecutionPlanNode)> {
    if children.len() == 2 {
      let mut iter = children.into_iter();
      let first = self.walk_tree(path.as_slice(), iter.next().unwrap(), value_resolver)?;
      let second = self.walk_tree(path.as_slice(), iter.next().unwrap(), value_resolver)?;
      Ok((first, second))
    } else {
      Err(anyhow!("Action '{}' requires two arguments, got {}", action, children.len()))
    }
  }

  fn validate_three_args(
    &mut self,
    children: Vec<ExecutionPlanNode>,
    action: &str,
    value_resolver: &dyn ValueResolver,
    path: &Vec<String>
  ) -> anyhow::Result<(ExecutionPlanNode, ExecutionPlanNode, ExecutionPlanNode)> {
    if children.len() == 3 {
      let mut iter = children.into_iter();
      let first = self.walk_tree(path.as_slice(), iter.next().unwrap(), value_resolver)?;
      let second = self.walk_tree(path.as_slice(), iter.next().unwrap(), value_resolver)?;
      let third = self.walk_tree(path.as_slice(), iter.next().unwrap(), value_resolver)?;
      Ok((first, second, third))
    } else {
      Err(anyhow!("Action '{}' requires three arguments, got {}", action, children.len()))
    }
  }

  fn validate_args(
    &mut self,
    required: usize,
    optional: usize,
    children: Vec<ExecutionPlanNode>,
    action: &str,
    value_resolver: &dyn ValueResolver,
    path: &Vec<String>
  ) -> anyhow::Result<(Vec<ExecutionPlanNode>, Vec<ExecutionPlanNode>)> {
    let count = children.len();
    if count < required {
      Err(anyhow!("{} requires {} arguments, got {}", action, required, count))
    } else if count > required + optional {
      Err(anyhow!("{} supports at most {} arguments, got {}", action, required + optional, count))
    } else {
      let mut iter = children.into_iter();
      let mut required_args = vec![];
      for child in iter.by_ref().take(required) {
        let value = self.walk_tree(path.as_slice(), child, value_resolver)?;
        required_args.push(value);
      }
      Ok((required_args, iter.collect()))
    }
  }

  fn execute_join(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    let (result_children, str_values) = match self.evaluate_children(value_resolver, &node_type, children, path, true) {
      Ok((children, values)) => {
        (children, values.iter().flat_map(|v| {
          let v = v.as_value().unwrap_or_default();
          match v {
            NodeValue::STRING(s) => vec![s.clone()],
            NodeValue::BOOL(b) => vec![b.to_string()],
            NodeValue::MMAP(_) => vec![v.str_form()],
            NodeValue::SLIST(list) => list.clone(),
            NodeValue::BARRAY(_) => vec![v.str_form()],
            NodeValue::NAMESPACED(_, _) => vec![v.str_form()],
            NodeValue::UINT(u) => vec![u.to_string()],
            NodeValue::JSON(json) => vec![json.to_string()],
            _ => vec![]
          }
        }).collect_vec())
      },
      Err(value) => return value
    };

    let result = if action == "join-with" && !str_values.is_empty() {
      let first = &str_values[0];
      str_values.iter().dropping(1).join(first.as_str())
    } else {
      str_values.iter().join("")
    };

    ExecutionPlanNode {
      node_type,
      result: Some(NodeResult::VALUE(NodeValue::STRING(result))),
      children: result_children
    }
  }

  fn execute_error(
    &mut self,
    _action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    let (result_children, str_values) = match self.evaluate_children(value_resolver, &node_type, children, path, true) {
      Ok((children, values)) => {
        (children, values.iter().flat_map(|v| {
          let v = v.as_value().unwrap_or_default();
          match v {
            NodeValue::STRING(s) => vec![s.clone()],
            NodeValue::BOOL(b) => vec![b.to_string()],
            NodeValue::MMAP(_) => vec![v.str_form()],
            NodeValue::SLIST(list) => list.clone(),
            NodeValue::BARRAY(_) => vec![v.str_form()],
            NodeValue::NAMESPACED(_, _) => vec![v.str_form()],
            NodeValue::UINT(u) => vec![u.to_string()],
            NodeValue::JSON(json) => vec![json.to_string()],
            _ => vec![]
          }
        }).collect_vec())
      },
      Err(value) => return value
    };

    let result = str_values.iter().join("");
    ExecutionPlanNode {
      node_type,
      result: Some(NodeResult::ERROR(result)),
      children: result_children
    }
  }

  fn evaluate_children(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node_type: &PlanNodeType,
    children: Vec<ExecutionPlanNode>,
    path: &Vec<String>,
    short_circuit: bool
  ) -> Result<(Vec<ExecutionPlanNode>, Vec<NodeResult>), ExecutionPlanNode> {
    let mut result_children = vec![];
    let mut values = vec![];
    let mut loop_items: VecDeque<ExecutionPlanNode> = children.into_iter().collect();

    while let Some(child) = loop_items.pop_front() {
      let value = if let Some(child_value) = child.value() {
        child_value
      } else {
        match self.walk_tree(path.as_slice(), child, value_resolver) {
          Ok(value) => {
            if let Some(NodeResult::ERROR(_)) = &value.result {
              if short_circuit {
                result_children.push(value.clone());
                result_children.extend(loop_items);
                return Err(ExecutionPlanNode {
                  node_type: node_type.clone(),
                  result: value.result.clone(),
                  children: result_children.clone()
                });
              }
            }
            if value.is_splat() {
              for splat_child in value.children.iter().rev() {
                loop_items.push_front(splat_child.clone());
              }
              let v = value.value().unwrap_or_default();
              result_children.push(value);
              v
            } else {
              let v = value.value().unwrap_or_default();
              result_children.push(value);
              v
            }
          },
          Err(err) => {
            return Err(ExecutionPlanNode {
              node_type: node_type.clone(),
              result: Some(NodeResult::ERROR(err.to_string())),
              children: result_children.clone()
            })
          }
        }
      };

      values.push(value);
    }
    Ok((result_children, values))
  }

  fn execute_check_entries(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(2, 1, children, action, value_resolver, &action_path) {
      Ok((values, optional)) => {
        let first = values[0].value()
          .unwrap_or_default()
          .as_value()
          .unwrap_or_default()
          .as_slist()
          .unwrap_or_default();
        let expected_keys = first.iter()
          .cloned()
          .collect::<HashSet<_>>();
        let second = values[1].value()
          .unwrap_or_default()
          .as_value()
          .unwrap_or_default();
        let result = match &second {
          NodeValue::MMAP(map) => {
            let actual_keys = map.keys()
              .cloned()
              .collect::<HashSet<_>>();
            Self::check_diff(action, &expected_keys, &actual_keys)
          }
          NodeValue::MAP(map) => {
            let actual_keys = map.keys()
              .cloned()
              .collect::<HashSet<_>>();
            Self::check_diff(action, &expected_keys, &actual_keys)
          }
          NodeValue::SLIST(list) => {
            let actual_keys = list.iter()
              .cloned()
              .collect::<HashSet<_>>();
            Self::check_diff(action, &expected_keys, &actual_keys)
          }
          NodeValue::STRING(str) => {
            let actual_keys = hashset![str.clone()];
            Self::check_diff(action, &expected_keys, &actual_keys)
          }
          NodeValue::JSON(json) => match json {
            Value::Object(map) => {
              let actual_keys = map.keys()
                .cloned()
                .collect::<HashSet<_>>();
              Self::check_diff(action, &expected_keys, &actual_keys)
            }
            Value::Array(list) => {
              let actual_keys = list.iter()
                .map(|v| v.to_string())
                .collect::<HashSet<_>>();
              Self::check_diff(action, &expected_keys, &actual_keys)
            }
            _ => Err((format!("'{}' can't be used with a {:?} node", action, second), None))
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(xml) => match xml {
            XmlValue::Element(element) => {
              let actual_keys = element.child_elements()
                .map(|child| child.name())
                .collect::<HashSet<_>>();
              Self::check_diff(action, &expected_keys, &actual_keys)
            }
            _ => Err((format!("'{}' can't be used with a {:?} node", action, second), None))
          }
          _ => Err((format!("'{}' can't be used with a {:?} node", action, second), None))
        };

        match result {
          Ok(_) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::OK),
              children: values.iter().chain(optional.iter()).cloned().collect()
            }
          }
          Err((err, diff)) => {
            debug!("expect:empty failed with an error: {}", err);
            if optional.len() > 0 {
              if let Some(diff) = diff {
                self.push_result(Some(NodeResult::VALUE(NodeValue::SLIST(diff.iter().cloned().collect()))));
                let result = if let Ok(value) = self.walk_tree(action_path.as_slice(), optional[0].clone(), value_resolver) {
                  let message = value.value().unwrap_or_default().as_string().unwrap_or_default();
                  ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(message)),
                    children: values.iter().chain(once(&value)).cloned().collect()
                  }
                } else {
                  // There was an error generating the optional message, so just return the
                  // original error
                  ExecutionPlanNode {
                    node_type,
                    result: Some(NodeResult::ERROR(err.to_string())),
                    children: values.iter().chain(optional.iter()).cloned().collect()
                  }
                };
                self.pop_result();
                result
              } else {
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::ERROR(err.to_string())),
                  children: values.iter().chain(optional.iter()).cloned().collect()
                }
              }
            } else {
              ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::ERROR(err.to_string())),
                children: values.iter().chain(optional.iter()).cloned().collect()
              }
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_expect_count(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(2, 1, children, action, value_resolver, &action_path) {
      Ok((values, optional)) => {
        let expected_length = values[0].value()
          .unwrap_or_default()
          .as_value()
          .unwrap_or_default()
          .as_uint()
          .unwrap_or_default() as usize;
        let second = values[1].value()
          .unwrap_or_default()
          .as_value()
          .unwrap_or_default();
        let result = match &second {
          NodeValue::MMAP(map) => {
            if map.len() == expected_length {
              Ok(())
            } else {
              Err(format!("Expected {} map entries but there were {}", expected_length, map.len()))
            }
          }
          NodeValue::SLIST(list) => {
            if list.len() == expected_length {
              Ok(())
            } else {
              Err(format!("Expected {} items but there were {}", expected_length, list.len()))
            }
          }
          NodeValue::STRING(str) => {
            if str.len() == expected_length {
              Ok(())
            } else {
              Err(format!("Expected a string with a length of {} but it was {}", expected_length, str.len()))
            }
          }
          NodeValue::LIST(list) => {
            if list.len() == expected_length {
              Ok(())
            } else {
              Err(format!("Expected {} items but there were {}", expected_length, list.len()))
            }
          }
          NodeValue::JSON(json) => match json {
            Value::Object(map) => {
              if map.len() == expected_length {
                Ok(())
              } else {
                Err(format!("Expected {} object entries but there were {}", expected_length, map.len()))
              }
            }
            Value::Array(list) => {
              if list.len() == expected_length {
                Ok(())
              } else {
                Err(format!("Expected {} array items but there were {}", expected_length, list.len()))
              }
            }
            _ => Err(format!("'{}' can't be used with a {:?} node", action, second))
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(xml) => match xml {
            XmlValue::Element(_) => {
              if expected_length == 1 {
                Ok(())
              } else {
                Err(format!("Expected {} elements but there were 1", expected_length))
              }
            }
            _ => Err(format!("'{}' can't be used with a {:?} node", action, second))
          }
          _ => Err(format!("'{}' can't be used with a {:?} node", action, second))
        };

        match result {
          Ok(_) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::OK),
              children: values.iter().chain(optional.iter()).cloned().collect()
            }
          }
          Err(err) => {
            debug!("expect:count failed with an error: {}", err);
            if optional.len() > 0 {
              if let Ok(value) = self.walk_tree(action_path.as_slice(), optional[0].clone(), value_resolver) {
                let message = value.value().unwrap_or_default().as_string().unwrap_or_default();
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::ERROR(message)),
                  children: values.iter().chain(once(&value)).cloned().collect()
                }
              } else {
                // There was an error generating the optional message, so just return the
                // original error
                ExecutionPlanNode {
                  node_type,
                  result: Some(NodeResult::ERROR(err.to_string())),
                  children: values.iter().chain(optional.iter()).cloned().collect()
                }
              }
            } else {
              ExecutionPlanNode {
                node_type,
                result: Some(NodeResult::ERROR(err.to_string())),
                children: values.iter().chain(optional.iter()).cloned().collect()
              }
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_and(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.evaluate_children(value_resolver, &node_type, children, path, true) {
      Ok((result_children, values)) => {
        ExecutionPlanNode {
          node_type,
          result: Some(values.iter().fold(NodeResult::OK, |result, value| {
            result.and(value)
          })),
          children: result_children
        }
      }
      Err(err) => err
    }
  }

  fn execute_or(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.evaluate_children(value_resolver, &node_type, children, path, false) {
      Ok((result_children, values)) => {
        ExecutionPlanNode {
          node_type,
          result: Some(values.iter().fold(NodeResult::OK, |result, value| {
            result.or(value)
          })),
          children: result_children
        }
      }
      Err(err) => err
    }
  }

  fn check_diff(
    action: &str,
    expected_keys: &HashSet<String>,
    actual_keys: &HashSet<String>
  ) -> Result<(), (String, Option<HashSet<String>>)> {
    match action {
      "expect:entries" => {
        let diff = expected_keys - actual_keys;
        if diff.is_empty() {
          Ok(())
        } else {
          let keys = NodeValue::SLIST(diff.iter().cloned().collect_vec());
          Err((format!("The following expected entries were missing: {}", keys), Some(diff)))
        }
      }
      "expect:only-entries" => {
        let diff = actual_keys - expected_keys;
        if diff.is_empty() {
          Ok(())
        } else {
          let keys = NodeValue::SLIST(diff.iter().cloned().collect_vec());
          Err((format!("The following unexpected entries were received: {}", keys), Some(diff)))
        }
      }
      _ => Err((format!("'{}' is not a valid action", action), None))
    }
  }

  fn execute_header_parse(
    &mut self,
    action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_one_arg(children, action, value_resolver, &action_path) {
      Ok(value) => {
        let arg_value = value.value()
          .unwrap_or_default()
          .as_string()
          .unwrap_or_default();
        let values: Vec<&str> = strip_whitespace(arg_value.as_str(), ";");
        let (header_value, header_params) = values.as_slice()
          .split_first()
          .unwrap_or((&"", &[]));
        let parameter_map = parse_charset_parameters(header_params);

        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::VALUE(NodeValue::JSON(json!({
            "value": header_value,
            "parameters": parameter_map
          })))),
          children: vec![value]
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  fn execute_normalize_comma_whitespace(
    &mut self,
    _action: &str,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    let (result_children, values) = match self.evaluate_children(value_resolver, &node_type, children, action_path, true) {
      Ok(value) => value,
      Err(value) => return value
    };

    let normalize = |s: &str| -> String {
      s.split(',').map(|v| v.trim()).collect::<Vec<_>>().join(",")
    };

    let results = values.iter()
      .map(|v| {
        let v = v.as_value().unwrap_or_default();
        match v {
          NodeValue::STRING(s) => NodeValue::STRING(normalize(&s)),
          NodeValue::SLIST(list) => NodeValue::SLIST(list.iter().map(|s| normalize(s)).collect()),
          _ => v.clone()
        }
      })
      .collect_vec();

    let result = if results.len() == 1 {
      results[0].clone()
    } else {
      NodeValue::LIST(results)
    };

    ExecutionPlanNode {
      node_type,
      result: Some(NodeResult::VALUE(result)),
      children: result_children
    }
  }

  fn execute_for_each(
    &mut self,
    value_resolver: &dyn ValueResolver,
    node: ExecutionPlanNode,
    action_path: &Vec<String>
  ) -> ExecutionPlanNode {
    let ExecutionPlanNode { node_type, children, .. } = node;
    match self.validate_args(2, 1, children, "for-each", value_resolver, &action_path) {
      Ok((values, optional)) => {
        let marker_result = &values[0];
        let loop_items = &values[1];
        let marker = marker_result.value()
          .unwrap_or_default()
          .as_string()
          .unwrap_or_else(|| "[*]".to_string());
        match loop_items.value().unwrap_or_default() {
          NodeResult::ERROR(err) => {
            ExecutionPlanNode {
              node_type,
              result: Some(NodeResult::ERROR(err)),
              children: [marker_result.clone(), loop_items.clone()]
                .iter()
                .chain(optional.iter())
                .cloned()
                .collect()
            }
          }
          _ => {
            let mut result = NodeResult::OK;
            let mut child_results = vec![marker_result.clone(), loop_items.clone()];

            if let Some(template) = optional.first() {
              let loop_items_list = loop_items
                .value()
                .unwrap_or_default()
                .as_value()
                .unwrap_or_default()
                .to_list();
              for (index, _) in loop_items_list.iter().enumerate() {
                let updated_child = inject_index(template, marker.as_str(), index);
                match self.walk_tree(&action_path, updated_child.clone(), value_resolver) {
                  Ok(value) => {
                    result = result.and(&value.result.clone().unwrap_or_default());
                    child_results.push(value.clone());
                  }
                  Err(err) => {
                    let node_result = NodeResult::ERROR(err.to_string());
                    result = result.and(&node_result);
                    child_results.push(updated_child.clone_with_result(node_result));
                  }
                }
              }
            }

            ExecutionPlanNode {
              node_type,
              result: Some(result.truthy()),
              children: child_results
            }
          }
        }
      }
      Err(err) => {
        ExecutionPlanNode {
          node_type,
          result: Some(NodeResult::ERROR(err.to_string())),
          children: vec![]
        }
      }
    }
  }

  #[instrument(ret, skip_all, fields(%path), level = "trace")]
  fn resolve_stack_value(&self, path: &DocPath) -> anyhow::Result<NodeValue> {
    if let Some(result) = self.stack_value() {
      if let NodeResult::VALUE(value) = result {
        match value {
          NodeValue::NULL => {
            Err(anyhow!("Can not resolve '{}', current stack value does not contain a value (is NULL)", path))
          }
          NodeValue::JSON(json) => {
            if path.is_root() {
              Ok(NodeValue::JSON(json))
            } else {
              let json_paths = pact_models::json_utils::resolve_path(&json, path);
              trace!("resolved path {} -> {:?}", path, json_paths);
              if json_paths.is_empty() {
                Ok(NodeValue::NULL)
              } else if json_paths.len() == 1 {
                if let Some(value) = json.pointer(json_paths[0].as_str()) {
                  Ok(NodeValue::JSON(value.clone()))
                } else {
                  Ok(NodeValue::NULL)
                }
              } else {
                let values = json_paths.iter()
                  .map(|path| json.pointer(path.as_str()).cloned().unwrap_or_default())
                  .collect();
                Ok(NodeValue::JSON(Value::Array(values)))
              }
            }
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(value) => {
            if path.is_root() {
              Ok(NodeValue::XML(value.clone()))
            } else if let Some(element) = value.as_element() {
              let xml_paths = pact_models::xml_utils::resolve_path(&element, path);
              trace!("resolved path {} -> {:?}", path, xml_paths);
              if xml_paths.is_empty() {
                Ok(NodeValue::NULL)
              } else if xml_paths.len() == 1 {
                if let Some(value) = resolve_matching_node(&element, xml_paths[0].as_str()) {
                  Ok(NodeValue::XML(value.into()))
                } else {
                  Ok(NodeValue::NULL)
                }
              } else {
                let values = xml_paths.iter()
                  .map(|path| {
                    resolve_matching_node(&element, path.as_str())
                      .map(|node| NodeValue::XML(node.into()))
                      .unwrap_or_default()
                  })
                  .collect();
                Ok(NodeValue::LIST(values))
              }
            } else {
              todo!("Deal with other XML types: {}", value)
            }
          }
          NodeValue::MMAP(map) => {
            if path.is_root() {
              Ok(NodeValue::MMAP(map.clone()))
            } else if let Some(field) = path.first_field() {
              if let Some(values) = map.get(field) {
                if values.len() == 1 {
                  Ok(NodeValue::STRING(values[0].clone()))
                } else {
                  Ok(NodeValue::SLIST(values.clone()))
                }
              } else {
                Ok(NodeValue::NULL)
              }
            } else {
              Err(anyhow!("Can not resolve '{}' from a map value", path))
            }
          }
          NodeValue::MAP(map) => {
            if path.is_root() {
              Ok(NodeValue::MAP(map.clone()))
            } else {
              let path_tokens = path.tokens();
              if let Some(PathToken::Field(part_name)) = path_tokens.get(1) {
                if let Some(value) = map.get(part_name.as_str()) {
                  if path_tokens.len() <= 2 {
                    Ok(value.clone())
                  } else {
                    // Build remaining path (tokens after the part name) and navigate into part value
                    let mut remaining = DocPath::root();
                    for token in path_tokens.iter().skip(2) {
                      remaining.push(token.clone());
                    }
                    match value {
                      NodeValue::JSON(json) => {
                        let json_paths = pact_models::json_utils::resolve_path(json, &remaining);
                        if json_paths.is_empty() {
                          Ok(NodeValue::NULL)
                        } else if json_paths.len() == 1 {
                          if let Some(v) = json.pointer(json_paths[0].as_str()) {
                            Ok(NodeValue::JSON(v.clone()))
                          } else {
                            Ok(NodeValue::NULL)
                          }
                        } else {
                          let values = json_paths.iter()
                            .map(|p| json.pointer(p.as_str()).cloned().unwrap_or_default())
                            .collect();
                          Ok(NodeValue::JSON(Value::Array(values)))
                        }
                      }
                      _ => Ok(NodeValue::NULL)
                    }
                  }
                } else {
                  Ok(NodeValue::NULL)
                }
              } else {
                Err(anyhow!("Can not resolve '{}' from a MAP value", path))
              }
            }
          }
          _ => {
            Err(anyhow!("Can not resolve '{}', current stack value does not contain a value that is resolvable", path))
          }
        }
      } else {
        Err(anyhow!("Can not resolve '{}', current stack value does not contain a value", path))
      }
    } else {
      Err(anyhow!("Can not resolve '{}', current value stack is either empty or contains an empty value", path))
    }
  }
}

fn inject_index(node: &ExecutionPlanNode, marker: &str, index: usize) -> ExecutionPlanNode {
  match &node.node_type {
    PlanNodeType::CONTAINER(label) => {
      if let Ok(path) = DocPath::new(label) {
        ExecutionPlanNode {
          node_type: PlanNodeType::CONTAINER(inject_index_in_path(&path, marker, index).to_string()),
          result: node.result.clone(),
          children: node.children.iter()
            .map(|child| inject_index(child, marker, index))
            .collect()
        }
      } else {
        node.clone_with_children(node.children.iter()
          .map(|child| inject_index(child, marker, index)))
      }
    }
    PlanNodeType::ACTION(_) => node.clone_with_children(node.children.iter()
      .map(|child| inject_index(child, marker, index))),
    PlanNodeType::PIPELINE => node.clone_with_children(node.children.iter()
      .map(|child| inject_index(child, marker, index))),
    PlanNodeType::RESOLVE_CURRENT(exp) => {
      ExecutionPlanNode {
        node_type: PlanNodeType::RESOLVE_CURRENT(inject_index_in_path(exp, marker, index)),
        result: node.result.clone(),
        children: vec![]
      }
    }
    PlanNodeType::SPLAT => node.clone_with_children(node.children.iter()
      .map(|child| inject_index(child, marker, index))),
    _ => node.clone()
  }
}

fn inject_index_in_path(path: &DocPath, marker: &str, index: usize) -> DocPath {
  if path.starts_with("$['$*']") {
    // Special case where for-each is applied at the root
    let tokens = [PathToken::Root, PathToken::Index(index)].iter()
      .chain(path.tokens().iter().dropping(2))
      .flat_map(|token| {
        if let PathToken::Field(name) = token && name == marker {
          vec![PathToken::Field(name.trim_end_matches('*').to_string()), PathToken::Index(index)]
        } else {
          vec![token.clone()]
        }
      })
      .collect_vec();
    DocPath::from_tokens(tokens)
  } else {
    let tokens = path.tokens().iter()
      .flat_map(|token| {
        if let PathToken::Field(name) = token && name == marker {
          vec![PathToken::Field(name.trim_end_matches('*').to_string()), PathToken::Index(index)]
        } else {
          vec![token.clone()]
        }
      })
      .collect_vec();
    DocPath::from_tokens(tokens)
  }
}

fn json_check_length(length: usize, json: &Value) -> anyhow::Result<()> {
  match json {
    Value::Array(a) => if a.len() == length {
      Ok(())
    } else {
      Err(anyhow!("Was expecting a length of {}, but actual length is {}", length, a.len()))
    }
    Value::Object(o) => if o.len() == length {
      Ok(())
    } else {
      Err(anyhow!("Was expecting a length of {}, but actual length is {}", length, o.len()))
    }
    _ => Ok(())
  }
}

fn json_check_type(expected_type: String, json_value: &Value) -> anyhow::Result<()> {
  match expected_type.as_str() {
    "NULL" => json_value.as_null()
      .ok_or_else(|| anyhow!("Was expecting a JSON NULL but got a {}", type_of(&json_value))),
    "BOOL" => json_value.as_bool()
      .ok_or_else(|| anyhow!("Was expecting a JSON Bool but got a {}", type_of(&json_value)))
      .map(|_| ()),
    "NUMBER" => json_value.as_number()
      .ok_or_else(|| anyhow!("Was expecting a JSON Number but got a {}", type_of(&json_value)))
      .map(|_| ()),
    "STRING" => json_value.as_str()
      .ok_or_else(|| anyhow!("Was expecting a JSON String but got a {}", type_of(&json_value)))
      .map(|_| ()),
    "ARRAY" => json_value.as_array()
      .ok_or_else(|| anyhow!("Was expecting a JSON Array but got a {}", type_of(&json_value)))
      .map(|_| ()),
    "OBJECT" => json_value.as_object()
      .ok_or_else(|| anyhow!("Was expecting a JSON Object but got a {}", type_of(&json_value)))
      .map(|_| ()),
    _ => Err(anyhow!("'{}' is not a valid JSON type", expected_type))
  }
}
