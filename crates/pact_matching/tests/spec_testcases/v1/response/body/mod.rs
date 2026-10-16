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
    println!("FILE: tests/spec_testcases/v1/response/body/null found at key where not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should not be null",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "name": null
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_no_matches() {
    println!("FILE: tests/spec_testcases/v1/response/body/objects in array no matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Array of objects, properties match on incorrect objects",
        "expected" : {
          "headers": {},
          "body": [
      		{"favouriteColor": "red"},
      		{"favouriteNumber": 2}
      	]
        },
        "actual": {
          "headers": {},
          "body": [
      		{"favouriteColor": "blue",
      		"favouriteNumber": 4},
      		{"favouriteColor": "red",
      		"favouriteNumber": 2}
      	]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array no matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array no matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_key() {
    println!("FILE: tests/spec_testcases/v1/response/body/missing key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary",
              "age": 3
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/missing key.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/missing key.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v1/response/body/unexpected index with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue", null]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/null found in array when not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to be strings found a null",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_second_matches() {
    println!("FILE: tests/spec_testcases/v1/response/body/objects in array second matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Property of second object matches, but unexpected element recieved",
        "expected" : {
          "headers": {},
          "body": [
      		{"favouriteColor": "red"}
      	]
        },
        "actual": {
          "headers": {},
          "body": [
      		{"favouriteColor": "blue",
      		"favouriteNumber": 4},
      		{"favouriteColor": "red",
      		"favouriteNumber": 2}
      	]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array second matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array second matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/not null found in array when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to contain null, but not null found",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v1/response/body/plain text that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Plain text that does not match",
        "expected" : {
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        },
        "actual": {
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named fred"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/plain text that does not match.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/plain text that does not match.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v1/response/body/array in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["blue", "red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/array in different order.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/array in different order.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn property_name_is_different_case() {
    println!("FILE: tests/spec_testcases/v1/response/body/property name is different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Property names on objects are case sensitive",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "FavouriteColour": "red"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouritecolour": "red"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/property name is different case.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/property name is different case.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_matches() {
    println!("FILE: tests/spec_testcases/v1/response/body/plain text that matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Plain text that matches",
        "expected" : {
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        },
        "actual": {
          "headers": { "Content-Type": "text/plain" },
          "body": "alligator named mary"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/plain text that matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/plain text that matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_index() {
    println!("FILE: tests/spec_testcases/v1/response/body/missing index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator": {
              "favouriteColours": ["red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/missing index.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/missing index.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v1/response/body/different value found at index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/different value found at index.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/different value found at index.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v1/response/body/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Responses match",
        "expected" : {
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/number found in array when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to be strings found a number",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",2,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn deeply_nested_objects() {
    println!("FILE: tests/spec_testcases/v1/response/body/deeply nested objects.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
      	"match": true,
      	"comment": "Comparisons should work even on nested objects",
      	"expected" : {
      		"headers": {},
      		"body": {
      			"object1": {
      				"object2": { 
      					"object4": {
      						"object5": {
      							"name": "Mary",
      							"friends": ["Fred", "John"]
      						},
      						"object6": {
      							"phoneNumber": 1234567890
      						}
      					}
      				}
      			}
      		}
      	},
      	"actual": {
      		"headers": {},
      		"body": {
      			"object1":{
      				"object2": { 
      					"object4":{
      						"object5": {
      							"name": "Mary",
      							"friends": ["Fred", "John"],
      							"gender": "F"
      						},
      						"object6": {
      							"phoneNumber": 1234567890
      						}
      					}
      				},
      				"color": "red"
      			}
      		}
      	}
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/deeply nested objects.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/deeply nested objects.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/string found in array when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,2,3]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,"2",3]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v1/response/body/unexpected index with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/number found at key when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn keys_out_of_order_match() {
    println!("FILE: tests/spec_testcases/v1/response/body/keys out of order match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Favourite number and favourite colours out of order",
        "expected" : {
          "headers": {},
          "body": {
      		"favouriteNumber": 7,
              "favouriteColours": ["red","blue"]
          }
        },
        "actual": {
          "headers": {},
          "body": {
              "favouriteColours": ["red","blue"],
      		"favouriteNumber": 7
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/keys out of order match.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/keys out of order match.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/not null found at key when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": null
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v1/response/body/different value found at key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/different value found at key.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/different value found at key.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v1/response/body/unexpected key with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number with null value",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v1/response/body/string found at key when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v1/response/body/unexpected key with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number",
        "expected" : {
          "headers": {},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_first_matches() {
    println!("FILE: tests/spec_testcases/v1/response/body/objects in array first matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Properties match but unexpected element recieved",
        "expected" : {
          "headers": {},
          "body": [
      		{"favouriteColor": "red"}
      	]
        },
        "actual": {
          "headers": {},
          "body": [
      		{"favouriteColor": "red",
      		"favouriteNumber": 2},
      		{"favouriteColor": "blue",
      		"favouriteNumber": 2}
      	]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array first matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/response/body/objects in array first matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V1).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
