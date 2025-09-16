//! Traits to represent a Pact

use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
#[cfg(not(target_family = "wasm"))] use std::fs;
#[cfg(not(target_family = "wasm"))] use std::fs::File;
#[cfg(not(target_family = "wasm"))] use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(not(target_family = "wasm"))] use std::ops::Deref;
use std::panic::RefUnwindSafe;
#[cfg(not(target_family = "wasm"))] use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
#[cfg(not(target_family = "wasm"))] use anyhow::Context;
use lazy_static::lazy_static;
use maplit::btreemap;
use serde_json::{json, Value};
use tracing::{debug, error, trace, warn};

use crate::{Consumer, PactSpecification, Provider};
#[cfg(not(target_family = "wasm"))] use crate::file_utils::{with_read_lock_for_open_file, with_write_lock};
#[cfg(not(target_family = "wasm"))] use crate::http_utils;
#[cfg(not(target_family = "wasm"))] use crate::http_utils::HttpAuth;
use crate::interaction::Interaction;
use crate::message_pact::MessagePact;
use crate::plugins::PluginData;
use crate::sync_pact::RequestResponsePact;
use crate::v4;
use crate::v4::pact::V4Pact;
use crate::verify_json::{json_type_of, PactFileVerificationResult, ResultLevel};

/// Trait for a Pact (request/response or message)
pub trait Pact: Debug + ReadWritePact {
  /// Consumer side of the pact
  fn consumer(&self) -> Consumer;

  /// Provider side of the pact
  fn provider(&self) -> Provider;

  /// Interactions in the Pact
  fn interactions(&self) -> Vec<Box<dyn Interaction + Send + Sync + RefUnwindSafe>>;

  /// Mutable collection of interactions in the Pact
  fn interactions_mut(&mut self) -> Vec<&mut (dyn Interaction + Send + Sync)>;

  /// Pact metadata
  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>>;

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value>;

  /// Attempt to downcast to a concrete Pact
  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact>;

  /// Attempt to downcast to a concrete Message Pact
  fn as_message_pact(&self) -> anyhow::Result<MessagePact>;

  /// Attempt to downcast to a concrete V4 Pact
  fn as_v4_pact(&self) -> anyhow::Result<V4Pact>;

  /// Specification version of this Pact
  fn specification_version(&self) -> PactSpecification;

  /// Clones this Pact and wraps it in a Box
  fn boxed(&self) -> Box<dyn Pact + Send + Sync + RefUnwindSafe>;

  /// Clones this Pact and wraps it in an Arc
  fn arced(&self) -> Arc<dyn Pact + Send + Sync + RefUnwindSafe>;

  /// Clones this Pact and wraps it in an Arc and Mutex
  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync + RefUnwindSafe>>;

  /// Adds an interactions in the Pact
  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()>;

  /// If this Pact needs any plugins loaded
  fn requires_plugins(&self) -> bool;

  /// Plugins required for this Pact. These will be taken from the 'plugins' key in the pact
  /// metadata.
  fn plugin_data(&self) -> Vec<PluginData>;

  /// If this is a V4 Pact
  fn is_v4(&self) -> bool;

  /// Add the plugin and plugin data to this Pact. If an entry already exists for the plugin,
  /// the plugin data will be merged
  fn add_plugin(
    &mut self,
    name: &str,
    version: &str,
    plugin_data: Option<HashMap<String, Value>>
  ) -> anyhow::Result<()>;

  /// Adds some version info to the Pact-Rust metadata section
  fn add_md_version(&mut self, key: &str, version: &str);
}

impl Default for Box<dyn Pact> {
  fn default() -> Self {
    V4Pact::default().boxed()
  }
}

impl Clone for Box<dyn Pact> {
  fn clone(&self) -> Self {
    self.boxed()
  }
}

