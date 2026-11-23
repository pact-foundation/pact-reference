//! The suite's interaction tables as plain data, and how they are registered through the FFI.
//! Mirrors `setup_common_interactions` / `setup_body` in the pact-reference compatibility suite.

use std::collections::HashMap;
use std::env::{self, consts::{DLL_PREFIX, DLL_SUFFIX}};
use std::fs;

use crate::ffi::{self, InteractionHandle, InteractionPart, InteractionPart_Request, InteractionPart_Response, PactHandle};

/// The copy of the suite that `compatibility-suite` also runs.
pub const FEATURES: &str = "../compatibility-suite/pact-compatibility-suite/features";
pub const FIXTURES: &str = "../compatibility-suite/pact-compatibility-suite/fixtures";

/// The libpact_ffi to load: `PACT_FFI_LIB`, or the one `.github/scripts/ffi-examples.sh` installs.
pub fn ffi_lib() -> String {
  env::var("PACT_FFI_LIB")
    .unwrap_or_else(|_| format!("../rust/pact_ffi/build/install/lib/{DLL_PREFIX}pact_ffi{DLL_SUFFIX}"))
}

#[derive(Debug, Clone, Default)]
pub struct Part {
  pub headers: Vec<(String, String)>,
  pub body: Option<Vec<u8>>,
  /// Matching rules as JSON text, passed to the FFI unchanged.
  pub matching_rules: Option<String>,
}

impl Part {
  pub fn header(&self, name: &str) -> Option<&str> {
    self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
  }

  pub fn set_header(&mut self, name: &str, value: &str) {
    self.headers.retain(|(k, _)| !k.eq_ignore_ascii_case(name));
    self.headers.push((name.to_string(), value.to_string()));
  }
}

#[derive(Debug, Clone, Default)]
pub struct Interaction {
  pub description: String,
  pub method: String,
  pub path: String,
  pub query: String,
  pub request: Part,
  pub status: u16,
  pub response: Part,
}

/// Builds interactions from a Gherkin table whose first row names the columns.
pub fn setup_common_interactions(rows: &[Vec<String>]) -> Vec<Interaction> {
  let cols: HashMap<&str, usize> = rows[0].iter().enumerate().map(|(i, h)| (h.as_str(), i)).collect();
  rows.iter().skip(1).enumerate().map(|(row, values)| {
    let get = |name: &str| cols.get(name).and_then(|i| values.get(*i)).cloned().unwrap_or_default();
    let mut interaction = Interaction {
      description: format!("Interaction {row}"),
      method: get("method"),
      path: get("path"),
      query: get("query"),
      status: get("response").parse().unwrap_or(200),
      ..Interaction::default()
    };
    interaction.request.headers = parse_headers(&get("headers"));
    setup_body(&get("body"), &mut interaction.request);
    interaction.response.headers = parse_headers(&get("response headers"));
    setup_body(&get("response body"), &mut interaction.response);
    interaction.request.matching_rules = matching_rules(&get("matching rules"));
    interaction.response.matching_rules = matching_rules(&get("response matching rules"));
    interaction
  }).collect()
}

/// A `matching rules` cell: a fixture file name or inline `JSON: {...}`.
pub fn matching_rules(cell: &str) -> Option<String> {
  if cell.is_empty() {
    None
  } else if let Some(json) = cell.strip_prefix("JSON:") {
    Some(json.trim().to_string())
  } else {
    Some(fs::read_to_string(format!("{FIXTURES}/{}", cell.trim())).unwrap_or_else(|e| panic!("fixture {cell}: {e}")))
  }
}

/// Parses `'Name: value', 'Other: value'` cells.
pub fn parse_headers(cell: &str) -> Vec<(String, String)> {
  if cell.is_empty() {
    return vec![];
  }
  cell.split(',').filter_map(|h| {
    let h = h.trim();
    let h = h.strip_prefix('\'').unwrap_or(h);
    let h = h.strip_suffix('\'').unwrap_or(h);
    h.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
  }).collect()
}

/// Query string to ordered (name, value) pairs, percent-decoded.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
  url::form_urlencoded::parse(query.as_bytes()).into_owned().collect()
}

/// Query pairs grouped by name, for order-insensitive comparison.
pub fn query_map(pairs: &[(String, String)]) -> HashMap<String, Vec<String>> {
  let mut map: HashMap<String, Vec<String>> = HashMap::new();
  for (k, v) in pairs {
    map.entry(k.clone()).or_default().push(v.clone());
  }
  map
}

pub fn setup_body(cell: &str, part: &mut Part) {
  setup_body_as(cell, part, None)
}

