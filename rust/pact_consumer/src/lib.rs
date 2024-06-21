//! The `pact_consumer` crate provides tools for writing consumer [Pact
//! tests][pact]. It implements the [V3 Pact specification][spec]. You can also
//! use it as a simple HTTP mocking library for Rust.
//!
//! [pact]: https://docs.pact.io/ [spec]:
//! https://github.com/pact-foundation/pact-specification
//!
//! ## What is Pact?
//!
//! [Pact][pact] is a [cross-language standard][spec] for testing the
//! communication between the consumer of a REST API, and the code that provides
//! that API. Test cases are written from the consumer's perspective, and they
//! can then be exported testing the provider.
//!
//! The big advantages of Pact are:
//!
//! 1. The mocks you write to test the client can also be reused to verify that
//!    the server would actually respond the way the client expects. This gives
//!    the end-to-end assurance of integration tests (well, almost), but with
//!    the speed and convenience of unit tests.
//! 2. Pact has been implemented in many popular languages, so you can test
//!    clients and servers in multiple languages.
//!
//! Whenever possible, we try to use vocabulary similar to the Ruby or
//! JavaScript API for basic concepts, and we try to provide the same behavior.
//! But we offer many handy builder methods to make tests cleaner.
//!
//! ## How to use it
//!
//! To use this crate, add it to your `[dev-dependencies]` in your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! pact_consumer = "1.1"
//! ```
//!
//! Once this is done, you can then write the following inside a function marked
//! with `#[tokio::test]`:
//!
//! ```no_run
//! use pact_consumer::prelude::*;
//!
//! // Define the Pact for the test, specify the names of the consuming
//! // application and the provider application.
//! let provider_service = PactBuilder::new("Consumer", "Alice Service")
//!     // Start a new interaction. We can add as many interactions as we want.
//!     .interaction("a retrieve Mallory request", "", |mut i| {
//!         // Defines a provider state. It is optional.
//!         i.given("there is some good mallory");
//!         // Define the request, a GET (default) request to '/mallory'.
//!         i.request.path("/mallory");
//!         // Define the response we want returned. We assume a 200 OK
//!         // response by default.
//!         i.response
//!             .content_type("text/plain")
//!             .body("That is some good Mallory.");
//!         // Return the interaction builder back to the pact framework
//!         i
//!     })
//!     .start_mock_server(None, None);
//! ```
//!
//! You can than use an HTTP client like `reqwest` to make requests against your
//! server.
//!
//! ```rust
//! # tokio_test::block_on(async {
//! # use pact_models::pact::Pact;
//! # use std::io::Read;
//! # use pact_consumer::prelude::*;
//! # let provider_service = PactBuilder::new("Consumer", "Alice Service")
//! #     // Start a new interaction. We can add as many interactions as we want.
//! #     .interaction("a retrieve Mallory request", "", |mut i| {
//! #         // Defines a provider state. It is optional.
//! #         i.given("there is some good mallory");
//! #         // Define the request, a GET (default) request to '/mallory'.
//! #         i.request.path("/mallory");
//! #         // Define the response we want returned. We assume a 200 OK
//! #         // response by default.
//! #         i.response
//! #             .content_type("text/plain")
//! #             .body("That is some good Mallory.");
//! #         // Return the interaction builder back to the pact framework
//! #         i
//! #     }).start_mock_server(None, None);
//!
//! // You would use your actual client code here.
//! let mallory_url = provider_service.path("/mallory");
//! let mut response = reqwest::get(mallory_url).await.expect("could not fetch URL")
//!   .text().await.expect("Could not read response body");
//! assert_eq!(response, "That is some good Mallory.");
//!
//! // When `provider_service` goes out of scope, your pact will be validated,
//! // and the test will fail if the mock server didn't receive matching
//! // requests.
//! # });
//! ```
//!
//! ## Matching using patterns
//!
//! You can also use patterns like `like!`, `each_like!` or `term!` to allow
//! more general matches, and you can build complex patterns using the
//! `json_pattern!` macro:
//!
//! ```
//! use pact_consumer::prelude::*;
//!
//! PactBuilder::new("quotes client", "quotes service")
//!     .interaction("add a new quote to the database", "", |mut i| {
//!         i.request
//!             .post()
//!             .path("/quotes")
//!             .json_utf8()
//!             .json_body(json_pattern!({
//!                  // Allow the client to send any string as a quote.
//!                  // When testing the server, use "Eureka!".
//!                  "quote": like!("Eureka!"),
//!                  // Allow the client to send any string as an author.
//!                  // When testing the server, use "Archimedes".
//!                  "by": like!("Archimedes"),
//!                  // Allow the client to send an array of strings.
//!                  // When testing the server, send a single-item array
//!                  // containing the string "greek".
//!                  "tags": each_like!("greek"),
//!              }));
//!
//!         i.response
//!             .created()
//!             // Return a location of "/quotes/12" to the client. When
//!             // testing the server, allow it to return any numeric ID.
//!             .header("Location", term!("^/quotes/[0-9]+$", "/quotes/12"));
//!         i
//!     });
//! ```
//!
//! The key insight here is this "pact" can be used to test both the client and
//! the server:
//!
//! - When testing the **client**, we allow the request to be anything which
//!   matches the patterns—so `"quote"` can be any string, not just `"Eureka!"`.
//!   But we respond with the specified values, such as `"/quotes/12"`.
//! - When testing the **server**, we send the specified values, such as
//!   `"Eureka!"`. But we allow the server to respond with anything matching the
//!   regular expression `^/quotes/[0-9]+$`, because we don't know what database
//!   ID it will use.
//!
//! Also, when testing the server, we may need to set up particular database
//! fixtures. This can be done using the string passed to `given` in the
//! examples above.
//!
//! ## Testing using domain objects
//!
//! Normally, it's best to generate your JSON using your actual domain objects.
//! This is easier, and it reduces duplication in your code.
//!
//! ```
//! use pact_consumer::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! /// Our application's domain object representing a user.
//! #[derive(Deserialize, Serialize)]
//! struct User {
//!     /// All users have this field.
//!     name: String,
//!
//!     /// The server may omit this field when sending JSON, or it may send it
//!     /// as `null`.
//!     comment: Option<String>,
//! }
//!
//! // Create our example user using our normal application objects.
//! let example = User {
//!     name: "J. Smith".to_owned(),
//!     comment: None,
//! };
//!
//! PactBuilder::new("consumer", "provider")
//!     .interaction("get all users", "", |mut i| {
//!         i.given("a list of users in the database");
//!         i.request.path("/users");
//!         i.response
//!             .json_utf8()
//!             .json_body(each_like!(
//!                 // Here, `strip_null_fields` will remove `comment` from
//!                 // the generated JSON, allowing our pattern to match
//!                 // missing comments, null comments, and comments with
//!                 // strings.
//!                 strip_null_fields(serde_json::json!(example)),
//!             ));
//!         i
//!     })
//!     .build();
//! ```
//!
//! ## Testing messages
//!
//! Testing message consumers is supported. There are two types: asynchronous messages and synchronous request/response.
//!
//! ### Asynchronous messages
//!
//! Asynchronous messages are you normal type of single shot or fire and forget type messages. They are typically sent to a
//! message queue or topic as a notification or event. With Pact tests, we will be testing that our consumer of the messages
//! works with the messages setup as the expectations in test. This should be the message handler code that processes the
//! actual messages that come off the message queue in production.
//!
//! The generated Pact file from the test run can then be used to verify whatever created the messages adheres to the Pact
//! file.
//!
//! ```rust
//! use pact_consumer::prelude::*;
//! use expectest::prelude::*;
//! use serde_json::{Value, from_slice};
//!
//! // Define the Pact for the test (you can setup multiple interactions by chaining the given or message_interaction calls)
//! // For messages we need to use the V4 Pact format.
//! let mut pact_builder = PactBuilder::new_v4("message-consumer", "message-provider"); // Define the message consumer and provider by name
//! pact_builder
//!   // Adds an interaction given the message description and type.
//!   .message_interaction("Mallory Message", |mut i| {
//!     // defines a provider state. It is optional.
//!     i.given("there is some good mallory".to_string());
//!     // Can set the test name (optional)
//!     i.test_name("a_message_consumer_side_of_a_pact_goes_a_little_something_like_this");
//!     // Set the contents of the message. Here we use a JSON pattern, so that matching rules are applied
//!     i.json_body(json_pattern!({
//!       "mallory": like!("That is some good Mallory.")
//!     }));
//!     // Need to return the mutated interaction builder
//!     i
//!   });
//!
//! // This will return each message configured with the Pact builder. We need to process them
//! // with out message handler (it should be the one used to actually process your messages).
//! for message in pact_builder.messages() {
//!   let bytes = message.contents.contents.value().unwrap();
//!
//!   // Process the message here as it would if it came off the queue
//!   let message: Value = serde_json::from_slice(&bytes).unwrap();
//!
//!   // Make some assertions on the processed value
//!   expect!(message.as_object().unwrap().get("mallory")).to(be_some().value("That is some good Mallory."));
//! }
//! ```
//!
//! ### Synchronous request/response messages
//!
//! Synchronous request/response messages are a form of message interchange were a request message is sent to another service and
//! one or more response messages are returned. Examples of this would be things like Websockets and gRPC.
//!
//! ```rust
//! # use bytes::Bytes;
//! # struct MessageHandler {}
//! # struct MockProvider { pub message: Bytes }
//! # impl MessageHandler { fn process(bytes: Bytes, provider: &MockProvider) -> anyhow::Result<&str> { Ok("That is some good Mallory.") } }
//! use pact_consumer::prelude::*;
//! use pact_consumer::*;
//! use expectest::prelude::*;
//! use serde_json::{Value, from_slice};
//!
//! // Define the Pact for the test (you can setup multiple interactions by chaining the given or message_interaction calls)
//! // For synchronous messages we also need to use the V4 Pact format.
//! let mut pact_builder = PactBuilder::new_v4("message-consumer", "message-provider"); // Define the message consumer and provider by name
//! pact_builder
//!   // Adds an interaction given the message description and type.
//!   .synchronous_message_interaction("Mallory Message", |mut i| {
//!     // defines a provider state. It is optional.
//!     i.given("there is some good mallory".to_string());
//!     // Can set the test name (optional)
//!     i.test_name("a_synchronous_message_consumer_side_of_a_pact_goes_a_little_something_like_this");
//!     // Set the contents of the request message. Here we use a JSON pattern, so that matching rules are applied.
//!     // This is the request message that is going to be forwarded to the provider
//!     i.request_json_body(json_pattern!({
//!       "requestFor": like!("Some good Mallory, please.")
//!     }));
//!     // Add a response message we expect the provider to return. You can call this multiple times to add multiple messages.
//!     i.response_json_body(json_pattern!({
//!       "mallory": like!("That is some good Mallory.")
//!     }));
//!     // Need to return the mutated interaction builder
//!     i
//!   });
//!
//! // For our test we want to invoke our message handling code that is going to initialise the request
//! // to the provider with the request message. But we need some mechanism to mock the response
//! // with the resulting response message so we can confirm our message handler works with it.
//! for message in pact_builder.synchronous_messages() {
//!   // the request message we must make
//!   let request_message_bytes = message.request.contents.value().unwrap();
//!   // the response message we expect to receive from the provider
//!   let response_message_bytes = message.response.first().unwrap().contents.value().unwrap();
//!
//!   // We use a mock here, assuming there is a Trait that controls the response message that our
//!   // mock can implement.
//!   let mock_provider = MockProvider { message: response_message_bytes };
//!   // Invoke our message handler to send the request message from the Pact interaction and then
//!   // wait for the response message. In this case it will be the response via the mock provider.
//!   let response = MessageHandler::process(request_message_bytes, &mock_provider);
//!
//!   // Make some assertions on the processed value
//!   expect!(response).to(be_ok().value("That is some good Mallory."));
//! }
//! ```
//!
//! ## Using Pact plugins
//!
//! The consumer test builders support using Pact plugins. Plugins are defined in the [Pact plugins project](https://github.com/pact-foundation/pact-plugins).
//! To use plugins requires the use of Pact specification V4 Pacts.
//!
//! To use a plugin, first you need to let the builder know to load the plugin and then configure the interaction based on
//! the requirements for the plugin. Each plugin may have different requirements, so you will have to consult the plugin
//! docs on what is required. The plugins will be loaded from the plugin directory. By default, this is `~/.pact/plugins` or
//! the value of the `PACT_PLUGIN_DIR` environment variable.
//!
//! There are generic functions that take JSON data structures and pass these on to the plugin to
//! setup the interaction. For request/response HTTP interactions, there is the `contents` function on the request and
//! response builders. For message interactions, the function is called `contents_from`.
//!
//! For example, if we use the CSV plugin from the plugins project, our test would look like:
//!
//! ```no_run
//! use expectest::prelude::*;
//! use regex::Regex;
//! use pact_consumer::prelude::*;
//! #[tokio::test]
//! async fn test_csv_client() {
//!     // Create a new V4 Pact
//!     let csv_service = PactBuilder::new_v4("CsvClient", "CsvServer")
//!     // Tell the builder we are using the CSV plugin
//!     .using_plugin("csv", None).await
//!     // Add the interaction for the CSV request
//!     .interaction("request for a CSV report", "core/interaction/http", |mut i| async move {
//!         // Path to the request we are going to make
//!         i.request.path("/reports/report001.csv");
//!         // Response we expect back
//!         i.response
//!           .ok()
//!           // We use the generic "contents" function to send the expected response data to the plugin in JSON format
//!           .contents(ContentType::from("text/csv"), json!({
//!             "csvHeaders": false,
//!             "column:1": "matching(type,'Name')",
//!             "column:2": "matching(number,100)",
//!             "column:3": "matching(datetime, 'yyyy-MM-dd','2000-01-01')"
//!           })).await;
//!         i.clone()
//!     })
//!     .await
//!     // Now start the mock server
//!     .start_mock_server_async(None, None)
//!     .await;
//!
//!     // Now we can make our actual request for the CSV file and validate the response
//!     let client = CsvClient::new(csv_service.url().clone());
//!     let data = client.fetch("report001.csv").await.unwrap();
//!
//!     let columns: Vec<&str> = data.trim().split(",").collect();
//!     expect!(columns.get(0)).to(be_some().value(&"Name"));
//!     expect!(columns.get(1)).to(be_some().value(&"100"));
//!     let date = columns.get(2).unwrap();
//!     let re = Regex::new("\\d{4}-\\d{2}-\\d{2}").unwrap();
//!     expect!(re.is_match(date)).to(be_true());
//! }
//! ```
//!
//! ## More Info
//!
//! For more advice on writing good pacts, see [Best Practices][].
//!
//! [Best Practices]: https://docs.pact.io/best_practices/consumer.html
#![warn(missing_docs)]
#[doc = include_str!("../README.md")]

// Child modules which define macros (must be first because macros are resolved)
// in source inclusion order).
#[macro_use]
pub mod patterns;
#[cfg(test)]
#[macro_use]
mod test_support;

// Other child modules.
pub mod builders;
pub mod mock_server;
pub mod util;

/// A "prelude" or a default list of import types to include. This includes
/// the basic DSL, but it avoids including rarely-used types.
///
/// ```
/// use pact_consumer::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        like,
        each_like,
        each_like_helper,
        term,
        json_pattern,
        json_pattern_internal
    };
    pub use crate::builders::{HttpPartBuilder, PactBuilder, PactBuilderAsync};
    #[cfg(feature = "plugins")] pub use crate::builders::plugin_builder::PluginInteractionBuilder;
    pub use crate::mock_server::{StartMockServer, ValidatingMockServer};
    pub use crate::patterns::{
        EachLike,
        Like,
        Term,
        ObjectMatching,
        EachKey,
        EachValue,
        JsonPattern,
        Pattern,
        StringPattern,
        each_key,
        each_value
    };
    #[cfg(feature = "datetime")] pub use crate::patterns::{DateTime};
    pub use crate::util::strip_null_fields;
}

/// Consumer version
pub const PACT_CONSUMER_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
