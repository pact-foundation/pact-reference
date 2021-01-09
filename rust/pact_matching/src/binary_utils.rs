use serde_json::Value;
use crate::{Mismatch, MatchingContext};
use crate::models::matchingrules::{RuleLogic, MatchingRule};
use crate::matchers::{Matches, match_values};
use itertools::Itertools;
use log::*;
use crate::models::HttpPart;
use http::header::{HeaderMap, HeaderName};
use std::collections::HashMap;
use std::convert::TryInto;
use onig::Regex;

static ROOT: &str = "$";

pub fn match_content_type<S>(data: &[u8], expected_content_type: S) -> Result<(), String>
  where S: Into<String> {
  let result = tree_magic_mini::from_u8(data);
  let expected = expected_content_type.into();
  let matches = result == expected;
  debug!("Matching binary contents by content type: expected '{}', detected '{}' -> {}",
         expected, result, matches);
  if matches {
    Ok(())
  } else {
    Err(format!("Expected binary contents to have content type '{}' but detected contents was '{}'",
      expected, result))
  }
}

pub fn convert_data(data: &Value) -> Vec<u8> {
  match data {
    &Value::String(ref s) => base64::decode(s.as_str()).unwrap_or_else(|_| s.clone().into_bytes()),
    _ => data.to_string().into_bytes()
  }
}

pub fn match_octet_stream(expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  let expected = expected.body().value();
  let actual = actual.body().value();
  debug!("matching binary contents ({} bytes)", actual.len());
  let path = vec!["$"];
  if context.matcher_is_defined(&path) {
    match context.select_best_matcher(&path) {
      None => mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        mismatch: format!("No matcher found for category 'body' and path '{}'", path.iter().join("."))}),
      Some(ref rulelist) => {
        let results = rulelist.rules.iter().map(|rule| expected.matches(&actual.as_slice(), rule)).collect::<Vec<Result<(), String>>>();
        match rulelist.rule_logic {
          RuleLogic::And => for result in results {
            if let Err(err) = result {
              mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
                actual: Some(actual.clone()),
                mismatch: err })
            }
          },
          RuleLogic::Or => {
            if results.iter().all(|result| result.is_err()) {
              for result in results {
                if let Err(err) = result {
                  mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
                    actual: Some(actual.clone()),
                    mismatch: err })
                }
              }
            }
          }
        }
      }
    }
  } else if expected != actual {
    mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
      actual: Some(actual.clone()),
      mismatch: format!("Expected binary data of {} bytes but received {} bytes", expected.len(), actual.len()) });
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

enum MimePart {
  Field(MimeField),
  File(MimeFile)
}

impl MimePart {
  fn name(&self) -> &String {
    match self {
      Self::Field(field) => &field.name,
      Self::File(file) => &file.name,
    }
  }
}

struct MimeField {
  name: String,
  data: String
}

#[derive(Debug)]
struct MimeFile {
  name: String,
  content_type: Option<mime::Mime>,
  filename: String,
  data: Vec<u8>
}

pub fn match_mime_multipart(expected: &dyn HttpPart, actual: &dyn HttpPart, context: &MatchingContext) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  debug!("matching MIME multipart contents");

  let actual_parts = parse_multipart(&mut actual.body().value().as_slice(), actual.headers());
  let expected_parts = parse_multipart(&mut expected.body().value().as_slice(), expected.headers());

  if expected_parts.is_err() || actual_parts.is_err() {
    match expected_parts {
      Err(e) => {
        mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
          actual: Some(actual.body().value().clone().into()),
          mismatch: format!("Failed to parse the expected body as a MIME multipart body: '{}'", e)});
      },
      _ => ()
    }
    match actual_parts {
      Err(e) => {
        mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
          actual: Some(actual.body().value().clone().into()),
          mismatch: format!("Failed to parse the actual body as a MIME multipart body: '{}'", e)});
      },
      _ => ()
    }
  } else {
    let actual_parts = actual_parts.unwrap();
    let expected_parts = expected_parts.unwrap();

    for expected_part in expected_parts {
      let name = expected_part.name();

      debug!("Comparing MIME field multipart '{}'", expected_part.name());
      match actual_parts.iter().find(|part| part.name() == expected_part.name()) {
        Some(actual_part) => for error in match_mime_part(&expected_part, actual_part, context).err().unwrap_or_default() {
          mismatches.push(error);
        },
        None => {
          debug!("MIME multipart '{}' is missing in the actual body", name);
          mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(name.clone().into_bytes()),
            actual: None,
            mismatch: format!("Expected a MIME part '{}' but was missing", name)});
        }
      }
    }
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

fn match_mime_part(expected: &MimePart, actual: &MimePart, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let key = expected.name();

  match (expected, actual) {
    (MimePart::Field(expected_field), MimePart::Field(actual_field)) => {
      match_field(key, &expected_field.data, &actual_field.data, context)
    },
    (MimePart::File(expected_file), MimePart::File(actual_file)) => {
      match_file(key, expected_file, actual_file, context)
    }
    (MimePart::Field(_), MimePart::File(_)) => {
      Err(vec![
        Mismatch::BodyMismatch { path: s!("$"), expected: Some(key.clone().into_bytes()),
          actual: None,
          mismatch: format!("Expected a MIME field '{}' but was file", key)}
      ])
    },
    (MimePart::File(_), MimePart::Field(_)) => {
      Err(vec![
        Mismatch::BodyMismatch { path: s!("$"), expected: Some(key.clone().into_bytes()),
          actual: None,
          mismatch: format!("Expected a MIME file '{}' but was field", key)}
      ])
    }
  }
}

