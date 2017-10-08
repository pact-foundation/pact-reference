use pact_matching::models::*;

use super::request_builder::RequestBuilder;
use super::response_builder::ResponseBuilder;

/// Builder for `Interaction` objects. Normally created via
/// `PactBuilder::interaction`.
pub struct InteractionBuilder {
    description: String,
    provider_state: Option<String>,

    /// A builder for this interaction's `Request`.
    pub request: RequestBuilder,

    /// A builder for this interaction's `Response`.
    pub response: ResponseBuilder,
}

impl InteractionBuilder {
    /// Create a new interaction.
    pub fn new<D: Into<String>>(description: D) -> Self {
        InteractionBuilder {
            description: description.into(),
            provider_state: None,
            request: RequestBuilder::default(),
            response: ResponseBuilder::default(),
        }
    }

    /// Specify a "provider state" for this interaction. This is normally use to
    /// set up database fixtures when using a pact to test a provider.
    pub fn given<G: Into<String>>(&mut self, given: G) -> &mut Self {
        self.provider_state = Some(given.into());
        self
    }

    /// The interaction we've built.
    pub fn build(&self) -> Interaction {
        Interaction {
            description: self.description.clone(),
            provider_state: self.provider_state.clone(),
            request: self.request.build(),
            response: self.response.build(),
        }
    }
}
