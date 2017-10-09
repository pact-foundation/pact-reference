//! Small internal utility routines and extensions to other people's types.
//! Most of these are `pub(crate)`, which makes them available to the rest of
//! the crate, but prevents them from winding up in our public API.

use regex::{Captures, Regex};

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


