//! Module for handling content types

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::str::{from_utf8, FromStr};

use anyhow::anyhow;
use itertools::Itertools;
use lazy_static::*;
use mime::Mime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Content type of a body
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct ContentType {
  /// Main content type
  pub main_type: String,
  /// Sub content type
  pub sub_type: String,
  /// Content type attributes
  pub attributes: BTreeMap<String, String>,
  /// Suffix
  pub suffix: Option<String>
}

lazy_static! {
  /// XML Content Type
  pub static ref XML: ContentType = ContentType {
    main_type: "application".into(),
    sub_type: "xml".into(),
    .. ContentType::default()
  };

  /// HTML Content Type
  pub static ref HTML: ContentType = ContentType {
    main_type: "text".into(),
    sub_type: "html".into(),
    .. ContentType::default()
  };

  /// JSON Content Type
  pub static ref JSON: ContentType = ContentType {
    main_type: "application".into(),
    sub_type: "json".into(),
    .. ContentType::default()
  };

  /// TEXT Content Type
  pub static ref TEXT: ContentType = ContentType {
    main_type: "text".into(),
    sub_type: "plain".into(),
    .. ContentType::default()
  };

  /// FORM POST URL encoded format
  pub static ref FORM_URLENCODED: ContentType = ContentType {
    main_type: "application".into(),
    sub_type: "x-www-form-urlencoded".into(),
    .. ContentType::default()
  };

  static ref XMLREGEXP: Regex = Regex::new(r"^\s*<\?xml\s*version.*").unwrap();
  static ref HTMLREGEXP: Regex = Regex::new(r"^\s*(<!DOCTYPE)|(<HTML>).*").unwrap();
  static ref JSONREGEXP: Regex = Regex::new(r#"^\s*(true|false|null|[0-9]+|"\w*|\{\s*(}|"\w+)|\[\s*)"#).unwrap();
  static ref XMLREGEXP2: Regex = Regex::new(r#"^\s*<\w+\s*(:\w+=["”][^"”]+["”])?.*"#).unwrap();
}

const KNOWN_TEXT_TYPES: [(&'static str, &'static str); 1] = [
  ("application", "x-www-form-urlencoded")
];

impl ContentType {
  /// Parses a string into a ContentType
  pub fn parse<'a, S: Into<&'a str>>(content_type: S) -> anyhow::Result<ContentType> {
    let content_type = content_type.into();
    match Mime::from_str(content_type) {
      Ok(mime) => {
        Ok(ContentType::from(mime))
      },
      Err(err) => {
        let message = format!("Failed to parse '{}' as a content type: {}",
                              content_type, err);
        warn!("{}", message);
        Err(anyhow!(message))
      }
    }
  }

  /// If it is a JSON type
  pub fn is_json(&self) -> bool {
    self.main_type == "application" && (self.sub_type.starts_with("json") ||
      self.suffix.as_ref().unwrap_or(&String::default()) == "json" ||
      self.sub_type == "graphql")
  }

  /// If it is a XML type
  pub fn is_xml(&self) -> bool {
    (self.main_type == "application" || self.main_type == "text") && (self.sub_type == "xml" ||
      self.suffix.as_ref().unwrap_or(&String::default()) == "xml")
  }

  /// If it is a XML type (not including ones with suffixes like atom+xml)
  pub fn is_strict_xml(&self) -> bool {
    (self.main_type == "application" || self.main_type == "text") && self.sub_type == "xml"
  }

  /// If it is a text type
  pub fn is_text(&self) -> bool {
    self.main_type == "text" || self.is_xml() || self.is_json() || self.is_known_text_type()
  }

  /// If it is a known binary type
  pub fn is_binary(&self) -> bool {
    match self.main_type.as_str() {
      "audio" | "font" | "image" | "video" => true,
      "text" => false,
      _ => !self.is_text()
    }
  }

  /// Returns the base type with no attributes or suffix
  pub fn base_type(&self) -> ContentType {
    match self.suffix.as_ref() {
      Some(suffix) => ContentType {
        main_type: self.main_type.clone(),
        sub_type: suffix.clone(),
        .. ContentType::default()
      },
      None => ContentType {
        main_type: self.main_type.clone(),
        sub_type: self.sub_type.clone(),
        .. ContentType::default()
      }
    }
  }

  /// If the content type is the default type
  pub fn is_unknown(&self) -> bool {
    self.main_type == "*" || self.sub_type == "*"
  }

  /// Equals, ignoring attributes if not present on self
  pub fn is_equivalent_to(&self, other: &ContentType) -> bool {
    if self.is_strict_xml() && other.is_strict_xml() {
      self.attributes.is_empty() || self.attributes == other.attributes
    }
    else if self.attributes.is_empty() {
      self.main_type == other.main_type && self.sub_type == other.sub_type
    } else {
      self == other
    }
  }

  /// If this content type is in the list of known text types
  fn is_known_text_type(&self) -> bool {
    KNOWN_TEXT_TYPES.iter().find(|(t, st)| {
      self.main_type == *t && self.sub_type == *st
    }).is_some()
  }

  pub fn is_form_urlencoded(&self) -> bool {
    self.main_type == "application" && self.sub_type == "x-www-form-urlencoded"
  }
}

impl Default for ContentType {
  fn default() -> Self {
    ContentType {
      main_type: "*".into(),
      sub_type: "*".into(),
      attributes: BTreeMap::new(),
      suffix: None
    }
  }
}

impl Display for ContentType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let base = if let Some(suffix) = &self.suffix {
      format!("{}/{}+{}", self.main_type, self.sub_type, suffix)
    } else {
      format!("{}/{}", self.main_type, self.sub_type)
    };
    if self.attributes.is_empty() {
      write!(f, "{}", base)
    } else {
      write!(f, "{};{}", base, self.attributes.iter()
        .map(|(key, value)| format!("{}={}", key, value)).join(";"))
    }
  }
}

