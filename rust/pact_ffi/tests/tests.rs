use std::env;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::ptr::null;

use bytes::Bytes;
use expectest::prelude::*;
use itertools::Itertools;
use libc::c_char;
use log::LevelFilter;
use maplit::*;
use pact_ffi::log::pactffi_log_to_buffer;
use pact_models::bodies::OptionalBody;
use pact_models::PactSpecification;
use pretty_assertions::assert_eq;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use tempfile::TempDir;
use serde_json::{json, Value};
use rstest::rstest;
use regex::Regex;
#[allow(deprecated)]
use pact_ffi::mock_server::{
  pactffi_cleanup_mock_server,
  pactffi_create_mock_server,
  pactffi_create_mock_server_for_pact,
  pactffi_mock_server_mismatches,
  pactffi_write_pact_file,
  pactffi_mock_server_logs,
};
#[allow(deprecated)]
use pact_ffi::mock_server::handles::{
  InteractionPart,
  PactHandle,
  pact_default_file_name,
  pactffi_add_text_comment,
  pactffi_free_pact_handle,
  pactffi_given_with_params,
  pactffi_message_expects_to_receive,
  pactffi_message_given,
  pactffi_message_reify,
  pactffi_message_with_contents,
  pactffi_message_with_metadata,
  pactffi_message_with_metadata_v2,
  pactffi_new_interaction,
  pactffi_new_message,
  pactffi_new_message_pact,
  pactffi_new_pact,
  pactffi_pact_handle_write_file,
  pactffi_response_status,
  pactffi_set_comment,
  pactffi_set_key,
  pactffi_set_pending,
  pactffi_upon_receiving,
  pactffi_with_binary_file,
  pactffi_with_body,
  pactffi_with_header,
  pactffi_with_header_v2,
  pactffi_with_multipart_file,
  pactffi_with_multipart_file_v2,
  pactffi_with_query_parameter_v2,
  pactffi_with_request,
  pactffi_with_specification,
  pactffi_write_message_pact_file,
};
use pact_ffi::verifier::{
  OptionsFlags,
  pactffi_verifier_add_directory_source,
  pactffi_verifier_add_file_source,
  pactffi_verifier_cli_args,
  pactffi_verifier_execute,
  pactffi_verifier_new_for_application,
  pactffi_verifier_output,
  pactffi_verifier_set_provider_info,
  pactffi_verifier_shutdown
};

#[test]
fn post_to_mock_server_with_mismatches() {
  let pact_json = include_str!("post-pact.json");
  let pact_json_c = CString::new(pact_json).expect("Could not construct C string from json");
  let address = CString::new("127.0.0.1:0").unwrap();
  #[allow(deprecated)]
  let port = pactffi_create_mock_server(pact_json_c.as_ptr(), address.as_ptr(), false);
  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  client.post(format!("http://127.0.0.1:{}/path", port).as_str())
    .header(CONTENT_TYPE, "application/json")
    .body(r#"{"foo":"no-very-bar"}"#)
    .send().expect("Sent POST request to mock server");

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  pactffi_cleanup_mock_server(port);

  assert_eq!(
    "[{\"method\":\"POST\",\"mismatches\":[{\"actual\":\"\\\"no-very-bar\\\"\",\"expected\":\"\\\"bar\\\"\",\"mismatch\":\"Expected 'no-very-bar' (String) to be equal to 'bar' (String)\",\"path\":\"$.foo\",\"type\":\"BodyMismatch\"}],\"path\":\"/path\",\"type\":\"request-mismatch\"}]",
    mismatches
  );
}

#[test]
#[allow(deprecated)]
fn create_header_with_multiple_values() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_header_with_multiple_values").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());
  let name = CString::new("accept").unwrap();
  let value_1 = CString::new("application/hal+json").unwrap();
  let value_2 = CString::new("application/json").unwrap();
  pactffi_with_header(interaction.clone(), InteractionPart::Request, name.as_ptr(), 1, value_2.as_ptr());
  pactffi_with_header(interaction.clone(), InteractionPart::Request, name.as_ptr(), 0, value_1.as_ptr());
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    expect!(interaction.request.headers.as_ref()).to(be_some().value(&hashmap!{
      "accept".to_string() => vec!["application/hal+json".to_string(), "application/json".to_string()]
    }));
  });
}

#[test]
fn create_query_parameter_with_multiple_values() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_query_parameter_with_multiple_values").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());
  let name = CString::new("q").unwrap();
  let value_1 = CString::new("1").unwrap();
  let value_2 = CString::new("2").unwrap();
  let value_3 = CString::new("3").unwrap();
  pactffi_with_query_parameter_v2(interaction.clone(), name.as_ptr(), 2, value_3.as_ptr());
  pactffi_with_query_parameter_v2(interaction.clone(), name.as_ptr(), 0, value_1.as_ptr());
  pactffi_with_query_parameter_v2(interaction.clone(), name.as_ptr(), 1, value_2.as_ptr());
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    expect!(interaction.request.query.as_ref()).to(be_some().value(&hashmap!{
      "q".to_string() => vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string())]
    }));
  });
}

#[test]
fn create_multipart_file() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_multipart_file").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());
  let content_type = CString::new("application/json").unwrap();
  let content_type2 = CString::new("text/plain").unwrap();
  let file = CString::new("tests/multipart-test-file.json").unwrap();
  let file2 = CString::new("tests/note.text").unwrap();
  let part_name = CString::new("file").unwrap();
  let part_name2 = CString::new("note").unwrap();

  pactffi_with_multipart_file(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), file.as_ptr(), part_name.as_ptr());
  pactffi_with_multipart_file(interaction.clone(), InteractionPart::Request, content_type2.as_ptr(), file2.as_ptr(), part_name2.as_ptr());

  let (boundary, headers, body) = interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    let boundary = match &interaction.request.headers {
      Some(hashmap) => {
        hashmap.get("Content-Type")
          .map(|vec| vec[0].as_str())
          // Sorry for awful mime parsing..
          .map(|content_type: &str| content_type.split("boundary=").collect::<Vec<_>>())
          .map(|split| split[1])
          .unwrap_or("")
      },
      None => ""
    };

    let actual_req_body_str = match &interaction.request.body {
      OptionalBody::Present(body, _, _) => body.clone(),
      _ => Bytes::new(),
    };

    (boundary.to_string(), interaction.request.headers.clone(), actual_req_body_str)
  }).unwrap();

  expect!(headers).to(be_some().value(hashmap!{
    "Content-Type".to_string() => vec![format!("multipart/form-data; boundary={}", boundary)],
  }));

  let expected_req_body = Bytes::from(format!(
    "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"multipart-test-file.json\"\r\nContent-Type: application/json\r\n\r\ntrue\r\n\
     --{boundary}\r\nContent-Disposition: form-data; name=\"note\"; filename=\"note.text\"\r\nContent-Type: text/plain\r\n\r\nThis is a note. Truth.\r\n--{boundary}--\r\n",
    boundary = boundary
  ));
  assert_eq!(expected_req_body, body);
}

