//! Structs and traits to support a general matching engine

use std::cell::Cell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use ansi_term::Colour::{Green, Red};
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bytes::Bytes;
use itertools::Itertools;
#[cfg(feature = "xml")] use kiss_xml::dom::Element;
use maplit::hashmap;
use serde_json::Value;
use serde_json::Value::Object;
use tracing::debug;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::headers::PARAMETERISED_HEADERS;
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::{MatchingRule, RuleList, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};

use crate::engine::bodies::{get_body_plan_builder, PlainTextBuilder, PlanBodyBuilder};
use crate::engine::context::PlanMatchingContext;
use crate::engine::interpreter::ExecutionPlanInterpreter;
use crate::engine::value_resolvers::{HttpRequestValueResolver, HttpResponseValueResolver};
#[cfg(feature = "xml")] use crate::engine::xml::XmlValue;
use crate::headers::{parse_charset_parameters, strip_whitespace};
use crate::{BodyMatchResult, Mismatch};
use crate::matchingrules::{DoMatch, value_for_mismatch};
use crate::Mismatch::{BodyMismatch, HeaderMismatch, QueryMismatch};

mod bodies;
mod value_resolvers;
pub mod context;
#[cfg(feature = "xml")] pub mod xml;
mod interpreter;

/// Enum for the type of Plan Node
#[derive(Clone, Debug, Default)]
#[allow(non_camel_case_types)]
pub enum PlanNodeType {
  /// Default plan node is empty
  #[default]
  EMPTY,
  /// Container node with a label
  CONTAINER(String),
  /// Action node with a function reference
  ACTION(String),
  /// Leaf node that contains a value
  VALUE(NodeValue),
  /// Leaf node that stores an expression to resolve against the test context
  RESOLVE(DocPath),
  /// Pipeline node (apply), which applies each node to the next as a pipeline returning the last
  PIPELINE,
  /// Leaf node that stores an expression to resolve against the current stack item
  RESOLVE_CURRENT(DocPath),
  /// Splat node, which executes its children and then replaces itself with the result
  SPLAT,
  /// Annotation node to help with the description of the plan. Not executable.
  ANNOTATION(String),
}

/// Enum for the value stored in a leaf node
#[derive(Clone, Debug, Default, PartialEq)]
pub enum NodeValue {
  /// Default is no value
  #[default]
  NULL,
  /// A string value
  STRING(String),
  /// Boolean value
  BOOL(bool),
  /// Multi-string map (String key to one or more string values)
  MMAP(HashMap<String, Vec<String>>),
  /// List of String values
  SLIST(Vec<String>),
  /// Byte Array
  BARRAY(Vec<u8>),
  /// Namespaced value
  NAMESPACED(String, String),
  /// Unsigned integer
  UINT(u64),
  /// JSON
  JSON(Value),
  /// Key/Value Pair
  ENTRY(String, Box<NodeValue>),
  /// List of values
  LIST(Vec<NodeValue>),
  /// XML
  #[cfg(feature = "xml")]
  XML(XmlValue)
}

impl NodeValue {
  /// Returns the encoded string form of the node value
  pub fn str_form(&self) -> String {
    match self {
      NodeValue::NULL => "NULL".to_string(),
      NodeValue::STRING(str) => {
        escape(str.as_str())
      }
      NodeValue::BOOL(b) => {
        format!("BOOL({})", b)
      }
      NodeValue::MMAP(map) => {
        let mut buffer = String::new();
        buffer.push('{');

        let mut first = true;
        for (key, values) in map.iter().sorted_by(|a, b| Ord::cmp(&a.0, &b.0)) {
          if first {
            first = false;
          } else {
            buffer.push_str(", ");
          }
          buffer.push_str(escape(key.as_str()).as_str());
          if values.is_empty() {
            buffer.push_str(": []");
          } else if values.len() == 1 {
            buffer.push_str(": ");
            buffer.push_str(escape(&values[0]).as_str());
          } else {
            buffer.push_str(": [");
            buffer.push_str(values.iter().map(|v| escape(v)).join(", ").as_str());
            buffer.push(']');
          }
        }

        buffer.push('}');
        buffer
      }
      NodeValue::SLIST(list) => {
        let mut buffer = String::new();
        buffer.push('[');
        buffer.push_str(list.iter().map(|v| escape(v)).join(", ").as_str());
        buffer.push(']');
        buffer
      }
      NodeValue::BARRAY(bytes) => {
        let mut buffer = String::new();
        buffer.push_str("BYTES(");
        buffer.push_str(bytes.len().to_string().as_str());
        buffer.push_str(", ");
        buffer.push_str(BASE64.encode(bytes).as_str());
        buffer.push(')');
        buffer
      }
      NodeValue::NAMESPACED(name, value) => {
        let mut buffer = String::new();
        buffer.push_str(name);
        buffer.push(':');
        buffer.push_str(value);
        buffer
      }
      NodeValue::UINT(i) => format!("UINT({})", i),
      NodeValue::JSON(json) => format!("json:{}", json),
      NodeValue::ENTRY(key, value) => {
        let mut buffer = String::new();
        buffer.push_str(escape(key.as_str()).as_str());
        buffer.push_str(" -> ");
        buffer.push_str(value.str_form().as_str());
        buffer
      }
      NodeValue::LIST(list) => {
        let mut buffer = String::new();
        buffer.push('[');
        buffer.push_str(list.iter().map(|v| v.str_form()).join(", ").as_str());
        buffer.push(']');
        buffer
      }
      #[cfg(feature = "xml")]
      NodeValue::XML(node) => match node {
        XmlValue::Element(element) => format!("xml:{}", escape(element.to_string().as_str())),
        XmlValue::Text(text) => format!("xml:text:{}", escape(text.as_str())),
        XmlValue::Attribute(name, value) => format!("xml:attribute:{}={}",
          escape(name.as_str()), escape(value.as_str()))
      }
    }
  }

  /// Returns the type of the value
  pub fn value_type(&self) -> &str {
    match self {
      NodeValue::NULL => "NULL",
      NodeValue::STRING(_) => "String",
      NodeValue::BOOL(_) => "Boolean",
      NodeValue::MMAP(_) => "Multi-Value String Map",
      NodeValue::SLIST(_) => "String List",
      NodeValue::BARRAY(_) => "Byte Array",
      NodeValue::NAMESPACED(_, _) => "Namespaced Value",
      NodeValue::UINT(_) => "Unsigned Integer",
      NodeValue::JSON(_) => "JSON",
      NodeValue::ENTRY(_, _) => "Entry",
      NodeValue::LIST(_) => "List",
      #[cfg(feature = "xml")]
      NodeValue::XML(_) => "XML"
    }
  }

  /// If this value is a JSON value, returns it, otherwise returns None
  pub fn as_json(&self) -> Option<Value> {
    match self {
      NodeValue::JSON(json) => Some(json.clone()),
      _ => None
    }
  }

  /// If this value is an XML value, returns it, otherwise returns None
  #[cfg(feature = "xml")]
  pub fn as_xml(&self) -> Option<XmlValue> {
    match self {
      NodeValue::XML(xml) => Some(xml.clone()),
      _ => None
    }
  }

  /// If this value is a String value, returns it, otherwise returns None
  pub fn as_string(&self) -> Option<String> {
    match self {
      NodeValue::STRING(s) => Some(s.clone()),
      _ => None
    }
  }

