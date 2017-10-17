//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V2 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-2).
//!
//! There are a number of exported functions using C bindings for controlling the mock server. These can be used in any
//! language that supports C bindings.
//!
//! ## [create_mock_server](fn.create_mock_server.html)
//!
//! External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
//! as well as the port for the mock server to run on. A value of 0 for the port will result in a
//! port being allocated by the operating system. The port of the mock server is returned.
//!
//! ## [mock_server_matched](fn.mock_server_matched.html)
//!
//! Simple function that returns a boolean value given the port number of the mock service. This value will be true if all
//! the expectations of the pact that the mock server was created with have been met. It will return false if any request did
//! not match, an un-recognised request was received or an expected request was not received.
//!
//! ## [mock_server_mismatches](fn.mock_server_mismatches.html)
//!
//! This returns all the mismatches, un-expected requests and missing requests in JSON format, given the port number of the
//! mock server.
//!
//! **IMPORTANT NOTE:** The JSON string for the result is allocated on the rust heap, and will have to be freed once the
//! code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is provided for this purpose. If the mock
//! server is not cleaned up properly, this will result in memory leaks as the rust heap will not be reclaimed.
//!
//! ## [cleanup_mock_server](fn.cleanup_mock_server.html)
//!
//! This function will try terminate the mock server with the given port number and cleanup any memory allocated for it by
//! the [`mock_server_mismatches`](fn.mock_server_mismatches.html) function. Returns `true`, unless a mock server with the given port number does not exist,
//! or the function fails in some way.
//!
//! **NOTE:** Although `close()` on the listerner for the mock server is called, this does not currently work and the
//! listerner will continue handling requests. In this case, it will always return a 501 once the mock server has been
//! cleaned up.
//!
//! ## [write_pact_file](fn.write_pact_file.html)
//!
//! External interface to trigger a mock server to write out its pact file. This function should
//! be called if all the consumer tests have passed. The directory to write the file to is passed
//! as the second parameter. If a NULL pointer is passed, the current working directory is used.
//!
//! Returns 0 if the pact file was successfully written. Returns a positive code if the file can
//! not be written, or there is no mock server running on that port or the function panics.

#![warn(missing_docs)]

#[macro_use] extern crate log;
#[macro_use] extern crate p_macro;
#[macro_use] extern crate maplit;
#[macro_use] extern crate lazy_static;
extern crate libc;
#[macro_use] extern crate pact_matching;
#[macro_use] extern crate serde_json;
extern crate env_logger;
#[macro_use] extern crate hyper;
extern crate uuid;
#[macro_use] extern crate itertools;

use libc::{c_char, int32_t};
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use std::panic::catch_unwind;
use std::fmt;
use pact_matching::models::{Pact, Interaction, Request, OptionalBody};
use pact_matching::models::parse_query_string;
use pact_matching::Mismatch;

use std::collections::{BTreeMap, HashMap};
use std::thread;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use hyper::server::{Server, Listening};
use hyper::status::StatusCode;
use hyper::header::{Headers, ContentType, AccessControlAllowOrigin, ContentLength};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::uri::RequestUri;
use uuid::Uuid;
use itertools::Itertools;

/// Enum to define a server status
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    /// The server working correctly
    OK,
    /// Error at mock server
    Error
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Enum to define a match result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    /// Match result where the request was sucessfully matched
    RequestMatch(Interaction),
    /// Match result where there were a number of mismatches
    RequestMismatch(Interaction, Vec<Mismatch>),
    /// Match result where the request was not expected
    RequestNotFound(Request),
    /// Match result where an expected request was not received
    MissingRequest(Interaction)
}

impl MatchResult {
    /// Returns the match key for this mismatch
    pub fn match_key(&self) -> String {
        match self {
            &MatchResult::RequestMatch(_) => s!("Request-Matched"),
            &MatchResult::RequestMismatch(_, _) => s!("Request-Mismatch"),
            &MatchResult::RequestNotFound(_) => s!("Unexpected-Request"),
            &MatchResult::MissingRequest(_) => s!("Missing-Request")
        }
    }