#[test]
fn create_multipart_file_v2() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_multipart_file").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());
  let content_type = CString::new("application/json").unwrap();
  let content_type2 = CString::new("text/plain").unwrap();
  let file = CString::new("tests/multipart-test-file.json").unwrap();
  let file2 = CString::new("tests/note.text").unwrap();
  let part_name = CString::new("file").unwrap();
  let part_name2 = CString::new("note").unwrap();
  let boundary = "test boundary";
  let boundary_cstring = CString::new(boundary).unwrap();

  pactffi_with_multipart_file_v2(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), file.as_ptr(), part_name.as_ptr(), boundary_cstring.as_ptr());
  pactffi_with_multipart_file_v2(interaction.clone(), InteractionPart::Request, content_type2.as_ptr(), file2.as_ptr(), part_name2.as_ptr(), boundary_cstring.as_ptr());

  let ( headers, body) = interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();

    let actual_req_body_str = match &interaction.request.body {
      OptionalBody::Present(body, _, _) => body.clone(),
      _ => Bytes::new(),
    };

    (interaction.request.headers.clone(), actual_req_body_str)
  }).unwrap();

  expect!(headers).to(be_some().value(hashmap!{
    "Content-Type".to_string() => vec![format!("multipart/form-data; boundary={}", boundary)],
  }));

  let expected_req_body = Bytes::from(format!(
    "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"multipart-test-file.json\"\r\nContent-Type: application/json\r\n\r\ntrue\r\n\
     --{boundary}\r\nContent-Disposition: form-data; name=\"note\"; filename=\"note.text\"\r\nContent-Type: text/plain\r\n\r\nThis is a note. Truth.\r\n--{boundary}--\r\n",
    boundary = boundary
  ));
  assert_eq!(expected_req_body, body);
}

#[test]
fn set_key() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("set_key").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());
  let key = CString::new("foobar").unwrap();

  assert!(pactffi_set_key(interaction, key.as_ptr()));

  interaction.with_interaction(&|_, _, i| {
    assert_eq!(
      i.as_v4_http().unwrap().key,
      Some("foobar".to_string())
    )
  });

  assert!(pactffi_set_key(interaction, null()));

  interaction.with_interaction(&|_, _, i| {
    assert_eq!(
      i.as_v4_http().unwrap().key,
      None
    )
  });
}

#[test]
fn set_pending() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("set_pending").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());

  assert!(pactffi_set_pending(interaction, true));

  interaction.with_interaction(&|_, _, i| {
    assert_eq!(
      i.as_v4_http().unwrap().pending,
      true,
    )
  });

  assert!(pactffi_set_pending(interaction, false));

  interaction.with_interaction(&|_, _, i| {
    assert_eq!(
      i.as_v4_http().unwrap().pending,
      false,
    )
  });
}

#[test]
fn set_comment() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("set_comment").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());

  let key_int = CString::new("key_int").unwrap();
  let value_int = CString::new("1234").unwrap();
  let key_str = CString::new("key_str").unwrap();
  let value_str = CString::new("some string").unwrap();
  let key_bool = CString::new("key_bool").unwrap();
  let value_bool = CString::new("true").unwrap();
  let key_float = CString::new("key_float").unwrap();
  let value_float = CString::new("12.34").unwrap();
  let key_array = CString::new("key_array").unwrap();
  let value_array = CString::new("[1, 2, 3]").unwrap();
  let key_obj = CString::new("key_object").unwrap();
  let value_obj = CString::new("{\"key\": \"value\"}").unwrap();

  assert!(pactffi_set_comment(interaction, key_int.as_ptr(), value_int.as_ptr()));
  assert!(pactffi_set_comment(interaction, key_str.as_ptr(), value_str.as_ptr()));
  assert!(pactffi_set_comment(interaction, key_bool.as_ptr(), value_bool.as_ptr()));
  assert!(pactffi_set_comment(interaction, key_float.as_ptr(), value_float.as_ptr()));
  assert!(pactffi_set_comment(interaction, key_array.as_ptr(), value_array.as_ptr()));
  assert!(pactffi_set_comment(interaction, key_obj.as_ptr(), value_obj.as_ptr()));

  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(interaction.comments["key_int"], json!(1234));
    assert_eq!(interaction.comments["key_str"], json!("some string"));
    assert_eq!(interaction.comments["key_bool"], json!(true));
    assert_eq!(interaction.comments["key_float"], json!(12.34));
    assert_eq!(interaction.comments["key_array"], json!([1, 2, 3]));
    assert_eq!(interaction.comments["key_object"], json!({"key": "value"}));
  });

  assert!(pactffi_set_comment(interaction, key_int.as_ptr(), null()));
  assert!(pactffi_set_comment(interaction, key_str.as_ptr(), null()));
  assert!(pactffi_set_comment(interaction, key_bool.as_ptr(), null()));
  assert!(pactffi_set_comment(interaction, key_float.as_ptr(), null()));
  assert!(pactffi_set_comment(interaction, key_array.as_ptr(), null()));
  assert!(pactffi_set_comment(interaction, key_obj.as_ptr(), null()));

  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(
      interaction.comments,
      hashmap!{}
    )
  });
}

