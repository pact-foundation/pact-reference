use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use bytes::Bytes;

use maplit::hashmap;
use serde_json::json;
use pretty_assertions::assert_eq;

use pact_models::pact::{Pact, ReadWritePact};
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::request::Request;
use pact_models::response::Response;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;

// Issue #246
#[test_log::test]
fn write_v4_and_read_v3_pact_test() {
  let body_json = json!([
    [
      1673222400000_i64,
      "0.00015856",
      "0.00018524",
      "0.00015744",
      "0.00016610",
      "344512338693.26000000",
      1673827199999_i64,
      "57919636.39781495",
      183337,
      "165369377629.69000000",
      "27805375.21572790",
      "0"
    ]
  ]);
  let body = Bytes::from(body_json.to_string());
  let pact = RequestResponsePact {
    consumer: Consumer {
      name: "write_v4_and_read_v3_pact_test_consumer".to_string(),
    },
    provider: Provider {
      name: "write_v4_and_read_v3_pact_test_provider".to_string(),
    },
    interactions: vec![
      RequestResponseInteraction {
        description: "get data".to_string(),
        request: Request {
          method: "GET".to_string(),
          path: "/api/v3/klines".to_string(),
          query: Some(hashmap!{
            "interval".to_string() => vec![ Some("1w".to_string()) ],
            "limit".to_string() => vec![ Some("1".to_string()) ],
            "symbol".to_string() => vec![ Some("LUNCUSDT".to_string()) ]
          }),
          .. Request::default()
        },
        response: Response {
          status: 200,
          headers: Some(hashmap!{
            "access-control-allow-methods".to_string() => vec!["GET".to_string(), "HEAD".to_string(), "OPTIONS".to_string()],
            "access-control-allow-origin".to_string() => vec!["*".to_string()],
            "cache-control".to_string() => vec!["no-cache".to_string(), "no-store".to_string(), "must-revalidate".to_string()],
            "content-length".to_string() => vec!["182".to_string()],
            "content-security-policy".to_string() => vec!["default-src 'self'".to_string()],
            "content-type".to_string() => vec!["application/json;charset=UTF-8".to_string()],
            "date".to_string() => vec!["Tue, 10 Jan 2023 06:51:20 GMT".to_string()],
            "expires".to_string() => vec!["0".to_string()],
            "pragma".to_string() => vec!["no-cache".to_string()],
            "server".to_string() => vec!["nginx".to_string()],
            "strict-transport-security".to_string() => vec!["max-age=31536000; includeSubdomains".to_string()],
            "via".to_string() => vec!["1.1 67ea8416eafbda87528a822ad116e5a4.cloudfront.net (CloudFront)".to_string()],
            "x-amz-cf-id".to_string() => vec!["-O1yry-11OB5-w2otAcAEhoL2tIdsF13vYZJtRlBRJkjGqwDgXIPPg==".to_string()],
            "x-amz-cf-pop".to_string() => vec!["MEL50-C2".to_string()],
            "x-cache".to_string() => vec!["Miss from cloudfront".to_string()],
            "x-content-security-policy".to_string() => vec!["default-src 'self'".to_string()],
            "x-content-type-options".to_string() => vec!["nosniff".to_string()],
            "x-frame-options".to_string() => vec!["SAMEORIGIN".to_string()],
            "x-mbx-used-weight".to_string() => vec!["1".to_string()],
            "x-mbx-used-weight-1m".to_string() => vec!["1".to_string()],
            "x-mbx-uuid".to_string() => vec!["f20848fe-bcfe-4ebb-9913-a267a01e706d".to_string()],
            "x-webkit-csp".to_string() => vec!["default-src 'self'".to_string()],
            "x-xss-protection".to_string() => vec!["1; mode=block".to_string()]
          }),
          body: OptionalBody::Present(body, Some(JSON.clone()), None),
          .. Response::default()
        },
        .. RequestResponseInteraction::default()
      }
    ],
    .. RequestResponsePact::default()
  };

  // save pact to file
  let pact_path = temp_dir().join("write_v4_and_read_v3_pact_test.json");
  let pact_json = pact.to_json(PactSpecification::V4).unwrap();
  let mut file = File::create(pact_path.clone()).unwrap();
  file.write_all(pact_json.to_string().as_bytes()).unwrap();

  // read pact from file
  let pact_from_file = RequestResponsePact::read_pact(&pact_path).unwrap();

  assert_eq!(pact, pact_from_file);
}
