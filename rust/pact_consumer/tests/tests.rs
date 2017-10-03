// Put these in your crate root. You can add `#[cfg(test)]` before any
// crate that you only use in test mode.
#[macro_use]
extern crate maplit;
extern crate pact_consumer;
#[macro_use]
extern crate pact_matching;
extern crate reqwest;

use pact_consumer::*;
use pact_matching::models::OptionalBody;
use std::io::prelude::*;

/// This is supposed to be a doctest in lib.rs, but it's breaking there. This
/// is written in a "neutral" Rust style, using the standard test framework and
/// popular libraries.
#[test]
fn relocated_doctest() {
    // Define the Pact for the test (you can setup multiple interactions by chaining the given or upon_receiving calls)
    // Define the service consumer by name
    let pact_runner = ConsumerPactBuilder::consumer(s!("Consumer"))
        // Define the service provider that it has a pact with
        .has_pact_with(s!("Alice Service"))
        // defines a provider state. It is optional.
        .given("there is some good mallory".to_string())
        // upon_receiving starts a new interaction
        .upon_receiving("a retrieve Mallory request".to_string())
            // define the request, a GET (default) request to '/mallory'
            .path(s!("/mallory"))
        // define the response we want returned
        .will_respond_with()
            .status(200)
            .headers(hashmap!{ s!("Content-Type") => s!("text/html") })
            .body(OptionalBody::Present(s!("That is some good Mallory.")))
        .build();

    // Execute the run method to have the mock server run (the URL to the mock server will be passed in).
    // It takes a closure to execute your requests and returns a Pact VerificationResult.
    let result = pact_runner.run(&|url| {
        // You would use your actual client code here
        let mut response = reqwest::get(&format!("{}/mallory", url))
          .expect("could not fetch URL");
        let mut body = String::new();
        response.read_to_string(&mut body)
          .expect("could not read response body");
        assert_eq!(body, "That is some good Mallory.");
        Ok(())
    });

    // This means it is all good
    assert_eq!(result, VerificationResult::PactVerified);
}