  /// If this value is a bool value, returns it, otherwise returns None
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      NodeValue::BOOL(b) => Some(*b),
      _ => None
    }
  }

  /// If this value is an UInt value, returns it, otherwise returns None
  pub fn as_uint(&self) -> Option<u64> {
    match self {
      NodeValue::UINT(u) => Some(*u),
      _ => None
    }
  }

  /// If this value is a string value, returns it, otherwise returns None
  pub fn as_slist(&self) -> Option<Vec<String>> {
    match self {
      NodeValue::SLIST(list) => Some(list.clone()),
      _ => None
    }
  }

  /// If this value is a byte array, returns it, otherwise returns None
  pub fn as_bytes(&self) -> Option<&Vec<u8>> {
    match self {
      NodeValue::BARRAY(b) => Some(b),
      _ => None
    }
  }

  /// Calculates an AND of two values
  pub fn and(&self, other: &Self) -> Self {
    match self {
      NodeValue::NULL => other.clone(),
      NodeValue::BOOL(b) => NodeValue::BOOL(*b && other.truthy()),
      _ => NodeValue::BOOL(self.truthy() && other.truthy())
    }
  }

  /// Calculates an OR of two values
  pub fn or(&self, other: &Self) -> Self {
    match self {
      NodeValue::NULL => other.clone(),
      NodeValue::BOOL(b) => NodeValue::BOOL(*b || other.truthy()),
      _ => NodeValue::BOOL(self.truthy() || other.truthy())
    }
  }

  /// Convert this value into a boolean using a "truthy" test
  pub fn truthy(&self) -> bool {
    match self {
      NodeValue::STRING(s) => !s.is_empty(),
      NodeValue::BOOL(b) => *b,
      NodeValue::MMAP(m) => !m.is_empty(),
      NodeValue::SLIST(s) => !s.is_empty(),
      NodeValue::BARRAY(b) => !b.is_empty(),
      NodeValue::UINT(u) => *u != 0,
      NodeValue::LIST(l) => !l.is_empty(),
      _ => false
    }
  }

  /// Converts this value into a list of values
  pub fn to_list(&self) -> Vec<NodeValue> {
    match self {
      NodeValue::MMAP(entries) => {
        entries.iter()
          .map(|(k, v)| NodeValue::ENTRY(k.clone(), Box::new(NodeValue::SLIST(v.clone()))))
          .collect()
      }
      NodeValue::SLIST(list) => {
        list.iter()
          .map(|v| NodeValue::STRING(v.clone()))
          .collect()
      }
      NodeValue::JSON(json) => match json {
        Value::Array(a) => {
          a.iter()
            .map(|v| NodeValue::JSON(v.clone()))
            .collect()
        }
        _ => vec![ self.clone() ]
      }
      NodeValue::LIST(list) => list.clone(),
      _ => vec![ self.clone() ]
    }
  }

  /// Returns the inner value as a string
  pub fn inner_str(&self) -> String {
    match self {
      NodeValue::BOOL(inner) => inner.to_string(),
      NodeValue::UINT(inner) => inner.to_string(),
      NodeValue::JSON(inner) => inner.to_string(),
      #[cfg(feature = "xml")] NodeValue::XML(inner) => inner.to_string(),
      _ => self.to_string()
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

impl From<usize> for NodeValue {
  fn from(value: usize) -> Self {
    NodeValue::UINT(value as u64)
  }
}

impl From<u64> for NodeValue {
  fn from(value: u64) -> Self {
    NodeValue::UINT(value)
  }
}

impl From<u16> for NodeValue {
  fn from(value: u16) -> Self {
    NodeValue::UINT(value as u64)
  }
}

impl From<bool> for NodeValue {
  fn from(value: bool) -> Self {
    NodeValue::BOOL(value)
  }
}

impl From<Value> for NodeValue {
  fn from(value: Value) -> Self {
    NodeValue::JSON(value.clone())
  }
}

impl From<&Value> for NodeValue {
  fn from(value: &Value) -> Self {
    NodeValue::JSON(value.clone())
  }
}

impl From<HashMap<&str, Value>> for NodeValue {
  fn from(value: HashMap<&str, Value>) -> Self {
    let json = Object(value.iter().map(|(k, v)| (k.to_string(), v.clone())).collect());
    NodeValue::JSON(json)
  }
}

impl From<Vec<String>> for NodeValue {
  fn from(value: Vec<String>) -> Self {
    NodeValue::SLIST(value)
  }
}

impl From<Vec<&String>> for NodeValue {
  fn from(value: Vec<&String>) -> Self {
    NodeValue::SLIST(value.iter().map(|v| (*v).clone()).collect())
  }
}

#[cfg(feature = "xml")]
impl From<Element> for NodeValue {
  fn from(value: Element) -> Self {
    NodeValue::XML(XmlValue::Element(value.clone()))
  }
}

#[cfg(feature = "xml")]
impl From<&Element> for NodeValue {
  fn from(value: &Element) -> Self {
    NodeValue::XML(XmlValue::Element(value.clone()))
  }
}

impl DoMatch<&NodeValue> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &NodeValue,
    actual_value: &NodeValue,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    match self {
      MatchingRule::Equality | MatchingRule::Values => {
        match expected_value {
          NodeValue::JSON(json) => {
            self.match_value(json, &actual_value.as_json().unwrap_or_default(), cascaded, show_types)
          }
          NodeValue::NULL => {
            if let Some(json) = actual_value.as_json() {
              self.match_value(&Value::Null, &json, cascaded, show_types)
            } else if actual_value == &NodeValue::NULL {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to be equal to {}",
                value_for_mismatch(actual_value.str_form(), actual_value.value_type(), show_types),
                value_for_mismatch(expected_value.str_form(), expected_value.value_type(), show_types)
              ))
            }
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(xml_value) => if let Some(actual) = actual_value.as_xml() {
            self.match_value(xml_value, &actual, cascaded, show_types)
          } else {
            Err(anyhow!("Was expecting an XML value but got {}", actual_value))
          },
          _ => {
            if expected_value == actual_value {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to be equal to {}",
                value_for_mismatch(actual_value.inner_str(), actual_value.value_type(), show_types),
                value_for_mismatch(expected_value.inner_str(), expected_value.value_type(), show_types)
              ))
            }
          }
        }
      }
      MatchingRule::Regex(_) => {
        match actual_value {
          NodeValue::STRING(s) => {
            self.match_value(expected_value.as_string().unwrap_or_default().as_str(), s.as_str(),
              cascaded, show_types)
          }
          NodeValue::BARRAY(bytes) => {
            self.match_value("", String::from_utf8_lossy(bytes.as_slice()).as_ref(),
              cascaded, show_types)
          }
          NodeValue::JSON(json) => {
            match json {
              Value::String(s) => self.match_value("", s.as_str(), cascaded, show_types),
              _ => self.match_value("", json.to_string().as_str(), cascaded, show_types)
            }
          }
          #[cfg(feature = "xml")]
          NodeValue::XML(xml_value) => self.match_value(xml_value, xml_value, cascaded, show_types),
          _ => self.match_value("", actual_value.as_string().unwrap_or_default().as_str(),
            cascaded, show_types)
        }
      }
      MatchingRule::Type | MatchingRule::MinType(_) | MatchingRule::MaxType(_) | MatchingRule::MinMaxType(_, _) => {
        match (expected_value, actual_value) {
          (NodeValue::NULL, NodeValue::NULL) => Ok(()),
          (NodeValue::UINT(_), NodeValue::UINT(_)) => Ok(()),
          (NodeValue::STRING(_), NodeValue::STRING(_)) => Ok(()),
          (NodeValue::ENTRY(_, _), NodeValue::ENTRY(_, _)) => Ok(()),
          (NodeValue::BOOL(_), NodeValue::BOOL(_)) => Ok(()),
          (NodeValue::NAMESPACED(_, _), NodeValue::NAMESPACED(_, _)) => Ok(()),
          (NodeValue::LIST(elist), NodeValue::LIST(alist)) => {
            self.match_value(elist, alist, cascaded, show_types)
          }
          (NodeValue::STRING(elist), NodeValue::SLIST(alist)) => {
            self.match_value(&vec![elist.clone()], alist, cascaded, show_types)
          }
          (NodeValue::SLIST(elist), NodeValue::STRING(alist)) => {
            self.match_value(elist, &vec![alist.clone()], cascaded, show_types)
          }
          (NodeValue::SLIST(elist), NodeValue::SLIST(alist)) => {
            self.match_value(elist, alist, cascaded, show_types)
          }
          #[cfg(feature = "xml")]
          (NodeValue::XML(exml), NodeValue::XML(axml)) => {
            self.match_value(exml, axml, cascaded, show_types)
          }
          (NodeValue::MMAP(emap), NodeValue::MMAP(amap)) => {
            self.match_value(emap, amap, cascaded, show_types)
          }
          (NodeValue::JSON(ejson), NodeValue::JSON(ajson)) => {
            self.match_value(ejson, ajson, cascaded, show_types)
          }
          (NodeValue::BARRAY(e), NodeValue::BARRAY(a)) => {
            self.match_value(Bytes::from(e.clone()), Bytes::from(a.clone()), cascaded, show_types)
          }
          _ => {
            Err(anyhow!("Expected {} ({}) to be the same type as {} ({})", actual_value,
              actual_value.value_type(), expected_value, expected_value.value_type()))
          }
        }
      }
      _ => {
        match (expected_value, actual_value) {
          (NodeValue::STRING(s), NodeValue::SLIST(list)) => {
            let result = list.iter()
              .map(|item| self.match_value(s.as_str(), item.as_str(), cascaded, show_types))
              .filter_map(|r| match r {
                Ok(_) => None,
                Err(err) => Some(err.to_string())
              })
              .collect_vec();
            if result.is_empty() {
              Ok(())
            } else {
              Err(anyhow!(result.join(", ")))
            }
          }
          (NodeValue::STRING(e), NodeValue::STRING(a)) => {
            self.match_value(e.as_str(), a.as_str(), cascaded, show_types)
          }
          (NodeValue::UINT(e), NodeValue::UINT(a)) => {
            self.match_value(*e, *a, cascaded, show_types)
          }
          _ => Err(anyhow!("Matching rules can not be applied to {} values", actual_value.value_type()))
        }
      }
    }
  }
}