    /// Returns true if this match result is a `RequestMatch`
    pub fn matched(&self) -> bool {
        match self {
            &MatchResult::RequestMatch(_) => true,
            _ => false
        }
    }

    /// Converts this match result to a `Value` struct
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            &MatchResult::RequestMatch(_) => json!({ s!("type") : s!("request-match")}),
            &MatchResult::RequestMismatch(ref interaction, ref mismatches) => mismatches_to_json(&interaction.request, mismatches),
            &MatchResult::RequestNotFound(ref req) => json!({
                s!("type") : json!("request-not-found"),
                s!("method") : json!(req.method),
                s!("path") : json!(req.path),
                s!("request") : json!(req)
            }),
            &MatchResult::MissingRequest(ref interaction) => json!({
                s!("type") : json!("missing-request"),
                s!("method") : json!(interaction.request.method),
                s!("path") : json!(interaction.request.path),
                s!("request") : json!(interaction.request)
            })
        }
    }
}

fn mismatches_to_json(request: &Request, mismatches: &Vec<Mismatch>) -> serde_json::Value {
    json!({
        s!("type") : json!("request-mismatch"),
        s!("method") : json!(request.method),
        s!("path") : json!(request.path),
        s!("mismatches") : mismatches.iter().map(|m| m.to_json()).collect::<serde_json::Value>()
    })
}

/// Struct to represent a mock server
pub struct MockServer {
    /// Mock server unique ID
    pub id: String,
    /// Port the mock server is running on
    pub port: i32,
    /// Address of the server implementing the `Listening` trait
    pub server: u64,
    /// List of all match results for requests this mock server has received
    pub matches: Vec<MatchResult>,
    /// List of resources that need to be cleaned up when the mock server completes
    pub resources: Vec<CString>,
    /// Pact that this mock server is based on
    pub pact: Pact,
    /// Server status
    pub status : ServerStatus
}

impl MockServer {
    /// Creates a new mock server with the given ID and pact
    pub fn new(id: String, pact: &Pact) -> MockServer {
        MockServer { id: id.clone(), port: -1, server: 0, matches: vec![], resources: vec![],
            pact : pact.clone(),  status : ServerStatus::OK}
    }

    /// Sets the port that the mock server is listening on
    pub fn port(&mut self, port: i32) {
        self.port = port;
    }

    /// Sets the address of the server implementing the `Listening` trait
    pub fn server(&mut self, server: &Listening) {
        let p = server as *const Listening;
        self.server = p as u64;
    }

    /// Converts this mock server to a `Value` struct
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            s!("id") : json!(self.id.clone()),
            s!("port") : json!(self.port as u64),
            s!("provider") : json!(self.pact.provider.name.clone()),
            s!("status") : json!(format!("{}", self.status))
        })
    }

    /// Returns all the mismatches that have occured with this mock server
    pub fn mismatches(&self) -> Vec<MatchResult> {
        let mismatches = self.matches.iter()
            .filter(|m| !m.matched())
            .map(|m| m.clone());
        let interactions: Vec<&Interaction> = self.matches.iter().map(|m| {
            match *m {
                MatchResult::RequestMatch(ref interaction) => Some(interaction),
                MatchResult::RequestMismatch(ref interaction, _) => Some(interaction),
                MatchResult::RequestNotFound(_) => None,
                MatchResult::MissingRequest(_) => None
            }
        }).filter(|o| o.is_some()).map(|o| o.unwrap()).collect();
        let missing = self.pact.interactions.iter()
            .filter(|i| !interactions.contains(i))
            .map(|i| MatchResult::MissingRequest(i.clone()));
        mismatches.chain(missing).collect()
    }

    /// Mock server writes it pact out to the provided directory
    pub fn write_pact(&self, output_path: &Option<String>) -> io::Result<()> {
        let pact_file_name = self.pact.default_file_name();
        let filename = match *output_path {
            Some(ref path) => {
                let mut path = PathBuf::from(path);
                path.push(pact_file_name);
                path
            },
            None => PathBuf::from(pact_file_name)
        };
        info!("Writing pact out to '{}'", filename.display());
        match self.pact.write_pact(filename.as_path()) {
            Ok(_) => Ok(()),
            Err(err) => {
                warn!("Failed to write pact to file - {}", err);
                Err(err)
            }
        }
    }

    /// Returns the URL of the mock server
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

