# Pact test DSL for writing consumer pact tests in Rust

This library provides a test DSL for writing consumer pact tests in Rust. It is based on the
[V3 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3).

[Online rust docs](https://docs.rs/pact_consumer/)

## To use it

To use it, add it to your dev-dependencies in your cargo manifest:

```
[dev-dependencies]
pact_consumer = "0.2.0"
```

You can now write a pact test using the consumer DSL.

// TODO: This is out of date now

```rust
use pact_consumer::prelude::*;
use pact_consumer::*;

#[test]
fn a_service_consumer_side_of_a_pact_goes_a_little_something_like_this() {

    // Define the Pact for the test (you can setup multiple interactions by chaining the given or upon_receiving calls)
    let pact_runner = ConsumerPactBuilder::consumer(s!("Consumer")) // Define the service consumer by name
        .has_pact_with(s!("Alice Service"))                         // Define the service provider that it has a pact with
        .given("there is some good mallory".to_string())            // defines a provider state. It is optional.
        .upon_receiving("a retrieve Mallory request".to_string())   // upon_receiving starts a new interaction
            .path(s!("/mallory"))                                   // define the request, a GET (default) request to '/mallory'
        .will_respond_with()                                        // define the response we want returned
            .status(200)
            .headers(hashmap!{ s!("Content-Type") => s!("text/html") })
            .body(OptionalBody::Present(s!("That is some good Mallory.")))
        .build();

    // Execute the run method to have the mock server run (the URL to the mock server will be passed in).
    // It takes a closure to execute your requests and returns a Pact VerificationResult.
    let result = pact_runner.run(&|url| {
        let client = Client { url: url.clone(), .. Client::default() }; // You would use your actual client code here
        let result = client.fetch("/mallory"); // we get our client code to execute the request
        expect!(result).to(be_ok().value("That is some good Mallory."));
        Ok(())
    });
    expect!(result).to(be_equal_to(VerificationResult::PactVerified)); // This means it is all good
}
```

### Changing the output directory

By default, the pact files will be written to `target/pacts`. To change this, set the environment variable `PACT_OUTPUT_DIR`.
