use super::*;
use expectest::prelude::*;
use expectest::expect;
use crate::models::{Request, Response, OptionalBody, DetectedContentType};
use crate::models::generators::{JsonHandler, ContentTypeHandler};
use std::str::FromStr;
use serde_json::Value;
use maplit::*;
use hamcrest2::prelude::*;
use hamcrest2::*;
use xmltree::{Element, XMLNode};
use tempfile::tempfile;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

#[test]
fn returns_original_response_if_there_are_no_generators() {
  let response = Response::default();
  expect!(generate_response(&response, &hashmap!{})).to(be_equal_to(response));
}

#[test]
fn applies_status_generator_for_status_to_the_copy_of_the_response() {
  let response = Response { status: 200, generators: generators! {
    "STATUS" => Generator::RandomInt(400, 499)
  }, .. Response::default() };
  expect!(generate_response(&response, &hashmap!{}).status).to(be_greater_or_equal_to(400));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_response() {
  let response = Response { headers: Some(hashmap!{
      s!("A") => vec![s!("a")],
      s!("B") => vec![s!("b")]
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Response::default()
  };
  let headers = generate_response(&response, &hashmap!{}).headers.unwrap().clone();
  expect!(headers.get("A").unwrap().first().unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn returns_original_request_if_there_are_no_generators() {
  let request = Request::default();
  expect!(generate_request(&request, &hashmap!{})).to(be_equal_to(request));
}

#[test]
fn applies_path_generator_for_the_path_to_the_copy_of_the_request() {
  let request = Request { path: s!("/path"), generators: generators! {
    "PATH" => Generator::RandomInt(1, 10)
  }, .. Request::default() };
  expect!(generate_request(&request, &hashmap!{}).path).to_not(be_equal_to("/path"));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_request() {
  let request = Request { headers: Some(hashmap!{
      s!("A") => vec![s!("a")],
      s!("B") => vec![s!("b")]
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default()
  };
  let headers = generate_request(&request, &hashmap!{}).headers.unwrap().clone();
  expect!(headers.get("A").unwrap().first().unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn applies_query_generator_for_query_parameters_to_the_copy_of_the_request() {
  let request = Request { query: Some(hashmap!{
      s!("A") => vec![ s!("a") ],
      s!("B") => vec![ s!("b") ]
    }), generators: generators! {
      "QUERY" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default()
  };
  let query = generate_request(&request, &hashmap!{}).query.unwrap().clone();
  let query_val = &query.get("A").unwrap()[0];
  expect!(query_val).to_not(be_equal_to("a"));
}

#[test]
fn apply_generator_to_empty_body_test() {
  let generators = Generators::default();
  expect!(generators.apply_body_generators(&OptionalBody::Empty, DetectedContentType::Text, &hashmap!{})).to(be_ok().value(OptionalBody::Empty));
  expect!(generators.apply_body_generators(&OptionalBody::Null, DetectedContentType::Text, &hashmap!{})).to(be_ok().value(OptionalBody::Null));
  expect!(generators.apply_body_generators(&OptionalBody::Missing, DetectedContentType::Text, &hashmap!{})).to(be_ok().value(OptionalBody::Missing));
}

#[test]
fn do_not_apply_generators_if_there_are_no_body_generators() {
  let generators = Generators::default();
  let body = OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into());
  expect!(generators.apply_body_generators(&body, DetectedContentType::Json, &hashmap!{})).to(be_ok().value(body));
}

#[test]
fn apply_generator_to_text_body_test() {
  let generators = Generators::default();
  let body = OptionalBody::Present("some text".into());
  expect!(generators.apply_body_generators(&body, DetectedContentType::Text, &hashmap!{})).to(be_ok().value(body));
}

#[test]
fn applies_body_generator_to_the_copy_of_the_request() {
  let request = Request { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into()),
    generators: generators! {
      "BODY" => {
        "$.a" => Generator::RandomInt(1, 10)
      }
    }, .. Request::default()
  };
  let generated_request = generate_request(&request, &hashmap!{});
  let body: Value = serde_json::from_str(generated_request.body.str_value()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[test]
fn applies_body_generator_to_the_copy_of_the_response() {
  let response = Response { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into()),
    generators: generators! {
      "BODY" => {
        "$.a" => Generator::RandomInt(1, 10)
      }
    }, .. Response::default()
  };
  let body: Value = serde_json::from_str(generate_response(&response, &hashmap!{}).body.str_value()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[test]
fn does_not_change_body_if_there_are_no_generators() {
  let body = OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into());
  let generators = generators!{};
  let processed = generators.apply_body_generators(&body, DetectedContentType::Json,
    &hashmap!{});
  expect!(processed).to(be_ok().value(body));
}

#[test]
fn applies_the_generator_to_a_json_map_entry() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&s!("$.b"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
}

#[test]
fn json_generator_handles_invalid_path_expressions() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&s!("$["), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(json_handler.value).to(be_equal_to(json!({"a": 100, "b": "B", "c": "C"})));
}

#[test]
fn does_not_apply_the_generator_when_field_is_not_in_map() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&s!("$.d"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(json_handler.value).to(be_equal_to(json!({"a": 100, "b": "B", "c": "C"})));
}

#[test]
fn does_not_apply_the_generator_when_not_a_map() {
  let map = json!(100);
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&s!("$.d"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(json_handler.value).to(be_equal_to(json!(100)));
}

#[test]
fn applies_the_generator_to_a_list_item() {
  let list = json!([100, 200, 300]);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&s!("$[1]"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value[1]).to_not(be_equal_to(&json!(200)));
}

#[test]
fn does_not_apply_the_generator_when_index_is_not_in_list() {
  let list = json!([100, 200, 300]);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&s!("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(json_handler.value).to(be_equal_to(json!([100, 200, 300])));
}

#[test]
fn does_not_apply_the_generator_when_not_a_list() {
  let list = json!(100);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&s!("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(json_handler.value).to(be_equal_to(json!(100)));
}

#[test]
fn applies_the_generator_to_the_root() {
  let value = json!(100);
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value).to_not(be_equal_to(&json!(100)));
}

#[test]
fn applies_the_generator_to_the_object_graph() {
  let value = json!({
    "a": ["A", {"a": "A", "b": {"1": "1", "2": "2"}, "c": "C"}, "C"],
    "b": "B",
    "c": "C"
  });
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$.a[1].b['2']"), &Generator::RandomInt(3, 10), &hashmap!{});

  expect!(&json_handler.value["a"][1]["b"]["2"]).to_not(be_equal_to(&json!("2")));
}

#[test]
fn does_not_apply_the_generator_to_the_object_graph_when_the_expression_does_not_match() {
  let value = json!({
    "a": "A",
    "b": "B",
    "c": "C"
  });
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$.a[1].b['2']"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value).to(be_equal_to(&json!({
    "a": "A",
    "b": "B",
    "c": "C"
  })));
}

#[test]
fn applies_the_generator_to_all_map_entries() {
  let value = json!({
    "a": "A",
    "b": "B",
    "c": "C"
  });
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$.*"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value["a"]).to_not(be_equal_to(&json!("A")));
  expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
  expect!(&json_handler.value["c"]).to_not(be_equal_to(&json!("C")));
}

#[test]
fn applies_the_generator_to_all_list_items() {
  let value = json!(["A", "B", "C"]);
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$[*]"), &Generator::RandomInt(0, 10), &hashmap!{});

  expect!(&json_handler.value[0]).to_not(be_equal_to(&json!("A")));
  expect!(&json_handler.value[1]).to_not(be_equal_to(&json!("B")));
  expect!(&json_handler.value[2]).to_not(be_equal_to(&json!("C")));
}

#[test]
fn applies_the_generator_to_the_object_graph_with_wildcard() {
  let value = json!({
    "a": ["A", {"a": "A", "b": ["1", "2"], "c": "C"}, "C"],
    "b": "B",
    "c": "C"
  });
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&s!("$.*[1].b[*]"), &Generator::RandomInt(3, 10), &hashmap!{});

  expect!(&json_handler.value["a"][0]).to(be_equal_to(&json!("A")));
  expect!(&json_handler.value["a"][1]["a"]).to(be_equal_to(&json!("A")));
  expect!(&json_handler.value["a"][1]["b"][0]).to_not(be_equal_to(&json!("1")));
  expect!(&json_handler.value["a"][1]["b"][1]).to_not(be_equal_to(&json!("2")));
  expect!(&json_handler.value["a"][1]["c"]).to(be_equal_to(&json!("C")));
  expect!(&json_handler.value["a"][2]).to(be_equal_to(&json!("C")));
  expect!(&json_handler.value["b"]).to(be_equal_to(&json!("B")));
  expect!(&json_handler.value["c"]).to(be_equal_to(&json!("C")));
}

#[test]
fn date_generator_test() {
  let generated = Generator::Date(None).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}$"));

  let generated2 = Generator::Date(Some("yyyy-MM-ddZ".into())).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}[-+]\d{4}$"));
}

#[test]
fn time_generator_test() {
  let generated = Generator::Time(None).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}$"));

  let generated2 = Generator::Time(Some("HH:mm:ssZ".into())).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated2.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}[-+]\d+$"));
}

#[test]
fn datetime_generator_test() {
  let generated = Generator::DateTime(None).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}[-+]\d+$"));

  let generated2 = Generator::DateTime(Some("yyyy-MM-dd HH:mm:ssZ".into())).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[-+]\d+$"));
}

#[test]
fn regex_generator_test() {
  let generated = Generator::Regex(r"\d{4}\w{1,4}".into()).generate_value("".to_string(), &hashmap!{});
  assert_that!(generated.unwrap(), matches_regex(r"^\d{4}\w{1,4}$"));
}

#[test]
fn applies_the_generator_to_xml() {
  let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <root>
      <a>100</a>
      <b></b>
      <c>C</c>
    </root>
  "#;
  let doc = Element::parse(xml.as_bytes()).unwrap();
  let mut xml_handler = XmlHandler { value: doc };

  xml_handler.apply_key(&s!("$.root.b"), &Generator::RandomInt(0, 10), &hashmap!{});

  let mut file: File = tempfile().unwrap();
  xml_handler.value.write(file.try_clone().unwrap()).unwrap();
  file.seek(SeekFrom::Start(0)).unwrap();
  let mut buffer = String::new();
  file.read_to_string(&mut buffer).unwrap();
  expect!(buffer.clone()).to_not(be_equal_to(xml.to_string()));

  let doc2 = Element::parse(buffer.as_bytes()).unwrap();
  match &doc2.children[1] {
    XMLNode::Element(element) => {
      expect!(element.get_text().unwrap().to_string().chars()).to_not(be_empty())
    },
    _ => panic!("Invalid XML node found")
  };
}