impl PartialEq for MockServer {
    fn eq(&self, other: &MockServer) -> bool {
        self.id == other.id
    }
}

lazy_static! {
    static ref MOCK_SERVERS: Mutex<BTreeMap<String, Box<MockServer>>> = Mutex::new(BTreeMap::new());
    static ref FIRST_PORT : Mutex<Option<i32>> = Mutex::new(None);
}

fn match_request(req: &Request, interactions: &Vec<Interaction>) -> MatchResult {
    let match_results = interactions
        .into_iter()
        .map(|i| (i.clone(), pact_matching::match_request(i.request.clone(), req.clone())))
        .sorted_by(|i1, i2| {
            let list1 = i1.1.clone().into_iter().map(|m| m.mismatch_type()).unique().count();
            let list2 = i2.1.clone().into_iter().map(|m| m.mismatch_type()).unique().count();
            Ord::cmp(&list1, &list2)
        });
    match match_results.first() {
        Some(res) => {
            if res.1.is_empty() {
                MatchResult::RequestMatch(res.0.clone())
            } else if method_or_path_mismatch(&res.1) {
                MatchResult::RequestNotFound(req.clone())
            } else {
                MatchResult::RequestMismatch(res.0.clone(), res.1.clone())
            }
        },
        None => MatchResult::RequestNotFound(req.clone())
    }
}

fn method_or_path_mismatch(mismatches: &Vec<Mismatch>) -> bool {
    let mismatch_types: Vec<String> = mismatches.iter()
        .map(|mismatch| mismatch.mismatch_type())
        .collect();
    mismatch_types.contains(&s!("MethodMismatch")) || mismatch_types.contains(&s!("PathMismatch"))
}

fn extract_path(uri: &RequestUri) -> String {
    match uri {
        &RequestUri::AbsolutePath(ref s) => s!(s.splitn(2, "?").next().unwrap_or("/")),
        &RequestUri::AbsoluteUri(ref url) => s!(url.path()),
        _ => uri.to_string()
    }
}

fn extract_query_string(uri: &RequestUri) -> Option<HashMap<String, Vec<String>>> {
    match uri {
        &RequestUri::AbsolutePath(ref s) => {
            if s.contains("?") {
                match s.splitn(2, "?").last() {
                    Some(q) => parse_query_string(&s!(q)),
                    None => None
                }
            } else {
                None
            }
        },
        &RequestUri::AbsoluteUri(ref url) => match url.query() {
            Some(q) => parse_query_string(&s!(q)),
            None => None
        },
        _ => None
    }
}

fn extract_headers(headers: &Headers) -> Option<HashMap<String, String>> {
    if headers.len() > 0 {
        Some(headers.iter().map(|h| (s!(h.name()), h.value_string()) ).collect())
    } else {
        None
    }
}

fn extract_body(req: &mut hyper::server::Request) -> OptionalBody {
    let mut buffer = String::new();
    match req.read_to_string(&mut buffer) {
        Ok(size) => if size > 0 {
                OptionalBody::Present(buffer)
            } else {
                OptionalBody::Empty
            },
        Err(err) => {
            warn!("Failed to read request body: {}", err);
            OptionalBody::Empty
        }
    }
}

fn hyper_request_to_pact_request(req: &mut hyper::server::Request) -> Request {
    Request {
        method: req.method.to_string(),
        path: extract_path(&req.uri),
        query: extract_query_string(&req.uri),
        headers: extract_headers(&req.headers),
        body: extract_body(req),
        matching_rules: None
    }
}

fn error_body(req: &Request, error: &String) -> String {
    let body = json!({ "error" : format!("{} : {:?}", error, req) });
    body.to_string()
}

fn insert_new_mock_server(id: &String, pact: &Pact) {
    MOCK_SERVERS.lock().unwrap().insert(id.clone(), Box::new(MockServer::new(id.clone(), pact)));
}

fn update_mock_server<R>(id: &String, f: &Fn(&mut MockServer) -> R) -> Option<R> {
    match MOCK_SERVERS.lock().unwrap().get_mut(id) {
        Some(mock_server) => Some(f(mock_server)),
        _ => None
    }
}

