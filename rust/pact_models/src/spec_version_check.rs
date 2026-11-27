//! Checks that a serialised pact only uses the matching rules and generators its
//! specification version defines.
//!
//! The model is version-agnostic, so it is possible to build an interaction with V4-only
//! matchers (`eachKey`, `arrayContains`, ...) and then write it out as a V3 pact, or V3
//! matchers into a V2 pact. Consumers reading those files against the spec (or a schema)
//! will reject them, and older verifiers will silently ignore the rules. This check is run
//! on the JSON that is about to be written, so it sees exactly what lands on disk.

use std::env;

use serde_json::Value;

use crate::PactSpecification;

/// Environment variable that turns spec violations into errors instead of warnings.
pub const STRICT_SPEC_ENV_VAR: &str = "PACT_STRICT_SPEC_VERSION";

/// A matching rule or generator that is not defined by the pact specification being written
#[derive(Debug, Clone, PartialEq)]
pub struct SpecViolation {
  /// JSON pointer to the offending rule or generator
  pub path: String,
  /// The `match` or `type` value
  pub name: String,
  /// If it is a matching rule (true) or a generator (false)
  pub matcher: bool,
  /// The specification version that would be required for it
  pub required: PactSpecification
}

impl std::fmt::Display for SpecViolation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} '{}' at {} requires a {} pact", if self.matcher { "matcher" } else { "generator" },
      self.name, self.path, self.required)
  }
}

/// If strict spec checking has been enabled via the `PACT_STRICT_SPEC_VERSION` environment variable
pub fn strict_spec_version() -> bool {
  env::var(STRICT_SPEC_ENV_VAR)
    .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
    .unwrap_or(false)
}

/// The lowest specification version that defines the given `match` type.
/// Unknown types are treated as plugin matchers, which need V4.
pub fn matcher_spec_version(name: &str) -> PactSpecification {
  match name {
    "regex" | "type" => PactSpecification::V2,
    "equality" | "include" | "integer" | "decimal" | "number" | "null" | "boolean" | "date"
    | "time" | "datetime" | "timestamp" | "values" | "contentType" => PactSpecification::V3,
    _ => PactSpecification::V4
  }
}

/// The lowest specification version that defines the given generator type.
/// Unknown types are treated as plugin generators, which need V4.
pub fn generator_spec_version(name: &str) -> PactSpecification {
  match name {
    "RandomInt" | "RandomDecimal" | "RandomHexadecimal" | "RandomString" | "Regex" | "Uuid"
    | "Date" | "Time" | "DateTime" | "RandomBoolean" => PactSpecification::V3,
    _ => PactSpecification::V4
  }
}

/// Returns all the matching rules and generators in the serialised pact that the given
/// specification version does not define. V4 (and unknown) pacts are never reported.
pub fn spec_violations(pact_json: &Value, spec: PactSpecification) -> Vec<SpecViolation> {
  let mut violations = vec![];
  if matches!(spec, PactSpecification::V2 | PactSpecification::V3) {
    walk(pact_json, "", spec, &mut violations);
  }
  violations
}

fn walk(json: &Value, pointer: &str, spec: PactSpecification, violations: &mut Vec<SpecViolation>) {
  match json {
    Value::Object(map) => for (key, value) in map {
      let path = format!("{}/{}", pointer, escape(key));
      match key.as_str() {
        "matchingRules" => collect(value, &path, "match", true, spec, violations),
        "generators" if spec == PactSpecification::V2 => if !value.is_null() {
          violations.push(SpecViolation {
            path, name: "generators".to_string(), matcher: false, required: PactSpecification::V3
          })
        },
        "generators" => collect(value, &path, "type", false, spec, violations),
        _ => walk(value, &path, spec, violations)
      }
    },
    Value::Array(items) => for (i, item) in items.iter().enumerate() {
      walk(item, &format!("{}/{}", pointer, i), spec, violations);
    },
    _ => {}
  }
}

// Finds the rule/generator objects (those with a `match`/`type` string) below a
// matchingRules or generators block. Does not descend into a found rule: anything nested
// in it (eachKey rules, arrayContains variants) is only reachable through a V4 matcher,
// which is already reported.
fn collect(
  json: &Value,
  pointer: &str,
  field: &str,
  matcher: bool,
  spec: PactSpecification,
  violations: &mut Vec<SpecViolation>
) {
  match json {
    Value::Object(map) => if let Some(Value::String(name)) = map.get(field) {
      let required = if matcher { matcher_spec_version(name) } else { generator_spec_version(name) };
      if required > spec {
        violations.push(SpecViolation { path: pointer.to_string(), name: name.clone(), matcher, required });
      }
    } else {
      for (key, value) in map {
        collect(value, &format!("{}/{}", pointer, escape(key)), field, matcher, spec, violations);
      }
    },
    Value::Array(items) => for (i, item) in items.iter().enumerate() {
      collect(item, &format!("{}/{}", pointer, i), field, matcher, spec, violations);
    },
    _ => {}
  }
}

