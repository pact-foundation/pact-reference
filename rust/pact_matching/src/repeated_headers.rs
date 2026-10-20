//! Matching of headers whose values arrive over several field lines, or are combined into one.

use itertools::Itertools;
use pact_models::headers::{MULTI_VALUE_HEADERS, PARAMETERISED_HEADERS, SINGLE_VALUE_HEADERS};
use pact_models::path_exp::DocPath;

use crate::{CommonMismatch, MatchingContext};
use crate::headers::{match_header_lines, split_header_field_lines};

// MARK: Classification

/// How a header's field lines may be combined and split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderKind {
  /// A value that may itself contain commas, so it is never split.
  Singleton,
  /// A comma-separated list whose members carry `;` parameters.
  Parameterised,
  /// A comma-separated list of plain tokens.
  MultiValue,
  /// Any header not named in the pact_models lists.
  Custom,
}

/// Classifies a header by name, ignoring case. A name in several lists takes the first of
/// `Singleton`, `Parameterised`, `MultiValue`.
pub(crate) fn header_kind(name: &str) -> HeaderKind {
  let name = name.to_lowercase();
  let name = name.as_str();
  if SINGLE_VALUE_HEADERS.contains(&name) {
    HeaderKind::Singleton
  } else if PARAMETERISED_HEADERS.contains(&name) {
    HeaderKind::Parameterised
  } else if MULTI_VALUE_HEADERS.contains(&name) {
    HeaderKind::MultiValue
  } else {
    HeaderKind::Custom
  }
}

// MARK: Repeated headers

/// True when either side of a header has more than one field line.
pub(crate) fn is_repeated(expected: &[String], actual: &[String]) -> bool {
  expected.len() > 1 || actual.len() > 1
}

/// Combines field lines into one value. RFC 9110 §5.3 makes repeated field lines equivalent
/// to a single line with the values joined by commas.
pub(crate) fn join_field_lines(values: &[String]) -> String {
  values.join(", ")
}

/// Normalises a list value for comparison: whitespace around commas outside quoted strings
/// is dropped, as are empty elements (RFC 9110 §5.6.1.2).
pub(crate) fn normalise_list(value: &str) -> String {
  let mut elements = vec![];
  let mut quoted = false;
  let mut escaped = false;
  let mut start = 0;
  for (index, character) in value.char_indices() {
    if escaped {
      escaped = false;
    } else if quoted && character == '\\' {
      escaped = true;
    } else if character == '"' {
      quoted = !quoted;
    } else if character == ',' && !quoted {
      elements.push(value[start..index].trim());
      start = index + 1;
    }
  }
  elements.push(value[start..].trim());
  elements.iter().filter(|element| !element.is_empty()).join(",")
}

/// The outcome of matching a repeated header, with any warning to log.
#[derive(Debug, Clone, Default)]
pub(crate) struct RepeatedHeaderResult {
  pub mismatches: Vec<CommonMismatch>,
  pub warning: Option<String>
}

fn rules_defined(key: &str, context: &dyn MatchingContext) -> bool {
  let key = key.to_lowercase();
  context.matcher_is_defined(&DocPath::root().join(key.as_str())) ||
    context.matchers().rules.keys()
      .any(|path| path.first_field().is_some_and(|field| field.to_lowercase() == key))
}

