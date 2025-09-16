//! The `message_pact` module defines a Pact
//! that contains Messages instead of Interactions.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
#[cfg(not(target_family = "wasm"))] use std::fs;
#[cfg(not(target_family = "wasm"))] use std::fs::File;
#[cfg(not(target_family = "wasm"))] use std::io::prelude::*;
use std::panic::RefUnwindSafe;
#[cfg(not(target_family = "wasm"))] use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use maplit::*;
use serde_json::{json, Map, Value};
use tracing::debug;

use crate::{Consumer, PactSpecification, Provider};
#[cfg(not(target_family = "wasm"))] use crate::file_utils::with_read_lock;
#[cfg(not(target_family = "wasm"))] use crate::http_utils::{self, HttpAuth};
use crate::interaction::Interaction;
use crate::message::Message;
use crate::pact::{determine_spec_version, Pact, parse_meta_data, ReadWritePact};
use crate::PACT_RUST_VERSION;
use crate::plugins::PluginData;
use crate::sync_pact::RequestResponsePact;
use crate::v4::pact::V4Pact;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// Struct that represents a pact between the consumer and provider of a service.
/// It contains a list of Messages instead of Interactions, but is otherwise
/// identical to `struct Pact`.
#[derive(Debug, Clone, PartialEq)]
pub struct MessagePact {
    /// Consumer side of the pact
    pub consumer: Consumer,
    /// Provider side of the pact
    pub provider: Provider,
    /// List of messages between the consumer and provider.
    pub messages: Vec<Message>,
    /// Metadata associated with this pact file.
    pub metadata: BTreeMap<String, BTreeMap<String, String>>,
    /// Specification version of this pact
    pub specification_version: PactSpecification,
}

