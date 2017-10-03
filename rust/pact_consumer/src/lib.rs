//! The `pact_consumer` crate provides the test DSL for writing consumer pact tests.
//! It implements the V2 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-2).
//!
//! ## To use it
//!
//! To use it, add it to your dev-dependencies in your cargo manifest and add an extern crate definition for it.
//!
//! ```ignore
//! [dev-dependencies]
//! pact_consumer = "0.2.0"
//! ```
//!
//! You can now write a pact test using the consumer DSL.
//!
//! ```
//! // TODO: This doctest has been moved to pact_consumer/tests/tests.rs
//! // pending a fix for https://github.com/rust-lang/cargo/issues/4567
//! ```

#![warn(missing_docs)]

#[macro_use] extern crate log;
#[macro_use] extern crate p_macro;
#[macro_use] extern crate maplit;
#[macro_use] extern crate pact_matching;
extern crate pact_mock_server;
extern crate uuid;

use pact_matching::models::*;
pub use pact_matching::models::OptionalBody;
use pact_mock_server::*;
use std::collections::HashMap;
use uuid::Uuid;
use std::panic::{self, AssertUnwindSafe};
use std::error::Error;

/// Result of running the pact test
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// The pact was verified OK
    PactVerified,
    /// There was a mismatch between the expectations and the actual requests
    PactMismatch(Vec<MatchResult>),
    /// The provided test code returned an error
    UserCodeFailed(String),
    /// There was a mismatch between the expectations and the actual requests and the user code
    /// returned an error
    PactMismatchAndUserCodeFailed(Vec<MatchResult>, String),
    /// There was an error trying to setup the pact test
    PactError(String)
}

/// Runner for a consumer pact test
#[derive(Debug, Clone)]
pub struct ConsumerPactRunner {
    /// The Pact that represents the expectations of the consumer test
    pact: Pact
}

impl ConsumerPactRunner {

    /// Starts a mock server for the pact and executes the closure
    pub fn run(&self, test: &Fn(String) -> Result<(), String>) -> VerificationResult {
        match start_mock_server(Uuid::new_v4().simple().to_string(), self.pact.clone(), 0) {
            Ok(mock_server_port) => {
                debug!("Mock server port is {}, running test ...", mock_server_port);
                let mock_server_url = lookup_mock_server_by_port(mock_server_port, &|ms| ms.url());
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    test(mock_server_url.unwrap())
                }));
                debug!("Test result = {:?}", result);
                let mock_server_result = lookup_mock_server_by_port(mock_server_port, &|ref mock_server| {
                    mock_server.mismatches().clone()
                }).unwrap();
                let test_result = match result {
                    Ok(result) => {
                        debug!("Pact test result: {:?}", result);
                        match result {
                            Ok(_) => {
                                if mock_server_result.is_empty() {
                                    VerificationResult::PactVerified
                                } else {
                                    VerificationResult::PactMismatch(mock_server_result)
                                }
                            },
                            Err(err) => {
                                if mock_server_result.is_empty() {
                                    VerificationResult::UserCodeFailed(err)
                                } else {
                                    VerificationResult::PactMismatchAndUserCodeFailed(
                                        mock_server_result, err)
                                }
                            }
                        }
                    },
                    Err(err) => {
                        debug!("Pact test panicked: {:?}", err);
                        if mock_server_result.is_empty() {
                            VerificationResult::UserCodeFailed(s!("Pact test panicked"))
                        } else {
                            VerificationResult::PactMismatchAndUserCodeFailed(mock_server_result,
                                s!("Pact test panicked"))
                        }
                    }
                };

                let final_test_result = match test_result {
                    VerificationResult::PactVerified => {
                        let write_pact_result = lookup_mock_server_by_port(mock_server_port, &|ref mock_server| {
                            mock_server.write_pact(&Some(s!("target/pacts")))
                        }).unwrap();
                        match write_pact_result {
                            Ok(_) => test_result,
                            Err(err) => VerificationResult::PactError(s!(err.description()))
                        }
                    },
                    _ => test_result
                };

                shutdown_mock_server_by_port(mock_server_port);

                final_test_result
            },
            Err(msg) => {
                error!("Could not start mock server: {}", msg);
                VerificationResult::PactError(msg)
            }
        }
    }

}

enum BuilderState {
    None,
    BuildingRequest,
    BuildingResponse
}

/// Struct to setup the consumer pact test expectations
pub struct ConsumerPactBuilder {
    pact: Pact,
    interaction: Interaction,
    state: BuilderState
}

impl ConsumerPactBuilder {

    /// Defines the consumer involved in the Pact
    pub fn consumer<S: Into<String>>(consumer_name: S) -> Self {
        ConsumerPactBuilder {
            pact: Pact { consumer: Consumer { name: consumer_name.into() }, .. Pact::default() },
            interaction: Interaction::default(),
            state: BuilderState::None
        }
    }

    /// Defines the provider involved in the Pact
    pub fn has_pact_with<S: Into<String>>(&mut self, provider_name: S) -> &mut Self {
        self.pact.provider.name = provider_name.into();
        self
    }

