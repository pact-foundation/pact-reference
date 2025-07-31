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
    println!("FILE: tests/spec_testcases/v4/request/body/null found at key where not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": null
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/null found at key where not null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn no_body_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/no body xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Missing body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_at_least_one_element_matching_by_example() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with at least one element matching by example.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Mary"
                },
                {
                  "name": "Susan"
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element matching by example.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element matching by example.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/missing body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body found, when an empty body was expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body found when empty expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/matches xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Requests match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" feet=\"4\"><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator feet=\"4\" name=\"Mary\"><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn no_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/no body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "No body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_at_top_level() {
    println!("FILE: tests/spec_testcases/v4/request/body/array at top level.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "top level array matches",
        "expected": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": [
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
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": [
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
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array at top level.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array at top level.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_nested_array_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with nested array that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Nested arrays do not match, age is wrong type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children": {
                "matchers": [
                  {
                    "min": 1
                  }
                ]
              },
              "$.animals[*].children[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred",
                  "children": [
                    {
                      "age": 9
                    }
                  ]
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Mary",
                  "children": [
                    {
                      "age": "9"
                    }
                  ]
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with nested array that does not match.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with nested array that does not match.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_at_least_one_element_not_matching_example_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with at least one element not matching example type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Wrong type for name key",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Mary"
                },
                {
                  "name": 1
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element not matching example type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element not matching example type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_key() {
    println!("FILE: tests/spec_testcases/v4/request/body/missing key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "age": 3
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing key.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing key.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_nested_array_that_matches() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with nested array that matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Nested arrays match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred",
                  "children": [
                    {
                      "age": 9
                    }
                  ]
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Mary",
                  "children": [
                    {
                      "age": 3
                    },
                    {
                      "age": 5
                    },
                    {
                      "age": 5456
                    }
                  ]
                },
                {
                  "name": "Jo",
                  "children": [
                    {
                      "age": 0
                    }
                  ]
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with nested array that matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with nested array that matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected index with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue",
                  null
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn non_empty_body_found_when_empty_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/non empty body found when empty expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Non empty body found, when an empty body was expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": null
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/non empty body found when empty expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/null found in array when not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  "2",
                  "3"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  null,
                  "3"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/null found in array when not null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/empty body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "body": {
            "content": ""
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": ""
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/value found in array when empty expected xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Favourite numbers expected to be strings found an empty value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber>2</favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber></favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/value found in array when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/value found in array when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_regular_expression_that_does_not_match_in_element_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with regular expression that does not match in element xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Types and regular expressions match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals.0": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals.1": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals.alligator['@phoneNumber']": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator phoneNumber=\"0415674567\"/></animals>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator phoneNumber=\"123\"/><alligator phoneNumber=\"abc\"/></animals>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression that does not match in element xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression that does not match in element xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/array in different order xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Favourite colours in wrong order",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>blue</favouriteColour><favouriteColour>red</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array in different order xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array in different order xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_size_less_than_required() {
    println!("FILE: tests/spec_testcases/v4/request/body/array size less than required.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Array must have at least 2 elements",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 2
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array size less than required.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array size less than required.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_at_least_one_element_matching_by_example_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with at least one element matching by example xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Tag with at least one element match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals.alligator": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator name=\"Fred\"/></animals>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator name=\"Mary\"/><alligator name=\"Susan\"/></animals>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element matching by example xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with at least one element matching by example xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body() {
    println!("FILE: tests/spec_testcases/v4/request/body/missing body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex_with_bracket_notation() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with regex with bracket notation.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match with regex",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$['2'].str": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "2": {
                "str": "jildrdmxddnVzcQZfjCA"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "2": {
                "str": "saldfhksajdhffdskkjh"
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex with bracket notation.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex with bracket notation.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/null body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "body": {
            "content": null
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": null
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/null body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/null body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/not null found in array when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to contain null, but not null found",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  null,
                  "3"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  "2",
                  "3"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/not null found in array when null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected index with non-empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Unexpected favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour><favouriteColour>taupe</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with non-empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with non-empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/different value found at index xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Incorrect favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>taupe</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at index xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at index xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_floats() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with floats.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Request matches with floats",
        "expected": {
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.product.price": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d(\\.\\d{1,2})"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": [
              {
                "product": {
                  "id": 123,
                  "description": "Television",
                  "price": 500.55
                }
              }
            ]
          }
        },
        "actual": {
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": [
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
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with floats.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with floats.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/matches with regex xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Requests match with regex",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$.alligator['@name']": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              },
              "$.alligator.favouriteColours.favouriteColour.#text": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "red|blue"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" feet=\"4\"><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Harry\" feet=\"4\"><favouriteColours><favouriteColour>blue</favouriteColour><favouriteColour>red</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v4/request/body/plain text that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Plain text that does not match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": { "Content-Type": "text/plain" },
          "body": {
            "contentType": "text/plain",
            "encoded": false,
            "content": "alligator named mary"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": { "Content-Type": "text/plain" },
          "body": {
            "contentType": "text/plain",
            "encoded": false,
            "content": "alligator named fred"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/plain text that does not match.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/plain text that does not match.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v4/request/body/array in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "blue",
                  "red"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array in different order.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array in different order.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with regex.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match with regex",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.alligator.name": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              },
              "$.alligator.favouriteColours[0]": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "red|blue"
                  }
                ]
              },
              "$.alligator.favouriteColours[1]": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "red|blue"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "feet": 4,
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 4,
                "name": "Harry",
                "favouriteColours": [
                  "blue",
                  "red"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected index with missing value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Unexpected favourite colour with empty value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour><favouriteColour></favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with missing value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with missing value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/array at top level xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML top level array matches",
        "expected": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><rogger dob=\"06/10/2015\" name=\"Rogger the Dogger\" id=\"1014753708\" timestamp=\"2015-06-10T20:41:37\"/><cat dob=\"06/10/2015\" name=\"Cat in the Hat\" id=\"8858030303\" timestamp=\"2015-06-10T20:41:37\"/></people>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><people><rogger dob=\"06/10/2015\" name=\"Rogger the Dogger\" id=\"1014753708\" timestamp=\"2015-06-10T20:41:37\"/><cat dob=\"06/10/2015\" name=\"Cat in the Hat\" id=\"8858030303\" timestamp=\"2015-06-10T20:41:37\"/></people>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array at top level xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array at top level xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/no body no content type xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML No body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body no content type xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body no content type xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn plain_text_that_matches() {
    println!("FILE: tests/spec_testcases/v4/request/body/plain text that matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Plain text that matches",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": { "Content-Type": "text/plain" },
          "body": {
            "contentType": "text/plain",
            "encoded": false,
            "content": "alligator named mary"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": { "Content-Type": "text/plain" },
          "body": {
            "contentType": "text/plain",
            "encoded": false,
            "content": "alligator named mary"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/plain text that matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/plain text that matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_with_regular_expression_in_element_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with regular expression in element xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Types and regular expressions match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals.alligator": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals.alligator['@phoneNumber']": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator phoneNumber=\"0415674567\"/></animals>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator phoneNumber=\"333\"/><alligator phoneNumber=\"983479823479283478923\"/></animals>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression in element xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression in element xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected key with empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Unexpected phone number with empty value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" phoneNumber=\"\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_index() {
    println!("FILE: tests/spec_testcases/v4/request/body/missing index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing index.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing index.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v4/request/body/different value found at index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "taupe"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at index.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at index.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn no_body() {
    println!("FILE: tests/spec_testcases/v4/request/body/no body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/no body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn not_empty_found_at_key_when_empty_expected_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/not empty found at key when empty expected xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Name should be empty",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/not empty found at key when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/not empty found at key when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/missing body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body, no content-type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "age": 3
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing body no content type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "feet": 4,
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 4,
                "name": "Mary",
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/number found in array when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  "2",
                  "3"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  "1",
                  2,
                  "3"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/number found in array when string expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_body() {
    println!("FILE: tests/spec_testcases/v4/request/body/empty body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": ""
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": ""
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected key with non-empty value xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Unexpected phone number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" phoneNumber=\"12345678\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with non-empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with non-empty value xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/missing key xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Missing key alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\" age=\"3\"></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator age=\"3\"></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing key xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing key xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/string found in array when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  1,
                  2,
                  3
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteNumbers": [
                  1,
                  "2",
                  3
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/string found in array when number expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected index with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "favouriteColours": [
                  "red",
                  "blue",
                  "taupe"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected index with not null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/number found at key when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": "4"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 4
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/number found at key when string expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_integers() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with integers.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Request match with integers",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.alligator.feet": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "[0-9]"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "feet": 4,
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 4,
                "name": "Mary",
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with integers.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with integers.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/not null found at key when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": null
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Fred"
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/not null found at key when null expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_body() {
    println!("FILE: tests/spec_testcases/v4/request/body/null body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "NULL body",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": null
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "content": null
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/null body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/null body.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/different value found at key xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Incorrect value at alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Fred\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at key xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at key xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn empty_found_at_key_where_not_empty_expected_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/empty found at key where not empty expected xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Name should not be empty",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"Mary\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator name=\"\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty found at key where not empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/empty found at key where not empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn matches_with_regex_with_bracket_notation_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with regex with bracket notation xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "XML Requests match with regex",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$['two']['@str']": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><two str=\"jildrdmxddnVzcQZfjCA\"/>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><two str=\"saldfhksajdhffdskkjh\"/>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex with bracket notation xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with regex with bracket notation xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v4/request/body/different value found at key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Fred"
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at key.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/different value found at key.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_regular_expression_in_element() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with regular expression in element.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].phoneNumber": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "0415674567"
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "333"
                },
                {
                  "phoneNumber": "983479823479283478923"
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression in element.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression in element.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_regular_expression_that_does_not_match_in_element() {
    println!("FILE: tests/spec_testcases/v4/request/body/array with regular expression that does not match in element.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Types and regular expressions match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].phoneNumber": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "0415674567"
                }
              ]
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "333"
                },
                {
                  "phoneNumber": "abc"
                }
              ]
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression that does not match in element.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array with regular expression that does not match in element.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected key with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected phone number with null value",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "phoneNumber": null
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v4/request/body/string found at key when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 4
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": "4"
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/string found at key when number expected.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn array_size_less_than_required_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/array size less than required xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Array must have at least 2 elements",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 2
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator name=\"Mary\"/></animals>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><animals><alligator name=\"Mary\"/></animals>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/array size less than required xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/array size less than required xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

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
    println!("FILE: tests/spec_testcases/v4/request/body/missing index xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Missing favorite colour",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour><favouriteColour>blue</favouriteColour></favouriteColours></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteColours><favouriteColour>red</favouriteColour></favouriteColours></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing index xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/missing index xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
#[cfg(feature = "xml")]
async fn not_empty_found_in_array_when_empty_expected_xml() {
    println!("FILE: tests/spec_testcases/v4/request/body/not empty found in array when empty expected xml.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "XML Favourite numbers expected to contain empty, but non-empty found",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber></favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/xml"},
          "body": {
            "contentType": "application/xml",
            "encoded": false,
            "content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><alligator><favouriteNumbers><favouriteNumber>1</favouriteNumber><favouriteNumber>2</favouriteNumber><favouriteNumber>3</favouriteNumber></favouriteNumbers></alligator>"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/not empty found in array when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/not empty found in array when empty expected xml.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v4/request/body/unexpected key with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected phone number",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary"
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "phoneNumber": "12345678"
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/unexpected key with not null value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_type() {
    println!("FILE: tests/spec_testcases/v4/request/body/matches with type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Requests match with same type",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "matchingRules": {
            "body": {
              "$.alligator.name": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.alligator.feet": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          },
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "name": "Mary",
                "feet": 4,
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {"Content-Type": "application/json"},
          "body": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "alligator": {
                "feet": 5,
                "name": "Harry the very hungry alligator with an extra foot",
                "favouriteColours": [
                  "red",
                  "blue"
                ]
              }
            }
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/body/matches with type.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().request.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_request(expected, actual, pact, &PactSpecification::V4).await.unwrap().mismatches();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
