// Profiling target for the V2 matching engine.
// Run with: cargo flamegraph --package pact_matching --example profile_engine
//
// Exercises four representative scenarios in a tight loop so flamegraph gets
// enough samples to show where time is actually spent:
//
//   1. Simple text body request (baseline — minimal plan)
//   2. JSON body request with equality matching
//   3. JSON body request with type + regex matching rules
//   4. Request with headers and query parameters

use pact_matching::engine::{build_request_plan, execute_request_plan};
use pact_matching::engine::context::PlanMatchingContext;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::matchingrules;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::synch_http::SynchronousHttp;
use pact_models::matchingrules::MatchingRule;
use serde_json::json;

const ITERATIONS: usize = 10_000;

fn scenario_text_body(expected: &HttpRequest, actual: &HttpRequest) {
    let mut context = PlanMatchingContext::default();
    let plan = build_request_plan(expected, &context).unwrap();
    let _ = execute_request_plan(&plan, actual, &mut context).unwrap();
}

fn scenario_json_equality(expected: &HttpRequest, actual: &HttpRequest) {
    let mut context = PlanMatchingContext::default();
    let plan = build_request_plan(expected, &context).unwrap();
    let _ = execute_request_plan(&plan, actual, &mut context).unwrap();
}

fn scenario_json_with_rules(expected: &HttpRequest, actual: &HttpRequest, context: &mut PlanMatchingContext) {
    let plan = build_request_plan(expected, context).unwrap();
    let _ = execute_request_plan(&plan, actual, context).unwrap();
}

fn scenario_headers_and_query(expected: &HttpRequest, actual: &HttpRequest) {
    let mut context = PlanMatchingContext::default();
    let plan = build_request_plan(expected, &context).unwrap();
    let _ = execute_request_plan(&plan, actual, &mut context).unwrap();
}

fn main() {
    // --- Scenario 1: simple text body ---
    let text_expected = HttpRequest {
        method: "POST".to_string(),
        path: "/api/things".to_string(),
        body: OptionalBody::Present("hello world".into(), Some(TEXT.clone()), None),
        ..Default::default()
    };
    let text_actual = HttpRequest {
        method: "POST".to_string(),
        path: "/api/things".to_string(),
        body: OptionalBody::Present("hello world".into(), Some(TEXT.clone()), None),
        ..Default::default()
    };

    // --- Scenario 2: JSON equality ---
    let json_expected = HttpRequest {
        method: "POST".to_string(),
        path: "/api/orders".to_string(),
        body: OptionalBody::from(&json!({
            "id": 42,
            "name": "widget",
            "price": 9.99,
            "tags": ["sale", "new"],
            "address": {
                "street": "1 Main St",
                "city": "Springfield",
                "zip": "12345"
            }
        })),
        ..Default::default()
    };
    let json_actual = HttpRequest {
        method: "POST".to_string(),
        path: "/api/orders".to_string(),
        body: OptionalBody::from(&json!({
            "id": 42,
            "name": "widget",
            "price": 9.99,
            "tags": ["sale", "new"],
            "address": {
                "street": "1 Main St",
                "city": "Springfield",
                "zip": "12345"
            }
        })),
        ..Default::default()
    };

    // --- Scenario 3: JSON with type + regex matching rules ---
    let matching_rules = matchingrules! {
        "body" => {
            "$.id"    => [ MatchingRule::Integer ],
            "$.name"  => [ MatchingRule::Type ],
            "$.price" => [ MatchingRule::Decimal ],
            "$.zip"   => [ MatchingRule::Regex("[0-9]{5}".to_string()) ]
        }
    };
    let json_rules_expected = HttpRequest {
        method: "POST".to_string(),
        path: "/api/orders".to_string(),
        body: OptionalBody::from(&json!({
            "id": 1,
            "name": "example",
            "price": 1.0,
            "zip": "00000"
        })),
        matching_rules: matching_rules.clone(),
        ..Default::default()
    };
    let json_rules_actual = HttpRequest {
        method: "POST".to_string(),
        path: "/api/orders".to_string(),
        body: OptionalBody::from(&json!({
            "id": 99,
            "name": "sprocket",
            "price": 14.50,
            "zip": "90210"
        })),
        ..Default::default()
    };
    let interaction = SynchronousHttp {
        request: json_rules_expected.clone(),
        ..SynchronousHttp::default()
    };
    let mut rules_context = PlanMatchingContext {
        interaction: interaction.boxed_v4(),
        ..PlanMatchingContext::default()
    };

    // --- Scenario 4: headers + query parameters ---
    let headers_expected = HttpRequest {
        method: "GET".to_string(),
        path: "/api/search".to_string(),
        query: Some(vec![
            ("q".to_string(), vec![Some("rust".to_string())]),
            ("page".to_string(), vec![Some("1".to_string())]),
            ("limit".to_string(), vec![Some("20".to_string())]),
        ].into_iter().collect()),
        headers: Some(vec![
            ("Accept".to_string(), vec!["application/json".to_string()]),
            ("Authorization".to_string(), vec!["Bearer token123".to_string()]),
            ("X-Request-Id".to_string(), vec!["abc-123".to_string()]),
        ].into_iter().collect()),
        ..Default::default()
    };
    let headers_actual = headers_expected.clone();

    // Run all scenarios in a tight loop so perf/flamegraph gets enough samples.
    for _ in 0..ITERATIONS {
        scenario_text_body(&text_expected, &text_actual);
        scenario_json_equality(&json_expected, &json_actual);
        scenario_json_with_rules(&json_rules_expected, &json_rules_actual, &mut rules_context);
        scenario_headers_and_query(&headers_expected, &headers_actual);
    }

    println!("Done: {} iterations x 4 scenarios", ITERATIONS);
}