#[test]
fn add_text_comment() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("set_comment").unwrap();
  let interaction = pactffi_new_interaction(pact_handle, description.as_ptr());

  let values = vec![
    CString::new("foo").unwrap(),
    CString::new("bar").unwrap(),
    CString::new("hello").unwrap(),
    CString::new("world").unwrap(),
  ];

  // Testing appending (regular use case)
  assert!(pactffi_add_text_comment(interaction, values[0].as_ptr()));
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(interaction.comments["text"], json!(["foo"]));
  });

  assert!(pactffi_add_text_comment(interaction, values[1].as_ptr()));
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(interaction.comments["text"], json!(["foo", "bar"]));
  });

  // Test appending to a non-array value
  let text_key = CString::new("text").unwrap();
  let int_value = CString::new("123").unwrap();
  pactffi_set_comment(interaction, text_key.as_ptr(), int_value.as_ptr());

  assert!(pactffi_add_text_comment(interaction, values[2].as_ptr()));
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(interaction.comments["text"], json!(["hello"]));
  });

  assert!(pactffi_add_text_comment(interaction, values[3].as_ptr()));
  interaction.with_interaction(&|_, _, i| {
    let interaction = i.as_v4_http().unwrap();
    assert_eq!(interaction.comments["text"], json!(["hello", "world"]));
  });
}

