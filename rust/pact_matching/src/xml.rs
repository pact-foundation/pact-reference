use std::collections::btree_map::BTreeMap;

use anyhow::anyhow;
use bytes::Bytes;
use itertools::{EitherOrBoth, Itertools};
use log::*;
use maplit::*;
use onig::Regex;
use sxd_document::dom::*;
use sxd_document::QName;

use pact_models::bodies::OptionalBody;
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::MatchingRule;
use pact_models::xml_utils::parse_bytes;

use crate::matchers::*;
use crate::MatchingContext;

use super::DiffConfig;
use super::Mismatch;

pub fn match_xml(expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  match (expected.body(), actual.body()) {
    (OptionalBody::Missing, _) => (),
    (OptionalBody::Empty, _) => (),
    (OptionalBody::Null, _) => (),
    (OptionalBody::Present(expected_body, _), OptionalBody::Present(actual_body, _)) => {
      let expected_result = parse_bytes(expected_body);
      let actual_result = parse_bytes(actual_body);

      if expected_result.is_err() || actual_result.is_err() {
        match expected_result {
          Err(e) => {
            mismatches.push(Mismatch::BodyMismatch {
              path: "$".to_string(),
              expected: expected.body().value(),
              actual: actual.body().value(),
              mismatch: format!("Failed to parse the expected body: '{:?}'", e)});
          },
          _ => ()
        }
        match actual_result {
          Err(e) => {
            mismatches.push(Mismatch::BodyMismatch {
              path: "$".to_string(),
              expected: expected.body().value(),
              actual: actual.body().value(),
              mismatch: format!("Failed to parse the actual body: '{:?}'", e)});
          },
          _ => ()
        }
      } else {
        let expected_package = expected_result.unwrap();
        let expected_root = expected_package.as_document().root();
        let expected_root_node = expected_root.children().iter().cloned().find(|n| n.element().is_some());
        let actual_package = actual_result.unwrap();
        let actual_root = actual_package.as_document().root();
        let actual_root_node = actual_root.children().iter().cloned().find(|n| n.element().is_some());
        let element = expected_root_node.unwrap().element().unwrap();
        let name = name(element.name());
        let path = vec!["$", name.as_str()];
        compare_element(&path, &element, &actual_root_node.unwrap().element().unwrap(), &mut mismatches, &context);
      }
    },
    _ => {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: expected.body().value(),
        actual: None,
        mismatch: format!("Expected an XML body {} but was missing", expected.body())
      });
    }
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

fn name(name: QName) -> String {
  if let Some(namespace) = name.namespace_uri() {
    format!("{}:{}", namespace, name.local_part())
  } else {
    name.local_part().to_string()
  }
}

impl<'a> Matches<&'a Element<'a>> for &'a Element<'a> {
    fn matches_with(&self, actual: &Element, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
        let result = match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(actual.name().local_part()) {
                  Ok(())
                } else {
                  Err(anyhow!("Expected '{}' to match '{}'", name(actual.name()), regex))
                }
              },
              Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type => if self.name() == actual.name() {
             Ok(())
          } else {
             Err(anyhow!("Expected '{}' to be the same type as '{}'", name(self.name()),
                         name(actual.name())))
          },
          MatchingRule::MinType(min) => if !cascaded && actual.children().len() < min {
             Err(anyhow!("Expected '{}' to have at least {} children", name(actual.name()), min))
          } else {
             Ok(())
          },
          MatchingRule::MaxType(max) => if !cascaded && actual.children().len() > max {
             Err(anyhow!("Expected '{}' to have at most {} children", name(actual.name()), max))
          } else {
             Ok(())
          },
          MatchingRule::MinMaxType(min, max) => if !cascaded && actual.children().len() < min {
            Err(anyhow!("Expected '{}' to have at least {} children", name(actual.name()), min))
          } else if !cascaded && actual.children().len() > max {
            Err(anyhow!("Expected '{}' to have at most {} children", name(actual.name()), max))
          } else {
            Ok(())
          },
          MatchingRule::Equality => {
             if self.name() == actual.name() {
                 Ok(())
             } else {
                  Err(anyhow!("Expected '{}' to be equal to '{}'", name(self.name()), name(actual.name())))
             }
          },
          _ => Err(anyhow!("Unable to match {:?} using {:?}", self, matcher))
        };
        debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
        result
    }
}