fn update_mock_server_by_port<R>(port: i32, f: &Fn(&mut MockServer) -> R) -> Option<R> {
    let mut map = MOCK_SERVERS.lock().unwrap();
    match map.iter_mut().find(|ms| ms.1.port == port ) {
        Some(mock_server) => Some(f(mock_server.1)),
        None => None
    }
}

fn record_result(id: &String, match_result: &MatchResult) {
    update_mock_server(id, &|mock_server: &mut MockServer| {
        mock_server.matches.push(match_result.clone());
    });
}

/// Starts a mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_mock_server(id: String, pact: Pact, port: i32) -> Result<i32, String> {
    insert_new_mock_server(&id, &pact);
    let (out_tx, out_rx) = channel();
    let (in_tx, in_rx) = channel();
    in_tx.send((id.clone(), pact, port)).unwrap();
    thread::spawn(move || {
        let (mock_server_id, pact, port) = in_rx.recv().unwrap();
        let server = Server::http(format!("0.0.0.0:{}", port).as_str()).unwrap();
        let server_result = server.handle(move |mut req: hyper::server::Request, mut res: hyper::server::Response| {
            debug!("--> Hyper request to mock server {}", mock_server_id);
            match lookup_mock_server(mock_server_id.clone(), &|_| ()) {
                None => {
                    warn!("Mock server {} has been shutdown", mock_server_id);
                    *res.status_mut() = StatusCode::NotImplemented;
                    res.headers_mut().set_raw("X-Pact", vec!["Mock server has been shut down".as_bytes().to_vec()]);
                },
                Some(_) => {
                    debug!("Creating pact request from hyper request");
                    let req = hyper_request_to_pact_request(&mut req);
                    info!("Received request {:?}", req);
                    let match_result = match_request(&req, &pact.interactions);
                    record_result(&mock_server_id, &match_result);
                    match match_result {
                        MatchResult::RequestMatch(ref interaction) => {
                            info!("Request matched, sending response {:?}", interaction.response);
                            *res.status_mut() = StatusCode::from_u16(interaction.response.status);
                            res.headers_mut().set(AccessControlAllowOrigin::Any);
                            match interaction.response.headers {
                                Some(ref headers) => {
                                    for (k, v) in headers.clone() {
                                        res.headers_mut().set_raw(k, vec![v.into_bytes()]);
                                    }
                                },
                                None => ()
                            }
                            match interaction.response.body {
                                OptionalBody::Present(ref body) => {
                                    res.send(body.as_bytes()).unwrap();
                                },
                                _ => ()
                            }
                        },
                        _ => {
                            *res.status_mut() = StatusCode::InternalServerError;
                            res.headers_mut().set(
                                ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                                 vec![(Attr::Charset, Value::Utf8)]))
                            );
                            res.headers_mut().set(AccessControlAllowOrigin::Any);
                            res.headers_mut().set_raw("X-Pact", vec![match_result.match_key().as_bytes().to_vec()]);
                            let body = error_body(&req, &match_result.match_key());
                            res.headers_mut().set(ContentLength(body.as_bytes().len() as u64));
                            let mut res = res.start().unwrap();
                            res.write_all(body.as_bytes()).unwrap();
                        }
                    }
                }
            }
        });

        match server_result {
            Ok(ref server) => {
                let port = server.socket.port() as i32;
                info!("Mock Provider Server started on port {}", port);
                update_mock_server(&id, &|mock_server| {
                    mock_server.port(port);
                    mock_server.server(server);
                });
                out_tx.send(Ok(port)).unwrap();
            },
            Err(e) => {
                error!("Could not start server: {}", e);
                out_tx.send(Err(format!("Could not start server: {}", e))).unwrap();
            }
        }
    });

    out_rx.recv().unwrap()
}

/// Looks up the mock server by ID, and passes it into the given closure. The result of the
/// closure is returned wrapped in an `Option`. If no mock server is found with that ID, `None`
/// is returned.
pub fn lookup_mock_server<R>(id: String, f: &Fn(&MockServer) -> R) -> Option<R> {
    debug!("Looking up mock server with ID {}", id);
    let map = MOCK_SERVERS.lock().unwrap();
    match map.get(&id) {
        Some(ref mock_server) => {
            debug!("Found mock server, invoking function ...");
            Some(f(mock_server))
        },
        None => {
            debug!("Did not find mock server");
            None
        }
    }
}

