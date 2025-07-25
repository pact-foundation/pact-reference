//! Collection of utilities for working with XML

use std::collections::{BTreeMap, HashMap};
use std::ops::Index;
use std::str;
use anyhow::anyhow;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use kiss_xml::dom::{Element, Node};
use lazy_static::lazy_static;
use onig::EncodedChars;
use regex::Regex;
use sxd_document::{Package, parser};
use tracing::trace;

use crate::path_exp::{DocPath, PathToken};

/// Parses a vector of bytes into a XML document
pub fn parse_bytes(bytes: &[u8]) -> anyhow::Result<Package> {
  let string = str::from_utf8(bytes)?;
  match parser::parse(string) {
    Ok(doc) => Ok(doc),
    Err(err) => Err(anyhow!("Failed to parse bytes as XML - {}", err))
  }
}

/// Resolve the path expression against the XML, returning a list of pointer values that match.
pub fn resolve_path(value: &Element, expression: &DocPath) -> Vec<String> {
  let mut tree = Arena::new();
  let root = tree.new_node("".into());

  let tokens = expression.tokens();
  query_graph(tokens.as_slice(), &mut tree, root, value, 0);

  let tokens = expression.tokens().iter()
    .filter(|t| match t {
      PathToken::Index(_) => false,
      _ => true
    }).collect_vec();
  let expanded_paths = root.descendants(&tree)
    .fold(Vec::<String>::new(), |mut acc, node_id| {
    let node = tree.index(node_id);
    if !node.get().is_empty() && node.first_child().is_none() {
      let path: Vec<String> = node_id.ancestors(&tree)
        .map(|n| format!("{}", tree.index(n).get()))
        .collect();
      if path.len() == tokens.len() {
        acc.push(path.iter().rev().join("/"));
      }
    }
    acc
  });
  expanded_paths
}

fn query_graph(
  path_iter: &[PathToken],
  tree: &mut Arena<String>,
  parent_id: NodeId,
  element: &Element,
  index: usize
) {
  trace!(?path_iter, %parent_id, index, %element, ">>> query_graph");

  if let Some(token) = path_iter.first() {
    trace!(?token, "next token");
    match token {
      PathToken::Field(name) => {
        let matches = if element.name() == name.as_str() {
          trace!(name, %parent_id, "Field name matches element");
          Some(parent_id.append_value(format!("{}[{}]", name, index), tree))
        } else {
          if let Some(ns) = element.namespace() {
            let name_with_ns = format!("{}:{}", ns, element.name());
            if name_with_ns == name.as_str() {
              trace!(name, %parent_id, "Field name matches element including namespace");
              Some(parent_id.append_value(format!("{}[{}]", name_with_ns, index), tree))
            } else {
              None
            }
          } else {
            None
          }
        };

        if let Some(node_id) = matches {
          let remaining_tokens = &path_iter[1..];
          if !remaining_tokens.is_empty() {
            query_attributes(remaining_tokens, tree, node_id, element, index);
            query_text(remaining_tokens, tree, node_id, element, index);

            if let Some(PathToken::Index(_)) = remaining_tokens.first() {
              query_graph(remaining_tokens, tree, node_id, element, index);
            } else {
              let grouped_children = group_children(element);
              for children in grouped_children.values() {
                for (index, child) in children.iter().enumerate() {
                  query_graph(remaining_tokens, tree, node_id, *child, index);
                }
              }
            }
          }
        }
      },
      PathToken::Index(i) => {
        if *i == index {
          trace!(index, i, %parent_id, "Index matches element index");
          let remaining_tokens = &path_iter[1..];
          if !remaining_tokens.is_empty() {
            query_attributes(remaining_tokens, tree, parent_id, element, index);
            query_text(remaining_tokens, tree, parent_id, element, index);

            let grouped_children = group_children(element);
            for (_, children) in grouped_children {
              for (index, child) in children.iter().enumerate() {
                query_graph(remaining_tokens, tree, parent_id, *child, index);
              }
            }
          }
        } else {
          trace!(index, i, %parent_id, "Index does not match element index, removing");
          parent_id.detach(tree);
        }
      }
      PathToken::Star | PathToken::StarIndex => {
        trace!(%parent_id, name = element.name(), "* -> Adding current node to parent");
        let node_id = parent_id.append_value(format!("{}[{}]", element.name(), index), tree);

        let remaining_tokens = &path_iter[1..];
        if !remaining_tokens.is_empty() {
          query_attributes(remaining_tokens, tree, node_id, element, index);
          query_text(remaining_tokens, tree, node_id, element, index);

          let grouped_children = group_children(element);
          for (_, children) in grouped_children {
            for (index, child) in children.iter().enumerate() {
              query_graph(remaining_tokens, tree, node_id, *child, index);
            }
          }
        }
      },
      PathToken::Root => {
        query_graph(&path_iter[1..], tree, parent_id, element, index);
      }
    }
  }
}

