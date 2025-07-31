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
async fn order_of_comma_separated_header_values_different() {
    println!("FILE: tests/spec_testcases/v3/response/headers/order of comma separated header values different.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Comma separated headers out of order, order can matter http://tools.ietf.org/html/rfc2616",
        "expected" : {
          "headers": {
            "Accept": "alligators, hippos"
          }
        },
        "actual": {
          "headers": {
            "Accept": "hippos, alligators"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/order of comma separated header values different.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/order of comma separated header values different.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn header_name_is_different_case() {
    println!("FILE: tests/spec_testcases/v3/response/headers/header name is different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Header name is case insensitive",
        "expected" : {
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "headers": {
            "ACCEPT": "alligators"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/header name is different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/header name is different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn unexpected_header_found() {
    println!("FILE: tests/spec_testcases/v3/response/headers/unexpected header found.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Extra headers allowed",
        "expected" : {
          "headers": {}
        },
        "actual": {
          "headers": {
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/unexpected header found.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/unexpected header found.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn header_value_is_different_case() {
    println!("FILE: tests/spec_testcases/v3/response/headers/header value is different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers values are case sensitive",
        "expected" : {
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "headers": {
            "Accept": "Alligators"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/header value is different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/header value is different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn content_type_parameters_do_not_match() {
    println!("FILE: tests/spec_testcases/v3/response/headers/content type parameters do not match.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers don't match when the parameters are different",
        "expected" : {
          "headers": {
            "Content-Type": "application/json; charset=UTF-16"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "application/json; charset=UTF-8"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/content type parameters do not match.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/content type parameters do not match.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn whitespace_after_comma_different() {
    println!("FILE: tests/spec_testcases/v3/response/headers/whitespace after comma different.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Whitespace between comma separated headers does not matter",
        "expected" : {
          "headers": {
            "Accept": "alligators,hippos"
          }
        },
        "actual": {
          "headers": {
            "Accept": "alligators, hippos"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/whitespace after comma different.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/whitespace after comma different.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_content_type_with_charset() {
    println!("FILE: tests/spec_testcases/v3/response/headers/matches content type with charset.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match when the actual includes additional parameters",
        "expected" : {
          "headers": {
            "Content-Type": "application/json"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "application/json; charset=UTF-8"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with charset.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with charset.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v3/response/headers/matches with regex.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match with regex",
        "expected" : {
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          },
          "matchingRules": {
            "header": {
              "Accept": {
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
          "headers": {
            "Content-Type": "hippos",
            "Accept": "godzilla"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches with regex.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches with regex.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches() {
    println!("FILE: tests/spec_testcases/v3/response/headers/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match",
        "expected" : {
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "hippos",
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_content_type_with_charset_with_different_case() {
    println!("FILE: tests/spec_testcases/v3/response/headers/matches content type with charset with different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Content-Type and Accept Headers match when the charset differs in case",
        "expected" : {
          "headers": {
            "Accept": "application/json;charset=utf-8",
            "Content-Type": "application/json;charset=utf-8"
          }
        },
        "actual": {
          "headers": {
            "Accept": "application/json; charset=UTF-8",
            "Content-Type": "application/json; charset=UTF-8"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with charset with different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with charset with different case.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn empty_headers() {
    println!("FILE: tests/spec_testcases/v3/response/headers/empty headers.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
        {
        "match": true,
        "comment": "Empty headers match",
        "expected" : {
          "headers": {}
        },
        "actual": {
          "headers": {}
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/empty headers.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/empty headers.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test_log::test(tokio::test)]
async fn matches_content_type_with_parameters_in_different_order() {
    println!("FILE: tests/spec_testcases/v3/response/headers/matches content type with parameters in different order.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match when the content type parameters are in a different order",
        "expected" : {
          "headers": {
            "Content-Type": "Text/x-Okie; charset=iso-8859-1;\n    declaration=\"<950118.AEB0@XIson.com>\""
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "Text/x-Okie; declaration=\"<950118.AEB0@XIson.com>\";\n    charset=iso-8859-1"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with parameters in different order.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.as_request_response().unwrap().response.body.display_string());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "response": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v3/response/headers/matches content type with parameters in different order.json", &interaction_json, &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.as_request_response().unwrap().response.body.display_string());
    let pact_match = pact.get("match").unwrap();

    #[cfg(feature = "plugins")] pact_matching::matchingrules::configure_core_catalogue();
    let pact = RequestResponsePact { interactions: vec![ expected.as_request_response().unwrap_or_default() ], .. RequestResponsePact::default() }.boxed();
    let result = match_interaction_response(expected, actual, pact, &PactSpecification::V3).await.unwrap();

    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