impl Pact for MessagePact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<Box<dyn Interaction + Send + Sync + RefUnwindSafe>> {
    self.messages.iter().map(|i| i.boxed()).collect()
  }

  fn interactions_mut(&mut self) -> Vec<&mut (dyn Interaction + Send + Sync)> {
    self.messages.iter_mut().map(|m| m as &mut (dyn Interaction + Send + Sync)).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.clone()
  }

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V3 => Ok(json!({
        "consumer": self.consumer.to_json(),
        "provider": self.provider.to_json(),
        "messages":
        Value::Array(self.messages.iter().map(|m| m.to_json(&pact_spec)).collect()),
        "metadata": self.metadata_to_json(&pact_spec)
      })),
      PactSpecification::V4 => self.as_v4_pact()?.to_json(pact_spec),
      _ => Err(anyhow!("Message Pacts require minimum V3 specification"))
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    Err(anyhow!("Can't convert a Message Pact to a different type"))
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    Ok(self.clone())
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    let interactions = self.messages.iter()
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
    match interaction.as_message() {
      None => Err(anyhow!("Can only add message interactions to this Pact")),
      Some(interaction) => {
        self.messages.push(interaction);
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

impl MessagePact {

    /// Returns the specification version of this pact
    pub fn spec_version(&self) -> PactSpecification {
      determine_spec_version("<MessagePact>", &self.metadata)
    }

    /// Creates a `MessagePact` from a `Value` struct.
    pub fn from_json(file: &str, pact_json: &Value) -> anyhow::Result<MessagePact> {
        let metadata = parse_meta_data(pact_json);
        let spec_version = determine_spec_version(file, &metadata);

        let consumer = match pact_json.get("consumer") {
            Some(v) => Consumer::from_json(v),
            None => Consumer { name: "consumer".to_string() }
        };
        let provider = match pact_json.get("provider") {
            Some(v) => Provider::from_json(v),
            None => Provider { name: "provider".to_string() }
        };

        let messages = match pact_json.get("messages") {
            Some(Value::Array(msg_arr)) => {
                let mut messages = Vec::with_capacity(msg_arr.len());
                for (ix, msg) in msg_arr.iter().enumerate() {
                    messages.push(
                        Message::from_json(ix, msg, &spec_version)?
                    );
                }
                messages
            }
            Some(_) => bail!("Expecting 'messages' field to be Array"),
            None => vec![],
        };

        Ok(MessagePact {
            consumer,
            provider,
            messages,
            metadata,
            specification_version: spec_version.clone(),
        })
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

        md_map.insert(
            "pactSpecification".to_string(),
            json!({"version" : pact_spec.version_str()}));
        let version_entry = md_map.entry("pactRust".to_string())
          .or_insert(Value::Object(Map::default()));
        if let Value::Object(map) = version_entry {
          map.insert("models".to_string(), Value::String(PACT_RUST_VERSION.unwrap_or("unknown").to_string()));
        }
        md_map
    }

    /// Determines the default file name for the pact.
    /// This is based on the consumer and provider names.
    pub fn default_file_name(&self) -> String {
        format!("{}-{}.json", self.consumer.name, self.provider.name)
    }

    /// Reads the pact file from a URL and parses the resulting JSON
    /// into a `MessagePact` struct
    #[cfg(not(target_family = "wasm"))]
    pub fn from_url(url: &String, auth: &Option<HttpAuth>) -> anyhow::Result<MessagePact> {
        let (url, json) = http_utils::fetch_json_from_url(url, auth)?;
        MessagePact::from_json(&url, &json)
    }

    /// Writes this pact out to the provided file path.
    /// All directories in the path will automatically created.
    /// If there is already a file at the path, it will be overwritten.
    #[cfg(not(target_family = "wasm"))]
    pub fn overwrite_pact(
        &self,
        path: &Path,
        pact_spec: PactSpecification,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;

        debug!("Writing new pact file to {:?}", path);
        let mut file = File::create(path)?;

        file.write_all(
          serde_json::to_string_pretty(&self.to_json(pact_spec)?)?.as_bytes()
        )?;

        Ok(())
    }

    /// Returns a default MessagePact struct
    pub fn default() -> MessagePact {
        MessagePact {
            consumer: Consumer { name: "default_consumer".to_string() },
            provider: Provider { name: "default_provider".to_string() },
            messages: Vec::new(),
            metadata: MessagePact::default_metadata(),
            specification_version: PactSpecification::V3,
        }
    }

    /// Returns the default metadata
    pub fn default_metadata()
    -> BTreeMap<String, BTreeMap<String, String>> {
        btreemap!{
            "pact-specification".to_string() =>
                btreemap!{ "version".to_string() =>
                    PactSpecification::V3.version_str() },
            "pact-rust".to_string() =>
                btreemap!{ "version".to_string() =>
                    PACT_RUST_VERSION.unwrap_or("unknown").to_string() },
        }
    }
}

impl ReadWritePact for MessagePact {
  #[cfg(not(target_family = "wasm"))]
  fn read_pact(path: &Path) -> anyhow::Result<MessagePact> {
    with_read_lock(path, 3, &mut |f| {
      let pact_json: Value = serde_json::from_reader(f)?;
      MessagePact::from_json(&format!("{:?}", path), &pact_json)
        .map_err(|e| anyhow!(e))
    })
  }

  fn merge(&self, pact: &dyn Pact) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let messages: Vec<Result<Message, String>> = self.messages.iter()
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
          Right(i) => i.as_message()
            .ok_or(format!("Can't convert interaction of type {} to V3 Asynchronous/Messages", i.type_of())),
          Both(_, i) => i.as_message()
            .ok_or(format!("Can't convert interaction of type {} to V3 Asynchronous/Messages", i.type_of()))
        })
        .collect();
      let errors: Vec<String> = messages.iter()
        .filter(|i| i.is_err())
        .map(|i| i.as_ref().unwrap_err().to_string())
        .collect();
      if errors.is_empty() {
        Ok(Box::new(MessagePact {
          provider: self.provider.clone(),
          consumer: self.consumer.clone(),
          messages: messages.iter()
            .filter(|i| i.is_ok())
            .map(|i| i.as_ref().unwrap().clone()).collect(),
          metadata: self.metadata.clone(),
          specification_version: self.specification_version
        }))
      } else {
        Err(anyhow!("Unable to merge pacts: {}", errors.join(", ")))
      }
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

impl PactJsonVerifier for MessagePact {
  fn verify_json(path: &str, pact_json: &Value, _strict: bool, _spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(_values) => {

      }
      _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
        &format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

#[cfg(test)]
mod tests {
    use expectest::expect;
    use expectest::prelude::*;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn default_file_name_is_based_in_the_consumer_and_provider() {
        let pact = MessagePact { consumer: Consumer { name: "consumer".to_string() },
            provider: Provider { name: "provider".to_string() },
            messages: vec![],
            metadata: btreemap!{},
            specification_version: PactSpecification::V1_1
        };
        expect!(pact.default_file_name()).to(be_equal_to("consumer-provider.json"));
    }

    #[test]
    fn load_empty_pact() {
        let pact_json = r#"{}"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.provider.name).to(be_equal_to("provider"));
        expect!(pact.consumer.name).to(be_equal_to("consumer"));
        expect!(pact.messages.iter()).to(have_count(0));
        expect!(pact.metadata.iter()).to(have_count(0));
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_metadata() {
        let pact_json = r#"{}"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_spec_version() {
        let pact_json = r#"{
            "metadata" : {
            }
        }"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {

                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn empty_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": ""
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
    }

    #[test]
    fn correct_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": "1.0.0"
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V1));
    }

    #[test]
    fn invalid_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": "znjclkazjs"
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &"".to_string(),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
    }

    #[test]
    fn load_basic_pact() {
        let pact_json = r#"
        {
            "provider": {
                "name": "Alice Service"
            },
            "consumer": {
                "name": "Consumer"
            },
            "messages": [
                {
                    "description": "Message Description",
                    "contents": {
                        "hello": "world"
                    },
                    "metadata": {
                        "contentType": "application/json"
                    }
                }
            ]
        }
        "#;
        let pact = MessagePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
        expect!(pact.as_ref()).to(be_ok());
        let pact = pact.unwrap();
        expect!(&pact.provider.name).to(be_equal_to("Alice Service"));
        expect!(&pact.consumer.name).to(be_equal_to("Consumer"));

        expect!(pact.messages.iter()).to(have_count(1));
        let message = pact.messages[0].clone();
        expect!(message.description)
            .to(be_equal_to("Message Description"));
        expect!(message.contents.value_as_string())
            .to(be_some().value("{\"hello\":\"world\"}"));

        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
        expect!(pact.metadata.iter()).to(have_count(0));
    }

    #[test]
    fn to_json() {
        let pact_json = r#"
        {
            "provider": {
                "name": "Alice Service"
            },
            "consumer": {
                "name": "Consumer"
            },
            "messages": [
                {
                    "description": "Message Description",
                    "contents": {
                        "hello": "world"
                    },
                    "metadata": {
                        "contentType": "application/json"
                    }
                }
            ]
        }
        "#;
        let pact = MessagePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
        expect!(pact.as_ref()).to(be_ok());
        let pact = pact.unwrap();
        let contents = pact.to_json(PactSpecification::V3);
        assert_eq!(contents.unwrap().to_string(),
          "{\"consumer\":{\"name\":\"Consumer\"},\"messages\":[\
          {\"contents\":{\"hello\":\"world\"},\"description\":\"Message Description\",\
          \"metadata\":{\"contentType\":\"application/json\"}}],\
          \"metadata\":{\"pactRust\":{\"models\":\"".to_owned() + env!("CARGO_PKG_VERSION") +
            "\"},\"pactSpecification\":{\"version\":\"3.0.0\"}},\"provider\":{\"name\":\"Alice Service\"}}");
    }
}
