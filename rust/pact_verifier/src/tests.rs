use std::collections::HashMap;
use std::env;
use std::panic::{catch_unwind, RefUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use async_trait::async_trait;
use expectest::prelude::*;
use maplit::hashmap;
use reqwest::Client;
use serde_json::{json, Value};

use pact_consumer::prelude::*;
use pact_models::Consumer;
use pact_models::pact::Pact;
use pact_models::provider_states::*;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;
use pact_mock_server::mock_server::MockServerConfig;

use crate::{
  NullRequestFilterExecutor,
  PactSource,
  ProviderInfo,
  ProviderStateExecutor,
  ProviderTransport,
  publish_result,
  PublishOptions,
  VerificationOptions
};
use crate::callback_executors::HttpRequestProviderStateExecutor;
use crate::pact_broker::Link;
use crate::verification_result::VerificationInteractionResult;
use crate::VERIFIER_VERSION;

use super::{execute_state_change, filter_consumers, filter_interaction, FilterInfo};

#[test]
fn if_no_interaction_filter_is_defined_returns_true() {
  let interaction = RequestResponseInteraction::default();
  expect!(filter_interaction(&interaction, &FilterInfo::None)).to(be_true());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: "bob".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("fred".to_string()))).to(be_false());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_true_if_the_description_does_match() {
  let interaction = RequestResponseInteraction { description: "bob".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("bob".to_string()))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_description() {
  let interaction = RequestResponseInteraction { description: "bobby".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("bob.*".to_string()))).to(be_true());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_false_if_the_state_does_not_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bob".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("fred".to_string()))).to(be_false());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_true_if_the_state_does_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bob".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("bob".to_string()))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_state() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("bob.*".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_is_empty_returns_false_if_the_interaction_state_is_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_is_empty_returns_true_if_the_interaction_state_is_not_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_must_match_both() {
  let interaction = RequestResponseInteraction { description: "freddy".to_string(), provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_provider_state_does_not_match() {
  let interaction = RequestResponseInteraction { description: "freddy".to_string(), provider_states: vec![ ProviderState::default(&"boddy".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: "frebby".to_string(), provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_both_do_not_match() {
  let interaction = RequestResponseInteraction { description: "joe".to_string(), provider_states: vec![ ProviderState::default(&"author".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_no_consumer_filter_is_defined_returns_true() {
  let consumers = vec![];
  let result = Err(anyhow!(""));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_false_if_the_consumer_name_does_not_match() {
  let consumers = vec!["fred".to_string(), "joe".to_string()];
  let result = Ok((
    Box::new(RequestResponsePact {
      consumer: Consumer { name: "bob".to_string() }, .. RequestResponsePact::default()
    }) as Box<dyn Pact + Send + Sync + RefUnwindSafe>,
    None,
    PactSource::Unknown,
    Duration::default()
  ));
  expect!(filter_consumers(&consumers, &result)).to(be_false());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_result_is_an_error() {
  let consumers = vec!["fred".to_string(), "joe".to_string()];
  let result = Err(anyhow!(""));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_consumer_name_does_match() {
  let consumers = vec!["fred".to_string(), "joe".to_string(), "bob".to_string()];
  let result = Ok((
    Box::new(RequestResponsePact {
      consumer: Consumer { name: "bob".to_string() }, .. RequestResponsePact::default()
    }) as Box<dyn Pact + Send + Sync + RefUnwindSafe>,
    None,
    PactSource::Unknown,
    Duration::default()
  ));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test_log::test(tokio::test)]
async fn test_state_change_with_parameters() {
  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request", "", |mut i| {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"params\":{\"A\":\"1\",\"B\":\"2\"},\"action\":\"setup\",\"state\":\"TestState\"}");
      i.response.status(200);
      i
    })
    .start_mock_server(None, None);

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{
        "A".to_string() => json!("1"),
        "B".to_string() => json!("2")
      }
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[test_log::test(tokio::test)]
async fn test_state_change_with_parameters_in_query() {
  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request with params in the query string", "", |mut i| {
      i.comment("testing state change with parameters in the query");
      i.test_name("test_state_change_with_parameters_in_query");
      i.request.method("POST");
      i.request.path("/");
      i.request.query_param("state", "TestState");
      i.request.query_param("action", "setup");
      i.request.query_param("A", "1");
      i.request.query_param("B", "2");
      i.response.status(200);
      i
    })
    .start_mock_server(None, None);

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{
        "A".to_string() => json!("1"),
        "B".to_string() => json!("2")
      }
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    state_change_body: false,
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();

  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[test_log::test(tokio::test)]
async fn test_state_change_returning_json_values() {
  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request which returns a map of values", "", |mut i| {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"action\":\"setup\",\"state\":\"TestState\",\"params\":{}}");
      i.response.status(200);
      i.response.header("Content-Type", "application/json");
      i.response.body("{\"a\": \"A\", \"b\": 100}");
      i
    })
    .start_mock_server(None, None);

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{}
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok().value(hashmap! {
    "a".into() => json!("A"),
    "b".into() => json!(100)
  }));
}

#[test_log::test]
fn publish_result_does_nothing_if_not_from_broker() {
  let server_response = catch_unwind(|| {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();

    runtime.block_on(async {
      let _server = PactBuilder::new("RustPactVerifier", "PactBroker")
        .interaction("publish results", "", |mut i| {
          i.request.method("POST");
          i.request.path("/");
          i.response.status(201);
          i
        })
        .start_mock_server(None, None);

      let options = super::PublishOptions {
        provider_version: None,
        build_url: None,
        provider_tags: vec![],
        .. super::PublishOptions::default()
      };
      super::publish_result(&vec![], &PactSource::File("/tmp/test".into()), &options, None).await;
    })
  });
  expect!(server_response).to(be_err());
}

#[test_log::test(tokio::test)]
async fn publish_successful_result_to_broker() {
  let server = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
    .interaction("publish results", "", |mut i| async move {
      i.request.method("POST");
      i.request.path("/path/to/pact/verification");
      i.request.json_body(json_pattern!({
        "providerApplicationVersion": "1",
        "success": true,
        "testResults": [
          { "interactionId": "1", "success": true }
        ],
        "verifiedBy": json!({
          "implementation": "Pact-Rust",
          "version": VERIFIER_VERSION
        })
      }));
      i.response.status(201);
      i
    })
    .await
    .start_mock_server(None, None);

  let options = super::PublishOptions {
    provider_version: Some("1".into()),
    .. super::PublishOptions::default()
  };

  let links = vec![
    Link {
      name: "pb:publish-verification-results".to_string(),
      href: Some(server.path("/path/to/pact/verification").to_string()),
      templated: false,
      title: None
    }
  ];
  
  let source = PactSource::BrokerUrl("Test".to_string(), server.url().to_string(), None, links.clone());
  publish_result(&[VerificationInteractionResult {
      interaction_id: Some("1".to_string()),
      interaction_key: None,
      description: "".to_string(),
      interaction_description: "".to_string(),
      result: Ok(()),
      pending: false,
      duration: Default::default(),
    }], &source, &options, None
  ).await;

  // Same publish but with dynamic configuration as pact source:
  let source = PactSource::BrokerWithDynamicConfiguration {
    provider_name: "Test".to_string(),
    broker_url: server.url().to_string(),
    enable_pending: false,
    include_wip_pacts_since: None,
    provider_tags: vec![],
    provider_branch: Some("branch".to_string()),
    selectors: vec![],
    auth: None,
    links
  };
  super::publish_result(&[VerificationInteractionResult {
      interaction_id: Some("1".to_string()),
      interaction_key: None,
      description: "".to_string(),
      interaction_description: "".to_string(),
      result: Ok(()),
      pending: false,
      duration: Default::default(),
    }], &source, &options, None
  ).await;
}

#[test]
fn is_pact_broker_source_test() {
  let result = super::is_pact_broker_source(&vec![]);
  expect!(result).to(be_false());

  let result = super::is_pact_broker_source(&vec![
    Link {
      name: "".to_string(),
      href: None,
      templated: false,
      title: None
    }
  ]);
  expect!(result).to(be_false());

  let result = super::is_pact_broker_source(&vec![
    Link {
      name: "pb:some_link".to_string(),
      href: None,
      templated: false,
      title: None
    }
  ]);
  expect!(result).to(be_false());

  let result = super::is_pact_broker_source(&vec![
    Link {
      name: "pb:publish-verification-results".to_string(),
      href: None,
      templated: false,
      title: Some("Publish verification results".to_string())
    }
  ]);
  expect!(result).to(be_true());

  let result = super::is_pact_broker_source(&vec![
    Link {
      name: "pb:some_link".to_string(),
      href: None,
      templated: false,
      title: None
    },
    Link {
      name: "pb:publish-verification-results".to_string(),
      href: None,
      templated: false,
      title: Some("Publish verification results".to_string())
    }
  ]);
  expect!(result).to(be_true());
}

#[test_log::test(tokio::test)]
async fn test_fetch_pact_from_url_with_links() {
  let path = "/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9";
  let server = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
  .interaction("a request for a Pact from a webhook", "", |mut i| async move {
      i.request.method("GET");
      i.request.path(path);
      i.response.status(200);
      i.response.header("content-type", "application/hal+json");
      i.response.body(json!({
        "consumer": {
          "name": "JVM Pact Broker Client"
        },
        "interactions": [],
        "metadata": {
          "pactSpecification": {
            "version": "3.0.0"
          }
        },
        "provider": {
          "name": "Pact Broker"
        },
        "_links": {
          "self": {
            "title": "Pact",
            "name": "Pact between JVM Pact Broker Client (4.3.9) and Pact Broker",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9"
          },
          "pb:consumer": {
            "title": "Consumer",
            "name": "JVM Pact Broker Client",
            "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client"
          },
          "pb:consumer-version": {
            "title": "Consumer version",
            "name": "4.3.9",
            "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9"
          },
          "pb:provider": {
            "title": "Provider",
            "name": "Pact Broker",
            "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker"
          },
          "pb:pact-version": {
            "title": "Pact content version permalink",
            "name": "4b6df5417cd7e999f13e1a32635268527bd20dbf",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf"
          },
          "pb:latest-pact-version": {
            "title": "Latest version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest"
          },
          "pb:all-pact-versions": {
            "title": "All versions of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/versions"
          },
          "pb:latest-untagged-pact-version": {
            "title": "Latest untagged version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest-untagged"
          },
          "pb:latest-tagged-pact-version": {
            "title": "Latest tagged version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest/{tag}",
            "templated": true
          },
          "pb:previous-distinct": {
            "title": "Previous distinct version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/previous-distinct"
          },
          "pb:diff-previous-distinct": {
            "title": "Diff with previous distinct version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/diff/previous-distinct"
          },
          "pb:diff": {
            "title": "Diff with another specified version of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/diff/pact-version/{pactVersion}",
            "templated": true
          },
          "pb:pact-webhooks": {
            "title": "Webhooks for the pact between JVM Pact Broker Client and Pact Broker",
            "href": "https://pact-foundation.pactflow.io/webhooks/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client"
          },
          "pb:consumer-webhooks": {
            "title": "Webhooks for all pacts with provider Pact Broker",
            "href": "https://pact-foundation.pactflow.io/webhooks/consumer/Pact%20Broker"
          },
          "pb:tag-prod-version": {
            "title": "PUT to this resource to tag this consumer version as 'production'",
            "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9/tags/prod"
          },
          "pb:tag-version": {
            "title": "PUT to this resource to tag this consumer version",
            "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9/tags/{tag}"
          },
          "pb:publish-verification-results": {
            "title": "Publish verification results",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/metadata/Y3Y9NTY4/verification-results"
          },
          "pb:latest-verification-results": {
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/verification-results/latest"
          },
          "pb:triggered-webhooks": {
            "title": "Webhooks triggered by the publication of this pact",
            "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/triggered-webhooks"
          },
          "pb:matrix-for-consumer-version": {
            "title": "View matrix rows for the consumer version to which this pact belongs",
            "href": "https://pact-foundation.pactflow.io/matrix?q[][pacticipant]=JVM+Pact+Broker+Client&q[][version]=4.3.9&latestby=cvpv"
          },
          "curies": [
            {
              "name": "pb",
              "href": "https://pact-foundation.pactflow.io/doc/{rel}?context=pact",
              "templated": true
            }
          ]
        }
      }).to_string());
      i
    })
    .await
    .start_mock_server(None, None);

  let url = server.url().join(path).unwrap();
  let provider = ProviderInfo::default();
  let result = super::fetch_pact(PactSource::URL(url.to_string(), None), &provider).await;

  let first_result = result.get(0).unwrap().as_ref();
  let (_, _, source, _) = &first_result.clone().unwrap();
  match source {
    PactSource::BrokerUrl(provider, url, auth, links) => {
      expect!(provider.clone()).to(be_equal_to("Pact Broker"));
      expect!(url.clone()).to(be_equal_to(url.to_string()));
      expect!(auth.clone()).to(be_none());
      expect!(links.iter()).to(have_count(21));
      expect!(links.iter().find(|link| link.name == "pb:publish-verification-results")).to(be_some());
    }
    _ => panic!("Expected a BrokerUrl source, got {}", source)
  }
}

#[test]
fn transport_base_url_test() {
  let transport = ProviderTransport::default();
  expect!(transport.base_url("HOST")).to(be_equal_to("http://HOST:8080"));

  let transport = ProviderTransport {
    transport: "https".to_string(),
    port: None,
    path: None,
    scheme: Some("https".to_string())
  };
  expect!(transport.base_url("HOST")).to(be_equal_to("https://HOST"));

  let transport = ProviderTransport {
    transport: "https".to_string(),
    port: None,
    path: Some("/a/b/c".to_string()),
    scheme: Some("https".to_string())
  };
  expect!(transport.base_url("HOST")).to(be_equal_to("https://HOST/a/b/c"));

  let transport = ProviderTransport {
    transport: "https".to_string(),
    port: Some(5678),
    path: None,
    scheme: Some("https".to_string())
  };
  expect!(transport.base_url("HOST")).to(be_equal_to("https://HOST:5678"));

  let transport = ProviderTransport {
    transport: "https".to_string(),
    port: Some(7765),
    path: Some("/a/b/c".to_string()),
    scheme: None
  };
  expect!(transport.base_url("HOST")).to(be_equal_to("http://HOST:7765/a/b/c"));
}

#[derive(Debug)]
struct DummyProviderStateExecutor;

#[async_trait]
impl ProviderStateExecutor for DummyProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    _provider_state: &ProviderState,
    _setup: bool,
    _client: Option<&Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    Ok(hashmap!{})
  }

  fn teardown(self: &Self) -> bool {
    return false
  }
}

#[test_log::test(tokio::test)]
async fn when_no_pacts_is_error_is_false_should_not_generate_error() {
  let server = PactBuilderAsync::new("RustPactVerifier", "PactBrokerNoPacts")
    .interaction("a request to the pact broker root", "", |mut i| async move {
      i.request
        .path("/")
        .header("Accept", "application/hal+json")
        .header("Accept", "application/json");
      i.response
        .header("Content-Type", "application/hal+json")
        .json_body(json_pattern!({
            "_links": {
                "pb:provider-pacts-for-verification": {
                  "href": like!("http://localhost/pacts/provider/{provider}/for-verification"),
                  "title": like!("Pact versions to be verified for the specified provider"),
                  "templated": like!(true)
                }
            }
        }));
      i
    })
    .await
    .interaction("a request to the pacts for verification endpoint", "", |mut i| async move {
      i.given("There are pacts to be verified");
      i.request
        .get()
        .path("/pacts/provider/sad_provider/for-verification")
        .header("Accept", "application/hal+json")
        .header("Accept", "application/json");
      i.response
        .header("Content-Type", "application/hal+json")
        .json_body(json_pattern!({
                "_links": {
                    "self": {
                      "href": like!("http://localhost/pacts/provider/sad_provider/for-verification"),
                      "title": like!("Pacts to be verified")
                    }
                }
            }));
      i
    })
    .await
    .interaction("a request for a providers pacts", "", |mut i| async move {
      i.given("There are no matching pacts in the pact broker");
      i.request
        .post()
        .path("/pacts/provider/sad_provider/for-verification");
      i.response
        .header("Content-Type", "application/hal+json")
        .json_body(json_pattern!({
          "_embedded": {
            "pacts": []
          }
        }));
      i
    })
    .await
    .start_mock_server(None, Some(MockServerConfig::with_keep_alive(true)));

  let provider = ProviderInfo {
    name: "sad_provider".to_string(),
    host: "127.0.0.1".to_string(),
    transports: vec![ ProviderTransport {
      transport: "HTTP".to_string(),
      port: None,
      path: None,
      scheme: Some("http".to_string())
    } ],
    .. ProviderInfo::default()
  };

  let pact_source = PactSource::BrokerWithDynamicConfiguration {
    provider_name: "sad_provider".to_string(),
    broker_url: server.url().to_string(),
    enable_pending: false,
    include_wip_pacts_since: None,
    provider_tags: vec![],
    provider_branch: None,
    selectors: vec![],
    auth: None,
    links: vec![]
  };
  let verification_options = VerificationOptions::<NullRequestFilterExecutor> {
    no_pacts_is_error: false,
    .. VerificationOptions::default()
  };
  let provider_states = Arc::new(DummyProviderStateExecutor{});

  let result = super::verify_provider_async(
    provider, vec![pact_source], FilterInfo::None, vec![],
    &verification_options, None, &provider_states, None
  ).await;

  let execution_result = result.unwrap();
  expect(execution_result.result).to(be_true());
  expect(execution_result.errors.iter()).to(be_empty());
}

#[test_log::test(tokio::test)]
async fn when_no_pacts_is_error_is_false_should_not_generate_error_if_it_is_404_error() {
  let server = PactBuilderAsync::new("RustPactVerifier", "PactBrokerError")
    .interaction("a request to the pact broker root", "", |mut i| async move {
      i.request
        .path("/")
        .header("Accept", "application/hal+json")
        .header("Accept", "application/json");
      i.response
        .header("Content-Type", "application/hal+json")
        .json_body(json_pattern!({
            "_links": {
                "pb:provider-pacts-for-verification": {
                  "href": like!("http://localhost/pacts/provider/{provider}/for-verification"),
                  "title": like!("Pact versions to be verified for the specified provider"),
                  "templated": like!(true)
                }
            }
        }));
      i
    })
    .await
    .interaction("a request to the pacts for verification endpoint", "", |mut i| async move {
      i.given("There are pacts to be verified");
      i.request
        .get()
        .path("/pacts/provider/sad_provider/for-verification")
        .header("Accept", "application/hal+json")
        .header("Accept", "application/json");
      i.response
        .header("Content-Type", "application/hal+json")
        .json_body(json_pattern!({
                "_links": {
                    "self": {
                      "href": like!("http://localhost/pacts/provider/sad_provider/for-verification"),
                      "title": like!("Pacts to be verified")
                    }
                }
            }));
      i
    })
    .await
    .interaction("a request for a providers pacts", "", |mut i| async move {
      i.given("Request to fetch pacts is gone");
      i.request
        .post()
        .path("/pacts/provider/sad_provider/for-verification");
      i.response.status(404);
      i
    })
    .await
    .start_mock_server(None, None);

  let provider = ProviderInfo {
    name: "sad_provider".to_string(),
    host: "127.0.0.1".to_string(),
    transports: vec![ ProviderTransport {
      transport: "HTTP".to_string(),
      port: None,
      path: None,
      scheme: Some("http".to_string())
    } ],
    .. ProviderInfo::default()
  };

  let pact_source = PactSource::BrokerWithDynamicConfiguration {
    provider_name: "sad_provider".to_string(),
    broker_url: server.url().to_string(),
    enable_pending: false,
    include_wip_pacts_since: None,
    provider_tags: vec![],
    provider_branch: None,
    selectors: vec![],
    auth: None,
    links: vec![]
  };
  let verification_options = VerificationOptions::<NullRequestFilterExecutor> {
    no_pacts_is_error: false,
    .. VerificationOptions::default()
  };
  let provider_states = Arc::new(DummyProviderStateExecutor{});

  let result = super::verify_provider_async(
    provider, vec![pact_source], FilterInfo::None, vec![],
    &verification_options, None, &provider_states, None
  ).await;

  let execution_result = result.unwrap();
  expect(execution_result.result).to(be_true());
  expect(execution_result.errors.iter()).to(be_empty());
}

#[test_log::test(tokio::test)]
async fn test_publish_results_from_url_source_with_provider_branch() {
  let path = "/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9";
  let server = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
      .interaction("a request for a Pact from a webhook", "", |mut i| async move {
        i.request.method("GET");
        i.request.path(path);
        i.response.status(200);
        i.response.header("content-type", "application/hal+json");
        i.response.json_body(json!({
          "consumer": {
            "name": "JVM Pact Broker Client"
          },
          "interactions": [],
          "metadata": {
            "pactSpecification": {
              "version": "3.0.0"
            }
          },
          "provider": {
            "name": "Pact Broker"
          },
          "_links": {
            "self": {
              "title": "Pact",
              "name": "Pact between JVM Pact Broker Client (4.3.9) and Pact Broker",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9"
            },
            "pb:consumer": {
              "title": "Consumer",
              "name": "JVM Pact Broker Client",
              "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client"
            },
            "pb:consumer-version": {
              "title": "Consumer version",
              "name": "4.3.9",
              "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9"
            },
            "pb:provider": {
              "title": "Provider",
              "name": "Pact Broker",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker"
            },
            "pb:pact-version": {
              "title": "Pact content version permalink",
              "name": "4b6df5417cd7e999f13e1a32635268527bd20dbf",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf"
            },
            "pb:latest-pact-version": {
              "title": "Latest version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest"
            },
            "pb:all-pact-versions": {
              "title": "All versions of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/versions"
            },
            "pb:latest-untagged-pact-version": {
              "title": "Latest untagged version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest-untagged"
            },
            "pb:latest-tagged-pact-version": {
              "title": "Latest tagged version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/latest/{tag}",
              "templated": true
            },
            "pb:previous-distinct": {
              "title": "Previous distinct version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/previous-distinct"
            },
            "pb:diff-previous-distinct": {
              "title": "Diff with previous distinct version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/diff/previous-distinct"
            },
            "pb:diff": {
              "title": "Diff with another specified version of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/diff/pact-version/{pactVersion}",
              "templated": true
            },
            "pb:pact-webhooks": {
              "title": "Webhooks for the pact between JVM Pact Broker Client and Pact Broker",
              "href": "https://pact-foundation.pactflow.io/webhooks/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client"
            },
            "pb:consumer-webhooks": {
              "title": "Webhooks for all pacts with provider Pact Broker",
              "href": "https://pact-foundation.pactflow.io/webhooks/consumer/Pact%20Broker"
            },
            "pb:tag-prod-version": {
              "title": "PUT to this resource to tag this consumer version as 'production'",
              "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9/tags/prod"
            },
            "pb:tag-version": {
              "title": "PUT to this resource to tag this consumer version",
              "href": "https://pact-foundation.pactflow.io/pacticipants/JVM%20Pact%20Broker%20Client/versions/4.3.9/tags/{tag}"
            },
            "pb:publish-verification-results": {
              "title": "Publish verification results",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/metadata/Y3Y9NTY4/verification-results"
            },
            "pb:latest-verification-results": {
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/verification-results/latest"
            },
            "pb:triggered-webhooks": {
              "title": "Webhooks triggered by the publication of this pact",
              "href": "https://pact-foundation.pactflow.io/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/version/4.3.9/triggered-webhooks"
            },
            "pb:matrix-for-consumer-version": {
              "title": "View matrix rows for the consumer version to which this pact belongs",
              "href": "https://pact-foundation.pactflow.io/matrix?q[][pacticipant]=JVM+Pact+Broker+Client&q[][version]=4.3.9&latestby=cvpv"
            },
            "curies": [
              {
                "name": "pb",
                "href": "https://pact-foundation.pactflow.io/doc/{rel}?context=pact",
                "templated": true
              }
            ]
          }
        }));
        i
      })
      .await
      .interaction("a request for a Pact provider", "", |mut i| async move {
        i.request.method("GET");
        i.request.path("/pacticipants/Pact%20Broker");
        i.response.status(200);
        i.response.header("content-type", "application/hal+json");
        i.response.json_body(json!({
          "name": "Pact Broker",
          "displayName": "Pact Broker",
          "updatedAt": "2019-05-04T06:20:15+00:00",
          "createdAt": "2019-05-04T06:20:15+00:00",
          "_embedded": {
            "labels": []
          },
          "_links": {
            "self": {
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker"
            },
            "pb:versions": {
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions"
            },
            "pb:version": {
              "title": "Get, create or delete a pacticipant version",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions/{version}",
              "templated": true
            },
            "pb:version-tag": {
              "title": "Get, create or delete a tag for a version of Pact Broker",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions/{version}/tags/{tag}",
              "templated": true
            },
            "pb:branch-version": {
              "title": "Get or add/create a version for a branch of Pact Broker",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/branches/{branch}/versions/{version}",
              "templated": true
            },
            "pb:label": {
              "title": "Get, create or delete a label for Pact Broker",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/labels/{label}",
              "templated": true
            },
            "versions": {
              "title": "Deprecated - use pb:versions",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions"
            },
            "pb:can-i-deploy-badge": {
              "title": "Can I Deploy Pact Broker badge",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/latest-version/{tag}/can-i-deploy/to/{environmentTag}/badge",
              "templated": true
            },
            "pb:can-i-deploy-branch-to-environment-badge": {
              "title": "Can I Deploy Pact Broker from branch to environment badge",
              "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/branches/{branch}/latest-version/can-i-deploy/to-environment/{environment}/badge",
              "templated": true
            },
            "curies": [
              {
                "name": "pb",
                "href": "https://pact-foundation.pactflow.io/doc/{rel}?context=pacticipant",
                "templated": true
              }
            ]
          }
        }));
        i
      })
      .await
      .interaction("a request to publish provider branch", "", |mut i| async move {
        i.request.method("PUT");
        i.request.path("/pacticipants/Pact%20Broker/branches/feat%2F1234/versions/1.2.3");
        i.request.json_body(json!({}));

        i.response
          .header("content-type", "application/json")
          .json_body("{}")
          .status(200);
        i
      })
      .await
      .interaction("a request to publish verification results", "", |mut i| async move {
        i.request.method("POST");
        i.request.path("/pacts/provider/Pact%20Broker/consumer/JVM%20Pact%20Broker%20Client/pact-version/4b6df5417cd7e999f13e1a32635268527bd20dbf/metadata/Y3Y9NTY4/verification-results");
        i.request.json_body(json_pattern!({
          "providerApplicationVersion": "1.2.3",
          "success": true,
          "testResults": [],
          "verifiedBy": { "implementation": "Pact-Rust", "version": like!("0.4.5") }
        }));

        i.response
          .status(200)
          .header("content-type", "application/json")
          .json_body("{}");
        i
      })
      .await
      .start_mock_server(None, None);

  let url = server.url().join(path).unwrap();
  let provider = ProviderInfo::default();
  let pact_result = super::fetch_pact(PactSource::URL(url.to_string(), None), &provider).await;

  let first_result = pact_result.get(0).unwrap().as_ref();
  let (_, _, source, _) = &first_result.clone().unwrap();
  let options = PublishOptions {
    provider_version: Some("1.2.3".to_string()),
    build_url: None,
    provider_tags: vec![],
    provider_branch: Some("feat/1234".to_string())
  };
  let verification_result = vec![];

  publish_result(&verification_result, &source, &options, None).await;
}

#[test_log::test(tokio::test)]
async fn fetch_pact_from_dir_filters_by_provider_name() {
  let provider = ProviderInfo {
    name: "test_provider".to_string(),
    .. ProviderInfo::default()
  };
  let pacts_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/pacts");
  let result = super::fetch_pact(PactSource::Dir(pacts_path.to_string_lossy().to_string()), &provider).await;
  expect!(result.len()).to(be_equal_to(1));
  let first_result = result.first().unwrap().as_ref();
  let (pact, _, _, _) = first_result.unwrap();
  expect!(pact.provider().name).to(be_equal_to(provider.name));
}

// Issue #441
#[test_log::test(tokio::test)]
async fn support_passing_provider_state_params_to_provider_state_generator() {
  let server = PactBuilderAsync::new("RustPactVerifier", "441Provider")
    .interaction("a request say hello to John", "", |mut i| async move {
      i.request
        .path("/api/hello/John");
      i.response
        .header("content-type", "application/json")
        .json_body(json_pattern!({
          "name": "John"
        }));
      i
    })
    .await
    .start_mock_server(None, None);

  #[allow(deprecated)]
  let provider = ProviderInfo {
    name: "provider_states_values".to_string(),
    host: server.url().host_str().unwrap().to_string(),
    port: Some(server.url().port().unwrap()),
    transports: vec![
      ProviderTransport {
        transport: "HTTP".to_string(),
        port: Some(server.url().port().unwrap()),
        path: None,
        scheme: Some("http".to_string())
      }
    ],
    .. ProviderInfo::default()
  };

  let verification_options = VerificationOptions::<NullRequestFilterExecutor> {
    no_pacts_is_error: false,
    .. VerificationOptions::default()
  };
  let provider_states = Arc::new(DummyProviderStateExecutor{});

  let pact = RequestResponsePact::from_json("test", &json!({
    "consumer": {
      "name": "SomeConsumer"
    },
    "interactions": [
      {
        "description": "Hello John",
        "providerStates": [
          {
            "name": "User exists",
            "params": {
              "name": "John"
            }
          }
        ],
        "request": {
          "generators": {
            "path": {
              "dataType": "STRING",
              "expression": "/api/hello/${name}",
              "type": "ProviderState"
            }
          },
          "method": "GET",
          "path": "/api/hello/James"
        },
        "response": {
          "body": {
            "name": "John"
          },
          "headers": {
            "Content-Type": "application/json"
          },
          "status": 200
        }
      }
    ],
    "metadata": {
      "pact-jvm": {
        "version": "4.6.7"
      },
      "pactSpecification": {
        "version": "3.0.0"
      }
    },
    "provider": {
      "name": "SomeProvider"
    }
  })).unwrap();
  let interaction = pact.interactions.first().unwrap();

  let result = super::verify_interaction(&provider, interaction, &pact.boxed(), &verification_options, &provider_states).await;
  expect!(result).to(be_ok());
}
