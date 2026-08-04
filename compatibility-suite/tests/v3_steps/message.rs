use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::panic::catch_unwind;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use bytes::Bytes;
use cucumber::{given, then, when, World};
use cucumber::gherkin::Step;
use itertools::Itertools;
use lazy_static::lazy_static;
use maplit::hashmap;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, JSON, XML};
use pact_models::generators::Generators;
use pact_models::matchingrules::matchers_from_json;
use pact_models::message::Message;
use pact_models::message_pact::MessagePact;
use pact_models::pact::{Pact, read_pact};
use pact_models::path_exp::DocPath;
use pact_models::provider_states::ProviderState;
use pact_models::xml_utils::parse_bytes;
use serde_json::{json, Value};

use pact_consumer::builders::{MessageInteractionBuilder, PactBuilder};
use pact_matching::Mismatch;
use pact_verifier::{FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, ProviderTransport, VerificationOptions, verify_provider_async};
use pact_verifier::verification_result::{VerificationExecutionResult, VerificationMismatchResult};

use crate::shared_steps::{assert_value_type, determine_content_type, element_text, IndexType};
use crate::shared_steps::provider::MockProviderStateExecutor;

lazy_static!{
  pub static ref MESSAGES: Arc<Mutex<HashMap<String, Message>>> = Arc::new(Mutex::new(hashmap![]));
}

#[derive(Debug, World)]
pub struct V3MessageWorld {
  pub scenario_id: String,
  pub builder: PactBuilder,
  pub message_builder: MessageInteractionBuilder,
  pub received_messages: Vec<Message>,
  pub failed: Option<String>,
  pub loaded_pact: MessagePact,
  pub message_proxy_port: u16,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub provider_state_executor: Arc<MockProviderStateExecutor>,
  pub verification_results: VerificationExecutionResult
}

impl Default for V3MessageWorld {
  fn default() -> Self {
    V3MessageWorld {
      scenario_id: "".to_string(),
      builder: PactBuilder::new_v3_message("V3-message-consumer", "V3-message-provider"),
      message_builder: MessageInteractionBuilder::new(""),
      received_messages: vec![],
      failed: None,
      loaded_pact: MessagePact::default(),
      message_proxy_port: 0,
      provider_info: Default::default(),
      sources: vec![],
      provider_state_executor: Arc::new(Default::default()),
      verification_results: VerificationExecutionResult::new(),
    }
  }
}

#[given("a message integration is being defined for a consumer test")]
fn a_message_integration_is_being_defined_for_a_consumer_test(world: &mut V3MessageWorld) {
  let dir = PathBuf::from("target/compatibility-suite/v3").join(&world.scenario_id);
  world.builder.with_output_dir(dir);
  world.message_builder = MessageInteractionBuilder::new("a message");
}

#[given(expr = "the message payload contains the {string} JSON document")]
fn the_message_payload_contains_the_json_document(
  world: &mut V3MessageWorld,
  fixture: String
) -> anyhow::Result<()> {
  let mut fixture = File::open(format!("pact-compatibility-suite/fixtures/{}.json", fixture))?;
  let mut buffer = Vec::new();
  fixture.read_to_end(&mut buffer)?;
  world.message_builder.body(buffer, Some("application/json".into()));
  Ok(())
}

#[given("a message is defined")]
fn a_message_is_defined(world: &mut V3MessageWorld) {
  let previous_builder = world.message_builder.clone();
  world.message_builder = MessageInteractionBuilder::new("a message");
  for state in previous_builder.build().provider_states {
    if state.params.is_empty() {
      world.message_builder.given(state.name);
    } else {
      world.message_builder.given_with_params(state.name, &Value::Object(state.params
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
      ));
    }
  }
}

