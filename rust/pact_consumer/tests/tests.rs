use pact_consumer::prelude::*;
use pact_consumer::{json_pattern, json_pattern_internal};
use std::{
  io::prelude::*,
  env,
  path::Path,
  fs
};
use expectest::prelude::*;
use expectest::expect;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use pact_matching::models::Pact;

/// This is supposed to be a doctest in lib.rs, but it's breaking there, so
/// we have an executable copy here.
#[test]
fn mock_server_passing_validation() {
    use expectest::*;

    let _ = env_logger::init();

    let output_dir = Path::new("target/pact_dir");
    fs::remove_dir_all(output_dir).unwrap_or(());

    env::set_var("PACT_OUTPUT_DIR", &output_dir);

    {
      // Define the Pact for the test, specify the names of the consuming
      // application and the provider application.
      let alice_service = PactBuilder::new("Consumer", "Alice Service")
        // Start a new interaction. We can add as many interactions as we want.
        .interaction("a retrieve Mallory request", |i| {
          // Defines a provider state. It is optional.
          i.given("there is some good mallory");
          // Define the request, a GET (default) request to '/mallory'.
          i.request.path("/mallory");
          // Define the response we want returned.
          i.response
            .success()
            .content_type("text/plain")
            .body("That is some good Mallory.");
        })
        .start_mock_server();

      // You would use your actual client code here.
      let mallory_url = alice_service.path("/mallory");
      let mut response = reqwest::blocking::get(mallory_url).expect("could not fetch URL");
      let mut body = String::new();
      response.read_to_string(&mut body).expect("could not read response body");
      assert_eq!(response.status(), StatusCode::OK);
      assert_eq!(body, "That is some good Mallory.");
    }

    // When your test has finished running, all verifications will be performed
    // automatically, and an error will be thrown if any have failed.

    env::remove_var("PACT_OUTPUT_DIR");

    let path_file = Path::new("target/pact_dir/Consumer-Alice Service.json");
    expect!(path_file.exists()).to(be_true());
}

#[test]
#[should_panic]
fn mock_server_failing_validation() {
    let _ = env_logger::init();

    let hello_service = PactBuilder::new("Hello CLI", "Hello Server")
        .interaction("request a greeting", |i| {
            i.request.path("/hello");
            i.response.body("Hello!");
        })
        .start_mock_server();
    // Call with the wrong URL, which should lead to a panic at the end of
    // the function.
    let url = hello_service.path("/goodbye");
    let _ = reqwest::blocking::get(url);
}

#[test]
fn duplicate_interactions() {
  let _ = env_logger::init();

  let output_dir = Path::new("/tmp/duplicate_interactions");
  fs::remove_dir_all(output_dir).unwrap_or(());

  env::set_var("PACT_OUTPUT_DIR", &output_dir);

  for _ in 1..3 {
    let mock_service = PactBuilder::new("consumer 1", "provider 1")
      .interaction("tricky test", |interaction| {
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
      })
      .build()
      .start_mock_server();

    let mock_url = mock_service.url().as_ref();

    assert_eq!(
      Client::new()
        .put(&format!("{}rolex.html", mock_url))
        .json(&serde_json::json!({
                      "name": "mai",
                      "street": "5th",
                      "state": "VA",
                  }))
        .send()
        .unwrap()
        .text()
        .unwrap(),
      "TrixR4Kidz",
    );
  }

  env::remove_var("PACT_OUTPUT_DIR");

  let path = output_dir.join("consumer 1-provider 1.json");
  let written_pact = Pact::read_pact(path.as_path()).unwrap();
  expect!(written_pact.interactions.len()).to(be_equal_to(1));
}
