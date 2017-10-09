//! JSON "patterns", which can be used to either generate JSON documents or
//! match them.

use pact_matching::models::Matchers;
use regex::{Captures, Regex};
use std::fmt::Debug;

#[macro_use]
mod json_macros;
mod json_pattern;
mod special_rules;
mod string_pattern;

pub use self::json_pattern::*;
pub use self::special_rules::*;
pub use self::string_pattern::*;

/// Abstract interface to types which can:
///
/// 1. Generate example data.
/// 2. Match data returned by tests in various flexible ways, for example,
///    accepting all strings which match a regular expression.
///
/// For an overview of how the matching rules work, and what kinds of special
/// matching rules exist, see the [`pact_matching` documentation][spec].
///
/// The current version of this API will only work for `JsonPattern` and
/// `serde_json::Value`. Extending this scheme to work for XML would require
/// parameterizing the input and output types, and possibly other changes.
///
/// [spec]: https://docs.rs/pact_matching/0.2.2/pact_matching/
pub trait Pattern: Debug {
    /// What type of data can this pattern be matched against? What kind of
    /// example data does it generate?
    type Matches;

    /// Convert this `Matchable` into an example data value, stripping out
    /// any special match rules.
    fn to_example(&self) -> Self::Matches;

    /// Extract the matching rules from this `Matchable`, and insert them into
    /// `rules_out`, using `path` as the base path.
    ///
    /// This API corresponds to the [`Extract` code in Ruby][ruby].
    ///
    /// (The `path` parameter is represented as a `&str` here, which forces each
    /// recursive call to allocate strings. We could optimize this by using a
    /// custom `path` representation which worked like a stack-based linked list
    /// stored in reverse order, but that would add significant complexity.)
    ///
    /// [ruby]:
    /// https://github.com/pact-foundation/pact-support/blob/master/lib/pact/matching_rules/extract.rb
    fn extract_matching_rules(&self, path: &str, rules_out: &mut Matchers);
}

/// Format a JSON object key for use in a JSON path expression. If we were
/// more concerned about performance, we might try to come up with a scheme
/// to minimize string allocation here.
pub(crate) fn obj_key_for_path(key: &str) -> String {
    lazy_static! {
        // Only use "." syntax for things which are obvious identifiers.
        static ref IDENT: Regex = Regex::new(r#"^[_A-Za-z][_A-Za-z0-9]*$"#)
            .expect("could not parse IDENT regex");
        // Escape these characters when using string syntax.
        static ref ESCAPE: Regex = Regex::new(r#"\\|'"#)
            .expect("could not parse ESCAPE regex");
    }

    if IDENT.is_match(key) {
        format!(".{}", key)
    } else {
        format!(
            "['{}']",
            ESCAPE.replace_all(key, |caps: &Captures| format!(r#"\{}"#, &caps[0]))
        )
    }
}

#[test]
fn obj_key_for_path_quotes_keys_when_necessary() {
    assert_eq!(obj_key_for_path("foo"), ".foo");
    assert_eq!(obj_key_for_path("_foo"), "._foo");
    assert_eq!(obj_key_for_path("["), "['[']");

    // I don't actually know how the JSON Path specification wants us to handle
    // these cases, but we need to _something_ to avoid panics or passing
    // `Result` around everywhere, so let's go with JavaScript string escape
    // syntax.
    assert_eq!(obj_key_for_path(r#"''"#), r#"['\'\'']"#);
    assert_eq!(obj_key_for_path(r#"a'"#), r#"['a\'']"#);
    assert_eq!(obj_key_for_path(r#"\"#), r#"['\\']"#);
}