#[test_log::test]
#[allow(deprecated)]
fn http_consumer_feature_test() {
  let consumer_name = CString::new("http-consumer").unwrap();
  let provider_name = CString::new("http-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("request_with_matchers").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());
  let special_header = CString::new("My-Special-Content-Type").unwrap();
  let content_type = CString::new("Content-Type").unwrap();
  let authorization = CString::new("Authorization").unwrap();
  let path_matcher = CString::new("{\"value\":\"/request/1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"\\/request\\/[0-9]+\"}").unwrap();
  let value_header_with_matcher = CString::new("{\"value\":\"application/json\",\"pact:matcher:type\":\"regex\",\"regex\":\"\\\\w+\\/\\\\w+\"}").unwrap();
  let auth_header_with_matcher = CString::new("{\"value\":\"Bearer 1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"Bearer [0-9]+\"}").unwrap();
  let query_param_matcher = CString::new("{\"value\":\"bar\",\"pact:matcher:type\":\"regex\", \"regex\":\"(bar|baz|bat)\"}").unwrap();
  let request_body_with_matchers = CString::new("{\"id\": {\"value\":1,\"pact:matcher:type\":\"type\"}}").unwrap();
  let response_body_with_matchers = CString::new("{\"created\": {\"value\":\"maybe\",\"pact:matcher:type\":\"regex\", \"regex\":\"(yes|no|maybe)\"}}").unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let description = CString::new("a request to test the FFI interface").unwrap();
  let method = CString::new("POST").unwrap();
  let query =  CString::new("foo").unwrap();
  let header = CString::new("application/json").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method  .as_ptr(), path_matcher.as_ptr());
  pactffi_with_header(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
  pactffi_with_header(interaction.clone(), InteractionPart::Request, authorization.as_ptr(), 0, auth_header_with_matcher.as_ptr());
  pactffi_with_query_parameter_v2(interaction.clone(), query.as_ptr(), 0, query_param_matcher.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Request, header.as_ptr(), request_body_with_matchers.as_ptr());
  // will respond with...
  pactffi_with_header(interaction.clone(), InteractionPart::Response, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
  pactffi_with_header(interaction.clone(), InteractionPart::Response, special_header.as_ptr(), 0, value_header_with_matcher.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Response, header.as_ptr(), response_body_with_matchers.as_ptr());
  pactffi_response_status(interaction.clone(), 200);
  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  // Mock server has started, we can't now modify the pact
  expect!(pactffi_upon_receiving(interaction.clone(), description.as_ptr())).to(be_false());

  let client = Client::default();
  let result = client.post(format!("http://127.0.0.1:{}/request/9999?foo=baz", port).as_str())
    .header("Content-Type", "application/json")
    .header("Authorization", "Bearer 9999")
    .body(r#"{"id": 7}"#)
    .send();

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
      expect!(res.headers().get("My-Special-Content-Type").unwrap()).to(be_eq("application/json"));
      let json: serde_json::Value = res.json().unwrap_or_default();
      expect!(json.get("created").unwrap().as_str().unwrap()).to(be_eq("maybe"));
    },
    Err(_) => {
      panic!("expected 200 response but request failed");
    }
  };

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  pactffi_write_pact_file(port, file_path.as_ptr(), true);
  pactffi_cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[]"));
}

#[test]
#[allow(deprecated)]
fn http_xml_consumer_feature_test() {
  let consumer_name = CString::new("http-consumer").unwrap();
  let provider_name = CString::new("http-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("request_with_matchers").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());
  let accept = CString::new("Accept").unwrap();
  let content_type = CString::new("Content-Type").unwrap();
  let response_body_with_matchers = CString::new(r#"{"version":"1.0","charset":"UTF-8","root":{"name":"ns1:projects","children":[{"pact:matcher:type":"type","value":{"name":"ns1:project","children":[{"name":"ns1:tasks","children":[{"pact:matcher:type":"type","value":{"name":"ns1:task","children":[],"attributes":{"id":{"pact:matcher:type":"integer","value":1},"name":{"pact:matcher:type":"type","value":"Task 1"},"done":{"pact:matcher:type":"type","value":true}}},"examples":5}],"attributes":{}}],"attributes":{"id":{"pact:matcher:type":"integer","value":1},"type":"activity","name":{"pact:matcher:type":"type","value":"Project 1"}}},"examples":2}],"attributes":{"id":"1234","xmlns:ns1":"http://some.namespace/and/more/stuff"}}}"#).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let description = CString::new("a request to test the FFI interface").unwrap();
  let method = CString::new("GET").unwrap();
  let path = CString::new("/xml").unwrap();
  let header = CString::new("application/xml").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_header(interaction.clone(), InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  // will respond with...
  pactffi_with_header(interaction.clone(), InteractionPart::Response, content_type.as_ptr(), 0, header.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Response, header.as_ptr(), response_body_with_matchers.as_ptr());
  pactffi_response_status(interaction.clone(), 200);
  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  // Mock server has started, we can't now modify the pact
  expect!(pactffi_upon_receiving(interaction.clone(), description.as_ptr())).to(be_false());

  let client = Client::default();
  let result = client.get(format!("http://127.0.0.1:{}/xml", port).as_str())
    .header("Accept", "application/xml")
    .send();

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
      expect!(res.headers().get("Content-Type").unwrap()).to(be_eq("application/xml"));
      expect!(res.text().unwrap_or_default()).to(be_equal_to("<?xml version='1.0'?><ns1:projects id='1234' xmlns:ns1='http://some.namespace/and/more/stuff'><ns1:project id='1' name='Project 1' type='activity'><ns1:tasks><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/></ns1:tasks></ns1:project><ns1:project id='1' name='Project 1' type='activity'><ns1:tasks><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/></ns1:tasks></ns1:project></ns1:projects>"));
    },
    Err(_) => {
      panic!("expected 200 response but request failed");
    }
  };

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  pactffi_write_pact_file(port, file_path.as_ptr(), true);
  pactffi_cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[]"));
}

#[test]
fn message_consumer_feature_test() {
  let consumer_name = CString::new("message-consumer").unwrap();
  let provider_name = CString::new("message-provider").unwrap();
  let description = CString::new("message_request_with_matchers").unwrap();
  let content_type = CString::new("application/json").unwrap();
  let metadata_key = CString::new("message-queue-name").unwrap();
  let metadata_val = CString::new("message-queue-val").unwrap();
  let request_body_with_matchers = CString::new("{\"id\": {\"value\":1,\"pact:matcher:type\":\"type\"}}").unwrap();
  let given = CString::new("a functioning FFI interface").unwrap();
  let receive_description = CString::new("a request to test the FFI interface").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  let message_pact_handle = pactffi_new_message_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let message_handle = pactffi_new_message(message_pact_handle.clone(), description.as_ptr());
  pactffi_message_given(message_handle.clone(), given.as_ptr());
  pactffi_message_expects_to_receive(message_handle.clone(), receive_description.as_ptr());
  let body_bytes = request_body_with_matchers.as_bytes();
  pactffi_message_with_contents(message_handle.clone(), content_type.as_ptr(), body_bytes.as_ptr(), body_bytes.len());
  pactffi_message_with_metadata(message_handle.clone(), metadata_key.as_ptr(), metadata_val.as_ptr());
  let res: *const c_char = pactffi_message_reify(message_handle.clone());
  let reified: &CStr = unsafe { CStr::from_ptr(res) };
  expect!(reified.to_str().to_owned()).to(be_ok().value("{\"contents\":{\"id\":1},\"description\":\"a request to test the FFI interface\",\"matchingRules\":{\"body\":{\"$.id\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]}}},\"metadata\":{\"contentType\":\"application/json\",\"message-queue-name\":\"message-queue-val\"},\"providerStates\":[{\"name\":\"a functioning FFI interface\"}]}".to_string()));
  let res = pactffi_write_message_pact_file(message_pact_handle.clone(), file_path.as_ptr(), true);
  expect!(res).to(be_eq(0));
}

#[test]
fn message_xml_consumer_feature_test() {
  let consumer_name = CString::new("message-consumer").unwrap();
  let provider_name = CString::new("message-provider").unwrap();
  let description = CString::new("message_request_with_matchers").unwrap();
  let content_type = CString::new("application/xml").unwrap();
  let metadata_key = CString::new("message-queue-name").unwrap();
  let metadata_val = CString::new("message-queue-val").unwrap();
  let request_body_with_matchers = CString::new(r#"{"version":"1.0","charset":"UTF-8","root":{"name":"ns1:projects","children":[{"pact:matcher:type":"type","value":{"name":"ns1:project","children":[{"name":"ns1:tasks","children":[{"pact:matcher:type":"type","value":{"name":"ns1:task","children":[],"attributes":{"id":{"pact:matcher:type":"integer","value":1},"name":{"pact:matcher:type":"type","value":"Task 1"},"done":{"pact:matcher:type":"type","value":true}}},"examples":5}],"attributes":{}}],"attributes":{"id":{"pact:matcher:type":"integer","value":1},"type":"activity","name":{"pact:matcher:type":"type","value":"Project 1"}}},"examples":2}],"attributes":{"id":"1234","xmlns:ns1":"http://some.namespace/and/more/stuff"}}}"#).unwrap();
  let given = CString::new("a functioning FFI interface").unwrap();
  let receive_description = CString::new("a request to test the FFI interface").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  let message_pact_handle = pactffi_new_message_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let message_handle = pactffi_new_message(message_pact_handle.clone(), description.as_ptr());
  pactffi_message_given(message_handle.clone(), given.as_ptr());
  pactffi_message_expects_to_receive(message_handle.clone(), receive_description.as_ptr());
  let body_bytes = request_body_with_matchers.as_bytes();
  pactffi_message_with_contents(message_handle.clone(), content_type.as_ptr(), body_bytes.as_ptr(), body_bytes.len());
  pactffi_message_with_metadata(message_handle.clone(), metadata_key.as_ptr(), metadata_val.as_ptr());
  let res: *const c_char = pactffi_message_reify(message_handle.clone());
  let reified: &CStr = unsafe { CStr::from_ptr(res) };
  expect!(reified.to_str().to_owned()).to(be_ok().value("{\"contents\":\"<?xml version='1.0'?><ns1:projects id='1234' xmlns:ns1='http://some.namespace/and/more/stuff'><ns1:project id='1' name='Project 1' type='activity'><ns1:tasks><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/></ns1:tasks></ns1:project><ns1:project id='1' name='Project 1' type='activity'><ns1:tasks><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/><ns1:task done='true' id='1' name='Task 1'/></ns1:tasks></ns1:project></ns1:projects>\",\"description\":\"a request to test the FFI interface\",\"matchingRules\":{\"body\":{\"$.ns1:projects.ns1:project\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]},\"$.ns1:projects.ns1:project.ns1:tasks.ns1:task\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]},\"$.ns1:projects.ns1:project.ns1:tasks.ns1:task['@done']\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]},\"$.ns1:projects.ns1:project.ns1:tasks.ns1:task['@id']\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"integer\"}]},\"$.ns1:projects.ns1:project.ns1:tasks.ns1:task['@name']\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]},\"$.ns1:projects.ns1:project['@id']\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"integer\"}]},\"$.ns1:projects.ns1:project['@name']\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]}}},\"metadata\":{\"contentType\":\"application/xml\",\"message-queue-name\":\"message-queue-val\"},\"providerStates\":[{\"name\":\"a functioning FFI interface\"}]}".to_string()));
  let res = pactffi_write_message_pact_file(message_pact_handle.clone(), file_path.as_ptr(), true);
  expect!(res).to(be_eq(0));
}

#[test]
fn message_consumer_with_matchers_and_generators_test() {
  let consumer_name = CString::new("message-consumer").unwrap();
  let provider_name = CString::new("message-provider").unwrap();
  let description = CString::new("message_request_with_matchers_and_generators").unwrap();
  let content_type = CString::new("application/json").unwrap();
  let metadata_key = CString::new("message-queue-name").unwrap();
  let metadata_val = CString::new("{\"pact:generator:type\":\"RandomString\",\"value\":\"some text\",\"pact:matcher:type\":\"type\"}").unwrap();
  let request_body_with_matchers = CString::new("{\"id\": {\"pact:generator:type\":\"RandomInt\",\"min\":1,\"pact:matcher:type\":\"integer\"}}").unwrap();
  let given = CString::new("a functioning FFI interface").unwrap();
  let receive_description = CString::new("a request to test the FFI interface").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  let message_pact_handle = pactffi_new_message_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let message_handle = pactffi_new_message(message_pact_handle.clone(), description.as_ptr());
  pactffi_message_given(message_handle.clone(), given.as_ptr());
  pactffi_message_expects_to_receive(message_handle.clone(), receive_description.as_ptr());
  let body_bytes = request_body_with_matchers.as_bytes();
  pactffi_message_with_contents(message_handle.clone(), content_type.as_ptr(), body_bytes.as_ptr(), body_bytes.len());
  pactffi_message_with_metadata_v2(message_handle.clone(), metadata_key.as_ptr(), metadata_val.as_ptr());
  let res: *const c_char = pactffi_message_reify(message_handle.clone());
  let reified = unsafe { CStr::from_ptr(res) }.to_str().unwrap();
  let message = serde_json::from_str(reified).unwrap_or(json!({}));
  expect!(Regex::new("\\d+").unwrap().is_match(message.get("contents").unwrap().get("id").unwrap().to_string().as_str())).to(be_true());
  expect!(Regex::new("[\\d\\w]+").unwrap().is_match(message.get("metadata").unwrap().get("message-queue-name").unwrap().to_string().as_str())).to(be_true());
  let res = pactffi_write_message_pact_file(message_pact_handle.clone(), file_path.as_ptr(), true);
  expect!(res).to(be_eq(0));
}

#[test]
fn pactffi_verifier_cli_args_test() {
    let data = pactffi_verifier_cli_args();
    let c_str: &CStr = unsafe { CStr::from_ptr(data) };
    let str_slice: &str = c_str.to_str().unwrap();

    let options_flags: OptionsFlags = serde_json::from_str(str_slice).unwrap();

    assert!(options_flags.options.len() > 0);
    assert!(options_flags.flags.len() > 0);
}

/// Get the path to one of our sample *.json files.
fn fixture_path(path: &str) -> PathBuf {
  env::current_dir()
    .expect("could not find current working directory")
    .join("tests")
    .join(path)
    .to_owned()
}

#[rstest(
  specification,                                          expected_value,
  case::specification_unknown(PactSpecification::Unknown, false),
  case::specification_v1(PactSpecification::V1,           false),
  case::specification_v1_1(PactSpecification::V1_1,       false),
  case::specification_v2(PactSpecification::V2,           false),
  case::specification_v3(PactSpecification::V3,           true),
  case::specification_v4(PactSpecification::V4,           true),
)]
fn pactffi_with_binary_file_feature_test(specification: PactSpecification, expected_value: bool) {
  let consumer_name = CString::new("http-consumer").unwrap();
  let provider_name = CString::new("image-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  pactffi_with_specification(pact_handle, specification);

  let description = CString::new("request_with_matchers").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("image/gif").unwrap();
  let path = CString::new("/upload").unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let description = CString::new("a request to test the FFI interface").unwrap();
  let method = CString::new("POST").unwrap();

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();

  let mut buffer = Vec::new();
  let gif_file = fixture_path("1px.gif");
  File::open(gif_file).unwrap().read_to_end(&mut buffer).unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_binary_file(interaction.clone(), InteractionPart::Request, content_type.as_ptr(),
                           buffer.as_ptr(), buffer.len());
  // will respond with...
  pactffi_response_status(interaction.clone(), 201);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let result = client.post(format!("http://127.0.0.1:{}/upload", port).as_str())
    .header("Content-Type", "image/gif")
    .body(buffer)
    .send();

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  println!("pactffi_with_binary_file_feature_test v{}: {}", specification, mismatches);
  match result {
    Ok(res) => {
      let status = res.status();
      expect!(status).to(be_eq(201));
    },
    Err(err) => {
      panic!("expected 201 response but request failed - {}", err);
    }
  };

  pactffi_write_pact_file(port, file_path.as_ptr(), true);
  pactffi_cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[]"));

  let actual_value = interaction.with_interaction(
    &|_, _, inner| inner.as_v4_http().unwrap().request.matching_rules.add_category("body").is_not_empty()
  ).unwrap_or(false);
  expect!(actual_value).to(be_equal_to(expected_value));
}

#[test_log::test]
#[allow(deprecated)]
fn http_verification_from_directory_feature_test() {
  let name = CString::new("tests").unwrap();
  let version = CString::new("1.0.0").unwrap();
  let handle = pactffi_verifier_new_for_application(name.as_ptr(), version.as_ptr());

  let provider_name = CString::new("test_provider").unwrap();
  pactffi_verifier_set_provider_info(handle, provider_name.as_ptr(), null(), null(), 0, null());

  let pacts_path = fixture_path("pacts");
  let path_str = CString::new(pacts_path.to_string_lossy().to_string()).unwrap();
  pactffi_verifier_add_directory_source(handle, path_str.as_ptr());

  let _result = pactffi_verifier_execute(handle);
  let output_ptr = pactffi_verifier_output(handle, 0);
  let output = unsafe { CString::from_raw(output_ptr as *mut c_char) };

  pactffi_verifier_shutdown(handle);

  expect!(output.to_string_lossy().contains("Verifying a pact between test_consumer and test_provider")).to(be_true());
  expect!(output.to_string_lossy().contains("Verifying a pact between test_consumer and test_provider2")).to(be_false());
}

#[test_log::test]
fn test_missing_plugin() {
  let name = CString::new("tests").unwrap();
  let version = CString::new("1.0.0").unwrap();
  let handle = pactffi_verifier_new_for_application(name.as_ptr(), version.as_ptr());

  let provider_name = CString::new("test_provider").unwrap();
  pactffi_verifier_set_provider_info(handle, provider_name.as_ptr(), null(), null(), 0, null());

  let pacts_path = fixture_path("missing-plugin-pact.json");
  let path_str = CString::new(pacts_path.to_string_lossy().to_string()).unwrap();
  pactffi_verifier_add_file_source(handle, path_str.as_ptr());

  let tmp_dir = TempDir::new().unwrap();
  env::set_var("PACT_PLUGIN_DIR", tmp_dir.path());

  let result = pactffi_verifier_execute(handle);
  let output_ptr = pactffi_verifier_output(handle, 0);
  let output = unsafe { CString::from_raw(output_ptr as *mut c_char) };

  env::remove_var("PACT_PLUGIN_DIR");
  pactffi_verifier_shutdown(handle);

  expect!(result).to(be_equal_to(2));
  expect!(output.to_string_lossy().contains("Verification execution failed: Plugin missing-csv:0.0.3 was not found")).to(be_true());
}

// Issue #299
#[test_log::test]
#[allow(deprecated)]
fn each_value_matcher() {
  let consumer_name = CString::new("each_value_matcher-consumer").unwrap();
  let provider_name = CString::new("each_value_matcher-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("each_value_matcher").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("application/json").unwrap();
  let path = CString::new("/book").unwrap();
  let json = json!({
    "pact:matcher:type": "each-value",
    "value": {
      "id1": "book1"
    },
    "rules": [
      {
        "pact:matcher:type": "regex",
        "regex": "\\w+\\d+"
      }
    ]
  });
  let body = CString::new(json.to_string()).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("PUT").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), body.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let result = client.put(format!("http://127.0.0.1:{}/book", port).as_str())
    .header("Content-Type", "application/json")
    .body(r#"{"id1": "book100", "id2": "book2"}"#)
    .send();

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
    },
    Err(err) => {
      panic!("expected 200 response but request failed: {}", err);
    }
  };

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  expect!(mismatches).to(be_equal_to("[]"));

  let tmp = TempDir::new().unwrap();
  let tmp_path = tmp.path().to_string_lossy().to_string();
  let file_path = CString::new(tmp_path.as_str()).unwrap();
  pactffi_write_pact_file(port, file_path.as_ptr(), true);
  pactffi_cleanup_mock_server(port);
}

// Issue #301
#[test_log::test]
#[allow(deprecated)]
fn each_key_matcher() {
  let consumer_name = CString::new("each_key_matcher-consumer").unwrap();
  let provider_name = CString::new("each_key_matcher-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("each_key_matcher").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("application/json").unwrap();
  let path = CString::new("/book").unwrap();
  let json = json!({
    "pact:matcher:type": "each-key",
    "value": {
      "key1": "a string we don't care about",
      "key2": "1",
    },
    "rules": [
      {
        "pact:matcher:type": "regex",
        "regex": "[a-z]{3,}[0-9]"
      }
    ]
  });
  let body = CString::new(json.to_string()).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("PUT").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), body.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let result = client.put(format!("http://127.0.0.1:{}/book", port).as_str())
    .header("Content-Type", "application/json")
    .body(r#"{"1": "foo","not valid": 1,"key": "value","key2": "value"}"#)
    .send();

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  pactffi_cleanup_mock_server(port);

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(500));
    },
    Err(err) => {
      panic!("expected 500 response but request failed: {}", err);
    }
  };

  let json: Value = serde_json::from_str(mismatches.as_str()).unwrap();
  let mismatches = json.as_array().unwrap().first().unwrap().as_object()
    .unwrap().get("mismatches").unwrap().as_array().unwrap();
  let messages = mismatches.iter()
    .map(|v| v.as_object().unwrap().get("mismatch").unwrap().as_str().unwrap())
    .sorted()
    .collect_vec();
  assert_eq!(vec![
    "Expected '1' to match '[a-z]{3,}[0-9]'",
    "Expected 'key' to match '[a-z]{3,}[0-9]'",
    "Expected 'not valid' to match '[a-z]{3,}[0-9]'"
  ], messages);
}

