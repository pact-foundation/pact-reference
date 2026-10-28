//! Matching of headers whose values arrive over several field lines, or are combined into one.

use itertools::Itertools;
use pact_models::headers::{MULTI_VALUE_HEADERS, PARAMETERISED_HEADERS, SINGLE_VALUE_HEADERS};
use pact_models::matchingrules::MatchingRule;
use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
use pact_models::path_exp::{DocPath, PathToken};

use crate::{CommonMismatch, MatchingContext};
use crate::headers::match_header_lines;

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

/// Delimiters inside which a comma does not separate list elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grammar {
  /// Only quoted strings protect a comma.
  Plain,
  /// `<…>` link targets protect a comma (RFC 8288 §3).
  Link,
  /// Nested `(…)` comments protect a comma (RFC 9110 §5.6.5). Quotes are ordinary text inside
  /// a comment, and `\` escapes the next character.
  Via
}

impl Grammar {
  fn for_header(name: &str) -> Grammar {
    if name.eq_ignore_ascii_case("link") {
      Grammar::Link
    } else if name.eq_ignore_ascii_case("via") {
      Grammar::Via
    } else {
      Grammar::Plain
    }
  }
}

/// Splits a list value on commas outside quoted strings (RFC 9110 §5.6.4) and outside the
/// grammar's delimiters, trimming each element and dropping empty ones (RFC 9110 §5.6.1.2).
/// An unbalanced quote or delimiter extends to the end of the value.
fn list_elements(value: &str, grammar: Grammar) -> Vec<&str> {
  let mut elements = vec![];
  let mut quoted = false;
  let mut escaped = false;
  let mut depth = 0usize;
  let mut start = 0;
  for (index, character) in value.char_indices() {
    let in_comment = grammar == Grammar::Via && depth > 0;
    if escaped {
      escaped = false;
    } else if (quoted || in_comment) && character == '\\' {
      escaped = true;
    } else if character == '"' && !in_comment {
      quoted = !quoted;
    } else if quoted {
      continue;
    } else if grammar == Grammar::Link && character == '<' {
      depth = 1;
    } else if grammar == Grammar::Link && character == '>' {
      depth = 0;
    } else if grammar == Grammar::Via && character == '(' {
      depth += 1;
    } else if grammar == Grammar::Via && character == ')' {
      depth = depth.saturating_sub(1);
    } else if character == ',' && depth == 0 {
      elements.push(value[start..index].trim());
      start = index + 1;
    }
  }
  elements.push(value[start..].trim());
  elements.retain(|element| !element.is_empty());
  elements
}

/// Normalises a list value for comparison: whitespace around commas outside quoted strings
/// and the header's delimiters is dropped, as are empty elements.
pub(crate) fn normalise_list(name: &str, value: &str) -> String {
  list_elements(value, Grammar::for_header(name)).join(",")
}