fn escape(key: &str) -> String {
  key.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use itertools::Itertools;
  use serde_json::json;

  use super::*;

  #[test]
  fn v3_pact_with_v4_matchers_and_generators() {
    let pact = json!({
      "interactions": [{
        "response": {
          "matchingRules": {
            "body": {
              "$": { "combine": "AND", "matchers": [{ "match": "eachKey", "rules": [{ "match": "regex", "regex": "\\w+" }] }] },
              "$.id": { "combine": "AND", "matchers": [{ "match": "integer" }] }
            },
            "status": { "combine": "AND", "matchers": [{ "match": "statusCode", "status": "success" }] }
          },
          "generators": {
            "body": { "$.url": { "type": "MockServerURL", "example": "http://x", "regex": ".*" }, "$.id": { "type": "RandomInt" } }
          }
        }
      }]
    });
    let violations = spec_violations(&pact, PactSpecification::V3);
    let names: Vec<_> = violations.iter().map(|v| v.name.as_str()).sorted().collect();
    expect!(names).to(be_equal_to(vec!["MockServerURL", "eachKey", "statusCode"]));
    let each_key = violations.iter().find(|v| v.name == "eachKey").unwrap();
    expect!(each_key.to_string()).to(be_equal_to(
      "matcher 'eachKey' at /interactions/0/response/matchingRules/body/$/matchers/0 requires a V4 pact"));
  }

  #[test]
  fn v2_pact_with_v3_matchers_and_generators() {
    let pact = json!({
      "interactions": [{
        "response": {
          "matchingRules": { "$.body.id": { "match": "integer" }, "$.body.name": { "match": "type" } },
          "generators": { "body": { "$.id": { "type": "RandomInt" } } }
        }
      }]
    });
    let violations = spec_violations(&pact, PactSpecification::V2);
    let names: Vec<_> = violations.iter().map(|v| v.name.as_str()).sorted().collect();
    expect!(names).to(be_equal_to(vec!["generators", "integer"]));
  }

  #[test]
  fn v4_pacts_are_not_checked() {
    let pact = json!({ "interactions": [{ "response": { "matchingRules": { "body": {
      "$": { "combine": "AND", "matchers": [{ "match": "some-plugin-matcher" }] }
    } } } }] });
    expect!(spec_violations(&pact, PactSpecification::V4).iter()).to(be_empty());
  }

  #[test]
  fn v2_matching_rules_are_downgraded_where_nothing_is_lost() {
    use crate::matchingrules;
    use crate::matchingrules::{matchers_to_json, MatchingRule};

    let rules = matchingrules! {
      "body" => {
        "$.name" => [ MatchingRule::Equality ],
        "$.desc" => [ MatchingRule::Include("a.b".to_string()) ],
        "$.items" => [ MatchingRule::MinType(1) ],
        "$.items[*].id" => [ MatchingRule::Equality ]
      }
    };
    let json = matchers_to_json(&rules, &PactSpecification::V2);
    expect!(json.clone()).to(be_equal_to(json!({
      "$.body.desc": { "match": "regex", "regex": "(?s).*a\\.b.*" },
      "$.body.items": { "match": "type", "min": 1 },
      // under a type matcher, so dropping it would change what is matched: kept, and reported
      "$.body.items[*].id": { "match": "equality" }
    })));
    let violations = spec_violations(&json!({ "matchingRules": json }), PactSpecification::V2);
    expect!(violations.iter().map(|v| v.name.as_str()).collect::<Vec<_>>()).to(be_equal_to(vec!["equality"]));
  }

  #[test]
  fn v2_requests_and_responses_do_not_write_generators() {
    use crate::generators;
    use crate::generators::Generator;
    use crate::request::Request;

    let request = Request {
      generators: generators! { "BODY" => { "$.id" => Generator::RandomInt(1, 10) } },
      .. Request::default()
    };
    expect!(request.to_json(&PactSpecification::V2).get("generators").cloned()).to(be_none());
    expect!(request.to_json(&PactSpecification::V3).get("generators").cloned()).to(be_some());
  }
}