// Issue #324
#[test_log::test]
fn array_contains_matcher() {
  let consumer_name = CString::new("array_contains_matcher-consumer").unwrap();
  let provider_name = CString::new("array_contains_matcher-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("array_contains_matcher").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("application/json").unwrap();
  let path = CString::new("/book").unwrap();
  let json = json!({
    "pact:matcher:type": "array-contains",
    "variants": [
      {
        "users": {
          "pact:matcher:type": "array-contains",
          "variants": [
            {
              "id": {
                "value": 1
              }
            },
            {
              "id": {
                "value": 2
              }
            },
          ]
        }
      },
    ]
  });
  let body = CString::new(json.to_string()).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("GET").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Response, content_type.as_ptr(), body.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let result = client.get(format!("http://127.0.0.1:{}/book", port).as_str())
    .header("Content-Type", "application/json")
    .send();

  pactffi_cleanup_mock_server(port);

  match result {
    Ok(ref res) => {
      expect!(res.status()).to(be_eq(200));
    },
    Err(err) => {
      panic!("expected 200 response but request failed: {}", err);
    }
  };

  let json: Value = result.unwrap().json().unwrap();
  let users = json.as_array().unwrap().first().unwrap().as_object()
    .unwrap().get("users").unwrap();

  if users.is_null() {
    panic!("'users' field is null in JSON");
  }
  expect!(users).to(be_equal_to(&json!([
    {
      "id": { "value": 1 }
    },
    {
      "id": { "value": 2 }
    },
  ])));
}

// Issue #332
#[test_log::test]
#[allow(deprecated)]
fn multiple_query_values_with_regex_matcher() {
  let consumer_name = CString::new("http-consumer-query").unwrap();
  let provider_name = CString::new("http-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("request_with_query_matcher").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());
  let path = CString::new("/request").unwrap();
  let query_param_matcher = CString::new("{\"value\":[\"1\"],\"pact:matcher:type\":\"regex\", \"regex\":\"\\\\d+\"}").unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("GET").unwrap();
  let query =  CString::new("foo").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_query_parameter_v2(interaction.clone(), query.as_ptr(), 0, query_param_matcher.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);
  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let result = client.get(format!("http://127.0.0.1:{}/request?foo=1&foo=443&foo=112", port).as_str())
    .send();

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
    },
    Err(_) => {
      panic!("expected 200 response but request failed");
    }
  };

  let mismatches = unsafe {
    CStr::from_ptr(pactffi_mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  pactffi_cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[]"));
}

// Issue #389
#[test_log::test]
fn merging_pact_file() {
  let pact_handle = PactHandle::new("MergingPactC", "MergingPactP");
  pactffi_with_specification(pact_handle, PactSpecification::V4);

  let description = CString::new("a request for an order with an unknown ID").unwrap();
  let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());

  let path = CString::new("/api/orders/404").unwrap();
  let method = CString::new("GET").unwrap();
  let result_1 = pactffi_with_request(i_handle, method.as_ptr(), path.as_ptr());

  let accept = CString::new("Accept").unwrap();
  let header = CString::new("application/json").unwrap();
  let result_2 = pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());

  let result_3 = pactffi_response_status(i_handle, 200);

  let tmp = tempfile::tempdir().unwrap();
  let tmp_dir = CString::new(tmp.path().to_string_lossy().as_bytes().to_vec()).unwrap();
  let result_4 = pactffi_pact_handle_write_file(pact_handle, tmp_dir.as_ptr(), false);

  pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  let result_5 = pactffi_pact_handle_write_file(pact_handle, tmp_dir.as_ptr(), false);

  let x_test = CString::new("X-Test").unwrap();
  pactffi_with_header_v2(i_handle, InteractionPart::Request, x_test.as_ptr(), 0, header.as_ptr());
  let result_6 = pactffi_pact_handle_write_file(pact_handle, tmp_dir.as_ptr(), false);

  let pact_file = pact_default_file_name(&pact_handle);
  pactffi_free_pact_handle(pact_handle);

  expect!(result_1).to(be_true());
  expect!(result_2).to(be_true());
  expect!(result_3).to(be_true());
  expect!(result_4).to(be_equal_to(0));
  expect!(result_5).to(be_equal_to(0));
  expect!(result_6).to(be_equal_to(0));

  let pact_path = tmp.path().join(pact_file.unwrap());
  let f= File::open(pact_path).unwrap();

  let mut json: Value = serde_json::from_reader(f).unwrap();
  json["metadata"] = Value::Null;
  assert_eq!(serde_json::to_string_pretty(&json).unwrap(),
  r#"{
  "consumer": {
    "name": "MergingPactC"
  },
  "interactions": [
    {
      "description": "a request for an order with an unknown ID",
      "pending": false,
      "request": {
        "headers": {
          "Accept": [
            "application/json"
          ],
          "X-Test": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/api/orders/404"
      },
      "response": {
        "status": 200
      },
      "type": "Synchronous/HTTP"
    }
  ],
  "metadata": null,
  "provider": {
    "name": "MergingPactP"
  }
}"#
  );
}