impl From<String> for ContentType {
  fn from(s: String) -> Self {
    ContentType::parse(s.as_str()).unwrap_or_default()
  }
}

impl From<&String> for ContentType {
  fn from(s: &String) -> Self {
    ContentType::parse(s.as_str()).unwrap_or_default()
  }
}

impl From<&str> for ContentType {
  fn from(s: &str) -> Self {
    ContentType::parse(s).unwrap_or_default()
  }
}

impl From<Mime> for ContentType {
  fn from(mime: Mime) -> Self {
    Self::from(&mime)
  }
}

impl From<&Mime> for ContentType {
  fn from(mime: &Mime) -> Self {
    ContentType {
      main_type: mime.type_().to_string(),
      sub_type: mime.subtype().to_string(),
      attributes: mime.params().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
      suffix: mime.suffix().map(|name| name.to_string()),
      .. ContentType::default()
    }
  }
}

impl FromStr for ContentType {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    ContentType::parse(s)
  }
}

impl Into<String> for &ContentType {
  fn into(self) -> String {
    self.to_string()
  }
}

impl PartialEq<str> for ContentType {
  fn eq(&self, other: &str) -> bool {
    match ContentType::parse(other) {
      Ok(other) => *self == other,
      Err(_) => false
    }
  }
}

impl PartialEq<&str> for ContentType {
  fn eq(&self, other: &&str) -> bool {
    match ContentType::parse(*other) {
      Ok(other) => *self == other,
      Err(_) => false
    }
  }
}

fn is_match(regex: &Regex, string: &str) -> bool {
  if let Some(m) = regex.find(string) {
    m.start() == 0
  } else {
    false
  }
}

