use expectest::prelude::*;
use maplit::hashmap;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};

use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::{HttpStatus, matchingrules};
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::synch_http::SynchronousHttp;

use crate::{BodyMatchResult, MatchingRule, RequestMatchResult};
use crate::engine::{
  build_request_plan,
  execute_request_plan,
  build_response_plan,
  execute_response_plan,
  NodeResult,
  NodeValue,
  PlanMatchingContext,
  setup_body_plan
};
use crate::Mismatch::{self, BodyMismatch, MethodMismatch};

mod walk_tree_tests;
mod query_tests;
mod header_tests;

#[rstest(
  case("", "''"),
  case("simple", "'simple'"),
  case("simple sentence", "'simple sentence'"),
  case("\"quoted sentence\"", "'\"quoted sentence\"'"),
  case("'quoted sentence'", "'\\'quoted sentence\\''"),
  case("new\nline", "'new\\nline'"),
)]
fn node_value_str_form_escapes_strings(#[case] input: &str, #[case] expected: &str) {
  let node = NodeValue::STRING(input.to_string());
  expect!(node.str_form()).to(be_equal_to(expected));
}

#[rstest(
  case(NodeValue::NULL, "NULL"),
  case(NodeValue::STRING("string".to_string()), "'string'"),
  case(NodeValue::STRING("a string".to_string()), "'a string'"),
  case(NodeValue::BOOL(true), "BOOL(true)"),
  case(NodeValue::MMAP(hashmap!{}), "{}"),
  case(NodeValue::MMAP(hashmap!{ "a".to_string() => vec!["A".to_string()] }), "{'a': 'A'}"),
  case(NodeValue::MMAP(hashmap!{ "a".to_string() => vec!["".to_string()] }), "{'a': ''}"),
  case(NodeValue::MMAP(hashmap!{ "a".to_string() => vec!["A".to_string()], "b".to_string() => vec!["B 1".to_string(), "B2".to_string()] }), "{'a': 'A', 'b': ['B 1', 'B2']}"),
  case(NodeValue::SLIST(vec!["A".to_string(), "B 1".to_string(), "B2".to_string()]), "['A', 'B 1', 'B2']"),
  case(NodeValue::SLIST(vec![]), "[]"),
  case(NodeValue::LIST(vec![NodeValue::STRING("A".to_string()), NodeValue::BOOL(true)]), "['A', BOOL(true)]"),
  case(NodeValue::LIST(vec![]), "[]"),
  case(NodeValue::BARRAY(vec![1, 2, 3, 65]), "BYTES(4, AQIDQQ==)"),
  case(NodeValue::NAMESPACED("stuff".to_string(), "*&^%$ %^&*&^".to_string()), "stuff:*&^%$ %^&*&^"),
  case(NodeValue::UINT(1234), "UINT(1234)"),
  case(NodeValue::JSON(Value::String("this is a string".to_string())), "json:\"this is a string\""),
  case(NodeValue::ENTRY("key".to_string(), Box::new(NodeValue::STRING("A".to_string()))), "'key' -> 'A'"),
  case(NodeValue::ENTRY("a key".to_string(), Box::new(NodeValue::BOOL(false))), "'a key' -> BOOL(false)")
)]
fn str_form_test(#[case] input: NodeValue, #[case] expected: &str) {
  expect!(input.str_form()).to(be_equal_to(expected));
}