impl PartialEq for Box<dyn Pact> {
  fn eq(&self, other: &Self) -> bool {
    if let Ok(pact) = self.as_v4_pact() {
      if let Ok(other) = other.as_v4_pact() {
        pact == other
      } else {
        false
      }
    } else if let Ok(pact) = self.as_request_response_pact() {
      if let Ok(other) = other.as_request_response_pact() {
        pact == other
      } else {
        false
      }
    } else if let Ok(pact) = self.as_message_pact() {
      if let Ok(other) = other.as_message_pact() {
        pact == other
      } else {
        false
      }
    } else {
      false
    }
  }
}

/// Reads the pact file and parses the resulting JSON into a `Pact` struct
#[cfg(not(target_family = "wasm"))]
pub fn read_pact(file: &Path) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
  let mut f = File::open(file)?;
  read_pact_from_file(&mut f, file)
}

/// Reads the pact from the file and parses the resulting JSON into a `Pact` struct
#[cfg(not(target_family = "wasm"))]
pub fn read_pact_from_file(file: &mut File, path: &Path) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
  let buf = with_read_lock_for_open_file(path, file, 3, &mut |f| {
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
  })?;
  let pact_json = serde_json::from_str(&buf)
    .context("Failed to parse Pact JSON")
    .map_err(|err| {
      error!("read_pact_from_file: {}", err);
      debug!("read_pact_from_file: file contents = '{}'", buf);
      err
    })?;
  load_pact_from_json(&*path.to_string_lossy(), &pact_json)
    .map_err(|e| anyhow!(e))
}

/// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
// TODO: For next major version, refactor this to also return any associated HAL links
#[cfg(not(target_family = "wasm"))]
pub fn load_pact_from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
  let (url, pact_json) = http_utils::fetch_json_from_url(&url.to_string(), auth)?;
  load_pact_from_json(&url, &pact_json)
}

/// Loads a Pact model from a JSON Value
pub fn load_pact_from_json(source: &str, json: &Value) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>> {
  match json {
    Value::Object(map) => {
      let metadata = parse_meta_data(json);
      let spec_version = determine_spec_version(source, &metadata);
      trace!("load_pact_from_json: found spec version {} in metadata", spec_version);
      match spec_version {
        PactSpecification::V4 => v4::pact::from_json(&source, json),
        _ => if map.contains_key("messages") {
          trace!("load_pact_from_json: JSON has a messages attribute, will load as a message pact");
          let pact = MessagePact::from_json(source, json)?;
          Ok(Box::new(pact))
        } else {
          trace!("load_pact_from_json: loading JSON as a request/response pact");
          Ok(Box::new(RequestResponsePact::from_json(source, json)?))
        }
      }
    },
    _ => Err(anyhow!("Failed to parse Pact JSON from source '{}' - it is not a valid pact file", source))
  }
}

/// Trait for objects that can represent Pacts and can be read and written
pub trait ReadWritePact {
  /// Reads the pact file and parses the resulting JSON into a `Pact` struct
  #[cfg(not(target_family = "wasm"))]
  fn read_pact(path: &Path) -> anyhow::Result<Self> where Self: std::marker::Sized + Send + Sync + RefUnwindSafe;

  /// Merges this pact with the other pact, and returns a new Pact with the interactions sorted.
  /// Returns an error if there is a merge conflict, which will occur if the other pact is a different
  /// type, or if a V3 Pact then if any interaction has the
  /// same description and provider state and the requests and responses are different.
  fn merge(&self, other: &dyn Pact) -> anyhow::Result<Box<dyn Pact + Send + Sync + RefUnwindSafe>>;

  /// Determines the default file name for the pact. This is based on the consumer and
  /// provider names.
  fn default_file_name(&self) -> String;
}

lazy_static!{
  static ref WRITE_LOCK: Mutex<()> = Mutex::new(());
}

