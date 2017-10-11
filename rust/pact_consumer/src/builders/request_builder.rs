use pact_matching::models::*;
#[cfg(test)]
use regex::Regex;
use std::collections::HashMap;

use prelude::*;
use util::{GetDefaulting, obj_key_for_path};

/// Builder for `Request` objects. Normally created via `PactBuilder`.
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    /// Specify the request method. Defaults to `"GET"`.
    ///
    /// ```
    /// use pact_consumer::builders::RequestBuilder;
    /// use pact_consumer::prelude::*;
    ///
    /// let request = RequestBuilder::default().method("POST").build();
    /// assert_eq!(request.method, "POST");
    /// ```
    pub fn method<M: Into<String>>(&mut self, method: M) -> &mut Self {
        self.request.method = method.into();
        self
    }

    /// Set the HTTP method to `GET`. This is the default, so we don't actually
    /// care.
    pub fn get(&mut self) -> &mut Self {
        self.method("GET")
    }

    /// Set the HTTP method to `POST`.
    pub fn post(&mut self) -> &mut Self {
        self.method("POST")
    }

    /// Set the HTTP method to `PUT`.
    pub fn put(&mut self) -> &mut Self {
        self.method("PUT")
    }

    /// Set the HTTP method to `DELETE`.
    pub fn delete(&mut self) -> &mut Self {
        self.method("DELETE")
    }

    /// Specify the request path. Defaults to `"/"`.
    pub fn path<P: Into<StringPattern>>(&mut self, path: P) -> &mut Self {
        let path = path.into();
        self.request.path = path.to_example();
        path.extract_matching_rules(
            "$.path",
            &mut self.request.matching_rules.get_defaulting(),
        );
        self
    }

    /// Specify a query parameter. You may pass either a single value or
    /// a list of values to represent a repeated parameter.
    ///
    /// ```
    /// #[macro_use]
    /// extern crate pact_consumer;
    /// extern crate regex;
    ///
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::builders::RequestBuilder;
    /// use regex::Regex;
    ///
    /// # fn main() {
    /// RequestBuilder::default()
    ///     .query_param("simple", "value")
    ///     .query_param("pattern", term!("^[0-9]+$", "123"));
    /// # }
    /// ```
    ///
    /// To pass multiple parameters with the same name, call `query_param` more
    /// than once with the same `key`.
    pub fn query_param<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<StringPattern>,
    {
        let key = key.into();
        let value = value.into();

        // Extract our example JSON and add it the `Vec` for the appropriate
        // parameter.
        self.request
            .query
            .get_defaulting()
            .entry(key.clone())
            .or_insert_with(Default::default)
            .push(value.to_example());

        // Extract our matching rules.
        value.extract_matching_rules(
            &format!("$.query{}", obj_key_for_path(&key)),
            &mut self.request.matching_rules.get_defaulting(),
        );

        self
    }

    /// Build the specified `Request` object.
    pub fn build(&self) -> Request {
        let mut result = self.request.clone();
        if result.matching_rules.as_ref().map_or(false, |r| r.is_empty()) {
            // Empty matching rules break pact merging, so clean them up.
            result.matching_rules = None;
        }
        result
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        RequestBuilder { request: Request::default_request() }
    }
}

impl HttpPartBuilder for RequestBuilder {
    fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, String>, &mut Matchers) {
        (
            self.request.headers.get_defaulting(),
            self.request.matching_rules.get_defaulting(),
        )
    }

    fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut Matchers) {
        (
            &mut self.request.body,
            self.request.matching_rules.get_defaulting(),
        )
    }
}

#[test]
fn path_pattern() {
    let greeting_regex = Regex::new("/greeting/.*").unwrap();
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.path(Term::new(greeting_regex, "/greeting/hello"));
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.path("/greeting/hi"); })
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.path("/farewell/bye"); })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[test]
fn query_param_pattern() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.query_param("greeting", term!("^h.*$", "hello"));
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.query_param("greeting", "hi"); })
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.query_param("greeting", "bye"); })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[test]
fn query_param_with_underscore() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("get a user", |i| {
            i.request
                .path("/users")
                // This `term!` was being ignored in `pact_matching`, but only
                // if there was an underscore.
                .query_param("user_id", term!("^[0-9]+$", "1"));
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request
                .path("/users")
                // Call with a different ID than we expected.
                .query_param("user_id", "2"); })
        .build();
    assert_requests_match!(good, pattern);
}

#[test]
fn term_does_not_require_anchors() {
    use prelude::*;

    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            // Unfortunatley, we appear to need a leading "^" and trailing "$"
            // on this regex, or else it will match the other examples below.
            i.request.path(term!("^/users/[0-9]+$", "/users/12"));
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.path("/users/2"); })
        .build();
    let bad1 = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.path("/users/2/posts"); })
        .build();
    let bad2 = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.path("/account/1/users/2"); })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad1, pattern);
    assert_requests_do_not_match!(bad2, pattern);
}
