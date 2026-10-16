use std::{
  env,
  fs::File,
  io::prelude::*,
  iter::Iterator,
  path::PathBuf
};

use expectest::prelude::*;
use maplit::*;
use serde_json::json;

use pact_models::bodies::OptionalBody;
use pact_models::pact::{Pact, read_pact, ReadWritePact};
use pact_models::PactSpecification;
use pact_models::sync_pact::RequestResponsePact;

mod spec_testcases;

/// Get the path to one of our sample *.json files.
fn fixture_path(path: &str) -> PathBuf {
    env::current_dir()
        .expect("could not find current working directory")
        .join("tests")
        .join(path)
        .to_owned()
}

#[test]
fn test_load_pact() {
  let pact_file = fixture_path("pact.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let mut f = File::open(pact_file).unwrap();
      let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
      let pact_json = pact.as_request_response_pact().unwrap().to_json(PactSpecification::V2).unwrap();
      expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
      expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
      expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

      expect!(pact.metadata().get("pactSpecification")).to(be_none());
      let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
      let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
      expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
      expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_test_pact() {
    let pact_file = fixture_path("test_pact.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_pact_encoded_query() {
    let pact_file = fixture_path("test_pact_encoded_query.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

            let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
            expect!(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect!(pact_interaction.get("providerState")).to(be_equal_to(file_interaction.get("providerState")));
                expect!(pact_interaction.get("description")).to(be_equal_to(file_interaction.get("description")));
                expect!(pact_interaction.get("response")).to(be_equal_to(file_interaction.get("response")));

                let pact_request = pact_interaction.get("request").unwrap();
                let file_request = file_interaction.get("request").unwrap();
                expect!(pact_request.get("method")).to(be_equal_to(file_request.get("method")));
                expect!(pact_request.get("path")).to(be_equal_to(file_request.get("path")));
                expect!(pact_request.get("headers")).to(be_equal_to(file_request.get("headers")));
                expect!(pact_request.get("body")).to(be_equal_to(file_request.get("body")));
                expect!(pact_request.get("matchers")).to(be_equal_to(file_request.get("matchers")));
                expect!(pact_request.get("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.get("query").unwrap().to_string().to_uppercase()));
            }

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_lowercase_method() {
    let pact_file = fixture_path("test_pact_lowercase_method.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(PactSpecification::V3).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

            let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
            expect!(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect!(pact_interaction.get("providerState")).to(be_equal_to(file_interaction.get("providerState")));
                expect!(pact_interaction.get("description")).to(be_equal_to(file_interaction.get("description")));
                expect!(pact_interaction.get("response")).to(be_equal_to(file_interaction.get("response")));

                let pact_request = pact_interaction.get("request").unwrap();
                let file_request = file_interaction.get("request").unwrap();
                expect!(pact_request.get("method").unwrap().to_string()).to(be_equal_to(file_request.get("method").unwrap().to_string().to_uppercase()));
                expect!(pact_request.get("path")).to(be_equal_to(file_request.get("path")));
                expect!(pact_request.get("headers")).to(be_equal_to(file_request.get("headers")));
                expect!(pact_request.get("body")).to(be_equal_to(file_request.get("body")));
                expect!(pact_request.get("matchers")).to(be_equal_to(file_request.get("matchers")));
                expect!(pact_request.get("query")).to(be_equal_to(file_request.get("query")));
            }

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("3.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"3.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_bodies() {
    let pact_file = fixture_path("test_pact_no_bodies.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            for pact_interaction in pact.interactions.clone() {
                expect!(pact_interaction.request.body).to(be_equal_to(OptionalBody::Missing));
                expect!(pact_interaction.response.body).to(be_equal_to(OptionalBody::Missing));
            }

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_metadata() {
    let pact_file = fixture_path("test_pact_no_metadata.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(PactSpecification::V2).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification")).to(be_none());
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_spec_version() {
    let pact_file = fixture_path("test_pact_no_spec_version.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(PactSpecification::V2).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification")).to(be_none());
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_with_camel_case_spec_version() {
    let pact_file = fixture_path("test_pact_camel_case_spec_version.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            expect(pact.clone().specification_version).to(be_equal_to(PactSpecification::V1_1));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_version() {
    let pact_file = fixture_path("test_pact_no_version.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(PactSpecification::V2).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("null"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
            expect!(pact.specification_version.clone()).to(be_equal_to(PactSpecification::Unknown));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_query_old_format() {
    let pact_file = fixture_path("test_pact_query_old_format.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_root_string_json() {
    let pact_file = fixture_path("test_pact_root_string_json.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(PactSpecification::V3).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

            let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
            let pact_response = pact_interactions[0].get("response").unwrap();
            expect!(pact_response.get("body").unwrap().to_string()).to(be_equal_to("\"That is some good Mallory.\"".to_string()));

            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"3.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_with_bodies() {
    let pact_file = fixture_path("test_pact_with_bodies.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_v2_pact() {
    let pact_file = fixture_path("v2-pact.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_v2_pact_query() {
    let pact_file = fixture_path("v2_pact_query.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

            let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
            expect!(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect!(pact_interaction.get("providerState")).to(be_equal_to(file_interaction.get("providerState")));
                expect!(pact_interaction.get("description")).to(be_equal_to(file_interaction.get("description")));
                expect!(pact_interaction.get("response")).to(be_equal_to(file_interaction.get("response")));

                let pact_request = pact_interaction.get("request").unwrap();
                let file_request = file_interaction.get("request").unwrap();
                expect!(pact_request.get("method")).to(be_equal_to(file_request.get("method")));
                expect!(pact_request.get("path")).to(be_equal_to(file_request.get("path")));
                expect!(pact_request.get("headers")).to(be_equal_to(file_request.get("headers")));
                expect!(pact_request.get("body")).to(be_equal_to(file_request.get("body")));
                expect!(pact_request.get("matchers")).to(be_equal_to(file_request.get("matchers")));
                expect!(pact_request.get("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.get("query").unwrap().to_string().to_uppercase()));
            }

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_matchers() {
    let pact_file = fixture_path("test_pact_matchers.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
            expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_matchers_old_format() {
    let pact_file = fixture_path("test_pact_matchers_old_format.json");
    let pact_result = RequestResponsePact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json(pact.spec_version()).unwrap();
            expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
            expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

            let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
            expect!(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect!(pact_interaction.get("providerState")).to(be_equal_to(file_interaction.get("providerState")));
                expect!(pact_interaction.get("description")).to(be_equal_to(file_interaction.get("description")));

                let pact_request = pact_interaction.get("request").unwrap();
                let file_request = file_interaction.get("request").unwrap();
                expect!(pact_request.get("method")).to(be_equal_to(file_request.get("method")));
                expect!(pact_request.get("path")).to(be_equal_to(file_request.get("path")));
                expect!(pact_request.get("headers")).to(be_equal_to(file_request.get("headers")));
                expect!(pact_request.get("body")).to(be_equal_to(file_request.get("body")));
                expect!(pact_request.get("matchers")).to(be_equal_to(file_request.get("matchers")));
                expect!(pact_request.get("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.get("query").unwrap().to_string().to_uppercase()));

                let pact_response = pact_interaction.get("response").unwrap();
                let file_response = file_interaction.get("response").unwrap();
                expect!(pact_response.get("status")).to(be_equal_to(file_response.get("status")));
                expect!(pact_response.get("headers")).to(be_equal_to(file_response.get("headers")));
                expect!(pact_response.get("body")).to(be_equal_to(file_response.get("body")));
                expect!(pact_response.get("matchers")).to(be_equal_to(file_response.get("matchers")));
            }

            expect!(pact.metadata.get("pactSpecification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec!["pact-jvm".to_string(), "pactRust".to_string(), "pactSpecification".to_string()];
            expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"2.0.0\"}".to_string()));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_pact_with_binary_body() {
  let pact_file = fixture_path("test_pact_binary_body.json");

  let mut buffer = Vec::new();
  let gif_file = fixture_path("1px.gif");
  File::open(gif_file).unwrap().read_to_end(&mut buffer).unwrap();

  let pact_result = RequestResponsePact::read_pact(&pact_file);

  match pact_result {
    Ok(ref pact) => {
      let mut f = File::open(pact_file).unwrap();
      let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
      let pact_json = pact.to_json(PactSpecification::V2).unwrap();

      let interaction = pact.interactions.first().unwrap();
      expect!(interaction.response.body.value().unwrap_or_default().to_vec()).to(be_equal_to(buffer));
      let pact_interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
      let pact_interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
      expect!(pact_interactions).to(be_equal_to(pact_interactions_from_file));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_v4_pact() {
  let pact_file = fixture_path("v4-http-pact.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let mut f = File::open(pact_file).unwrap();
      let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
      let pact_json = pact.to_json(PactSpecification::V4).unwrap();
      expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
      expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
      expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

      expect!(pact.metadata().get("pactSpecification").clone()).to(be_some().value(&btreemap!("version".to_string() => "4.0".to_string())));
      let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
      let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
      expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
      expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"4.0\"}".to_string()));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_v4_message_pact() {
  let pact_file = fixture_path("v4-message-pact.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let mut f = File::open(pact_file).unwrap();
      let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
      let pact_json = pact.to_json(PactSpecification::V4).unwrap();
      expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
      expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));
      expect!(pact_json.get("interactions")).to(be_equal_to(pact_json_from_file.get("interactions")));

      expect!(pact.metadata().get("pactSpecification").clone()).to(be_some().value(&btreemap!("version".to_string() => "4.0".to_string())));
      let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
      let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
      expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
      expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"4.0\"}".to_string()));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_v4_combined_pact() {
  let pact_file = fixture_path("v4-combined-pact.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let mut f = File::open(pact_file).unwrap();
      let pact_json_from_file : serde_json::Value = serde_json::de::from_reader(&mut f).unwrap();
      let pact_json = pact.to_json(PactSpecification::V4).unwrap();
      expect!(pact_json.get("consumer")).to(be_equal_to(pact_json_from_file.get("consumer")));
      expect!(pact_json.get("provider")).to(be_equal_to(pact_json_from_file.get("provider")));

      let interactions = pact_json.get("interactions").unwrap().as_array().unwrap();
      let interactions_from_file = pact_json_from_file.get("interactions").unwrap().as_array().unwrap();
      expect!(interactions.len()).to(be_equal_to(interactions_from_file.len()));
      for interaction in interactions {
        let i2 = interactions_from_file.iter()
          .find(|i| i.get("description") == interaction.get("description"))
          .unwrap();
        expect!(interaction).to(be_equal_to(i2));
      }

      expect!(pact.metadata().get("pactSpecification").clone()).to(be_some().value(&btreemap!("version".to_string() => "4.0".to_string())));
      let metadata = pact_json.get("metadata").unwrap().as_object().unwrap();
      let expected_keys : Vec<String> = vec!["pactRust".to_string(), "pactSpecification".to_string()];
      expect!(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
      expect!(metadata.get("pactSpecification").unwrap().to_string()).to(be_equal_to("{\"version\":\"4.0\"}".to_string()));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_v4_pact_with_comments() {
  let pact_file = fixture_path("v4-http-pact-comments.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let interaction = pact.interactions().first().unwrap().as_v4().unwrap();
      expect!(interaction.comments()).to(be_equal_to(hashmap!{
        "text".to_string() => json!([
          "This allows me to specify just a bit more information about the interaction",
          "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
          "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
        ]),
        "testname".to_string() => json!("example_test.groovy")
      }));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}

#[test]
fn test_load_v4_pact_with_message_with_comments() {
  let pact_file = fixture_path("v4-message-pact-comments.json");
  let pact_result = read_pact(&pact_file);

  match pact_result {
    Ok(pact) => {
      let interaction = pact.interactions().first().unwrap().as_v4().unwrap();
      expect!(interaction.comments()).to(be_equal_to(hashmap!{
        "text".to_string() => json!([
          "This allows me to specify just a bit more information about the interaction",
          "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
          "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
        ]),
        "testname".to_string() => json!("example_test.groovy")
      }));
    },
    Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
  }
}
