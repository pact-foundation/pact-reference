//! `generators` module includes all the classes to deal with V3 format generators

use std::{
  collections::HashMap,
  hash::{Hash, Hasher},
  str::FromStr,
  ops::Index
};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value, json};
use maplit::*;
use super::PactSpecification;
use rand::prelude::*;
use rand::distributions::Alphanumeric;
use uuid::Uuid;
use crate::models::{OptionalBody, DetectedContentType};
use crate::models::json_utils::{JsonToNum, json_to_string};
use crate::path_exp::*;
use itertools::Itertools;
use indextree::{Arena, NodeId};
use chrono::prelude::*;
use crate::time_utils::{parse_pattern, to_chrono_pattern};
use nom::types::CompleteStr;
use regex_syntax;
use xmltree::{Element, XMLNode};
use log::*;

/// Trait to represent a generator
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum Generator {
  /// Generates a random integer between the min and max values
  RandomInt(i32, i32),
  /// Generates a random UUID value
  Uuid,
  /// Generates a random sequence of digits
  RandomDecimal(u16),
  /// Generates a random sequence of hexadecimal digits
  RandomHexadecimal(u16),
  /// Generates a random string of the provided size
  RandomString(u16),
  /// Generates a random string that matches the provided regex
  Regex(String),
  /// Generates a random date that matches either the provided format or the ISO format
  Date(Option<String>),
  /// Generates a random time that matches either the provided format or the ISO format
  Time(Option<String>),
  /// Generates a random timestamp that matches either the provided format or the ISO format
  DateTime(Option<String>),
  /// Generates a random boolean value
  RandomBoolean,
  /// Generates a value that is looked up from the provider state context
  ProviderStateGenerator(String)
}

impl Generator {
  /// Convert this generator to a JSON struct
  pub fn to_json(&self) -> Value {
    match self {
      &Generator::RandomInt(min, max) => json!({ "type": "RandomInt", "min": min, "max": max }),
      &Generator::Uuid => json!({ "type": "Uuid" }),
      &Generator::RandomDecimal(digits) => json!({ "type": "RandomDecimal", "digits": digits }),
      &Generator::RandomHexadecimal(digits) => json!({ "type": "RandomHexadecimal", "digits": digits }),
      &Generator::RandomString(size) => json!({ "type": "RandomString", "size": size }),
      &Generator::Regex(ref regex) => json!({ "type": "Regex", "regex": regex }),
      &Generator::Date(ref format) => match format {
        &Some(ref format) => json!({ "type": "Date", "format": format }),
        &None => json!({ "type": "Date" })
      },
      &Generator::Time(ref format) => match format {
        &Some(ref format) => json!({ "type": "Time", "format": format }),
        &None => json!({ "type": "Time" })
      },
      &Generator::DateTime(ref format) => match format {
        &Some(ref format) => json!({ "type": "DateTime", "format": format }),
        &None => json!({ "type": "DateTime" })
      },
      &Generator::RandomBoolean => json!({ "type": "RandomBoolean" }),
      &Generator::ProviderStateGenerator(ref expression) => json!({"type": "ProviderState", "expression": expression})
    }
  }