#[rstest(
  case(NodeResult::OK, NodeResult::OK, NodeResult::OK),
  case(NodeResult::OK, NodeResult::VALUE(NodeValue::NULL), NodeResult::VALUE(NodeValue::NULL)),
  case(NodeResult::OK, NodeResult::ERROR("error".to_string()), NodeResult::ERROR("error".to_string())),
  case(NodeResult::VALUE(NodeValue::NULL), NodeResult::OK, NodeResult::VALUE(NodeValue::NULL)),
  case(NodeResult::VALUE(NodeValue::NULL), NodeResult::VALUE(NodeValue::NULL), NodeResult::VALUE(NodeValue::NULL)),
  case(NodeResult::VALUE(NodeValue::NULL), NodeResult::VALUE(NodeValue::UINT(100)), NodeResult::VALUE(NodeValue::UINT(100))),
  case(NodeResult::VALUE(NodeValue::BOOL(false)), NodeResult::VALUE(NodeValue::UINT(100)), NodeResult::VALUE(NodeValue::BOOL(false))),
  case(NodeResult::VALUE(NodeValue::BOOL(true)), NodeResult::VALUE(NodeValue::NULL), NodeResult::VALUE(NodeValue::BOOL(false))),
  case(NodeResult::VALUE(NodeValue::BOOL(true)), NodeResult::VALUE(NodeValue::BOOL(false)), NodeResult::VALUE(NodeValue::BOOL(false))),
  case(NodeResult::VALUE(NodeValue::NULL), NodeResult::ERROR("error".to_string()), NodeResult::ERROR("error".to_string())),
  case(NodeResult::ERROR("error".to_string()), NodeResult::OK, NodeResult::ERROR("error".to_string())),
  case(NodeResult::ERROR("error".to_string()), NodeResult::VALUE(NodeValue::NULL), NodeResult::ERROR("error".to_string())),
  case(NodeResult::ERROR("error".to_string()), NodeResult::ERROR("error2".to_string()), NodeResult::ERROR("error".to_string())),
)]
fn node_result_and(#[case] a: NodeResult, #[case] b: NodeResult, #[case] result: NodeResult) {
  expect!(a.and(&b)).to(be_equal_to(result));
}