/// Groups all the child element by name
pub fn group_children(element: &Element) -> BTreeMap<String, Vec<&Element>> {
  element.child_elements()
    .fold(BTreeMap::new(), |mut acc, child| {
      acc.entry(child.name())
        .and_modify(|entry: &mut Vec<_>| entry.push(child))
        .or_insert_with(|| vec![child]);
      acc
    })
}

fn query_attributes(
  path_iter: &[PathToken],
  tree: &mut Arena<String>,
  parent_id: NodeId,
  element: &Element,
  index: usize
) {
  trace!(?path_iter, %parent_id, index, %element, ">>> query_attributes");

  if let Some(token) = path_iter.first() {
    trace!(?token, "next token");
    if let PathToken::Field(name) = token {
      if name.starts_with('@') {
        let attribute_name = &name[1..];
        let attributes = resolve_namespaces(element.attributes());
        if attributes.contains_key(attribute_name) {
          trace!(name, "Field name matches element attribute");
          parent_id.append_value(name.clone(), tree);
        }
      }
    }
  }
}

fn resolve_namespaces(attributes: &HashMap<String, String>) -> HashMap<String, String> {
  let namespaces: HashMap<_, _> = attributes.iter()
    .filter_map(|(key, value)| if key.starts_with("xmlns:") {
      Some((key.strip_prefix("xmlns:").unwrap(), value.as_str()))
    } else {
      None
    }).collect();
  if namespaces.is_empty() {
    attributes.clone()
  } else {
    attributes.iter()
      .flat_map(|(k, v)| {
        if let Some((ns, attr)) = k.split_once(':') {
          if let Some(name) = namespaces.get(ns) {
            vec![(k.clone(), v.clone()), (format!("{}:{}", *name, attr), v.clone())]
          } else {
            vec![(k.clone(), v.clone())]
          }
        } else {
          vec![(k.clone(), v.clone())]
        }
      }).collect()
  }
}

fn query_text(
  path_iter: &[PathToken],
  tree: &mut Arena<String>,
  parent_id: NodeId,
  element: &Element,
  index: usize
) {
  trace!(?path_iter, %parent_id, index, %element, ">>> query_text");

  if let Some(token) = path_iter.first() {
    trace!(?token, "next token");
    if let PathToken::Field(name) = token {
      let text_nodes = text_nodes(element);
      if name == "#text" && !text_nodes.is_empty() {
        trace!(name, "Field name matches element text");
        parent_id.append_value(name.clone(), tree);
      }
    }
  }
}

/// Return all the content of the element text nodes
pub fn text_nodes(element: &Element) -> Vec<String> {
  element.children()
    .filter_map(|child| if let Ok(text) = child.as_text() {
      if text.content.is_empty() {
        None
      } else {
        Some(text.content.clone())
      }
    } else {
      None
    })
    .collect_vec()
}