impl Display for NodeValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.str_form())
  }
}

/// Enum to store the result of executing a node
#[derive(Clone, Debug, Default, PartialEq)]
pub enum NodeResult {
  /// Default value to make a node as successfully executed
  #[default]
  OK,
  /// Marks a node as successfully executed with a result
  VALUE(NodeValue),
  /// Marks a node as unsuccessfully executed with an error
  ERROR(String)
}

impl NodeResult {
  /// Return the AND of this result with the given one
  pub fn and(&self, option: &NodeResult) -> NodeResult {
    match self {
      NodeResult::OK => match option {
        NodeResult::OK => NodeResult::OK,
        NodeResult::VALUE(_) => option.clone(),
        NodeResult::ERROR(_) => option.clone()
      }
      NodeResult::VALUE(v1) => match option {
        NodeResult::OK => self.clone(),
        NodeResult::VALUE(v2) => NodeResult::VALUE(v1.and(v2)),
        NodeResult::ERROR(_) => option.clone()
      }
      NodeResult::ERROR(_) => self.clone()
    }
  }

  /// Return the OR of this result with the given one
  pub fn or(&self, option: &NodeResult) -> NodeResult {
    match self {
      NodeResult::OK => NodeResult::OK,
      NodeResult::VALUE(v1) => match option {
        NodeResult::OK => self.clone(),
        NodeResult::VALUE(v2) => NodeResult::VALUE(v1.or(v2)),
        NodeResult::ERROR(_) => self.clone()
      }
      NodeResult::ERROR(_) => option.clone()
    }
  }

  /// Converts the result value to a string
  pub fn as_string(&self) -> Option<String> {
    match self {
      NodeResult::OK => None,
      NodeResult::VALUE(val) => match val {
        NodeValue::NULL => Some("".to_string()),
        NodeValue::STRING(s) => Some(s.clone()),
        NodeValue::BOOL(b) => Some(b.to_string()),
        NodeValue::MMAP(m) => Some(format!("{:?}", m)),
        NodeValue::SLIST(list) => Some(format!("{:?}", list)),
        NodeValue::BARRAY(bytes) => Some(BASE64.encode(bytes)),
        NodeValue::NAMESPACED(name, value) => Some(format!("{}:{}", name, value)),
        NodeValue::UINT(ui) => Some(ui.to_string()),
        NodeValue::JSON(json) => Some(json.to_string()),
        NodeValue::ENTRY(k, v) => Some(format!("{} -> {}", k, v)),
        NodeValue::LIST(list) => Some(format!("{:?}", list)),
        #[cfg(feature = "xml")]
        NodeValue::XML(node) => Some(node.to_string())
      }
      NodeResult::ERROR(_) => None
    }
  }

  /// If the result is a number, returns it
  pub fn as_number(&self) -> Option<u64> {
    match self {
      NodeResult::OK => None,
      NodeResult::VALUE(val) => match val {
        NodeValue::UINT(ui) => Some(*ui),
        _ => None
      }
      NodeResult::ERROR(_) => None
    }
  }

  /// Returns the associated value if there is one
  pub fn as_value(&self) -> Option<NodeValue> {
    match self {
      NodeResult::OK => None,
      NodeResult::VALUE(val) => Some(val.clone()),
      NodeResult::ERROR(_) => None
    }
  }

  /// If the result is a list of Strings, returns it
  pub fn as_slist(&self) -> Option<Vec<String>> {
    match self {
      NodeResult::OK => None,
      NodeResult::VALUE(val) => match val {
        NodeValue::SLIST(list) => Some(list.clone()),
        _ => None
      }
      NodeResult::ERROR(_) => None
    }
  }

  /// If this value represents a truthy value (not NULL, false ot empty)
  pub fn is_truthy(&self) -> bool {
    match self {
      NodeResult::OK => true,
      NodeResult::VALUE(v) => match v {
        NodeValue::STRING(s) => !s.is_empty(),
        NodeValue::BOOL(b) => *b,
        NodeValue::MMAP(m) => !m.is_empty(),
        NodeValue::SLIST(l) => !l.is_empty(),
        NodeValue::BARRAY(b) => !b.is_empty(),
        NodeValue::UINT(ui) => *ui != 0,
        NodeValue::LIST(l) => !l.is_empty(),
        _ => false
      }
      NodeResult::ERROR(_) => false
    }
  }

  /// Converts this node result into a truthy value
  pub fn truthy(&self) -> NodeResult {
    NodeResult::VALUE(NodeValue::BOOL(self.is_truthy()))
  }

  /// Unwraps the result into a value, or returns the error results as an error
  pub fn value_or_error(&self) -> anyhow::Result<NodeValue> {
    match self {
      NodeResult::OK => Ok(NodeValue::BOOL(true)),
      NodeResult::VALUE(v) => Ok(v.clone()),
      NodeResult::ERROR(err) => Err(anyhow!(err.clone()))
    }
  }

  /// If the result is an error result
  pub fn is_err(&self) -> bool {
    match self {
      NodeResult::ERROR(_) => true,
      _ => false
    }
  }

  /// If the result is an ok result
  pub fn is_ok(&self) -> bool {
    match self {
      NodeResult::OK => true,
      _ => false
    }
  }
}

impl Display for NodeResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      NodeResult::OK => write!(f, "OK"),
      NodeResult::VALUE(val) => write!(f, "{}", val.str_form()),
      NodeResult::ERROR(err) => write!(f, "ERROR({})", err.replace('(', "\\(")
        .replace(')', "\\)"))
    }
  }
}

/// Terminator for tree transversal
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Terminator {
  /// No termination
  ALL,
  /// Terminate at containers
  CONTAINERS
}

/// Node in an executable plan tree
#[derive(Clone, Debug, Default)]
pub struct ExecutionPlanNode {
  /// Type of the node
  pub node_type: PlanNodeType,
  /// Any result associated with the node
  pub result: Option<NodeResult>,
  /// Child nodes
  pub children: Vec<ExecutionPlanNode>
}