/// Splits a header's field lines into its list elements.
pub(crate) fn split_list_elements(name: &str, values: &[String]) -> Vec<String> {
  let grammar = Grammar::for_header(name);
  values.iter()
    .flat_map(|value| list_elements(value, grammar))
    .map(str::to_string)
    .collect()
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

/// True for rules that apply to a header's list of values rather than to one value.
pub(crate) fn is_list_aware(rule: &MatchingRule) -> bool {
  matches!(rule,
    MatchingRule::MinType(_) | MatchingRule::MaxType(_) | MatchingRule::MinMaxType(_, _) |
    MatchingRule::EachValue(_) | MatchingRule::ArrayContains(_))
}

fn per_line_warning(key: &str, joined_actual: &str) -> String {
  format!("Header '{}' only matched when its field lines were compared one at a time. \
    Per-line matching of repeated headers is deprecated: RFC 9110 §5.3 treats repeated field lines as one line with the values joined by commas, \
    and a future release will compare only the joined value '{}'.", key, joined_actual)
}

fn list_warning(key: &str, count: usize) -> String {
  format!("Header '{}' only matched when its field lines were treated as whole values. \
    Matching repeated field lines as whole values is deprecated: RFC 9110 §5.3 treats them as one comma-joined list, \
    and a future release will split it into list elements (RFC 9110 §5.6.1), which gives {} elements here. \
    Quote values that contain commas, as RFC 9110 §5.5 advises, \
    e.g. Example-Dates: \"Sat, 04 May 1996\", \"Wed, 14 Sep 2005\".", key, count)
}

fn is_header_path(path: &DocPath, key: &str) -> bool {
  path.first_field().is_some_and(|field| field.to_lowercase() == key)
}

/// The rules at the header's own path.
fn bare_rules(key: &str, context: &dyn MatchingContext) -> Vec<MatchingRule> {
  let key = key.to_lowercase();
  context.matchers().rules.iter()
    .filter(|(path, _)| path.len() == 2 && is_header_path(path, &key))
    .flat_map(|(_, list)| list.rules.iter().cloned())
    .collect()
}

/// True when a rule sits at `$.<key>.<n>` or `$.<key>[<n>]` for some `n < len`, or at
/// `$.<key>[*]` or `$.<key>.*`, which address every element.
fn has_indexed_rule(key: &str, len: usize, context: &dyn MatchingContext) -> bool {
  let key = key.to_lowercase();
  context.matchers().rules.keys()
    .filter(|path| path.len() == 3 && is_header_path(path, &key))
    .any(|path| match path.tokens().last() {
      Some(PathToken::Index(index)) => *index < len,
      Some(PathToken::Field(field)) => field.parse::<usize>().is_ok_and(|index| index < len),
      Some(PathToken::Star | PathToken::StarIndex) => len > 0,
      _ => false
    })
}

/// Matches a custom header where either side has more than one field line.
pub(crate) fn match_repeated_header(
  key: &str,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> RepeatedHeaderResult {
  if rules_defined(key, context) {
    let bare = bare_rules(key, context);
    let elements = split_list_elements(key, actual);
    if bare.iter().any(is_list_aware) || has_indexed_rule(key, expected.len().max(elements.len()), context) {
      let primary = match_header_lines(key, expected, &elements, context);
      if primary.is_empty() {
        return RepeatedHeaderResult::default();
      }
      if match_header_lines(key, expected, actual, context).is_empty() {
        return RepeatedHeaderResult { mismatches: vec![], warning: Some(list_warning(key, elements.len())) };
      }
      return RepeatedHeaderResult { mismatches: primary, warning: None };
    }

    let joined_actual = join_field_lines(actual);
    let primary = match_header_lines(key, &[join_field_lines(expected)], std::slice::from_ref(&joined_actual), context);
    if primary.is_empty() {
      return RepeatedHeaderResult::default();
    }
    let per_line = match_header_lines(key, expected, actual, context);
    if per_line.is_empty() {
      let warning = if bare.is_empty() {
        per_line_warning(key, &joined_actual)
      } else {
        let each_value = bare.iter()
          .map(|rule| MatchingRule::EachValue(MatchingRuleDefinition::new(
            expected.first().cloned().unwrap_or_default(), ValueType::String, rule.clone(), None, String::new())).to_json().to_string())
          .join(", ");
        format!("{} To match each value separately, use {} instead.", per_line_warning(key, &joined_actual), each_value)
      };
      return RepeatedHeaderResult { mismatches: vec![], warning: Some(warning) };
    }
    // The joined mismatch alone doesn't say which value failed, so report the per-line ones as well.
    return RepeatedHeaderResult { mismatches: primary.into_iter().chain(per_line).collect(), warning: None };
  }

  let joined_expected = join_field_lines(expected);
  let joined_actual = join_field_lines(actual);
  if normalise_list(key, &joined_expected) == normalise_list(key, &joined_actual) {
    RepeatedHeaderResult::default()
  } else if match_header_lines(key, expected, actual, context).is_empty() {
    RepeatedHeaderResult {
      mismatches: vec![],
      warning: Some(per_line_warning(key, &joined_actual))
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

  use pact_models::matchingrules::{MatchingRuleCategory, RuleLogic};
  use pact_models::path_exp::DocPath;

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
    // An unbalanced quote runs to the end of the joined value, but not past the end of its own line.
    let expected = vec!["\"a".to_string(), "b".to_string()];
    let actual = vec!["\"a ".to_string(), "b".to_string()];
    let result = match_repeated_header("X-Request", &expected, &actual, &CoreMatchingContext::default());
    assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    assert_eq!(result.warning.as_deref(), Some("Header 'X-Request' only matched when its field lines were \
      compared one at a time. Per-line matching of repeated headers is deprecated: RFC 9110 §5.3 treats \
      repeated field lines as one line with the values joined by commas, and a future release will compare \
      only the joined value '\"a , b'."));
    assert!(result.warning.unwrap().contains("deprecated"));
  }

  #[test]
  fn scalar_fallback_warning_suggests_each_value() {
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap("$.x-request"), MatchingRule::Regex(r"^first-\d+$".to_string()), RuleLogic::And);
    let context = CoreMatchingContext { matchers: category, ..CoreMatchingContext::default() };
    let expected = vec!["first-1".to_string(), "first-2".to_string()];
    let actual = vec!["first-42".to_string(), "first-99".to_string()];
    let result = match_repeated_header("X-Request", &expected, &actual, &context);
    assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    let warning = result.warning.unwrap();
    assert!(warning.contains("RFC 9110 §5.3"), "{warning}");
    assert!(warning.contains("eachValue"), "{warning}");
    assert!(warning.contains("first-"), "{warning}");
  }

  #[test]
  fn scalar_rule_mismatch_names_the_failing_value() {
    // compatibility-suite V2 http_consumer "Supports matchers for repeated request headers (negative case)"
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap("$.x-test"), MatchingRule::Regex(r"\d{1,4}".to_string()), RuleLogic::And);
    let context = CoreMatchingContext { matchers: category, ..CoreMatchingContext::default() };
    let expected = vec!["1000".to_string()];
    let actual = vec!["1000".to_string(), "1234".to_string(), "9999ABC".to_string()];
    let result = match_repeated_header("X-Test", &expected, &actual, &context);
    assert!(result.warning.is_none());
    assert!(result.mismatches.iter().any(|m| m.description.contains(r"Expected '9999ABC' to match '\d{1,4}'")),
      "{:?}", result.mismatches);
  }

  #[test]
  fn scalar_rules_with_no_expected_lines_do_not_panic() {
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap("$.x-request"), MatchingRule::Regex(r"^first-\d+$".to_string()), RuleLogic::And);
    let context = CoreMatchingContext { matchers: category, ..CoreMatchingContext::default() };
    let actual = vec!["first-42".to_string(), "first-99".to_string()];
    let result = match_repeated_header("X-Request", &[], &actual, &context);
    assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    assert!(result.warning.is_some());
  }

  #[rstest]
  #[case(&["a, b", "c"], &["a", "b", "c"])]
  #[case(&["a, , b,"], &["a", "b"])]
  #[case(&[r#""one,two", three"#], &[r#""one,two""#, "three"])]
  #[case(&[r#""one\",two", three"#], &[r#""one\",two""#, "three"])]
  #[case(&[r#"{"a":1,"b":2}, x"#], &[r#"{"a":1"#, r#""b":2}"#, "x"])]
  fn split_list_elements_splits_on_commas_outside_quotes(#[case] lines: &[&str], #[case] elements: &[&str]) {
    let lines = lines.iter().map(|line| line.to_string()).collect_vec();
    assert_eq!(split_list_elements("X-Request", &lines), elements);
  }

  #[test]
  fn split_list_elements_keeps_an_unbalanced_quote_as_one_element() {
    let lines = vec!["\"abc, def".to_string()];
    assert_eq!(split_list_elements("X-Request", &lines), vec!["\"abc, def"]);
  }

  #[test]
  fn split_list_elements_keeps_link_targets_whole() {
    let lines = vec![r#"<https://a.example/?f=id,name>; rel="next", <https://a.example/?p=9>; rel="last""#.to_string()];
    assert_eq!(split_list_elements("Link", &lines), vec![
      r#"<https://a.example/?f=id,name>; rel="next""#,
      r#"<https://a.example/?p=9>; rel="last""#
    ]);
    assert_eq!(split_list_elements("LINK", &lines).len(), 2);
  }

  #[test]
  fn split_list_elements_keeps_via_comments_whole() {
    let lines = vec!["1.1 proxy (Squid, v5), 1.0 fred".to_string()];
    assert_eq!(split_list_elements("Via", &lines), vec!["1.1 proxy (Squid, v5)", "1.0 fred"]);
    assert_eq!(split_list_elements("via", &lines).len(), 2);
  }

  #[test]
  fn split_list_elements_nests_via_comments() {
    let lines = vec!["1.1 a (x (y, z), w), 1.0 b".to_string()];
    assert_eq!(split_list_elements("Via", &lines), vec!["1.1 a (x (y, z), w)", "1.0 b"]);
  }

  #[test]
  fn split_list_elements_reads_brackets_literally_for_other_headers() {
    let lines = vec!["(a, b)".to_string()];
    assert_eq!(split_list_elements("X-Request", &lines), vec!["(a", "b)"]);
    let lines = vec!["<a, b>".to_string()];
    assert_eq!(split_list_elements("X-Request", &lines), vec!["<a", "b>"]);
    assert_eq!(split_list_elements("Via", &["<a, b>".to_string()]), vec!["<a", "b>"]);
    assert_eq!(split_list_elements("Link", &["(a, b)".to_string()]), vec!["(a", "b)"]);
  }

  #[test]
  fn split_list_elements_ignores_delimiters_inside_quotes() {
    let lines = vec![r#"<a>; title="x<y", <b>"#.to_string()];
    assert_eq!(split_list_elements("Link", &lines), vec![r#"<a>; title="x<y""#, "<b>"]);
    let lines = vec![r#"1.1 a "x(", 1.0 b"#.to_string()];
    assert_eq!(split_list_elements("Via", &lines), vec![r#"1.1 a "x(""#, "1.0 b"]);
  }

  #[test]
  fn split_list_elements_treats_quotes_in_via_comments_as_text() {
    let lines = vec![r#"1.1 a (the "best, proxy), 1.0 b"#.to_string()];
    assert_eq!(split_list_elements("Via", &lines), vec![r#"1.1 a (the "best, proxy)"#, "1.0 b"]);
  }

  #[test]
  fn split_list_elements_honours_quoted_pairs_in_via_comments() {
    let lines = vec![r#"1.1 a (x \) y, z), 1.0 b"#.to_string()];
    assert_eq!(split_list_elements("Via", &lines), vec![r#"1.1 a (x \) y, z)"#, "1.0 b"]);
  }

  #[test]
  fn split_list_elements_keeps_an_unbalanced_delimiter_as_one_element() {
    assert_eq!(split_list_elements("Link", &["<a, b".to_string()]), vec!["<a, b"]);
    assert_eq!(split_list_elements("Via", &["1.1 a (b, c".to_string()]), vec!["1.1 a (b, c"]);
    assert_eq!(split_list_elements("Via", &["1.1 a), 1.0 b".to_string()]), vec!["1.1 a)", "1.0 b"]);
  }

  #[test]
  fn split_list_elements_keeps_multibyte_characters_next_to_commas() {
    let lines = vec!["é,ü".to_string(), "ß".to_string()];
    assert_eq!(split_list_elements("X-Request", &lines), vec!["é", "ü", "ß"]);
  }

  #[test]
  fn list_fallback_warns_of_deprecation() {
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap("$.x-request"), MatchingRule::MaxType(2), RuleLogic::And);
    let context = CoreMatchingContext { matchers: category, ..CoreMatchingContext::default() };
    let expected = vec!["a".to_string(), "b".to_string()];
    let actual = vec!["x, y".to_string(), "z".to_string()];
    let result = match_repeated_header("X-Request", &expected, &actual, &context);
    assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    assert_eq!(result.warning.as_deref(), Some("Header 'X-Request' only matched when its field lines were \
      treated as whole values. Matching repeated field lines as whole values is deprecated: RFC 9110 §5.3 \
      treats them as one comma-joined list, and a future release will split it into list elements \
      (RFC 9110 §5.6.1), which gives 3 elements here. Quote values that contain commas, as RFC 9110 §5.5 \
      advises, e.g. Example-Dates: \"Sat, 04 May 1996\", \"Wed, 14 Sep 2005\"."));
  }

  #[rstest]
  #[case("$.x-request", false)]
  #[case("$.x-request.1", true)]
  #[case("$.x-request[1]", true)]
  #[case("$.x-request[2]", false)]
  #[case("$.x-request[*]", true)]
  #[case("$.x-request.*", true)]
  #[case("$.x-request.foo", false)]
  #[case("$.x-other[0]", false)]
  fn indexed_rules_need_an_index_within_the_list(#[case] path: &'static str, #[case] indexed: bool) {
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap(path), MatchingRule::Regex("^a$".to_string()), RuleLogic::And);
    let context = CoreMatchingContext { matchers: category, ..CoreMatchingContext::default() };
    assert_eq!(has_indexed_rule("X-Request", 2, &context), indexed);
  }

  #[rstest]
  #[case("a, b", "a,b")]
  #[case(" a ,\t b ", "a,b")]
  #[case("a, , b,", "a,b")]
  #[case(r#""x , y", z"#, r#""x , y",z"#)]
  #[case(r#""x\", y", z"#, r#""x\", y",z"#)]
  #[case("", "")]
  fn normalise_list_ignores_whitespace_and_empty_elements_outside_quotes(#[case] value: &str, #[case] normalised: &str) {
    assert_eq!(normalise_list("X-Request", value), normalised);
  }
}

#[cfg(test)]
pub(crate) mod cases {
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleLogic};
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
  use pact_models::path_exp::DocPath;

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

  fn rule(path: &'static str, matcher: MatchingRule) -> MatchingRuleCategory {
    let mut category = MatchingRuleCategory::empty("header");
    category.add_rule(DocPath::new_unwrap(path), matcher, RuleLogic::And);
    category
  }

  fn regex_joined() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::Regex("^a, b$".to_string())) }
  fn regex_first() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::Regex(r"^first-\d+$".to_string())) }
  fn regex_request() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::Regex(r"^request\d$".to_string())) }
  fn type_rule() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::Type) }
  fn each_value(path: &'static str, regex: &str) -> MatchingRuleCategory {
    rule(path, MatchingRule::EachValue(MatchingRuleDefinition::new("1".into(), ValueType::String,
      MatchingRule::Regex(regex.to_string()), None, String::new())))
  }
  fn each_value_digits() -> MatchingRuleCategory { each_value("$.x-request", r"^\d+$") }
  fn each_value_dates() -> MatchingRuleCategory {
    each_value("$.x-sunset", r"^\w{3}, \d{2} \w{3} \d{4} \d{2}:\d{2}:\d{2} GMT$")
  }
  fn each_value_link() -> MatchingRuleCategory { each_value("$.link", r#"^<[^>]+>; rel="\w+"$"#) }
  fn via_max_type() -> MatchingRuleCategory { rule("$.via", MatchingRule::MaxType(2)) }
  fn min_type() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::MinType(2)) }
  fn max_type() -> MatchingRuleCategory { rule("$.x-request", MatchingRule::MaxType(2)) }
  fn indexed() -> MatchingRuleCategory {
    let mut category = rule("$.x-request.0", MatchingRule::Regex("^first-[0-9]+$".to_string()));
    category.add_rule(DocPath::new_unwrap("$.x-request.1"), MatchingRule::Regex("^second-[0-9]+$".to_string()), RuleLogic::And);
    category
  }

  fn wildcard_regex() -> MatchingRuleCategory { rule("$.x-request[*]", MatchingRule::Regex(r"^\d+$".to_string())) }

  const SUNSET: &[&str] = &["Sat, 01 Mar 2025 00:00:00 GMT", "Sun, 01 Jun 2025 00:00:00 GMT"];
  const LINKS: &[&str] = &[r#"<https://api.example.com/items?page=2&fields=id,name>; rel="next""#,
    r#"<https://api.example.com/items?page=9&fields=id,name>; rel="last""#];

  pub(crate) fn all() -> Vec<Case> {
    vec![
      Case { name: "regex joined, combined", header: "X-Request", expected: &["a", "b"], actual: &["a, b"], rules: regex_joined, matches: true, warns: false },
      Case { name: "regex joined, lines", header: "X-Request", expected: &["a", "b"], actual: &["a", "b"], rules: regex_joined, matches: true, warns: false },
      Case { name: "regex per-line fallback", header: "X-Request", expected: &["first-1", "first-2"], actual: &["first-42", "first-99"], rules: regex_first, matches: true, warns: true },
      Case { name: "regex combined fails", header: "X-Request", expected: &["first-1", "first-2"], actual: &["first-42, first-99"], rules: regex_first, matches: false, warns: false },
      Case { name: "regex one expected", header: "X-Request", expected: &["first-1"], actual: &["first-42", "first-99"], rules: regex_first, matches: true, warns: true },
      Case { name: "type", header: "X-Request", expected: &["a", "b"], actual: &["x, y, z"], rules: type_rule, matches: true, warns: false },
      Case { name: "each-value mixed", header: "X-Request", expected: &["1", "2"], actual: &["3, 4", "567"], rules: each_value_digits, matches: true, warns: false },
      Case { name: "each-value lines", header: "X-Request", expected: &["1", "2"], actual: &["3", "4", "567"], rules: each_value_digits, matches: true, warns: false },
      Case { name: "min-type combined", header: "X-Request", expected: &["a", "b"], actual: &["x, y"], rules: min_type, matches: true, warns: false },
      Case { name: "max-type fallback", header: "X-Request", expected: &["a", "b"], actual: &["x, y", "z"], rules: max_type, matches: true, warns: true },
      Case { name: "each-value dates fallback", header: "X-Sunset", expected: SUNSET, actual: SUNSET, rules: each_value_dates, matches: true, warns: true },
      Case { name: "each-value link", header: "Link", expected: LINKS, actual: LINKS, rules: each_value_link, matches: true, warns: false },
      Case { name: "each-value link combined", header: "Link", expected: LINKS,
        actual: &[r#"<https://api.example.com/items?page=2&fields=id,name>; rel="next", <https://api.example.com/items?page=9&fields=id,name>; rel="last""#],
        rules: each_value_link, matches: true, warns: false },
      Case { name: "via max-type", header: "Via", expected: &["1.1 a", "1.1 b"], actual: &["1.1 proxy (Squid, v5), 1.0 fred"], rules: via_max_type, matches: true, warns: false },
      Case { name: "indexed combined", header: "X-Request", expected: &["first-1", "second-2"], actual: &["first-42, second-99"], rules: indexed, matches: true, warns: false },
      Case { name: "indexed wrong", header: "X-Request", expected: &["first-1", "second-2"], actual: &["first-42, first-99"], rules: indexed, matches: false, warns: false },
      Case { name: "wildcard regex combined", header: "X-Request", expected: &["1", "2"], actual: &["3, 4", "567"], rules: wildcard_regex, matches: true, warns: false },
      Case { name: "case mix", header: "X-Request", expected: &["request1", "request2"], actual: &["request1", "request2"], rules: regex_request, matches: true, warns: true },
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
      plain("quoted OWS", "X-Request", &["\"x , y\"", "z"], &["\"x, y\"", "z"], false, false),
      plain("unbalanced quote fallback", "X-Request", &["\"a", "b"], &["\"a ", "b"], true, true),
      plain("singleton untouched", "Set-Cookie", &["one", "two"], &["one, two"], false, false),
      plain("singleton separate lines", "Set-Cookie", &["a=1; Path=/", "b=2; Expires=Sat, 01 Mar 2025 00:00:00 GMT"],
        &["a=1; Path=/", "b=2; Expires=Sat, 01 Mar 2025 00:00:00 GMT"], true, false),
      plain("link whitespace inside target", "Link",
        &[r#"<https://a.example/x,y>; rel="a""#, r#"<https://b.example/>; rel="b""#],
        &[r#"<https://a.example/x, y>; rel="a", <https://b.example/>; rel="b""#], false, false),
      plain("via comment whitespace", "Via", &["1.0 fred (a,b)", "1.1 p"], &["1.0 fred (a, b), 1.1 p"], false, false),
      plain("single multi-value OWS", "Cache-Control", &["no-cache,no-store"], &["no-cache, no-store"], true, false),
      plain("single multi-value quoted whitespace", "Cache-Control", &["private=\"x,y\""], &["private=\"x, y\""], false, false),
      plain("single link whitespace inside target", "Link",
        &[r#"<https://a.example/?x=1,2>; rel="a""#], &[r#"<https://a.example/?x=1, 2>; rel="a""#], false, false),
      plain("single quoted whitespace", "X-Request", &["\"x,y\""], &["\"x, y\""], false, false),
      plain("single via comment whitespace", "Via", &["1.0 fred (a,b)"], &["1.0 fred (a, b)"], false, false)
    ]
  }
}
