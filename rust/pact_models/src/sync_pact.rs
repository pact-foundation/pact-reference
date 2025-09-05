//! Synchronous HTTP Request/Response Pact

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::panic::RefUnwindSafe;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use maplit::{btreemap, hashset};
use serde_json::{json, Map, Value};
use tracing::warn;

use crate::{Consumer, PactSpecification, Provider};
#[cfg(not(target_family = "wasm"))] use crate::file_utils::with_read_lock;
#[cfg(not(target_family = "wasm"))] use crate::http_utils::{self, HttpAuth};
use crate::interaction::{Interaction, PactConflict, parse_interactions};
use crate::iterator_utils::CartesianProductIterator;
use crate::message_pact::MessagePact;
use crate::pact::{determine_spec_version, metadata_schema, Pact, parse_meta_data, ReadWritePact, verify_metadata};
use crate::PACT_RUST_VERSION;
use crate::plugins::PluginData;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::pact::V4Pact;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestResponsePact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of interactions between the consumer and provider.
  pub interactions: Vec<RequestResponseInteraction>,
  /// Metadata associated with this pact file.
  pub metadata: BTreeMap<String, BTreeMap<String, String>>,
  /// Specification version of this pact
  pub specification_version: PactSpecification
}

impl Pact for RequestResponsePact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<Box<dyn Interaction + Send + Sync + RefUnwindSafe>> {
    self.interactions.iter().map(|i| i.boxed()).collect()
  }

  fn interactions_mut(&mut self) -> Vec<&mut (dyn Interaction + Send + Sync)> {
    self.interactions.iter_mut().map(|m| m as &mut (dyn Interaction + Send + Sync)).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.clone()
  }

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V4 => self.as_v4_pact()?.to_json(pact_spec),
      _ => Ok(json!({
          "consumer": self.consumer.to_json(),
          "provider": self.provider.to_json(),
          "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json(&pact_spec)).collect()),
          "metadata": self.metadata_to_json(&pact_spec)
      }))
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    Ok(self.clone())
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    Err(anyhow!("Can't convert a Request/response Pact to a different type"))
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_v4())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    Ok(V4Pact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      interactions,
      metadata: self.metadata.iter().map(|(k, v)| (k.clone(), json!(v))).collect(),
      .. V4Pact::default()
    })
  }

  fn specification_version(&self) -> PactSpecification {
    self.specification_version.clone()
  }

  fn boxed(&self) -> Box<dyn Pact + Send + Sync + RefUnwindSafe> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Pact + Send + Sync + RefUnwindSafe> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync + RefUnwindSafe>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()> {
    match interaction.as_request_response() {
      None => Err(anyhow!("Can only add request/response interactions to this Pact")),
      Some(interaction) => {
        self.interactions.push(interaction);
        Ok(())
      }
    }
  }

  fn requires_plugins(&self) -> bool {
    false
  }

  fn plugin_data(&self) -> Vec<PluginData> {
    Vec::default()
  }

  fn is_v4(&self) -> bool {
    false
  }

  fn add_plugin(
    &mut self,
    _name: &str,
    _version: &str,
    _plugin_data: Option<HashMap<String, Value>>
  ) -> anyhow::Result<()> {
    Err(anyhow!("Plugins can only be used with V4 format pacts"))
  }

  fn add_md_version(&mut self, key: &str, version: &str) {
    if let Some(md) = self.metadata.get_mut("pactRust") {
      md.insert(key.to_string(), version.to_string());
    } else {
      self.metadata.insert("pactRust".to_string(), btreemap! {
        key.to_string() => version.to_string()
      });
    }
  }
}

impl RequestResponsePact {

  /// Returns the specification version of this pact
  pub fn spec_version(&self) -> PactSpecification {
    determine_spec_version(&"<Pact>".to_string(), &self.metadata)
  }

