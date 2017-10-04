//! Macros for building `JsonPattern` objects.
//!
//! Much of the macro code below is directly copied from `serde_json` and
//! modified to construct `JsonPattern` objects instead of `serde_json::Value`
//! objects. The following copyright notice applies to this code:
//!
//! Copyright 2017 Serde Developers
//!
//! Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
//! http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
//! or http://opensource.org/licenses/MIT>, at your option. This file may not be
//! copied, modified, or distributed except according to those terms.

/// Construct a `pact_consumer::JsonPattern` object using a convenient syntax.
///
/// ```rust
/// // Place this declaration in your top-level `main.rs` or `lib.rs` file.
/// #[macro_use]
/// extern crate pact_consumer;
///
/// # fn main() {
/// json_pattern!({
///     "message": "Hello, world!",
///     "location": { "x": 1, "y": 2 },
///     "tags": ["interesting"]
/// });
/// # }
/// ```
///
/// The `json_pattern!` macro supports nested Rust expressions:
///
/// ```
/// // Place these declarations in your top-level `main.rs` or `lib.rs` file.
/// #[macro_use]
/// extern crate pact_consumer;
/// #[macro_use]
/// extern crate serde_derive;
/// #[macro_use]
/// extern crate serde_json;
///
/// use pact_consumer::JsonPattern;
///
/// #[derive(Serialize)]
/// struct Point {
///    x: f32,
///    y: f32,
/// }
///
/// # fn main() {
/// json_pattern!({
///     // You can use Rust expressions, as long as they support
///     // `Into<PatternJson>`.
///     "message": format!("Hello, {}!", "world"),
///
///     // You can also nest the `json!` macro to embed types which
///     // support `Serialize`.
///     "location": json!(Point { x: 1.0, y: 2.0 }),
/// });
/// # }
/// ```
#[macro_export]
macro_rules! json_pattern {
    // Hide distracting implementation details from the generated rustdoc.
    ($($json:tt)+) => {
        json_pattern_internal!($($json)+)
    };
}

// Our internal helper macro.
#[macro_export]
#[doc(hidden)]
macro_rules! json_pattern_internal {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...]. Produces a vec![...]
    // of the elements.
    //
    // Must be invoked as: json_internal!(@array [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////

    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        json_pattern_internal!(@array [$($elems,)* json_pattern_internal!(null)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        json_pattern_internal!(@array [$($elems,)* json_pattern_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        json_pattern_internal!(@array [$($elems,)* json_pattern_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        json_pattern_internal!(@array [$($elems,)* json_pattern_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        json_pattern_internal!(@array [$($elems,)* json_pattern_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        json_pattern_internal!(@array [$($elems,)*] $($rest)*)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: json_pattern_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        $object.insert(($($key)+).into(), $value);
        json_pattern_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        $object.insert(($($key)+).into(), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object [$($key)+] (json_pattern_internal!(null)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object [$($key)+] (json_pattern_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object [$($key)+] (json_pattern_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object [$($key)+] (json_pattern_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        json_pattern_internal!(@object $object [$($key)+] (json_pattern_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        json_pattern_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        json_pattern_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        unimplemented!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        unimplemented!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        json_pattern_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: json_internal!($($json)+)
    //////////////////////////////////////////////////////////////////////////

    (null) => {
        $crate::JsonPattern::null()
    };

    ([]) => {
        $crate::JsonPattern::Array(vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::JsonPattern::Array(json_pattern_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::JsonPattern::Object($crate::Map::new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::JsonPattern::Object({
            let mut object = $crate::Map::new();
            json_pattern_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        {
            let v: $crate::JsonPattern = $other.into();
            v
        }
    };
}

#[test]
fn trailing_commas() {
    json_pattern!({
        "a": 1,
        "b": 2,
    });

    json_pattern!([
        true,
        false,
    ]);
}

#[test]
fn true_false_and_null() {
    // These were all special-cased in the original `json!` macro, so make sure
    // they all still work.
    json_pattern!([
        true,
        null,
        false,
        { "false": false, "null": null, "true": true },
    ]);
}