// Issue #389
#[test_log::test]
fn repeated_interaction() {
  let pact_handle = PactHandle::new("MergingPactC2", "MergingPactP2");
  pactffi_with_specification(pact_handle, PactSpecification::V4);

  let description = CString::new("a request for an order with an unknown ID").unwrap();
  let path = CString::new("/api/orders/404").unwrap();
  let method = CString::new("GET").unwrap();
  let accept = CString::new("Accept").unwrap();
  let header = CString::new("application/json").unwrap();

  let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());
  pactffi_with_request(i_handle, method.as_ptr(), path.as_ptr());
  pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  pactffi_response_status(i_handle, 200);

  let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());
  pactffi_with_request(i_handle, method.as_ptr(), path.as_ptr());
  pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  pactffi_response_status(i_handle, 200);

  let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());
  pactffi_with_request(i_handle, method.as_ptr(), path.as_ptr());
  pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  pactffi_response_status(i_handle, 200);

  let tmp = tempfile::tempdir().unwrap();
  let tmp_dir = CString::new(tmp.path().to_string_lossy().as_bytes().to_vec()).unwrap();
  let result = pactffi_pact_handle_write_file(pact_handle, tmp_dir.as_ptr(), false);

  let pact_file = pact_default_file_name(&pact_handle);
  pactffi_free_pact_handle(pact_handle);

  expect!(result).to(be_equal_to(0));

  let pact_path = tmp.path().join(pact_file.unwrap());
  let f= File::open(pact_path).unwrap();

  let mut json: Value = serde_json::from_reader(f).unwrap();
  json["metadata"] = Value::Null;
  assert_eq!(serde_json::to_string_pretty(&json).unwrap(),
  r#"{
  "consumer": {
    "name": "MergingPactC2"
  },
  "interactions": [
    {
      "description": "a request for an order with an unknown ID",
      "pending": false,
      "request": {
        "headers": {
          "Accept": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/api/orders/404"
      },
      "response": {
        "status": 200
      },
      "type": "Synchronous/HTTP"
    }
  ],
  "metadata": null,
  "provider": {
    "name": "MergingPactP2"
  }
}"#
  );
}