fn path_to_string(path: &Vec<&str>) -> String {
  path.iter().enumerate().map(|(i, p)| {
    if i > 0 && !p.starts_with("[") {
      ".".to_owned() + p
    } else {
      p.to_string()
    }
  }).collect()
}

fn compare_element(path: &Vec<&str>, expected: &Element, actual: &Element,
  mismatches: &mut Vec<super::Mismatch>, context: &MatchingContext) {
  let matcher_result = if context.matcher_is_defined(&path) {
    trace!("calling match_values {:?} on {:?}", path, actual);
    match_values(&path, context, expected, actual)
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err| vec![err.to_string()])
  };
  debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual,
    path_to_string(&path), matcher_result);
  match matcher_result {
    Err(messages) => {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: path_to_string(&path),
          expected: Some(name(expected.name()).into()),
          actual: Some(name(actual.name()).into()),
          mismatch: message.clone()
        })
      }
    },
    Ok(_) => {
      compare_attributes(&path, expected, actual, mismatches, context);
      compare_children(&path, expected, actual, mismatches, context);
      compare_text(&path, expected, actual, mismatches, context);
    }
  }
}

fn compare_attributes(path: &Vec<&str>, expected: &Element, actual: &Element,
    mismatches: &mut Vec<super::Mismatch>, context: &MatchingContext) {
    let expected_attributes: BTreeMap<String, String> = expected.attributes()
        .iter().map(|attr| (name(attr.name()), s!(attr.value()))).collect();
    let actual_attributes: BTreeMap<String, String> = actual.attributes()
        .iter().map(|attr| (name(attr.name()), s!(attr.value()))).collect();
    if expected_attributes.is_empty() && !actual_attributes.is_empty() && context.config == DiffConfig::NoUnexpectedKeys {
      mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(format!("{:?}", expected_attributes).into()),
          actual: Some(format!("{:?}", actual_attributes).into()),
          mismatch: format!("Did not expect any attributes but received {:?}", actual_attributes)});
    } else {
        match context.config {
            DiffConfig::AllowUnexpectedKeys if expected_attributes.len() > actual_attributes.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(format!("{:?}", expected_attributes).into()),
                    actual: Some(format!("{:?}", actual_attributes).into()),
                    mismatch: format!("Expected at least {} attribute(s) but received {} attribute(s)",
                    expected_attributes.len(), actual_attributes.len())});
            },
            DiffConfig::NoUnexpectedKeys if expected_attributes.len() != actual_attributes.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(format!("{:?}", expected_attributes).into()),
                    actual: Some(format!("{:?}", actual_attributes).into()),
                    mismatch: format!("Expected {} attribute(s) but received {} attribute(s)",
                    expected_attributes.len(), actual_attributes.len())});
            },
            _ => ()
        }

        for (key, value) in expected_attributes.iter() {
          let mut p = path.to_vec();
          let new_p = "@".to_owned() + key;
          p.push(&*new_p);
          if actual_attributes.contains_key(key) {
            if let Err(m) = compare_value(&p, value, &actual_attributes[key], context) {
              mismatches.extend_from_slice(&*m);
            }
          } else {
            mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
              expected: Some(Bytes::from(key.clone())),
              actual: None,
              mismatch: format!("Expected attribute '{}'='{}' but was missing", key, value)});
          }
        }
    }
}

fn children<'a>(element: &Element<'a>) -> Vec<Element<'a>> {
  element.children().iter().cloned()
    .map(|child| child.element())
    .filter(|child| child.is_some())
    .map(|child| child.unwrap())
    .collect()
}

fn desc_children(children: &Vec<Element>) -> String {
  children.iter().map(|child| name(child.name())).join(", ")
}

