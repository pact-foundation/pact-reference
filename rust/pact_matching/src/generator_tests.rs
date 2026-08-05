use std::str::FromStr;

use expectest::expect;
use expectest::prelude::*;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::generators;
use pact_models::generators::{ContentTypeHandler, Generator, JsonHandler};
use pact_models::message::Message;
use pact_models::path_exp::DocPath;
use serde_json::Value;

use crate::generators::generate_message;

use super::*;

#[tokio::test]
async fn returns_original_response_if_there_are_no_generators() {
  let response = HttpResponse::default();
  expect!(generate_response(&response, &GeneratorTestMode::Provider, &hashmap!{}).await).to(be_equal_to(response));
}

#[tokio::test]
async fn applies_status_generator_for_status_to_the_copy_of_the_response() {
  let response = HttpResponse { status: 200, generators: generators! {
    "STATUS" => Generator::RandomInt(400, 499)
  }, .. HttpResponse::default() };
  expect!(generate_response(&response, &GeneratorTestMode::Provider, &hashmap!{}).await.status).to(be_greater_or_equal_to(400));
}

#[tokio::test]
async fn applies_header_generator_for_headers_to_the_copy_of_the_response() {
  let response = HttpResponse { headers: Some(hashmap!{
      s!("A") => vec![s!("a")],
      s!("B") => vec![s!("b")]
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid(None)
      }
    }, .. HttpResponse::default()
  };
  let headers = generate_response(&response, &GeneratorTestMode::Provider, &hashmap!{}).await.headers.unwrap().clone();
  expect!(headers.get("A").unwrap().first().unwrap()).to_not(be_equal_to("a"));
}

#[tokio::test]
async fn returns_original_request_if_there_are_no_generators() {
  let request = HttpRequest::default();
  expect!(generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await).to(be_equal_to(request));
}

#[tokio::test]
async fn applies_path_generator_for_the_path_to_the_copy_of_the_request() {
  let request = HttpRequest { path: s!("/path"), generators: generators! {
    "PATH" => Generator::RandomInt(1, 10)
  }, .. HttpRequest::default() };
  expect!(generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await.path).to_not(be_equal_to("/path"));
}

#[tokio::test]
async fn applies_header_generator_for_headers_to_the_copy_of_the_request() {
  let request = HttpRequest { headers: Some(hashmap!{
      s!("A") => vec![s!("a")],
      s!("B") => vec![s!("b")]
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid(None)
      }
    }, .. HttpRequest::default()
  };
  let headers = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await.headers.unwrap().clone();
  expect!(headers.get("A").unwrap().first().unwrap()).to_not(be_equal_to("a"));
}

#[tokio::test]
async fn applies_query_generator_for_query_parameters_to_the_copy_of_the_request() {
  let request = HttpRequest { query: Some(hashmap!{
      "A".to_string() => vec![ Some("a".to_string()) ],
      "B".to_string() => vec![ Some("b".to_string()) ]
    }), generators: generators! {
      "QUERY" => {
        "A" => Generator::Uuid(None)
      }
    }, .. HttpRequest::default()
  };
  let query = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await.query.unwrap().clone();
  let query_val = query.get("A").unwrap()[0].as_ref();
  expect!(query_val.unwrap()).to_not(be_equal_to("a"));
}

#[test_log::test(tokio::test)]
async fn applies_provider_state_generator_for_query_parameters_with_square_brackets() {
  let request = HttpRequest {
    query: Some(hashmap!{
      "A".to_string() => vec![ Some("a".to_string()) ],
      "q[]".to_string() => vec![ Some("q1".to_string()), Some("q2".to_string()) ]
    }),
    generators: generators! {
      "QUERY" => {
        "A" => Generator::ProviderStateGenerator("exp1".to_string(), None),
        "$['q[]']" => Generator::ProviderStateGenerator("${exp2}".to_string(), None)
      }
    }, .. HttpRequest::default()
  };
  let context = hashmap! {
    "exp1" => json!("1234"),
    "exp2" => json!("5678")
  };
  let result = generate_request(&request, &GeneratorTestMode::Provider, &context).await;
  let query = result.query.unwrap();
  let a_val = query.get("A").unwrap();
  expect!(a_val).to(be_equal_to(&vec![Some("1234".to_string())]));
  let q_val = query.get("q[]").unwrap();
  expect!(q_val).to(be_equal_to(&vec![Some("5678".to_string()), Some("5678".to_string())]));
}