impl ExecutionPlanNode {
  /// Clones this node, replacing the value with the given one
  pub fn clone_with_value(&self, value: NodeValue) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: self.node_type.clone(),
      result: Some(NodeResult::VALUE(value)),
      children: self.children.clone()
    }
  }

  /// Clones this node, replacing the result with the given one
  pub fn clone_with_result(&self, result: NodeResult) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: self.node_type.clone(),
      result: Some(result),
      children: self.children.clone()
    }
  }

  /// Clones this node, replacing the result with the given one
  pub fn clone_with_children<I>(&self, children: I) -> ExecutionPlanNode
    where I: IntoIterator<Item = ExecutionPlanNode> {
    ExecutionPlanNode {
      node_type: self.node_type.clone(),
      result: self.result.clone(),
      children: children.into_iter().collect()
    }
  }

  /// Returns the human-readable text from of the node
  pub fn pretty_form(&self, buffer: &mut String, indent: usize) {
    let pad = " ".repeat(indent);

    match &self.node_type {
      PlanNodeType::EMPTY => {}
      PlanNodeType::CONTAINER(label) => {
        buffer.push_str(pad.as_str());
        buffer.push(':');
        if label.contains(|ch: char| ch.is_whitespace()) {
          buffer.push_str(format!("\"{}\"", label).as_str());
        } else {
          buffer.push_str(label.as_str());
        }
        if self.is_empty() {
          buffer.push_str(" ()");
        } else {
          buffer.push_str(" (\n");
          self.pretty_form_children(buffer, indent);
          buffer.push_str(pad.as_str());
          buffer.push(')');
        }

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::ACTION(value) => {
        buffer.push_str(pad.as_str());
        buffer.push('%');
        buffer.push_str(value.as_str());
        if self.is_empty() {
          buffer.push_str(" ()");
        } else {
          buffer.push_str(" (\n");
          self.pretty_form_children(buffer, indent);
          buffer.push_str(pad.as_str());
          buffer.push(')');
        }

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::VALUE(value) => {
        buffer.push_str(pad.as_str());
        buffer.push_str(value.str_form().as_str());

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::RESOLVE(str) => {
        buffer.push_str(pad.as_str());
        buffer.push_str(str.to_string().as_str());

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::PIPELINE => {
        buffer.push_str(pad.as_str());
        buffer.push_str("->");
        if self.is_empty() {
          buffer.push_str(" ()");
        } else {
          buffer.push_str(" (\n");
          self.pretty_form_children(buffer, indent);
          buffer.push_str(pad.as_str());
          buffer.push(')');
        }

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::RESOLVE_CURRENT(str) => {
        buffer.push_str(pad.as_str());
        buffer.push_str("~>");
        buffer.push_str(str.to_string().as_str());

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::SPLAT => {
        buffer.push_str(pad.as_str());
        buffer.push_str("**");
        if self.is_empty() {
          buffer.push_str(" ()");
        } else {
          buffer.push_str(" (\n");
          self.pretty_form_children(buffer, indent);
          buffer.push_str(pad.as_str());
          buffer.push(')');
        }

        if let Some(result) = &self.result {
          buffer.push_str(" => ");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::ANNOTATION(label) => {
        buffer.push_str(pad.as_str());
        buffer.push_str("#{");
        buffer.push_str(escape(label).as_ref());
        buffer.push('}')
      }
    }
  }

  fn pretty_form_children(&self, buffer: &mut String, indent: usize) {
    let len = self.children.len();
    for (index, child) in self.children.iter().enumerate() {
      child.pretty_form(buffer, indent + 2);
      if index < len - 1 {
        buffer.push(',');
      }
      buffer.push('\n');
    }
  }

  /// Returns the serialised text form of the node
  pub fn str_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push('(');

    match &self.node_type {
      PlanNodeType::EMPTY => {}
      PlanNodeType::CONTAINER(label) => {
        buffer.push(':');
        if label.contains(|ch: char| ch.is_whitespace()) {
          buffer.push_str(format!("\"{}\"", label).as_str());
        } else {
          buffer.push_str(label.as_str());
        }
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::ACTION(value) => {
        buffer.push('%');
        buffer.push_str(value.as_str());
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::VALUE(value) => {
        buffer.push_str(value.str_form().as_str());

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::RESOLVE(str) => {
        buffer.push_str(str.to_string().as_str());

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::PIPELINE => {
        buffer.push_str("->");
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::RESOLVE_CURRENT(str) => {
        buffer.push_str("~>");
        buffer.push_str(str.to_string().as_str());

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::SPLAT => {
        buffer.push_str("**");
        buffer.push('(');
        self.str_form_children(&mut buffer);
        buffer.push(')');

        if let Some(result) = &self.result {
          buffer.push_str("=>");
          buffer.push_str(result.to_string().as_str());
        }
      }
      PlanNodeType::ANNOTATION(label) => {
        buffer.push_str("#{");
        buffer.push_str(escape(label).as_ref());
        buffer.push('}')
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

  /// Constructor for a container node
  pub fn container<S: Into<String>>(label: S) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::CONTAINER(label.into()),
      result: None,
      children: vec![],
    }
  }

  /// Constructor for an action node
  pub fn action(value: &str) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::ACTION(value.to_string()),
      result: None,
      children: vec![],
    }
  }

  /// Constructor for a value node
  pub fn value_node<T: Into<NodeValue>>(value: T) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::VALUE(value.into()),
      result: None,
      children: vec![]
    }
  }

  /// Constructor for a resolve node
  pub fn resolve_value<T: Into<DocPath>>(resolve_str: T) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::RESOLVE(resolve_str.into()),
      result: None,
      children: vec![]
    }
  }

  /// Constructor for a resolve current node
  pub fn resolve_current_value<T: Into<DocPath>>(resolve_str: T) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::RESOLVE_CURRENT(resolve_str.into()),
      result: None,
      children: vec![]
    }
  }

  /// Constructor for an apply node
  pub fn apply() -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::PIPELINE,
      result: None,
      children: vec![],
    }
  }

  /// Constructor for the splat node
  pub fn splat() -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::SPLAT,
      result: None,
      children: vec![]
    }
  }

  fn annotation<S: Into<String>>(description: S) -> ExecutionPlanNode {
    ExecutionPlanNode {
      node_type: PlanNodeType::ANNOTATION(description.into()),
      result: None,
      children: vec![]
    }
  }

  /// Adds the node as a child
  pub fn add<N>(&mut self, node: N) -> &mut Self where N: Into<ExecutionPlanNode> {
    self.children.push(node.into());
    self
  }

  /// Pushes the node onto the front of the list
  pub fn push_node(&mut self, node: ExecutionPlanNode) {
    self.children.insert(0, node.into());
  }

  /// If the node is a leaf node
  pub fn is_empty(&self) -> bool {
    match self.node_type {
      PlanNodeType::EMPTY => true,
      _ => self.children.is_empty()
    }
  }

  /// If the node is a splat node
  pub fn is_splat(&self) -> bool {
    match self.node_type {
      PlanNodeType::SPLAT => true,
      _ => false
    }
  }

  /// Returns the value for the node
  pub fn value(&self) -> Option<NodeResult> {
    self.result.clone()
  }

  /// Return a summary of the execution to display in a console
  pub fn generate_summary(
    &self,
    ansi_color: bool,
    buffer: &mut String,
    indent: usize
  ) {
    let pad = " ".repeat(indent);

    match &self.node_type {
      PlanNodeType::CONTAINER(label) => {
        buffer.push_str(pad.as_str());
        buffer.push_str(label.as_str());
        buffer.push(':');

        if let Some(annotation) = self.annotation_node() {
          buffer.push(' ');
          buffer.push_str(annotation.as_str());
        }

        if let Some(result) = &self.result {
          if self.is_leaf_node() || self.is_terminal_container() {
            if result.is_truthy() {
              if ansi_color {
                buffer.push_str(format!(" - {}", Green.paint("OK")).as_str());
              } else {
                buffer.push_str(" - OK");
              }
            } else {
              let errors = self.child_errors(Terminator::ALL);
              if let NodeResult::ERROR(err) = result {
                if ansi_color {
                  buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(err)).as_str());
                } else {
                  buffer.push_str(format!(" - ERROR {}", err).as_str());
                }
                let error_pad = " ".repeat(indent + label.len() + 2);
                for error in errors {
                  buffer.push('\n');
                  buffer.push_str(error_pad.as_str());
                  if ansi_color {
                    buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(error)).as_str());
                  } else {
                    buffer.push_str(format!(" - ERROR {}", error).as_str());
                  }
                }
              } else if errors.len() == 1 {
                if ansi_color {
                  buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(errors[0].as_str())).as_str());
                } else {
                  buffer.push_str(format!(" - ERROR {}", errors[0]).as_str())
                }
              } else if errors.is_empty() {
                if ansi_color {
                  buffer.push_str(format!(" - {}", Red.paint("FAILED")).as_str());
                } else {
                  buffer.push_str(" - FAILED")
                }
              } else {
                let error_pad = " ".repeat(indent + label.len() + 2);
                for error in errors {
                  buffer.push('\n');
                  buffer.push_str(error_pad.as_str());
                  if ansi_color {
                    buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(error)).as_str());
                  } else {
                    buffer.push_str(format!(" - ERROR {}", error).as_str());
                  }
                }
              }
            }
          } else {
            let errors = self.child_errors(Terminator::CONTAINERS);
            if let NodeResult::ERROR(err) = result {
              if ansi_color {
                buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(err)).as_str());
              } else {
                buffer.push_str(format!(" - ERROR {}", err).as_str());
              }
              let error_pad = " ".repeat(indent + label.len() + 2);
              for error in errors {
                buffer.push('\n');
                buffer.push_str(error_pad.as_str());
                if ansi_color {
                  buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(error)).as_str());
                } else {
                  buffer.push_str(format!(" - ERROR {}", error).as_str());
                }
              }
            } else if errors.len() == 1 {
              if ansi_color {
                buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(errors[0].as_str())).as_str());
              } else {
                buffer.push_str(format!(" - ERROR {}", errors[0]).as_str())
              }
            } else if !errors.is_empty() {
              let error_pad = " ".repeat(indent + label.len() + 2);
              for error in errors {
                buffer.push('\n');
                buffer.push_str(error_pad.as_str());
                if ansi_color {
                  buffer.push_str(format!(" - {} {}", Red.paint("ERROR"), Red.paint(error)).as_str());
                } else {
                  buffer.push_str(format!(" - ERROR {}", error).as_str());
                }
              }
            }
          }
        }

        buffer.push('\n');

        self.generate_children_summary(ansi_color, buffer, indent + 2);
      }
      _ => self.generate_children_summary(ansi_color, buffer, indent)
    }
  }

  fn generate_children_summary(&self, ansi_color: bool, buffer: &mut String, indent: usize) {
    for child in &self.children {
      child.generate_summary(ansi_color, buffer, indent);
    }
  }

  fn annotation_node(&self) -> Option<String> {
    self.children.iter().find_map(|child| {
      if let PlanNodeType::ANNOTATION(annotation) = &child.node_type {
        Some(annotation.clone())
      } else {
        None
      }
    })
  }

  /// If this node has no children
  pub fn is_leaf_node(&self) -> bool {
    self.children.is_empty()
  }

  /// If this node is a container and there are no more containers in the subtree
  pub fn is_terminal_container(&self) -> bool {
    self.is_container() && !self.has_child_containers()
  }

  /// If this node is a container
  pub fn is_container(&self) -> bool {
    if let PlanNodeType::CONTAINER(_) = &self.node_type {
      true
    } else {
      false
    }
  }

  /// If this node has any containers in the subtree
  pub fn has_child_containers(&self) -> bool {
    self.children.iter().any(|child| {
      if let PlanNodeType::CONTAINER(_) = &child.node_type {
        true
      } else {
        child.has_child_containers()
      }
    })
  }

  /// Returns all the errors from the child nodes, either terminating at any child containers
  /// ot just returning all errors.
  pub fn child_errors(&self, terminator: Terminator) -> Vec<String> {
    let mut errors = vec![];
    for child in &self.children {
      if child.is_container() && terminator == Terminator::ALL || !child.is_container() {
        if let Some(NodeResult::ERROR(error)) = &child.result {
          errors.push(error.clone());
        }
        errors.extend_from_slice(child.child_errors(terminator).as_slice());
      }
    }
    errors
  }

  /// Returns the first error found from this node stopping at child containers
  pub fn error(&self) -> Option<String> {
    if let Some(NodeResult::ERROR(err)) = &self.result {
      Some(err.clone())
    } else {
      self.child_errors(Terminator::CONTAINERS).first().cloned()
    }
  }

  /// Returns all the errors found from this node
  pub fn errors(&self) -> Vec<String> {
    let mut errors = vec![];
    if let Some(NodeResult::ERROR(err)) = &self.result {
      errors.push(err.clone());
    }
    errors.extend_from_slice(&self.child_errors(Terminator::ALL));
    errors
  }

  /// Walks the tree to return any node that matches the given path starting from this node
  pub fn fetch_node(&self, path: &[&str]) -> Option<ExecutionPlanNode> {
    if path.is_empty() {
      None
    } else if self.matches(path[0]) {
      if path.len() > 1 {
        self.children.iter().find_map(|child| child.fetch_node(&path[1..]))
      } else {
        Some(self.clone())
      }
    } else {
      None
    }
  }

  /// Walks the tree to return any node that matches the given path starting from this nodes children
  pub fn fetch_child_node(&self, path: &[&str]) -> Option<ExecutionPlanNode> {
    self.children.iter().find_map(|child| child.fetch_node(&path))
  }

  fn matches(&self, identifier: &str) -> bool {
    match &self.node_type {
      PlanNodeType::EMPTY => false,
      PlanNodeType::CONTAINER(label) => format!(":{}", label) == identifier,
      PlanNodeType::ACTION(action) => format!("%{}", action) == identifier,
      PlanNodeType::VALUE(_) => false,
      PlanNodeType::RESOLVE(exp) => exp.to_string() == identifier,
      PlanNodeType::PIPELINE => "->" == identifier,
      PlanNodeType::RESOLVE_CURRENT(exp) => format!("~>{}", exp) == identifier,
      PlanNodeType::SPLAT => "**" == identifier,
      PlanNodeType::ANNOTATION(_) => false
    }
  }

  /// This a fold operation over the depth-first transversal of the containers in the tree
  /// from this node.
  pub fn traverse_containers<ACC, F>(&self, acc: ACC, mut callback: F) -> ACC
    where F: FnMut(ACC, String, &ExecutionPlanNode) -> ACC + Clone,
          ACC: Default
  {
    let acc_cell = Cell::new(acc);
    for child in &self.children {
      if let PlanNodeType::CONTAINER(label) = &child.node_type {
        let acc = acc_cell.take();
        let result = callback(acc, label.clone(), child);
        acc_cell.set(result);
      }
      let acc = acc_cell.take();
      let result = child.traverse_containers(acc, callback.clone());
      acc_cell.set(result);
    }
    acc_cell.into_inner()
  }
}