    /// Describe the state the provider needs to be in for the pact test to be verified. (Optional)
    pub fn given<S: Into<String>>(&mut self, provider_state: S) -> &mut Self {
        match self.state {
            BuilderState::None => (),
            _ => self.pact.interactions.push(self.interaction.clone())
        }
        self.interaction = Interaction {
            provider_state: Some(provider_state.into()),
            .. Interaction::default()
        };
        self.state = BuilderState::BuildingRequest;
        self
    }

    /// Description of the request that is expected to be received
    pub fn upon_receiving<S: Into<String>>(&mut self, description: S) -> &mut Self {
        self.push_interaction();
        self.interaction.description = description.into();
        self
    }

    /// The path of the request
    pub fn path<S: Into<String>>(&mut self, path: S) -> &mut Self {
        self.push_interaction();
        self.interaction.request.path = path.into();
        self
    }

    /// The HTTP method for the request
    pub fn method<S: Into<String>>(&mut self, method: S) -> &mut Self {
        self.push_interaction();
        self.interaction.request.method = method.into();
        self
    }

    /// Internal API for fetching a mutable version of our current `headers`.
    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        let opt_headers_mut: &mut Option<_> = match self.state {
            BuilderState::BuildingRequest => &mut self.interaction.request.headers,
            BuilderState::BuildingResponse => &mut self.interaction.response.headers,
            BuilderState::None => {
                self.state = BuilderState::BuildingRequest;
                &mut self.interaction.request.headers
            }
        };
        opt_headers_mut.get_or_insert_with(|| HashMap::new())
    }

    /// A header to be included in the request. Will overwrite previous headers
    /// of the same name.
    pub fn header<S1, S2>(&mut self, key: S1, value: S2) -> &mut Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.headers_mut().insert(key.into(), value.into());
        self
    }

    /// Headers to be included in the request. If called multiple times, this
    /// will merge the new headers with the old, overriding any duplicates.
    pub fn headers<M, S1, S2>(&mut self, headers: M) -> &mut Self
    where
        // This is a bit of Rust magic: We take some type that can be turned
        // into an iterator, and that iterator returns a pair of values that
        // can be turned into strings. This allows mixing `&str`, `String`,
        // `Vec<_>`, `&[_]` and actual iterators fairly freely. Generally,
        // this kind of type is overkill for anything except builder APIs.
        M: IntoIterator<Item = (S1, S2)>,
        S1: Into<String>,
        S2: Into<String>,
    {
        self.headers_mut()
            .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Internal function which returns a mutable version of our query.
    fn query_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        self.push_interaction();
        self.interaction.request.query.get_or_insert_with(|| HashMap::new())
    }

    /// The query string for the request. If called multiple times, this will
    /// merge the new query parameters with the old, overriding any duplicates.
    pub fn query<M, S1, V, S2>(&mut self, query: M) -> &mut Self
    where
        // See `headers` for an explanation of these type constraints.
        M: IntoIterator<Item = (S1, V)>,
        S1: Into<String>,
        V: IntoIterator<Item = S2>,
        S2: Into<String>,
    {
        self.query_mut().extend(query.into_iter().map(|(k, v)| {
            (k.into(), v.into_iter().map(|s| s.into()).collect())
        }));
        self
    }

    /// Internal function which returns a mutable version of our current body.
    fn body_mut(&mut self) -> &mut OptionalBody {
        match self.state {
            BuilderState::BuildingRequest => &mut self.interaction.request.body,
            BuilderState::BuildingResponse => &mut self.interaction.response.body,
            BuilderState::None => {
                self.state = BuilderState::BuildingRequest;
                &mut self.interaction.request.body
            }
        }
    }

    /// The body of the request.
    ///
    /// TODO: See discussion about `body` naming on
    /// https://github.com/pact-foundation/pact-reference/pull/18
    pub fn body(&mut self, body: OptionalBody) -> &mut Self {
        *self.body_mut() = body;
        self
    }

    /// The body of the request, which will be wrapped in
    /// `OptionalBody::Present` (the common default case).
    ///
    /// TODO: We may want to change this to `B: Into<Vec<u8>>` depending on what
    /// happens with https://github.com/pact-foundation/pact-reference/issues/19
    /// That will still do the right thing with `&str`.
    pub fn body_present<B: Into<String>>(&mut self, body: B) -> &mut Self {
        *self.body_mut() = OptionalBody::Present(body.into());
        self
    }

    fn push_interaction(&mut self) {
        match self.state {
            BuilderState::BuildingRequest => (),
            BuilderState::None => (),
            _ => {
                self.pact.interactions.push(self.interaction.clone());
                self.interaction = Interaction::default();
                self.state = BuilderState::BuildingRequest;
            }
        }
    }

    /// Define the response to return
    pub fn will_respond_with(&mut self) -> &mut Self {
        self.state = BuilderState::BuildingResponse;
        self
    }

    /// Response status code
    pub fn status(&mut self, status: u16) -> &mut Self {
        self.interaction.response.status = status;
        self.state = BuilderState::BuildingResponse;
        self
    }

    /// Terminates the DSL and builds a pact fragment to represent the interactions
    pub fn build(&mut self) -> ConsumerPactRunner {
        self.pact.interactions.push(self.interaction.clone());
        self.state = BuilderState::None;
        ConsumerPactRunner {
            pact: self.pact.clone()
        }
    }
}