/// Looks up the mock server by port number, and passes it into the given closure. The result of the
/// closure is returned wrapped in an `Option`. If no mock server is found with that port number, `None`
/// is returned.
pub fn lookup_mock_server_by_port<R>(mock_server_port: i32, f: &Fn(&MockServer) -> R) -> Option<R> {
    let map = MOCK_SERVERS.lock().unwrap();
    match map.iter().find(|ms| ms.1.port == mock_server_port ) {
        Some(mock_server) => Some(f(mock_server.1)),
        None => None
    }
}

/// Iterates through all the mock servers, passing each one to the given closure.
pub fn iterate_mock_servers(f: &mut FnMut(&String, &MockServer)) {
    let map = MOCK_SERVERS.lock().unwrap();
    for (key, value) in map.iter() {
        f(key, value);
    }
}

fn cleanup_mock_server_impl(mock_server: &mut MockServer) -> String {
    mock_server.resources.clear();
    if mock_server.server > 0 {
        let server_raw = mock_server.server as *mut Listening;
        let mut server_ref = unsafe { &mut *server_raw };
        server_ref.close().unwrap();
    }
    mock_server.id.clone()
}

/// Shuts and cleans up the mock server with the given id. Returns true if a mock server was
/// found, false otherwise.
///
/// **NOTE:** Although `close()` on the listerner for the mock server is called, this does not
/// currently work and the listerner will continue handling requests. In this
/// case, it will always return a 404 once the mock server has been cleaned up.
pub fn shutdown_mock_server(id: &String) -> bool {
    debug!("Shutting down mock server with ID {}", id);
    let id_result = update_mock_server(id, &cleanup_mock_server_impl);

    match id_result {
        Some(ref id) => {
            MOCK_SERVERS.lock().unwrap().remove(id);
            true
        },
        None => false
    }
}

/// Shuts and cleans up the mock server with the given port. Returns true if a mock server was
/// found, false otherwise.
///
/// **NOTE:** Although `close()` on the listerner for the mock server is called, this does not
/// currently work and the listerner will continue handling requests. In this
/// case, it will always return a 404 once the mock server has been cleaned up.
pub fn shutdown_mock_server_by_port(port: i32) -> bool {
    debug!("Shutting down mock server with port {}", port);
    let id_result = update_mock_server_by_port(port, &cleanup_mock_server_impl);

    match id_result {
        Some(ref id) => {
            MOCK_SERVERS.lock().unwrap().remove(id);
            true
        },
        None => false
    }
}

/// Set the first port if needed
pub fn set_first_port(port : i32) {
    let mut value  = FIRST_PORT.lock().unwrap();
    *value = Some(port);
}

/// Get the next port if first one was available, otherwise return 0
pub fn get_next_port() -> i32 {
    if let Some(value)  = *(FIRST_PORT.lock().unwrap()) {
        value + MOCK_SERVERS.lock().unwrap().len() as i32
    }
    else{
        0
    }
}


/// External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
/// as well as the port for the mock server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// # Errors
///
/// Errors are returned as negative values.
///
/// | Error | Description |
/// |-------|-------------|
/// | -1 | A null pointer was received |
/// | -2 | The pact JSON could not be parsed |
/// | -3 | The mock server could not be started |
/// | -4 | The method paniced |
///
#[no_mangle]
pub extern fn create_mock_server(pact_str: *const c_char, port: int32_t) -> int32_t {
    env_logger::init().unwrap_or(());

    let result = catch_unwind(|| {
        let c_str = unsafe {
            if pact_str.is_null() {
                error!("Got a null pointer instead of pact json");
                return -1;
            }
            CStr::from_ptr(pact_str)
        };

        let pact_json = str::from_utf8(c_str.to_bytes()).unwrap();
        let result = serde_json::from_str(pact_json);
        match result {
            Ok(pact_json) => {
                let pact = Pact::from_json(&s!("<create_mock_server>"), &pact_json);
                match start_mock_server(Uuid::new_v4().simple().to_string(), pact, port) {
                    Ok(mock_server) => mock_server as i32,
                    Err(msg) => {
                        error!("Could not start mock server: {}", msg);
                        -3
                    }
                }
            },
            Err(err) => {
                error!("Could not parse pact json: {}", err);
                -2
            }
        }
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            error!("Caught a general panic: {:?}", cause);
            -4
        }
    }
}