/// Try detect the content type from the contents of a string
pub fn detect_content_type_from_string(s: &str) -> Option<ContentType> {
  debug!("Detecting content type from contents: '{}'", s);
  if is_match(&XMLREGEXP, s) {
    Some(XML.clone())
  } else if is_match(&HTMLREGEXP, s.to_uppercase().as_str()) {
    Some(HTML.clone())
  } else if is_match(&XMLREGEXP2, s) {
    Some(XML.clone())
  } else if is_match(&JSONREGEXP, s) {
    Some(JSON.clone())
  } else {
    Some(TEXT.clone())
  }
}

/// Try detect the content type from a sequence bytes
pub fn detect_content_type_from_bytes(s: &[u8]) -> Option<ContentType> {
  debug!("Detecting content type from byte contents");
  let header = if s.len() > 32 {
    &s[0..32]
  } else {
    s
  };
  match from_utf8(header) {
    Ok(s) => {
      if is_match(&XMLREGEXP, s) {
        Some(XML.clone())
      } else if is_match(&HTMLREGEXP, &*s.to_uppercase()) {
        Some(HTML.clone())
      } else if is_match(&XMLREGEXP2, s) {
        Some(XML.clone())
      } else if is_match(&JSONREGEXP, s) {
        Some(JSON.clone())
      } else {
        Some(TEXT.clone())
      }
    },
    Err(_) => None
  }
}

/// Override of the content type
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum ContentTypeHint {
  BINARY,
  TEXT,
  DEFAULT
}

impl Default for ContentTypeHint {
  fn default() -> Self {
    ContentTypeHint::DEFAULT
  }
}

impl Display for ContentTypeHint {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ContentTypeHint::BINARY => write!(f, "BINARY"),
      ContentTypeHint::TEXT => write!(f, "TEXT"),
      ContentTypeHint::DEFAULT => write!(f, "DEFAULT")
    }
  }
}

impl TryFrom<&str> for ContentTypeHint {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "BINARY" => Ok(ContentTypeHint::BINARY),
      "TEXT" => Ok(ContentTypeHint::TEXT),
      "DEFAULT" => Ok(ContentTypeHint::DEFAULT),
      _ => Err(anyhow!("'{}' is not a valid value for ContentTypeHint", value))
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::btreemap;
  use rstest::rstest;

  use super::ContentType;

  #[test]
  fn parse_test() {
    let content_type = &ContentType::parse("application/json").unwrap();
    expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
    expect!(&content_type.sub_type).to(be_equal_to(&"json".to_string()));
    expect!(content_type.attributes.iter()).to(be_empty());
    expect!(content_type.clone().suffix).to(be_none());

    let content_type = &ContentType::parse("application/json;charset=UTF-16").unwrap();
    expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
    expect!(&content_type.sub_type).to(be_equal_to(&"json".to_string()));
    expect!(content_type.clone().attributes).to(be_equal_to(btreemap! {
    "charset".to_string() => "utf-16".to_string()
  }));
    expect!(content_type.clone().suffix).to(be_none());

    let content_type = &ContentType::parse("application/hal+json; charset=UTF-8").unwrap();
    expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
    expect!(&content_type.sub_type).to(be_equal_to(&"hal".to_string()));
    expect!(content_type.clone().attributes).to(be_equal_to(btreemap! {
    "charset".to_string() => "utf-8".to_string()
  }));
    expect!(content_type.clone().suffix).to(be_some().value("json".to_string()));
  }