#[given("the message is configured with the following:")]
fn the_message_configured_with_the_following(world: &mut V3MessageWorld, step: &Step) {
  world.message_builder = MessageInteractionBuilder::new("a message");

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => {
            let mut message = Message::default();
            setup_body(value, &mut message);
            world.message_builder.message_contents.body = message.contents;
            let md = world.message_builder.message_contents.metadata
              .get_or_insert_with(|| hashmap!{});
            md.extend(message.metadata.iter().map(|(k, v)| (k.clone(), v.clone())));
          },
          "generators" => {
            let json: Value = if value.starts_with("JSON:") {
              serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
            } else {
              let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
                .expect(format!("could not load fixture '{}'", value).as_str());
              let reader = BufReader::new(f);
              serde_json::from_reader(reader).unwrap()
            };
            let mut generators = Generators::default();
            generators.load_from_map(json.as_object().unwrap()).unwrap();
            world.message_builder.message_contents.generators = Some(generators);
          }
          "metadata" => {
            let json: Value = serde_json::from_str(value).unwrap();
            let md = world.message_builder.message_contents.metadata
              .get_or_insert_with(|| hashmap!{});
            md.extend(json.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())))
          }
          _ => {}
        }
      }
    }
  }
}

#[given("the message contains the following metadata:")]
fn the_message_contains_the_following_metadata(world: &mut V3MessageWorld, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (h.clone(), index))
      .collect::<HashMap<String, usize>>();
    for values in table.rows.iter().skip(1) {
      let key = values.get(*headers.get("key").unwrap()).unwrap();
      let value = values.get(*headers.get("value").unwrap()).unwrap();
      let json: Value = if value.starts_with("JSON:") {
        serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
      } else {
        Value::String(value.clone())
      };
      world.message_builder.metadata(key, json);
    }
  }
}

#[given(expr = "a provider state {string} for the message is specified")]
fn a_provider_state_for_the_message_is_specified(world: &mut V3MessageWorld, state: String) {
  world.message_builder.given(state);
}

#[given(expr = "a provider state {string} for the message is specified with the following data:")]
fn a_provider_state_for_the_message_is_specified_with_the_following_data(
  world: &mut V3MessageWorld,
  step: &Step,
  state: String) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    let params = table.rows.get(1).unwrap().iter().enumerate().map(|(i, v)| {
      let key = headers.get(&i).unwrap();
      let json: Value = serde_json::from_str(v).unwrap();
      (key.clone(), json)
    }).collect();
    world.message_builder.given_with_params(state, &Value::Object(params));
  }
}

#[when("the message is successfully processed")]
fn the_message_is_successfully_processed(world: &mut V3MessageWorld) {
  world.builder.push_interaction(&world.message_builder.build_v3());
  world.received_messages = world.builder.v3_messages()
    .collect();
  world.failed = None;
}

#[then("the consumer test will have passed")]
fn consumer_test_will_have_passed(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  match &world.failed {
    None => Ok(()),
    Some(err) => Err(anyhow!(err.clone()))
  }
}

#[then(expr = "the received message payload will contain the {string} JSON document")]
fn the_received_message_payload_will_contain_the_json_document(
  world: &mut V3MessageWorld,
  fixture: String
) -> anyhow::Result<()> {
  let mut fixture = File::open(format!("pact-compatibility-suite/fixtures/{}.json", fixture))?;
  let mut buffer = Vec::new();
  fixture.read_to_end(&mut buffer)?;
  let message = world.received_messages.first().unwrap();
  if message.contents.value().unwrap() == buffer.as_slice() {
    Ok(())
  } else {
    let body = OptionalBody::Present(Bytes::from(buffer), None, None);
    Err(anyhow!("Expected payload with {} but got {}", message.contents.display_string(),
      body.display_string()))
  }
}

#[then(expr = "the received message content type will be {string}")]
fn the_received_message_content_type_will_be(
  world: &mut V3MessageWorld,
  content_type: String
) -> anyhow::Result<()> {
  let message = world.received_messages.first().unwrap();
  let ct = message.message_content_type().unwrap();
  if ct.to_string() == content_type {
    Ok(())
  } else {
    Err(anyhow!("Expected message with content type {} but got {:?}", content_type, ct))
  }
}

