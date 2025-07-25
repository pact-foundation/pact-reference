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
    println!("FILE: tests/spec_testcases/v2/response/body/null found at key where not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should not be null",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": null
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_no_matches() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array no matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Array of objects, properties match on incorrect objects",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": [
      		{"favouriteColor": "red"},
      		{"favouriteNumber": 2}
      	]
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array no matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array no matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body found, when an empty body was expected",
        "expected" : {
          "body": null
        },
        "actual": {
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn matches_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Responses match",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" feet=\"4\"><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator feet=\"4\" name=\"Mary\"><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_at_top_level_with_matchers_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array at top level with matchers xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML top level array matches",
        "expected": {
          "headers": {"Content-Type": "application/xml"},
          "body" : "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person dob=\"06/10/2015\" name=\"Rogger the Dogger\" id=\"1014753708\" timestamp=\"2015-06-10T20:41:37\"/><cat dob=\"06/10/2015\" name=\"Cat in the Hat\" id=\"8858030303\" timestamp=\"2015-06-10T20:41:37\"/></people>",
          "matchingRules" : {
            "$.body.people.*['@id']" : {
              "match" : "type"
            },
            "$.body.people.*['@name']" : {
              "match" : "type"
            },
            "$.body.people.*['@dob']" : {
              "match": "regex", "regex" : "\\d{2}/\\d{2}/\\d{4}"
            },
            "$.body.people.*['@timestamp']" : {
              "match": "regex", "regex" : "\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person dob=\"11/06/2015\" name=\"Bob The Builder\" id=\"1234567890\" timestamp=\"2000-06-10T20:41:37\"/><cat dob=\"12/10/2000\" name=\"Slinky Malinky\" id=\"6677889900\" timestamp=\"2015-06-10T22:98:78\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level with matchers xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level with matchers xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_type_matcher_mismatch_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with type matcher mismatch xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML array with type matcher mismatch",
        "expected": {
          "headers": {},
          "body" : "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person>Fred</person></people>",
          "matchingRules" : {
            "$.body.people" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person>Fred</person><person>Fred</person><cat>Fred</cat></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher mismatch xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher mismatch xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn no_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v2/response/body/no body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "No body, no content-type",
        "expected" : {
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/no body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/no body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn keys_out_of_order_match_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/keys out of order match xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Favourite number and favourite colours out of order",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator favouriteNumber=\"7\" favouriteColours=\"red, blue\" />"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator favouriteColours=\"red, blue\" favouriteNumber=\"7\" />"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/keys out of order match xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/keys out of order match xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_at_top_level_with_matchers() {
    println!("FILE: tests/spec_testcases/v2/response/body/array at top level with matchers.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "top level array matches",
        "expected": {
          "headers": {"Content-Type": "application/json"},
          "body" : [ {
            "dob" : "06/11/2015",
            "name" : "Rogger the Dogger",
            "id" : 3380634027,
            "timestamp" : "2015-06-11T13:17:29"
          }, {
            "dob" : "06/11/2015",
            "name" : "Cat in the Hat",
            "id" : 1284270029,
            "timestamp" : "2015-06-11T13:17:29"
          } ],
          "matchingRules" : {
            "$.body[0].id" : {
              "match" : "type"
            },
            "$.body[1].id" : {
              "match" : "type"
            },
            "$.body[0].name" : {
              "match" : "type"
            },
            "$.body[1].name" : {
              "match" : "type"
            },
            "$.body[1].dob" : {
              "match": "regex", "regex" : "\\d{2}/\\d{2}/\\d{4}"
            },
            "$.body[1].timestamp" : {
              "match": "regex", "regex" : "\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}"
            },
            "$.body[0].timestamp" : {
              "match": "regex", "regex" : "\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}"
            },
            "$.body[0].dob" : {
              "match": "regex", "regex" : "\\d{2}/\\d{2}/\\d{4}"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "dob": "11/06/2015",
              "name": "Bob The Builder",
              "id": 1234567890,
              "timestamp": "2000-06-10T20:41:37"
            },
            {
              "dob": "12/10/2000",
              "name": "Slinky Malinky",
              "id": 6677889900,
              "timestamp": "2015-06-10T22:98:78"
            }
          ]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level with matchers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level with matchers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_at_top_level() {
    println!("FILE: tests/spec_testcases/v2/response/body/array at top level.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "top level array matches",
        "expected": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "dob": "06/10/2015",
              "name": "Rogger the Dogger",
              "id": 1014753708,
              "timestamp": "2015-06-10T20:41:37"
            },
            {
              "dob": "06/10/2015",
              "name": "Cat in the Hat",
              "id": 8858030303,
              "timestamp": "2015-06-10T20:41:37"
            }
          ]
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "dob": "06/10/2015",
              "name": "Rogger the Dogger",
              "id": 1014753708,
              "timestamp": "2015-06-10T20:41:37"
            },
            {
              "dob": "06/10/2015",
              "name": "Cat in the Hat",
              "id": 8858030303,
              "timestamp": "2015-06-10T20:41:37"
            }
          ]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_key() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary",
              "age": 3
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing key.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing key.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected index with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue", null]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn non_empty_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/non empty body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Non empty body found, when an empty body was expected",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": null
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/null found in array when not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to be strings found a null",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn deeply_nested_objects_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/deeply nested objects xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
      	"match": true,
      	"comment": "XML Comparisons should work even on nested objects",
      	"expected" : {
      		"headers": {"Content-Type": "application/xml"},
      		"body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><object1><object2><object4><object5 name=\"Mary\"><friends><friend>Fred</friend><friend>John</friend></friends></object5><object6 phoneNumber=\"1234567890\"/></object4></object2></object1>"
      	},
      	"actual": {
      		"headers": {"Content-Type": "application/xml"},
      		"body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><object1 color=\"red\"><object2><object4><object5 name=\"Mary\" gender=\"F\"><friends><friend>Fred</friend><friend>John</friend></friends></object5><object6 phoneNumber=\"1234567890\"/></object4></object2></object1>"
      	}
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/deeply nested objects xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/deeply nested objects xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v2/response/body/empty body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body, no content-type",
        "expected" : {
          "body": ""
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": ""
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/empty body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/empty body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn value_found_in_array_when_empty_expected_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/value found in array when empty expected xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Favourite numbers expected to contain empty, but non-empty found",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber></favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber>2</favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/value found in array when empty expected xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/value found in array when empty expected xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn additional_property_with_type_matcher_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v2/response/body/additional property with type matcher that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "additional property with type matcher wildcards that don't match",
        "expected": {
          "headers": {},
          "body" : {
            "myPerson": {
              "name": "Any name"
            }
          },
          "matchingRules" : {
            "$.body.myPerson.*" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "myPerson": {
              "name": 39,
              "age": "39",
              "nationality": "Australian"
            }
          }    
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/additional property with type matcher that does not match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/additional property with type matcher that does not match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_type_matching() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array type matching.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "objects in array type matching",
        "expected": {
          "headers": {},
          "body": [{
            "name": "John Smith",
            "age": 50
          }],
          "matchingRules": {
            "$.body": {
              "match": "type"
            },
            "$.body[*]": {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": [{
            "name": "Peter Peterson",
            "age": 22,
            "gender": "Male"
          }, {
            "name": "John Johnston",
            "age": 64
          }]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array type matching.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array type matching.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_second_matches() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array second matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Property of second object matches, but unexpected element recieved",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": [
      		{"favouriteColor": "red"}
      	]
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array second matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array second matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn objects_in_array_no_matches_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array no matches xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Array of objects, properties match on incorrect objects",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"red\" favouriteNumber=\"2\"/></people>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"blue\" favouriteNumber=\"4\"/><person favouriteColour=\"red\" favouriteNumber=\"2\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array no matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array no matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_type_matcher_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with type matcher xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "array with type matcher",
        "expected": {
          "headers": {},
          "body" : "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person>Fred</person></people>",
          "matchingRules" : {
            "$.body.people" : {
              "match": "type"
            },
            "$.body.people[*]" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person>Fred</person><person>George</person><person>Cat</person></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_in_different_order_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array in different order xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Favourite colours in wrong order",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><red/><blue/></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><blue/><red/></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array in different order xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array in different order xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected" : {
          "headers": {"Content-Type": "application/json"}
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v2/response/body/null body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body, no content-type",
        "expected" : {
          "body": null
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": null
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/null body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/null body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/not null found in array when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to contain null, but not null found",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn unexpected_index_with_non_empty_value_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected index with non-empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Unexpected favourite colour",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour><favouriteColour>taupe</favouriteColour></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with non-empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with non-empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn different_value_found_at_index_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/different value found at index xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Incorrect favourite colour",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>purple</favouriteColour></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at index xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at index xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_floats() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches with floats.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Response match with floats",
        "expected": {
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "$.body.product.price": {"match": "regex", "regex": "\\d(\\.\\d{1,2})"}
          },
          "body": [
            {
              "product": {
                  "id": 123,
                  "description": "Television",
                  "price": 500.55
              }
            }
          ]
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "product": {
                  "id": 123,
                  "description": "Television",
                  "price": 500.55
              }
            }
          ]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with floats.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with floats.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn matches_with_regex_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches with regex xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Requests match with regex",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "$.body.alligator['@name']": {"match": "regex", "regex": "\\w+"}
          },
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" feet=\"4\" favouriteNumber=\"7\" favouriteColours=\"red, blue\" />"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Harry\" feet=\"4\" favouriteNumber=\"7\" favouriteColours=\"red, blue\" />"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with regex xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with regex xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v2/response/body/plain text that does not match.json");
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/plain text that does not match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/plain text that does not match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v2/response/body/array in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["blue", "red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array in different order.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array in different order.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn property_name_is_different_case() {
    println!("FILE: tests/spec_testcases/v2/response/body/property name is different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Property names on objects are case sensitive",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "FavouriteColour": "red"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouritecolour": "red"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/property name is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/property name is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches with regex.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match with regex",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "$.body.alligator.name": {"match": "regex", "regex": "\\w+"}
          },
          "body": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": 4,
              "name": "Harry",
              "favouriteColours": ["red","blue"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with regex.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with regex.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_type_matcher() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with type matcher.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "array with type matcher",
        "expected": {
          "headers": {},
          "body" : {
            "myDates": [
              10
            ]
          },
          "matchingRules" : {
            "$.body.myDates" : {
              "match": "type"
            },
            "$.body.myDates[*]" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "myDates": [
              20,
              5,
              1910
            ]
          }    
        }
      }   
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn unexpected_index_with_missing_value_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected index with missing value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Unexpected favourite colour with missing value",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour><favouriteColour></favouriteColour></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with missing value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with missing value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_at_top_level_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array at top level xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML top level array matches",
        "expected": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><rogger dob=\"06/10/2015\" name=\"Rogger the Dogger\" id=\"1014753708\" timestamp=\"2015-06-10T20:41:37\"/><cat dob=\"06/10/2015\" name=\"Cat in the Hat\" id=\"8858030303\" timestamp=\"2015-06-10T20:41:37\"/></people>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><rogger dob=\"06/10/2015\" name=\"Rogger the Dogger\" id=\"1014753708\" timestamp=\"2015-06-10T20:41:37\"/><cat dob=\"06/10/2015\" name=\"Cat in the Hat\" id=\"8858030303\" timestamp=\"2015-06-10T20:41:37\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array at top level xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn no_body_no_content_type_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/no body no content type xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML No body, no content-type",
        "expected" : {
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/no body no content type xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/no body no content type xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_regex_matcher_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with regex matcher xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML array with regex matcher",
        "expected": {
          "headers": {},
          "body" : "<?xml version=\"1.0\" encoding=\"UTF-8\"?><myDates><date>29/10/2015</date></myDates>",
          "matchingRules" : {
            "$.body.myDates": {
              "match": "type"
            },
            "$.body.myDates.date['#text']" : {
              "match": "regex", "regex" : "\\d{2}/\\d{2}/\\d{4}"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><myDates><date>01/11/2010</date><date>15/12/2014</date><date>30/06/2015</date></myDates>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with regex matcher xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with regex matcher xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_matches() {
    println!("FILE: tests/spec_testcases/v2/response/body/plain text that matches.json");
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/plain text that matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/plain text that matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn unexpected_key_with_empty_value_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected key with empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Unexpected phone number with empty value",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" phoneNumber=\"\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_index() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator": {
              "favouriteColours": ["red"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing index.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing index.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v2/response/body/different value found at index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at index.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at index.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn missing_body_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing body xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Missing body",
        "expected" : {
          "headers": {"Content-Type": "application/xml"}
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body, no content-type",
        "expected" : {
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing body no content type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Responses match",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/number found in array when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite numbers expected to be strings found a number",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": ["1",2,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body() {
    println!("FILE: tests/spec_testcases/v2/response/body/empty body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": ""
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": ""
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/empty body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/empty body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn deeply_nested_objects() {
    println!("FILE: tests/spec_testcases/v2/response/body/deeply nested objects.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
      	"match": true,
      	"comment": "Comparisons should work even on nested objects",
      	"expected" : {
      		"headers": {"Content-Type": "application/json"},
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
      		"headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/deeply nested objects.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/deeply nested objects.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn unexpected_key_with_non_empty_value_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected key with non-empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Unexpected phone number",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" phoneNumber=\"12345678\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with non-empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with non-empty value xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn missing_key_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing key xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Missing key alligator name",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" age=\"3\"></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator age=\"3\"></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing key xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing key xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/string found in array when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,2,3]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteNumbers": [1,"2",3]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_type_matcher_mismatch() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with type matcher mismatch.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "array with type matcher mismatch",
        "expected": {
          "headers": {},
          "body" : {
            "myDates": [
              10
            ]
          },
          "matchingRules" : {
            "$.body.myDates[*]" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "myDates": [
              20,
              5,
              "100299"
            ]
          }    
        }
      }   
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher mismatch.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with type matcher mismatch.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected index with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "favouriteColours": ["red","blue","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/number found at key when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn keys_out_of_order_match() {
    println!("FILE: tests/spec_testcases/v2/response/body/keys out of order match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Favourite number and favourite colours out of order",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
      		"favouriteNumber": 7,
              "favouriteColours": ["red","blue"]
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
              "favouriteColours": ["red","blue"],
      		"favouriteNumber": 7
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/keys out of order match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/keys out of order match.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_integers() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches with integers.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Response match with integers",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "$.body.alligator.feet": {"match": "regex", "regex": "[0-9]"}
          },
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
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with integers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with integers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/not null found at key when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": null
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body() {
    println!("FILE: tests/spec_testcases/v2/response/body/null body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": null
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": null
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/null body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/null body.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn different_value_found_at_key_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/different value found at key xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Incorrect value at alligator name",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Fred\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at key xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at key xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_regex_matcher() {
    println!("FILE: tests/spec_testcases/v2/response/body/array with regex matcher.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "array with regex matcher",
        "expected": {
          "headers": {},
          "body" : {
            "myDates": [
              "29/10/2015"
            ]
          },
          "matchingRules" : {
            "$.body.myDates": {
              "match": "type"
            },
            "$.body.myDates[*]" : {
              "match": "regex", "regex" : "\\d{2}/\\d{2}/\\d{4}"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "myDates": [
              "01/11/2010",
              "15/12/2014",
              "30/06/2015"
            ]
          }    
        }
      }   
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with regex matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/array with regex matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v2/response/body/different value found at key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at key.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/different value found at key.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected key with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number with null value",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v2/response/body/string found at key when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": 4
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": "4"
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn objects_in_array_second_matches_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array second matches xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Property of second object matches, but unexpected element received",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"red\"/></people>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"blue\" favouriteNumber=\"4\"/><person favouriteColour=\"red\" favouriteNumber=\"2\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array second matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array second matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn missing_index_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/missing index xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour></favouriteColours></alligator>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing index xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/missing index xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn objects_in_array_first_matches_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array first matches xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Properties match but unexpected element received",
        "expected": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"red\"/></people>"
        },
        "actual": {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person favouriteColour=\"blue\" favouriteNumber=\"4\"/><person favouriteColour=\"red\" favouriteNumber=\"2\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array first matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array first matches xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v2/response/body/unexpected key with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
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
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn additional_property_with_type_matcher() {
    println!("FILE: tests/spec_testcases/v2/response/body/additional property with type matcher.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "additional property with type matcher wildcards",
        "expected": {
          "headers": {},
          "body" : {
            "myPerson": {
              "name": "Any name"
            }
          },
          "matchingRules" : {
            "$.body.myPerson.*" : {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": {
            "myPerson": {
              "name": "Jon Peterson",
              "age": 39,
              "nationality": "Australian"
            }
          }    
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/additional property with type matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/additional property with type matcher.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn objects_in_array_type_matching_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array type matching xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML objects in array type matching",
        "expected": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person name=\"John Smith\" age=\"50\"/></people>",
          "matchingRules": {
            "$.body": {
              "match": "type"
            },
            "$.body[*]": {
              "match": "type"
            },
            "$.body[*].*": {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person name=\"Peter Peterson\" age=\"22\" gender=\"Male\"/><person name=\"John Johnston\" age=\"64\"/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array type matching xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array type matching xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_with_type_mismatching() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array with type mismatching.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "objects in array with type mismatching",
        "expected": {
          "headers": {},
          "body": [{
            "Name": "John Smith",
            "Age": 50
          }],
          "matchingRules": {
            "$.body[*]": {
              "match": "type"
            },
            "$.body[*].*": {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": [{
            "name": "Peter Peterson",
            "age": 22,
            "gender": "Male"
          }, {}]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array with type mismatching.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array with type mismatching.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn objects_in_array_with_type_mismatching_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array with type mismatching xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML objects in array with type mismatching",
        "expected": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person name=\"John Smith\" age=\"50\"/></people>",
          "matchingRules": {
            "$.body[*]": {
              "match": "type"
            },
            "$.body[*].*": {
              "match": "type"
            }
          }
        },
        "actual": {
          "headers": {},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><person name=\"Peter Peterson\" age=\"22\" gender=\"Male\"/><person/></people>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array with type mismatching xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array with type mismatching xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn property_name_is_different_case_xml() {
    println!("FILE: tests/spec_testcases/v2/response/body/property name is different case xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Property names on objects are case sensitive",
        "expected" : {
          "headers": {"Content-Type": "application/xml"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator FavouriteColour=\"red\"/>"
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator favouritecolour=\"red\"/>"
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/property name is different case xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/property name is different case xml.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_type() {
    println!("FILE: tests/spec_testcases/v2/response/body/matches with type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Response match with same type",
        "expected" : {
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "$.body.alligator.name": {"match": "type"},
            "$.body.alligator.feet": {"match": "type"}
          },
          "body": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "alligator":{
              "feet": 5,
              "name": "Harry the very hungry alligator with an extra foot",
              "favouriteColours": ["red","blue"]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/matches with type.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn objects_in_array_first_matches() {
    println!("FILE: tests/spec_testcases/v2/response/body/objects in array first matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Properties match but unexpected element received",
        "expected": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "favouriteColor": "red"
            }
          ]
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": [
            {
              "favouriteColor": "red",
              "favouriteNumber": 2
            },
            {
              "favouriteColor": "blue",
              "favouriteNumber": 2
            }
          ]
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array first matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/response/body/objects in array first matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchers::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V2).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
