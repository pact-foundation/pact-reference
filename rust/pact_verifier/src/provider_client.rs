//! Module for building HTTP requests required by the verifier. Note that this is an internal
//! module made public for testing purposes, and there are no guarantees that the structures
//! and functions won't change.

use std::collections::hash_map::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use anyhow::anyhow;
use futures::future::*;
use http::{HeaderMap, HeaderValue, Method};
use http::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use http::header::CONTENT_TYPE;
use http::method::InvalidMethod;
use itertools::Itertools;
use maplit::hashmap;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::headers::parse_header;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use reqwest::{Client, Error, RequestBuilder};
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::{ProviderInfo, ProviderTransport, RequestFilterExecutor, VerificationOptions};
use crate::utils::with_retries;

#[derive(Debug)]
#[allow(dead_code)]
/// Error returned by the provider client
pub enum ProviderClientError {
    /// Invalid HTTP method
    RequestMethodError(String, InvalidMethod),
    /// Inavlid header name
    RequestHeaderNameError(String, InvalidHeaderName),
    /// Invalid header value
    RequestHeaderValueError(String, InvalidHeaderValue),
    /// Error ocurred with the request body
    RequestBodyError(String),
    /// Error dealing with the response
    ResponseError(String),
    /// Response status was an error
    ResponseStatusCodeError(u16),
}

impl From<reqwest::Error> for ProviderClientError {
  fn from(err: Error) -> Self {
    ProviderClientError::ResponseError(err.to_string())
  }
}

impl Display for ProviderClientError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      ProviderClientError::RequestMethodError(ref method, _) =>
        write!(f, "Invalid request method: '{}'", method),
      ProviderClientError::RequestHeaderNameError(ref name, _) =>
        write!(f, "Invalid header name: '{}'", name),
      ProviderClientError::RequestHeaderValueError(ref value, _) =>
        write!(f, "Invalid header value: '{}'", value),
      ProviderClientError::RequestBodyError(ref message) =>
        write!(f, "Invalid request body: '{}'", message),
      ProviderClientError::ResponseError(ref message) =>
        write!(f, "Invalid response: {}", message),
      ProviderClientError::ResponseStatusCodeError(ref code) =>
        write!(f, "Invalid status code: {}", code)
    }
  }
}

impl std::error::Error for ProviderClientError {}

/// Joins a path to a base path, taking into account trailing slashes
pub fn join_paths(base: &str, path: &str) -> String {
  if !path.is_empty() && path != "/" {
    let mut full_path = base.trim_end_matches('/').to_string();
    full_path.push('/');
    full_path.push_str(path.trim_start_matches('/'));
    full_path
  } else if !base.is_empty() && base != "/" {
    base.trim_end_matches('/').to_string()
  } else {
    "/".to_string()
  }
}

fn create_native_request(
  client: &Client,
  base_url: &str,
  request: &HttpRequest,
  custom_headers: &HashMap<String, String>
) -> Result<RequestBuilder, ProviderClientError> {
  let url = join_paths(base_url, &request.path.clone());
  let mut builder = client.request(Method::from_bytes(
    &request.method.clone().into_bytes()).unwrap_or(Method::GET), &url);

  if let Some(query) = &request.query {
    builder = builder.query(&query.iter()
      .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
      .flat_map(|(k, v)| {
        v.iter().map(|v| (k, v)).collect_vec()
      }).collect_vec());
  }

  if let Some(headers) = &request.headers {
    let mut header_map = HeaderMap::new();
    for (k, vals) in headers {
      if !custom_headers.contains_key(k) {
        for header_value in vals {
          let header_name = HeaderName::try_from(k)
            .map_err(|err| ProviderClientError::RequestHeaderNameError(
              format!("Failed to parse header value: {}", header_value), err))?;
          header_map.append(header_name, HeaderValue::from_str(header_value.as_str())
            .map_err(|err| ProviderClientError::RequestHeaderValueError(
              format!("Failed to parse header value: {}", header_value), err))?);
        }
      }
    }
    if !header_map.is_empty() {
      builder = builder.headers(header_map);
    }
  }

  match request.body {
    OptionalBody::Present(ref s, _, _) => builder = builder.body(s.clone()),
    OptionalBody::Null => {
      if request.content_type().unwrap_or_default().is_json() {
        builder = builder.body("null");
      }
    },
    _ => ()
  };

  Ok(builder)
}

