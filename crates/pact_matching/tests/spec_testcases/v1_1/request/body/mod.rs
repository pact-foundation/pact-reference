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
async fn null_found_at_key_where_not_null_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/null found at key where not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": null
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/missing body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body found, when an empty body was expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": ""
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_key() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/missing key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary",
              "age": 3
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing key.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing key.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/unexpected index with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue", null]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn non_empty_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/non empty body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Non empty body found, when an empty body was expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/null found in array when not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/empty body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "body": ""
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": ""
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/empty body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/empty body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/missing body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/null body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": null
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/not null found in array when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to contain null, but not null found",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/plain text that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Plain text that does not match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named fred"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/plain text that does not match.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/plain text that does not match.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/array in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["blue", "red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/array in different order.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/array in different order.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_matches() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/plain text that matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Plain text that matches",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/plain text that matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/plain text that matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_index() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/missing index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator": {
              "favouriteColours": ["red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing index.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing index.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/different value found at index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/different value found at index.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/different value found at index.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/missing body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": ""
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/missing body no content type.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "feet": 4,
              "name": "Mary",
              "favouriteColours": ["red","blue"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/matches.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/number found in array when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",2,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/empty body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": ""
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": ""
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/empty body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/empty body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/string found in array when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,2,3]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,"2",3]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/unexpected index with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/number found at key when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/not null found at key when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": null
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/null body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": null
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/null body.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/different value found at key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/different value found at key.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/different value found at key.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/unexpected key with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected phone number with null value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary",
              "phoneNumber": null
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/string found at key when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v1_1/request/body/unexpected key with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected phone number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary",
              "phoneNumber": "12345678"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1_1/request/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V1_1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V1_1).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
