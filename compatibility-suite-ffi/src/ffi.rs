//! Safe wrappers over the pact_ffi C ABI. Every wrapper records the call, its arguments and its
//! result in the step trace, so the trace reads as the exact call sequence a client language makes.

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::OnceLock;

use crate::trace;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]
mod sys {
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use sys::{InteractionHandle, InteractionPart, PactHandle, PactSpecification};
pub use sys::{InteractionPart_Request, InteractionPart_Response};
pub use sys::{PactSpecification_V1, PactSpecification_V1_1, PactSpecification_V2, PactSpecification_V3, PactSpecification_V4};

static LIB: OnceLock<sys::PactFfi> = OnceLock::new();

/// Loads libpact_ffi from `path`. Must be called once before any other function here.
pub fn load(path: &str) -> anyhow::Result<()> {
  let lib = unsafe { sys::PactFfi::new(path)? };
  LIB.set(lib).map_err(|_| anyhow::anyhow!("pact_ffi already loaded"))
}

fn lib() -> &'static sys::PactFfi {
  LIB.get().expect("pact_ffi not loaded")
}

fn c(s: &str) -> CString {
  CString::new(s).expect("string contains a NUL byte")
}

/// Renders a string argument as a C literal, eliding long bodies.
fn lit(s: &str) -> String {
  if s.chars().count() > 120 {
    let head: String = s.chars().take(100).collect();
    format!("{head:?}/* …{} bytes */", s.len())
  } else {
    format!("{s:?}")
  }
}

fn part(p: InteractionPart) -> &'static str {
  if p == InteractionPart_Request { "InteractionPart_Request" } else { "InteractionPart_Response" }
}

#[allow(non_upper_case_globals)]
pub fn spec_name(s: PactSpecification) -> &'static str {
  match s {
    PactSpecification_V1 => "PactSpecification_V1",
    PactSpecification_V1_1 => "PactSpecification_V1_1",
    PactSpecification_V2 => "PactSpecification_V2",
    PactSpecification_V3 => "PactSpecification_V3",
    PactSpecification_V4 => "PactSpecification_V4",
    _ => "PactSpecification_Unknown",
  }
}

pub fn new_pact(consumer: &str, provider: &str) -> PactHandle {
  let r = unsafe { lib().pactffi_new_pact(c(consumer).as_ptr(), c(provider).as_ptr()) };
  trace::call(format!("pactffi_new_pact({}, {})", lit(consumer), lit(provider)), r);
  r
}

pub fn with_specification(pact: PactHandle, spec: PactSpecification) -> bool {
  let r = unsafe { lib().pactffi_with_specification(pact, spec) };
  trace::call(format!("pactffi_with_specification({pact}, {})", spec_name(spec)), r);
  r
}

pub fn new_interaction(pact: PactHandle, description: &str) -> InteractionHandle {
  let r = unsafe { lib().pactffi_new_interaction(pact, c(description).as_ptr()) };
  trace::call(format!("pactffi_new_interaction({pact}, {})", lit(description)), r);
  r
}

pub fn with_request(i: InteractionHandle, method: &str, path: &str) -> bool {
  let r = unsafe { lib().pactffi_with_request(i, c(method).as_ptr(), c(path).as_ptr()) };
  trace::call(format!("pactffi_with_request({i}, {}, {})", lit(method), lit(path)), r);
  r
}

pub fn with_query_parameter_v2(i: InteractionHandle, name: &str, index: usize, value: &str) -> bool {
  let r = unsafe { lib().pactffi_with_query_parameter_v2(i, c(name).as_ptr(), index, c(value).as_ptr()) };
  trace::call(format!("pactffi_with_query_parameter_v2({i}, {}, {index}, {})", lit(name), lit(value)), r);
  r
}

pub fn with_header_v2(i: InteractionHandle, p: InteractionPart, name: &str, index: usize, value: &str) -> bool {
  let r = unsafe { lib().pactffi_with_header_v2(i, p, c(name).as_ptr(), index, c(value).as_ptr()) };
  trace::call(format!("pactffi_with_header_v2({i}, {}, {}, {index}, {})", part(p), lit(name), lit(value)), r);
  r
}

/// `rules` is a JSON object of matching rules, V2 style (`{"$.body.a": {"match": "type"}}`) or V3+ by category.
pub fn with_matching_rules(i: InteractionHandle, p: InteractionPart, rules: &str) -> bool {
  let r = unsafe { lib().pactffi_with_matching_rules(i, p, c(rules).as_ptr()) };
  trace::call(format!("pactffi_with_matching_rules({i}, {}, {})", part(p), lit(rules)), r);
  r
}

