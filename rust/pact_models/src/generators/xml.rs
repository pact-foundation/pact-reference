
use std::collections::HashMap;

use rand::Rng;
use serde_json::Value;
use sxd_document::dom::{Document, Element, Attribute, ChildOfRoot, ChildOfElement};
use sxd_document::writer::format_document;
use tracing::{debug, error, trace};
use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::generators::{ContentTypeHandler, Generator, GeneratorTestMode, VariantMatcher, GenerateValue};
use crate::path_exp::DocPath;
use crate::bodies::OptionalBody;

/// Implementation of a content type handler for XML.
pub struct XmlHandler<'a> {
  /// XML document to apply the generators to.
  pub value: Document<'a>
}

impl <'a> ContentTypeHandler<String> for XmlHandler<'a> {
  fn process_body(
    &mut self,
    generators: &HashMap<DocPath, Generator>,
    mode: &GeneratorTestMode,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) -> Result<OptionalBody, String> {
    let mut filtered: Vec<_> = generators.iter()
      .filter(|(_, g)| g.corresponds_to_mode(mode))
      .collect();
    filtered.sort_by_key(|(_, g)| g.processing_category().priority());

    for (key, generator) in filtered {
      debug!("Applying generator {:?} (category: {:?}) to key {}", generator, generator.processing_category(), key);
      self.apply_key(key, generator, context, matcher);
    }

    let mut w = Vec::new();
    match format_document(&self.value, &mut w) {
      Ok(()) => Ok(OptionalBody::Present(w.into(), Some("application/xml".into()), None)),
      Err(err) => Err(anyhow!("Failed to format xml document: {}", err).to_string())
    }
  }

  fn apply_key(
    &mut self,
    key: &DocPath,
    generator: &Generator,
    context: &HashMap<&str, Value>,
    matcher: &Box<dyn VariantMatcher + Send + Sync>
  ) {
    for child in self.value.root().children() {
      if let ChildOfRoot::Element(el) = child {
        generate_values_for_xml_element(&el, key, generator, context, matcher, vec!["$".to_string()])
      }
    }
  }
}

fn generate_values_for_xml_element<'a>(
  el: &Element<'a>,
  key: &DocPath,
  generator: &Generator,
  context: &HashMap<&str, Value>,
  matcher: &Box<dyn VariantMatcher + Send + Sync>,
  parent_path: Vec<String>
) {
  trace!("generate_values_for_xml_element(parent_path: '{:?}')", parent_path);

  if key.len() < parent_path.len() + 2 {
    return
  }

  let mut path = parent_path.clone();
  path.push(xml_element_name(el));

  if let Generator::RandomArray(min, max) = generator {
    if key.len() == path.len() + 1 {
      duplicate_elements(el, key, *min, *max);
      return
    }
  }

  if generate_values_for_xml_attribute(&el, key, generator, context, matcher, path.clone()) {
    return
  }

  if generate_values_for_xml_text(&el, key, generator, context, matcher, path.clone()) {
    return
  }

  if key.len() < path.len() + 2 {
    return
  }

  for child in el.children() {
    if let ChildOfElement::Element(child_el) = child {
      generate_values_for_xml_element(&child_el, key, generator, context, matcher, path.clone())
    }
  }
}

fn generate_values_for_xml_attribute<'a>(
  el: &Element<'a>,
  key: &DocPath,
  generator: &Generator,
  context: &HashMap<&str, Value>,
  matcher: &Box<dyn VariantMatcher + Send + Sync>,
  path: Vec<String>
) -> bool {
  trace!("generate_values_for_xml_attribute(path: '{:?}')", path);

  if let Some(v) = key.last_field() {
    if v.starts_with("@") {
      for attr in el.attributes() {
        let mut attr_path = path.clone();
        attr_path.push(format!("@{}", xml_attribute_name(attr)));
        if key.matches_path_exactly(attr_path.iter().map(|p| p.as_str()).collect_vec().as_slice()) {
          debug!("Generating xml attribute value at '{:?}'", attr_path);
          match generator.generate_value(&attr.value().to_string(), context, matcher) {
            Ok(new_value) => {
              let new_attr = el.set_attribute_value(attr.name(), new_value.as_str());
              new_attr.set_preferred_prefix(attr.preferred_prefix());
              debug!("Generated value for attribute '{}' of xml element '{}'", xml_attribute_name(attr), xml_element_name(el));
            }
            Err(err) => {
              error!("Failed to generate the attribute, will use the original: {}", err);
            }
          }
          return true
        }
      }
    }
  };
  false
}