impl From<&mut ExecutionPlanNode> for ExecutionPlanNode {
  fn from(value: &mut ExecutionPlanNode) -> Self {
    value.clone()
  }
}

impl From<anyhow::Error> for ExecutionPlanNode {
  fn from(value: anyhow::Error) -> Self {
    ExecutionPlanNode {
      result: Some(NodeResult::ERROR(value.to_string())),
      .. ExecutionPlanNode::default()
    }
  }
}

/// An executable plan that contains a tree of execution nodes
#[derive(Clone, Debug, Default)]
pub struct ExecutionPlan {
  /// Root node for the plan tree
  pub plan_root: ExecutionPlanNode,
  /// Total execution time
  pub execution_time: Option<Duration>
}

impl ExecutionPlan {
  /// Creates a new empty execution plan with a single root container
  fn new(label: &str) -> ExecutionPlan {
    ExecutionPlan {
      plan_root: ExecutionPlanNode::container(label),
      execution_time: None
    }
  }

  /// Adds the node as the root node if the node is not empty (i.e. not a leaf node).
  pub fn add(&mut self, node: ExecutionPlanNode) {
    if !node.is_empty() {
      self.plan_root.add(node);
    }
  }

  /// Returns the serialised text form of the execution  plan.
  pub fn str_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push('(');
    buffer.push_str(self.plan_root.str_form().as_str());
    buffer.push(')');
    buffer
  }

  /// Returns the human-readable text form of the execution plan.
  pub fn pretty_form(&self) -> String {
    let mut buffer = String::new();
    buffer.push_str("(\n");
    self.plan_root.pretty_form(&mut buffer, 2);
    buffer.push_str("\n)\n");
    buffer
  }

  /// Return a summary of the execution to display in a console
  pub fn generate_summary(&self, ansi_color: bool) -> String {
    let mut buffer = String::new();
    self.plan_root.generate_summary(ansi_color, &mut buffer, 0);
    buffer
  }

  /// Walks the tree to return any node that matches the given path
  pub fn fetch_node(&self, path: &[&str]) -> Option<ExecutionPlanNode> {
    self.plan_root.fetch_node(path)
  }
}

impl From<ExecutionPlanNode> for ExecutionPlan {
  fn from(value: ExecutionPlanNode) -> Self {
    ExecutionPlan {
      plan_root: value,
      execution_time: None
    }
  }
}

impl From<&ExecutionPlanNode> for ExecutionPlan {
  fn from(value: &ExecutionPlanNode) -> Self {
    ExecutionPlan {
      plan_root: value.clone(),
      execution_time: None
    }
  }
}

impl Into<Vec<Mismatch>> for ExecutionPlan {
  fn into(self) -> Vec<Mismatch> {
    let mut result = vec![];

    if let Some(request) = self.fetch_node(&[":request"]) {
      if let Some(mismatch) = method_mismatch(&request) {
        result.push(mismatch);
      }
      if let Some(mismatches) = path_mismatch(&request) {
        result.extend(mismatches);
      }
      let query = query_mismatches(&request);
      for mismatches in query.values() {
        result.extend(mismatches.clone());
      }
      let headers = header_mismatches(&request);
      for mismatches in headers.values() {
        result.extend(mismatches.clone());
      }
      let body = body_mismatches(&request);
      result.extend(body.mismatches());
    }

    if let Some(response) = self.fetch_node(&[":response"]) {
      if let Some(mismatch) = status_mismatch(&response) {
        result.push(mismatch);
      }
      let headers = header_mismatches(&response);
      for mismatches in headers.values() {
        result.extend(mismatches.clone());
      }
      let body = body_mismatches(&response);
      result.extend(body.mismatches());
    }

    result
  }
}

