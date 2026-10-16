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
async fn same_parameter_different_values() {
    println!("FILE: tests/spec_testcases/v4/request/query/same parameter different values.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - animals are alligator, hippo versus alligator, elephant",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter different values.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter different values.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn unexpected_param() {
    println!("FILE: tests/spec_testcases/v4/request/query/unexpected param.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - elephant is not expected",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"],
            "elephant": ["unexpected"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/unexpected param.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/unexpected param.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn same_parameter_multiple_times() {
    println!("FILE: tests/spec_testcases/v4/request/query/same parameter multiple times.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same - multiple values are in same order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo", "elephant"],
            "hippo": ["Fred"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "hippo": ["Fred"],
            "animal": ["alligator", "hippo", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter multiple times.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter multiple times.json", &interaction_json, &PactSpecification::V4).unwrap();
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
    println!("FILE: tests/spec_testcases/v4/request/query/matches with regex.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries match with regex",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {},
          "matchingRules": {
            "query": {
              "hippo": {
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
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches with regex.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches with regex.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn same_parameter_multiple_times_in_different_order() {
    println!("FILE: tests/spec_testcases/v4/request/query/same parameter multiple times in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - values are in different order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo", "elephant"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["hippo", "alligator", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter multiple times in different order.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/same parameter multiple times in different order.json", &interaction_json, &PactSpecification::V4).unwrap();
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
    println!("FILE: tests/spec_testcases/v4/request/query/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn matches_with_equals_in_the_query_value() {
    println!("FILE: tests/spec_testcases/v4/request/query/matches with equals in the query value.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are equivalent",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "options": ["delete.topic.enable=true"],
            "broker": ["1"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "options": ["delete.topic.enable=true"],
            "broker": ["1"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches with equals in the query value.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/matches with equals in the query value.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn different_params() {
    println!("FILE: tests/spec_testcases/v4/request/query/different params.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - hippo is Fred instead of John",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/different params.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/different params.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn missing_params() {
    println!("FILE: tests/spec_testcases/v4/request/query/missing params.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - elephant is missing",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"],
            "elephant": ["missing"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/missing params.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/missing params.json", &interaction_json, &PactSpecification::V4).unwrap();
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
async fn different_order() {
    println!("FILE: tests/spec_testcases/v4/request/query/different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same but in different key order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "hippo": ["John"],
            "alligator": ["Mary"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/query/different order.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().request.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/query/different order.json", &interaction_json, &PactSpecification::V4).unwrap();
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