fn extract_headers(headers: &HeaderMap) -> Option<HashMap<String, Vec<String>>> {
  if !headers.is_empty() {
    let result = headers.keys()
      .map(|name| {
        let values = headers.get_all(name);
        let parsed_vals: Vec<Result<String, ()>> = values.iter()
          .map(|val| val.to_str()
            .map(|v| v.to_string())
            .map_err(|err| warn!("Failed to parse HTTP header value: {}", err))
          ).collect();
        let name = name.as_str();
        (name.into(), parsed_vals.iter().cloned()
            .filter(|val| val.is_ok())
            .map(|val| val.unwrap_or_default())
            .flat_map(|val| parse_header(name, val.as_str()))
            .map(|val| val.trim().to_string())
            .collect())
      })
      .collect();

    Some(result)
  } else {
    None
  }
}

async fn extract_body(response: reqwest::Response, pact_response: &HttpResponse) -> anyhow::Result<OptionalBody> {
  let body = response.bytes().await?;
  if !body.is_empty() {
    Ok(OptionalBody::Present(body, pact_response.content_type(), None))
  } else {
    Ok(OptionalBody::Empty)
  }
}

async fn native_response_to_pact_response(native_response: reqwest::Response) -> anyhow::Result<HttpResponse> {
  debug!("Received native response: {:?}", native_response);

  let status = native_response.status().as_u16();
  let headers = extract_headers(native_response.headers());
  let response = HttpResponse {
    status,
    headers,
    .. HttpResponse::default()
  };

  let body = extract_body(native_response, &response).await?;

  Ok(HttpResponse {
    body, .. response.clone()
  })
}

/// This function makes the actual request to the provider, executing any request filter before
/// executing the request
pub async fn make_provider_request<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  request: &HttpRequest,
  options: &VerificationOptions<F>,
  client: &Client,
  transport: Option<ProviderTransport>
) -> anyhow::Result<HttpResponse> {
  let request_filter_option = options.request_filter.clone();
  let request = if request_filter_option.is_some() {
    let request_filter = request_filter_option.unwrap();
    info!("Invoking request filter for request");
    request_filter.call(request)
  } else {
    request.clone()
  };

  #[allow(deprecated)]
  let base_url = transport
    .map(|trans| trans.base_url(&provider.host))
    .unwrap_or_else(|| {
      match provider.port {
        Some(port) => format!("{}://{}:{}{}", provider.protocol, provider.host, port, provider.path),
        None => format!("{}://{}{}", provider.protocol, provider.host, provider.path),
      }
    });

  info!("Sending request to provider at {base_url}");
  debug!("Provider details = {provider:?}");
  info!("Sending request {request}");
  debug!("body:\n{}", request.body.display_string());
  let request = create_native_request(client, &base_url, &request, &options.custom_headers)?;

  let response = request.send()
    .map_err(|err| anyhow!(err))
    .and_then(native_response_to_pact_response)
    .await?;

  info!("Received response: {}", response);
  debug!("body:\n{}", response.body.display_string());

  Ok(response)
}