// Issue #298
#[test_log::test]
fn provider_states_ignoring_parameter_types() {
  let pact_handle = PactHandle::new("PSIPTC", "PSIPTP");
  pactffi_with_specification(pact_handle, PactSpecification::V4);

  let description = CString::new("an order with ID {id} exists").unwrap();
  let path = CString::new("/api/orders/404").unwrap();
  let method = CString::new("GET").unwrap();
  let accept = CString::new("Accept").unwrap();
  let header = CString::new("application/json").unwrap();
  let state_params = CString::new(r#"{"id": "1"}"#).unwrap();

  let i_handle = pactffi_new_interaction(pact_handle, description.as_ptr());
  pactffi_with_request(i_handle, method.as_ptr(), path.as_ptr());
  pactffi_given_with_params(i_handle, description.as_ptr(), state_params.as_ptr());
  pactffi_with_header_v2(i_handle, InteractionPart::Request, accept.as_ptr(), 0, header.as_ptr());
  pactffi_response_status(i_handle, 200);

  let tmp = tempfile::tempdir().unwrap();
  let tmp_dir = CString::new(tmp.path().to_string_lossy().as_bytes().to_vec()).unwrap();
  let result = pactffi_pact_handle_write_file(pact_handle, tmp_dir.as_ptr(), false);

  let pact_file = pact_default_file_name(&pact_handle);
  pactffi_free_pact_handle(pact_handle);

  expect!(result).to(be_equal_to(0));

  let pact_path = tmp.path().join(pact_file.unwrap());
  let f= File::open(pact_path).unwrap();

  let mut json: Value = serde_json::from_reader(f).unwrap();
  json["metadata"] = Value::Null;
  assert_eq!(serde_json::to_string_pretty(&json).unwrap(),
  r#"{
  "consumer": {
    "name": "PSIPTC"
  },
  "interactions": [
    {
      "description": "an order with ID {id} exists",
      "pending": false,
      "providerStates": [
        {
          "name": "an order with ID {id} exists",
          "params": {
            "id": "1"
          }
        }
      ],
      "request": {
        "headers": {
          "Accept": [
            "application/json"
          ]
        },
        "method": "GET",
        "path": "/api/orders/404"
      },
      "response": {
        "status": 200
      },
      "type": "Synchronous/HTTP"
    }
  ],
  "metadata": null,
  "provider": {
    "name": "PSIPTP"
  }
}"#
  );
}

