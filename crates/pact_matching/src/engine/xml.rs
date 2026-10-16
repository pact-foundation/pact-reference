//! Structs and functions for dealing with XML

use std::fmt::{Display, Formatter};

use anyhow::anyhow;
use kiss_xml::dom::Element;

use pact_models::matchingrules::MatchingRule;
use pact_models::xml_utils::XmlResult;

use crate::engine::escape;
use crate::matchingrules::DoMatch;

/// Enum to store different XML nodes
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum XmlValue {
  /// XML element
  Element(Element),
  /// XML text
  Text(String),
  /// Attribute
  Attribute(String, String)
}

impl XmlValue {
  /// Returns the value if it is an XML element
  pub fn as_element(&self) -> Option<Element> {
    match self {
      XmlValue::Element(element) => Some(element.clone()),
      _ => None
    }
  }

  /// Returns the value if it is XML text
  pub fn as_text(&self) -> Option<String> {
    match self {
      XmlValue::Text(text) => Some(text.clone()),
      _ => None
    }
  }

  /// Returns the value if it is an XML attribute
  pub fn as_attribute(&self) -> Option<(String, String)> {
    match self {
      XmlValue::Attribute(name, value) => Some((name.clone(), value.clone())),
      _ => None
    }
  }
}

impl Display for XmlValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      XmlValue::Element(el) => write!(f, "{}", el),
      XmlValue::Text(txt) => write!(f, "{}", escape(txt.as_str())),
      XmlValue::Attribute(name, value) => write!(f, "@{}={}", name, escape(value.as_str()))
    }
  }
}

impl From<XmlResult> for XmlValue {
  fn from(value: XmlResult) -> Self {
    match value {
      XmlResult::ElementNode(element) => XmlValue::Element(element),
      XmlResult::TextNode(text) => XmlValue::Text(text),
      XmlResult::Attribute(name, value) => XmlValue::Attribute(name, value)
    }
  }
}

impl DoMatch<XmlValue> for MatchingRule {
  fn match_value(
    &self,
    expected_value: XmlValue,
    actual_value: XmlValue,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    self.match_value(&expected_value, &actual_value, cascaded, show_types)
  }
}

impl DoMatch<&XmlValue> for MatchingRule {
  fn match_value(
    &self,
    expected_value: &XmlValue,
    actual_value: &XmlValue,
    cascaded: bool,
    show_types: bool
  ) -> anyhow::Result<()> {
    match expected_value {
      XmlValue::Element(expected) => if let Some(actual) = actual_value.as_element() {
        self.match_value(expected, &actual, cascaded, show_types)
      } else {
        Err(anyhow!("Was expecting an XML element but got {}", actual_value))
      }
      XmlValue::Text(expected) => if let Some(actual) = actual_value.as_text() {
        self.match_value(expected.as_str(), actual.as_str(), cascaded, show_types)
      } else {
        Err(anyhow!("Was expecting XML text but got {}", actual_value))
      }
      XmlValue::Attribute(_, expected_value) => if let Some((_, value)) = actual_value.as_attribute() {
        self.match_value(expected_value.as_str(), value.as_str(), cascaded, show_types)
      } else {
        Err(anyhow!("Was expecting an XML attribute but got {}", actual_value))
      }
    }
  }
}