fn generate_values_for_xml_text<'a>(
  el: &Element<'a>,
  key: &DocPath,
  generator: &Generator,
  context: &HashMap<&str, Value>,
  matcher: &Box<dyn VariantMatcher + Send + Sync>,
  path: Vec<String>
) -> bool {
  trace!("generate_values_for_xml_text(path: '{:?}')", path);

  let mut txt_path = path.clone();
  txt_path.push("#text".to_string());

  if !key.matches_path_exactly(txt_path.iter().map(|p| p.as_str()).collect_vec().as_slice()) {
    return false
  }

  let mut has_txt = false;
  for child in el.children() {
    if let ChildOfElement::Text(txt) = child {
      has_txt = true;
      debug!("Generating xml text at '{:?}'", txt_path);
      match generator.generate_value(&txt.text().to_string(), context, matcher) {
        Ok(new_value) => {
          txt.set_text(new_value.as_str());
          debug!("Generated value for text of xml element '{}'", xml_element_name(el));
        }
        Err(err) => {
          error!("Failed to generate the text, will use the original: {}", err);
        }
      }
    }
  }
  if !has_txt {
    debug!("Generating xml text at '{:?}'", txt_path);
    match generator.generate_value(&"".to_string(), context, matcher) {
      Ok(new_value) => {
        let text = el.document().create_text(new_value.as_str());
        el.append_child(text);
        debug!("Generated value for text of xml element '{}'", xml_element_name(el));
      }
      Err(err) => {
        error!("Failed to generate the text, will use the original: {}", err);
      }
    }
  }
  true
}

fn xml_element_name(el: &Element) -> String {
  if let Some(ns) = el.preferred_prefix() {
    format!("{}:{}", ns, el.name().local_part())
  } else {
    el.name().local_part().to_string()
  }
}

fn xml_attribute_name(attr: Attribute) -> String {
  if let Some(ns) = attr.preferred_prefix() {
    format!("{}:{}", ns, attr.name().local_part())
  } else {
    attr.name().local_part().to_string()
  }
}

fn duplicate_elements<'a>(el: &Element<'a>, key: &DocPath, min: u16, max: u16) {
  if min > max {
    error!("RandomArray: invalid bounds - min ({}) is greater than max ({})", min, max);
    return;
  }

  let length = rand::rng().random_range(min..max.saturating_add(1));
  let last_field = key.last_field().unwrap_or("");
  let element_name = last_field.trim_start_matches('@');

  let children = el.children();
  let matching_children: Vec<_> = children.iter().filter_map(|c| {
    if let ChildOfElement::Element(e) = c {
      if e.name().local_part() == element_name {
        Some(e)
      } else {
        None
      }
    } else {
      None
    }
  }).collect();

  if matching_children.is_empty() {
    return;
  }

  let template = matching_children[0];
  let items_to_add = length.saturating_sub(1);

  for _ in 0..items_to_add {
    let cloned = clone_element(&el.document(), template);
    el.append_child(cloned);
  }
}

