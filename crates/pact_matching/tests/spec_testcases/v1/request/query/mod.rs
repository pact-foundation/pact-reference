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
async fn trailing_amperand() {
    println!("FILE: tests/spec_testcases/v1/request/query/trailing amperand.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Query strings are matched using basic string equality, these are not equal. (not supported by pact-jvm)",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John&",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/query/trailing amperand.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/query/trailing amperand.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_param_values() {
    println!("FILE: tests/spec_testcases/v1/request/query/different param values.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - hippo is Fred instead of John",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=Fred",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/query/different param values.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/query/different param values.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v1/request/query/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/query/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/query/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_equals_in_the_query_value() {
    println!("FILE: tests/spec_testcases/v1/request/query/matches with equals in the query value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are equivalent",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "options=delete.topic.enable=true&broker=1",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "options=delete.topic.enable%3Dtrue&broker=1",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/query/matches with equals in the query value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/query/matches with equals in the query value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_param_order() {
    println!("FILE: tests/spec_testcases/v1/request/query/different param order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Query strings are matched using basic string equality, these are not equal. (not supported by pact-jvm)",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "hippo=John&alligator=Mary",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/query/different param order.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/query/different param order.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