fn compare_children(path: &Vec<&str>, expected: &Element, actual: &Element,
  mismatches: &mut Vec<super::Mismatch>, context: &MatchingContext) {
  let expected_children = children(expected);
  let actual_children = children(actual);

  if expected_children.is_empty() && !actual_children.is_empty() && context.config == DiffConfig::NoUnexpectedKeys {
    mismatches.push(Mismatch::BodyMismatch {
      path: path_to_string(path),
      expected: Some(desc_children(&expected_children).into()),
      actual: Some(desc_children(&actual_children).into()),
      mismatch: format!("Expected no children but received [{}]", desc_children(&actual_children))
    });
  } else {
    let mut expected_children_by_name: BTreeMap<String, Vec<Element>> = btreemap!{};
    for child in &expected_children {
      let key = name(child.name());
      if expected_children_by_name.contains_key(&key) {
        expected_children_by_name.get_mut(&key).unwrap().push(child.clone());
      } else {
        expected_children_by_name.insert(key, vec![ child.clone() ]);
      }
    }
    let mut actual_children_by_name: BTreeMap<String, Vec<Element>> = btreemap!{};
    for child in &actual_children {
      let key = name(child.name());
      if actual_children_by_name.contains_key(&key) {
        actual_children_by_name.get_mut(&key).unwrap().push(child.clone());
      } else {
        actual_children_by_name.insert(key, vec![ child.clone() ]);
      }
    }
    for (key, group) in actual_children_by_name {
      let mut p = path.to_vec();
      p.push(key.as_str());
      if expected_children_by_name.contains_key(&key) {
        let expected_children = expected_children_by_name.remove(&key).unwrap();
        let expected = expected_children.iter().next().unwrap();
        if context.type_matcher_defined(&p) {
          log::trace!("Matcher defined for path {}", path_to_string(&p));
          for child in group {
            compare_element(&p, &expected, &child, mismatches, context);
          }
        } else {
          for pair in expected_children.iter().zip_longest(group) {
            match pair {
              EitherOrBoth::Right(actual) => if context.config == DiffConfig::NoUnexpectedKeys {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
                  expected: Some(desc_children(&expected_children).into()),
                  actual: Some(desc_children(&actual_children).into()),
                  mismatch: format!("Unexpected child <{}/>", name(actual.name()))});
              },
              EitherOrBoth::Left(expected) => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
                  expected: Some(desc_children(&expected_children.clone()).into()),
                  actual: Some(desc_children(&actual_children.clone()).into()),
                  mismatch: format!("Expected child <{}/> but was missing", name(expected.name()))});
              },
              EitherOrBoth::Both(expected, actual) => {
                compare_element(&p, &expected, &actual, mismatches, context);
              }
            }
          }
        }
      } else if context.config == DiffConfig::NoUnexpectedKeys || context.type_matcher_defined(&p) {
        mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(desc_children(&expected_children.clone()).into()),
          actual: Some(desc_children(&actual_children.clone()).into()),
          mismatch: format!("Unexpected child <{}/>", key)});
      }
    }

    if !expected_children_by_name.is_empty() {
      for key in expected_children_by_name.keys() {
        mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(desc_children(&expected_children.clone()).into()),
          actual: Some(desc_children(&actual_children.clone()).into()),
          mismatch: format!("Expected child <{}/> but was missing", key)});
      }
    }
  }
}

fn compare_text(path: &Vec<&str>, expected: &Element, actual: &Element,
    mismatches: &mut Vec<super::Mismatch>, context: &MatchingContext) {
    let expected_text = expected.children().iter().cloned()
        .filter(|child| child.text().is_some())
        .map(|child| child.text().unwrap().text().trim())
        .collect::<String>();
    let actual_text = actual.children().iter().cloned()
        .filter(|child| child.text().is_some())
        .map(|child| child.text().unwrap().text().trim())
        .collect::<String>();
    let mut p = path.to_vec();
    p.push("#text");
    let matcher_result = if context.matcher_is_defined(&p) {
      match_values(&p, context, expected_text.trim(), actual_text.trim())
    } else {
      expected_text.matches_with(actual_text.trim(), &MatchingRule::Equality, false)
        .map_err(|err| vec![err.to_string()])
    };
    debug!("Comparing text '{}' to '{}' at path '{}' -> {:?}", expected_text, actual_text,
        path_to_string(path), matcher_result);
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::BodyMismatch {
              path: path_to_string(path) + ".#text",
              expected: Some(expected_text.clone().into()),
              actual: Some(actual_text.clone().into()),
              mismatch: message.clone()
            })
          }
        },
        Ok(_) => ()
    }
}