/// Writes the pact out to the provided path. If there is an existing pact at the path, the two
/// pacts will be merged together unless overwrite is true. Returns an error if the file can not
/// be written or the pacts can not be merged.
#[cfg(not(target_family = "wasm"))]
pub fn write_pact(
  pact: Box<dyn Pact>,
  path: &Path,
  pact_spec: PactSpecification,
  overwrite: bool
) -> anyhow::Result<()> {
  fs::create_dir_all(path.parent().unwrap())?;
  let _lock = WRITE_LOCK.lock().unwrap();
  if !overwrite && path.exists() {
    debug!("Merging pact with file {:?}", path);
    let mut f = fs::OpenOptions::new().read(true).write(true).open(&path)?;
    let existing_pact = read_pact_from_file(&mut f, path)?;

    if existing_pact.specification_version() < pact.specification_version() {
      warn!("Note: Existing pact is an older specification version ({:?}), and will be upgraded",
            existing_pact.specification_version());
    }

    let merged_pact = pact.merge(existing_pact.deref())?;
    let pact_json = serde_json::to_string_pretty(&merged_pact.to_json(pact_spec)?)?;

    with_write_lock(path, &mut f, 3, &mut |f| {
      f.set_len(0)?;
      f.seek(SeekFrom::Start(0))?;
      f.write_all(pact_json.as_bytes())?;
      Ok(())
    })
  } else {
    debug!("Writing new pact file to {:?}", path);
    let result = serde_json::to_string_pretty(&pact.to_json(pact_spec)?)?;
    let mut file = File::create(path)?;
    with_write_lock(path, &mut file, 3, &mut |f| {
      f.write_all(result.as_bytes())?;
      Ok(())
    })
  }
}


/// Construct Metadata from JSON value
pub fn parse_meta_data(pact_json: &Value) -> BTreeMap<String, BTreeMap<String, String>> {
  match pact_json.get("metadata") {
    Some(v) => match *v {
      Value::Object(ref obj) => obj.iter().map(|(k, v)| {
        let val = match *v {
          Value::Object(ref obj) => obj.iter().map(|(k, v)| {
            match *v {
              Value::String(ref s) => (k.clone(), s.clone()),
              _ => (k.clone(), v.to_string())
            }
          }).collect(),
          _ => btreemap!{}
        };
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, val)
      }).collect(),
      _ => btreemap!{}
    },
    None => btreemap!{}
  }
}

pub(crate) fn metadata_schema(spec_version: PactSpecification) -> Value {
  if spec_version < PactSpecification::V3 {
    json!({
      "properties": {
        "pactSpecification": {
          "additionalProperties": false,
          "properties": {
            "version": {
              "type": "string"
            }
          },
          "required": ["version"],
          "type": "object"
        },
        "pactSpecificationVersion": {
          "type": "string"
        },
        "pact-specification": {
          "additionalProperties": false,
          "properties": {
              "version": {
                  "type": "string"
              }
          },
          "required": ["version"],
          "type": "object"
        }
      },
      "type": "object"
    })
  } else {
    json!({
      "properties": {
        "pactSpecification": {
          "additionalProperties": false,
          "properties": {
            "version": {
              "type": "string"
            }
          },
          "required": ["version"],
          "type": "object"
        }
      },
      "type": "object"
    })
  }
}

/// Determines the Pact specification version from the metadata of the Pact file
pub fn determine_spec_version(file: &str, metadata: &BTreeMap<String, BTreeMap<String, String>>) -> PactSpecification {
  let specification = if metadata.contains_key("pact-specification") {
    metadata.get("pact-specification")
  } else {
    metadata.get("pactSpecification")
  };
  match specification {
    Some(spec) => {
      match spec.get("version") {
        Some(ver) => match lenient_semver::parse(ver) {
          Ok(ver) => match ver.major {
            1 => match ver.minor {
              0 => PactSpecification::V1,
              1 => PactSpecification::V1_1,
              _ => {
                warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V1 specification", ver, file);
                PactSpecification::V1
              }
            },
            2 => PactSpecification::V2,
            3 => PactSpecification::V3,
            4 => PactSpecification::V4,
            _ => {
              warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V3 specification", ver, file);
              PactSpecification::Unknown
            }
          },
          Err(err) => {
            warn!("Could not parse specification version '{}' found in the metadata in the pact file {:?}, assuming V3 specification - {}", ver, file, err);
            PactSpecification::Unknown
          }
        },
        None => {
          warn!("No specification version found in the metadata in the pact file {:?}, assuming V3 specification", file);
          PactSpecification::V3
        }
      }
    },
    None => {
      warn!("No metadata found in pact file {:?}, assuming V3 specification", file);
      PactSpecification::V3
    }
  }
}