  /// Creates a `Pact` from a `Value` struct.
  pub fn from_json(source: &str, pact_json: &Value) -> anyhow::Result<RequestResponsePact> {
    let metadata = parse_meta_data(pact_json);
    let spec_version = determine_spec_version(source, &metadata);

    match spec_version {
      PactSpecification::V4 => V4Pact::pact_from_json(pact_json, source)
        .and_then(|pact| pact.as_request_response_pact()),
      _ => {
        let consumer = match pact_json.get("consumer") {
          Some(v) => Consumer::from_json(v),
          None => Consumer { name: "consumer".to_string() }
        };
        let provider = match pact_json.get("provider") {
          Some(v) => Provider::from_json(v),
          None => Provider { name: "provider".to_string() }
        };
        Ok(RequestResponsePact {
          consumer,
          provider,
          interactions: parse_interactions(pact_json, spec_version.clone())?,
          metadata,
          specification_version: spec_version,
        })
      }
    }
  }

  /// Creates a BTreeMap of the metadata of this pact.
  pub fn metadata_to_json(&self, pact_spec: &PactSpecification) -> BTreeMap<String, Value> {
    let mut md_map: BTreeMap<String, Value> = self.metadata.iter()
      .map(|(k, v)| {
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, json!(v.iter()
                  .map(|(k, v)| (k.clone(), v.clone()))
                  .collect::<BTreeMap<String, String>>()))
      })
      .collect();

    md_map.insert("pactSpecification".to_string(), json!({"version" : pact_spec.version_str()}));

    let version_entry = md_map.entry("pactRust".to_string())
      .or_insert(Value::Object(Map::default()));
    if let Value::Object(map) = version_entry {
      map.insert("models".to_string(), Value::String(PACT_RUST_VERSION.unwrap_or("unknown").to_string()));
    }

    md_map
  }

  /// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
  #[cfg(not(target_family = "wasm"))]
  pub fn from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<RequestResponsePact> {
    let (url, json) = http_utils::fetch_json_from_url(&url.to_string(), auth)?;
    RequestResponsePact::from_json(&url, &json)
  }

  /// Returns a default RequestResponsePact struct
  pub fn default() -> RequestResponsePact {
    RequestResponsePact {
      consumer: Consumer { name: "default_consumer".to_string() },
      provider: Provider { name: "default_provider".to_string() },
      interactions: Vec::new(),
      metadata: RequestResponsePact::default_metadata(),
      specification_version: PactSpecification::V3
    }
  }

  /// Returns the default metadata
  pub fn default_metadata() -> BTreeMap<String, BTreeMap<String, String>> {
    btreemap!{
      "pactSpecification".to_string() => btreemap!{ "version".to_string() => PactSpecification::V3.version_str() },
      "pactRust".to_string() => btreemap!{ "models".to_string() => PACT_RUST_VERSION.unwrap_or("unknown").to_string() }
    }
  }

  /// Generate the JSON schema properties for the given Pact specification
  pub fn schema(spec_version: PactSpecification) -> Value {
    json!({
      "properties": {
        "consumer": Consumer::schema(spec_version),
        "interactions": {
          "description": "The interactions between the consumer and provider",
          "type": "array",
          "items": RequestResponseInteraction::schema(spec_version),
        },
        "metadata": {
          "description": "Metadata associated with the Pact file",
          "$ref": "#/definitions/metadata"
        },
        "provider": Provider::schema(spec_version)
      },
      "required": [
        "consumer",
        "interactions",
        "provider"
      ],
      "definitions": {
        "metadata": metadata_schema(spec_version)
      }
    })
  }
}

impl ReadWritePact for RequestResponsePact {
  #[cfg(not(target_family = "wasm"))]
  fn read_pact(path: &Path) -> anyhow::Result<RequestResponsePact> {
    with_read_lock(path, 3, &mut |f| {
      let pact_json = serde_json::from_reader(f)
        .context("Failed to parse Pact JSON")?;
      RequestResponsePact::from_json(&format!("{:?}", path), &pact_json)
    })
  }