  /// Converts a JSON map into a `Generator` struct, returning `None` if it can not be converted.
  pub fn from_map(gen_type: &String, map: &serde_json::Map<String, Value>) -> Option<Generator> {
    match gen_type.as_str() {
      "RandomInt" => {
        let min = <i32>::json_to_number(map, "min", 0);
        let max = <i32>::json_to_number(map, "max", 10);
        Some(Generator::RandomInt(min, max))
      },
      "Uuid" => Some(Generator::Uuid),
      "RandomDecimal" => Some(Generator::RandomDecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomHexadecimal" => Some(Generator::RandomHexadecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomString" => Some(Generator::RandomString(<u16>::json_to_number(map, "size", 10))),
      "Regex" => map.get("regex").map(|val| Generator::Regex(json_to_string(val))),
      "Date" => Some(Generator::Date(map.get("format").map(|f| json_to_string(f)))),
      "Time" => Some(Generator::Time(map.get("format").map(|f| json_to_string(f)))),
      "DateTime" => Some(Generator::DateTime(map.get("format").map(|f| json_to_string(f)))),
      "RandomBoolean" => Some(Generator::RandomBoolean),
      "ProviderState" => Some(Generator::ProviderStateGenerator(map.get("expression").map(|f| json_to_string(f)).unwrap())),
      _ => {
        log::warn!("'{}' is not a valid generator type", gen_type);
        None
      }
    }
  }
}

/// Trait that represents generation of a value based on a source value.
pub trait GenerateValue<T> {
  /// Generates a new value based on the source value. `None` will be returned if the value can not
  /// be generated.
  fn generate_value(&self, value: T, context: &HashMap<String, Value>) -> Option<T>;
}

impl GenerateValue<u16> for Generator {
  fn generate_value(&self, _: u16, _context: &HashMap<String, Value>) -> Option<u16> {
    match self {
      &Generator::RandomInt(min, max) => Some(rand::thread_rng().gen_range(min as u16, (max as u16).saturating_add(1))),
      &Generator::ProviderStateGenerator(ref _exp) => None,
      _ => None
    }
  }
}

fn generate_decimal(digits: usize) -> String {
  const DIGIT_CHARSET: &'static str = "0123456789";
  let mut rnd = rand::thread_rng();
  DIGIT_CHARSET.chars().choose_multiple(&mut rnd, digits).iter().join("")
}

fn generate_hexadecimal(digits: usize) -> String {
  const HEX_CHARSET: &'static str = "0123456789ABCDEF";
  let mut rnd = rand::thread_rng();
  HEX_CHARSET.chars().choose_multiple(&mut rnd, digits).iter().join("")
}

fn generate_ascii_string(size: usize) -> String {
  rand::thread_rng().sample_iter(&Alphanumeric).take(size).collect()
}

impl GenerateValue<String> for Generator {
  fn generate_value(&self, _: String, _context: &HashMap<String, Value>) -> Option<String> {
    let mut rnd = rand::thread_rng();
    match self {
      &Generator::RandomInt(min, max) => Some(format!("{}", rnd.gen_range(min, max.saturating_add(1)))),
      &Generator::Uuid => Some(Uuid::new_v4().simple().to_string()),
      &Generator::RandomDecimal(digits) => Some(generate_decimal(digits as usize)),
      &Generator::RandomHexadecimal(digits) => Some(generate_hexadecimal(digits as usize)),
      &Generator::RandomString(size) => Some(generate_ascii_string(size as usize)),
      &Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(regex) {
          Ok(hir) => {
            let gen = rand_regex::Regex::with_hir(hir, 20).unwrap();
            Some(rnd.sample(gen))
          },
          Err(err) => {
            log::warn!("'{}' is not a valid regular expression - {}", regex, err);
            None
          }
        }
      },
      &Generator::Date(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(Local::now().date().format(&to_chrono_pattern(&tokens.1)).to_string()),
          Err(err) => {
            log::warn!("Date format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(Local::now().naive_local().date().to_string())
      },
      &Generator::Time(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(Local::now().format(&to_chrono_pattern(&tokens.1)).to_string()),
          Err(err) => {
            log::warn!("Time format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(Local::now().time().format("%H:%M:%S").to_string())
      },
      &Generator::DateTime(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(Local::now().format(&to_chrono_pattern(&tokens.1)).to_string()),
          Err(err) => {
            log::warn!("DateTime format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(Local::now().format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string())
      },
      &Generator::RandomBoolean => Some(format!("{}", rnd.gen::<bool>())),
      &Generator::ProviderStateGenerator(ref _exp) => None
    }
  }
}

impl GenerateValue<Vec<u8>> for Generator {
  fn generate_value(&self, value: Vec<u8>, context: &HashMap<String, Value>) -> Option<Vec<u8>> {
    self.generate_value(String::from_utf8(value).unwrap_or(String::default()), context)
      .map(|v| v.as_bytes().to_vec())
  }
}

impl GenerateValue<Vec<String>> for Generator {
  fn generate_value(&self, vals: Vec<String>, context: &HashMap<String, Value>) -> Option<Vec<String>> {
    self.generate_value(vals.first().unwrap_or(&s!("")).clone(), context).map(|v| vec![v.clone()])
  }
}

impl GenerateValue<Value> for Generator {
  fn generate_value(&self, value: Value, _context: &HashMap<String, Value>) -> Option<Value> {
    match self {
      &Generator::RandomInt(min, max) => {
        let rand_int = rand::thread_rng().gen_range(min, max.saturating_add(1));
        match value {
          Value::String(_) => Some(json!(format!("{}", rand_int))),
          Value::Number(_) => Some(json!(rand_int)),
          _ => None
        }
      },
      &Generator::Uuid => match value {
        Value::String(_) => Some(json!(Uuid::new_v4().simple().to_string())),
        _ => None
      },
      &Generator::RandomDecimal(digits) => match value {
        Value::String(_) => Some(json!(generate_decimal(digits as usize))),
        Value::Number(_) => match generate_decimal(digits as usize).parse::<u64>() {
          Ok(val) => Some(json!(val)),
          Err(_) => None
        },
        _ => None
      },
      &Generator::RandomHexadecimal(digits) => match value {
        Value::String(_) => Some(json!(generate_hexadecimal(digits as usize))),
        _ => None
      },
      &Generator::RandomString(size) => match value {
        Value::String(_) => Some(json!(generate_ascii_string(size as usize))),
        _ => None
      },
      &Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(regex) {
          Ok(hir) => {
            let gen = rand_regex::Regex::with_hir(hir, 20).unwrap();
            Some(json!(rand::thread_rng().sample::<String, _>(gen)))
          },
          Err(err) => {
            log::warn!("'{}' is not a valid regular expression - {}", regex, err);
            None
          }
        }
      },
      &Generator::Date(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(json!(Local::now().date().format(&to_chrono_pattern(&tokens.1)).to_string())),
          Err(err) => {
            log::warn!("Date format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(json!(Local::now().naive_local().date().to_string()))
      },
      &Generator::Time(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(json!(Local::now().format(&to_chrono_pattern(&tokens.1)).to_string())),
          Err(err) => {
            log::warn!("Time format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(json!(Local::now().time().format("%H:%M:%S").to_string()))
      },
      &Generator::DateTime(ref format) => match format {
        Some(pattern) => match parse_pattern(CompleteStr(pattern)) {
          Ok(tokens) => Some(json!(Local::now().format(&to_chrono_pattern(&tokens.1)).to_string())),
          Err(err) => {
            log::warn!("DateTime format {} is not valid - {}", pattern, err);
            None
          }
        },
        None => Some(json!(Local::now().format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string()))
      },
      &Generator::RandomBoolean => Some(json!(rand::thread_rng().gen::<bool>())),
      &Generator::ProviderStateGenerator(ref _exp) => None
    }
  }
}

/// Category that the generator is applied to
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum GeneratorCategory {
  /// Request Method
  METHOD,
  /// Request Path
  PATH,
  /// Request/Response Header
  HEADER,
  /// Request Query Parameter
  QUERY,
  /// Body
  BODY,
  /// Response Status
  STATUS
}

impl FromStr for GeneratorCategory {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "method" => Ok(GeneratorCategory::METHOD),
      "path" => Ok(GeneratorCategory::PATH),
      "header" => Ok(GeneratorCategory::HEADER),
      "query" => Ok(GeneratorCategory::QUERY),
      "body" => Ok(GeneratorCategory::BODY),
      "status" => Ok(GeneratorCategory::STATUS),
      _ => Err(format!("'{}' is not a valid GeneratorCategory", s))
    }
  }
}

impl <'a> Into<&'a str> for GeneratorCategory {
  fn into(self) -> &'a str {
    match self {
      GeneratorCategory::METHOD => "method",
      GeneratorCategory::PATH => "path",
      GeneratorCategory::HEADER => "header",
      GeneratorCategory::QUERY => "query",
      GeneratorCategory::BODY => "body",
      GeneratorCategory::STATUS => "status"
    }
  }
}

impl Into<String> for GeneratorCategory {
  fn into(self) -> String {
    let s: &str = self.into();
    s.to_string()
  }
}

/// Trait to define a handler for applying generators to data of a particular content type.
pub trait ContentTypeHandler<T> {
  /// Processes the body using the map of generators, returning a (possibly) updated body.
  fn process_body(&mut self, generators: &HashMap<String, Generator>, context: &HashMap<String, Value>) -> Result<OptionalBody, String>;
  /// Applies the generator to the key in the body.
  fn apply_key(&mut self, key: &String, generator: &Generator, context: &HashMap<String, Value>);
}

/// Implementation of a content type handler for JSON
pub struct JsonHandler {
  /// JSON document to apply the generators to.
  pub value: Value
}

impl JsonHandler {
  fn query_object_graph(&self, path_exp: &Vec<PathToken>, tree: &mut Arena<String>, root: NodeId, body: Value) {
    let mut body_cursor = body;
    let mut it = path_exp.iter();
    let mut node_cursor = root;
    loop {
      match it.next() {
        Some(token) => {
          match token {
            &PathToken::Field(ref name) => {
              match body_cursor.clone().as_object() {
                Some(map) => match map.get(name) {
                  Some(val) => {
                    let node = tree.new_node(name.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    node_cursor = node;
                  },
                  None => return
                },
                None => return
              }
            },
            &PathToken::Index(index) => {
              match body_cursor.clone().as_array() {
                Some(list) => if list.len() > index {
                  let node = tree.new_node(format!("{}", index));
                  node_cursor.append(node, tree);
                  body_cursor = list[index].clone();
                  node_cursor = node;
                },
                None => return
              }
            }
            &PathToken::Star => {
              match body_cursor.clone().as_object() {
                Some(map) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (key, val) in map {
                    let node = tree.new_node(key.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node, val.clone());
                  }
                },
                None => return
              }
            },
            &PathToken::StarIndex => {
              match body_cursor.clone().as_array() {
                Some(list) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (index, val) in list.iter().enumerate() {
                    let node = tree.new_node(format!("{}", index));
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node,val.clone());
                  }
                },
                None => return
              }
            },
            _ => ()
          }
        },
        None => break
      }
    }
  }
}

impl ContentTypeHandler<Value> for JsonHandler {
  fn process_body(&mut self, generators: &HashMap<String, Generator>, context: &HashMap<String, Value>) -> Result<OptionalBody, String> {
    for (key, generator) in generators {
      self.apply_key(key, generator, context);
    };
    Ok(OptionalBody::Present(self.value.to_string().into()))
  }

  fn apply_key(&mut self, key: &String, generator: &Generator, context: &HashMap<String, Value>) {
    match parse_path_exp(key.clone()) {
      Ok(path_exp) => {
        let mut tree = Arena::new();
        let root = tree.new_node("".into());
        self.query_object_graph(&path_exp, &mut tree, root, self.value.clone());
        let expanded_paths = root.descendants(&tree).fold(Vec::<String>::new(), |mut acc, node_id| {
          let node = tree.index(node_id);
          if !node.data.is_empty() && node.first_child().is_none() {
            let path: Vec<String> = node_id.ancestors(&tree).map(|n| format!("{}", tree.index(n).data)).collect();
            if path.len() == path_exp.len() {
              acc.push(path.iter().rev().join("/"));
            }
          }
          acc
        });

        if !expanded_paths.is_empty() {
          for pointer_str in expanded_paths {
            match self.value.pointer_mut(&pointer_str) {
              Some(json_value) => match generator.generate_value(json_value.clone(), context) {
                Some(new_value) => *json_value = new_value,
                None => ()
              },
              None => ()
            }
          }
        } else if path_exp.len() == 1 {
          match generator.generate_value(self.value.clone(), context) {
            Some(new_value) => self.value = new_value,
            None => ()
          }
        }
      },
      Err(err) => log::warn!("Generator path '{}' is invalid, ignoring: {}", key, err)
    }
  }
}

/// Implementation of a content type handler for XML (currently unimplemented).
pub struct XmlHandler {
  /// XML document to apply the generators to.
  pub value: Element
}

#[derive(Debug)]
enum XmlNode<'a> {
  Element(&'a mut xmltree::Element),
  Children(&'a mut Vec<XMLNode>)
}

impl XmlHandler {
  fn find_child<'a>(&self, children: &'a mut Vec<XMLNode>, name: &str) -> Option<&'a mut Element> {
    children.iter_mut().find(|child| match child {
      XMLNode::Element(element) => element.name == name,
      _ => false
    }).map(|child| match child {
      XMLNode::Element(element) => Some(element),
      _ => None
    }).unwrap_or(None)
  }

  fn query_object_graph<F: FnOnce(&mut XmlNode)>(&self, path_exp: &Vec<PathToken>, mut doc: Element, callback: F) -> Option<Element> {
    let mut body_cursor = XmlNode::Element(&mut doc);
    let mut it = path_exp.iter();
    loop {
      match it.next() {
        Some(token) => {
          match token {
            &PathToken::Field(ref name) => {
              match body_cursor {
                XmlNode::Element(elem) => if elem.name == name.as_str() {
                  body_cursor = XmlNode::Children(&mut elem.children)
                } else {
                  return None
                },
                XmlNode::Children(children) => match self.find_child(children, name.as_str()) {
                  Some(node) => body_cursor = XmlNode::Element(node),
                  None => return None
                }
              }
            },
            // &PathToken::Index(index) => {
            //   match body_cursor.clone().as_array() {
            //     Some(list) => if list.len() > index {
            //       let node = tree.new_node(format!("{}", index));
            //       node_cursor.append(node, tree);
            //       body_cursor = list[index].clone();
            //       node_cursor = node;
            //     },
            //     None => return
            //   }
            // }
            // &PathToken::Star => {
            //   match body_cursor.clone().as_object() {
            //     Some(map) => {
            //       let remaining = it.by_ref().cloned().collect();
            //       for (key, val) in map {
            //         let node = tree.new_node(key.clone());
            //         node_cursor.append(node, tree);
            //         body_cursor = val.clone();
            //         self.query_object_graph(&remaining, tree, node, val.clone());
            //       }
            //     },
            //     None => return
            //   }
            // },
            // &PathToken::StarIndex => {
            //   match body_cursor.clone().as_array() {
            //     Some(list) => {
            //       let remaining = it.by_ref().cloned().collect();
            //       for (index, val) in list.iter().enumerate() {
            //         let node = tree.new_node(format!("{}", index));
            //         node_cursor.append(node, tree);
            //         body_cursor = val.clone();
            //         self.query_object_graph(&remaining, tree, node,val.clone());
            //       }
            //     },
            //     None => return
            //   }
            // },
            _ => ()
          }
        },
        None => break
      }
    }

    callback(&mut body_cursor);
    Some(doc)
  }
}

impl ContentTypeHandler<Element> for XmlHandler {
  fn process_body(&mut self, generators: &HashMap<String, Generator>, context: &HashMap<String, Value>) -> Result<OptionalBody, String> {
    for (key, generator) in generators {
      self.apply_key(key, generator, context);
    };
    let mut output = Vec::new();
    self.value.write(output.as_mut_slice()).map_err(|err| err.to_string())?;
    Ok(OptionalBody::Present(output.clone()))
  }

  fn apply_key(&mut self, key: &String, generator: &Generator, context: &HashMap<String, Value>) {
    match parse_path_exp(key.clone()) {
      Ok(path_exp) => {
        match self.query_object_graph(&path_exp, self.value.clone(), |node| {
          match node {
            XmlNode::Element(element) => {
              let text = element.get_text().map(|t| t.to_string()).unwrap_or(String::default());
              if let Some(val) = generator.generate_value(text, context) {
                element.children.push(XMLNode::Text(val));
              }
            },
            _ => ()
          }
        }) {
          Some(val) => self.value = val,
          None => ()
        }
      },
      Err(err) => warn!("Generator path '{}' is invalid, ignoring: {}", key, err)
    }
  }
}

/// Data structure for representing a collection of generators
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct Generators {
  /// Map of generator categories to maps of generators
  pub categories: HashMap<GeneratorCategory, HashMap<String, Generator>>
}

impl Generators {
  /// If the generators are empty (that is there are no rules assigned to any categories)
  pub fn is_empty(&self) -> bool {
    self.categories.values().all(|category| category.is_empty())
  }

  /// If the generators are not empty (that is there is at least one rule assigned to a category)
  pub fn is_not_empty(&self) -> bool {
    self.categories.values().any(|category| !category.is_empty())
  }

  /// Loads the generators for a JSON map
  pub fn load_from_map(&mut self, map: &serde_json::Map<String, Value>) {
    for (k, v) in map {
      match v {
        &Value::Object(ref map) =>  match GeneratorCategory::from_str(k) {
          Ok(ref category) => match category {
            &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
              self.parse_generator_from_map(category, map, None);
            },
            _ => for (sub_k, sub_v) in map {
              match sub_v {
                &Value::Object(ref map) => self.parse_generator_from_map(category, map, Some(sub_k.clone())),
                _ => log::warn!("Ignoring invalid generator JSON '{}' -> {:?}", sub_k, sub_v)
              }
            }
          },
          Err(err) => log::warn!("Ignoring generator with invalid category '{}' - {}", k, err)
        },
        _ => log::warn!("Ignoring invalid generator JSON '{}' -> {:?}", k, v)
      }
    }
  }

  fn parse_generator_from_map(&mut self, category: &GeneratorCategory,
                              map: &serde_json::Map<String, Value>, subcat: Option<String>) {
    match map.get("type") {
      Some(gen_type) => match gen_type {
        &Value::String(ref gen_type) => match Generator::from_map(gen_type, map) {
          Some(generator) => match subcat {
            Some(s) => self.add_generator_with_subcategory(category, s, generator),
            None => self.add_generator(category, generator)
          },
          None => log::warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
        },
        _ => log::warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
      },
      None => log::warn!("Ignoring invalid generator JSON '{:?}' with no type attribute -> {:?}", category, map)
    }
  }

  fn to_json(&self) -> Value {
    Value::Object(self.categories.iter().fold(serde_json::Map::new(), |mut map, (name, category)| {
      let cat: String = name.clone().into();
      match name {
        &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
          match category.get("") {
            Some(generator) => {
              map.insert(cat.clone(), generator.to_json());
            },
            None => ()
          }
        },
        _ => {
          for (key, val) in category {
            map.insert(cat.clone(), json!({ key.as_str(): val.to_json() }));
          }
        }
      }
      map
    }))
  }