#[then("a Pact file for the message interaction will have been written")]
fn a_pact_file_for_the_message_interaction_will_have_been_written(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  let dir = PathBuf::from("target/compatibility-suite/v3").join(&world.scenario_id);
  let pact_file = dir.join("V3-message-consumer-V3-message-provider.json");
  if pact_file.exists() {
    let pact = read_pact(&pact_file)?;
    if pact.specification_version() == PactSpecification::V3 {
      world.loaded_pact = pact.as_message_pact()?;
      Ok(())
    } else {
      Err(anyhow!("Expected Pact file to be V3 Pact, but was {}", pact.specification_version()))
    }
  } else {
    Err(anyhow!("No pact file found: {}", pact_file.to_string_lossy()))
  }
}

#[then(expr = "the pact file will contain {int} message interaction(s)")]
fn the_pact_file_will_contain_message_interaction(
  world: &mut V3MessageWorld,
  messages: usize
) -> anyhow::Result<()> {
  let actual = world.loaded_pact.messages.len();
  if actual == messages {
    Ok(())
  } else {
    Err(anyhow!("Expected {} messages in the Pact, but there were {}", messages, actual))
  }
}

#[then(expr = "the {numType} message in the pact file will contain the {string} document")]
fn the_first_message_in_the_pact_file_will_contain_the_document(
  world: &mut V3MessageWorld,
  index: IndexType,
  fixture: String
) -> anyhow::Result<()> {
  let message = world.loaded_pact.messages.get(index.val()).unwrap();

  let mut fixture_file = File::open(format!("pact-compatibility-suite/fixtures/{}", fixture))?;
  let mut buffer = Vec::new();
  fixture_file.read_to_end(&mut buffer)?;

  let mut expected = Vec::new();
  if fixture.ends_with(".json") {
    let json: Value = serde_json::from_slice(&buffer)?;
    let string = json.to_string();
    expected.extend_from_slice(string.as_bytes());
  } else {
    expected.extend_from_slice(&buffer);
  }

  let actual_body = message.contents.value().unwrap_or_default();
  if &actual_body == expected.as_slice() {
    Ok(())
  } else {
    let body = OptionalBody::Present(Bytes::from(buffer), None, None);
    Err(anyhow!("Expected Interaction {} message payload with {} but got {}", index.val() + 1,
      message.contents.display_string(), body.display_string()))
  }
}

#[then(expr = "the {numType} message in the pact file content type will be {string}")]
fn the_first_message_in_the_pact_file_content_type_will_be(
  world: &mut V3MessageWorld,
  index: IndexType,
  content_type: String
) -> anyhow::Result<()> {
  let message = world.loaded_pact.messages.get(index.val()).unwrap();
  if let Some(ct) = message.message_content_type() {
    if ct.to_string() == content_type {
      Ok(())
    } else {
      Err(anyhow!("Message {} content type {}, but expected {}", index.val() + 1, ct, content_type))
    }
  } else {
    Err(anyhow!("Message has no content type set"))
  }
}

#[when(expr = "the message is NOT successfully processed with a {string} exception")]
fn the_message_is_not_successfully_processed_with_a_exception(
  world: &mut V3MessageWorld,
  error: String
) {
  world.builder.push_interaction(&world.message_builder.build());
  let result = catch_unwind(|| {
    let _messages = world.builder.v3_messages();
    // This panic will cause the message iterator to not write out the Pact file when dropped
    panic!("{}", error);
  });
  world.failed = result.err().map(|err| {
    if let Some(err) = err.downcast_ref::<&str>() {
      err.to_string()
    } else if let Some(err) = err.downcast_ref::<String>() {
      err.clone()
    } else {
      format!("Unknown error: {:?}", err)
    }
  });
}

#[then("the consumer test will have failed")]
fn the_consumer_test_will_have_failed(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if world.failed.is_some() {
    Ok(())
  } else {
    Err(anyhow!("Expected test to fail. It did not. Very rude."))
  }
}

#[then(expr = "the consumer test error will be {string}")]
fn the_consumer_test_error_will_be_blah(
  world: &mut V3MessageWorld,
  error: String
) -> anyhow::Result<()> {
  if let Some(err) = &world.failed {
    if *err == error {
      Ok(())
    } else {
      Err(anyhow!("Expected test to fail with error '{}' but the error was '{}'", error, err))
    }
  } else {
    Err(anyhow!("Expected test to fail with error '{}'. It did not. Very rude.", error))
  }
}

