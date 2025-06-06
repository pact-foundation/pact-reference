use std::{
  env,
  fs,
  path::Path
};
use std::io::Write;
use std::path::PathBuf;

use bytes::Bytes;
use expectest::prelude::*;
use maplit::hashmap;
use pretty_assertions::assert_eq;
use rand::prelude::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use pact_consumer::{json_pattern, json_pattern_internal, like, matching_regex, object_matching};
use pact_consumer::mock_server::StartMockServerAsync;
use pact_consumer::prelude::*;
use pact_models::content_types::ContentType;
use pact_models::pact::{read_pact, ReadWritePact};
use pact_models::pact::write_pact;
use pact_models::PactSpecification;
use pact_models::prelude::OptionalBody;
use pact_models::provider_states::ProviderState;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::synch_http::SynchronousHttp;

/// This is supposed to be a doctest in mod, but it's breaking there, so
/// we have an executable copy here.
#[test_log::test(tokio::test)]
async fn mock_server_passing_validation() -> anyhow::Result<()> {
    let output_dir = output_dir("target/pact_dir");
    unsafe { env::set_var("PACT_OUTPUT_DIR", &output_dir); }

    // clean out any previous Pact file
    let path_file = Path::new("target/pact_dir/Consumer-Alice Service.json");
    if path_file.exists() {
      fs::remove_file(path_file)?;
    }

    {
      // Define the Pact for the test, specify the names of the consuming
      // application and the provider application.
      let alice_service = PactBuilder::new_v4("Consumer", "Alice Service")
        // Start a new interaction. We can add as many interactions as we want.
        .interaction("a retrieve Mallory request", "", |mut i| {
          // Defines a provider state. It is optional.
          i.given("there is some good mallory");
          // Define the request, a GET (default) request to '/mallory'.
          i.request.path("/mallory");
          i.request.header("content-type", "application/json");
          // Define the response we want returned.
          i.response
            .ok()
            .content_type("text/plain")
            .body("That is some good Mallory.");

          // Return the interaction back to the pact framework
          i.clone()
        })
        .start_mock_server(None, None);

      // You would use your actual client code here.
      let mallory_url = alice_service.path("/mallory");
      let client = reqwest::Client::new();
      let response = client.get(mallory_url)
        .header("content-type", "application/json")
        .send().await
        .expect("could not fetch URL");
      let body = response.text().await.expect("could not read response body");
      assert_eq!(body, "That is some good Mallory.");
    }

    // When your test has finished running, all verifications will be performed
    // automatically, and an error will be thrown if any have failed.

    unsafe { env::remove_var("PACT_OUTPUT_DIR"); }

    expect!(path_file.exists()).to(be_true());
    let pact = read_pact(path_file)?.as_v4_pact()?;
    expect!(pact.interactions.len()).to(be_equal_to(1));
  let expected = SynchronousHttp {
    description: "a retrieve Mallory request".to_string(),
    provider_states: vec![ProviderState::default("there is some good mallory")],
    request: HttpRequest {
      path: "/mallory".to_string(),
      headers: Some(hashmap! { "content-type".to_string() => vec![ "application/json".to_string() ] }),
      ..HttpRequest::default()
    },
    response: HttpResponse {
      headers: Some(hashmap! { "content-type".to_string() => vec![ "text/plain".to_string() ] }),
      body: OptionalBody::Present(Bytes::from("That is some good Mallory."), Some(ContentType::from("*/*")), None),
      ..HttpResponse::default()
    },
    transport: Some("http".to_string()),
    ..SynchronousHttp::default()
  };
  pretty_assertions::assert_eq!(pact.interactions.first().unwrap().as_v4_http().unwrap(),
    expected);

  Ok(())
}

#[test_log::test]
fn mock_server_passing_validation_blocking() -> anyhow::Result<()> {
  let alice_service = PactBuilder::new_v4("BlockingConsumer", "Alice Service")
    .interaction("a retrieve Mallory request", "", |mut i| {
      i.given("there is some good mallory");
      i.request.path("/mallory");
      i.request.header("content-type", "application/json");
      i.response
        .ok()
        .content_type("text/plain")
        .body("That is some good Mallory.");
      i.clone()
    })
    .start_mock_server(None, None);

  let mallory_url = alice_service.path("/mallory");
  let client = reqwest::blocking::Client::new();
  let response = client.get(mallory_url)
    .header("content-type", "application/json")
    .send()
    .expect("could not fetch URL");
  let body = response.text().expect("could not read response body");
  assert_eq!(body, "That is some good Mallory.");

  Ok(())
}