pub fn response_status(i: InteractionHandle, status: u16) -> bool {
  let r = unsafe { lib().pactffi_response_status(i, status) };
  trace::call(format!("pactffi_response_status({i}, {status})"), r);
  r
}

pub fn with_body(i: InteractionHandle, p: InteractionPart, content_type: &str, body: &str) -> bool {
  let r = unsafe { lib().pactffi_with_body(i, p, c(content_type).as_ptr(), c(body).as_ptr()) };
  trace::call(format!("pactffi_with_body({i}, {}, {}, {})", part(p), lit(content_type), lit(body)), r);
  r
}

pub fn with_binary_body(i: InteractionHandle, p: InteractionPart, content_type: &str, body: &[u8]) -> bool {
  let r = unsafe { lib().pactffi_with_binary_body(i, p, c(content_type).as_ptr(), body.as_ptr(), body.len()) };
  trace::call(format!("pactffi_with_binary_body({i}, {}, {}, /* {} bytes */, {})", part(p), lit(content_type),
    body.len(), body.len()), r);
  r
}

pub fn create_mock_server_for_transport(pact: PactHandle, addr: &str, port: u16, transport: &str) -> i32 {
  let r = unsafe {
    lib().pactffi_create_mock_server_for_transport(pact, c(addr).as_ptr(), port, c(transport).as_ptr(), ptr::null())
  };
  trace::call(format!("pactffi_create_mock_server_for_transport({pact}, {}, {port}, {}, NULL)", lit(addr), lit(transport)), r);
  r
}

pub fn mock_server_matched(port: i32) -> bool {
  let r = unsafe { lib().pactffi_mock_server_matched(port) };
  trace::call(format!("pactffi_mock_server_matched({port})"), r);
  r
}

/// The returned string is owned by the mock server and freed by `pactffi_cleanup_mock_server`,
/// so it is copied here and must not be passed to `pactffi_string_delete`.
pub fn mock_server_mismatches(port: i32) -> Option<String> {
  let p: *mut c_char = unsafe { lib().pactffi_mock_server_mismatches(port) };
  let r = (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned());
  trace::call(format!("pactffi_mock_server_mismatches({port})"), r.as_deref().map(lit).unwrap_or("NULL".into()));
  r
}

pub fn write_pact_file(port: i32, dir: &str, overwrite: bool) -> i32 {
  let r = unsafe { lib().pactffi_write_pact_file(port, c(dir).as_ptr(), overwrite) };
  trace::call(format!("pactffi_write_pact_file({port}, {}, {overwrite})", lit(dir)), r);
  r
}

pub fn cleanup_mock_server(port: i32) -> bool {
  let r = unsafe { lib().pactffi_cleanup_mock_server(port) };
  trace::call(format!("pactffi_cleanup_mock_server({port})"), r);
  r
}

pub fn free_pact_handle(pact: PactHandle) -> u32 {
  let r = unsafe { lib().pactffi_free_pact_handle(pact) };
  trace::call(format!("pactffi_free_pact_handle({pact})"), r);
  r
}

pub fn given(i: InteractionHandle, state: &str) -> bool {
  let r = unsafe { lib().pactffi_given(i, c(state).as_ptr()) };
  trace::call(format!("pactffi_given({i}, {})", lit(state)), r);
  r
}

/// `params` is a JSON object of the state's parameters.
pub fn given_with_params(i: InteractionHandle, state: &str, params: &str) -> i32 {
  let r = unsafe { lib().pactffi_given_with_params(i, c(state).as_ptr(), c(params).as_ptr()) };
  trace::call(format!("pactffi_given_with_params({i}, {}, {})", lit(state), lit(params)), r);
  r
}

/// Writes the pact for a handle that never had a mock server, as a provider test fixture would.
pub fn pact_handle_write_file(pact: PactHandle, dir: &str, overwrite: bool) -> i32 {
  let r = unsafe { lib().pactffi_pact_handle_write_file(pact, c(dir).as_ptr(), overwrite) };
  trace::call(format!("pactffi_pact_handle_write_file({pact}, {}, {overwrite})", lit(dir)), r);
  r
}

/// An opaque `VerifierHandle *`. Not `Send` in C terms, but the verifier owns no thread-affine state,
/// so the pointer is carried across to the blocking thread that runs the verification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Verifier(usize);