#[tokio::test]
async fn applies_body_generator_to_the_copy_of_the_request() {
  let request = HttpRequest { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into(), None, None),
    generators: generators! {
      "BODY" => {
        "$.a" => Generator::RandomInt(1, 10)
      }
    }, .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[tokio::test]
async fn applies_body_generator_to_the_copy_of_the_response() {
  let response = HttpResponse { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into(), None, None),
    generators: generators! {
      "BODY" => {
        "$.a" => Generator::RandomInt(1, 10)
      }
    }, .. HttpResponse::default()
  };
  let body: Value = serde_json::from_str(generate_response(&response,
   &GeneratorTestMode::Provider, &hashmap!{}).await.body.display_string().as_str()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[test]
fn applies_the_generator_to_a_json_map_entry() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&DocPath::new_unwrap("$.b"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
}

#[test]
fn does_not_apply_the_generator_when_field_is_not_in_map() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&DocPath::new_unwrap("$.d"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(json_handler.value).to(be_equal_to(json!({"a": 100, "b": "B", "c": "C"})));
}

#[test]
fn does_not_apply_the_generator_when_not_a_map() {
  let map = json!(100);
  let mut json_handler = JsonHandler { value: map };

  json_handler.apply_key(&DocPath::new_unwrap("$.d"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(json_handler.value).to(be_equal_to(json!(100)));
}

#[test]
fn applies_the_generator_to_a_list_item() {
  let list = json!([100, 200, 300]);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&DocPath::new_unwrap("$[1]"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(&json_handler.value[1]).to_not(be_equal_to(&json!(200)));
}

#[test]
fn does_not_apply_the_generator_when_index_is_not_in_list() {
  let list = json!([100, 200, 300]);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&DocPath::new_unwrap("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(json_handler.value).to(be_equal_to(json!([100, 200, 300])));
}

#[test]
fn does_not_apply_the_generator_when_not_a_list() {
  let list = json!(100);
  let mut json_handler = JsonHandler { value: list };

  json_handler.apply_key(&DocPath::new_unwrap("$[3]"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(json_handler.value).to(be_equal_to(json!(100)));
}

#[test]
fn applies_the_generator_to_the_root() {
  let value = json!(100);
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&DocPath::root(), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

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

  json_handler.apply_key(&DocPath::new_unwrap("$.a[1].b['2']"), &Generator::RandomInt(3, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

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

  json_handler.apply_key(&DocPath::new_unwrap("$.a[1].b['2']"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

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

  json_handler.apply_key(&DocPath::new_unwrap("$.*"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(&json_handler.value["a"]).to_not(be_equal_to(&json!("A")));
  expect!(&json_handler.value["b"]).to_not(be_equal_to(&json!("B")));
  expect!(&json_handler.value["c"]).to_not(be_equal_to(&json!("C")));
}

#[test]
fn applies_the_generator_to_all_list_items() {
  let value = json!(["A", "B", "C"]);
  let mut json_handler = JsonHandler { value };

  json_handler.apply_key(&DocPath::new_unwrap("$[*]"), &Generator::RandomInt(0, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

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

  json_handler.apply_key(&DocPath::new_unwrap("$.*[1].b[*]"), &Generator::RandomInt(3, 10), &hashmap!{}, &DefaultVariantMatcher.boxed());

  expect!(&json_handler.value["a"][0]).to(be_equal_to(&json!("A")));
  expect!(&json_handler.value["a"][1]["a"]).to(be_equal_to(&json!("A")));
  expect!(&json_handler.value["a"][1]["b"][0]).to_not(be_equal_to(&json!("1")));
  expect!(&json_handler.value["a"][1]["b"][1]).to_not(be_equal_to(&json!("2")));
  expect!(&json_handler.value["a"][1]["c"]).to(be_equal_to(&json!("C")));
  expect!(&json_handler.value["a"][2]).to(be_equal_to(&json!("C")));
  expect!(&json_handler.value["b"]).to(be_equal_to(&json!("B")));
  expect!(&json_handler.value["c"]).to(be_equal_to(&json!("C")));
}

#[tokio::test]
async fn returns_original_message_if_there_are_no_generators() {
  let message = Message::default();
  expect!(generate_message(&message, &GeneratorTestMode::Provider, &hashmap!{}, &vec![], &hashmap!{}).await).to(be_equal_to(message));
}

#[tokio::test]
async fn applies_metadata_generator_for_to_the_copy_of_the_message() {
  let message = Message {
    metadata: hashmap!{
      "A".to_string() => json!("a"),
      "B".to_string() => json!("b")
    },
    generators: generators! {
      "METADATA" => {
        "A" => Generator::Uuid(None)
      }
    }, ..  Message::default()
  };
  let generated = generate_message(&message, &GeneratorTestMode::Provider, &hashmap!{}, &vec![], &hashmap!{}).await;
  expect!(generated.metadata.get("A").unwrap()).to_not(be_equal_to(&json!("a")));
}

#[tokio::test]
async fn applies_body_generator_to_the_copy_of_the_message() {
  let message = Message {
    contents: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into(), Some(JSON.clone()), None),
    generators: generators! {
      "BODY" => {
        "$.a" => Generator::RandomInt(1, 10)
      }
    }, ..  Message::default()
  };
  let generated = generate_message(&message, &GeneratorTestMode::Provider, &hashmap!{}, &vec![], &hashmap!{}).await;
  let json_str = generated.contents.value_as_string().unwrap();
  let body: Value = serde_json::from_str(json_str.as_str()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[tokio::test]
async fn applies_random_array_generator_to_request_body() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"name": "xxx", "price": 12, "count": 2}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$.items" => Generator::RandomArray(2, 4) } },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(4));
  let first_item = &items[0];
  for item in items {
    expect!(item).to(be_equal_to(first_item));
  }
}

#[tokio::test]
async fn applies_random_array_generator_to_response_body() {
  let response = HttpResponse {
    body: OptionalBody::Present(r#"{"items": [{"name": "yyy", "price": 6, "count": 2}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$.items" => Generator::RandomArray(2, 4) } },
    .. HttpResponse::default()
  };
  let generated_response = generate_response(&response, &GeneratorTestMode::Consumer, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_response.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(4));
  let first_item = &items[0];
  for item in items {
    expect!(item).to(be_equal_to(first_item));
  }
}

#[tokio::test]
async fn applies_random_array_generator_with_nested_generators_to_request_body() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"name": "xxx", "price": 12, "count": 2}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! {
      "BODY" => {
        "$.items" => Generator::RandomArray(2, 4),
        "$.items[*].name" => Generator::RandomString(5),
        "$.items[*].price" => Generator::RandomInt(1, 100),
        "$.items[*].count" => Generator::RandomInt(1, 10)
      }
    },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(4));
  for i in 1..items.len() {
    expect!(&items[i]).not_to(be_equal_to(&items[i - 1]));
  }
}

#[tokio::test]
async fn applies_random_array_generator_with_nested_generators_to_response_body() {
  let response = HttpResponse {
    body: OptionalBody::Present(r#"{"items": [{"name": "yyy", "price": 6, "count": 2}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! {
      "BODY" => {
        "$.items" => Generator::RandomArray(2, 4),
        "$.items[*].name" => Generator::RandomString(5),
        "$.items[*].price" => Generator::RandomInt(1, 100),
        "$.items[*].count" => Generator::RandomInt(1, 10)
      }
    },
    .. HttpResponse::default()
  };
  let generated_response = generate_response(&response, &GeneratorTestMode::Consumer, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_response.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(4));
  for i in 1..items.len() {
    expect!(&items[i]).not_to(be_equal_to(&items[i - 1]));
  }
}

#[tokio::test]
async fn applies_nested_random_array_generator() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"subitems": [{"value": 1}]}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! {
      "BODY" => {
        "$.items" => Generator::RandomArray(2, 3),
        "$.items[*].subitems" => Generator::RandomArray(2, 3),
        "$.items[*].subitems[*].value" => Generator::RandomInt(1, 1000)
      }
    },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(3));
  for i in 1..items.len() {
    expect!(&items[i]).not_to(be_equal_to(&items[i - 1]));
    let subitems_curr = items[i]["subitems"].as_array().unwrap();
    let subitems_prev = items[i - 1]["subitems"].as_array().unwrap();
    expect!(subitems_curr).not_to(be_equal_to(subitems_prev));
    for j in 1..subitems_curr.len() {
      expect!(&subitems_curr[j]).not_to(be_equal_to(&subitems_curr[j - 1]));
    }
  }
}

#[tokio::test]
async fn applies_random_array_generator_with_same_min_max() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"value": 1}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$.items" => Generator::RandomArray(3, 3) } },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  expect!(items.len()).to(be_equal_to(3));
}

#[tokio::test]
async fn cant_applies_random_array_generator_with_min_max_zero() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"value": 1}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$.items" => Generator::RandomArray(0, 0) } },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  expect!(generated_request.body).to(be_equal_to(OptionalBody::Present(r#"{"items":[{"value":1}]}"#.into(), Some("application/json".into()), None)));
}

#[tokio::test]
async fn applies_random_array_generator_to_root_level_array() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"[{"value": 1}]"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$" => Generator::RandomArray(2, 4) } },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body.as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(4));
}

#[tokio::test]
async fn cant_applies_random_array_generator_to_empty_array() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": []}"#.into(), Some(JSON.clone()), None),
    generators: generators! { "BODY" => { "$.items" => Generator::RandomArray(2, 4) } },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  expect!(generated_request.body).to(be_equal_to(OptionalBody::Present(r#"{"items":[]}"#.into(), Some("application/json".into()), None)));
}

#[tokio::test]
async fn applies_multiple_independent_array_generators() {
  let request = HttpRequest {
    body: OptionalBody::Present(r#"{"items": [{"value": 1}], "other": [{"x": 2}]}"#.into(), Some(JSON.clone()), None),
    generators: generators! {
      "BODY" => {
        "$.items" => Generator::RandomArray(2, 3),
        "$.other" => Generator::RandomArray(3, 4)
      }
    },
    .. HttpRequest::default()
  };
  let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
  let body: Value = serde_json::from_str(generated_request.body.display_string().as_str()).unwrap();
  let items = body["items"].as_array().unwrap();
  let other = body["other"].as_array().unwrap();
  expect!(items.len()).to(be_ge(2));
  expect!(items.len()).to(be_le(3));
  expect!(other.len()).to(be_ge(3));
  expect!(other.len()).to(be_le(4));
  let first_item = &items[0];
  let first_other = &other[0];
  for item in items {
    expect!(item).to(be_equal_to(first_item));
  }
  for item in other {
    expect!(item).to(be_equal_to(first_other));
  }
}

#[cfg(feature = "xml")]
mod xml_tests {
  use super::*;
  use pact_models::content_types::XML;

  fn count_xml_elements(body: &str, tag: &str) -> usize {
    let open_tag_with_space = format!("<{} ", tag);
    let open_tag_with_bracket = format!("<{}>", tag);
    let self_close_tag = format!("<{}/", tag);
    body.split(&open_tag_with_space).count() - 1
      + body.split(&open_tag_with_bracket).count() - 1
      + body.split(&self_close_tag).count() - 1
  }

  fn count_substring(body: &str, pattern: &str) -> usize {
    body.split(pattern).count() - 1
  }

  #[tokio::test]
  async fn applies_random_array_generator_to_xml_request_body() {
    let request = HttpRequest {
      body: OptionalBody::Present("<items><item name='xxx' price='12'/></items>".into(), Some(XML.clone()), None),
      generators: generators! { "BODY" => { "$.items.item" => Generator::RandomArray(2, 4) } },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(4));
  }

  #[tokio::test]
  async fn applies_random_array_generator_to_xml_response_body() {
    let response = HttpResponse {
      body: OptionalBody::Present("<items><item name='yyy' price='6'/></items>".into(), Some(XML.clone()), None),
      generators: generators! { "BODY" => { "$.items.item" => Generator::RandomArray(2, 4) } },
      .. HttpResponse::default()
    };
    let generated_response = generate_response(&response, &GeneratorTestMode::Consumer, &hashmap!{}).await;
    let body_str = generated_response.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(4));
  }

  #[tokio::test]
  async fn applies_random_array_generator_with_same_min_max_to_xml() {
    let request = HttpRequest {
      body: OptionalBody::Present("<items><item value='1'/></items>".into(), Some(XML.clone()), None),
      generators: generators! { "BODY" => { "$.items.item" => Generator::RandomArray(3, 3) } },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    expect!(item_count).to(be_equal_to(3));
  }

  #[tokio::test]
  async fn applies_random_array_generator_with_nested_generators_to_xml() {
    let request = HttpRequest {
      body: OptionalBody::Present("<items><item name='xxx' price='12'/></items>".into(), Some(XML.clone()), None),
      generators: generators! {
        "BODY" => {
          "$.items.item" => Generator::RandomArray(2, 4),
          "$.items.item['@name']" => Generator::RandomString(5)
        }
      },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(4));

    let name_count = count_substring(&body_str, "name='");
    expect!(name_count).to(be_ge(2));
  }

  #[tokio::test]
  async fn applies_random_array_generator_with_nested_elements_to_xml() {
    let request = HttpRequest {
      body: OptionalBody::Present("<people><person id='1'><address city='NYC'/></person></people>".into(), Some(XML.clone()), None),
      generators: generators! { "BODY" => { "$.people.person" => Generator::RandomArray(2, 2) } },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let person_count = count_xml_elements(&body_str, "person");
    let address_count = count_xml_elements(&body_str, "address");

    expect!(person_count).to(be_equal_to(2));
    expect!(address_count).to(be_equal_to(2));
  }

  #[tokio::test]
  async fn applies_multiple_independent_array_generators_to_xml() {
    let request = HttpRequest {
      body: OptionalBody::Present("<root><items><item value='1'/></items><other><entry x='2'/></other></root>".into(), Some(XML.clone()), None),
      generators: generators! {
        "BODY" => {
          "$.root.items.item" => Generator::RandomArray(2, 3),
          "$.root.other.entry" => Generator::RandomArray(3, 4)
        }
      },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    let entry_count = count_xml_elements(&body_str, "entry");

    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(3));
    expect!(entry_count).to(be_ge(3));
    expect!(entry_count).to(be_le(4));
  }

  #[tokio::test]
  async fn applies_nested_random_array_generator_to_xml() {
    let request = HttpRequest {
      body: OptionalBody::Present("<items><item><subitems><subitem value='1'/></subitems></item></items>".into(), Some(XML.clone()), None),
      generators: generators! {
        "BODY" => {
          "$.items.item" => Generator::RandomArray(2, 3),
          "$.items.item.subitems.subitem" => Generator::RandomArray(2, 3),
          "$.items.item.subitems.subitem['@value']" => Generator::RandomInt(1, 1000)
        }
      },
      .. HttpRequest::default()
    };
    let generated_request = generate_request(&request, &GeneratorTestMode::Provider, &hashmap!{}).await;
    let body_str = generated_request.body.display_string();

    let item_count = count_xml_elements(&body_str, "item");
    expect!(item_count).to(be_ge(2));
    expect!(item_count).to(be_le(3));

    let subitem_count = count_xml_elements(&body_str, "subitem");
    expect!(subitem_count).to(be_ge(4));
    expect!(subitem_count).to(be_le(9));
  }
}
