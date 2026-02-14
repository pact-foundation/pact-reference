pub static PARAMETERISED_HEADERS: [&str; 2] = ["accept", "content-type"];
pub static SINGLE_VALUE_HEADERS: [&str; 7] = [
  "date",
  "accept-datetime",
  "if-modified-since",
  "if-unmodified-since",
  "expires",
  "retry-after",
  "last-modified"
];

/// HTTP headers that are known to support multiple comma-separated values per RFC 7230/9110.
/// Only these headers should be parsed into multiple values; unknown headers should NOT be split
/// to avoid breaking values that legitimately contain commas (e.g., JSON-encoded values).
/// See: https://github.com/pact-foundation/pact-js/issues/1058
pub static MULTI_VALUE_HEADERS: [&str; 28] = [
  "accept",
  "accept-charset",
  "accept-encoding",
  "accept-language",
  "accept-ranges",
  "access-control-allow-headers",
  "access-control-allow-methods",
  "access-control-expose-headers",
  "access-control-request-headers",
  "allow",
  "cache-control",
  "connection",
  "content-encoding",
  "content-language",
  "expect",
  "if-match",
  "if-none-match",
  "pragma",
  "proxy-authenticate",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
  "vary",
  "via",
  "warning",
  "www-authenticate",
  "x-forwarded-for"
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
}
