//! JSON "patterns", which can be used to either generate JSON documents or
//! match them.

use pact_matching::models::Matchers;
use serde_json;
use std::fmt::Debug;

#[macro_use]
mod json_macros;
mod json_pattern;
mod special_rules;

pub use self::json_pattern::*;
pub use self::special_rules::*;

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
pub trait Matchable: Debug {
    /// Convert this `Matchable` into an example data value, stripping out
    /// any special match rules.
    fn to_example(&self) -> serde_json::Value;

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
    fn extract_matching_rules(
        &self,
        path: &str,
        rules_out: &mut Matchers,
    );
}