fn compare_value(path: &Vec<&str>, expected: &String, actual: &String, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let matcher_result = if context.matcher_is_defined(&path) {
    match_values(path, context, expected.as_str(), actual.as_str())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err| vec![err.to_string()])
  };
  debug!("Comparing '{}' to '{}' at path '{}' -> {:?}", expected, actual, path_to_string(path), matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::BodyMismatch {
        path: path_to_string(path),
        expected: Some(expected.clone().into()),
        actual: Some(actual.clone().into()),
        mismatch: message.clone()
      }
    }).collect()
  })
}

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use expectest::expect;
  use expectest::prelude::*;
  use ntest::test_case;
  use test_env_log::test;

  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::request::Request;

  use crate::DiffConfig;
  use crate::Mismatch;

  use super::*;

  macro_rules! request {
    ($e:expr) => (Request {
        body: OptionalBody::Present(Bytes::from($e), None), .. Request::default()
      })
  }

  #[test]
  fn match_xml_comparing_missing_bodies() {
    let expected = Request { body: OptionalBody::Missing, .. Request::default() };
    let actual = Request { body: OptionalBody::Missing, .. Request::default() };
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_comparing_empty_bodies() {
    let expected = Request { body: OptionalBody::Empty, .. Request::default() };
    let actual = Request { body: OptionalBody::Empty, .. Request::default() };
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_comparing_a_missing_body_to_anything() {
    let expected = Request { body: OptionalBody::Empty, .. Request::default() };
    let actual = request!("<blah/>");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_comparing_anything_to_an_empty_body_returns_a_mismatch() {
    let expected = request!("<blah/>");
    let actual = Request { body: OptionalBody::Empty, .. Request::default() };
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    let mismatch = mismatches.first().unwrap();
    expect!(mismatch.description()).to(be_equal_to("$ -> Expected an XML body Present(7 bytes) but was missing"));
  }

  #[test]
  fn match_xml_handles_empty_strings() {
    let expected = request!("");
    let actual = request!("");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch {
      path: s!("$"),
      expected: Some(Bytes::new()),
      actual: Some(Bytes::new()),
      mismatch: s!("")
    }));
  }

  #[test]
  fn match_xml_handles_invalid_expected_xml() {
    let expected = request!(r#"<xml-is-bad"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_xml_handles_invalid_actual_xml() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
    let actual = request!(r#"{json: "is bad"}"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("")
    } ]));
  }

  fn mismatch_message(result: &Result<(), Vec<Mismatch>>) -> String {
    match result {
      Err(mismatches) => match mismatches.first() {
        Some(Mismatch::BodyMismatch{ mismatch, .. }) => mismatch.clone(),
        _ => String::default()
      },
      _ => String::default()
    }
  }

  #[test]
  fn match_xml_with_equal_bodies() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_allow_unexpected_keys_is_true_and_comparing_an_empty_list_to_a_non_empty_one() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo></foo>"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item/></foo>"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_allow_unexpected_keys_is_true_and_comparing_a_list_to_a_super_set() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item1/></foo>"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item1/><item2/></foo>"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_bodies_differ_only_in_whitespace() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>
        <bar></bar>
    </foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><bar></bar></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_actual_has_different_root() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <bar></bar>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 'foo' to be equal to 'bar'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("foo".into()),
        actual: Some("bar".into()), mismatch: s!("") } ]));
  }

  #[test]
  fn match_xml_with_equal_attributes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b" c="d"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b" c="d"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_nonequal_attributes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="c" c="b"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(3));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah"),
        expected: Some("{\"a\": \"c\", \"c\": \"b\"}".into()),
        actual: Some("{\"a\": \"b\"}".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.blah -> Expected at least 2 attribute(s) but received 1 attribute(s)")));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah.@a"), expected: Some("c".into()),
        actual: Some("b".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.blah.@a -> Expected 'c' to be equal to 'b'")));
    let mismatch = mismatches[2].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah.@c"), expected: Some("c".into()),
        actual: None, mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.blah.@c -> Expected attribute \'c\'=\'b\' but was missing")));
  }

  #[test]
  fn match_xml_with_when_not_expecting_attributes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b" c="d"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Did not expect any attributes but received {\"a\": \"b\", \"c\": \"d\"}")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.blah"), expected: Some("{}".into()),
      actual: Some("{\"a\": \"b\", \"c\": \"d\"}".into()), mismatch: s!("") } ]));
  }

  #[test]
  fn match_xml_with_comparing_a_tags_attributes_to_one_with_more_entries() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <blah a="b" c="d"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_comparing_a_tags_attributes_to_one_with_less_entries() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100" somethingElse="101"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 1 attribute(s) but received 2 attribute(s)")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("{\"something\": \"100\"}".into()),
        actual: Some("{\"something\": \"100\", \"somethingElse\": \"101\"}".into()), mismatch: s!("") } ]));
  }

  #[test]
  fn match_xml_when_a_tag_has_the_same_number_of_attributes_but_different_keys() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100" somethingElse="100"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100" somethingDifferent="100"/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected attribute \'somethingElse\'=\'100\' but was missing")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.foo.@somethingElse"), expected: Some("somethingElse".into()),
        actual: None, mismatch: s!("") } ]));
  }

  #[test]
  fn match_xml_when_a_tag_has_the_same_number_of_attributes_but_different_values() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100" somethingElse="100"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="100" somethingElse="101"/>
    "#);
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected \'100\' to be equal to \'101\'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.foo.@somethingElse"), expected: Some("100".into()),
        actual: Some("101".into()), mismatch: s!("") } ]));

    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::NoUnexpectedKeys, &matchingrules!{
      "body" => {
        "$.foo.*" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_actual_is_non_empty_and_we_do_not_allow_extra_keys() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><bar></bar></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected no children but received [bar]")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$.foo"),
      expected: Some(Bytes::new()),
      actual: Some("bar".into()),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_xml_when_actual_is_non_empty_and_we_allow_extra_keys() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><bar></bar></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_actual_is_a_super_set() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><item1/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><item1/><item2/></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_actual_is_empty() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><bar></bar></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo/>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected child <bar/> but was missing")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$.foo"),
      expected: Some("bar".into()),
      actual: Some(Bytes::new()),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_xml_when_actual_is_different_size() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/><four/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected child <four/> but was missing")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.foo"),
        expected: Some("one, two, three, four".into()),
        actual: Some("one, two, three".into()), mismatch: s!("") } ]));
  }

  #[test]
  fn match_xml_comparing_a_list_to_one_with_with_the_same_size_but_different_children() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/><three/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/><four/></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"),
      expected: Some("one, two, three, three".into()),
      actual: Some("one, two, three, four".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.foo -> Unexpected child <four/>")));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.three"),
      expected: Some("three, three".into()),
      actual: Some("one, two, three, four".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.foo.three -> Expected child <three/> but was missing")));
  }

  #[test]
  fn match_xml_comparing_a_list_to_one_where_the_items_are_in_the_wrong_order() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><three/><two/></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_the_same_text() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello world</foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello world</foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_the_same_text_between_nodes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello<bar/>world</foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello<bar/>world</foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_the_different_text() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello world</foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello mars</foo>
    "#);
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 'hello world' to be equal to 'hello mars'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$.foo.#text".to_string(),
        expected: Some("hello world".into()),
        actual: Some("hello mars".into()), mismatch: "".to_string() } ]));

    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
        "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex(r"[a-z\s]+".into()) ]
        }
    }.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_the_different_text_between_nodes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello<bar/>world</foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo>hello<bar/>mars </foo>
    "#);
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 'helloworld' to be equal to 'hellomars'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$.foo.#text".to_string(),
        expected: Some("helloworld".into()),
        actual: Some("hellomars".into()), mismatch: "".to_string() } ]));

    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$.foo['#text']" => [ MatchingRule::Regex("[a-z]+".to_string()) ]
      }
    }.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_a_matcher() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><one/><one/></foo>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::NoUnexpectedKeys, &matchingrules!{
      "body" => {
        "$.foo" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_unexpected_elements() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <note> <to>John</to> <from>Jane</from> <subject>Reminder</subject>
        <address> <city>Manchester</city> </address> </note>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <note> <to>John</to> <from>Jane</from> <subject>Reminder</subject>
         <address> <firstName>John</firstName> <lastName>Doe</lastName> <street>Prince Street</street>
         <number>34</number> <city>Manchester</city> </address> </note>
        "#);
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_cdata_elements() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData>ABCD***************010101</hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    let result = match_xml(&expected.clone(), &expected.clone(), &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_with_cdata_elements_matching_with_regex() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData>OWY0NzEyYTAyMmMzZjI2Y2RmYzZiMTcx</hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData><![CDATA[Mjc5MGJkNDVjZTI3OWNjYjJjMmYzZTVh]]></hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    let rules = matchingrules! {
      "body" => {
        "$.providerService.attribute1.newattribute2.hiddenData" => [ MatchingRule::Regex("[a-zA-Z0-9]*".into()) ]
      }
    }.rules_for_category("body").unwrap();
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::new(DiffConfig::AllowUnexpectedKeys, &rules));
    expect!(result).to(be_ok());
  }

  #[test_case(r#"<blah xmlns="urn:ns"/>"#, r#"<blah xmlns="urn:ns"/>"#)]
  #[test_case(r#"<b:blah xmlns:b="urn:ns"/>"#, r#"<blah xmlns="urn:ns"/>"#)]
  #[test_case(r#"<blah xmlns="urn:ns"/>"#, r#"<a:blah xmlns:a="urn:ns"/>"#)]
  #[test_case(r#"<b:blah xmlns:b="urn:ns"/>"#, r#"<a:blah xmlns:a="urn:ns"/>"#)]
  fn match_xml_with_different_namespace_declarations(expected: &str, actual: &str) {
    let expected = request!(expected);
    let actual = request!(actual);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test_case(r#"<foo xmlns="urn:ns"><item/></foo>"#, r#"<foo xmlns="urn:ns"><item/></foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<foo xmlns="urn:ns"><item/></foo>"#)]
  #[test_case(r#"<foo xmlns="urn:ns"><item/></foo>"#, r#"<a:foo xmlns:a="urn:ns"><a:item/></a:foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<a:foo xmlns:a="urn:ns"><a:item/></a:foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<a:foo xmlns:a="urn:ns"><a2:item xmlns:a2="urn:ns"/></a:foo>"#)]
  fn match_xml_with_different_namespace_declarations_on_child_elements(expected: &str, actual: &str) {
    let expected = request!(expected);
    let actual = request!(actual);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_different_namespaces_are_used() {
    let expected = request!("<blah xmlns=\"urn:other\"/>");
    let actual = request!(r#"<blah xmlns="urn:ns"/>"#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$.urn:other:blah".to_string(),
      expected: Some("urn:other:blah".into()),
      actual: Some("urn:ns:blah".into()),
      mismatch: "Expected 'urn:other:blah' to be equal to 'urn:ns:blah'".to_string()
    } ]));
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_expected_namespace_is_not_used() {
    let expected = request!("<blah xmlns=\"urn:other\"/>");
    let actual = request!("<blah/>");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$.urn:other:blah".to_string(),
      expected: Some("urn:other:blah".into()),
      actual: Some("blah".into()),
      mismatch: "Expected 'urn:other:blah' to be equal to 'blah'".to_string()
    } ]));
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_allow_unexpected_keys_is_true_and_no_namespace_is_expected() {
    let expected = request!("<blah/>");
    let actual = request!("<blah xmlns=\"urn:ns\"/>");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$.blah".to_string(),
      expected: Some("blah".into()),
      actual: Some("urn:ns:blah".into()),
      mismatch: "Expected 'blah' to be equal to 'urn:ns:blah'".to_string()
    } ]));
  }

  #[test]
  fn matching_xml_bodies_when_attribute_uses_different_prefix() {
    let expected = request!("<foo xmlns:a=\"urn:ns\" a:something=\"100\"/>");
    let actual = request!("<foo xmlns:b=\"urn:ns\" b:something=\"100\"/>");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_attribute_uses_different_namespace() {
    let expected = request!("<foo xmlns:ns=\"urn:b\" ns:something=\"100\"/>");
    let actual = request!("<foo xmlns:ns=\"urn:a\" ns:something=\"100\"/>");
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: "$.foo.@urn:b:something".to_string(),
      expected: Some("urn:b:something".into()),
      actual: None,
      mismatch: "Expected attribute 'urn:b:something'='100' but was missing".to_string()
    } ]));
  }

  #[test]
  fn matching_xml_bodies_with_namespaces_and_a_matcher_defined_delegate_to_matcher_for_attribute() {
    let expected = request!("<foo xmlns:b=\"urn:ns\" b:something=\"101\"/>");
    let actual = request!("<foo xmlns:a=\"urn:ns\" a:something=\"100\"/>");
    let matching_rules = matchingrules! {
      "body" => { "$.foo['@urn:ns:something']" => [ MatchingRule::Regex(s!("^[0-9]+$")) ] }
    };
    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                                                     &matching_rules.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn matching_xml_bodies_with_namespaces_and_a_matcher_defined_delegate_to_the_matcher() {
    let expected = request!("<ns:foo xmlns:ns=\"urn:ns\"><ns:something>101</ns:something></ns:foo>");
    let actual = request!("<ns:foo xmlns:ns=\"urn:ns\"><ns:something>100</ns:something></ns:foo>");
    let matching_rules = matchingrules! {
      "body" => { "$['urn:ns:foo']['urn:ns:something'].#text" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ] }
    };
    let result = match_xml(&expected, &actual, &MatchingContext::new(DiffConfig::NoUnexpectedKeys,
      &matching_rules.rules_for_category("body").unwrap()));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_an_element_has_different_types_of_children_but_we_allow_unexpected_keys() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_xml_when_an_element_has_different_types_of_children_but_we_do_not_allow_unexpected_keys() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
      <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
      </animals>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
      <animals>
          <dog id="1" name="Canine"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
      </animals>
    "#);
    let result = match_xml(&expected, &actual, &MatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(3));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.cat"),
      expected: Some("cat".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.animals.cat -> Unexpected child <cat/>")));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.cat"),
      expected: Some("cat".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: s!("")}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.animals.cat -> Unexpected child <cat/>")));
    let mismatch = mismatches[2].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.dog"),
      expected: Some("dog".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: "Unexpected child <dog/>".into()}));
    expect!(mismatch.description()).to(be_equal_to(s!("$.animals.dog -> Unexpected child <dog/>")));
  }

  #[test]
  fn match_xml_type_matcher_when_an_element_has_different_types_of_children() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let matching_rules = matchingrules! {
      "body" => {
        "$.animals.*" => [ MatchingRule::Type ],
        "$.animals.*['@id']" => [ MatchingRule::Integer ]
      }
    }.rules_for_category("body").unwrap();
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::new(DiffConfig::NoUnexpectedKeys, &matching_rules));
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_boolean_attributes() {
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="true" somethingElse="true"/>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo something="false" somethingElse="101"/>
    "#);
    let matching_rules = matchingrules! {
      "body" => {
        "$.foo['@something']" => [ MatchingRule::Boolean ],
        "$.foo['@somethingElse']" => [ MatchingRule::Boolean ]
      }
    }.rules_for_category("body").unwrap();
    let result = match_xml(&expected.clone(), &actual.clone(), &MatchingContext::new(DiffConfig::NoUnexpectedKeys, &matching_rules));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected \'101\' to match a boolean".to_string()));
    expect!(result).to(be_err().value(vec![
      Mismatch::BodyMismatch {
        path: "$.foo.@somethingElse".into(),
        expected: Some("true".into()),
        actual: Some("101".into()),
        mismatch: Default::default()
      }
    ]));
  }
}
