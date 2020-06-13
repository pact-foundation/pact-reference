use pact_matching::models::*;
use pact_matching::models::matchingrules::{MatchingRules, MatchingRule, RuleLogic};
use std::collections::HashMap;
use maplit::*;

use crate::prelude::*;

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
    pub fn success(&mut self) -> &mut Self {
        self.status(200);
        self.response.matching_rules.add_category("status").add_rule(&"status".to_string(), MatchingRule::MustBeOneOf(vec![200,201,202,203,204,205,206,207,208,209]), &RuleLogic::And);
        self
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
        self.response.clone()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        ResponseBuilder { response: Response::default() }
    }
}

impl HttpPartBuilder for ResponseBuilder {
  fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, Vec<String>>, &mut MatchingRules) {
    (
      self.response.headers.get_or_insert(hashmap!{}),
      &mut self.response.matching_rules,
    )
  }

  fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut MatchingRules) {
    (
      &mut self.response.body,
      &mut self.response.matching_rules,
    )
  }
}