fn output_dir(path: &str) -> PathBuf {
  match Path::new(path).canonicalize() {
    Ok(path) => {
      fs::remove_dir_all(path.clone()).unwrap_or(());
      path
    }
    Err(_) => {
      let path = Path::new(path);
      fs::create_dir_all(path).unwrap_or(());
      path.canonicalize().unwrap()
    }
  }
}

#[test_log::test(tokio::test)]
async fn mock_server_passing_validation_async_version() {
  // Define the Pact for the test, specify the names of the consuming
  // application and the provider application.
  let alice_service = PactBuilderAsync::new("Consumer", "Alice Service Async")
    // Start a new interaction. We can add as many interactions as we want.
    .interaction("a retrieve Mallory request", "", |mut i| async move {
      // Defines a provider state. It is optional.
      i.given("there is some good mallory");
      // Define the request, a GET (default) request to '/mallory'.
      i.request.path("/mallory");
      // Define the response we want returned.
      i.response
        .ok()
        .content_type("text/plain")
        .body("That is some good Mallory.");

      // Return the interaction back to the pact framework
      i.clone()
    })
    .await
    .start_mock_server_async(None, None)
    .await;

  // You would use your actual client code here.
  let mallory_url = alice_service.path("/mallory");
  let response = reqwest::get(mallory_url).await.expect("could not fetch URL");
  let body = response.text().await.expect("could not read response body");
  assert_eq!(body, "That is some good Mallory.");
}

#[test_log::test]
#[should_panic]
fn mock_server_failing_validation() {
    let hello_service = PactBuilder::new("Hello CLI", "Hello Server")
        .interaction("request a greeting", "", |mut i| {
          i.request.path("/hello");
          i.response.body("Hello!");
          i.clone()
        })
      .start_mock_server(None, None);
    // Call with the wrong URL, which should lead to a panic at the end of
    // the function.
    let url = hello_service.path("/goodbye");
    let _ = reqwest::blocking::get(url);
}