impl Verifier {
  fn h(self) -> *mut sys::VerifierHandle {
    self.0 as *mut sys::VerifierHandle
  }
}

impl std::fmt::Display for Verifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "handle")
  }
}

fn opt(s: Option<&str>) -> (Option<CString>, String) {
  match s {
    Some(s) => (Some(c(s)), lit(s)),
    None => (None, "NULL".into()),
  }
}

fn opt_ptr(s: &Option<CString>) -> *const c_char {
  s.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null())
}

/// Copies and frees a string the verifier handed over; these are caller-owned (`pactffi_string_delete`).
fn take_string(p: *const c_char) -> Option<String> {
  if p.is_null() {
    return None;
  }
  let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
  unsafe { lib().pactffi_string_delete(p as *mut c_char) };
  Some(s)
}

pub fn verifier_new_for_application(name: &str, version: &str) -> Verifier {
  let h = unsafe { lib().pactffi_verifier_new_for_application(c(name).as_ptr(), c(version).as_ptr()) };
  trace::call(format!("handle = pactffi_verifier_new_for_application({}, {})", lit(name), lit(version)),
    if h.is_null() { "NULL" } else { "VerifierHandle *" });
  Verifier(h as usize)
}

pub fn verifier_set_provider_info(v: Verifier, name: &str, scheme: &str, host: &str, port: u16, path: &str) {
  unsafe {
    lib().pactffi_verifier_set_provider_info(v.h(), c(name).as_ptr(), c(scheme).as_ptr(), c(host).as_ptr(), port,
      c(path).as_ptr())
  };
  trace::call(format!("pactffi_verifier_set_provider_info({v}, {}, {}, {}, {port}, {})", lit(name), lit(scheme),
    lit(host), lit(path)), "void");
}

pub fn verifier_set_provider_state(v: Verifier, url: &str, teardown: bool, body: bool) {
  unsafe { lib().pactffi_verifier_set_provider_state(v.h(), c(url).as_ptr(), teardown as u8, body as u8) };
  trace::call(format!("pactffi_verifier_set_provider_state({v}, {}, {}, {})", lit(url), teardown as u8, body as u8),
    "void");
}

pub fn verifier_set_publish_options(v: Verifier, provider_version: &str) -> i32 {
  let r = unsafe {
    lib().pactffi_verifier_set_publish_options(v.h(), c(provider_version).as_ptr(), ptr::null(), ptr::null(), 0,
      ptr::null())
  };
  trace::call(format!("pactffi_verifier_set_publish_options({v}, {}, NULL, NULL, 0, NULL)", lit(provider_version)), r);
  r
}

pub fn verifier_add_custom_header(v: Verifier, name: &str, value: &str) {
  unsafe { lib().pactffi_verifier_add_custom_header(v.h(), c(name).as_ptr(), c(value).as_ptr()) };
  trace::call(format!("pactffi_verifier_add_custom_header({v}, {}, {})", lit(name), lit(value)), "void");
}

pub fn verifier_add_file_source(v: Verifier, file: &str) {
  unsafe { lib().pactffi_verifier_add_file_source(v.h(), c(file).as_ptr()) };
  trace::call(format!("pactffi_verifier_add_file_source({v}, {})", lit(file)), "void");
}

/// A broker source with dynamic configuration (`pacts for verification`), no auth, selectors, tags or branch.
pub fn verifier_broker_source_with_selectors(v: Verifier, url: &str, enable_pending: bool, branch: Option<&str>)
  -> i32 {
  let (branch_c, branch_lit) = opt(branch);
  let r = unsafe {
    lib().pactffi_verifier_broker_source_with_selectors(v.h(), c(url).as_ptr(), ptr::null(), ptr::null(),
      ptr::null(), enable_pending as u8, ptr::null(), ptr::null(), 0, opt_ptr(&branch_c), ptr::null(), 0,
      ptr::null(), 0)
  };
  trace::call(format!("pactffi_verifier_broker_source_with_selectors({v}, {}, NULL, NULL, NULL, {}, NULL, NULL, 0, \
    {branch_lit}, NULL, 0, NULL, 0)", lit(url), enable_pending as u8), r);
  r
}

/// Blocks until the verification has run against every source. 0 is success, 1 a failed verification.
pub fn verifier_execute(v: Verifier) -> i32 {
  let r = unsafe { lib().pactffi_verifier_execute(v.h()) };
  trace::call(format!("pactffi_verifier_execute({v})"), r);
  r
}

