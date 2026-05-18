use iai::{black_box, main};
use pact_matching::engine::{build_request_plan, execute_request_plan};
use pact_matching::engine::context::PlanMatchingContext;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::v4::http_parts::HttpRequest;

fn iai_benchmark_simple() {
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
  let plan = build_request_plan(&expected_request, &context).unwrap();
  let _executed_plan = execute_request_plan(&plan, &request, &mut context).unwrap();
}

iai::main!(iai_benchmark_simple);
