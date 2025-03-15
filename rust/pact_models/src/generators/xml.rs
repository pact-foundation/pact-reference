
use std::collections::HashMap;

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
    for (key, generator) in generators {
      if generator.corresponds_to_mode(mode) {
        debug!("Applying generator {:?} to key {}", generator, key);
        self.apply_key(key, generator, context, matcher);
      }
    };

    let mut w = Vec::new();
    match format_document(&self.value, &mut w) {
      Ok(()) => Ok(OptionalBody::Present(w.into(), Some("application/xml".into()), None)),
      Err(err) => Err(anyhow!("Failed to format xml document: {}", err).to_string())
    }
  }

  fn apply_key(
    &mut self,
    key: &DocPath,
    generator: &dyn GenerateValue<String>,
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
  generator: &dyn GenerateValue<String>,
  context: &HashMap<&str, Value>,
  matcher: &Box<dyn VariantMatcher + Send + Sync>,
  parent_path: Vec<String>
) {
  trace!("generate_values_for_xml_element(parent_path: '{:?}')", parent_path);
  let mut path = parent_path.clone();
  path.push(xml_element_name(el));
  trace!("Generating xml values at '{:?}'", path);
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
          return
        }
        Err(err) => {
          error!("Failed to generate the attribute, will use the original: {}", err);
          return
        }
      }
    }
  }
  let mut txt_path = path.clone();
  txt_path.push("#text".to_string());
  let mut has_txt = false;
  for child in el.children() {
    if let ChildOfElement::Text(txt) = child {
      has_txt = true;
      if key.matches_path_exactly(txt_path.iter().map(|p| p.as_str()).collect_vec().as_slice()) {
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
    if let ChildOfElement::Element(child_el) = child {
      generate_values_for_xml_element(&child_el, key, generator, context, matcher, path.clone())
    }
  }
  if key.matches_path_exactly(txt_path.iter().map(|p| p.as_str()).collect_vec().as_slice()) && !has_txt {
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
}