  fn merge(&self, pact: &dyn Pact) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let conflicts = CartesianProductIterator::new(&self.interactions, &pact.interactions())
        .map(|(i1, i2)| i1.conflicts_with(i2.as_ref()))
        .filter(|conflicts| !conflicts.is_empty())
        .collect::<Vec<Vec<PactConflict>>>();
      let num_conflicts = conflicts.len();
      if num_conflicts > 0 {
        warn!("The following conflicting interactions where found:");
        for interaction_conflicts in conflicts {
          warn!(" Interaction '{}':", interaction_conflicts.first().unwrap().interaction);
          for conflict in interaction_conflicts {
            warn!("   {}", conflict.description);
          }
        }
        Err(anyhow!("Unable to merge pacts, as there were {} conflict(s) between the interactions. Please clean out your pact directory before running the tests.",
                    num_conflicts))
      } else {
        let interactions: Vec<Result<RequestResponseInteraction, String>> = self.interactions.iter()
          .merge_join_by(pact.interactions().iter(), |a, b| {
            let cmp = Ord::cmp(&a.description, &b.description());
            if cmp == Ordering::Equal && ! &a.provider_states().is_empty(){
              Ord::cmp(&a.provider_states.iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                       &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>())
            } else {
              cmp
            }
          })
          .map(|either| match either {
            Left(i) => Ok(i.clone()),
            Right(i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of())),
            Both(_, i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of()))
          })
          .collect();

        let errors: Vec<String> = interactions.iter()
          .filter(|i| i.is_err())
          .map(|i| i.as_ref().unwrap_err().to_string())
          .collect();
        if errors.is_empty() {
          Ok(Box::new(RequestResponsePact {
            provider: self.provider.clone(),
            consumer: self.consumer.clone(),
            interactions: interactions.iter()
              .filter(|i| i.is_ok())
              .map(|i| i.as_ref().unwrap().clone()).collect(),
            metadata: self.metadata.clone(),
            specification_version: self.specification_version.clone()
          }))
        } else {
          Err(anyhow!("Unable to merge pacts: {}", errors.join(", ")))
        }
      }
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