#[then("a Pact file for the message interaction will NOT have been written")]
fn a_pact_file_for_the_message_interaction_will_not_have_been_written(
  world: &mut V3MessageWorld
) -> anyhow::Result<()> {
  let dir = PathBuf::from("target/compatibility-suite/v3").join(&world.scenario_id);
  let pact_file = dir.join("V3-message-consumer-V3-message-provider.json");
  if pact_file.exists() {
    Err(anyhow!("Expected no pact file, but found: {}", pact_file.to_string_lossy()))
  } else {
    Ok(())
  }
}

#[then(expr = "the received message metadata will contain {string} == {string}")]
fn the_received_message_metadata_will_contain(
  world: &mut V3MessageWorld,
  key: String,
  value: String
) -> anyhow::Result<()> {
  let json: Value = if value.starts_with("JSON:") {
    let value_str = value.strip_prefix("JSON:")
      .unwrap_or(value.as_str())
      .trim()
      .replace("\\\"", "\"");
    serde_json::from_str(value_str.as_str()).unwrap()
  } else {
    Value::String(value.clone())
  };
  if let Some(md_value) = world.received_messages.first().unwrap().metadata.get(&key) {
    if *md_value == json {
      Ok(())
    } else {
      Err(anyhow!("Expected message metadata with key {} == {} but was {}", key, json, md_value))
    }
  } else {
    Err(anyhow!("Received message did not have a metadata value with key {}", key))
  }
}

#[then(expr = "the {numType} message in the pact file will contain the message metadata {string} == {string}")]
fn the_first_message_in_the_pact_file_will_contain_the_message_metadata(
  world: &mut V3MessageWorld,
  index: IndexType,
  key: String,
  value: String
) -> anyhow::Result<()> {
  let message = world.loaded_pact.messages.get(index.val()).unwrap();
  let json: Value = if value.starts_with("JSON:") {
    let value_str = value.strip_prefix("JSON:")
      .unwrap_or(value.as_str())
      .trim()
      .replace("\\\"", "\"");
    serde_json::from_str(value_str.as_str()).unwrap()
  } else {
    Value::String(value.clone())
  };
  if let Some(md_value) = message.metadata.get(&key) {
    if *md_value == json {
      Ok(())
    } else {
      Err(anyhow!("Expected message metadata with key {} == {} but was {}", key, json, md_value))
    }
  } else {
    Err(anyhow!("Received message did not have a metadata value with key {}", key))
  }
}

#[then(expr = "the {numType} message in the pact file will contain {int} provider state(s)")]
fn the_first_message_in_the_pact_file_will_contain_provider_states(
  world: &mut V3MessageWorld,
  index: IndexType,
  states: usize
) -> anyhow::Result<()> {
  let message = world.loaded_pact.messages.get(index.val()).unwrap();
  let actual = message.provider_states.len();
  if actual == states {
    Ok(())
  } else {
    Err(anyhow!("Expected message to have {} provider states, but it has {}", states, actual))
  }
}

#[then(expr = "the {numType} message in the Pact file will contain provider state {string}")]
fn the_first_message_in_the_pact_file_will_contain_provider_state(
  world: &mut V3MessageWorld,
  index: IndexType,
  state_name: String
) -> anyhow::Result<()> {
  let message = world.loaded_pact.messages.get(index.val()).unwrap();
  if message.provider_states.iter().any(|ps| ps.name == state_name) {
    Ok(())
  } else {
    Err(anyhow!("Did not find a provider state '{}'", state_name))
  }
}

