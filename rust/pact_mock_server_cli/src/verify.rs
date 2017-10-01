use clap::ArgMatches;
use hyper::Client;
use hyper::Url;
use hyper::status::*;
use std::io::prelude::*;
use serde_json;
use pact_mock_server::{
    lookup_mock_server,
    lookup_mock_server_by_port,
    MockServer
};

pub fn verify_mock_server(host: &str, port: u16, matches: &ArgMatches) -> Result<(), i32> {
    let mock_server_id = matches.value_of("mock-server-id");
    let mock_server_port = matches.value_of("mock-server-port");
    let id = if mock_server_id.is_some() {
        (mock_server_id.unwrap(), "id")
    } else {
        (mock_server_port.unwrap(), "port")
    };

    let client = Client::new();
    let url = Url::parse(format!("http://{}:{}/mockserver/{}/verify", host, port, id.0)
        .as_str()).unwrap();
    let res = client.post(url.clone()).send();

    match res {
        Ok(mut result) => {
            if !result.status.is_success() {
                match result.status {
                    StatusCode::NotFound => {
                        println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.",
                            id.1, id.0);
                        Err(3)
                    },
                    StatusCode::UnprocessableEntity => {
                        let mut body = String::new();
                        result.read_to_string(&mut body).unwrap();
                        let json_result: Result<serde_json::Value, _> = serde_json::from_str(body.as_str());
                        match json_result {
                            Ok(json) => {
                                let mock_server = json.get("mockServer").unwrap();
                                let id = mock_server.get("id").unwrap().as_str().unwrap();
                                let port = mock_server.get("port").unwrap().as_u64().unwrap();
                                display_verification_errors(id, port, &json);
                                Err(2)
                            },
                            Err(err) => {
                                error!("Failed to parse JSON: {}\n{}", err, body);
                                ::display_error(format!("Failed to parse JSON: {}\n{}", err, body), matches);
                            }
                        }
                    },
                    _ => ::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status), matches)
                }
            } else {
                println!("Mock server with {} '{}' verified ok", id.1, id.0);
                Ok(())
            }
        },
        Err(err) => {
            ::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
        }
    }
}

fn validate_port(id: u16) -> Result<MockServer, String> {
    lookup_mock_server_by_port(id as i32, &|ref ms| {
        MockServer {
            id: ms.id.clone(),
            port: ms.port,
            server: ms.server,
            matches: ms.matches.clone(),
            resources: vec![],
            pact: ms.pact.clone(),
            status: ms.status.clone()
        }
    }).ok_or(format!("No mock server running with port '{}'", id))
}

fn validate_uuid(id: &String) -> Result<MockServer, String> {
    lookup_mock_server(id.clone(), &|ref ms| {
        MockServer {
            id: ms.id.clone(),
            port: ms.port,
            server: ms.server,
            matches: ms.matches.clone(),
            resources: vec![],
            pact: ms.pact.clone(),
            status: ms.status.clone()
        }
    }).ok_or(format!("No mock server running with id '{}'", id))
}

pub fn validate_id(id: &str) -> Result<MockServer, String> {
    if id.chars().all(|ch| ch.is_digit(10)) {
        validate_port(id.parse::<u16>().unwrap())
    } else {
        validate_uuid(&s!(id))
    }
}

fn display_verification_errors(id: &str, port: u64, json: &serde_json::Value) {
    let mismatches = json.get("mismatches").unwrap().as_array().unwrap();
    println!("Mock server {}/{} failed verification with {} errors\n", id, port, mismatches.len());

    for (i, mismatch) in mismatches.iter().enumerate() {
//        mismatch.get("type").unwrap().to_string().fpp();
        match mismatch.get("type").unwrap().to_string().as_ref() {
            "missing-request" => {
                let request = mismatch.get("request").unwrap();
                println!("{} - Expected request was not received - {}", i, request)
            },
            "request-not-found" => {
                let request = mismatch.get("request").unwrap();
                println!("{} - Received a request that was not expected - {}", i, request)
            },
            "request-mismatch" => {
                let path = mismatch.get("path").unwrap().to_string();
                let method = mismatch.get("method").unwrap().to_string();
                println!("{} - Received a request that did not match with expected - {} {}", i, method, path);
                let request_mismatches = mismatch.get("mismatches").unwrap().as_array().unwrap();
                for request_mismatch in request_mismatches {
                    println!("        {}", request_mismatch.get("mismatch").unwrap().to_string())
                }
            },
            _ => println!("{} - Known failure - {}", i, mismatch),
        }
    }
}