#[test_log::test]
fn simple_match_request_test() -> anyhow::Result<()> {
  let request = HttpRequest {
    method: "put".to_string(),
    path: "/test".to_string(),
    body: OptionalBody::Present("Some nice bit of text".into(), Some(TEXT.clone()), None),
    .. Default::default()
  };
  let expected_request = HttpRequest {
    method: "POST".to_string(),
    path: "/test".to_string(),
    query: None,
    headers: None,
    body: OptionalBody::Present("Some nice bit of text".into(), Some(TEXT.clone()), None),
    .. Default::default()
  };
  let mut context = PlanMatchingContext::default();
  let plan = build_request_plan(&expected_request, &context)?;

  assert_eq!(r#"(
  :request (
    :method (
      #{'method == POST'},
      %match:equality (
        'POST',
        %upper-case (
          $.method
        ),
        NULL
      )
    ),
    :path (
      #{'path == \'/test\''},
      %match:equality (
        '/test',
        $.path,
        NULL
      )
    ),
    :"query parameters" (
      %expect:empty (
        $.query,
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      )
    ),
    :body (
      %if (
        %match:equality (
          'text/plain',
          $.content-type,
          NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ),
        %match:equality (
          'Some nice bit of text',
          %convert:UTF8 (
            $.body
          ),
          NULL
        )
      )
    )
  )
)
"#, plan.pretty_form());

  let executed_plan = execute_request_plan(&plan, &request, &mut context)?;
  assert_eq!(r#"(
  :request (
    :method (
      #{'method == POST'},
      %match:equality (
        'POST' => 'POST',
        %upper-case (
          $.method => 'put'
        ) => 'PUT',
        NULL => NULL
      ) => ERROR(Expected 'PUT' to be equal to 'POST')
    ) => BOOL(false),
    :path (
      #{'path == \'/test\''},
      %match:equality (
        '/test' => '/test',
        $.path => '/test',
        NULL => NULL
      ) => BOOL(true)
    ) => BOOL(true),
    :"query parameters" (
      %expect:empty (
        $.query => {},
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      ) => BOOL(true)
    ) => BOOL(true),
    :body (
      %if (
        %match:equality (
          'text/plain' => 'text/plain',
          $.content-type => 'text/plain',
          NULL => NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ) => BOOL(true),
        %match:equality (
          'Some nice bit of text' => 'Some nice bit of text',
          %convert:UTF8 (
            $.body => BYTES(21, U29tZSBuaWNlIGJpdCBvZiB0ZXh0)
          ) => 'Some nice bit of text',
          NULL => NULL
        ) => BOOL(true)
      ) => BOOL(true)
    ) => BOOL(true)
  ) => BOOL(false)
)
"#, executed_plan.pretty_form());

  assert_eq!(r#"request:
  method: method == POST - ERROR Expected 'PUT' to be equal to 'POST'
  path: path == '/test' - OK
  query parameters: - OK
  body: - OK
"#, executed_plan.generate_summary(false));

  let mismatches: RequestMatchResult = executed_plan.into();
  assert_eq!(RequestMatchResult {
    method: Some(MethodMismatch {
      expected: "".to_string(),
      actual: "".to_string(),
      mismatch: "Expected 'PUT' to be equal to 'POST'".to_string()
    }),
    path: None,
    headers: hashmap!{},
    query: hashmap!{},
    body: BodyMatchResult::Ok,
  }, mismatches);

  Ok(())
}

#[test_log::test]
fn simple_match_response_test() -> anyhow::Result<()> {
  let response = HttpResponse {
    status: 204,
    body: OptionalBody::Present("Some nice bit of text".into(), Some(TEXT.clone()), None),
    .. Default::default()
  };
  let expected_response = HttpResponse {
    status: 200,
    headers: None,
    body: OptionalBody::Present("Some nice bit of text".into(), Some(TEXT.clone()), None),
    .. Default::default()
  };
  let mut context = PlanMatchingContext::default();
  let plan = build_response_plan(&expected_response, &context)?;

  assert_eq!(r#"(
  :response (
    :status (
      #{'status == 200'},
      %match:equality (
        UINT(200),
        $.status,
        NULL
      )
    ),
    :body (
      %if (
        %match:equality (
          'text/plain',
          $.content-type,
          NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ),
        %match:equality (
          'Some nice bit of text',
          %convert:UTF8 (
            $.body
          ),
          NULL
        )
      )
    )
  )
)
"#, plan.pretty_form());

  let executed_plan = execute_response_plan(&plan, &response, &mut context)?;
  assert_eq!(r#"(
  :response (
    :status (
      #{'status == 200'},
      %match:equality (
        UINT(200) => UINT(200),
        $.status => UINT(204),
        NULL => NULL
      ) => ERROR(Expected 204 to be equal to 200)
    ) => BOOL(false),
    :body (
      %if (
        %match:equality (
          'text/plain' => 'text/plain',
          $.content-type => 'text/plain',
          NULL => NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ) => BOOL(true),
        %match:equality (
          'Some nice bit of text' => 'Some nice bit of text',
          %convert:UTF8 (
            $.body => BYTES(21, U29tZSBuaWNlIGJpdCBvZiB0ZXh0)
          ) => 'Some nice bit of text',
          NULL => NULL
        ) => BOOL(true)
      ) => BOOL(true)
    ) => BOOL(true)
  ) => BOOL(false)
)
"#, executed_plan.pretty_form());

  assert_eq!(r#"response:
  status: status == 200 - ERROR Expected 204 to be equal to 200
  body: - OK
"#, executed_plan.generate_summary(false));

  let mismatches: Vec<Mismatch> = executed_plan.into();
  assert_eq!(vec![Mismatch::StatusMismatch { expected: 0, actual: 0, mismatch: "".to_string() }], mismatches);

  Ok(())
}

#[test_log::test]
fn simple_json_match_request_test() -> anyhow::Result<()> {
  let request = HttpRequest {
    method: "POST".to_string(),
    path: "/test".to_string(),
    query: None,
    headers: None,
    body: OptionalBody::from(&json!({
        "b": "22"
      })),
    matching_rules: Default::default(),
    generators: Default::default(),
  };
  let expected_request = HttpRequest {
    method: "POST".to_string(),
    path: "/test".to_string(),
    query: None,
    headers: None,
    body: OptionalBody::from(&json!({
        "a": 100,
        "b": 200.1
      })),
    matching_rules: Default::default(),
    generators: Default::default(),
  };
  let mut context = PlanMatchingContext::default();
  let plan = build_request_plan(&expected_request, &context)?;

  assert_eq!(r#"(
  :request (
    :method (
      #{'method == POST'},
      %match:equality (
        'POST',
        %upper-case (
          $.method
        ),
        NULL
      )
    ),
    :path (
      #{'path == \'/test\''},
      %match:equality (
        '/test',
        $.path,
        NULL
      )
    ),
    :"query parameters" (
      %expect:empty (
        $.query,
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      )
    ),
    :body (
      %if (
        %match:equality (
          'application/json;charset=utf-8',
          $.content-type,
          NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ),
        %tee (
          %json:parse (
            $.body
          ),
          :$ (
            %json:expect:entries (
              'OBJECT',
              ['a', 'b'],
              ~>$
            ),
            %expect:only-entries (
              ['a', 'b'],
              ~>$
            ),
            :$.a (
              %match:equality (
                json:100,
                ~>$.a,
                NULL
              )
            ),
            :$.b (
              %match:equality (
                json:200.1,
                ~>$.b,
                NULL
              )
            )
          )
        )
      )
    )
  )
)
"#, plan.pretty_form());

  let executed_plan = execute_request_plan(&plan, &request, &mut context)?;
  assert_eq!(r#"(
  :request (
    :method (
      #{'method == POST'},
      %match:equality (
        'POST' => 'POST',
        %upper-case (
          $.method => 'POST'
        ) => 'POST',
        NULL => NULL
      ) => BOOL(true)
    ) => BOOL(true),
    :path (
      #{'path == \'/test\''},
      %match:equality (
        '/test' => '/test',
        $.path => '/test',
        NULL => NULL
      ) => BOOL(true)
    ) => BOOL(true),
    :"query parameters" (
      %expect:empty (
        $.query => {},
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      ) => BOOL(true)
    ) => BOOL(true),
    :body (
      %if (
        %match:equality (
          'application/json;charset=utf-8' => 'application/json;charset=utf-8',
          $.content-type => 'application/json;charset=utf-8',
          NULL => NULL,
          %error (
            'Body type error - ',
            %apply ()
          )
        ) => BOOL(true),
        %tee (
          %json:parse (
            $.body => BYTES(10, eyJiIjoiMjIifQ==)
          ) => json:{"b":"22"},
          :$ (
            %json:expect:entries (
              'OBJECT' => 'OBJECT',
              ['a', 'b'] => ['a', 'b'],
              ~>$ => json:{"b":"22"}
            ) => ERROR(The following expected entries were missing from the actual Object: a),
            %expect:only-entries (
              ['a', 'b'] => ['a', 'b'],
              ~>$ => json:{"b":"22"}
            ) => OK,
            :$.a (
              %match:equality (
                json:100 => json:100,
                ~>$.a => NULL,
                NULL => NULL
              ) => ERROR(Expected null (Null) to be equal to 100 (Integer))
            ) => BOOL(false),
            :$.b (
              %match:equality (
                json:200.1 => json:200.1,
                ~>$.b => json:"22",
                NULL => NULL
              ) => ERROR(Expected '22' (String) to be equal to 200.1 (Decimal))
            ) => BOOL(false)
          ) => BOOL(false)
        ) => BOOL(false)
      ) => BOOL(false)
    ) => BOOL(false)
  ) => BOOL(false)
)
"#, executed_plan.pretty_form());

  assert_eq!(r#"request:
  method: method == POST - OK
  path: path == '/test' - OK
  query parameters: - OK
  body:
    $: - ERROR The following expected entries were missing from the actual Object: a
      $.a: - ERROR Expected null (Null) to be equal to 100 (Integer)
      $.b: - ERROR Expected '22' (String) to be equal to 200.1 (Decimal)
"#, executed_plan.generate_summary(false));

  let mismatches: RequestMatchResult = executed_plan.into();
  assert_eq!(RequestMatchResult {
    method: None,
    path: None,
    headers: hashmap!{},
    query: hashmap!{},
    body: BodyMatchResult::BodyMismatches(hashmap!{
      "$.a".to_string() => vec![
        BodyMismatch {
          path: "$.a".to_string(),
          expected: None,
          actual: None,
          mismatch: "Expected null (Null) to be equal to 100 (Integer)".to_string()
        }
      ],
      "$.b".to_string() => vec![
        BodyMismatch {
          path: "$.b".to_string(),
          expected: None,
          actual: None,
          mismatch: "Expected '22' (String) to be equal to 200.1 (Decimal)".to_string()
        }
      ],
      "$".to_string() => vec![
        BodyMismatch {
          path: "$".to_string(),
          expected: None,
          actual: None,
          mismatch: "The following expected entries were missing from the actual Object: a".to_string()
        }
      ]
    })
  }, mismatches);

  Ok(())
}

#[test_log::test]
fn match_path_with_matching_rule() -> anyhow::Result<()> {
  let request = HttpRequest {
    method: "get".to_string(),
    path: "/test12345".to_string(),
    .. Default::default()
  };
  let matching_rules = matchingrules! {
    "path" => { "" => [ MatchingRule::Regex("\\/test[0-9]+".to_string()) ] }
  };
  let expected_request = HttpRequest {
    method: "get".to_string(),
    path: "/test".to_string(),
    matching_rules: matching_rules.clone(),
    .. Default::default()
  };
  let expected_interaction = SynchronousHttp {
    request: expected_request.clone(),
    .. SynchronousHttp::default()
  };
  let mut context = PlanMatchingContext {
    interaction: expected_interaction.boxed_v4(),
    .. PlanMatchingContext::default()
  };
  let plan = build_request_plan(&expected_request, &context)?;

  assert_eq!(r#"(
  :request (
    :method (
      #{'method == GET'},
      %match:equality (
        'GET',
        %upper-case (
          $.method
        ),
        NULL
      )
    ),
    :path (
      #{'path must match the regular expression /\\/test[0-9]+/'},
      %match:regex (
        '/test',
        $.path,
        json:{"regex":"\\/test[0-9]+"}
      )
    ),
    :"query parameters" (
      %expect:empty (
        $.query,
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      )
    )
  )
)
"#, plan.pretty_form());

  let executed_plan = execute_request_plan(&plan, &request, &mut context)?;
  assert_eq!(r#"(
  :request (
    :method (
      #{'method == GET'},
      %match:equality (
        'GET' => 'GET',
        %upper-case (
          $.method => 'get'
        ) => 'GET',
        NULL => NULL
      ) => BOOL(true)
    ) => BOOL(true),
    :path (
      #{'path must match the regular expression /\\/test[0-9]+/'},
      %match:regex (
        '/test' => '/test',
        $.path => '/test12345',
        json:{"regex":"\\/test[0-9]+"} => json:{"regex":"\\/test[0-9]+"}
      ) => BOOL(true)
    ) => BOOL(true),
    :"query parameters" (
      %expect:empty (
        $.query => {},
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      ) => BOOL(true)
    ) => BOOL(true)
  ) => BOOL(true)
)
"#, executed_plan.pretty_form());

  let request = HttpRequest {
    method: "get".to_string(),
    path: "/test12345X".to_string(),
    .. Default::default()
  };
  let executed_plan = execute_request_plan(&plan, &request, &mut context)?;
  assert_eq!(r#"(
  :request (
    :method (
      #{'method == GET'},
      %match:equality (
        'GET' => 'GET',
        %upper-case (
          $.method => 'get'
        ) => 'GET',
        NULL => NULL
      ) => BOOL(true)
    ) => BOOL(true),
    :path (
      #{'path must match the regular expression /\\/test[0-9]+/'},
      %match:regex (
        '/test' => '/test',
        $.path => '/test12345X',
        json:{"regex":"\\/test[0-9]+"} => json:{"regex":"\\/test[0-9]+"}
      ) => ERROR(Expected '/test12345X' to match '\/test[0-9]+')
    ) => BOOL(false),
    :"query parameters" (
      %expect:empty (
        $.query => {},
        %join (
          'Expected no query parameters but got ',
          $.query
        )
      ) => BOOL(true)
    ) => BOOL(true)
  ) => BOOL(false)
)
"#, executed_plan.pretty_form());

  Ok(())
}

#[test_log::test]
fn match_status_with_matching_rule() -> anyhow::Result<()> {
  let response = HttpResponse {
    status: 204,
    .. Default::default()
  };
  let matching_rules = matchingrules! {
    "status" => { "" => [ MatchingRule::StatusCode(HttpStatus::Success) ] }
  };
  let expected_response = HttpResponse {
    status: 200,
    matching_rules: matching_rules.clone(),
    .. Default::default()
  };
  let expected_interaction = SynchronousHttp {
    response: expected_response.clone(),
    .. SynchronousHttp::default()
  };
  let mut context = PlanMatchingContext {
    interaction: expected_interaction.boxed_v4(),
    .. PlanMatchingContext::default()
  };
  let plan = build_response_plan(&expected_response, &context)?;

  assert_eq!(r#"(
  :response (
    :status (
      #{'status must be a Success (20x) status'},
      %match:status-code (
        UINT(200),
        $.status,
        json:{"status":"success"}
      )
    )
  )
)
"#, plan.pretty_form());

  let executed_plan = execute_response_plan(&plan, &response, &mut context)?;
  assert_eq!(r#"(
  :response (
    :status (
      #{'status must be a Success (20x) status'},
      %match:status-code (
        UINT(200) => UINT(200),
        $.status => UINT(204),
        json:{"status":"success"} => json:{"status":"success"}
      ) => BOOL(true)
    ) => BOOL(true)
  ) => BOOL(true)
)
"#, executed_plan.pretty_form());

  let response = HttpResponse {
    status: 404,
    .. Default::default()
  };
  let executed_plan = execute_response_plan(&plan, &response, &mut context)?;
  assert_eq!(r#"(
  :response (
    :status (
      #{'status must be a Success (20x) status'},
      %match:status-code (
        UINT(200) => UINT(200),
        $.status => UINT(404),
        json:{"status":"success"} => json:{"status":"success"}
      ) => ERROR(Expected status code 404 to be a Successful response (200–299))
    ) => BOOL(false)
  ) => BOOL(false)
)
"#, executed_plan.pretty_form());

  Ok(())
}

#[test_log::test]
fn body_with_root_matcher() {
  let matching_rules = matchingrules! {
      "body" => { "$" => [ MatchingRule::Regex(".*[0-9]+.*".to_string()) ] }
    };
  let mut context = PlanMatchingContext::default();
  let response = HttpResponse {
    body: OptionalBody::from("This is a 100+ body"),
    matching_rules,
    .. Default::default()
  };
  let body_plan = setup_body_plan(&response, &context).unwrap();
  let mut buffer = String::new();
  body_plan.pretty_form(&mut buffer, 0);
  assert_eq!(r#":body (
  %match:regex (
    NULL,
    $.body,
    json:{"regex":".*[0-9]+.*"}
  )
)"#, buffer);

  let plan = body_plan.into();
  let executed_plan = execute_response_plan(&plan, &response, &mut context).unwrap();
  assert_eq!(r#"(
  :body (
    %match:regex (
      NULL => NULL,
      $.body => BYTES(19, VGhpcyBpcyBhIDEwMCsgYm9keQ==),
      json:{"regex":".*[0-9]+.*"} => json:{"regex":".*[0-9]+.*"}
    ) => BOOL(true)
  ) => BOOL(true)
)
"#, executed_plan.pretty_form());
}