#[test_log::test(tokio::test)]
async fn duplicate_interactions() {
  let u8 = random::<u8>();
  let output_dir = output_dir(&*format!("target/pact_dir_{:03}", u8));

  for _ in 1..=3 {
    let mock_service = PactBuilder::new("consumer 1", "provider 1")
      .interaction("tricky test", "", |mut interaction| {
        interaction
          .request
          .put()
          .json_body(pact_consumer::json_pattern!({
                          "name": pact_consumer::like!("mai"),
                          "street": pact_consumer::like!("5th"),
                          "state": pact_consumer::like!("VA"),
                      }))
          .path("/rolex.html");
        interaction.response.body("TrixR4Kidz");
        interaction
      })
      .with_output_dir(&output_dir)
      .start_mock_server(None, None);

    let mock_url = mock_service.url();

    assert_eq!(
      Client::new()
        .put(&format!("{}rolex.html", mock_url))
        .json(&serde_json::json!({
                      "name": "mai",
                      "street": "5th",
                      "state": "VA",
                  }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(),
      "TrixR4Kidz",
    );
  }

  let path = output_dir.join("consumer 1-provider 1.json");
  let written_pact = RequestResponsePact::read_pact(path.as_path()).unwrap();
  let _ = fs::remove_dir_all(output_dir);

  expect!(written_pact.interactions.len()).to(be_equal_to(1));
}

#[test_log::test(tokio::test)]
async fn test_two_interactions() {
  {
    let mock_service = PactBuilder::new("test_two_interactions_consumer", "test_two_interactions_provider")
      .interaction("looks for something that doesn't exist", "", |mut i| {
        i.request
          .post()
          .path("/")
          .content_type("application/json")
          .json_body(like!(json!({"key": "i_dont_exist"})));
        i.response
          .content_type("application/json")
          .json_body(json!({"count": 0, "results": []}));
        i
      })
      .start_mock_server(None, None);

    let mock_url = mock_service.url();
    Client::new().post(mock_url).json(&json!({"key": "i_dont_exist"})).send().await.unwrap();
  }

  {
    let mock_service = PactBuilder::new("test_two_interactions_consumer", "test_two_interactions_provider")
      .interaction("looks for something that exists", "", |mut i| {
        i.request
          .post()
          .path("/")
          .content_type("application/json")
          .json_body(like!(json!({"key": "i_exist"})));
        i.response
          .content_type("application/json")
          .json_body(json!({"count": 1, "results": ["i_exist"]}));
        i
      })
      .start_mock_server(None, None);

    let mock_url = mock_service.url();
    Client::new().post(mock_url).json(&json!({"key": "i_exist"})).send().await.unwrap();
  }

  let path_file = Path::new("target/pacts/test_two_interactions_consumer-test_two_interactions_provider.json");
  expect!(path_file.exists()).to(be_true());

  let pact = RequestResponsePact::read_pact(&path_file).unwrap();
  expect!(pact.interactions.len()).to(be_equal_to(2));
}

#[derive(Deserialize, Serialize, Default, Copy, Clone, Debug, Eq, PartialEq)]
struct Data {
  id: usize,
}

#[tokio::test]
#[should_panic]
async fn post_json_with_incorrect_content_type() {
  let pact = PactBuilder::new_v4("SHIT", "TEST")
    .interaction("posting a Test", "post", |mut i| {
      i.test_name("post_test");
      i.request
        .post()
        .path("/")
        .json_utf8()
        .json_body(json_pattern!(like!(json!(Data::default()))));
      i.response
        .created()
        .json_utf8()
        .json_body(json_pattern!(like!(json!(Data::default()))));
      i.clone()
    })
    .start_mock_server(None, None);

  let response = reqwest::Client::new().post(pact.path("/"))
    .json(&Data::default())
    .send()
    .await
    .expect("could not fetch URL");
  println!("response: {:#?}", response);
  // response: 500
  assert_eq!(StatusCode::CREATED, response.status());
  // "x-pact": "Request-Mismatch"

  let body = response.json::<Data>().await.expect("could not read response body");
  assert_eq!(Data::default(), body);
}

// Issue #300
#[test_log::test(tokio::test)]
async fn multi_value_headers()     {
  let alice_service = PactBuilder::new_v4("Consumer", "Alice Service")
    .interaction("a retrieve Mallory request", "", |mut i| {
      i.given("there is some good mallory");
      i.request.path("/mallory");
      i.request.header("accept", "application/problem+json, application/json, text/plain, */*");
      i.response
        .ok()
        .content_type("text/plain")
        .body("That is some good Mallory.");

      i.clone()
    })
    .start_mock_server(None, None);

  let mallory_url = alice_service.path("/mallory");
  let client = reqwest::Client::new();
  let response = client.get(mallory_url)
    .header("accept", "application/problem+json, application/json, text/plain, */*")
    .send().await
    .expect("could not fetch URL");
  let body = response.text().await.expect("could not read response body");
  assert_eq!(body, "That is some good Mallory.");
}

// Issue #301
#[test_log::test]
#[should_panic]
fn each_key_matcher()     {
  let service = PactBuilder::new_v4("Consumer", "Service")
    .interaction("a request only checks the keys and ignores the values", "", |mut i| {
      i.request.put()
        .path("/eachKeyMatches")
        .json_body(object_matching!(
          json!({
            "key1": "a string we don't care about",
            "key2": 1,
          }),
          [
            each_key(matching_regex!("[a-z]{3,}[0-9]", "key1")),
            each_value(matching_regex!("[a-z]{3,}[0-9]", "value1"))
          ]
        ));
      i.response.ok();
      i
    })
    .start_mock_server(None, None);

  let url = service.path("/eachKeyMatches");
  let client = reqwest::blocking::Client::new();
  let response = client.put(url)
    .json(&json!({
      "1": "foo",
      "not valid": 1,
      "key": "value",
      "key2": "value2"
    }))
    .send()
    .unwrap();
  expect!(response.status().is_server_error()).to(be_true());
}

const PROTO: &str = "
syntax = \"proto3\";

package area_calculator;

service Calculator {
  rpc calculate (ShapeMessage) returns (AreaResponse) {}
}

message ShapeMessage {
  oneof shape {
    Square square = 1;
    Rectangle rectangle = 2;
    Circle circle = 3;
    Triangle triangle = 4;
    Parallelogram parallelogram = 5;
  }
}

message Square {
  float edge_length = 1;
}

message Rectangle {
  float length = 1;
  float width = 2;
}

message Circle {
  float radius = 1;
}

message Triangle {
  float edge_a = 1;
  float edge_b = 2;
  float edge_c = 3;
}

message Parallelogram {
  float base_length = 1;
  float height = 2;
}

message AreaResponse {
  float value = 1;
}
";

// Issue https://github.com/YOU54F/pact-ruby-ffi/issues/6
// Note, this test requires the gRPC plugin to be installed
#[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
#[ignore = "test requires Protobuf plugin"]
async fn test_protobuf_plugin_contents_merge_with_existing_interaction() {
  let tmp = tempfile::tempdir().unwrap();
  let proto_path = tmp.path().join("area-calculator.proto");
  let mut proto_file = fs::File::create(proto_path.clone()).unwrap();
  proto_file.write_all(PROTO.as_bytes()).unwrap();

  // 1. create an existing pact with single plugin interaction
  // 2. create a new plugin interaction that only differs by interaction description
  // expected: 2 interactions are generated, identical bar interaction description
  // actual: existing interaction is modified at response[0].contents, when merging 2nd interaction

  let pact_1_file_path = {
    let proto_path_str = proto_path.to_string_lossy();
    let mut pact_builder = PactBuilderAsync::new_v4("PluginMergeConsumer", "PluginMergeProvider");
    let pact_1 = pact_builder
      .using_plugin("protobuf", None).await
      .synchronous_message_interaction("description 1", |mut i| async move {
        i.contents_from(json!({
          "pact:proto": proto_path_str,
          "pact:content-type": "application/protobuf",
          "pact:proto-service": "Calculator/calculate",

          "request": {
            "rectangle": {
              "length": "matching(number, 3)",
              "width": "matching(number, 4)"
            }
          },

          "responseMetadata": {
            "grpc-status": "UNIMPLEMENTED",
            "grpc-message": "Not implemented"
          }
        })).await;
        i
      })
      .await
      .build();
    let pact_file_name = pact_1.default_file_name();
    let path_file_path = tmp.path().join(pact_file_name);
    write_pact(pact_1, &path_file_path, PactSpecification::V4, false).unwrap();
    path_file_path.clone()
  };

  let pact_file = fs::File::open(pact_1_file_path).unwrap();
  let json: serde_json::Value = serde_json::from_reader(&pact_file).unwrap();

  let interaction_1_response_contents = &json["interactions"][0]["response"][0]["contents"];
  let expected_response_contents = json!({
    "content": "",
  });
  assert_eq!(
    &expected_response_contents,
    interaction_1_response_contents
  );

  // Setup New interaction and write to existing pact file - validate .interactions[0].response[0].contents
  let pact_2_file_path = {
    let mut pact_builder_2 = PactBuilderAsync::new_v4("PluginMergeConsumer", "PluginMergeProvider");
    let pact_2 = pact_builder_2
      .using_plugin("protobuf", None).await
      .synchronous_message_interaction("description 2", |mut i| async move {
        i.contents_from(json!({
          "pact:proto": proto_path,
          "pact:content-type": "application/protobuf",
          "pact:proto-service": "Calculator/calculate",

          "request": {
            "rectangle": {
              "length": "matching(number, 3)",
              "width": "matching(number, 4)"
            }
          },

          "responseMetadata": {
            "grpc-status": "UNIMPLEMENTED",
            "grpc-message": "Not implemented"
          }
        })).await;
        i
      })
      .await
      .build();
    let pact_file_name = pact_2.default_file_name();
    let path_file_path = tmp.path().join(pact_file_name);
    write_pact(pact_2, &path_file_path, PactSpecification::V4, false).unwrap();
    path_file_path.clone()
  };

  let pact_file = fs::File::open(pact_2_file_path).unwrap();
  let json_2: serde_json::Value = serde_json::from_reader(pact_file).unwrap();

  let interaction_2_description = &json_2["interactions"][0]["description"];
  let interaction_2_description_2 = &json_2["interactions"][1]["description"];
  let interaction_2_response_contents = &json_2["interactions"][0]["response"][0]["contents"];
  let interaction_2_response_contents_2 = &json_2["interactions"][1]["response"][0]["contents"];

  assert_eq!("description 1", interaction_2_description.as_str().unwrap());
  assert_eq!("description 2", interaction_2_description_2.as_str().unwrap());
  assert_eq!(
    &expected_response_contents,
    interaction_2_response_contents_2
  );
  assert_eq!(
    &expected_response_contents,
    interaction_2_response_contents
  );
}
