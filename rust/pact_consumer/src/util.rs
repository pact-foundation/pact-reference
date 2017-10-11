//! Small internal utility routines and extensions to other people's types.
//! Most of these are `pub(crate)`, which makes them available to the rest of
//! the crate, but prevents them from winding up in our public API.

use regex::{Captures, Regex};
use serde_json;

/// Internal helper method for `strip_null_fields`.
fn strip_null_fields_mut(json: &mut serde_json::Value) {
    use serde_json::Value;

    match *json {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            // Nothing to do.
        }
        Value::Array(ref mut arr) => {
            // Walk the array recursively, but leave `null` elements intact.
            for mut v in arr {
                strip_null_fields_mut(&mut v);
            }
        }
        Value::Object(ref mut obj) => {
            // Build a list of keys to remove. We need to do his first, because
            // we can't mutate a Rust collection while iterating over it. (Not
            // that you can't in _most_ languages, but Rust actually enforces
            // this.)
            let keys_to_remove = obj.iter().filter_map(|(k, v)| {
                if v.is_null() {
                    // Allocate a new copy of the string so that we don't hold
                    // on to a pointer into `obj`.
                    Some(k.to_owned())
                } else {
                    None
                }
            }).collect::<Vec<_>>();

            // Now remove our keys.
            for key in keys_to_remove {
                obj.remove(&key);
            }
        }
    }
}

/// Given a `serde_json::Value`, walk it recursively, removing any null fields,
/// and return the updated value. Because of how this is normally called, it
/// consumes its input value and returns the stripped value.
///
/// This function is most useful when serializing Rust types to JSON for
/// use with `each_like!`, because it follows the pact convention of removing
/// optional fields.
///
/// ```
/// #[macro_use]
/// extern crate pact_consumer;
/// #[macro_use]
/// extern crate serde_json;
///
/// use pact_consumer::prelude::*;
///
/// # fn main() {
/// let actual = strip_null_fields(json!([
///     null,
///     { "a": 1, "b": null },
/// ]));
/// let expected = json!([
///     null,       // nulls in arrays are left alone.
///     { "a": 1 }, // nulls in objects are stripped.
/// ]);
/// assert_eq!(actual, expected);
/// # }
/// ```
pub fn strip_null_fields<V>(json: V) -> serde_json::Value
where
    V: Into<serde_json::Value>,
{
    let mut json = json.into();
    strip_null_fields_mut(&mut json);
    json
}

/// Wrapper for `get_or_insert_with(Default::default)`, to simplify a common
/// pattern of code in this crate and reduce ugly line wrapping.
pub(crate) trait GetDefaulting<T: Default> {
    /// Get the contained value, or no contained value is present, create a
    /// value using `Default` and insert it, than return that.
    fn get_defaulting(&mut self) -> &mut T;
}

impl<T: Default> GetDefaulting<T> for Option<T> {
    fn get_defaulting(&mut self) -> &mut T {
        self.get_or_insert_with(Default::default)
    }
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