impl PactJsonVerifier for RequestResponsePact {
  fn verify_json(_path: &str, pact_json: &Value, strict: bool, spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(consumer) = values.get("consumer") {
          results.extend(Consumer::verify_json("/consumer", consumer, strict, spec_version));
        } else if strict {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::ERROR, "Missing consumer"))
        } else {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::WARNING, "Missing consumer"))
        }

        if let Some(provider) = values.get("provider") {
          results.extend(Provider::verify_json("/provider", provider, strict, spec_version));
        } else if strict {
          results.push(PactFileVerificationResult::new("/provider", ResultLevel::ERROR, "Missing provider"))
        } else {
          results.push(PactFileVerificationResult::new("/provider", ResultLevel::WARNING, "Missing provider"))
        }

        if let Some(interactions) = values.get("interactions") {
          match interactions {
            Value::Array(values) => if values.is_empty() {
              results.push(PactFileVerificationResult::new("/interactions", ResultLevel::WARNING, "Interactions is empty"))
            } else {
              results.extend(values.iter().enumerate()
                .flat_map(|(index, interaction)| {
                  RequestResponseInteraction::verify_json(&format!("/interactions/{}", index), interaction, strict, spec_version)
                }))
            }
            _ => results.push(PactFileVerificationResult::new("/interactions", ResultLevel::ERROR,
                                                              &format!("Must be an Object, got {}", json_type_of(pact_json))))
          }
        } else {
          results.push(PactFileVerificationResult::new("/interactions", ResultLevel::WARNING, "Missing interactions"))
        }

        if let Some(metadata) = values.get("metadata") {
          results.extend(verify_metadata(metadata, spec_version));
        }

        let valid_attr = hashset! { "consumer", "provider", "interactions", "metadata" };
        for (key, _) in values {
          if !valid_attr.contains(key.as_str()) {
            results.push(PactFileVerificationResult::new(&format!("/{}", key),
                                                         if strict { ResultLevel::ERROR } else { ResultLevel::WARNING },
                                                         &format!("Unexpected attribute '{}'", key)));
          }
        }
      }
      _ => results.push(PactFileVerificationResult::new("/", ResultLevel::ERROR,
                                                        &format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::{PactSpecification, pact::Pact, sync_pact::RequestResponsePact};

  #[test_log::test]
  fn convert_from_v4_json() -> anyhow::Result<()> {
    let pact_json = json!({
      "consumer": {
        "name": "convert_from_v4_json_consumer"
      },
      "interactions": [
        {
          "description": "get data",
          "key": "585ec0b0a23ba35f",
          "pending": false,
          "request": {
            "method": "GET",
            "path": "/api/v3/klines",
            "query": {
              "interval": [
                "1w"
              ],
              "limit": [
                "1"
              ],
              "symbol": [
                "LUNCUSDT"
              ]
            }
          },
          "response": {
            "body": {
              "content": [
                [
                  1673222400000_u64,
                  "0.00015856",
                  "0.00018524",
                  "0.00015744",
                  "0.00016610",
                  "344512338693.26000000",
                  1673827199999_u64,
                  "57919636.39781495",
                  183337,
                  "165369377629.69000000",
                  "27805375.21572790",
                  "0"
                ]
              ],
              "contentType": "application/json",
              "encoded": false
            },
            "headers": {
              "access-control-allow-methods": [
                "GET",
                "HEAD",
                "OPTIONS"
              ],
              "access-control-allow-origin": [
                "*"
              ],
              "cache-control": [
                "no-cache",
                "no-store",
                "must-revalidate"
              ],
              "content-length": [
                "182"
              ],
              "content-security-policy": [
                "default-src 'self'"
              ],
              "content-type": [
                "application/json;charset=UTF-8"
              ],
              "date": [
                "Tue, 10 Jan 2023 06:51:20 GMT"
              ],
              "expires": [
                "0"
              ],
              "pragma": [
                "no-cache"
              ],
              "server": [
                "nginx"
              ],
              "strict-transport-security": [
                "max-age=31536000; includeSubdomains"
              ],
              "via": [
                "1.1 67ea8416eafbda87528a822ad116e5a4.cloudfront.net (CloudFront)"
              ],
              "x-amz-cf-id": [
                "-O1yry-11OB5-w2otAcAEhoL2tIdsF13vYZJtRlBRJkjGqwDgXIPPg=="
              ],
              "x-amz-cf-pop": [
                "MEL50-C2"
              ],
              "x-cache": [
                "Miss from cloudfront"
              ],
              "x-content-security-policy": [
                "default-src 'self'"
              ],
              "x-content-type-options": [
                "nosniff"
              ],
              "x-frame-options": [
                "SAMEORIGIN"
              ],
              "x-mbx-used-weight": [
                "1"
              ],
              "x-mbx-used-weight-1m": [
                "1"
              ],
              "x-mbx-uuid": [
                "f20848fe-bcfe-4ebb-9913-a267a01e706d"
              ],
              "x-webkit-csp": [
                "default-src 'self'"
              ],
              "x-xss-protection": [
                "1; mode=block"
              ]
            },
            "status": 200
          },
          "type": "Synchronous/HTTP"
        }
      ],
      "metadata": {
        "pactRust": {
          "models": "1.0.4"
        },
        "pactSpecification": {
          "version": "4.0"
        }
      },
      "provider": {
        "name": "convert_from_v4_json_provider"
      }
    });
    let pact = RequestResponsePact::from_json(&"test", &pact_json)?;

    expect!(pact.consumer.name).to(be_equal_to("convert_from_v4_json_consumer"));
    expect!(pact.provider.name).to(be_equal_to("convert_from_v4_json_provider"));
    expect!(pact.interactions.len()).to(be_equal_to(1));
    let first_interaction = pact.interactions.first().unwrap();
    expect!(&first_interaction.description).to(be_equal_to("get data"));
    expect!(&first_interaction.request.method).to(be_equal_to("GET"));
    expect!(&first_interaction.request.path).to(be_equal_to("/api/v3/klines"));
    expect!(first_interaction.request.body.is_present()).to(be_false());
    expect!(first_interaction.request.query.as_ref().unwrap()).to(be_equal_to(&hashmap!{
      "interval".to_string() => vec![Some("1w".to_string())],
      "limit".to_string() => vec![Some("1".to_string())],
      "symbol".to_string() => vec![Some("LUNCUSDT".to_string())]
    }));
    expect!(first_interaction.request.headers.clone()).to(be_none());
    expect!(first_interaction.response.status).to(be_equal_to(200));
    expect!(&first_interaction.response.body.value_as_string().unwrap()).to(be_equal_to(r#"[[1673222400000,"0.00015856","0.00018524","0.00015744","0.00016610","344512338693.26000000",1673827199999,"57919636.39781495",183337,"165369377629.69000000","27805375.21572790","0"]]"#));
    expect!(&first_interaction.response.body.content_type().unwrap()).to(be_equal_to("application/json"));
    expect!(first_interaction.response.headers.as_ref().unwrap()).to(be_equal_to(&hashmap!{
      "access-control-allow-methods".to_string() => vec!["GET".to_string(), "HEAD".to_string(), "OPTIONS".to_string()],
      "access-control-allow-origin".to_string() => vec!["*".to_string()],
      "cache-control".to_string() => vec!["no-cache".to_string(), "no-store".to_string(), "must-revalidate".to_string()],
      "content-length".to_string() => vec!["182".to_string()],
      "content-security-policy".to_string() => vec!["default-src 'self'".to_string()],
      "content-type".to_string() => vec!["application/json;charset=UTF-8".to_string()],
      "date".to_string() => vec!["Tue, 10 Jan 2023 06:51:20 GMT".to_string()],
      "expires".to_string() => vec!["0".to_string()],
      "pragma".to_string() => vec!["no-cache".to_string()],
      "server".to_string() => vec!["nginx".to_string()],
      "strict-transport-security".to_string() => vec!["max-age=31536000; includeSubdomains".to_string()],
      "via".to_string() => vec!["1.1 67ea8416eafbda87528a822ad116e5a4.cloudfront.net (CloudFront)".to_string()],
      "x-amz-cf-id".to_string() => vec!["-O1yry-11OB5-w2otAcAEhoL2tIdsF13vYZJtRlBRJkjGqwDgXIPPg==".to_string()],
      "x-amz-cf-pop".to_string() => vec!["MEL50-C2".to_string()],
      "x-cache".to_string() => vec!["Miss from cloudfront".to_string()],
      "x-content-security-policy".to_string() => vec!["default-src 'self'".to_string()],
      "x-content-type-options".to_string() => vec!["nosniff".to_string()],
      "x-frame-options".to_string() => vec!["SAMEORIGIN".to_string()],
      "x-mbx-used-weight".to_string() => vec!["1".to_string()],
      "x-mbx-used-weight-1m".to_string() => vec!["1".to_string()],
      "x-mbx-uuid".to_string() => vec!["f20848fe-bcfe-4ebb-9913-a267a01e706d".to_string()],
      "x-webkit-csp".to_string() => vec!["default-src 'self'".to_string()],
      "x-xss-protection".to_string() => vec!["1; mode=block".to_string()]
    }));

    Ok(())
  }
  #[test_log::test]
  fn convert_from_v4_to_v2_with_repeated_query_params_preserves_order() -> anyhow::Result<()> {
    let pact_json = json!({
      "consumer": {
        "name": "convert_from_v4_to_v2_consumer"
      },
      "interactions": [
        {
          "description": "get participants",
          "key": "abc123",
          "pending": false,
          "request": {
            "method": "GET",
            "path": "/participants",
            "query": {
              "pacticipant": [
                "foo",
                "bar"
              ],
              "version": [
                "1.2.3",
                "4.5.6"
              ]
            }
          },
          "response": {
            "status": 200
          },
          "type": "Synchronous/HTTP"
        }
      ],
      "metadata": {
        "pactRust": {
          "models": "1.0.4"
        },
        "pactSpecification": {
          "version": "4.0"
        }
      },
      "provider": {
        "name": "convert_from_v4_to_v2_provider"
      }
    });

    let pact = RequestResponsePact::from_json("test", &pact_json)?;
    let v2_json = pact.to_json(PactSpecification::V2)?;

    // Check that the query string is preserved in order in the output
    let interactions = v2_json.get("interactions").unwrap().as_array().unwrap();
    let req = interactions[0].get("request").unwrap();
    let query_str = req.get("query").unwrap().as_str().unwrap();

    // The order must be: pacticipant=foo&version=1.2.3&pacticipant=bar&version=4.5.6
    expect!(query_str).to(be_equal_to("pacticipant=foo&version=1%2e2%2e3&pacticipant=bar&version=4%2e5%2e6"));

    Ok(())
  }
}