pub(crate) fn verify_metadata(metadata: &Value, _spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
  let mut results = vec![];

  match metadata {
    Value::Object(values) => {
      let spec_value = if let Some(spec_value) = values.get("pactSpecification") {
        Some(spec_value)
      } else if let Some(spec_value) = values.get("pact-specification") {
        results.push(PactFileVerificationResult::new("/metadata", ResultLevel::WARNING,
                                                     &format!("'pact-specification' is deprecated, use 'pactSpecification' instead")));
        Some(spec_value)
      } else {
        None
      };

      if values.contains_key("pactSpecificationVersion") {
        results.push(PactFileVerificationResult::new("/metadata", ResultLevel::WARNING,
                                                     &format!("'pactSpecificationVersion' is deprecated, use 'pactSpecification/version' instead")));
      }

      if let Some(spec) = spec_value {
        match spec {
          Value::Object(values) => {
            if let Some(version) = values.get("version") {
              match version {
                Value::Null => results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::WARNING,
                                                                            &format!("pactSpecification version is NULL"))),
                Value::String(version) => if PactSpecification::parse_version(version).is_err() {
                  results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::ERROR,
                                                               &format!("'{}' is not a valid Pact specification version", version)))
                }
                _ => results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::ERROR,
                                                                  &format!("Version must be a String, got {}", json_type_of(version))))
              }
            } else {
              results.push(PactFileVerificationResult::new("/metadata/pactSpecification", ResultLevel::WARNING,
                                                           &format!("pactSpecification is missing the version attribute")));
            }
          }
          _ => results.push(PactFileVerificationResult::new("/metadata/pactSpecification", ResultLevel::ERROR,
                                                            &format!("pactSpecification must be an Object, got {}", json_type_of(spec))))
        }
      }
    }
    _ => results.push(PactFileVerificationResult::new("/metadata", ResultLevel::ERROR,
                                                      &format!("Metadata must be an Object, got {}", json_type_of(metadata))))
  }

  results
}

#[cfg(test)]
mod tests {
  use std::{env, fs, io};
  use std::fs::File;
  use std::io::Read;

  use expectest::prelude::*;
  use maplit::{btreemap, hashmap};
  use pretty_assertions::assert_eq;
  use serde_json::{json, Value};

  use crate::message::Message;
  use crate::message_pact::MessagePact;
  use crate::{Consumer, PactSpecification, Provider};
  use crate::bodies::OptionalBody;
  use crate::content_types::JSON;
  use crate::generators;
  use crate::generators::Generator;
  use crate::matchingrules;
  use crate::matchingrules::MatchingRule;
  use crate::pact::{Pact, ReadWritePact, write_pact};
  use crate::PACT_RUST_VERSION;
  use crate::provider_states::ProviderState;
  use crate::request::Request;
  use crate::response::Response;
  use crate::sync_interaction::RequestResponseInteraction;
  use crate::sync_pact::RequestResponsePact;
  use crate::v4::pact::V4Pact;
  use crate::v4::synch_http::SynchronousHttp;

  #[test]
  fn load_empty_pact() {
    let pact_json = r#"{}"#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(pact.provider.name).to(be_equal_to("provider"));
    expect!(pact.consumer.name).to(be_equal_to("consumer"));
    expect!(pact.interactions.iter()).to(have_count(0));
    expect!(pact.metadata.iter()).to(have_count(0));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
  }

  #[test]
  fn missing_metadata() {
    let pact_json = r#"{}"#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
  }