#[then(expr = "the provider state {string} for the message will contain the following parameters:")]
fn the_provider_state_for_the_message_will_contain_the_following_parameters(
  world: &mut V3MessageWorld,
  step: &Step,
  state: String
) -> anyhow::Result<()> {
  let table = step.table.as_ref().unwrap();
  let params_str = table.rows.get(1).unwrap().first().unwrap();
  let params = serde_json::from_str::<Value>(params_str.as_str())
    .unwrap()
    .as_object()
    .unwrap()
    .iter()
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
  let message = world.loaded_pact.messages.first().unwrap();
  let provider_state = message.provider_states.iter().find(|ps| ps.name == state).unwrap();
  if provider_state.params == params {
    Ok(())
  } else {
    Err(anyhow!("Expected provider state '{}' to have parameters {:?} but were {:?}", state,
      params, provider_state.params))
  }
}

#[then(expr = "the message contents for {string} will have been replaced with a(n) {string}")]
fn the_message_contents_for_will_have_been_replaced_with_an(
  world: &mut V3MessageWorld,
  path: String,
  value_type: String
) -> anyhow::Result<()> {
  let message_pact = world.builder.build().as_message_pact().unwrap();
  let message = message_pact.messages.first().unwrap();
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(message.contents.value_as_string().unwrap().as_str()).unwrap();
  let pointer = path.as_json_pointer().unwrap();
  let pointer = pointer.as_str();
  let original_element = original_json.pointer(pointer).unwrap();
  let json: Value = serde_json::from_str(world.received_messages.first().unwrap().contents.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(pointer).unwrap();

  if element == original_element {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  }

  assert_value_type(value_type, element)
}

#[then(expr = "the received message metadata will contain {string} replaced with a(n) {string}")]
fn the_received_message_metadata_will_contain_replaced_with_an(
  world: &mut V3MessageWorld,
  key: String,
  value_type: String
) -> anyhow::Result<()> {
  let message_pact = world.builder.build().as_message_pact().unwrap();
  let message = message_pact.messages.first().unwrap();
  let original = message.metadata.get(&key).unwrap();
  let generated = world.received_messages.first().unwrap().metadata.get(&key).unwrap();

  if generated == original {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original))
  }

  assert_value_type(value_type, generated)
}

// TODO: Message in pact-models needs to implement add_header correctly, then this can be replaced
// with the version from shared steps.
pub fn setup_body(body: &String, httppart: &mut Message) {
  if !body.is_empty() {
    if body.starts_with("JSON:") {
      httppart.metadata.insert("contentType".to_string(), json!("application/json"));
      httppart.contents = OptionalBody::Present(Bytes::from(body.strip_prefix("JSON:").unwrap_or(body).trim().to_string()),
        Some(JSON.clone()), None);
    } else if body.starts_with("XML:") {
      httppart.metadata.insert("contentType".to_string(), json!("application/xml"));
      httppart.contents = OptionalBody::Present(Bytes::from(body.strip_prefix("XML:").unwrap_or(body).trim().to_string()),
        Some(XML.clone()), None);
    } else if body.starts_with("file:") {
      if body.ends_with("-body.xml") {
        let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
        let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
          .expect(format!("could not load fixture '{}'", body).as_str());
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
          .expect(format!("could not read fixture '{}'", body).as_str());
        let fixture = parse_bytes(buffer.as_slice())
          .expect(format!("could not parse fixture as XML: '{}'", body).as_str());
        let root = fixture.as_document().root();
        let body_node = root.children().iter().find_map(|n| n.element()).unwrap();
        let content_type = element_text(body_node, "contentType").unwrap_or("text/plain".to_string());
        httppart.metadata.insert("contentType".to_string(), json!(content_type));
        httppart.contents = OptionalBody::Present(Bytes::from(element_text(body_node, "contents").unwrap_or_default()),
          ContentType::parse(content_type.as_str()).ok(), None);
      } else {
        let content_type = determine_content_type(body, httppart);
        httppart.metadata.insert("contentType".to_string(), json!(content_type.to_string()));

        let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
        let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
          .expect(format!("could not load fixture '{}'", body).as_str());
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
          .expect(format!("could not read fixture '{}'", body).as_str());
        httppart.contents = OptionalBody::Present(Bytes::from(buffer),
          Some(content_type), None);
      }
    } else {
      let content_type = determine_content_type(body, httppart);
      httppart.metadata.insert("contentType".to_string(), json!(content_type.to_string()));
      let body = Bytes::from(body.clone());
      httppart.contents = OptionalBody::Present(body, Some(content_type), None);
    }
  }
}