fn match_field(key: &String, expected: &String, actual: &String, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = vec![ROOT, key.as_str()];
  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path $.{}", key);
    match_values(&path, context, expected, actual)
  } else {
    expected.matches(actual, &MatchingRule::Equality).map_err(|err|
      vec![format!("MIME part '{}': {}", key, err)]
    )
  };
  log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.join("."), matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::BodyMismatch {
        path: path.join("."),
        expected: Some(expected.as_bytes().to_vec()),
        actual: Some(actual.as_bytes().to_vec()),
        mismatch: message.clone()
      }
    }).collect()
  })
}

fn first(bytes: &[u8], len: usize) -> &[u8] {
  if bytes.len() <= len {
    bytes
  } else {
    bytes.split_at(len).0
  }
}

impl Matches<MimeFile> for MimeFile {
  fn matches(&self, actual: &MimeFile, matcher: &MatchingRule) -> Result<(), String> {
    log::debug!("FilePart: comparing binary data to '{:?}' using {:?}", actual.content_type, matcher);
    match matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            match String::from_utf8(actual.data.clone()) {
              Ok(a) => if re.is_match(&a) {
                  Ok(())
                } else {
                  Err(format!("Expected binary file '{}' to match '{}'", actual.filename, regex))
                },
              Err(err) => Err(format!("Expected binary file to match '{}' but could convert the file to a string '{}' - {}",
                                      regex, actual.filename, err))
            }
          },
          Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        if self.data == actual.data {
          Ok(())
        } else {
          Err(format!("Expected binary file ({} bytes) starting with {:?} to be equal to ({} bytes) starting with {:?}",
          actual.data.len(), first(&actual.data, 20),
          self.data.len(), first(&self.data, 20)))
        }
      },
      MatchingRule::Include(ref substr) => {
        match String::from_utf8(actual.data.clone()) {
          Ok(actual_contents) => if actual_contents.contains(substr) {
            Ok(())
          } else {
            Err(format!("Expected binary file ({}) to include '{}'", actual.filename, substr))
          },
          Err(err) => Err(format!("Expected binary file to include '{}' but could not convert the file to a string '{}' - {}",
                                  substr, actual.filename, err))
        }
      },
      MatchingRule::ContentType(content_type) => match_content_type(&actual.data, content_type),
      _ => Err(format!("Unable to match binary file using {:?}", matcher))
    }
  }
}

fn match_file(key: &String, expected: &MimeFile, actual: &MimeFile, context: &MatchingContext) -> Result<(), Vec<Mismatch>> {
  let path = vec![ROOT, key.as_str()];
  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path $.{}", key);
    match_values( &path, context, expected, actual).map_err(|errors| {
      errors.iter().map(|err| Mismatch::BodyMismatch {
        path: path.join("."),
        expected: None,
        actual: None,
        mismatch: format!("MIME part '{}': {}", key, err)
      }).collect()
    })
  } else {
    if expected.content_type == actual.content_type {
      expected.matches(actual, &MatchingRule::Equality).map_err(|err|
        vec![Mismatch::BodyMismatch {
          path: path.join("."),
          expected: None,
          actual: None,
          mismatch: format!("MIME part '{}': {}", key, err)
        }]
      )
    } else {
      let expected_str = expected.content_type.as_ref().map(|mime| mime.to_string()).unwrap_or_else(|| format!("None"));
      let actual_str = actual.content_type.as_ref().map(|mime| mime.to_string()).unwrap_or_else(|| format!("None"));
      Err(vec![Mismatch::BodyTypeMismatch {
        expected: expected_str.clone(),
        actual: actual_str.clone(),
        mismatch: format!("Expected MIME part '{}' with content type '{}' but was '{}'",
                          key, expected_str, actual_str)
      }])
    }
  };
  log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.join("."), matcher_result);
  matcher_result
}

fn parse_multipart<R: std::io::Read>(body: R, headers: &Option<HashMap<String, Vec<String>>>) -> Result<Vec<MimePart>, String> {
  let boundary = get_multipart_boundary(headers)?;
  let mut mp = multipart::server::Multipart::with_body(body, boundary);

  let mut parts = vec![];

  use std::io::Read;

  loop {
    match mp.read_entry() {
      Ok(Some(mut entry)) => {
        let name = entry.headers.name.to_string();
        let content_type = entry.headers.content_type;

        let mut data = vec![];
        entry.data.read_to_end(&mut data).map_err(|e| format!("Failed to read multipart data: {}", e))?;

        if let Some(filename) = entry.headers.filename {
          parts.push(MimePart::File(MimeFile {
            name,
            content_type,
            filename,
            data,
          }));
        } else {
          parts.push(MimePart::Field(MimeField {
            name,
            data: String::from_utf8(data).map_err(|e| format!("Decode error: {}", e))?
          }))
        }
      },
      Ok(None) => return Ok(parts),
      Err(e) => return Err(format!("Failed to read multipart entry: {}", e)),
    }
  }
}

