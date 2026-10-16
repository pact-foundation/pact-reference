#[allow(unused_imports)]
use test_log::test;
#[allow(unused_imports)]
use pact_models::PactSpecification;
#[allow(unused_imports)]
use serde_json;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::register_core_entries;
#[allow(unused_imports)]
use pact_models::interaction::{Interaction, http_interaction_from_json};
#[allow(unused_imports)]
use pact_matching::{match_interaction_request, match_interaction_response};
#[allow(unused_imports)]
use pact_models::prelude::{Pact, RequestResponsePact};

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v1_1/response/status/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
          "match": true,
          "comment": "Status matches",
          "expected": {
              "status": 202
          },
          "actual": {
              "status": 202
          }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/response/status/matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/response/status/matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1_1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_status() {
    println!("FILE: tests/spec_testcases/v1_1/response/status/different status.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
          "match": false,
          "comment": "Status is incorrect",
          "expected": {
              "status": 202
          },
          "actual": {
              "status": 400
          }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/response/status/different status.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/response/status/different status.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1_1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