// ----------------------------------------------------------------------
// Provider steps
// ----------------------------------------------------------------------

#[given(expr = "a provider is started that can generate the {string} message with {string}")]
#[allow(deprecated)]
fn a_provider_is_started_that_can_generate_the_message(
  world: &mut V3MessageWorld,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    .. Message::default()
  };
  setup_body(&fixture, &mut message);

  {
    let mut guard = MESSAGES.lock().unwrap();
    guard.insert(key, message);
  }

  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "localhost".to_string(),
    port: Some(world.message_proxy_port),
    transports: vec![ProviderTransport {
      port: Some(world.message_proxy_port),
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };
}

#[given(expr = "a Pact file for {string}:{string} is to be verified")]
fn a_pact_file_for_is_to_be_verified(
  world: &mut V3MessageWorld,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    .. Message::default()
  };
  setup_body(&fixture, &mut message);

  let pact = MessagePact {
    consumer: Consumer { name: "c".to_string() },
    provider: Provider { name: "p".to_string() },
    messages: vec![ message ],
    specification_version: PactSpecification::V3,
    .. MessagePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V3).unwrap().to_string()));
}

#[given(expr = "a Pact file for {string}:{string} is to be verified with provider state {string}")]
fn a_pact_file_for_is_to_be_verified_with_provider_state(
  world: &mut V3MessageWorld,
  name: String,
  fixture: String,
  state: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    provider_states: vec![ ProviderState::default(state) ],
    .. Message::default()
  };
  setup_body(&fixture, &mut message);

  let pact = MessagePact {
    consumer: Consumer { name: "c".to_string() },
    provider: Provider { name: "p".to_string() },
    messages: vec![ message ],
    specification_version: PactSpecification::V3,
    .. MessagePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V3).unwrap().to_string()));
}

#[given(expr = "a provider is started that can generate the {string} message with {string} and the following metadata:")]
#[allow(deprecated)]
fn a_provider_is_started_that_can_generate_the_message_with_the_following_metadata(
  world: &mut V3MessageWorld,
  step: &Step,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    .. Message::default()
  };
  setup_body(&fixture, &mut message);

  if let Some(table) = &step.table {
    for row in table.rows.iter().skip(1) {
      let key = row[0].clone();
      let value = row[1].clone();
      if value.starts_with("JSON:") {
        let json = serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value.as_str()).trim()).unwrap();
        message.metadata.insert(key, json);
      } else {
        message.metadata.insert(key, Value::String(value));
      };
    }
  }

  {
    let mut guard = MESSAGES.lock().unwrap();
    guard.insert(key, message);
  }

  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "localhost".to_string(),
    port: Some(world.message_proxy_port),
    transports: vec![ProviderTransport {
      port: Some(world.message_proxy_port),
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };
}

#[given(expr = "a Pact file for {string}:{string} is to be verified with the following metadata:")]
fn a_pact_file_for_is_to_be_verified_with_the_following_metadata(
  world: &mut V3MessageWorld,
  step: &Step,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    .. Message::default()
  };
  setup_body(&fixture, &mut message);

  if let Some(table) = &step.table {
    for row in &table.rows {
      let key = row[0].clone();
      let value = row[1].clone();
      if value.starts_with("JSON:") {
        let json = serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value.as_str()).trim()).unwrap();
        message.metadata.insert(key, json);
      } else {
        message.metadata.insert(key, Value::String(value));
      };
    }
  }

  let pact = MessagePact {
    consumer: Consumer { name: "c".to_string() },
    provider: Provider { name: "p".to_string() },
    messages: vec![ message ],
    specification_version: PactSpecification::V3,
    .. MessagePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V3).unwrap().to_string()));
}