pub fn verifier_json(v: Verifier) -> Option<String> {
  let r = take_string(unsafe { lib().pactffi_verifier_json(v.h()) });
  trace::call(format!("json = pactffi_verifier_json({v})"), r.as_deref().map(lit).unwrap_or("NULL".into()));
  if r.is_some() {
    trace::call("pactffi_string_delete(json)".into(), "void");
  }
  r
}

pub fn verifier_output(v: Verifier, strip_ansi: bool) -> Option<String> {
  let r = take_string(unsafe { lib().pactffi_verifier_output(v.h(), strip_ansi as u8) });
  trace::call(format!("output = pactffi_verifier_output({v}, {})", strip_ansi as u8),
    r.as_deref().map(lit).unwrap_or("NULL".into()));
  if r.is_some() {
    trace::call("pactffi_string_delete(output)".into(), "void");
  }
  r
}

pub fn verifier_shutdown(v: Verifier) {
  unsafe { lib().pactffi_verifier_shutdown(v.h()) };
  trace::call(format!("pactffi_verifier_shutdown({v})"), "void");
}

// ---- V3/V4 interaction setters ----

/// `generators` is a JSON object of generators by category (`{"body": {"$.a": {"type": "RandomInt"}}}`).
pub fn with_generators(i: InteractionHandle, p: InteractionPart, generators: &str) -> bool {
  let r = unsafe { lib().pactffi_with_generators(i, p, c(generators).as_ptr()) };
  trace::call(format!("pactffi_with_generators({i}, {}, {})", part(p), lit(generators)), r);
  r
}

pub fn given_with_param(i: InteractionHandle, state: &str, name: &str, value: &str) -> bool {
  let r = unsafe { lib().pactffi_given_with_param(i, c(state).as_ptr(), c(name).as_ptr(), c(value).as_ptr()) };
  trace::call(format!("pactffi_given_with_param({i}, {}, {}, {})", lit(state), lit(name), lit(value)), r);
  r
}

/// `value` is parsed as JSON where it can be, else kept as a string.
pub fn with_metadata(i: InteractionHandle, key: &str, value: &str, p: InteractionPart) -> bool {
  let r = unsafe { lib().pactffi_with_metadata(i, c(key).as_ptr(), c(value).as_ptr(), p) };
  trace::call(format!("pactffi_with_metadata({i}, {}, {}, {})", lit(key), lit(value), part(p)), r);
  r
}

pub fn set_pending(i: InteractionHandle, pending: bool) -> bool {
  let r = unsafe { lib().pactffi_set_pending(i, pending) };
  trace::call(format!("pactffi_set_pending({i}, {pending})"), r);
  r
}

pub fn set_key(i: InteractionHandle, key: &str) -> bool {
  let r = unsafe { lib().pactffi_set_key(i, c(key).as_ptr()) };
  trace::call(format!("pactffi_set_key({i}, {})", lit(key)), r);
  r
}

pub fn set_comment(i: InteractionHandle, key: &str, value: &str) -> bool {
  let r = unsafe { lib().pactffi_set_comment(i, c(key).as_ptr(), c(value).as_ptr()) };
  trace::call(format!("pactffi_set_comment({i}, {}, {})", lit(key), lit(value)), r);
  r
}

pub fn add_text_comment(i: InteractionHandle, comment: &str) -> bool {
  let r = unsafe { lib().pactffi_add_text_comment(i, c(comment).as_ptr()) };
  trace::call(format!("pactffi_add_text_comment({i}, {})", lit(comment)), r);
  r
}

pub fn interaction_test_name(i: InteractionHandle, name: &str) -> u32 {
  let r = unsafe { lib().pactffi_interaction_test_name(i, c(name).as_ptr()) };
  trace::call(format!("pactffi_interaction_test_name({i}, {})", lit(name)), r);
  r
}

pub fn new_message_interaction(pact: PactHandle, description: &str) -> InteractionHandle {
  let r = unsafe { lib().pactffi_new_message_interaction(pact, c(description).as_ptr()) };
  trace::call(format!("pactffi_new_message_interaction({pact}, {})", lit(description)), r);
  r
}

pub fn new_sync_message_interaction(pact: PactHandle, description: &str) -> InteractionHandle {
  let r = unsafe { lib().pactffi_new_sync_message_interaction(pact, c(description).as_ptr()) };
  trace::call(format!("pactffi_new_sync_message_interaction({pact}, {})", lit(description)), r);
  r
}