  /// Adds the generator to the category (body, headers, etc.)
  pub fn add_generator(&mut self, category: &GeneratorCategory, generator: Generator) {
    self.add_generator_with_subcategory(category, "", generator);
  }

  /// Adds a generator to the category with a sub-category key (i.e. headers or query parameters)
  pub fn add_generator_with_subcategory<S: Into<String>>(&mut self, category: &GeneratorCategory,
                                                         subcategory: S, generator: Generator) {
    let category_map = self.categories.entry(category.clone()).or_insert(HashMap::new());
    category_map.insert(subcategory.into(), generator.clone());
  }

  /// If there are generators for the provided category, invokes the closure for all keys and values
  /// in the category.
  pub fn apply_generator<F>(&self, category: &GeneratorCategory, mut closure: F)
    where F: FnMut(&String, &Generator) {
    if self.categories.contains_key(category) && !self.categories[category].is_empty() {
      for (key, value) in self.categories[category].clone() {
        closure(&key, &value)
      }
    }
  }

  /// Applies all the body generators to the body and returns a new body (if anything was applied).
  pub fn apply_body_generators(&self, body: &OptionalBody, content_type: DetectedContentType, context: &HashMap<String, Value>) -> Result<OptionalBody, String> {
    if body.is_present() && self.categories.contains_key(&GeneratorCategory::BODY) &&
      !self.categories[&GeneratorCategory::BODY].is_empty() {
      let generators = &self.categories[&GeneratorCategory::BODY];
      match content_type {
        DetectedContentType::Json => {
          let result: Result<Value, serde_json::Error> = serde_json::from_slice(&body.value());
          match result {
            Ok(val) => {
              let mut handler = JsonHandler { value: val };
              handler.process_body(&generators, context)
            },
            Err(err) => Err(format!("Failed to parse the body, so not applying any generators: {}", err))
          }
        },
        DetectedContentType::Xml => match Element::parse(body.value().as_slice()) {
          Ok(val) => {
            let mut handler = XmlHandler { value: val };
            handler.process_body(&generators, context)
          },
          Err(err) => Err(format!("Failed to parse the body, so not applying any generators: {}", err))
        },
        _ => Ok(body.clone())
      }
    } else {
      Ok(body.clone())
    }
  }
}

impl Hash for Generators {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (k, v) in self.categories.iter() {
      k.hash(state);
      for (k2, v2) in v.iter() {
        k2.hash(state);
        v2.hash(state);
      }
    }
  }
}

