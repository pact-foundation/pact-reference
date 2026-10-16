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
use pact_models::interaction::{Interaction, message_interaction_from_json};
#[allow(unused_imports)]
use pact_matching::match_interaction;
#[allow(unused_imports)]
use pact_models::prelude::{MessagePact, Pact};

#[test_log::test(tokio::test)]
async fn null_found_at_key_where_not_null_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/null found at key where not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should not be null",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/null found at key where not null expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/null found at key where not null expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_at_least_one_element_matching_by_example() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with at least one element matching by example.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with at least one element matching by example.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with at least one element matching by example.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn no_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v4/message/body/no body no content type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "No body, no content-type",
        "expected": {},
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/no body no content type.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/no body no content type.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_at_top_level() {
    println!("FILE: tests/spec_testcases/v4/message/body/array at top level.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "top level array matches",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array at top level.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array at top level.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_nested_array_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with nested array that does not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Nested arrays do not match, age is wrong type",
        "expected": {
          "contents": {
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
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with nested array that does not match.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with nested array that does not match.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_at_least_one_element_not_matching_example_type() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with at least one element not matching example type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Wrong type for name key",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with at least one element not matching example type.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with at least one element not matching example type.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_key() {
    println!("FILE: tests/spec_testcases/v4/message/body/missing key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/missing key.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/missing key.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_nested_array_that_matches() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with nested array that matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Nested arrays match",
        "expected": {
          "contents": {
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
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with nested array that matches.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with nested array that matches.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v4/message/body/unexpected index with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected index with null value.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected index with null value.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/null found in array when not null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a null",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/null found in array when not null expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/null found in array when not null expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_size_less_than_required() {
    println!("FILE: tests/spec_testcases/v4/message/body/array size less than required.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Empty array",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "name": "Fred"
                }
              ]
            }
          },
          "matchingRules": {
            "content": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": []
            }
          }
        }
      }
    "#).unwrap();

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array size less than required.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array size less than required.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex_with_bracket_notation() {
    println!("FILE: tests/spec_testcases/v4/message/body/matches with regex with bracket notation.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with regex with bracket notation",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "2": {
                "str": "jildrdmxddnVzcQZfjCA"
              }
            }
          },
          "matchingRules": {
            "content": {
              "$['2'].str": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with regex with bracket notation.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with regex with bracket notation.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/not null found in array when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to contain null, but not null found",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/not null found in array when null expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/not null found in array when null expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v4/message/body/array in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array in different order.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array in different order.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v4/message/body/matches with regex.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with regex",
        "expected": {
          "contents": {
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
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with regex.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with regex.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn missing_index() {
    println!("FILE: tests/spec_testcases/v4/message/body/missing index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/missing index.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/missing index.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v4/message/body/different value found at index.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/different value found at index.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/different value found at index.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn no_body() {
    println!("FILE: tests/spec_testcases/v4/message/body/no body.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected": {
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/no body.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/no body.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v4/message/body/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/number found in array when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a number",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/number found in array when string expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/number found in array when string expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/string found in array when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/string found in array when number expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/string found in array when number expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v4/message/body/unexpected index with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected index with not null value.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected index with not null value.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/number found at key when string expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/number found at key when string expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/number found at key when string expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/not null found at key when null expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/not null found at key when null expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/not null found at key when null expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v4/message/body/different value found at key.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/different value found at key.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/different value found at key.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_regular_expression_in_element() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with regular expression in element.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "0415674567"
                }
              ]
            }
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with regular expression in element.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with regular expression in element.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn array_with_regular_expression_that_does_not_match_in_element() {
    println!("FILE: tests/spec_testcases/v4/message/body/array with regular expression that does not match in element.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Types and regular expressions don't match",
        "expected": {
          "contents": {
            "contentType": "application/json",
            "encoded": false,
            "content": {
              "animals": [
                {
                  "phoneNumber": "0415674567"
                }
              ]
            }
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with regular expression that does not match in element.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/array with regular expression that does not match in element.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v4/message/body/unexpected key with null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number with null value",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected key with null value.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected key with null value.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v4/message/body/string found at key when number expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/string found at key when number expected.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/string found at key when number expected.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v4/message/body/unexpected key with not null value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number",
        "expected": {
          "contents": {
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
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected key with not null value.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/unexpected key with not null value.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_type() {
    println!("FILE: tests/spec_testcases/v4/message/body/matches with type.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with same type",
        "expected": {
          "contents": {
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
          },
          "matchingRules": {
            "content": {
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
          }
        },
        "actual": {
          "contents": {
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

    let expected_json = pact.get_mut("expected").unwrap();
    let interaction_json = expected_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let expected = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with type.json", &expected_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_message().unwrap().contents.display_string());
    let actual_json = pact.get_mut("actual").unwrap();
    let interaction_json = actual_json.as_object_mut().unwrap();
    interaction_json.insert("type".to_string(), serde_json::json!("Asynchronous/Messages"));
    let actual = message_interaction_from_json("tests/spec_testcases/v4/message/body/matches with type.json", &actual_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_message().unwrap().contents.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = MessagePact { messages: vec![ expected.as_message().unwrap_or_default() ], .. MessagePact::default() }.boxed();
    let result = match_interaction(expected, actual, pact, &PactSpecification::V4).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