/// Matches a custom header where either side has more than one field line.
pub(crate) fn match_repeated_header(
  key: &str,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> RepeatedHeaderResult {
  if rules_defined(key, context) {
    let actual = if expected.len() > 1 { split_header_field_lines(actual) } else { actual.to_vec() };
    return RepeatedHeaderResult {
      mismatches: match_header_lines(key, expected, &actual, context),
      warning: None
    };
  }

  let joined_expected = join_field_lines(expected);
  let joined_actual = join_field_lines(actual);
  if normalise_list(&joined_expected) == normalise_list(&joined_actual) {
    RepeatedHeaderResult::default()
  } else if match_header_lines(key, expected, actual, context).is_empty() {
    RepeatedHeaderResult {
      mismatches: vec![],
      warning: Some(format!("Header '{}' only matched when its field lines were compared one at a time. \
        Per-line matching of repeated headers is deprecated: RFC 9110 §5.3 treats repeated field lines as one line with the values joined by commas, \
        and a future release will compare only the joined value '{}'.", key, joined_actual))
    }
  } else {
    RepeatedHeaderResult {
      mismatches: vec![CommonMismatch {
        path: key.to_string(),
        expected: joined_expected.clone(),
        actual: joined_actual.clone(),
        description: format!("Mismatch with header '{}': Expected '{}' to be equal to '{}'",
          key, joined_actual, joined_expected)
      }],
      warning: None
    }
  }
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  use crate::CoreMatchingContext;

  use super::*;

  #[rstest]
  #[case("Set-Cookie", HeaderKind::Singleton)]
  #[case("user-agent", HeaderKind::Singleton)]
  #[case("Content-Type", HeaderKind::Parameterised)]
  #[case("Accept", HeaderKind::Parameterised)]
  #[case("Cache-Control", HeaderKind::MultiValue)]
  #[case("X-Request", HeaderKind::Custom)]
  #[case("Link", HeaderKind::Custom)]
  fn header_kind_classifies(#[case] name: &str, #[case] kind: HeaderKind) {
    assert_eq!(header_kind(name), kind);
  }

  #[test]
  fn per_line_fallback_warns_of_deprecation() {
    let expected = vec!["\"x , y\"".to_string(), "z".to_string()];
    let actual = vec!["\"x, y\"".to_string(), "z".to_string()];
    let result = match_repeated_header("X-Request", &expected, &actual, &CoreMatchingContext::default());
    assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    assert_eq!(result.warning.as_deref(), Some("Header 'X-Request' only matched when its field lines were \
      compared one at a time. Per-line matching of repeated headers is deprecated: RFC 9110 §5.3 treats \
      repeated field lines as one line with the values joined by commas, and a future release will compare \
      only the joined value '\"x, y\", z'."));
    assert!(result.warning.unwrap().contains("deprecated"));
  }

  #[rstest]
  #[case("a, b", "a,b")]
  #[case(" a ,\t b ", "a,b")]
  #[case("a, , b,", "a,b")]
  #[case(r#""x , y", z"#, r#""x , y",z"#)]
  #[case(r#""x\", y", z"#, r#""x\", y",z"#)]
  #[case("", "")]
  fn normalise_list_ignores_whitespace_and_empty_elements_outside_quotes(#[case] value: &str, #[case] normalised: &str) {
    assert_eq!(normalise_list(value), normalised);
  }
}

#[cfg(test)]
pub(crate) mod cases {
  use pact_models::matchingrules::MatchingRuleCategory;

  /// One repeated-header scenario, run against both engines.
  pub(crate) struct Case {
    pub name: &'static str,
    pub header: &'static str,
    pub expected: &'static [&'static str],
    pub actual: &'static [&'static str],
    pub rules: fn() -> MatchingRuleCategory,
    pub matches: bool,
    pub warns: bool
  }

  fn no_rules() -> MatchingRuleCategory {
    MatchingRuleCategory::empty("header")
  }

  const fn plain(
    name: &'static str,
    header: &'static str,
    expected: &'static [&'static str],
    actual: &'static [&'static str],
    matches: bool,
    warns: bool
  ) -> Case {
    Case { name, header, expected, actual, rules: no_rules, matches, warns }
  }

  pub(crate) fn all() -> Vec<Case> {
    vec![
      plain("combined line", "X-Request", &["request1", "request2"], &["request1, request2"], true, false),
      plain("separate lines", "X-Request", &["request1", "request2"], &["request1", "request2"], true, false),
      plain("mixed grouping", "X-Request", &["request1", "request2", "request3"], &["request1, request2", "request3"], true, false),
      plain("no whitespace", "X-Request", &["request1", "request2"], &["request1,request2"], true, false),
      plain("OWS both sides", "X-Request", &["request1", "request2"], &["request1 ,\t request2"], true, false),
      plain("comma in value", "X-Request", &["a, b", "c"], &["a, b", "c"], true, false),
      plain("dates", "X-Sunset",
        &["Sat, 01 Mar 2025 00:00:00 GMT", "Sun, 01 Jun 2025 00:00:00 GMT"],
        &["Sat, 01 Mar 2025 00:00:00 GMT", "Sun, 01 Jun 2025 00:00:00 GMT"], true, false),
      plain("link", "Link",
        &[r#"<https://api.example.com/items?page=2&fields=id,name>; rel="next""#,
          r#"<https://api.example.com/items?page=9&fields=id,name>; rel="last""#],
        &[r#"<https://api.example.com/items?page=2&fields=id,name>; rel="next""#,
          r#"<https://api.example.com/items?page=9&fields=id,name>; rel="last""#], true, false),
      plain("quoted comma", "X-Request", &["\"one,two\"", "three"], &["\"one,two\", three"], true, false),
      plain("trailing comma", "X-Request", &["request1", "request2"], &["request1, request2,"], true, false),
      plain("middle empty", "X-Request", &["request1", "request2"], &["request1, , request2"], true, false),
      plain("one expected, two lines", "X-Request", &["request1, request2"], &["request1", "request2"], true, false),
      plain("empty expected member", "X-Request", &["", "a"], &["a"], true, false),
      plain("reversed", "X-Request", &["request1", "request2"], &["request2, request1"], false, false),
      plain("extra member", "X-Request", &["request1", "request2"], &["request1, request2, extra"], false, false),
      plain("missing member", "X-Request", &["request1", "request2"], &["request1"], false, false),
      plain("quoted whole", "X-Request", &["request1", "request2"], &["\"request1, request2\""], false, false),
      plain("quoted OWS fallback", "X-Request", &["\"x , y\"", "z"], &["\"x, y\"", "z"], true, true),
      plain("singleton untouched", "Set-Cookie", &["one", "two"], &["one, two"], false, false)
    ]
  }
}
