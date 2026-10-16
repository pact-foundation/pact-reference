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
/// not be split. Only known multi-value headers (per RFC 7230/9110) are split on commas.
pub fn parse_header(name: &str, value: &str) -> Vec<String> {
  if MULTI_VALUE_HEADERS.contains(&name.to_lowercase().as_str()) {
    value.split(',').map(|v| v.trim().to_string()).collect()
  } else {
    vec![ value.trim().to_string() ]
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

  /// Verifies that custom headers containing JSON (with commas) are not split into multiple values.
  /// See: https://github.com/pact-foundation/pact-reference/commit/6a985b9d0e512973a18943d3eaacae564a773c74
  /// This test covers the original bug from Pact JS.
  #[test]
  fn parse_custom_header_should_not_split_unknown_headers() {
    // JSON header value containing commas
    let header_name = "x-custom-json";
    let header_value = "{\"foo\":\"bar,baz\",\"arr\":[1,2,3]}";
    let parsed = parse_header(header_name, header_value);
    // Should not split into multiple values
    expect!(parsed).to(be_equal_to(vec![header_value.to_string()]));
  }
}
