//! Small internal utility routines and extensions to other people's types.
//! Most of these are `pub(crate)`, which makes them available to the rest of
//! the crate, but prevents them from winding up in our public API.

use std::{io, thread};
use std::io::Write;

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
///
/// ```
/// use pact_consumer::prelude::*;
///
/// let actual = strip_null_fields(serde_json::json!([
///     null,
///     { "a": 1, "b": null },
/// ]));
/// let expected = serde_json::json!([
///     null,       // nulls in arrays are left alone.
///     { "a": 1 }, // nulls in objects are stripped.
/// ]);
/// assert_eq!(actual, expected);
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

/// Either panic with `msg`, or if we're already in the middle of a panic,
/// just print `msg` to standard error.
pub(crate) fn panic_or_print_error(msg: &anyhow::Error) {
    if thread::panicking() {
        // The current thread is panicking, so don't try to panic again, because
        // double panics don't print useful explanations of why the test failed.
        // Instead, just print to `stderr`. Ignore any errors, because there's
        // not much we can do if we can't panic and we can't write to `stderr`.
        let _ = writeln!(io::stderr(), "{}", msg);
    } else {
        panic!("{}", msg);
    }
}