  #[test]
  fn to_string_test() {
    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "hal+json".into(),
      ..ContentType::default()
    };
    expect!(content_type.to_string()).to(be_equal_to("application/hal+json".to_string()));

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "hal".into(),
      suffix: Some("json".into()),
      ..ContentType::default()
    };
    expect!(content_type.to_string()).to(be_equal_to("application/hal+json".to_string()));

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "hal+json".into(),
      attributes: btreemap! {
      "charset".to_string() => "UTF-32".to_string(),
      "b".to_string() => "c".to_string()
    },
      suffix: None
    };
    expect!(content_type.to_string()).to(be_equal_to("application/hal+json;b=c;charset=UTF-32".to_string()));
  }

  #[test]
  fn is_json_test() {
    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "hal".into(),
      suffix: Some("json".to_string()),
      .. ContentType::default()
    };
    expect!(content_type.is_json()).to(be_true());

    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_json()).to(be_false());

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "json".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_json()).to(be_true());

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "json-rpc".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_json()).to(be_true());

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "graphql".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_json()).to(be_true());
  }

  #[test]
  fn is_xml_test() {
    let content_type = ContentType::parse("application/atom+xml").unwrap();
    expect!(content_type.is_xml()).to(be_true());

    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_xml()).to(be_false());

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "xml".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_xml()).to(be_true());

    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "xml".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_xml()).to(be_true());
  }

  #[test]
  fn base_type_test() {
    let content_type = ContentType::parse("application/atom+xml").unwrap();
    expect!(content_type.base_type()).to(be_equal_to(ContentType {
      main_type: "application".into(),
      sub_type: "xml".into(),
      .. ContentType::default()
    }));

    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    };
    expect!(content_type.base_type()).to(be_equal_to(ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    }));

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "xml".into(),
      attributes: btreemap! { "charset".to_string() => "UTF-8".to_string() },
      .. ContentType::default()
    };
    expect!(content_type.base_type()).to(be_equal_to(ContentType {
      main_type: "application".into(),
      sub_type: "xml".into(),
      .. ContentType::default()
    }));

    let content_type = ContentType::parse("application/x-www-form-urlencoded").unwrap();
    expect!(content_type.base_type()).to(be_equal_to(ContentType {
      main_type: "application".into(),
      sub_type: "x-www-form-urlencoded".into(),
      .. ContentType::default()
    }));
  }

  #[test]
  fn is_binary_test() {
    let content_type = ContentType::parse("application/atom+xml").unwrap();
    expect!(content_type.is_binary()).to(be_false());

    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_binary()).to(be_false());

    let content_type = ContentType {
      main_type: "image".into(),
      sub_type: "jpeg".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_binary()).to(be_true());
  }

  #[test]
  fn is_text_test() {
    let content_type = ContentType {
      main_type: "text".into(),
      sub_type: "javascript".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_text()).to(be_true());

    let content_type = ContentType::parse("application/atom+xml").unwrap();
    expect!(content_type.is_text()).to(be_true());

    let content_type = ContentType::parse("application/hal+json").unwrap();
    expect!(content_type.is_text()).to(be_true());

    let content_type = ContentType {
      main_type: "image".into(),
      sub_type: "jpeg".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_text()).to(be_false());

    let content_type = ContentType {
      main_type: "application".into(),
      sub_type: "x-www-form-urlencoded".into(),
      .. ContentType::default()
    };
    expect!(content_type.is_text()).to(be_true());
  }

  #[test]
  fn xml_equivalent_test() {
    let content_type = ContentType::parse("application/atom+xml").unwrap();
    let content_type2 = ContentType::parse("application/xml").unwrap();
    let content_type3 = ContentType::parse("text/xml").unwrap();
    let content_type4 = ContentType::parse("application/json").unwrap();

    expect!(content_type.is_equivalent_to(&content_type)).to(be_true());
    expect!(content_type.is_equivalent_to(&content_type2)).to(be_false());
    expect!(content_type2.is_equivalent_to(&content_type3)).to(be_true());
    expect!(content_type2.is_equivalent_to(&content_type4)).to(be_false());
  }

  #[rstest]
  #[case("text/plain", false)]
  #[case("multipart/form-data", false)]
  #[case("application/x-www-form-urlencoded", true)]
  #[case("application/json", false)]
  fn is_form_urlencoded_test(#[case] content_type: &str, #[case] result: bool) {
    expect!(ContentType::parse(content_type).unwrap().is_form_urlencoded()).to(be_eq(result));
  }
}
