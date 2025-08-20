use std::fs::File;
use std::io::BufReader;

use anyhow::anyhow;
use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use maplit::hashmap;
use pact_models::interaction::Interaction;
use pact_models::matchingrules::matchers_from_json;
use pact_models::pact::Pact;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::synch_http::SynchronousHttp;
use serde_json::{json, Value};
use pact_matching::{match_request, match_response, Mismatch};

use crate::shared_steps::setup_body;
use crate::v4_steps::V4World;

#[given("an expected response configured with the following:")]
fn an_expected_response_configured_with_the_following(world: &mut V4World, step: &Step) {
  let mut expected_response = HttpResponse::default();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "status" => expected_response.status = value.parse().unwrap(),
          "body" => setup_body(value, &mut expected_response, None),
          "matching rules" => {
            let json: Value = if value.starts_with("JSON:") {
              serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
            } else {
              let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
                .expect(format!("could not load fixture '{}'", value).as_str());
              let reader = BufReader::new(f);
              serde_json::from_reader(reader).unwrap()
            };
            expected_response.matching_rules = matchers_from_json(&json!({
              "matchingRules": json
            }), &None)
              .expect("Matching rules fixture is not valid JSON");
          }
          _ => {}
        }
      }
    }
  }

  world.expected_response = expected_response;
}

#[given(expr = "a status {int} response is received")]
fn a_status_response_is_received(world: &mut V4World, status: u16) {
  world.received_responses.push(HttpResponse {
    status,
    .. HttpResponse::default()
  });
}

#[when("the response is compared to the expected one")]
async fn the_response_is_compared_to_the_expected_one(world: &mut V4World) -> anyhow::Result<()> {
  world.response_results.extend(match_response(world.expected_response.clone(),
    world.received_responses.first().unwrap().clone(), &world.pact.boxed(),
    &SynchronousHttp::default().boxed())
    .await?
  );
  Ok(())
}

#[then("the response comparison should be OK")]
fn the_response_comparison_should_be_ok(world: &mut V4World) -> anyhow::Result<()> {
  if world.response_results.is_empty() {
    Ok(())
  } else {
    Err(anyhow!("Comparison resulted in {} mismatches", world.response_results.len()))
  }
}

#[then("the response comparison should NOT be OK")]
fn the_response_comparison_should_not_be_ok(world: &mut V4World) -> anyhow::Result<()> {
  if !world.response_results.is_empty() {
    Ok(())
  } else {
    Err(anyhow!("Comparison resulted in no mismatches"))
  }
}

#[then(expr = "the response mismatches will contain a {string} mismatch with error {string}")]
fn the_response_mismatches_will_contain_a_mismatch_with_error(
  world: &mut V4World,
  mismatch_type: String,
  error: String
) -> anyhow::Result<()> {
  if world.response_results.iter().any(|m| {
    let correct_type = match m {
      Mismatch::BodyTypeMismatch { .. } => mismatch_type == "body-content-type",
      Mismatch::StatusMismatch { .. } => mismatch_type == "status",
      _ => m.mismatch_type().to_lowercase().starts_with(mismatch_type.as_str())
    };
    correct_type && m.description() == error
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find a {} error with message '{}'", mismatch_type, error))
  }
}

#[given(expr = "an expected request configured with the following:")]
fn an_expected_request_configured_with_the_following(world: &mut V4World, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    let mut data = hashmap!{};
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        data.insert(field.as_str(), value);
      }
    }

    if let Some(body) = data.get("body") {
      setup_body(body, &mut world.expected_request, data.get("content type").map(|ct| ct.as_str()));
    }

    if let Some(value) = data.get("matching rules") {
      let json: Value = if value.starts_with("JSON:") {
        serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
      } else {
        let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
          .expect(format!("could not load fixture '{}'", value).as_str());
        let reader = BufReader::new(f);
        serde_json::from_reader(reader).unwrap()
      };
      world.expected_request.matching_rules = matchers_from_json(&json!({
              "matchingRules": json
            }), &None)
        .expect("Matching rules fixture is not valid JSON");
    }
  }
}

#[given(expr = "a request is received with the following:")]
fn a_request_is_received_with_the_following(world: &mut V4World, step: &Step) {
  let mut request = HttpRequest::default();
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    let mut data = hashmap!{};
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        data.insert(field.as_str(), value);
      }
    }

    if let Some(body) = data.get("body") {
      setup_body(body, &mut request, data.get("content type").map(|ct| ct.as_str()));
    }
  }
  world.received_requests.push(request);
}

#[when("the request is compared to the expected one")]
async fn the_request_is_compared_to_the_expected_one(world: &mut V4World) {
  world.request_results.push(
    match_request(
      world.expected_request.clone(),
      world.received_requests.first().unwrap().clone(),
      &world.pact.boxed(), &SynchronousHttp::default().boxed()
    ).await.unwrap()
  );
}

#[then("the comparison should be OK")]
fn the_comparison_should_be_ok(world: &mut V4World) -> anyhow::Result<()> {
  if world.request_results.iter().all(|result| result.all_matched()) {
    Ok(())
  } else {
    let count = world.request_results.iter()
      .filter_map(|res| {
        let mismatches = res.mismatches();
        if mismatches.is_empty() {
          None
        } else {
          Some(mismatches)
        }
      })
      .flatten()
      .collect::<Vec<_>>();
    Err(anyhow!("There were match results with mismatches ({:?})", count))
  }
}

#[then("the comparison should NOT be OK")]
fn the_comparison_should_not_be_ok(world: &mut V4World) -> anyhow::Result<()> {
  if world.request_results.iter().all(|result| result.all_matched()) {
    Err(anyhow!("All requests matched"))
  } else {
    Ok(())
  }
}

#[then(expr = "the mismatches will contain a mismatch with error {string} -> {string}")]
fn the_mismatches_will_contain_a_mismatch_with_error(
  world: &mut V4World,
  error_path: String,
  error: String
) -> anyhow::Result<()> {
  if world.request_results.iter().flat_map(|result| result.mismatches())
    .any(|mismatch| {
      let path_matches = match &mismatch {
        Mismatch::QueryMismatch { parameter, .. } => parameter.as_str() == error_path,
        Mismatch::HeaderMismatch { key, .. } => key.as_str() == error_path,
        Mismatch::BodyMismatch { path, .. } => path.as_str() == error_path,
        Mismatch::MetadataMismatch { key, .. } => key.as_str() == error_path,
        _ => false
      };
      let error = error.replace("\\\"", "\"");
      let desc_matches = mismatch.description().contains(error.as_str());
      path_matches && desc_matches
    }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find a mismatch with the required error message"))
  }
}
