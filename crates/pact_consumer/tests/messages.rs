use anyhow::anyhow;
use bytes::Bytes;
use expectest::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use pact_consumer::{json_pattern, json_pattern_internal, like};
use pact_consumer::prelude::*;

// Example message handler
struct MessageHandler {
  state_re: Regex
}

// Example processed message
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ProcessedMessage {
  pub name: String,
  pub street: String,
  pub state: String
}

impl MessageHandler {
  pub fn new() -> MessageHandler {
     MessageHandler {
       state_re: Regex::new("\\w+").unwrap()
     }
  }

  pub fn process(&self, data: Bytes) -> anyhow::Result<ProcessedMessage> {
    match serde_json::from_slice::<ProcessedMessage>(&data) {
      Ok(json) => if self.state_re.is_match(json.state.as_str()) {
        Ok(json)
      } else {
        Err(anyhow!("Failed to parse message: state is not valid"))
      }
      Err(err) => Err(anyhow!("Failed to parse message: {}", err))
    }
  }

  pub fn process_request(&self, request: Bytes) -> anyhow::Result<ProcessedMessage> {
    match serde_json::from_slice::<ProcessedMessage>(&request) {
      Ok(json) => if json.name == "mai" {
        Ok(ProcessedMessage {
          name: "mai".to_string(),
          street: "5th".to_string(),
          state: "VA".to_string()
        })
      } else {
        Err(anyhow!("Failed to parse message: name was not found"))
      }
      Err(err) => Err(anyhow!("Failed to parse message: {}", err))
    }
  }
}

// This is a test for async messages. We test that our message consumer can handle the messages
// configured by the builder
#[test_log::test]
fn test_message_client() {
  // Create out builder based on the consumer and provider
  let mut pact_builder = PactBuilder::new_v4("message-consumer", "message-provider");

  // Create a message interaction
  pact_builder.message_interaction("hello message", |mut i| {
      i.test_name("test_message_client");
      i.json_body(json_pattern!({
          "name": like!("mai"),
          "street": like!("5th"),
          "state": like!("VA"),
      }));
      i
    });

  // This will return each message configured with the Pact builder. We need to process them
  // with out message handler (it should be the one used to actually process your messages).
  let handler = MessageHandler::new();
  for message in pact_builder.messages() {
    let bytes = message.contents.contents.value().unwrap();
    let processed = handler.process(bytes).unwrap();

    expect!(processed.name).to(be_equal_to("mai"));
    expect!(processed.street).to(be_equal_to("5th"));
    expect!(processed.state).to(be_equal_to("VA"));
  }
}

// This is a test for sync messages. We test that our message consumer can handle the message request
// configured by the builder and returns a valid response
#[test_log::test]
fn test_req_res_message_client() {
  // Create out builder based on the consumer and provider
  let mut pact_builder = PactBuilder::new_v4("message-consumer", "message-provider");

  // Create a message interaction
  pact_builder.synchronous_message_interaction("hello message", |mut i| {
    i.test_name("test_req_res_message_client");
    // Define the request message
    i.request_json_body(json_pattern!({
      "name": like!("mai")
    }));
    // Add a response message
    i.response_json_body(json_pattern!({
      "street": like!("5th"),
      "state": like!("VA")
    }));
    i
  });

  // This will return each message configured with the Pact builder. We need to process them
  // with out message handler (it should be the one used to actually process your messages).
  let handler = MessageHandler::new();
  for message in pact_builder.synchronous_messages() {
    let request_bytes = message.request.contents.value().unwrap();
    let processed = handler.process_request(request_bytes).unwrap();

    expect!(processed.name).to(be_equal_to("mai"));
    expect!(processed.street).to(be_equal_to("5th"));
    expect!(processed.state).to(be_equal_to("VA"));
  }
}