lazy_static!{
   static ref PATH_RE: Regex = Regex::new(r#"(\w+)\[(\d+)]"#).unwrap();
}

/// Enum to box the result value from resolve_matching_node
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum XmlResult {
  /// Matched XML element
  ElementNode(Element),
  /// Matched XML text
  TextNode(String),
  /// Matches an attribute
  Attribute(String, String)
}

/// Returns the matching node from the XML for the given path.
pub fn resolve_matching_node(element: &Element, path: &str) -> Option<XmlResult> {
  trace!(path, %element, ">>> resolve_matching_node");
  let paths = path.split("/")
    .filter(|s| !s.is_empty())
    .collect_vec();
  if let Some(first_part) = paths.first() {
    if let Some(captures) = PATH_RE.captures(first_part) {
      let name = &captures[1];
      let index: usize = (&captures[2]).parse().unwrap_or_default();
      if index == 0 && name == element.name() {
        if paths.len() > 1 {
          match_next(element, &paths[1..])
        } else {
          Some(XmlResult::ElementNode(element.clone()))
        }
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

fn match_next(element: &Element, paths: &[&str]) -> Option<XmlResult> {
  trace!(?paths, %element, ">>> match_next");
  if let Some(first_part) = paths.first() {
    if first_part.starts_with('@') {
      element.attributes().get(&first_part[1..])
        .map(|value| XmlResult::Attribute(first_part[1..].to_string(), value.clone()))
    } else if *first_part == "#text" {
      let text = element.text();
      if text.is_empty() {
        None
      } else {
        Some(XmlResult::TextNode(text))
      }
    } else if let Some(captures) = PATH_RE.captures(first_part) {
      let name = &captures[1];
      let index: usize = (&captures[2]).parse().unwrap_or_default();
      let grouped_children = group_children(element);
      let child = grouped_children.get(name)
        .map(|values| values.get(index))
        .flatten()
        .map(|value| *value);
      if let Some(child) = child {
        if paths.len() > 1 {
          match_next(child, &paths[1..])
        } else {
          Some(XmlResult::ElementNode(child.clone()))
        }
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;

  use crate::path_exp::DocPath;

  use super::*;

  #[test_log::test]
  fn resolve_path_test() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <config>
        <name>My Settings</name>
        <sound>
          <property name="volume" value="11" />
          <property name="mixer" value="standard" />
        </sound>
      </config>
      "#;
    let dom = kiss_xml::parse_str(xml).unwrap();
    let root = dom.root_element();

    let path = DocPath::root();
    expect!(resolve_path(root, &path).is_empty()).to(be_true());

    let path = DocPath::new_unwrap("$.config");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/config[0]"]));

    let path = DocPath::new_unwrap("$.config.sound");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/config[0]/sound[0]"]));

    let path = DocPath::new_unwrap("$.config.sound.property");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/sound[0]/property[0]",
      "/config[0]/sound[0]/property[1]"
    ]));

    let path = DocPath::new_unwrap("$.config.sound[0].property[0]");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/sound[0]/property[0]"
    ]));

    let path = DocPath::new_unwrap("$.config.*");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/name[0]",
      "/config[0]/sound[0]"
    ]));

    let path = DocPath::new_unwrap("$.config[*]");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/name[0]",
      "/config[0]/sound[0]"
    ]));

    let path = DocPath::new_unwrap("$.config.sound.property.@name");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/sound[0]/property[0]/@name",
      "/config[0]/sound[0]/property[1]/@name"
    ]));

    let path = DocPath::new_unwrap("$.config.sound.property.@other");
    expect!(resolve_path(root, &path).is_empty()).to(be_true());

    let path = DocPath::new_unwrap("$.config.sound.*.@name");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/sound[0]/property[0]/@name",
      "/config[0]/sound[0]/property[1]/@name"
    ]));

    let path = DocPath::new_unwrap("$.config.name.#text");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/config[0]/name[0]/#text"]));

    let path = DocPath::new_unwrap("$.config.*.#text");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/config[0]/name[0]/#text"]));

    let path = DocPath::new_unwrap("$.config.sound.property.#text");
    expect!(resolve_path(root, &path).is_empty()).to(be_true());

    let path = DocPath::new_unwrap("$.config.sound.property[1].@name");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec![
      "/config[0]/sound[0]/property[1]/@name"
    ]));

    let path = DocPath::new_unwrap("$.config.sound.property[2].@name");
    expect!(resolve_path(root, &path).is_empty()).to(be_true());
  }

  #[test_log::test]
  fn resolve_path_with_xml_namespaces_test() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <a:alligator xmlns:a="urn:alligators" xmlns:n="urn:names" n:name="Mary">
        <a:favouriteNumbers>
          <favouriteNumber xmlns="urn:favourite:numbers">1</favouriteNumber>
        </a:favouriteNumbers>
      </a:alligator>
      "#;
    let dom = kiss_xml::parse_str(xml).unwrap();
    let root = dom.root_element();

    let path = DocPath::root();
    expect!(resolve_path(root, &path).is_empty()).to(be_true());

    let path = DocPath::new_unwrap("$.alligator");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/alligator[0]"]));

    let path = DocPath::new_unwrap("$['urn:alligators:alligator']");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/urn:alligators:alligator[0]"]));

    let path = DocPath::new_unwrap("$['urn:alligators:alligator']['@n:name']");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/urn:alligators:alligator[0]/@n:name"]));

    let path = DocPath::new_unwrap("$['urn:alligators:alligator']['@urn:names:name']");
    expect!(resolve_path(root, &path)).to(be_equal_to(vec!["/urn:alligators:alligator[0]/@urn:names:name"]));
  }

  #[test_log::test]
  fn resolve_matching_node_test() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
      <config>
        <name>My Settings</name>
        <sound>
          <property name="volume" value="11" />
          <property name="mixer" value="standard" />
        </sound>
      </config>
      "#;
    let dom = kiss_xml::parse_str(xml).unwrap();
    let root = dom.root_element();

    expect!(resolve_matching_node(root, "/config[0]")).to(be_some()
      .value(XmlResult::ElementNode(root.clone())));
    expect!(resolve_matching_node(root, "/config[1]")).to(be_none());

    let sound = root.elements_by_name("sound").next().unwrap().clone();
    expect!(resolve_matching_node(root, "/config[0]/sound[0]")).to(be_some()
      .value(XmlResult::ElementNode(sound.clone())));
    expect!(resolve_matching_node(root, "/config[0]/sound[1]")).to(be_none());

    let properties = sound.elements_by_name("property").cloned().collect_vec();
    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[0]")).to(be_some()
      .value(XmlResult::ElementNode(properties[0].clone())));
    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[1]")).to(be_some()
      .value(XmlResult::ElementNode(properties[1].clone())));

    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[0]/@name")).to(be_some()
      .value(XmlResult::Attribute("name".to_string(), "volume".to_string())));
    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[1]/@name")).to(be_some()
      .value(XmlResult::Attribute("name".to_string(), "mixer".to_string())));
    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[1]/@other")).to(be_none());

    expect!(resolve_matching_node(root, "/config[0]/name[0]/#text")).to(be_some()
      .value(XmlResult::TextNode("My Settings".to_string())));
    expect!(resolve_matching_node(root, "/config[0]/sound[0]/property[0]/#text")).to(be_none());
  }

  #[test_log::test]
  fn resolve_namespaces_test() {
    expect!(resolve_namespaces(&hashmap!{})).to(be_equal_to(hashmap!{}));

    let attributes = hashmap!{
      "a".to_string() => "b".to_string(),
      "c".to_string() => "d".to_string()
    };
    expect!(resolve_namespaces(&attributes)).to(be_equal_to(attributes));

    let attributes = hashmap!{
      "n:name".to_string() => "Mary".to_string(),
      "xmlns:a".to_string() => "urn:alligators".to_string(),
      "xmlns:n".to_string() => "urn:names".to_string()
    };
    expect!(resolve_namespaces(&attributes)).to(be_equal_to(hashmap!{
      "n:name".to_string() => "Mary".to_string(),
      "urn:names:name".to_string() => "Mary".to_string(),
      "xmlns:a".to_string() => "urn:alligators".to_string(),
      "xmlns:n".to_string() => "urn:names".to_string()
    }));
  }
}
