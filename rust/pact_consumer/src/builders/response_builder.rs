use pact_matching::models::*;
use std::collections::HashMap;

use prelude::*;

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
            self.response.headers.get_or_insert_with(Default::default),
            self.response.matching_rules.get_or_insert_with(
                Default::default,
            ),
        )
    }

    fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut Matchers) {
        (
            &mut self.response.body,
            self.response.matching_rules.get_or_insert_with(
                Default::default,
            ),
        )

    }
}
