use expectest::prelude::*;
use maplit::hashmap;
use serde_json::json;

use crate::{Consumer, generators, HttpStatus, matchingrules, PactSpecification, Provider};
use crate::bodies::OptionalBody;
use crate::generators::{Generator, Generators, generators_from_json, GeneratorCategory};
use crate::matchingrules::{matchers_from_json, MatchingRule, MatchingRules, Category, MatchingRuleCategory, RuleList, RuleLogic};
use crate::pact::Pact;
use crate::provider_states::ProviderState;
use crate::v4::http_parts::{HttpRequest, HttpResponse};
use crate::v4::pact::V4Pact;
use crate::v4::synch_http::SynchronousHttp;
use crate::path_exp::DocPath;
use crate::v4::interaction::V4Interaction;

#[test]
fn http_status_code_from_json() {
  expect!(HttpStatus::from_json(&json!({}))).to(be_err());
  expect!(HttpStatus::from_json(&json!("success"))).to(be_ok().value(HttpStatus::Success));
  expect!(HttpStatus::from_json(&json!("info"))).to(be_ok().value(HttpStatus::Information));
  expect!(HttpStatus::from_json(&json!("redirect"))).to(be_ok().value(HttpStatus::Redirect));
  expect!(HttpStatus::from_json(&json!("clientError"))).to(be_ok().value(HttpStatus::ClientError));
  expect!(HttpStatus::from_json(&json!("serverError"))).to(be_ok().value(HttpStatus::ServerError));
  expect!(HttpStatus::from_json(&json!("nonError"))).to(be_ok().value(HttpStatus::NonError));
  expect!(HttpStatus::from_json(&json!([200, 201, 204]))).to(be_ok().value(HttpStatus::StatusCodes(vec![200, 201, 204])));
}

#[test]
fn pact_spec_from_string() {
  expect!(PactSpecification::from("")).to(be_equal_to(PactSpecification::Unknown));
  expect!(PactSpecification::from("V1")).to(be_equal_to(PactSpecification::V1));
  expect!(PactSpecification::from("V1.1")).to(be_equal_to(PactSpecification::V1_1));
  expect!(PactSpecification::from("V2")).to(be_equal_to(PactSpecification::V2));
  expect!(PactSpecification::from("V3")).to(be_equal_to(PactSpecification::V3));
  expect!(PactSpecification::from("V4")).to(be_equal_to(PactSpecification::V4));
  expect!(PactSpecification::from("v2")).to(be_equal_to(PactSpecification::V2));
  expect!(PactSpecification::from("xxaasa")).to(be_equal_to(PactSpecification::Unknown));

  expect!(PactSpecification::from("V3".to_string())).to(be_equal_to(PactSpecification::V3));

  let _: PactSpecification = "v1".into();
}

#[test]
fn matchers_from_json_handles_missing_matchers() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_empty_matchers() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {}
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_matcher_with_no_matching_rules() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_correctly() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
            }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex("\\d+".to_string()) ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_from_deprecated_name() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "deprecatedName": {
              "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
              }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex(r#"\d+"#.to_string()) ]
        }
    }));
}

#[test]
fn generators_from_json_handles_missing_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_empty_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {}
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_generator_with_no_rules() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_ignores_invalid_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.invalid": {
                  "type": 100
                },
                "$.other": null
            },
            "invalid": {
                "path": "path"
            },
            "more_invalid": 100
          }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_carries_unknown_generator_types_through() {
  // An unknown type is not necessarily invalid - it may be provided by a plugin - so it is loaded
  // as a plugin generator and resolved against the catalogue when it is applied
  let json: serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "generators": {
            "body": {
                "$.card.number": {
                  "type": "creditcard",
                  "brand": "visa"
                }
            }
          }
      }
     "#).unwrap();
  let generators = generators_from_json(&json).unwrap();

  expect!(generators).to(be_equal_to(generators! {
    "BODY" => {
      "$.card.number" => Generator::Plugin {
        name: "creditcard".to_string(),
        values: serde_json::json!({ "brand": "visa" })
      }
    }
  }));
}

#[test]
fn generators_from_json_loads_generators_correctly() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
        "path": "/",
        "query": "",
        "headers": {},
        "generators": {
          "body": {
              "$.*.path": {
                  "type": "RandomInt",
                  "min": 1,
                  "max": 10
              }
          },
          "path": {
            "type": "RandomString"
          }
        }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(generators!{
        "BODY" => {
            "$.*.path" => Generator::RandomInt(1, 10)
        },
        "PATH" => { "" => Generator::RandomString(10) }
    }));
}

// Issue https://github.com/pact-foundation/pact-js-core/issues/400
#[test]
fn write_pact_file_with_provider_state_generator_test() {
  let pact = V4Pact {
    consumer: Consumer { name: "TransactionService".to_string() },
    provider: Provider { name: "AccountService".to_string() },
    interactions: vec![
      SynchronousHttp {
        id: None,
        key: None,
        description: "a request to get the plain data".to_string(),
        provider_states: vec![
          ProviderState {
            name: "set id".to_string(),
            params: hashmap!{ "id".to_string() => json!("42")}
          }
        ],
        request: HttpRequest {
          method: "GET".to_string(),
          path: "/data/42".to_string(),
          query: None,
          headers: None,
          body: OptionalBody::Missing,
          matching_rules: MatchingRules {
            rules: hashmap!{
              Category::PATH => MatchingRuleCategory {
                name: Category::PATH,
                rules: hashmap!{ DocPath::root() => RuleList {
                  rules: vec![MatchingRule::Type],
                  rule_logic: RuleLogic::And,
                  cascaded: false
                }}
              }
            }
          },
          generators: Generators {
            categories: hashmap!{
              GeneratorCategory::PATH => hashmap!{
                DocPath::root() => Generator::ProviderStateGenerator("/data/${id}".to_string(), None)
              }
            }
          }
        },
        response: HttpResponse {
          status: 200,
          headers: Some(hashmap!{"Content-Type".to_string() => vec!["text/plain; charset=utf-8".to_string()]}),
          body: OptionalBody::from("data: testData, id: 42"),
          matching_rules: MatchingRules {
            rules: hashmap!{
              Category::HEADER => MatchingRuleCategory {
                name: Category::HEADER,
                rules: hashmap!{}
              }
            }
          },
          generators: Generators { categories: hashmap!{} }
        },
        .. SynchronousHttp::default()
      }.boxed_v4()],
      .. V4Pact::default()
    };

  let json = pact.to_json(PactSpecification::V3).unwrap();
  let interaction = json.get("interactions").unwrap().as_array().unwrap().get(0).unwrap();
  let request = interaction.get("request").unwrap();
  let generators = request.get("generators").unwrap();
  expect!(generators.to_string()).to_not(be_equal_to("{}"));
}