pub(crate) fn method_mismatch(plan: &ExecutionPlanNode) -> Option<Mismatch> {
  let method_node = plan.fetch_node(&[":request", ":method"]).unwrap_or_default();
  let method = method_node.error()
    .map(|err| Mismatch::MethodMismatch {
      expected: "".to_string(),
      actual: "".to_string(),
      mismatch: err
    });
  method
}

pub(crate) fn status_mismatch(plan: &ExecutionPlanNode) -> Option<Mismatch> {
  let node = plan.fetch_node(&[":response", ":status"]).unwrap_or_default();
  node.error()
    .map(|err| Mismatch::StatusMismatch {
      expected: 0,
      actual: 0,
      mismatch: err
    })
}

pub(crate) fn path_mismatch(plan: &ExecutionPlanNode) -> Option<Vec<Mismatch>> {
  let path_node = plan.fetch_node(&[":request", ":path"]).unwrap_or_default();
  let path_errors = path_node.errors().iter()
    .map(|err| Mismatch::PathMismatch {
      expected: "".to_string(),
      actual: "".to_string(),
      mismatch: err.clone()
    }).collect_vec();
  let path = if path_errors.is_empty() {
    None
  } else {
    Some(path_errors)
  };
  path
}

pub(crate) fn query_mismatches(plan: &ExecutionPlanNode) -> HashMap<String, Vec<Mismatch>> {
  let query_node = plan.fetch_node(&[":request", ":query parameters"]).unwrap_or_default();
  let mut query = query_node.children.iter()
    .fold(hashmap! {}, |mut acc, child| {
      if let PlanNodeType::CONTAINER(label) = &child.node_type {
        let mismatches = child.errors().iter().map(|err| QueryMismatch {
          parameter: label.clone(),
          expected: "".to_string(),
          actual: "".to_string(),
          mismatch: err.clone(),
        }).collect_vec();
        acc.insert(label.clone(), mismatches);
      } else {
        let mismatches = child.errors().iter().map(|err| QueryMismatch {
          parameter: "".to_string(),
          expected: "".to_string(),
          actual: "".to_string(),
          mismatch: err.clone(),
        }).collect_vec();
        if !mismatches.is_empty() {
          acc.entry("".to_string())
            .and_modify(|entry| entry.extend_from_slice(&mismatches))
            .or_insert(vec![]);
        }
      };
      acc
    });
  let errors = query_node.child_errors(Terminator::CONTAINERS);
  if !errors.is_empty() {
    let mismatches = errors.iter()
      .map(|err| BodyMismatch {
        path: "".to_string(),
        expected: None,
        actual: None,
        mismatch: err.clone(),
      })
      .collect_vec();
    query.insert("".to_string(), mismatches);
  }
  query
}

pub(crate) fn header_mismatches(plan: &ExecutionPlanNode) -> HashMap<String, Vec<Mismatch>> {
  let headers_node = plan.fetch_child_node(&[":headers"]).unwrap_or_default();
  let mut headers = headers_node.children.iter()
    .fold(hashmap! {}, |mut acc, child| {
      if let PlanNodeType::CONTAINER(label) = &child.node_type {
        let mismatches = child.errors().iter().map(|err| HeaderMismatch {
          key: label.clone(),
          expected: "".to_string(),
          actual: "".to_string(),
          mismatch: err.clone(),
        }).collect_vec();
        acc.insert(label.clone(), mismatches);
      } else {
        let mismatches = child.errors().iter().map(|err| HeaderMismatch {
          key: "".to_string(),
          expected: "".to_string(),
          actual: "".to_string(),
          mismatch: err.clone(),
        }).collect_vec();
        if !mismatches.is_empty() {
          acc.entry("".to_string())
            .and_modify(|entry| entry.extend_from_slice(&mismatches))
            .or_insert(vec![]);
        }
      };
      acc
    });
  let errors = headers_node.child_errors(Terminator::CONTAINERS);
  if !errors.is_empty() {
    let mismatches = errors.iter()
      .map(|err| BodyMismatch {
        path: "".to_string(),
        expected: None,
        actual: None,
        mismatch: err.clone(),
      })
      .collect_vec();
    headers.insert("".to_string(), mismatches);
  }
  headers
}

pub(crate) fn body_mismatches(plan: &ExecutionPlanNode) -> BodyMatchResult {
  let body_node = plan.fetch_child_node(&[":body"]).unwrap_or_default();
  let body = if body_node.clone().result.unwrap_or_default().is_truthy() {
    BodyMatchResult::Ok
  } else if body_node.is_empty() {
    match &body_node.clone().result {
      Some(NodeResult::ERROR(err)) => {
        let mismatch = BodyMismatch {
          path: "".to_string(),
          expected: None,
          actual: None,
          mismatch: err.clone()
        };
        BodyMatchResult::BodyMismatches(hashmap!{"".to_string() => vec![mismatch]})
      }
      _ => BodyMatchResult::Ok
    }
  } else {
    let first_error = body_node.error().unwrap_or_default();
    if first_error.to_lowercase().starts_with("body type error") {
      BodyMatchResult::BodyTypeMismatch {
        expected_type: "".to_string(),
        actual_type: "".to_string(),
        message: first_error.clone(),
        expected: None,
        actual: None,
      }
    } else {
      let mut body_mismatches = body_node.traverse_containers(hashmap! {}, |mut acc, label, node| {
        let errors = node.child_errors(Terminator::CONTAINERS);
        if !errors.is_empty() {
          let mismatches = errors.iter()
            .map(|err| BodyMismatch {
              path: label.clone(),
              expected: None,
              actual: None,
              mismatch: err.clone(),
            })
            .collect_vec();
          acc.insert(label.clone(), mismatches);
        }
        acc
      });
      if !first_error.is_empty() {
        let mismatches = body_mismatches.entry("".to_string())
          .or_insert_with(|| vec![]);
        mismatches.push(BodyMismatch {
          path: "".to_string(),
          expected: None,
          actual: None,
          mismatch: first_error.clone()
        });
      }
      BodyMatchResult::BodyMismatches(body_mismatches)
    }
  };
  body
}

/// Constructs an execution plan for the HTTP request part.
pub fn build_request_plan(
  expected: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  let mut plan = ExecutionPlan::new("request");

  plan.add(setup_method_plan(expected, &context.for_method())?);
  plan.add(setup_path_plan(expected, &context.for_path())?);
  plan.add(setup_query_plan(expected, &context.for_query())?);
  plan.add(setup_header_plan(expected, &context.for_headers())?);
  plan.add(setup_body_plan(expected, &context.for_body())?);

  Ok(plan)
}

fn setup_method_plan(
  expected: &HttpRequest,
  _context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  let mut method_container = ExecutionPlanNode::container("method");

  let mut match_method = ExecutionPlanNode::action("match:equality");
  let expected_method = expected.method.as_str().to_uppercase();
  match_method
    .add(ExecutionPlanNode::value_node(expected_method.clone()))
    .add(ExecutionPlanNode::action("upper-case")
      .add(ExecutionPlanNode::resolve_value(DocPath::new("$.method")?)))
    .add(ExecutionPlanNode::value_node(NodeValue::NULL))
    .add(ExecutionPlanNode::value_node(NodeValue::BOOL(false)));

  method_container.add(ExecutionPlanNode::annotation(format!("method == {}", expected_method)));
  method_container.add(match_method);

  Ok(method_container)
}

fn setup_path_plan(
  expected: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  let mut plan_node = ExecutionPlanNode::container("path");
  let expected_node = ExecutionPlanNode::value_node(expected.path.as_str());
  let doc_path = DocPath::new("$.path")?;
  let actual_node = ExecutionPlanNode::resolve_value(doc_path.clone());
  if context.matcher_is_defined(&doc_path) {
    let matchers = context.select_best_matcher(&doc_path);
    plan_node.add(ExecutionPlanNode::annotation(format!("path {}", matchers.generate_description(false))));
    plan_node.add(build_matching_rule_node(&expected_node, &actual_node, &matchers,
      false, context.config.show_types_in_errors));
  } else {
    plan_node.add(ExecutionPlanNode::annotation(format!("path == '{}'", expected.path)));
    plan_node
      .add(
        ExecutionPlanNode::action("match:equality")
          .add(expected_node)
          .add(ExecutionPlanNode::resolve_value(doc_path))
          .add(ExecutionPlanNode::value_node(NodeValue::NULL))
          .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors))
      );
  }
  Ok(plan_node)
}

