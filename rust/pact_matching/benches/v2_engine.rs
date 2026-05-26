use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pact_matching::engine::{build_request_plan, execute_request_plan};
use pact_matching::engine::context::PlanMatchingContext;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRule;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::synch_http::SynchronousHttp;
use serde_json::json;

// --- Scenario fixtures ---

fn text_requests() -> (HttpRequest, HttpRequest) {
    let expected = HttpRequest {
        method: "POST".to_string(),
        path: "/api/things".to_string(),
        body: OptionalBody::Present("hello world".into(), Some(TEXT.clone()), None),
        ..Default::default()
    };
    let actual = expected.clone();
    (expected, actual)
}

fn json_equality_requests() -> (HttpRequest, HttpRequest) {
    let body = json!({
        "id": 42,
        "name": "widget",
        "price": 9.99,
        "tags": ["sale", "new"],
        "address": {
            "street": "1 Main St",
            "city": "Springfield",
            "zip": "12345"
        }
    });
    let expected = HttpRequest {
        method: "POST".to_string(),
        path: "/api/orders".to_string(),
        body: OptionalBody::from(&body),
        ..Default::default()
    };
    let actual = expected.clone();
    (expected, actual)
}

fn json_rules_requests() -> (HttpRequest, HttpRequest, PlanMatchingContext) {
    let matching_rules = matchingrules! {
        "body" => {
            "$.id"    => [ MatchingRule::Integer ],
            "$.name"  => [ MatchingRule::Type ],
            "$.price" => [ MatchingRule::Decimal ],
            "$.zip"   => [ MatchingRule::Regex("[0-9]{5}".to_string()) ]
        }
    };
    let expected = HttpRequest {
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
    let actual = HttpRequest {
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
    let context = PlanMatchingContext {
        interaction: SynchronousHttp {
            request: expected.clone(),
            ..SynchronousHttp::default()
        }.boxed_v4(),
        ..PlanMatchingContext::default()
    };
    (expected, actual, context)
}

fn headers_query_requests() -> (HttpRequest, HttpRequest) {
    let expected = HttpRequest {
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
    let actual = expected.clone();
    (expected, actual)
}

// --- Benchmarks ---

fn bench_text_body(c: &mut Criterion) {
    let (expected, actual) = text_requests();
    let mut group = c.benchmark_group("text_body");

    group.bench_function("build", |b| {
        b.iter(|| {
            let context = PlanMatchingContext::default();
            black_box(build_request_plan(black_box(&expected), &context).unwrap())
        })
    });

    group.bench_function("execute", |b| {
        let context = PlanMatchingContext::default();
        let plan = build_request_plan(&expected, &context).unwrap();
        b.iter(|| {
            let mut ctx = context.clone();
            black_box(execute_request_plan(black_box(&plan), black_box(&actual), &mut ctx).unwrap())
        })
    });

    group.bench_function("build+execute", |b| {
        b.iter(|| {
            let mut context = PlanMatchingContext::default();
            let plan = build_request_plan(black_box(&expected), &context).unwrap();
            black_box(execute_request_plan(&plan, black_box(&actual), &mut context).unwrap())
        })
    });

    group.finish();
}

fn bench_json_equality(c: &mut Criterion) {
    let (expected, actual) = json_equality_requests();
    let mut group = c.benchmark_group("json_equality");

    group.bench_function("build", |b| {
        b.iter(|| {
            let context = PlanMatchingContext::default();
            black_box(build_request_plan(black_box(&expected), &context).unwrap())
        })
    });

    group.bench_function("execute", |b| {
        let context = PlanMatchingContext::default();
        let plan = build_request_plan(&expected, &context).unwrap();
        b.iter(|| {
            let mut ctx = context.clone();
            black_box(execute_request_plan(black_box(&plan), black_box(&actual), &mut ctx).unwrap())
        })
    });

    group.bench_function("build+execute", |b| {
        b.iter(|| {
            let mut context = PlanMatchingContext::default();
            let plan = build_request_plan(black_box(&expected), &context).unwrap();
            black_box(execute_request_plan(&plan, black_box(&actual), &mut context).unwrap())
        })
    });

    group.finish();
}

fn bench_json_with_rules(c: &mut Criterion) {
    let (expected, actual, base_context) = json_rules_requests();
    let mut group = c.benchmark_group("json_with_rules");

    group.bench_function("build", |b| {
        b.iter(|| {
            let context = base_context.clone();
            black_box(build_request_plan(black_box(&expected), &context).unwrap())
        })
    });

    group.bench_function("execute", |b| {
        let plan = build_request_plan(&expected, &base_context).unwrap();
        b.iter(|| {
            let mut ctx = base_context.clone();
            black_box(execute_request_plan(black_box(&plan), black_box(&actual), &mut ctx).unwrap())
        })
    });

    group.bench_function("build+execute", |b| {
        b.iter(|| {
            let mut context = base_context.clone();
            let plan = build_request_plan(black_box(&expected), &context).unwrap();
            black_box(execute_request_plan(&plan, black_box(&actual), &mut context).unwrap())
        })
    });

    group.finish();
}

fn bench_headers_and_query(c: &mut Criterion) {
    let (expected, actual) = headers_query_requests();
    let mut group = c.benchmark_group("headers_and_query");

    group.bench_function("build", |b| {
        b.iter(|| {
            let context = PlanMatchingContext::default();
            black_box(build_request_plan(black_box(&expected), &context).unwrap())
        })
    });

    group.bench_function("execute", |b| {
        let context = PlanMatchingContext::default();
        let plan = build_request_plan(&expected, &context).unwrap();
        b.iter(|| {
            let mut ctx = context.clone();
            black_box(execute_request_plan(black_box(&plan), black_box(&actual), &mut ctx).unwrap())
        })
    });

    group.bench_function("build+execute", |b| {
        b.iter(|| {
            let mut context = PlanMatchingContext::default();
            let plan = build_request_plan(black_box(&expected), &context).unwrap();
            black_box(execute_request_plan(&plan, black_box(&actual), &mut context).unwrap())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_text_body,
    bench_json_equality,
    bench_json_with_rules,
    bench_headers_and_query,
);
criterion_main!(benches);