pub fn verifier_add_provider_transport(v: Verifier, protocol: &str, port: u16, path: &str, scheme: &str) {
  unsafe {
    lib().pactffi_verifier_add_provider_transport(v.h(), c(protocol).as_ptr(), port, c(path).as_ptr(),
      c(scheme).as_ptr())
  };
  trace::call(format!("pactffi_verifier_add_provider_transport({v}, {}, {port}, {}, {})", lit(protocol), lit(path),
    lit(scheme)), "void");
}

// ---- Reading messages back ----

/// One message's contents with generators applied: the body bytes and the metadata, values as JSON text.
#[derive(Debug, Clone, Default)]
pub struct Contents {
  pub body: Option<Vec<u8>>,
  pub metadata: Vec<(String, String)>,
}

impl Contents {
  pub fn body_str(&self) -> Option<String> {
    self.body.as_ref().map(|b| String::from_utf8_lossy(b).into_owned())
  }
}

/// Reads a `MessageContents *` the caller owns, then frees it with `pactffi_message_contents_delete`.
fn read_contents(contents: *const sys::MessageContents, name: &str) -> Contents {
  let lib = lib();
  let mut out = Contents::default();
  if contents.is_null() {
    return out;
  }
  let len = unsafe { lib.pactffi_message_contents_get_contents_length(contents) };
  trace::call(format!("pactffi_message_contents_get_contents_length({name})"), len);
  if len > 0 {
    let p = unsafe { lib.pactffi_message_contents_get_contents_bin(contents) };
    trace::call(format!("pactffi_message_contents_get_contents_bin({name})"), format!("/* {len} bytes */"));
    if !p.is_null() {
      out.body = Some(unsafe { std::slice::from_raw_parts(p, len) }.to_vec());
    }
  }
  let iter = unsafe { lib.pactffi_message_contents_get_metadata_iter(contents) };
  trace::call(format!("meta = pactffi_message_contents_get_metadata_iter({name})"), if iter.is_null() { "NULL" } else { "iter" });
  if !iter.is_null() {
    loop {
      let pair = unsafe { lib.pactffi_message_metadata_iter_next(iter) };
      if pair.is_null() {
        trace::call("pactffi_message_metadata_iter_next(meta)".into(), "NULL");
        break;
      }
      let (k, v) = unsafe {
        (CStr::from_ptr((*pair).key).to_string_lossy().into_owned(), CStr::from_ptr((*pair).value).to_string_lossy().into_owned())
      };
      trace::call("pactffi_message_metadata_iter_next(meta)".into(), format!("{{{}, {}}}", lit(&k), lit(&v)));
      unsafe { lib.pactffi_message_metadata_pair_delete(pair) };
      out.metadata.push((k, v));
    }
    unsafe { lib.pactffi_message_metadata_iter_delete(iter) };
    trace::call("pactffi_message_metadata_iter_delete(meta)".into(), "void");
  }
  unsafe { lib.pactffi_message_contents_delete(contents as *mut _) };
  trace::call(format!("pactffi_message_contents_delete({name})"), "void");
  out
}

/// Every async message in the pact, generated as a consumer test would see it.
pub fn async_messages(pact: PactHandle) -> Vec<Contents> {
  let lib = lib();
  let iter = unsafe { lib.pactffi_pact_handle_get_async_message_iter(pact) };
  trace::call(format!("iter = pactffi_pact_handle_get_async_message_iter({pact})"), if iter.is_null() { "NULL" } else { "iter" });
  let mut out = vec![];
  if iter.is_null() {
    return out;
  }
  loop {
    let m = unsafe { lib.pactffi_pact_async_message_iter_next(iter) };
    trace::call("message = pactffi_pact_async_message_iter_next(iter)".into(), if m.is_null() { "NULL" } else { "message" });
    if m.is_null() {
      break;
    }
    let contents = unsafe { lib.pactffi_async_message_generate_contents(m) };
    trace::call("contents = pactffi_async_message_generate_contents(message)".into(), if contents.is_null() { "NULL" } else { "contents" });
    out.push(read_contents(contents, "contents"));
  }
  unsafe { lib.pactffi_pact_async_message_iter_delete(iter) };
  trace::call("pactffi_pact_async_message_iter_delete(iter)".into(), "void");
  out
}