fn build_matching_rule_node(
  expected_node: &ExecutionPlanNode,
  actual_node: &ExecutionPlanNode,
  matchers: &RuleList,
  for_collection: bool,
  show_types: bool
) -> ExecutionPlanNode {
  if matchers.rules.len() == 1 {
    let matcher = if for_collection {
      matchers.rules[0].clone()
    } else {
      matchers.rules[0].for_single_item()
    };
    let mut plan_node = ExecutionPlanNode::action(format!("match:{}", matcher.name()).as_str());
    plan_node
      .add(expected_node.clone())
      .add(actual_node.clone())
      .add(ExecutionPlanNode::value_node(matcher.values()))
      .add(ExecutionPlanNode::value_node(show_types));
    plan_node
  } else {
    let mut logic_node = match matchers.rule_logic {
      RuleLogic::And => ExecutionPlanNode::action("and"),
      RuleLogic::Or => ExecutionPlanNode::action("or")
    };
    for rule in &matchers.rules {
      let matcher = if for_collection {
        rule.clone()
      } else {
        rule.for_single_item()
      };
      logic_node
        .add(
          ExecutionPlanNode::action(format!("match:{}", matcher.name()).as_str())
            .add(expected_node.clone())
            .add(actual_node.clone())
            .add(ExecutionPlanNode::value_node(matcher.values()))
            .add(ExecutionPlanNode::value_node(show_types))
        );
    }
    logic_node
  }
}

fn setup_query_plan(
  expected: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  let mut plan_node = ExecutionPlanNode::container("query parameters");
  let doc_path = DocPath::new("$.query")?;

  if let Some(query) = &expected.query {
    if query.is_empty() {
      plan_node
        .add(
          ExecutionPlanNode::action("expect:empty")
            .add(ExecutionPlanNode::resolve_value(doc_path.clone()))
            .add(
              ExecutionPlanNode::action("join")
                .add(ExecutionPlanNode::value_node("Expected no query parameters but got "))
                .add(ExecutionPlanNode::resolve_value(doc_path))
            )
        );
    } else {
      let keys = query.keys().cloned().sorted().collect_vec();
      for key in &keys {
        let value = query.get(key).unwrap();
        let mut item_node = ExecutionPlanNode::container(key);

        let mut presence_check = ExecutionPlanNode::action("if");
        let item_value = if value.len() == 1 {
          NodeValue::STRING(value[0].clone().unwrap_or_default())
        } else {
          NodeValue::SLIST(value.iter()
            .map(|v| v.clone().unwrap_or_default()).collect())
        };
        presence_check
          .add(
            ExecutionPlanNode::action("check:exists")
              .add(ExecutionPlanNode::resolve_value(doc_path.join(key)))
          );

        let item_path = DocPath::root().join(key);
        let path = doc_path.join(key);
        if context.matcher_is_defined(&item_path) {
          let matchers = context.select_best_matcher(&item_path);
          item_node.add(ExecutionPlanNode::annotation(format!("{} {}", key, matchers.generate_description(true))));
          presence_check.add(build_matching_rule_node(&ExecutionPlanNode::value_node(item_value),
            &ExecutionPlanNode::resolve_value(&path), &matchers, true,
            context.config.show_types_in_errors));
        } else {
          item_node.add(ExecutionPlanNode::annotation(format!("{}={}", key, item_value.to_string())));
          let mut item_check = ExecutionPlanNode::action("match:equality");
          item_check
            .add(ExecutionPlanNode::value_node(item_value))
            .add(ExecutionPlanNode::resolve_value(&path))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL))
            .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors));
          presence_check.add(item_check);
        }

        item_node.add(presence_check);
        plan_node.add(item_node);
      }

      plan_node.add(
        ExecutionPlanNode::action("expect:entries")
          .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone())))
          .add(ExecutionPlanNode::resolve_value(doc_path.clone()))
          .add(
            ExecutionPlanNode::action("join")
              .add(ExecutionPlanNode::value_node("The following expected query parameters were missing: "))
              .add(ExecutionPlanNode::action("join-with")
                .add(ExecutionPlanNode::value_node(", "))
                .add(
                  ExecutionPlanNode::splat()
                    .add(ExecutionPlanNode::action("apply"))
                )
              )
          )
      );

      plan_node.add(
        ExecutionPlanNode::action("expect:only-entries")
          .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone())))
          .add(ExecutionPlanNode::resolve_value(doc_path.clone()))
          .add(
            ExecutionPlanNode::action("join")
              .add(ExecutionPlanNode::value_node("The following query parameters were not expected: "))
              .add(ExecutionPlanNode::action("join-with")
                .add(ExecutionPlanNode::value_node(", "))
                .add(
                  ExecutionPlanNode::splat()
                    .add(ExecutionPlanNode::action("apply"))
                )
              )
          )
      );
    }
  } else {
    plan_node
      .add(
        ExecutionPlanNode::action("expect:empty")
          .add(ExecutionPlanNode::resolve_value(doc_path.clone()))
          .add(
            ExecutionPlanNode::action("join")
              .add(ExecutionPlanNode::value_node("Expected no query parameters but got "))
              .add(ExecutionPlanNode::resolve_value(doc_path))
          )
      );
  }

  Ok(plan_node)
}