  #[test]
  fn missing_spec_version() {
    let pact_json = r#"{
        "metadata" : {
        }
    }"#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

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
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

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
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

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
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

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
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

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
        "interactions": [
          {
              "description": "a retrieve Mallory request",
              "request": {
                "method": "GET",
                "path": "/mallory",
                "query": "name=ron&status=good"
              },
              "response": {
                "status": 200,
                "headers": {
                  "Content-Type": "text/html"
                },
                "body": "\"That is some good Mallory.\""
              }
          }
        ]
    }
    "#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(&pact.provider.name).to(be_equal_to("Alice Service"));
    expect!(&pact.consumer.name).to(be_equal_to("Consumer"));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("a retrieve Mallory request"));
    expect!(interaction.provider_states.iter()).to(be_empty());
    expect!(interaction.request).to(be_equal_to(Request {
      method: "GET".to_string(),
      path: "/mallory".to_string(),
      query: Some(hashmap!{
        "name".to_string() => vec![Some("ron".to_string())],
        "status".to_string() => vec![Some("good".to_string())]
      }),
      headers: None,
      body: OptionalBody::Missing,
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
      status: 200,
      headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }),
      body: OptionalBody::Present("\"That is some good Mallory.\"".into(), Some("text/html".into()), None),
      .. Response::default()
    }));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    expect!(pact.metadata.iter()).to(have_count(0));
  }

  #[test]
  fn load_pact() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : "q=p&q=p2&r=s",
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "1.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(&pact.provider.name).to(be_equal_to("test_provider"));
    expect!(&pact.consumer.name).to(be_equal_to("test_consumer"));
    expect!(pact.metadata.iter()).to(have_count(2));
    expect!(&pact.metadata["pactSpecification"]["version"]).to(be_equal_to("1.0.0"));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V1));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("test interaction"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
      ProviderState { name: "test state".to_string(), params: hashmap!{} } ]));
    expect!(interaction.request).to(be_equal_to(Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: Some(hashmap!{
        "q".to_string() => vec![Some("p".to_string()), Some("p2".to_string())],
        "r".to_string() => vec![Some("s".to_string())]
      }),
      headers: Some(hashmap!{ "testreqheader".to_string() => vec!["testreqheadervalue".to_string()] }),
      body: "{\"test\":true}".into(),
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
      status: 200,
      headers: Some(hashmap!{ "testreqheader".to_string() => vec!["testreqheaderval".to_string()] }),
      body: "{\"responsetest\":true}".into(),
      .. Response::default()
    }));
  }

  #[test]
  fn load_v3_pact() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : {
              "q": ["p", "p2"],
              "r": ["s"]
          },
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "3.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(&pact.provider.name).to(be_equal_to("test_provider"));
    expect!(&pact.consumer.name).to(be_equal_to("test_consumer"));
    expect!(pact.metadata.iter()).to(have_count(2));
    expect!(&pact.metadata["pactSpecification"]["version"]).to(be_equal_to("3.0.0"));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("test interaction"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
      ProviderState { name: "test state".to_string(), params: hashmap!{} } ]));
    expect!(interaction.request).to(be_equal_to(Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: Some(hashmap!{
        "q".to_string() => vec![Some("p".to_string()), Some("p2".to_string())],
        "r".to_string() => vec![Some("s".to_string())]
      }),
      headers: Some(hashmap!{ "testreqheader".to_string() => vec!["testreqheadervalue".to_string()] }),
      body: OptionalBody::Present("{\"test\":true}".into(), None, None),
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
      status: 200,
      headers: Some(hashmap!{ "testreqheader".to_string() => vec!["testreqheaderval".to_string()] }),
      body: OptionalBody::Present("{\"responsetest\":true}".into(), None, None),
      .. Response::default()
    }));
  }

  #[test]
  fn load_pact_encoded_query_string() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : "datetime=2011-12-03T10%3A15%3A30%2B01%3A00&description=hello+world%21",
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "2.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.request).to(be_equal_to(Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: Some(hashmap!{
        "datetime".to_string() => vec![Some("2011-12-03T10:15:30+01:00".to_string())],
        "description".to_string() => vec![Some("hello world!".to_string())]
      }),
      headers: Some(hashmap!{ "testreqheader".to_string() => vec!["testreqheadervalue".to_string()] }),
      body: OptionalBody::Present("{\"test\":true}".into(), None, None),
      .. Request::default()
    }));
  }

  #[test]
  fn load_pact_converts_methods_to_uppercase() {
    let pact_json = r#"
    {
      "interactions" : [ {
        "description" : "test interaction",
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    }
    "#;
    let pact = RequestResponsePact::from_json(&"".to_string(), &serde_json::from_str(pact_json).unwrap());
    let pact = pact.unwrap();

    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.request).to(be_equal_to(Request {
      method: "GET".to_string(),
      path: "/".to_string(),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      .. Request::default()
    }));
  }

  #[test]
  fn default_file_name_is_based_in_the_consumer_and_provider() {
    let pact = RequestResponsePact { consumer: Consumer { name: "consumer".to_string() },
      provider: Provider { name: "provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    expect!(pact.default_file_name()).to(be_equal_to("consumer-provider.json"));
  }

  fn read_pact_file(file: &str) -> io::Result<String> {
    let mut f = File::open(file)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    Ok(buffer)
  }

  #[test]
  fn write_pact_test() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    assert_eq!(pact_file, format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap()));
  }

  #[test]
  fn write_pact_test_should_merge_pacts() {
    let pact = RequestResponsePact { consumer: Consumer { name: "merge_consumer".to_string() },
      provider: Provider { name: "merge_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction 2".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "merge_consumer".to_string() },
      provider: Provider { name: "merge_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V2, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "merge_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }},
    {{
      "description": "Test Interaction 2",
      "providerState": "Good state to be in",
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "merge_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  // Issue #389
  #[test]
  fn write_pact_test_should_merge_duplicate_http_pacts_without_provider_states() {
    let existing_pact = RequestResponsePact { consumer: Consumer { name: "dupe_consumer".to_string() },
      provider: Provider { name: "dupe_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "description 1".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request { headers: Some(hashmap!{
            "Accept".to_string()=>vec!["application/json".to_string()]
          }), .. Request::default() },
          .. RequestResponseInteraction::default()
        },
        RequestResponseInteraction {
          description: "description 2".to_string(),
          request: Request { headers: Some(hashmap!{
            "Accept".to_string()=>vec!["application/json".to_string()]
          }), .. Request::default() },
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V3
    };
    let new_pact = RequestResponsePact { consumer: Consumer { name: "dupe_consumer".to_string() },
      provider: Provider { name: "dupe_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "description 2".to_string(),
          request: Request { headers: Some(hashmap!{
            "Accept".to_string()=>vec!["application/json".to_string()]
          }), .. Request::default() },
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V3
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(existing_pact.default_file_name());

    let result = write_pact(existing_pact.boxed(), dir.as_path(), PactSpecification::V3, false);
    let result2 = write_pact(new_pact.boxed(), dir.as_path(), PactSpecification::V3, false);

    let pact_file: String = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    let mut json: Value = serde_json::from_str(&pact_file).unwrap();
    json["metadata"]["pactRust"] = Value::Null;
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    assert_eq!(serde_json::to_string_pretty(&json).unwrap(),r#"{
  "consumer": {
    "name": "dupe_consumer"
  },
  "interactions": [
    {
      "description": "description 1",
      "providerStates": [
        {
          "name": "Good state to be in"
        }
      ],
      "request": {
        "headers": {
          "Accept": "application/json"
        },
        "method": "GET",
        "path": "/"
      },
      "response": {
        "status": 200
      }
    },
    {
      "description": "description 2",
      "request": {
        "headers": {
          "Accept": "application/json"
        },
        "method": "GET",
        "path": "/"
      },
      "response": {
        "status": 200
      }
    }
  ],
  "metadata": {
    "pactRust": null,
    "pactSpecification": {
      "version": "3.0.0"
    }
  },
  "provider": {
    "name": "dupe_provider"
  }
}"#);
  }
  
  // Issue #389
  #[test]
  fn write_pact_test_should_merge_duplicate_message_pacts_without_provider_states() {
    let existing_pact = MessagePact {
      consumer: Consumer { name: "dupe_consumer".to_string() },
      provider: Provider { name: "dupe_provider".to_string() },
      messages: vec![
        Message {
          description: "description 1".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. Message::default()
        },
        Message {
          description: "description 2".to_string(),
          .. Message::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V3
    };
    let new_pact = MessagePact { consumer: Consumer { name: "dupe_consumer".to_string() },
      provider: Provider { name: "dupe_provider".to_string() },
      messages: vec![
        Message {
          description: "description 2".to_string(),
          .. Message::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V3
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(existing_pact.default_file_name());

    let result = write_pact(existing_pact.boxed(), dir.as_path(), PactSpecification::V3, false);
    let result2 = write_pact(new_pact.boxed(), dir.as_path(), PactSpecification::V3, false);

    let pact_file: String = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    let mut json: Value = serde_json::from_str(&pact_file).unwrap();
    json["metadata"]["pactRust"] = Value::Null;
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    assert_eq!(serde_json::to_string_pretty(&json).unwrap(),r#"{
  "consumer": {
    "name": "dupe_consumer"
  },
  "messages": [
    {
      "description": "description 1",
      "metadata": {
        "contentType": "application/json"
      },
      "providerStates": [
        {
          "name": "Good state to be in"
        }
      ]
    },
    {
      "description": "description 2",
      "metadata": {
        "contentType": "application/json"
      }
    }
  ],
  "metadata": {
    "pactRust": null,
    "pactSpecification": {
      "version": "3.0.0"
    }
  },
  "provider": {
    "name": "dupe_provider"
  }
}"#);
  }
  
  #[test]
  fn write_pact_test_should_not_merge_pacts_with_conflicts() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          response: Response { status: 400, .. Response::default() },
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V2, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_err());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_test_should_upgrade_older_pacts_when_merging() {
    let pact = RequestResponsePact { consumer: Consumer { name: "merge_consumer".to_string() },
      provider: Provider { name: "merge_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction 2".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "merge_consumer".to_string() },
      provider: Provider { name: "merge_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V3
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V3, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "merge_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }},
    {{
      "description": "Test Interaction 2",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "3.0.0"
    }}
  }},
  "provider": {{
    "name": "merge_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_test_upgrades_older_pacts_to_v4_when_merging() {
    let pact = RequestResponsePact {
      consumer: Consumer { name: "merge_consumer".into() },
      provider: Provider { name: "merge_provider".into() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction 2".into(),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap! {} }],
          ..RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap! {},
      specification_version: PactSpecification::V1_1,
    };
    let pact2 = V4Pact {
      consumer: Consumer { name: "merge_consumer".into() },
      provider: Provider { name: "merge_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          id: None,
          key: None,
          description: "Test Interaction".into(),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap! {} }],
          .. Default::default()
        })
      ],
      .. V4Pact::default()
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V4, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "merge_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }},
    {{
      "description": "Test Interaction 2",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "merge_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn pact_merge_does_not_merge_different_consumers() {
    let pact = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "test_consumer2".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
  }

  #[test]
  fn pact_merge_does_not_merge_different_providers() {
    let pact = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider2".to_string() },
      interactions: vec![],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
  }

  #[test]
  fn pact_merge_does_not_merge_where_there_are_conflicting_interactions() {
    let pact = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request { path: "/other".to_string(), .. Request::default() },
          .. RequestResponseInteraction::default()
        }
      ],
      metadata: btreemap!{},
      specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
  }

  #[test]
  fn pact_merge_removes_duplicates() {
    let pact = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default()
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        },
        RequestResponseInteraction {
          description: "Test Interaction 2".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default()
    };

    let merged_pact = pact.merge(&pact2);
    expect!(merged_pact.unwrap().interactions().len()).to(be_equal_to(2));

    let merged_pact2 = pact.merge(&pact.clone());
    expect!(merged_pact2.unwrap().interactions().len()).to(be_equal_to(1));
  }

  #[test]
  fn write_pact_test_with_matchers() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request {
            matching_rules: matchingrules!{
                        "body" => {
                            "$" => [ MatchingRule::Type ]
                        }
                    },
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "matchingRules": {{
          "$.body": {{
            "match": "type"
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_v3_test_with_matchers() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer_v3".to_string() },
      provider: Provider { name: "write_pact_test_provider_v3".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request {
            matching_rules: matchingrules!{
                        "body" => {
                            "$" => [ MatchingRule::Type ]
                        },
                        "header" => {
                          "HEADER_A" => [ MatchingRule::Include("ValA".to_string()), MatchingRule::Include("ValB".to_string()) ]
                        }
                    },
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file.parse::<Value>().unwrap()).to(be_equal_to(json!({
      "consumer": {
        "name": "write_pact_test_consumer_v3"
      },
      "interactions": [
        {
          "description": "Test Interaction",
          "providerStates": [
            {
              "name": "Good state to be in"
            }
          ],
          "request": {
            "matchingRules": {
              "body": {
                "$": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "type"
                    }
                  ]
                }
              },
              "header": {
                "HEADER_A": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "include",
                      "value": "ValA"
                    },
                    {
                      "match": "include",
                      "value": "ValB"
                    }
                  ]
                }
              }
            },
            "method": "GET",
            "path": "/"
          },
          "response": {
            "status": 200
          }
        }
      ],
      "metadata": {
        "pactRust": {
          "models": PACT_RUST_VERSION
        },
        "pactSpecification": {
          "version": "3.0.0"
        }
      },
      "provider": {
        "name": "write_pact_test_provider_v3"
      }
    })));
  }

  #[test]
  fn write_v3_pact_test() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request {
            query: Some(hashmap!{
                "a".to_string() => vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string())],
                "b".to_string() => vec![Some("bill".to_string()), Some("bob".to_string())]
            }),
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/",
        "query": {{
          "a": [
            "1",
            "2",
            "3"
          ],
          "b": [
            "bill",
            "bob"
          ]
        }}
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "3.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_test_with_generators() {
    let pact = RequestResponsePact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction with generators".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          request: Request {
            generators: generators!{
                        "BODY" => {
                          "$" => Generator::RandomInt(1, 10)
                        },
                        "HEADER" => {
                          "A" => Generator::RandomString(20)
                        }
                    },
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction with generators",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "generators": {{
          "body": {{
            "$": {{
              "max": 10,
              "min": 1,
              "type": "RandomInt"
            }}
          }},
          "header": {{
            "A": {{
              "size": 20,
              "type": "RandomString"
            }}
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "models": "{}"
    }},
    "pactSpecification": {{
      "version": "3.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn merge_pact_test() {
    let pact = RequestResponsePact {
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction with matcher".to_string(),
          request: Request {
            body: OptionalBody::Present(json!({ "related": [1, 2, 3] }).to_string().into(), Some(JSON.clone()), None),
            matching_rules: matchingrules!{
            "body" => {
              "$.related" => [ MatchingRule::MinMaxType(0, 5) ]
            }
          },
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let updated_pact = RequestResponsePact {
      interactions: vec![
        RequestResponseInteraction {
          description: "Test Interaction with matcher".to_string(),
          request: Request {
            body: OptionalBody::Present(json!({ "related": [1, 2, 3] }).to_string().into(), Some(JSON.clone()), None),
            matching_rules: matchingrules!{
            "body" => {
              "$.related" => [ MatchingRule::MinMaxType(1, 10) ]
            }
          },
            .. Request::default()
          },
          .. RequestResponseInteraction::default()
        }
      ],
      .. RequestResponsePact::default() };
    let merged_pact = pact.merge(&updated_pact);
    expect(merged_pact.unwrap().as_request_response_pact().unwrap()).to(be_equal_to(updated_pact));
  }
}