fn clone_element<'a>(doc: &Document<'a>, el: &Element<'a>) -> Element<'a> {
  let new_el = doc.create_element(el.name().local_part());

  if let Some(prefix) = el.preferred_prefix() {
    new_el.set_preferred_prefix(Some(prefix));
  }

  for attr in el.attributes() {
    let new_attr = new_el.set_attribute_value(attr.name().local_part(), attr.value());
    if let Some(prefix) = attr.preferred_prefix() {
      new_attr.set_preferred_prefix(Some(prefix));
    }
  }

  for child in el.children() {
    match child {
      ChildOfElement::Element(child_el) => {
        let cloned_child = clone_element(doc, &child_el);
        new_el.append_child(cloned_child);
      }
      ChildOfElement::Text(txt) => {
        let new_text = doc.create_text(txt.text());
        new_el.append_child(new_text);
      }
      _ => {}
    }
  }

  new_el
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use test_log::test;
  use maplit::hashmap;
  use sxd_document::Package;

  use crate::generators::NoopVariantMatcher;

  use super::*;
  use super::Generator;

  #[test]
  fn applies_no_generator() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{}, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a/>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_non_existing_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.b['#text']") => Generator::RandomInt(0, 10),
      DocPath::new_unwrap("$.b['@att']") => Generator::RandomInt(0, 10)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a/>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_empty_text() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>999</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_empty_text_beside_an_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_element("b"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a><b/>999</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_non_empty_text_before_an_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    e.append_child(d.create_element("b"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>999<b/></a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_non_empty_text_after_an_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_element("b"));
    e.append_child(d.create_text("1"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a><b/>999</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_non_empty_text() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>999</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_multiple_non_empty_texts() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    e.append_child(d.create_element("b"));
    e.append_child(d.create_text("2"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>999<b/>999</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_multiple_elements() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    r.append_child(e);
    let e = d.create_element("a");
    e.append_child(d.create_text("2"));
    r.append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a>999</a><a>999</a></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_an_element_with_namespace() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element(("http://example.com/namespace", "a"));
    e.set_preferred_prefix(Some("n"));
    e.append_child(d.create_text("1"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.n:a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><n:a xmlns:n='http://example.com/namespace'>999</n:a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_multiple_elements_with_namespace() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let e = d.create_element(("http://example.com/namespace1", "a"));
    e.set_preferred_prefix(Some("n1"));
    e.append_child(d.create_text("1"));
    r.append_child(e);
    let e = d.create_element(("http://example.com/namespace2", "a"));
    e.set_preferred_prefix(Some("n2"));
    e.append_child(d.create_text("2"));
    r.append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.n1:a['#text']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.root.n2:a['#text']") => Generator::RandomInt(222, 222)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><n1:a xmlns:n1='http://example.com/namespace1'>111</n1:a><n2:a xmlns:n2='http://example.com/namespace2'>222</n2:a></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_an_element_with_mixed_namespace() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let e = d.create_element(("http://example.com/namespace", "a"));
    e.set_preferred_prefix(Some("n"));
    e.append_child(d.create_text("1"));
    r.append_child(e);
    let e = d.create_element("a");
    e.append_child(d.create_text("2"));
    r.append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.n:a['#text']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.root.a['#text']") => Generator::RandomInt(222, 222),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><n:a xmlns:n='http://example.com/namespace'>111</n:a><a>222</a></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_an_attribute() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.set_attribute_value("attr", "1");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['@attr']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a attr='999'/>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_multiple_attributes() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.set_attribute_value("attr1", "1");
    e.set_attribute_value("attr2", "2");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let _ = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['@attr1']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.a['@attr2']") => Generator::RandomInt(222, 222)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(e.attribute("attr1").unwrap().value()).to(be_equal_to("111"));
    expect!(e.attribute("attr2").unwrap().value()).to(be_equal_to("222"));
  }

  #[test]
  fn applies_the_generator_to_multiple_attributes_with_namespace() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    let a = e.set_attribute_value(("http://example.com/namespace1", "attr"), "1");
    a.set_preferred_prefix(Some("n1"));
    let a = e.set_attribute_value(("http://example.com/namespace2", "attr"), "2");
    a.set_preferred_prefix(Some("n2"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let _ = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['@n1:attr']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.a['@n2:attr']") => Generator::RandomInt(222, 222)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(e.attribute(("http://example.com/namespace1", "attr")).unwrap().value()).to(be_equal_to("111"));
    expect!(e.attribute(("http://example.com/namespace2", "attr")).unwrap().value()).to(be_equal_to("222"));
  }

  #[test]
  fn applies_the_generator_to_multiple_attributes_with_mixed_namespace() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    let a = e.set_attribute_value(("http://example.com/namespace", "attr"), "1");
    a.set_preferred_prefix(Some("n"));
    e.set_attribute_value("attr", "2");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let _ = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['@n:attr']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.a['@attr']") => Generator::RandomInt(222, 222)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(e.attribute(("http://example.com/namespace", "attr")).unwrap().value()).to(be_equal_to("111"));
    expect!(e.attribute("attr").unwrap().value()).to(be_equal_to("222"));
  }

  #[test]
  fn applies_the_generator_to_text_and_attribute() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    e.set_attribute_value("attr", "2");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.a['@attr']") => Generator::RandomInt(222, 222),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a attr='222'>111</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_and_attribute_of_nested_elements() {
    let p = Package::new();
    let d = p.as_document();
    let ea = d.create_element("a");
    ea.append_child(d.create_text("1"));
    d.root().append_child(ea);
    let eb = d.create_element("b");
    eb.set_attribute_value("attr", "2");
    ea.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(111, 111),
      DocPath::new_unwrap("$.a.b['@attr']") => Generator::RandomInt(222, 222),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>111<b attr='222'/></a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_attribute_of_multiple_elements() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let e = d.create_element("a");
    e.set_attribute_value("attr", "1");
    r.append_child(e);
    let e = d.create_element("a");
    e.set_attribute_value("attr", "2");
    r.append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.a['@attr']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a attr='999'/><a attr='999'/></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_multiple_elements_in_different_path() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    let e = d.create_element("d");
    e.append_child(d.create_text("1"));
    ec.append_child(e);
    let e = d.create_element("d");
    e.append_child(d.create_text("2"));
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    let e = d.create_element("e");
    e.append_child(d.create_text("3"));
    ec.append_child(e);
    let e = d.create_element("e");
    e.append_child(d.create_text("4"));
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.*.c.*['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c><d>999</d><d>999</d></c></a><b><c><e>999</e><e>999</e></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_attribute_of_multiple_elements_in_different_path() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    let e = d.create_element("d");
    e.set_attribute_value("attr", "1");
    ec.append_child(e);
    let e = d.create_element("d");
    e.set_attribute_value("attr", "2");
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    let e = d.create_element("e");
    e.set_attribute_value("attr", "3");
    ec.append_child(e);
    let e = d.create_element("e");
    e.set_attribute_value("attr", "4");
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.*.c.*['@attr']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c><d attr='999'/><d attr='999'/></c></a><b><c><e attr='999'/><e attr='999'/></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_unicode_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("俄语");
    e.append_child(d.create_text("данные"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.俄语['#text']") => Generator::Regex("语言".to_string()),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><俄语>语言</俄语>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_attribute_of_unicode_element() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("俄语");
    e.set_attribute_value("լեզու", "ռուսերեն");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.俄语['@լեզու']") => Generator::Regex("😊".to_string()),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><俄语 լեզու='😊'/>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_beside_comment() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    e.append_child(d.create_comment("some explanation"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>999<!--some explanation--></a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_and_escaping() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.append_child(d.create_text("1"));
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['#text']") => Generator::Regex("<foo/>".to_string()),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a>&lt;foo/&gt;</a>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_attribute_and_escaping() {
    let p = Package::new();
    let d = p.as_document();
    let e = d.create_element("a");
    e.set_attribute_value("attr", "1");
    d.root().append_child(e);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.a['@attr']") => Generator::Regex("' new-attr='\"val".to_string()),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><a attr='&apos; new-attr=&apos;&quot;val'/>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_attribute_of_elements_in_middle() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    ec.set_attribute_value("attr", "1");
    let e = d.create_element("d");
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    ec.set_attribute_value("attr", "2");
    let e = d.create_element("e");
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.*.c['@attr']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c attr='999'><d/></c></a><b><c attr='999'><e/></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_the_generator_to_text_of_elements_in_middle() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    ec.append_child(d.create_text("1"));
    let e = d.create_element("d");
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    ec.append_child(d.create_text("2"));
    let e = d.create_element("e");
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.*.c['#text']") => Generator::RandomInt(999, 999)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c>999<d/></c></a><b><c>999<e/></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn not_apply_generator_to_text_of_elements_located_too_deep() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    let e = d.create_element("d");
    e.append_child(d.create_text("1"));
    ec.append_child(e);
    let e = d.create_element("d");
    e.append_child(d.create_text("2"));
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    let e = d.create_element("e");
    e.append_child(d.create_text("3"));
    ec.append_child(e);
    let e = d.create_element("e");
    e.append_child(d.create_text("4"));
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.*.d['#text']") => Generator::RandomInt(999, 999),
      DocPath::new_unwrap("$.*.e['#text']") => Generator::RandomInt(999, 999),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c><d>1</d><d>2</d></c></a><b><c><e>3</e><e>4</e></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn not_apply_generator_to_attribute_of_elements_located_too_deep() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);
    let ea = d.create_element("a");
    let ec = d.create_element("c");
    let e = d.create_element("d");
    e.set_attribute_value("attr", "1");
    ec.append_child(e);
    let e = d.create_element("d");
    e.set_attribute_value("attr", "2");
    ec.append_child(e);
    ea.append_child(ec);
    r.append_child(ea);
    let eb = d.create_element("b");
    let ec = d.create_element("c");
    let e = d.create_element("e");
    e.set_attribute_value("attr", "3");
    ec.append_child(e);
    let e = d.create_element("e");
    e.set_attribute_value("attr", "4");
    ec.append_child(e);
    eb.append_child(ec);
    r.append_child(eb);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.*.d['@attr']") => Generator::RandomInt(999, 999),
      DocPath::new_unwrap("$.*.e['@attr']") => Generator::RandomInt(999, 999),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><a><c><d attr='1'/><d attr='2'/></c></a><b><c><e attr='3'/><e attr='4'/></c></b></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_random_array_generator_to_duplicate_elements() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("people");
    d.root().append_child(r);

    let person = d.create_element("person");
    person.set_attribute_value("id", "1");
    person.append_child(d.create_text("John"));
    r.append_child(person);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.people.person") => Generator::RandomArray(2, 3)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    let xml_str = result.unwrap().value().unwrap();
    let xml_str = String::from_utf8_lossy(&xml_str);

    let person_count = xml_str.matches("<person ").count() + xml_str.matches("<person/").count();
    expect!(person_count).to(be_ge(2));
    expect!(person_count).to(be_le(3));
  }

  #[test]
  fn applies_random_array_generator_with_exact_count() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("items");
    d.root().append_child(r);

    let item = d.create_element("item");
    item.set_attribute_value("name", "test");
    r.append_child(item);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.items.item") => Generator::RandomArray(3, 3)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><items><item name='test'/><item name='test'/><item name='test'/></items>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_random_array_generator_with_nested_elements() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);

    let container = d.create_element("container");
    r.append_child(container);

    let element = d.create_element("element");
    element.append_child(d.create_text("value"));
    container.append_child(element);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.container.element") => Generator::RandomArray(2, 2)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><container><element>value</element><element>value</element></container></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_random_array_generator_with_cloned_attributes_and_text() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("people");
    d.root().append_child(r);

    let person = d.create_element("person");
    person.set_attribute_value("id", "123");
    person.set_attribute_value("name", "John");
    r.append_child(person);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.people.person") => Generator::RandomArray(3, 3)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><people><person id='123' name='John'/><person id='123' name='John'/><person id='123' name='John'/></people>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_random_array_generator_with_nested_generators() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("items");
    d.root().append_child(r);

    let item = d.create_element("item");
    item.set_attribute_value("name", "xxx");
    item.set_attribute_value("price", "12");
    r.append_child(item);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.items.item") => Generator::RandomArray(2, 4),
      DocPath::new_unwrap("$.items.item['@name']") => Generator::RandomString(5),
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    let xml_str = result.unwrap().value().unwrap();
    let xml_str = String::from_utf8_lossy(&xml_str);

    let item_count = xml_str.matches("<item ").count() + xml_str.matches("<item/").count();
    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(4));

    let name_values: Vec<_> = xml_str.split("name='").skip(1).map(|s| s.split('\'').next().unwrap().to_string()).collect();
    expect!(name_values.len()).to(be_equal_to(item_count));

    for i in 1..name_values.len() {
        expect!(&name_values[i]).not_to(be_equal_to(&name_values[i - 1]));
    }
  }

  #[test]
  fn applies_random_array_generator_with_min_max_zero() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("items");
    d.root().append_child(r);

    let item = d.create_element("item");
    item.set_attribute_value("value", "1");
    r.append_child(item);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.items.item") => Generator::RandomArray(0, 0)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><items><item value='1'/></items>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_multiple_independent_array_generators() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("root");
    d.root().append_child(r);

    let items = d.create_element("items");
    let item = d.create_element("item");
    item.set_attribute_value("value", "1");
    items.append_child(item);
    r.append_child(items);

    let other = d.create_element("other");
    let other_item = d.create_element("entry");
    other_item.set_attribute_value("x", "2");
    other.append_child(other_item);
    r.append_child(other);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.root.items.item") => Generator::RandomArray(2, 2),
      DocPath::new_unwrap("$.root.other.entry") => Generator::RandomArray(3, 3)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><root><items><item value='1'/><item value='1'/></items><other><entry x='2'/><entry x='2'/><entry x='2'/></other></root>".into(), Some("application/xml".into()), None)));
  }

  #[test]
  fn applies_random_array_generator_with_nested_elements_and_attributes() {
    let p = Package::new();
    let d = p.as_document();
    let r = d.create_element("people");
    d.root().append_child(r);

    let person = d.create_element("person");
    person.set_attribute_value("id", "1");
    person.set_attribute_value("name", "John");

    let address = d.create_element("address");
    address.set_attribute_value("city", "NYC");
    person.append_child(address);

    r.append_child(person);

    let mut xml_handler = XmlHandler { value: d };

    let result = xml_handler.process_body(&hashmap!{
      DocPath::new_unwrap("$.people.person") => Generator::RandomArray(2, 2)
    }, &GeneratorTestMode::Consumer, &hashmap!{}, &NoopVariantMatcher.boxed());

    expect!(result.unwrap()).to(be_equal_to(OptionalBody::Present("<?xml version='1.0'?><people><person id='1' name='John'><address city='NYC'/></person><person id='1' name='John'><address city='NYC'/></person></people>".into(), Some("application/xml".into()), None)));
  }
}
