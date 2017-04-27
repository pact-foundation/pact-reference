#[macro_use] extern crate pact_matching;
extern crate rustc_serialize;
#[macro_use(expect)] extern crate expectest;
#[macro_use] extern crate p_macro;
extern crate env_logger;

mod spec_testcases;

use pact_matching::models::*;
use std::path::Path;
use expectest::prelude::*;
use rustc_serialize::json::Json;
use std::fs::File;

#[test]
fn test_load_pact() {
    let pact_file = Path::new(file!()).parent().unwrap().join("pact.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification")).to(be_none());
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_pact_encoded_query() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_encoded_query.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));

            let pact_interactions = pact_json.find("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.find("interactions").unwrap().as_array().unwrap();
            expect(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect(pact_interaction.find("providerState")).to(be_equal_to(file_interaction.find("providerState")));
                expect(pact_interaction.find("description")).to(be_equal_to(file_interaction.find("description")));
                expect(pact_interaction.find("response")).to(be_equal_to(file_interaction.find("response")));

                let pact_request = pact_interaction.find("request").unwrap();
                let file_request = file_interaction.find("request").unwrap();
                expect(pact_request.find("method")).to(be_equal_to(file_request.find("method")));
                expect(pact_request.find("path")).to(be_equal_to(file_request.find("path")));
                expect(pact_request.find("headers")).to(be_equal_to(file_request.find("headers")));
                expect(pact_request.find("body")).to(be_equal_to(file_request.find("body")));
                expect(pact_request.find("matchers")).to(be_equal_to(file_request.find("matchers")));
                expect(pact_request.find("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.find("query").unwrap().to_string().to_uppercase()));
            }

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_lowercase_method() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_lowercase_method.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));

            let pact_interactions = pact_json.find("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.find("interactions").unwrap().as_array().unwrap();
            expect(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect(pact_interaction.find("providerState")).to(be_equal_to(file_interaction.find("providerState")));
                expect(pact_interaction.find("description")).to(be_equal_to(file_interaction.find("description")));
                expect(pact_interaction.find("response")).to(be_equal_to(file_interaction.find("response")));

                let pact_request = pact_interaction.find("request").unwrap();
                let file_request = file_interaction.find("request").unwrap();
                expect(pact_request.find("method").unwrap().to_string()).to(be_equal_to(file_request.find("method").unwrap().to_string().to_uppercase()));
                expect(pact_request.find("path")).to(be_equal_to(file_request.find("path")));
                expect(pact_request.find("headers")).to(be_equal_to(file_request.find("headers")));
                expect(pact_request.find("body")).to(be_equal_to(file_request.find("body")));
                expect(pact_request.find("matchers")).to(be_equal_to(file_request.find("matchers")));
                // This is a V3 pact, so we can't load the query string
                expect(pact_request.find("query")).to(be_none());
            }

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("3.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_bodies() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_no_bodies.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            for pact_interaction in pact.interactions.clone() {
                expect(pact_interaction.request.body).to(be_equal_to(OptionalBody::Missing));
                expect(pact_interaction.response.body).to(be_equal_to(OptionalBody::Missing));
            }

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_metadata() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_no_metadata.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification")).to(be_none());
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_spec_version() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_no_spec_version.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification")).to(be_none());
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_with_camel_case_spec_version() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_camel_case_spec_version.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            expect(pact.clone().specification_version).to(be_equal_to(PactSpecification::V1_1));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_no_version() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_no_version.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("null"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
            expect(pact.specification_version.clone()).to(be_equal_to(PactSpecification::Unknown));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_query_old_format() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_query_old_format.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_with_bodies() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_with_bodies.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_v2_pact() {
    let pact_file = Path::new(file!()).parent().unwrap().join("v2-pact.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_v2_pact_query() {
    let pact_file = Path::new(file!()).parent().unwrap().join("v2_pact_query.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));

            let pact_interactions = pact_json.find("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.find("interactions").unwrap().as_array().unwrap();
            expect(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect(pact_interaction.find("providerState")).to(be_equal_to(file_interaction.find("providerState")));
                expect(pact_interaction.find("description")).to(be_equal_to(file_interaction.find("description")));
                expect(pact_interaction.find("response")).to(be_equal_to(file_interaction.find("response")));

                let pact_request = pact_interaction.find("request").unwrap();
                let file_request = file_interaction.find("request").unwrap();
                expect(pact_request.find("method")).to(be_equal_to(file_request.find("method")));
                expect(pact_request.find("path")).to(be_equal_to(file_request.find("path")));
                expect(pact_request.find("headers")).to(be_equal_to(file_request.find("headers")));
                expect(pact_request.find("body")).to(be_equal_to(file_request.find("body")));
                expect(pact_request.find("matchers")).to(be_equal_to(file_request.find("matchers")));
                expect(pact_request.find("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.find("query").unwrap().to_string().to_uppercase()));
            }

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_matcherst() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_matchers.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));
            expect(pact_json.find("interactions")).to(be_equal_to(pact_json_from_file.find("interactions")));

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

#[test]
fn test_load_test_pact_matchers_old_format() {
    let pact_file = Path::new(file!()).parent().unwrap().join("test_pact_matchers_old_format.json");
    let pact_result = Pact::read_pact(&pact_file);

    match pact_result {
        Ok(ref pact) => {
            let mut f = File::open(pact_file).unwrap();
            let pact_json_from_file = Json::from_reader(&mut f).unwrap();
            let pact_json = pact.to_json();
            expect(pact_json.find("consumer")).to(be_equal_to(pact_json_from_file.find("consumer")));
            expect(pact_json.find("provider")).to(be_equal_to(pact_json_from_file.find("provider")));

            let pact_interactions = pact_json.find("interactions").unwrap().as_array().unwrap();
            let pact_interactions_from_file = pact_json_from_file.find("interactions").unwrap().as_array().unwrap();
            expect(pact_interactions.len()).to(be_equal_to(pact_interactions_from_file.len()));

            for (pact_interaction, file_interaction) in pact_interactions.iter().zip(pact_interactions_from_file.iter()) {
                expect(pact_interaction.find("providerState")).to(be_equal_to(file_interaction.find("providerState")));
                expect(pact_interaction.find("description")).to(be_equal_to(file_interaction.find("description")));

                let pact_request = pact_interaction.find("request").unwrap();
                let file_request = file_interaction.find("request").unwrap();
                expect(pact_request.find("method")).to(be_equal_to(file_request.find("method")));
                expect(pact_request.find("path")).to(be_equal_to(file_request.find("path")));
                expect(pact_request.find("headers")).to(be_equal_to(file_request.find("headers")));
                expect(pact_request.find("body")).to(be_equal_to(file_request.find("body")));
                expect(pact_request.find("matchers")).to(be_equal_to(file_request.find("matchers")));
                expect(pact_request.find("query").unwrap().to_string().to_uppercase()).to(
                    be_equal_to(file_request.find("query").unwrap().to_string().to_uppercase()));

                let pact_response = pact_interaction.find("response").unwrap();
                let file_response = file_interaction.find("response").unwrap();
                expect(pact_response.find("status")).to(be_equal_to(file_response.find("status")));
                expect(pact_response.find("headers")).to(be_equal_to(file_response.find("headers")));
                expect(pact_response.find("body")).to(be_equal_to(file_response.find("body")));
                expect(pact_response.find("matchers")).to(be_equal_to(file_response.find("matchers")));
            }

            expect(pact.metadata.get("pact-specification").unwrap().get("version")).to(be_some().value("2.0.0"));
            let metadata = pact_json.find("metadata").unwrap().as_object().unwrap();
            let expected_keys : Vec<String> = vec![s!("pact-jvm"), s!("pact-rust"), s!("pact-specification")];
            expect(metadata.keys().cloned().collect::<Vec<String>>()).to(be_equal_to(expected_keys));
            expect(metadata.get("pact-specification").unwrap().to_string()).to(be_equal_to(s!("{\"version\":\"2.0.0\"}")));
        },
        Err(err) => panic!("Failed to load pact from '{:?}' - {}", pact_file, err)
    }
}

// v3-message-pact.json
// v3-pact.json
// test_pact_v3.json