impl Default for Generators {
  fn default() -> Self {
    Generators {
      categories: hashmap!{}
    }
  }
}

/// Parses the generators from the Value structure
pub fn generators_from_json(value: &Value) -> Generators {
  let mut generators = Generators::default();
  match value {
    &Value::Object(ref m) => match m.get("generators") {
      Some(gen_val) => match gen_val {
        &Value::Object(ref m) => generators.load_from_map(m),
        _ => ()
      },
      None => ()
    },
    _ => ()
  }
  generators
}

/// Generates a Value structure for the provided generators
pub fn generators_to_json(generators: &Generators, spec_version: &PactSpecification) -> Value {
  match spec_version {
    &PactSpecification::V3 => generators.to_json(),
    _ => Value::Null
  }
}

/// Macro to make constructing generators easy
/// Example usage:
/// ```ignore
/// generators! {
///   "HEADER" => {
///     "A" => Generator::Uuid
///   }
/// }
///```
#[macro_export]
macro_rules! generators {
  (
    $( $category:expr => {
      $( $subname:expr => $generator:expr ), *
    }), *
  ) => {{
    let mut _generators = $crate::models::generators::Generators::default();

  $(
    let _cat = $crate::models::generators::GeneratorCategory::from_str($category).unwrap();
    $(
      _generators.add_generator_with_subcategory(&_cat, $subname, $generator);
    )*
  )*

    _generators
  }};

  (
    $( $category:expr => $generator:expr ), *
  ) => {{
    let mut _generators = $crate::models::generators::Generators::default();
    $(
      let _cat = $crate::models::generators::GeneratorCategory::from_str($category).unwrap();
      _generators.add_generator(&_cat, $generator);
    )*
    _generators
  }};
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use expectest::expect;
  use super::Generator;
  use std::str::FromStr;
  use hamcrest2::*;

  #[test]
  fn rules_are_empty_when_there_are_no_categories() {
    expect!(Generators::default().is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_empty_when_there_are_only_empty_categories() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap!{}
            }
        }.is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_not_empty_when_there_is_a_nonempty_category() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap! {
                    "a".to_string() => Generator::RandomInt(1, 10)
                }
            }
        }.is_empty()).to(be_false());
  }

  #[test]
  fn matchers_from_json_test() {
    expect!(generators_from_json(&Value::Null).categories.iter()).to(be_empty());
  }

  #[test]
  fn generators_macro_test() {
    expect!(generators!{}).to(be_equal_to(Generators::default()));

    let mut expected = Generators::default();
    expected.add_generator(&GeneratorCategory::STATUS, Generator::RandomInt(400, 499));
    expect!(generators!{
      "STATUS" => Generator::RandomInt(400, 499)
    }).to(be_equal_to(expected));

    expected = Generators::default();
    expected.add_generator_with_subcategory(&GeneratorCategory::BODY, "$.a.b",
                           Generator::RandomInt(1, 10));
    expect!(generators!{
      "BODY" => {
        "$.a.b" => Generator::RandomInt(1, 10)
      }
    }).to(be_equal_to(expected));
  }

  #[test]
  fn generator_from_json_test() {
    expect!(Generator::from_map(&s!(""), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Invalid"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("uuid"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Uuid"), &serde_json::Map::new())).to(be_some().value(Generator::Uuid));
    expect!(Generator::from_map(&s!("RandomBoolean"), &serde_json::Map::new())).to(be_some().value(Generator::RandomBoolean));
  }

  #[test]
  fn randomint_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomInt"), &serde_json::Map::new())).to(be_some().value(Generator::RandomInt(0, 10)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 10)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "max": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 5)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 5, "max": 6 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 6)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 0, "max": 1234567890 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 1234567890)));
  }

  #[test]
  fn random_decimal_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomDecimal"), &serde_json::Map::new())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map(&s!("RandomDecimal"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map(&s!("RandomDecimal"), &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(5)));
  }

  #[test]
  fn random_hexadecimal_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &serde_json::Map::new())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(5)));
  }

  #[test]
  fn random_string_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomString"), &serde_json::Map::new())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map(&s!("RandomString"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map(&s!("RandomString"), &json!({ "size": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(5)));
  }

  #[test]
  fn regex_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Regex"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Regex"), &json!({ "min": 5 }).as_object().unwrap())).to(be_none());
    expect!(Generator::from_map(&s!("Regex"), &json!({ "regex": "\\d+" }).as_object().unwrap())).to(be_some().value(Generator::Regex(s!("\\d+"))));
    expect!(Generator::from_map(&s!("Regex"), &json!({ "regex": 5 }).as_object().unwrap())).to(be_some().value(Generator::Regex(s!("5"))));
  }

  #[test]
  fn date_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Date"), &serde_json::Map::new())).to(be_some().value(Generator::Date(None)));
    expect!(Generator::from_map(&s!("Date"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(None)));
    expect!(Generator::from_map(&s!("Date"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Date(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("Date"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(Some(s!("5")))));
  }

  #[test]
  fn time_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Time"), &serde_json::Map::new())).to(be_some().value(Generator::Time(None)));
    expect!(Generator::from_map(&s!("Time"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(None)));
    expect!(Generator::from_map(&s!("Time"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Time(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("Time"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(Some(s!("5")))));
  }

  #[test]
  fn datetime_generator_from_json_test() {
    expect!(Generator::from_map(&s!("DateTime"), &serde_json::Map::new())).to(be_some().value(Generator::DateTime(None)));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(None)));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some(s!("5")))));
  }

  #[test]
  fn generator_to_json_test() {
    expect!(Generator::RandomInt(5, 15).to_json()).to(be_equal_to(json!({
      "type": "RandomInt",
      "min": 5,
      "max": 15
    })));
    expect!(Generator::Uuid.to_json()).to(be_equal_to(json!({
      "type": "Uuid"
    })));
    expect!(Generator::RandomDecimal(5).to_json()).to(be_equal_to(json!({
      "type": "RandomDecimal",
      "digits": 5
    })));
    expect!(Generator::RandomHexadecimal(5).to_json()).to(be_equal_to(json!({
      "type": "RandomHexadecimal",
      "digits": 5
    })));
    expect!(Generator::RandomString(5).to_json()).to(be_equal_to(json!({
      "type": "RandomString",
      "size": 5
    })));
    expect!(Generator::Regex(s!("\\d+")).to_json()).to(be_equal_to(json!({
      "type": "Regex",
      "regex": "\\d+"
    })));
    expect!(Generator::RandomBoolean.to_json()).to(be_equal_to(json!({
      "type": "RandomBoolean"
    })));

    expect!(Generator::Date(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "Date",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Date(None).to_json()).to(be_equal_to(json!({
      "type": "Date"
    })));
    expect!(Generator::Time(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "Time",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Time(None).to_json()).to(be_equal_to(json!({
      "type": "Time"
    })));
    expect!(Generator::DateTime(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "DateTime",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::DateTime(None).to_json()).to(be_equal_to(json!({
      "type": "DateTime"
    })));
  }

  #[test]
  fn generate_decimal_test() {
    assert_that!(generate_decimal(4), matches_regex(r"^\d{4}$"));
    assert_that!(generate_hexadecimal(4), matches_regex(r"^[0-9A-F]{4}$"));
  }

  #[test]
  fn generate_int_with_max_int_test() {
    assert_that!(Generator::RandomInt(0, i32::max_value()).generate_value(0,
      &hashmap!{}).unwrap().to_string(), matches_regex(r"^\d+$"));
  }
}
