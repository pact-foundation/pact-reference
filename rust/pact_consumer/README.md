# Pact test DSL for writing consumer pact tests in Rust

This library provides a test DSL for writing consumer pact tests in Rust. It supports the
[V3 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3) and
[V4 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).

[Online rust docs](https://docs.rs/pact_consumer/)

## To use it

To use it, add it to your dev-dependencies in your cargo manifest:

```toml
[dev-dependencies]
pact_consumer = "~1.4.0"
```

You can now write a pact test using the consumer DSL.

```rust
use pact_consumer::prelude::*;

#[tokio::test]
async fn a_service_consumer_side_of_a_pact_goes_a_little_something_like_this() {
   let alice_service = PactBuilder::new("Consumer", "Alice Service")
       // Start a new interaction. We can add as many interactions as we want.
       .interaction("a retrieve Mallory request", "", |mut i| {
           // Defines a provider state. It is optional.
           i.given("there is some good mallory");
           // Define the request, a GET (default) request to '/mallory'.
           i.request.path("/mallory");
           // Define the response we want returned. We assume a 200 OK
           // response by default.
           i.response
               .content_type("text/plain")
               .body("That is some good Mallory.");
           // Return the interaction builder back to the pact framework
           i
       }).start_mock_server(None);
  
   // You would use your actual client code here.
   let mallory_url = alice_service.path("/mallory");
   let mut response = reqwest::get(mallory_url).await.expect("could not fetch URL")
     .text().await.expect("Could not read response body");
   assert_eq!(response, "That is some good Mallory.");
  
   // When `alice_service` goes out of scope, your pact will be validated,
   // and the test will fail if the mock server didn't receive matching
   // requests.
}
```

### Changing the output directory

By default, the pact files will be written to `target/pacts`. To change this, set the environment variable `PACT_OUTPUT_DIR`.

### Forcing pact files to be overwritten

Pacts are merged with existing pact files when written. To change this behaviour so that the files
are always overwritten, set the environment variable `PACT_OVERWRITE` to `true`.

## Testing messages

Testing message consumers is supported. There are two types: asynchronous messages and synchronous request/response.

### Asynchronous messages

Asynchronous messages are you normal type of single shot or fire and forget type messages. They are typically sent to a
message queue or topic as a notification or event. With Pact tests, we will be testing that our consumer of the messages
works with the messages setup as the expectations in test. This should be the message handler code that processes the
actual messages that come off the message queue in production.

The generated Pact file from the test run can then be used to verify whatever created the messages adheres to the Pact
file.

```rust
use pact_consumer::prelude::*;

#[test]
fn a_message_consumer_side_of_a_pact_goes_a_little_something_like_this() {

    // Define the Pact for the test (you can setup multiple interactions by chaining the given or message_interaction calls)
    // For messages we need to use the V4 Pact format.
    let pact_builder = PactBuilder::PactBuilder::new_v4("message-consumer", "message-provider"); // Define the message consumer and provider by name
    pact_builder
      // defines a provider state. It is optional.
      .given("there is some good mallory".to_string())                                           
      // Adds an interaction given the message description and type.
      .message_interaction("Mallory Message", "core/interaction/message", |mut i| { 
        // Can set the test name (optional)
        i.test_name("a_message_consumer_side_of_a_pact_goes_a_little_something_like_this");
        // Set the contents of the message. Here we use a JSON pattern, so that matching rules are applied
        i.json_body(json_pattern!({
          "mallory": like!("That is some good Mallory.")
        }));
        // Need to return the mutated interaction builder
        i
      });

    // This will return each message configured with the Pact builder. We need to process them
    // with out message handler (it should be the one used to actually process your messages).
    for message in pact_builder.messages() {
      let bytes = message.contents.contents.value().unwrap();
      
      // Process the message here as it would if it came off the queue
      let message: Value = serde_json::from_slice(&bytes);      

      // Make some assertions on the processed value
      expect!(message.as_object().unwrap().get("mallory")).to(be_some().value());
    }
}
```

### Synchronous request/response messages

Synchronous request/response messages are a form of message interchange were a request message is sent to another service and
one or more response messages are returned. Examples of this would be things like Websockets and gRPC. 

```rust
use pact_consumer::prelude::*;
use expectest::prelude::*;
use serde_json::{Value, from_slice};

#[test]
fn a_synchronous_message_consumer_side_of_a_pact_goes_a_little_something_like_this() {

  // Define the Pact for the test (you can setup multiple interactions by chaining the given or message_interaction calls)
  // For synchronous messages we also need to use the V4 Pact format.
  let mut pact_builder = PactBuilder::new_v4("message-consumer", "message-provider"); // Define the message consumer and provider by name
  pact_builder
    // Adds an interaction given the message description and type.
    .synchronous_message_interaction("Mallory Message", "core/interaction/synchronous-message", |mut i| {
      // defines a provider state. It is optional.
      i.given("there is some good mallory".to_string());
      // Can set the test name (optional)
      i.test_name("a_synchronous_message_consumer_side_of_a_pact_goes_a_little_something_like_this");
      // Set the contents of the request message. Here we use a JSON pattern, so that matching rules are applied.
      // This is the request message that is going to be forwarded to the provider
      i.request_json_body(json_pattern!({
          "requestFor": like!("Some good Mallory, please.")
        }));
      // Add a response message we expect the provider to return. You can call this multiple times to add multiple messages.
      i.response_json_body(json_pattern!({
          "mallory": like!("That is some good Mallory.")
        }));
      // Need to return the mutated interaction builder
      i
    });

  // For our test we want to invoke our message handling code that is going to initialise the request
  // to the provider with the request message. But we need some mechanism to mock the response
  // with the resulting response message so we can confirm our message handler works with it.
  for message in pact_builder.synchronous_messages() {
    // the request message we must make
    let request_message_bytes = message.request.contents.value().unwrap();
    // the response message we expect to receive from the provider
    let response_message_bytes = message.response.first().unwrap().contents.value().unwrap();

    // We use a mock here, assuming there is a Trait that controls the response message that our
    // mock can implement.
    let mock_provider = MockProvider { message: response_message_bytes };
    // Invoke our message handler to send the request message from the Pact interaction and then
    // wait for the response message. In this case it will be the response via the mock provider.
    let response = MessageHandler::process(request_message_bytes, &mock_provider);

    // Make some assertions on the processed value
    expect!(response).to(be_ok().value("That is some good Mallory."));
  }
}
```

## Using Pact plugins

The consumer test builders support using Pact plugins. Plugins are defined in the [Pact plugins project](https://github.com/pact-foundation/pact-plugins).
To use plugins requires the use of Pact specification V4 Pacts.

To use a plugin, first you need to let the builder know to load the plugin and then configure the interaction based on
the requirements for the plugin. Each plugin may have different requirements, so you will have to consult the plugin
docs on what is required. The plugins will be loaded from the plugin directory. By default, this is `~/.pact/plugins` or 
the value of the `PACT_PLUGIN_DIR` environment variable. 

There are generic functions that take JSON data structures and pass these on to the plugin to
setup the interaction. For request/response HTTP interactions, there is the `contents` function on the request and 
response builders. For message interactions, the function is called `contents_from`.

For example, if we use the CSV plugin from the plugins project, our test would look like:

```rust
#[tokio::test]
async fn test_csv_client() {
    // Create a new V4 Pact 
    let csv_service = PactBuilder::new_v4("CsvClient", "CsvServer")
    // Tell the builder we are using the CSV plugin  
    .using_plugin("csv", None).await
    // Add the interaction for the CSV request  
    .interaction("request for a CSV report", "core/interaction/http", |mut i| async move {
        // Path to the request we are going to make
        i.request.path("/reports/report001.csv");
        // Response we expect back
        i.response
          .ok()
          // We use the generic "contents" function to send the expected response data to the plugin in JSON format 
          .contents(ContentType::from("text/csv"), json!({
            "csvHeaders": false,
            "column:1": "matching(type,'Name')",
            "column:2": "matching(number,100)",
            "column:3": "matching(datetime, 'yyyy-MM-dd','2000-01-01')"
          })).await;
        i.clone()
    })
    .await
    // Now start the mock server  
    .start_mock_server_async()
    .await;
    
    // Now we can make our actual request for the CSV file and validate the response
    let client = CsvClient::new(csv_service.url().clone());
    let data = client.fetch("report001.csv").await.unwrap();
    
    let columns: Vec<&str> = data.trim().split(",").collect();
    expect!(columns.get(0)).to(be_some().value(&"Name"));
    expect!(columns.get(1)).to(be_some().value(&"100"));
    let date = columns.get(2).unwrap();
    let re = Regex::new("\\d{4}-\\d{2}-\\d{2}").unwrap();
    expect!(re.is_match(date)).to(be_true());
}
```
