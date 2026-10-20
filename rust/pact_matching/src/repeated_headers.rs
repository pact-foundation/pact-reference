//! Classification of headers for matching repeated expectations against combined field lines.

use pact_models::headers::{MULTI_VALUE_HEADERS, PARAMETERISED_HEADERS, SINGLE_VALUE_HEADERS};

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

#[cfg(test)]
mod tests {
  use rstest::rstest;

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
}