#[given(expr = "a Pact file for {string} is to be verified with the following:")]
fn a_pact_file_for_is_to_be_verified_with_the_following(
  world: &mut V3MessageWorld,
  step: &Step,
  name: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = Message {
    description: key.clone(),
    .. Message::default()
  };

  if let Some(table) = &step.table {
    for row in &table.rows {
      match row[0].as_str() {
        "body" => {
          setup_body(&row[1], &mut message);
        }
        "matching rules" => {
          let value = dbg!(&row[1]);
          let json: Value = if value.starts_with("JSON:") {
            serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
          } else {
            let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
              .expect(format!("could not load fixture '{}'", value).as_str());
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap()
          };
          message.matching_rules = matchers_from_json(&json!({"matchingRules": json}), &None).unwrap();
        }
        "metadata" => {
          for values in row[1].split(';').map(|v| v.trim().splitn(2, '=').collect_vec()) {
            let key = values[0];
            let value = values[1];
            if value.starts_with("JSON:") {
              let json = serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap();
              message.metadata.insert(key.to_string(), json);
            } else {
              message.metadata.insert(key.to_string(), Value::String(value.to_string()));
            };
          }
        }
        _ => {}
      }
    }
  }

  let pact = MessagePact {
    consumer: Consumer { name: "c".to_string() },
    provider: Provider { name: "p".to_string() },
    messages: vec![ message ],
    specification_version: PactSpecification::V3,
    .. MessagePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V3).unwrap().to_string()));
}

#[given("a provider state callback is configured")]
fn a_provider_state_callback_is_configured(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  world.provider_state_executor.set_fail_mode(false);
  Ok(())
}

#[when("the verification is run")]
async fn the_verification_is_run(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }

  world.verification_results = verify_provider_async(
    world.provider_info.clone(),
    world.sources.clone(),
    FilterInfo::None,
    vec![],
    &VerificationOptions::<NullRequestFilterExecutor>::default(),
    None,
    &world.provider_state_executor,
    None
  ).await?;
  Ok(())
}

#[then("the verification will be successful")]
fn the_verification_will_be_successful(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Ok(())
  } else {
    Err(anyhow!("Verification failed"))
  }
}

#[then("the verification will NOT be successful")]
fn the_verification_will_not_be_successful(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Err(anyhow!("Was expecting the verification to fail"))
  } else {
    Ok(())
  }
}

#[then("the provider state callback will be called before the verification is run")]
fn the_provider_state_callback_will_be_called_before_the_verification_is_run(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called"))
  }
}

#[then("the provider state callback will be called after the verification is run")]
fn the_provider_state_callback_will_be_called_after_the_verification_is_run(world: &mut V3MessageWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback teardown was not called"))
  }
}

#[then(expr = "the provider state callback will receive a setup call with {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_setup_call_with_as_the_provider_state_parameter(
  world: &mut V3MessageWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called for state '{}'", state))
  }
}

#[then(expr = "the provider state callback will receive a teardown call {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_teardown_call_as_the_provider_state_parameter(
  world: &mut V3MessageWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state teardown callback was not called for state '{}'", state))
  }
}

#[then(expr = "the verification results will contain a {string} error")]
fn the_verification_results_will_contain_a_error(world: &mut V3MessageWorld, err: String) -> anyhow::Result<()> {
  if world.verification_results.errors.iter().any(|(_, r)| {
    match r {
      VerificationMismatchResult::Mismatches { mismatches, .. } => {
        mismatches.iter().any(|mismatch| {
          match mismatch {
            Mismatch::MethodMismatch { .. } => false,
            Mismatch::PathMismatch { .. } => false,
            Mismatch::StatusMismatch { .. } => err == "Response status did not match",
            Mismatch::QueryMismatch { .. } => false,
            Mismatch::HeaderMismatch { .. } => err == "Headers had differences",
            Mismatch::BodyTypeMismatch { .. } => false,
            Mismatch::BodyMismatch { .. } => err == "Body had differences",
            Mismatch::MetadataMismatch { .. } => err == "Metadata had differences"
          }
        })
      }
      VerificationMismatchResult::Error { error, .. } => match err.as_str() {
        "State change request failed" => error == "One or more of the setup state change handlers has failed",
        _ => error.as_str() == err
      }
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find error message in verification results"))
  }
}