/// Every sync message in the pact as (request, responses), generated.
pub fn sync_messages(pact: PactHandle) -> Vec<(Contents, Vec<Contents>)> {
  let lib = lib();
  let iter = unsafe { lib.pactffi_pact_handle_get_sync_message_iter(pact) };
  trace::call(format!("iter = pactffi_pact_handle_get_sync_message_iter({pact})"), if iter.is_null() { "NULL" } else { "iter" });
  let mut out = vec![];
  if iter.is_null() {
    return out;
  }
  loop {
    let m = unsafe { lib.pactffi_pact_sync_message_iter_next(iter) };
    trace::call("message = pactffi_pact_sync_message_iter_next(iter)".into(), if m.is_null() { "NULL" } else { "message" });
    if m.is_null() {
      break;
    }
    let req = unsafe { lib.pactffi_sync_message_generate_request_contents(m) };
    trace::call("request = pactffi_sync_message_generate_request_contents(message)".into(), if req.is_null() { "NULL" } else { "request" });
    let request = read_contents(req, "request");
    let n = unsafe { lib.pactffi_sync_message_get_number_responses(m) };
    trace::call("pactffi_sync_message_get_number_responses(message)".into(), n);
    let responses = (0..n).map(|idx| {
      let r = unsafe { lib.pactffi_sync_message_generate_response_contents(m, idx) };
      trace::call(format!("response = pactffi_sync_message_generate_response_contents(message, {idx})"), if r.is_null() { "NULL" } else { "response" });
      read_contents(r, "response")
    }).collect();
    out.push((request, responses));
  }
  unsafe { lib.pactffi_pact_sync_message_iter_delete(iter) };
  trace::call("pactffi_pact_sync_message_iter_delete(iter)".into(), "void");
  out
}

/// The first async message's body generators, each run with `context` (a JSON object), as (path, value).
/// There's no call that generates contents with a context, so a client walks the generators itself.
pub fn async_body_generators_with_context(pact: PactHandle, context: &str) -> Vec<(String, Option<String>)> {
  let lib = lib();
  let iter = unsafe { lib.pactffi_pact_handle_get_async_message_iter(pact) };
  trace::call(format!("iter = pactffi_pact_handle_get_async_message_iter({pact})"), if iter.is_null() { "NULL" } else { "iter" });
  let mut out = vec![];
  if iter.is_null() {
    return out;
  }
  let m = unsafe { lib.pactffi_pact_async_message_iter_next(iter) };
  trace::call("message = pactffi_pact_async_message_iter_next(iter)".into(), if m.is_null() { "NULL" } else { "message" });
  if !m.is_null() {
    let contents = unsafe { lib.pactffi_async_message_get_contents(m) };
    trace::call("contents = pactffi_async_message_get_contents(message)".into(), if contents.is_null() { "NULL" } else { "contents" });
    let gens = unsafe { lib.pactffi_message_contents_get_generators_iter(contents, sys::GeneratorCategory_BODY) };
    trace::call("gens = pactffi_message_contents_get_generators_iter(contents, GeneratorCategory_BODY)".into(),
      if gens.is_null() { "NULL" } else { "gens" });
    if !gens.is_null() {
      loop {
        let pair = unsafe { lib.pactffi_generators_iter_next(gens) };
        if pair.is_null() {
          trace::call("pactffi_generators_iter_next(gens)".into(), "NULL");
          break;
        }
        let path = unsafe { CStr::from_ptr((*pair).path) }.to_string_lossy().into_owned();
        trace::call("pair = pactffi_generators_iter_next(gens)".into(), format!("{{path: {}, generator}}", lit(&path)));
        let v = unsafe { lib.pactffi_generator_generate_string((*pair).generator, c(context).as_ptr()) };
        let value = take_string(v);
        trace::call(format!("value = pactffi_generator_generate_string(pair->generator, {})", lit(context)),
          value.as_deref().map(lit).unwrap_or("NULL".into()));
        if value.is_some() {
          trace::call("pactffi_string_delete(value)".into(), "void");
        }
        unsafe { lib.pactffi_generators_iter_pair_delete(pair) };
        trace::call("pactffi_generators_iter_pair_delete(pair)".into(), "void");
        out.push((path, value));
      }
      unsafe { lib.pactffi_generators_iter_delete(gens) };
      trace::call("pactffi_generators_iter_delete(gens)".into(), "void");
    }
  }
  unsafe { lib.pactffi_pact_async_message_iter_delete(iter) };
  trace::call("pactffi_pact_async_message_iter_delete(iter)".into(), "void");
  out
}