/// As `setup_body`, with the content type a table's "content type" column gives, where it has one.
pub fn setup_body_as(cell: &str, part: &mut Part, content_type_override: Option<&str>) {
  if cell.is_empty() {
    return;
  }
  let (content_type, body) = if let Some(json) = cell.strip_prefix("JSON:") {
    ("application/json".to_string(), json.trim().as_bytes().to_vec())
  } else if let Some(xml) = cell.strip_prefix("XML:") {
    ("application/xml".to_string(), xml.trim().as_bytes().to_vec())
  } else if let Some(file) = cell.strip_prefix("file:") {
    let file = file.trim();
    if file.ends_with("-body.xml") {
      let (ct, contents) = body_fixture(file);
      (ct, contents.into_bytes())
    } else {
      let bytes = fs::read(format!("{FIXTURES}/{file}")).unwrap_or_else(|e| panic!("fixture {file}: {e}"));
      (determine_content_type(file, part), bytes)
    }
  } else {
    let body = if cell == "EMPTY" { "" } else { cell };
    (determine_content_type(body, part), body.as_bytes().to_vec())
  };
  let content_type = content_type_override.map(str::to_string).unwrap_or(content_type);
  part.set_header("content-type", &content_type);
  part.body = Some(body);
}

fn determine_content_type(body: &str, part: &Part) -> String {
  let ct = if body.ends_with(".json") {
    "application/json"
  } else if body.ends_with(".xml") {
    "application/xml"
  } else if body.ends_with(".jpg") {
    "image/jpeg"
  } else if body.ends_with(".pdf") {
    "application/pdf"
  } else {
    part.header("content-type").unwrap_or("text/plain")
  };
  ct.to_string()
}

/// Reads a `*-body.xml` fixture: `<body><contentType/><contents eol="CRLF"?/></body>`.
fn body_fixture(file: &str) -> (String, String) {
  let xml = fs::read_to_string(format!("{FIXTURES}/{file}")).unwrap_or_else(|e| panic!("fixture {file}: {e}"));
  let doc = roxmltree::Document::parse(&xml).unwrap_or_else(|e| panic!("fixture {file}: {e}"));
  let text = |name: &str| {
    doc.root_element().children().find(|n| n.has_tag_name(name)).map(|n| {
      let s: String = n.children().filter_map(|c| c.text()).map(str::trim).collect();
      // The XML parser normalises line endings to LF, even on a Windows checkout with CRLF files
      if n.attribute("eol") == Some("CRLF") { s.replace("\r\n", "\n").replace('\n', "\r\n") } else { s }
    })
  };
  (text("contentType").unwrap_or_else(|| "text/plain".into()), text("contents").unwrap_or_default())
}

pub fn is_binary(content_type: &str, body: &[u8]) -> bool {
  let ct = content_type.to_ascii_lowercase();
  ct.starts_with("image/") || ct.starts_with("application/pdf") || ct.starts_with("application/octet-stream")
    || std::str::from_utf8(body).is_err()
}

pub fn register_part(i: InteractionHandle, p: InteractionPart, part: &Part) {
  let mut index: HashMap<String, usize> = HashMap::new();
  for (k, v) in &part.headers {
    // The content type travels with the body; pactffi_with_body sets the header from it.
    if part.body.is_some() && k.eq_ignore_ascii_case("content-type") {
      continue;
    }
    let n = index.entry(k.to_ascii_lowercase()).or_default();
    ffi::with_header_v2(i, p, k, *n, v);
    *n += 1;
  }
  if let Some(body) = &part.body {
    let ct = part.header("content-type").unwrap_or("text/plain");
    if is_binary(ct, body) {
      ffi::with_binary_body(i, p, ct, body);
    } else {
      ffi::with_body(i, p, ct, std::str::from_utf8(body).unwrap());
    }
  }
  if let Some(rules) = &part.matching_rules {
    ffi::with_matching_rules(i, p, rules);
  }
}

/// Adds one HTTP interaction to `pact` with the calls a client library would make.
pub fn register(pact: PactHandle, interaction: &Interaction) -> InteractionHandle {
  let i = ffi::new_interaction(pact, &interaction.description);
  ffi::with_request(i, &interaction.method, &interaction.path);
  let mut index: HashMap<String, usize> = HashMap::new();
  for (k, v) in parse_query(&interaction.query) {
    let n = index.entry(k.clone()).or_default();
    ffi::with_query_parameter_v2(i, &k, *n, &v);
    *n += 1;
  }
  register_part(i, InteractionPart_Request, &interaction.request);
  ffi::response_status(i, interaction.status);
  register_part(i, InteractionPart_Response, &interaction.response);
  i
}
