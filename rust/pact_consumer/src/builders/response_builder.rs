use pact_matching::models::*;
use std::collections::HashMap;

use prelude::*;
use util::GetDefaulting;

/// Builder for `Response` objects. Normally created via `PactBuilder`.
pub struct ResponseBuilder {
    response: Response,
}

impl ResponseBuilder {
    /// Set the status code for the response. Defaults to `200`.
    ///
    /// ```
    /// use pact_consumer::builders::ResponseBuilder;
    /// use pact_consumer::prelude::*;
    ///
    /// let response = ResponseBuilder::default().status(404).build();
    /// assert_eq!(response.status, 404);
    /// ```
    pub fn status(&mut self, status: u16) -> &mut Self {
        self.response.status = status;
        self
    }

    // This is a partial list of popular HTTP status codes. If you use any
    // others regularly, feel free to add them.

    /// Set the status code to `200 OK`. (This is default.)
    pub fn ok(&mut self) -> &mut Self {
        self.status(200)
    }

    /// Set the status code to `201 Created`.
    pub fn created(&mut self) -> &mut Self {
        self.status(201)
    }

    /// Set the status code to `204 No Content`.
    pub fn no_content(&mut self) -> &mut Self {
        self.status(204)
    }

    /// Set the status code to `401 Unauthorized`.
    pub fn unauthorized(&mut self) -> &mut Self {
        self.status(401)
    }

    /// Set the status code to `403 Forbidden`.
    pub fn forbidden(&mut self) -> &mut Self {
        self.status(403)
    }

    /// Set the status code to `404 Not Found`.
    pub fn not_found(&mut self) -> &mut Self {
        self.status(404)
    }

    /// Build the specified `Response` object.
    pub fn build(&self) -> Response {
        let mut result = self.response.clone();
        if result.matching_rules.as_ref().map_or(false, |r| r.is_empty()) {
            // Empty matching rules break pact merging, so clean them up.
            result.matching_rules = None;
        }
        result
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        ResponseBuilder { response: Response::default_response() }
    }
}

impl HttpPartBuilder for ResponseBuilder {
    fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, String>, &mut Matchers) {
        (
            self.response.headers.get_defaulting(),
            self.response.matching_rules.get_defaulting(),
        )
    }

    fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut Matchers) {
        (
            &mut self.response.body,
            self.response.matching_rules.get_defaulting(),
        )

    }
}
