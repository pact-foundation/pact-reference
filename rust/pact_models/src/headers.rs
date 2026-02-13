pub static PARAMETERISED_HEADERS: [&str; 2] = ["accept", "content-type"];
pub static SINGLE_VALUE_HEADERS: [&str; 9] = [
  "date",
  "accept-datetime",
  "if-modified-since",
  "if-unmodified-since",
  "expires",
  "retry-after",
  "last-modified",
  "set-cookie",
  "user-agent",
];
pub static MULTI_VALUE_HEADERS: [&str; 12] = [
  "accept",
  "accept-encoding",
  "accept-language",
  "access-control-allow-headers",
  "access-control-allow-methods",
  "access-control-expose-headers",
  "access-control-request-headers",
  "allow",
  "cache-control",
  "if-match",
  "if-none-match",
  "vary"
];

/// Tries to parse the header value into multiple values, taking into account headers that should
/// not be split.
pub fn parse_header(name: &str, value: &str) -> Vec<String> {
  if SINGLE_VALUE_HEADERS.contains(&name.to_lowercase().as_str()) {
    vec![ value.trim().to_string() ]
  } else {
    value.split(',').map(|v| v.trim().to_string()).collect()
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use crate::headers::parse_header;

  #[test]
  fn parse_simple_header_value() {
    let parsed = parse_header("X", "Y");
    expect!(parsed).to(be_equal_to(vec!["Y"]));
  }

  #[test]
  fn parse_multi_value_header_value() {
    let parsed = parse_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
    expect!(parsed).to(be_equal_to(vec!["POST", "GET", "OPTIONS"]));
  }

  #[test]
  fn parse_multi_value_header_value_with_parameters() {
    let parsed = parse_header("accept", "text/html,application/xhtml+xml, application/xml;q=0.9,*/*; q=0.8");
    expect!(parsed).to(be_equal_to(vec!["text/html", "application/xhtml+xml", "application/xml;q=0.9", "*/*; q=0.8"]));
  }

  #[test]
  fn parse_known_single_value_header_value() {
    let parsed = parse_header("Last-Modified", "Mon, 01 Dec 2008 01:15:39 GMT");
    expect!(parsed).to(be_equal_to(vec!["Mon, 01 Dec 2008 01:15:39 GMT"]));
  }

  #[test]
  fn parse_user_agent_as_single_value() {
    let parsed = parse_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) QtWebEngine/6.6.3 Chrome/112.0.5615.213 Safari/537.36");
    expect!(parsed).to(be_equal_to(vec!["Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) QtWebEngine/6.6.3 Chrome/112.0.5615.213 Safari/537.36"]));
  }

  // ========== REPRODUCTION TESTS FOR ISSUE: pact-js#1058 ==========
  // See: https://github.com/pact-foundation/pact-js/issues/1058
  //
  // These tests demonstrate the bug where custom headers containing commas
  // (like JSON values) are incorrectly split.

  #[test]
  fn parse_custom_header_with_json_value_bug_demonstration() {
    // This test DEMONSTRATES THE BUG - it shows the CURRENT (incorrect) behavior
    // A custom header with JSON containing commas should NOT be split
    let parsed = parse_header(
      "X-Custom-Header",
      r#"{"id":"asd-asdasd-sd","additionalInfo":"some additional string"}"#
    );

    // CURRENT BUGGY BEHAVIOR: Header is incorrectly split at the comma
    // This assertion passes with the current code, but it SHOULD NOT - this is the bug!
    expect!(parsed.len()).to(be_greater_than(1)); // Bug: splits into multiple values
    expect!(parsed).to(be_equal_to(vec![
      r#"{"id":"asd-asdasd-sd""#,                     // First fragment - invalid JSON!
      r#""additionalInfo":"some additional string"}"# // Second fragment - invalid JSON!
    ]));
  }

  #[test]
  #[ignore] // This test represents the EXPECTED behavior, ignored until bug is fixed
  fn parse_custom_header_should_not_split_unknown_headers() {
    // EXPECTED BEHAVIOR: Unknown/custom headers should NOT be split by comma
    // They should be treated as single values (like Pact-JVM does after fix 8c5b0b1)
    let parsed = parse_header(
      "X-Custom-Header",
      r#"{"id":"asd-asdasd-sd","additionalInfo":"some additional string"}"#
    );

    // After the fix, this should be the behavior:
    expect!(parsed.len()).to(be_equal_to(1));
    expect!(parsed).to(be_equal_to(vec![
      r#"{"id":"asd-asdasd-sd","additionalInfo":"some additional string"}"#
    ]));
  }
}