// Issue #399
#[test_log::test]
fn combined_each_key_and_each_value_matcher() {
  let consumer_name = CString::new("combined_matcher-consumer").unwrap();
  let provider_name = CString::new("combined_matcher-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("combined_matcher").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("application/json").unwrap();
  let path = CString::new("/query").unwrap();
  let json = json!({
    "results": {
      "pact:matcher:type": [
        {
          "pact:matcher:type": "each-key",
          "value": "AUK-155332",
          "rules": [
            {
              "pact:matcher:type": "regex",
              "regex": "\\w{3}-\\d+"
            }
          ]
        }, {
          "pact:matcher:type": "each-value",
          "rules": [
            {
              "pact:matcher:type": "type"
            }
          ]
        }
      ],
      "AUK-155332": {
        "title": "...",
        "description": "...",
        "link": "http://....",
        "relatesTo": ["BAF-88654"]
      }
    }
  });
  let body = CString::new(json.to_string()).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("PUT").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), body.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let json_body = json!({
    "results": {
      "KGK-9954356": {
        "title": "Some title",
        "description": "Tells us what this is in more or less detail",
        "link": "http://....",
        "relatesTo": ["BAF-88654"]
      }
    }
  });
  let result = client.put(format!("http://127.0.0.1:{}/query", port).as_str())
    .header("Content-Type", "application/json")
    .body(json_body.to_string())
    .send();

  let mismatches = pactffi_mock_server_mismatches(port);
  println!("{}", unsafe { CStr::from_ptr(mismatches) }.to_string_lossy());

  pactffi_cleanup_mock_server(port);
  pactffi_free_pact_handle(pact_handle);

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
    },
    Err(err) => {
      panic!("expected 200 response but request failed: {}", err);
    }
  };
}

// Issue #399
#[test_log::test]
fn matching_definition_expressions_matcher() {
  let consumer_name = CString::new("combined_matcher-consumer").unwrap();
  let provider_name = CString::new("combined_matcher-provider").unwrap();
  let pact_handle = pactffi_new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("matching_definition_expressions").unwrap();
  let interaction = pactffi_new_interaction(pact_handle.clone(), description.as_ptr());

  let content_type = CString::new("application/json").unwrap();
  let path = CString::new("/query").unwrap();
  let json = json!({
    "results": {
      "pact:matcher:type": "eachKey(matching(regex, '\\w{3}-\\d+', 'AUK-155332')), eachValue(matching(type, ''))",
      "AUK-155332": {
        "title": "...",
        "description": "...",
        "link": "http://....",
        "relatesTo": ["BAF-88654"]
      }
    }
  });
  let body = CString::new(json.to_string()).unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let method = CString::new("PUT").unwrap();

  pactffi_upon_receiving(interaction.clone(), description.as_ptr());
  pactffi_with_request(interaction.clone(), method.as_ptr(), path.as_ptr());
  pactffi_with_body(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), body.as_ptr());
  pactffi_response_status(interaction.clone(), 200);

  let port = pactffi_create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  let client = Client::default();
  let json_body = json!({
    "results": {
      "KGK-9954356": {
        "title": "Some title",
        "description": "Tells us what this is in more or less detail",
        "link": "http://....",
        "relatesTo": ["BAF-88654"]
      }
    }
  });
  let result = client.put(format!("http://127.0.0.1:{}/query", port).as_str())
    .header("Content-Type", "application/json")
    .body(json_body.to_string())
    .send();

  let mismatches = pactffi_mock_server_mismatches(port);
  println!("{}", unsafe { CStr::from_ptr(mismatches) }.to_string_lossy());

  pactffi_cleanup_mock_server(port);
  pactffi_free_pact_handle(pact_handle);

  match result {
    Ok(res) => {
      expect!(res.status()).to(be_eq(200));
    },
    Err(err) => {
      panic!("expected 200 response but request failed: {}", err);
    }
  };
}

// Run independently as this log settings are global, and other tests affect this one.
// cargo test -p pact_ffi returns_mock_server_logs -- --nocapture --include-ignored
#[ignore]
#[test]
fn returns_mock_server_logs() {
  let pact_json = include_str!("post-pact.json");
  let pact_json_c = CString::new(pact_json).expect("Could not construct C string from json");
  let address = CString::new("127.0.0.1:0").unwrap();
  #[allow(deprecated)]
  let port = pactffi_create_mock_server(pact_json_c.as_ptr(), address.as_ptr(), false);
  expect!(port).to(be_greater_than(0));
  pactffi_log_to_buffer(LevelFilter::Debug.into());
  let client = Client::default();
  client.post(format!("http://127.0.0.1:{}/path", port).as_str())
    .header(CONTENT_TYPE, "application/json")
    .body(r#"{"foo":"no-very-bar"}"#)
    .send().expect("Sent POST request to mock server");

  let logs =  unsafe {
    CStr::from_ptr(pactffi_mock_server_logs(port)).to_string_lossy().into_owned()
  };
  println!("{}",logs);
  assert_ne!(logs,"", "logs are empty");

  pactffi_cleanup_mock_server(port);
}