fn setup_header_plan<T: HttpPart>(
  expected: &T,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  let mut plan_node = ExecutionPlanNode::container("headers");
  let doc_path = DocPath::new("$.headers")?;

  if let Some(headers) = &expected.headers() {
    if !headers.is_empty() {
      let keys = headers.keys().cloned().sorted().collect_vec();
      for key in &keys {
        let value = headers.get(key).unwrap();
        let mut item_node = ExecutionPlanNode::container(key);

        let mut presence_check = ExecutionPlanNode::action("if");
        let item_value = if value.len() == 1 {
          NodeValue::STRING(value[0].clone())
        } else {
          NodeValue::SLIST(value.clone())
        };
        presence_check
          .add(
            ExecutionPlanNode::action("check:exists")
              .add(ExecutionPlanNode::resolve_value(doc_path.join(key)))
          );

        let item_path = DocPath::root().join(key);
        let path = doc_path.join(key);
        if context.matcher_is_defined(&item_path) {
          let matchers = context.select_best_matcher(&item_path);
          item_node.add(ExecutionPlanNode::annotation(format!("{} {}", key, matchers.generate_description(true))));
          presence_check.add(build_matching_rule_node(&ExecutionPlanNode::value_node(item_value),
            &ExecutionPlanNode::resolve_value(&path), &matchers, true,
            context.config.show_types_in_errors));
        } else if PARAMETERISED_HEADERS.contains(&key.to_lowercase().as_str()) {
          item_node.add(ExecutionPlanNode::annotation(format!("{}={}", key, item_value.to_string())));
          if value.len() == 1 {
            let apply_node = build_parameterised_header_plan(&path, value[0].as_str());
            presence_check.add(apply_node);
          } else {
            for (index, item_value) in value.iter().enumerate() {
              let item_path = doc_path.join(key).join_index(index);
              let mut item_node = ExecutionPlanNode::container(index.to_string());
              let apply_node = build_parameterised_header_plan(
                &item_path, item_value.as_str());
              item_node.add(apply_node);
              presence_check.add(item_node);
            }
          }
        } else {
          item_node.add(ExecutionPlanNode::annotation(format!("{}={}", key, item_value.to_string())));
          let mut item_check = ExecutionPlanNode::action("match:equality");
          item_check
            .add(ExecutionPlanNode::value_node(item_value))
            .add(ExecutionPlanNode::resolve_value(doc_path.join(key)))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL))
            .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors));
          presence_check.add(item_check);
        }

        item_node.add(presence_check);
        plan_node.add(item_node);
      }

      plan_node.add(
        ExecutionPlanNode::action("expect:entries")
          .add(ExecutionPlanNode::action("lower-case")
            .add(ExecutionPlanNode::value_node(NodeValue::SLIST(keys.clone()))))
          .add(ExecutionPlanNode::resolve_value(doc_path.clone()))
          .add(
            ExecutionPlanNode::action("join")
              .add(ExecutionPlanNode::value_node("The following expected headers were missing: "))
              .add(ExecutionPlanNode::action("join-with")
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

  Ok(plan_node)
}

fn build_parameterised_header_plan(doc_path: &DocPath, val: &str) -> ExecutionPlanNode {
  let values: Vec<&str> = strip_whitespace(val, ";");
  let (header_value, header_params) = values.as_slice()
    .split_first()
    .unwrap_or((&"", &[]));
  let parameter_map = parse_charset_parameters(header_params);

  let mut apply_node = ExecutionPlanNode::action("tee");
  apply_node
    .add(ExecutionPlanNode::action("header:parse")
      .add(ExecutionPlanNode::resolve_value(doc_path)));
  apply_node.add(
    ExecutionPlanNode::action("match:equality")
      .add(ExecutionPlanNode::value_node(*header_value))
      .add(ExecutionPlanNode::action("to-string")
        .add(ExecutionPlanNode::resolve_current_value(&DocPath::new_unwrap("value"))))
      .add(ExecutionPlanNode::value_node(NodeValue::NULL))
      .add(ExecutionPlanNode::value_node(NodeValue::BOOL(false)))
  );

  if !parameter_map.is_empty() {
    let parameter_path = DocPath::new_unwrap("parameters");
    for (k, v) in &parameter_map {
      let mut parameter_node = ExecutionPlanNode::container(k.as_str());
      parameter_node.add(
        ExecutionPlanNode::action("if")
          .add(ExecutionPlanNode::action("check:exists")
            .add(ExecutionPlanNode::resolve_current_value(&parameter_path.join(k.as_str()))))
          .add(ExecutionPlanNode::action("match:equality")
            .add(ExecutionPlanNode::value_node(v.to_lowercase()))
            .add(ExecutionPlanNode::action("lower-case")
              .add(ExecutionPlanNode::resolve_current_value(&parameter_path.join(k.as_str()))))
            .add(ExecutionPlanNode::value_node(NodeValue::NULL))
            .add(ExecutionPlanNode::value_node(NodeValue::BOOL(false))))
          .add(ExecutionPlanNode::action("error")
            .add(ExecutionPlanNode::value_node(
              format!("Expected a {} value of '{}' but it was missing", k, v)))
          )
      );
      apply_node.add(parameter_node);
    }
  }
  apply_node
}

fn setup_body_plan<T: HttpPart>(
  expected: &T,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  // TODO: Look at the matching rules and generators here
  let mut plan_node = ExecutionPlanNode::container("body");
  let body_path = DocPath::body();

  match &expected.body() {
    OptionalBody::Missing => {}
    OptionalBody::Empty | OptionalBody::Null => {
      plan_node.add(ExecutionPlanNode::action("expect:empty")
        .add(ExecutionPlanNode::resolve_value(body_path)));
    }
    OptionalBody::Present(content, _, _) => {
      let content_type = expected.content_type().unwrap_or_else(|| TEXT.clone());
      let root_matcher = expected.matching_rules()
        .rules_for_category("body")
        .map(|category| category.rules.get(&DocPath::root()).cloned())
        .flatten();
      if let Some(root_matcher) = root_matcher && root_matcher.can_match(&content_type) {
        plan_node.add(build_matching_rule_node(
          &ExecutionPlanNode::value_node(NodeValue::NULL),
          &ExecutionPlanNode::resolve_value(body_path),
          &root_matcher,
          false,
          context.config.show_types_in_errors
        ));
      } else {
        let mut content_type_check_node = ExecutionPlanNode::action("if");
        content_type_check_node
          .add(
            ExecutionPlanNode::action("match:equality")
              .add(ExecutionPlanNode::value_node(content_type.to_string()))
              .add(ExecutionPlanNode::resolve_value(DocPath::new("$.content-type")?))
              .add(ExecutionPlanNode::value_node(NodeValue::NULL))
              .add(ExecutionPlanNode::value_node(false))
              .add(
                ExecutionPlanNode::action("error")
                  .add(ExecutionPlanNode::value_node(NodeValue::STRING("Body type error - ".to_string())))
                  .add(ExecutionPlanNode::action("apply"))
              )
          );
        if let Some(plan_builder) = get_body_plan_builder(&content_type) {
          content_type_check_node.add(plan_builder.build_plan(content, context)?);
        } else {
          let plan_builder = PlainTextBuilder::new();
          content_type_check_node.add(plan_builder.build_plan(content, context)?);
        }
        plan_node.add(content_type_check_node);
      }
    }
  }

  Ok(plan_node)
}

/// Executes the request plan against the actual request.
pub fn execute_request_plan(
  plan: &ExecutionPlan,
  actual: &HttpRequest,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  let value_resolver = HttpRequestValueResolver {
    request: actual.clone()
  };
  let mut interpreter = ExecutionPlanInterpreter::new_with_context(context);
  let result = interpreter.execute_plan(&plan, &value_resolver)?;
  debug!("Total execution time: {:?}", result.execution_time.unwrap_or_default());
  Ok(result)
}

/// Constructs an execution plan for the HTTP response part.
pub fn build_response_plan(
  expected: &HttpResponse,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  let mut plan = ExecutionPlan::new("response");

  plan.add(setup_status_plan(expected, &context.for_status())?);
  plan.add(setup_header_plan(expected, &context.for_resp_headers())?);
  plan.add(setup_body_plan(expected, &context.for_resp_body())?);

  Ok(plan)
}

fn setup_status_plan(
  expected: &HttpResponse,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlanNode> {
  let mut plan_node = ExecutionPlanNode::container("status");
  let expected_node = ExecutionPlanNode::value_node(expected.status);
  let doc_path = DocPath::new("$.status")?;
  let actual_node = ExecutionPlanNode::resolve_value(doc_path.clone());
  if context.matcher_is_defined(&doc_path) {
    let matchers = context.select_best_matcher(&doc_path);
    plan_node.add(ExecutionPlanNode::annotation(format!("status {}", matchers.generate_description(false))));
    plan_node.add(build_matching_rule_node(&expected_node, &actual_node, &matchers,
      false, context.config.show_types_in_errors));
  } else {
    plan_node.add(ExecutionPlanNode::annotation(format!("status == {}", expected.status)));
    plan_node
      .add(
        ExecutionPlanNode::action("match:equality")
          .add(expected_node)
          .add(ExecutionPlanNode::resolve_value(doc_path))
          .add(ExecutionPlanNode::value_node(NodeValue::NULL))
          .add(ExecutionPlanNode::value_node(context.config.show_types_in_errors))
      );
  }
  Ok(plan_node)
}

/// Executes the response plan against the actual response.
pub fn execute_response_plan(
  plan: &ExecutionPlan,
  actual: &HttpResponse,
  context: &PlanMatchingContext
) -> anyhow::Result<ExecutionPlan> {
  let value_resolver = HttpResponseValueResolver {
    response: actual.clone()
  };
  let mut interpreter = ExecutionPlanInterpreter::new_with_context(context);
  let result = interpreter.execute_plan(&plan, &value_resolver)?;
  debug!("Total execution time: {:?}", result.execution_time.unwrap_or_default());
  Ok(result)
}

pub(crate) fn escape(s: &str) -> String {
  if s.is_empty() {
    "''".to_string()
  } else {
    let mut buffer = String::with_capacity(s.len() + 2);
    buffer.push('\'');

    for c in s.chars() {
      match c {
        '\'' | '\\' => {
          buffer.push('\\');
          buffer.push(c);
        }
        '\n' => {
          buffer.push('\\');
          buffer.push('n');
        }
        '\t' => {
          buffer.push('\\');
          buffer.push('t');
        }
        '\r' => {
          buffer.push('\\');
          buffer.push('r');
        }
        _ => buffer.push(c)
      }
    }

    buffer.push('\'');
    buffer
  }
}

pub(crate) fn unescape(s: &str) -> String {
  if s.is_empty() || s == "''" {
    "".to_string()
  } else {
    let mut buffer = String::with_capacity(s.len() - 2);

    let mut chars = s.chars().dropping(1).dropping_back(1);
    while let Some(c) = chars.next() {
      match c {
        '\\' => {
          if let Some(c1) = chars.next() {
            match c1 {
              '\n' => buffer.push('\n'),
              '\t' => buffer.push('\t'),
              '\r' => buffer.push('\r'),
              _ => {
                buffer.push(c);
                buffer.push(c1);
              }
            }
          } else {
            buffer.push(c);
          }
        }
        _ => buffer.push(c)
      }
    }

    buffer
  }
}

#[cfg(test)]
mod tests;
