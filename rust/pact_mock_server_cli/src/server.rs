use hyper::server::{Handler, Server, Request, Response};
use pact_matching::models::Pact;
use pact_mock_server::{
    start_mock_server,
    iterate_mock_servers,
    lookup_mock_server,
    shutdown_mock_server,
    set_first_port,
    get_next_port,
    MockServer
};
use uuid::Uuid;
use serde_json;
use std::sync::Arc;
use std::iter::FromIterator;
use std::ops::Deref;
use verify;
use webmachine_rust::*;
use webmachine_rust::context::*;
use webmachine_rust::headers::*;
use clap::ArgMatches;

fn json_error(error: String) -> String {
    let json_response = json!({ s!("error") : json!(error) });
    json_response.to_string()
}

fn start_provider(context: &mut WebmachineContext) -> Result<bool, u16> {
    match context.request.body {
        Some(ref body) if !body.is_empty() => {
            match serde_json::from_str(body) {
                Ok(ref json) => {
                    let pact = Pact::from_json(&context.request.request_path, json);
                    let mock_server_id = Uuid::new_v4().simple().to_string();
                    match start_mock_server(mock_server_id.clone(), pact, get_next_port()) {
                        Ok(mock_server) => {
                            let mock_server_json = json!({
                                s!("id") : json!(mock_server_id.clone()),
                                s!("port") : json!(mock_server as i64),
                            });
                            let json_response = json!({ s!("mockServer") : mock_server_json });
                            context.response.body = Some(json_response.to_string());
                            context.response.add_header(s!("Location"),
                                vec![HeaderValue::basic(&format!("/mockserver/{}", mock_server_id))]);
                            Ok(true)
                        },
                        Err(msg) => {
                            context.response.body = Some(json_error(format!("Failed to start mock server - {}", msg)));
                            Err(422)
                        }
                    }
                },
                Err(err) => {
                    error!("Failed to parse json body - {}", err);
                    context.response.body = Some(json_error(format!("Failed to parse json body - {}", err)));
                    Err(422)
                }
            }
        },
        _ => {
            error!("No pact json was supplied");
            context.response.body = Some(json_error(s!("No pact json was supplied")));
            Err(422)
        }
    }
}

pub fn verify_mock_server_request(context: &mut WebmachineContext, output_path: &Option<String>) -> Result<bool, u16> {
    let id = context.metadata.get(&s!("id")).unwrap_or(&s!("")).clone();
    match verify::validate_id(&id) {
        Ok(ms) => {
            let mut map = btreemap!{ s!("mockServer") => ms.to_json() };
            let mismatches = ms.mismatches();
            if !mismatches.is_empty() {
                map.insert(s!("mismatches"), json!(
                    Vec::from_iter(mismatches.iter()
                        .map(|m| m.to_json()))));
                context.response.body = Some(json!(map).to_string());
                Err(422)
            } else {
                match ms.write_pact(&output_path) {
                    Ok(_) => Ok(true),
                    Err(err) => {
                        map.insert(s!("error"), json!(format!("Failed to write pact to file - {}", err)));
                        context.response.body = Some(json!(map).to_string());
                        Err(422)
                    }
                }
            }
        },
        Err(_) => Err(422)
    }
}

fn main_resource() -> WebmachineResource {
    WebmachineResource {
        allowed_methods: vec![s!("OPTIONS"), s!("GET"), s!("HEAD"), s!("POST")],
        resource_exists: Box::new(|context| context.request.request_path == "/"),
        render_response: Box::new(|_| {
            let mut mock_servers = vec![];
            iterate_mock_servers(&mut |_: &String, ms: &MockServer| {
                let mock_server_json = ms.to_json();
                mock_servers.push(mock_server_json);
            });
            let json_response = json!({ s!("mockServers") : json!(mock_servers) });
            Some(json_response.to_string())
        }),
        process_post: Box::new(|context| start_provider(context)),
        .. WebmachineResource::default()
    }
}

fn mock_server_resource(output_path: Arc<Option<String>>) -> WebmachineResource {
    WebmachineResource {
        allowed_methods: vec![s!("OPTIONS"), s!("GET"), s!("HEAD"), s!("POST"), s!("DELETE")],
        resource_exists: Box::new(|context| {
            let paths: Vec<String> = context.request.request_path
                .split("/")
                .filter(|p| !p.is_empty())
                .map(|p| p.to_string())
                .collect();
            if paths.len() >= 1 && paths.len() <= 2 {
                match verify::validate_id(&paths[0].clone()) {
                    Ok(ms) => {
                        context.metadata.insert(s!("id"), ms.id.clone());
                        context.metadata.insert(s!("port"), ms.port.to_string());
                        if paths.len() > 1 {
                            context.metadata.insert(s!("subpath"), paths[1].clone());
                            paths[1] == s!("verify")
                        } else {
                            true
                        }
                    },
                    Err(_) => false
                }
            } else {
                false
            }
        }),
        render_response: Box::new(|context| {
            match context.metadata.get(&s!("subpath")) {
                None => {
                    let id = context.metadata.get(&s!("id")).unwrap().clone();
                    lookup_mock_server(id, &|ms| ms.to_json()).map(|json| json.to_string())
                },
                Some(_) => {
                    context.response.status = 405;
                    None
                }
            }
        }),
        process_post: Box::new(move |context| {
            let subpath = context.metadata.get(&s!("subpath")).unwrap().clone();
            if subpath == "verify" {
                verify_mock_server_request(context, output_path.deref())
            } else {
                Err(422)
            }
        }),
        delete_resource: Box::new(|context| {
            match context.metadata.get(&s!("subpath")) {
                None => {
                    let id = context.metadata.get(&s!("id")).unwrap().clone();
                    shutdown_mock_server(&id);
                    Ok(true)
                },
                Some(_) => Err(405)
            }
        }),
        .. WebmachineResource::default()
    }
}

struct ServerHandler {
    output_path: Arc<Option<String>>
}

impl ServerHandler {
    fn new(output_path: Option<String>) -> ServerHandler {
        ServerHandler {
            output_path: Arc::new(output_path)
        }
    }
}

impl Handler for ServerHandler {

    fn handle(&self, req: Request, res: Response) {
        let dispatcher = WebmachineDispatcher::new(
            btreemap!{
                s!("/") => Arc::new(main_resource()),
                s!("/mockserver") => Arc::new(mock_server_resource(self.output_path.clone()))
            }
        );
        match dispatcher.dispatch(req, res) {
            Ok(_) => (),
            Err(err) => warn!("Error generating response - {}", err)
        };
    }
}

pub fn start_server(port: u16, matches: &ArgMatches) -> Result<(), i32> {
    let output_path = matches.value_of("output").map(|s| s.to_owned());
    if let Some(v) = matches.value_of("first-port") {
        set_first_port(v.parse().unwrap())
    }
    match Server::http(format!("0.0.0.0:{}", port).as_str()) {
        Ok(mut server) => {
            server.keep_alive(None);
            match server.handle(ServerHandler::new(output_path)) {
                Ok(listener) => {
                    info!("Server started on port {}", listener.socket.port());
                    Ok(())
                },
                Err(err) => {
                    error!("could not bind listener to port: {}", err);
                    Err(2)
                }
            }
        },
        Err(err) => {
            error!("could not start server: {}", err);
            Err(1)
        }
    }
}
