use pact_matching::models::*;

use prelude::*;
use super::interaction_builder::InteractionBuilder;

/// Builder for `Pact` objects.
///
/// ```
/// #[macro_use]
/// extern crate pact_consumer;
///
/// use pact_consumer::prelude::*;
///
/// # fn main() {
/// let pact = PactBuilder::new("Greeting Client", "Greeting Server")
///     .interaction("asks for a greeting", |i| {
///         i.request.path("/greeting/hello");
///         i.response
///             .header("Content-Type", "application/json")
///             .json_body(json_pattern!({ "message": "hello" }));
///     })
///     .build();
///
/// // The request method and response status default as follows.
/// assert_eq!(pact.interactions[0].request.method, "GET");
/// assert_eq!(pact.interactions[0].response.status, 200);
/// # }
/// ```
pub struct PactBuilder {
    pact: Pact,
}

impl PactBuilder {
    /// Create a new `PactBuilder`, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new<C, P>(consumer: C, provider: P) -> Self
    where
        C: Into<String>,
        P: Into<String>,
    {
        let mut pact = Pact::default();
        pact.consumer = Consumer { name: consumer.into() };
        pact.provider = Provider { name: provider.into() };
        PactBuilder { pact: pact }
    }

    /// Add a new `Interaction` to the `Pact`.
    pub fn interaction<D, F>(&mut self, description: D, build_fn: F) -> &mut Self
    where
        D: Into<String>,
        F: FnOnce(&mut InteractionBuilder),
    {
        let mut interaction = InteractionBuilder::new(description.into());
        build_fn(&mut interaction);
        self.push_interaction(interaction.build())
    }

    /// Directly add a pre-built `Interaction` to our `Pact`. Normally it's
    /// easier to use `interaction` instead of this function.
    pub fn push_interaction(&mut self, interaction: Interaction) -> &mut Self {
        self.pact.interactions.push(interaction);
        self
    }

    /// Return the `Pact` we've built.
    pub fn build(&self) -> Pact {
        self.pact.clone()
    }
}

impl StartMockServer for PactBuilder {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::new(self.build())
    }
}