/// Make a state change request. If the response returns a JSON body, convert that into a HashMap
/// and return it. The request will be retried on 50x errors to a maximum of the `retries` parameter.
pub async fn make_state_change_request(
  client: &reqwest::Client,
  state_change_url: &str,
  request: &HttpRequest,
  retries: u8
) -> anyhow::Result<HashMap<String, Value>> {
  debug!("Sending {} to state change handler", request);

  let request = create_native_request(client, state_change_url, request, &hashmap!{})?;
  let result = with_retries(retries, request).await;

  match result {
    Ok(response) => {
      debug!("State change request: {:?}", response);
      if response.status().is_success() {
        if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
          if let Ok(content_type) = ContentType::parse(content_type.to_str().unwrap_or_default()) {
            if content_type.is_json() {
              let body = response.bytes().await?;
              match serde_json::from_slice::<Value>(&body) {
                Ok(body) => match body {
                  Value::Object(map) => Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                  _ => Ok(hashmap!{})
                },
                Err(_) => Ok(hashmap!{})
              }
            } else {
              Ok(hashmap!{})
            }
          } else {
            Ok(hashmap!{})
          }
        } else {
          Ok(hashmap!{})
        }
      } else {
        Err(ProviderClientError::ResponseStatusCodeError(response.status().as_u16()).into())
      }
    },
    Err(err) => {
      debug!("State change request failed with error {}", err);
      Err(ProviderClientError::ResponseError(err.to_string()).into())
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use http::HeaderMap;
  use itertools::Itertools;
  use maplit::*;
  use pact_models::bodies::OptionalBody;
  use pact_models::v4::http_parts::HttpRequest;

  use pact_consumer::builders::{HttpPartBuilder, PactBuilderAsync};
  use pact_consumer::mock_server::StartMockServer;

  use crate::{
    configure_http_client,
    NullRequestFilterExecutor,
    ProviderInfo,
    VerificationOptions
  };

  use super::{create_native_request, extract_headers, join_paths};

  #[test]
  fn extract_headers_tests() {
    let mut headers = HeaderMap::new();
    headers.insert("HOST", "example.com".parse().unwrap());
    headers.insert("CONTENT_LENGTH", "123".parse().unwrap());
    let response = extract_headers(&headers).unwrap();
    expect!(&response["host"][0]).to(be_equal_to(&"example.com"));
    expect!(&response["content_length"][0]).to(be_equal_to(&"123"));
  }

  #[test]
  fn extract_headers_return_none_if_headers_are_empty() {
    expect!(extract_headers(&HeaderMap::new())).to(be_none());
  }

  #[test]
  fn extract_headers_when_header_value_is_a_comma_separated_string() {
    let mut headers = HeaderMap::new();
    headers.insert(
      "Access-Control-Expose-Headers",
      "Content-Length, Content-Type, Expires".parse().unwrap()
    );
    let response = extract_headers(&headers).unwrap();
    expect!(&response["access-control-expose-headers"][0]).to(be_equal_to(&"Content-Length"));
    expect!(&response["access-control-expose-headers"][1]).to(be_equal_to(&"Content-Type"));
    expect!(&response["access-control-expose-headers"][2]).to(be_equal_to(&"Expires"));
  }

  #[test]
  fn extract_headers_when_header_is_a_type_that_should_not_be_split() {
    let mut headers = HeaderMap::new();
    headers.insert("Last-Modified", "Sun, 12 Mar 2023 01:21:35 GMT".parse().unwrap());
    let response = extract_headers(&headers).unwrap();
    expect!(response["last-modified"].len()).to(be_equal_to(1));
    expect!(response["last-modified"][0].as_str()).to(be_equal_to("Sun, 12 Mar 2023 01:21:35 GMT"));
  }

  #[test]
  fn join_paths_test() {
    expect!(join_paths("", "")).to(be_equal_to("/"));
    expect!(join_paths("/", "")).to(be_equal_to("/"));
    expect!(join_paths("", "/")).to(be_equal_to("/"));
    expect!(join_paths("/", "/")).to(be_equal_to("/"));
    expect!(join_paths("/base", "/")).to(be_equal_to("/base"));
    expect!(join_paths("/a/b", "/c/d")).to(be_equal_to("/a/b/c/d"));
  }

  #[test]
  fn convert_request_to_native_request_test() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest::default();
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body()).to(be_none());
  }

  #[test]
  fn convert_request_to_native_request_with_query_parameters() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      query: Some(hashmap!{
        "a".to_string() => vec![Some("b".to_string())],
        "c".to_string() => vec![Some("d".to_string()), Some("e".to_string())]
      }),
      .. HttpRequest::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/?a=b&c=d&c=e"));
  }

  #[test]
  fn convert_request_to_native_request_with_headers() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      headers: Some(hashmap! {
        "A".to_string() => vec!["B".to_string()],
        "B".to_string() => vec!["C".to_string(), "D".to_string()]
      }),
      .. HttpRequest::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));

    let headers = request_builder.headers();
    expect!(headers.len()).to(be_equal_to(3));
    expect!(&headers["A"]).to(be_equal_to("B"));
    expect!(&headers["B"]).to(be_equal_to("C"));
    expect!(headers.get_all("B").iter().map(|v| v.to_str().unwrap()).collect_vec())
      .to(be_equal_to(vec!["C", "D"]));
  }

  #[test]
  fn convert_request_to_native_request_with_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      body: OptionalBody::from("body"),
      .. HttpRequest::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body().unwrap().as_bytes()).to(be_some().value("body".as_bytes()));
  }

  #[test]
  fn convert_request_to_native_request_with_null_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      body: OptionalBody::Null,
      .. HttpRequest::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body()).to(be_none());
  }

  #[test]
  fn convert_request_to_native_request_with_json_null_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      headers: Some(hashmap! {
        "Content-Type".to_string() => vec!["application/json".to_string()]
      }),
      body: OptionalBody::Null,
      .. HttpRequest::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &hashmap!{}).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body().unwrap().as_bytes()).to(be_some().value("null".as_bytes()));
  }

  #[tokio::test]
  async fn do_not_overwrite_custom_headers_with_headers_from_pact_file() {
    let request = HttpRequest {
      headers: Some(hashmap!{
        "X-A".to_string() => vec![ "val-a".to_string() ],
        "X-B".to_string() => vec![ "val-b".to_string() ],
        "X-C".to_string() => vec![ "val-c".to_string() ]
      }),
      .. HttpRequest::default()
    };
    let options = VerificationOptions {
      custom_headers: hashmap!{
        "X-B".to_string() => "other-b".to_string()
      },
      .. VerificationOptions::<NullRequestFilterExecutor>::default()
    };
    let client = configure_http_client(&options).unwrap();

    let server = PactBuilderAsync::new("make_provider_request", "provider")
      .interaction("request with headers to be overwritten", "", |mut i| async move {
        i.request
          .method("GET")
          .header("X-A", "val-a")
          .header("X-B", "other-b")
          .header("X-C", "val-c");
        i.response.ok();
        i
      })
      .await
      .start_mock_server(None, None);

    #[allow(deprecated)]
    let provider = ProviderInfo {
      port: server.url().port(),
      .. ProviderInfo::default()
    };
    super::make_provider_request(&provider, &request, &options, &client, None).await.unwrap();
  }

  #[test]
  fn convert_request_to_native_request_with_custom_headers() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = HttpRequest {
      headers: Some(hashmap! {
        "X-A".to_string() => vec![ "val-a".to_string() ],
        "X-B".to_string() => vec![ "val-b".to_string() ],
        "X-C".to_string() => vec![ "val-c".to_string() ]
      }),
      .. HttpRequest::default()
    };
    let custom_headers = hashmap!{
      "X-B".to_string() => "other-b".to_string(),
      "X-D".to_string() => "val-d".to_string()
    };
    let request_builder = create_native_request(&client, &base_url, &request, &custom_headers).unwrap().build().unwrap();

    let headers = request_builder.headers();
    let keys = headers.keys()
      .map(|k| k.as_str())
      .sorted()
      .collect_vec();
    expect!(keys).to(be_equal_to(vec![
      "x-a",
      "x-c"
    ]));
  }
}