/// External interface to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched, or
/// the method panics.
#[no_mangle]
pub extern fn mock_server_matched(mock_server_port: int32_t) -> bool {
    let result = catch_unwind(|| {
        lookup_mock_server_by_port(mock_server_port, &|mock_server| {
            mock_server.mismatches().is_empty()
        }).unwrap_or(false)
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            error!("Caught a general panic: {:?}", cause);
            false
        }
    }
}

/// External interface to get all the mismatches from a mock server. The port number of the mock
/// server is passed in, and a pointer to a C string with the mismatches in JSON format is
/// returned.
///
/// **NOTE:** The JSON string for the result is allocated on the heap, and will have to be freed
/// once the code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is
/// provided for this purpose.
///
/// # Errors
///
/// If there is no mock server with the provided port number, or the function panics, a NULL
/// pointer will be returned. Don't try to dereference it, it will not end well for you.
///
#[no_mangle]
pub extern fn mock_server_mismatches(mock_server_port: int32_t) -> *mut c_char {
    let result = catch_unwind(|| {
        let result = update_mock_server_by_port(mock_server_port, &|ref mut mock_server| {
            let mismatches = mock_server.mismatches().iter()
                .map(|mismatch| mismatch.to_json() )
                .collect::<Vec<serde_json::Value>>();
            let json = json!(mismatches);
            let s = CString::new(json.to_string()).unwrap();
            let p = s.as_ptr();
            mock_server.resources.push(s);
            p
        });
        match result {
            Some(p) => p as *mut _,
            None => 0 as *mut _
        }
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            error!("Caught a general panic: {:?}", cause);
            0 as *mut _
        }
    }
}

/// External interface to cleanup a mock server. This function will try terminate the mock server
/// with the given port number and cleanup any memory allocated for it. Returns true, unless a
/// mock server with the given port number does not exist, or the function panics.
///
/// **NOTE:** Although `close()` on the listerner for the mock server is called, this does not
/// currently work and the listerner will continue handling requests. In this
/// case, it will always return a 404 once the mock server has been cleaned up.
#[no_mangle]
pub extern fn cleanup_mock_server(mock_server_port: int32_t) -> bool {
    let result = catch_unwind(|| {
        shutdown_mock_server_by_port(mock_server_port)
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            error!("Caught a general panic: {:?}", cause);
            false
        }
    }
}

/// External interface to trigger a mock server to write out its pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If a NULL pointer is passed, the current working directory is used.
///
/// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
/// not be written, or there is no mock server running on that port or the function panics.
///
/// # Errors
///
/// Errors are returned as positive values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | A general panic was caught |
/// | 2 | The pact file was not able to be written |
/// | 3 | A mock server with the provided port was not found |
#[no_mangle]
pub extern fn write_pact_file(mock_server_port: int32_t, directory: *const c_char) -> int32_t {
    let result = catch_unwind(|| {
        let dir = unsafe {
            if directory.is_null() {
                warn!("Directory to write to is NULL, defaulting to the current working directory");
                None
            } else {
                let c_str = CStr::from_ptr(directory);
                let dir_str = str::from_utf8(c_str.to_bytes()).unwrap();
                if dir_str.is_empty() {
                    None
                } else {
                    Some(s!(dir_str))
                }
            }
        };

        lookup_mock_server_by_port(mock_server_port, &|mock_server| {
            match mock_server.write_pact(&dir) {
                Ok(_) => 0,
                Err(err) => {
                    error!("Failed to write pact to file - {}", err);
                    2
                }
            }
        }).unwrap_or(3)
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            error!("Caught a general panic: {:?}", cause);
            1
        }
    }
}

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
mod tests;