fn get_multipart_boundary(headers: &Option<HashMap<String, Vec<String>>>) -> Result<String, String> {
  let header_map = get_http_header_map(headers);
  let content_type = header_map.get(http::header::CONTENT_TYPE)
    .ok_or_else(|| "no content-type".to_owned())?
    .to_str()
    .map_err(|e| format!("invalid content-type: {}", e))?;

  let mime: mime::Mime = content_type.parse().map_err(|e| format!("invalid content-type: {}", e))?;

  if mime.type_() != mime::MULTIPART || mime.subtype() != mime::FORM_DATA {
    return Err(format!("expected content-type to be multipart/form-data"));
  }

  let boundary = mime.get_param(mime::BOUNDARY).ok_or_else(|| format!("no boundary in content-type"))?;

  Ok(boundary.as_str().to_owned())
}

fn get_http_header_map(h: &Option<HashMap<String, Vec<String>>>) -> HeaderMap {
  let mut headers = HeaderMap::new();
  if let Some(h) = h {
    for (key, values) in h {
      for value in values {
        if let (Ok(header_name), Ok(header_value)) = (HeaderName::from_bytes(key.as_bytes()), value.try_into()) {
          headers.append(header_name, header_value);
        }
      }
    }
  };
  headers
}

#[cfg(test)]
mod tests {
  use crate::models::{Request, OptionalBody};
  use crate::models::matchingrules::*;
  use crate::binary_utils::match_mime_multipart;
  use expectest::prelude::*;
  use hamcrest2::prelude::*;
  use crate::{DiffConfig, Mismatch, MatchingContext};
  use maplit::*;
  use std::str;

  fn mismatch(m: &Mismatch) -> &str {
    match m {
      Mismatch::BodyMismatch { mismatch, .. } => mismatch.as_str(),
      Mismatch::BodyTypeMismatch { mismatch, .. } => mismatch.as_str(),
      _ => ""
    }
  }

  #[test]
  fn match_mime_multipart_error_when_not_multipart() {
    let body = "not a multipart body";
    let request = Request {
      headers: Some(hashmap!{}),
      body: OptionalBody::Present(body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&request, &request, &context);

    let mismatches = result.unwrap_err();
    assert_that!(&mismatches, len(2));
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Failed to parse the expected body as a MIME multipart body: \'no content-type\'",
      "Failed to parse the actual body as a MIME multipart body: \'no content-type\'"
    ]));
  }

  #[test]
  fn match_mime_multipart_equal() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_missing_part() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected a MIME part \'age\' but was missing", "Expected a MIME part \'file\' but was missing"
    ]));
  }

  #[test]
  fn match_mime_multipart_different_values() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nFred\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n2 months\r\n\
      --4567\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'name': Expected 'Baxter' to be equal to 'Fred'",
      "MIME part 'age': Expected '1 month' to be equal to '2 months'",
      "MIME part 'file': Expected binary file (16 bytes) starting with [97, 44, 98, 44, 99, 44, 100, 13, 10, 52, 44, 53, 44, 54, 44, 55] to be equal to (16 bytes) starting with [49, 44, 50, 44, 51, 44, 52, 13, 10, 52, 44, 53, 44, 54, 44, 55]"
    ]));
  }

  #[test]
  fn match_mime_multipart_with_matching_rule() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.name" => [ MatchingRule::Regex(s!("^\\w+$")) ],
          "$.age" => [ MatchingRule::Regex(s!("^\\d+ months?+$")) ]
        }
      },
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nFred\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n2 months\r\n\
      --4567\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap());

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_different_content_type() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: text/html\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      <html>a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected MIME part 'file' with content type 'text/csv' but was 'text/html'"
    ]));
  }

  #[test]
  fn match_mime_multipart_content_type_matcher() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: image/png\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.htm\"\r\n\r\n\
      <html>1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.file" => [ MatchingRule::ContentType("image/png".into()) ]
        }
      },
      ..Request::default()
    };
    let bytes: [u8; 16] = [
        0x89, 0x50, 0x4e, 0x47,
        0x0d, 0x0a, 0x1a, 0x0a,
        0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52
     ];

    let mut actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: image/png\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.htm\"\r\n\r\n".as_bytes().to_vec();
    actual_body.extend_from_slice(&bytes);
    actual_body.extend_from_slice("\r\n--4567--\r\n".as_bytes());
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None),
      ..Request::default()
    };
    let context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap());

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_content_type_matcher_with_mismatch() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: application/jpeg\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.htm\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.file" => [ MatchingRule::ContentType("application/jpeg".into()) ]
        }
      },
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: application/jpeg\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.htm\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let context = MatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap());

    let result = match_mime_multipart(&expected, &actual, &context);

    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part \'file\': Expected binary contents to have content type \'application/jpeg\' but detected contents was \'text/plain\'"
    ]));
  }
}
