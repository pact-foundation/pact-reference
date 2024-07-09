//! Structs and functions for interacting with a Pact Broker

use std::collections::HashMap;
use std::panic::RefUnwindSafe;
use std::str::from_utf8;

use anyhow::anyhow;
use futures::stream::*;
use itertools::Itertools;
use maplit::hashmap;
use pact_models::http_utils;
use pact_models::http_utils::HttpAuth;
use pact_models::json_utils::json_to_string;
use pact_models::pact::{load_pact_from_json, Pact};
use regex::{Captures, Regex};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use tracing::{debug, error, info, trace, warn};

use pact_matching::Mismatch;

use crate::{MismatchResult, VERIFIER_VERSION};
use crate::metrics::VerificationMetrics;
use crate::utils::with_retries;

fn is_true(object: &serde_json::Map<String, Value>, field: &str) -> bool {
    match object.get(field) {
        Some(json) => match *json {
            serde_json::Value::Bool(b) => b,
            _ => false
        },
        None => false
    }
}

fn as_string(json: &Value) -> String {
    match *json {
        serde_json::Value::String(ref s) => s.clone(),
        _ => format!("{}", json)
    }
}

fn content_type(response: &reqwest::Response) -> String {
    match response.headers().get("content-type") {
        Some(value) => value.to_str().unwrap_or("text/plain").into(),
        None => "text/plain".to_string()
    }
}

fn json_content_type(response: &reqwest::Response) -> bool {
    match content_type(response).parse::<mime::Mime>() {
        Ok(mime) => {
            match (mime.type_().as_str(), mime.subtype().as_str(), mime.suffix()) {
                ("application", "json", None) => true,
                ("application", "hal", Some(mime::JSON)) => true,
                _ => false
            }
        }
        Err(_) => false
    }
}

fn find_entry(map: &serde_json::Map<String, Value>, key: &str) -> Option<(String, Value)> {
    match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
        Some(k) => map.get(k).map(|v| (key.to_string(), v.clone()) ),
        None => None
    }
}

/// Errors that can occur with a Pact Broker
#[derive(Debug, Clone, thiserror::Error)]
pub enum PactBrokerError {
  /// Error with a HAL link
  #[error("Error with a HAL link - {0}")]
  LinkError(String),
  /// Error with the content of a HAL resource
  #[error("Error with the content of a HAL resource - {0}")]
  ContentError(String),
  #[error("IO Error - {0}")]
  /// IO Error
  IoError(String),
  /// Link/Resource was not found
  #[error("Link/Resource was not found - {0}")]
  NotFound(String),
  /// Invalid URL
  #[error("Invalid URL - {0}")]
  UrlError(String),
  /// Validation error
  #[error("failed validation - {0:?}")]
  ValidationError(Vec<String>)
}

impl PartialEq<String> for PactBrokerError {
    fn eq(&self, other: &String) -> bool {
        let mut buffer = String::new();
        match self {
            PactBrokerError::LinkError(s) => buffer.push_str(s),
            PactBrokerError::ContentError(s) => buffer.push_str(s),
            PactBrokerError::IoError(s) => buffer.push_str(s),
            PactBrokerError::NotFound(s) => buffer.push_str(s),
            PactBrokerError::UrlError(s) => buffer.push_str(s),
            PactBrokerError::ValidationError(errors) => buffer.push_str(errors.iter().join(", ").as_str())
        };
        buffer == *other
    }
}

impl <'a> PartialEq<&'a str> for PactBrokerError {
    fn eq(&self, other: &&str) -> bool {
        let message = match self {
            PactBrokerError::LinkError(s) => s.clone(),
            PactBrokerError::ContentError(s) => s.clone(),
            PactBrokerError::IoError(s) => s.clone(),
            PactBrokerError::NotFound(s) => s.clone(),
            PactBrokerError::UrlError(s) => s.clone(),
            PactBrokerError::ValidationError(errors) => errors.iter().join(", ")
        };
        message.as_str() == *other
    }
}

impl From<url::ParseError> for PactBrokerError {
  fn from(err: url::ParseError) -> Self {
    PactBrokerError::UrlError(format!("{}", err))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
/// Structure to represent a HAL link
pub struct Link {
  /// Link name
  pub name: String,
  /// Link HREF
  pub href: Option<String>,
  /// If the link is templated (has expressions in the HREF that need to be expanded)
  pub templated: bool,
  /// Link title
  pub title: Option<String>
}

impl Link {

  /// Create a link from serde JSON data
  pub fn from_json(link: &str, link_data: &serde_json::Map<String, serde_json::Value>) -> Link {
    Link {
      name: link.to_string(),
      href: find_entry(link_data, &"href".to_string())
        .map(|(_, href)| as_string(&href)),
      templated: is_true(link_data, "templated"),
      title: link_data.get("title").map(|title| as_string(title))
    }
  }

  /// Converts the Link into a JSON representation
  pub fn as_json(&self) -> serde_json::Value {
    match (self.href.clone(), self.title.clone()) {
      (Some(href), Some(title)) => json!({
        "href": href,
        "title": title,
        "templated": self.templated
      }),
      (Some(href), None) => json!({
        "href": href,
        "templated": self.templated
      }),
      (None, Some(title)) => json!({
        "title": title,
        "templated": self.templated
      }),
      (None, None) => json!({
        "templated": self.templated
      })
    }
  }
}

impl Default for Link {
  fn default() -> Self {
    Link {
      name: "link".to_string(),
      href: None,
      templated: false,
      title: None
    }
  }
}

/// HAL aware HTTP client
#[derive(Clone)]
pub struct HALClient {
  client: reqwest::Client,
  url: String,
  path_info: Option<Value>,
  auth: Option<HttpAuth>,
  retries: u8
}

impl HALClient {
  /// Initialise a client with the URL and optional authentication
  pub fn with_url(url: &str, auth: Option<HttpAuth>) -> HALClient {
    HALClient { url: url.to_string(), auth, ..HALClient::default() }
  }

  fn update_path_info(&self, path_info: serde_json::Value) -> HALClient {
    HALClient {
      client: self.client.clone(),
      url: self.url.clone(),
      path_info: Some(path_info),
      auth: self.auth.clone(),
      retries: self.retries
    }
  }

  /// Navigate to the resource with the given link name. If the link is templated, the optional
  /// template values will be used to resolve the template variables. This will return a clone
  /// of the current client with the updated path.
  pub async fn navigate(
    &self,
    link: &'static str,
    template_values: &HashMap<String, String>
  ) -> Result<HALClient, PactBrokerError> {
    trace!("navigate(link='{}', template_values={:?})", link, template_values);

    let client = if self.path_info.is_none() {
      let path_info = self.fetch("").await?;
      self.update_path_info(path_info)
    } else {
      self.clone()
    };

    let path_info = client.fetch_link(link, template_values).await?;
    let client = client.update_path_info(path_info);

    Ok(client)
  }

    fn find_link(&self, link: &'static str) -> Result<Link, PactBrokerError> {
        match self.path_info {
            None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
                self.url, link))),
            Some(ref json) => match json.get("_links") {
                Some(json) => match json.get(link) {
                    Some(link_data) => link_data.as_object()
                        .map(|link_data| Link::from_json(&link.to_string(), &link_data))
                        .ok_or_else(|| PactBrokerError::LinkError(format!("Link is malformed, expected an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }

    async fn fetch_link(
      &self,
      link: &'static str,
      template_values: &HashMap<String, String>
    ) -> Result<Value, PactBrokerError> {
      trace!("fetch_link(link='{}', template_values={:?})", link, template_values);
      let link_data = self.find_link(link)?;
      self.fetch_url(&link_data, template_values).await
    }

  /// Fetch the resource at the Link from the Pact broker
  pub async fn fetch_url(
    &self,
    link: &Link,
    template_values: &HashMap<String, String>
  ) -> Result<Value, PactBrokerError> {
    trace!("fetch_url(link={:?}, template_values={:?})", link, template_values);

    let link_url = if link.templated {
      debug!("Link URL is templated");
      self.parse_link_url(&link, &template_values)
    } else {
      link.href.clone()
        .ok_or_else(|| PactBrokerError::LinkError(
          format!("Link is malformed, there is no href. URL: '{}', LINK: '{}'", self.url, link.name)
        ))
    }?;

    let base_url = self.url.parse::<Url>()?;
    let joined_url = base_url.join(&link_url)?;
    self.fetch(joined_url.path().into()).await
  }

  async fn fetch(&self, path: &str) -> Result<Value, PactBrokerError> {
    info!("Fetching path '{}' from pact broker", path);
    trace!(%path, broker_url = %self.url, ">> fetch");

    let url = self.resolve_path(path)?;
    debug!("Final broker URL: {}", url);

    let request_builder = match self.auth {
        Some(ref auth) => match auth {
            HttpAuth::User(username, password) => self.client.get(url).basic_auth(username, password.clone()),
            HttpAuth::Token(token) => self.client.get(url).bearer_auth(token),
            _ => self.client.get(url)
        },
        None => self.client.get(url)
    }.header("accept", "application/hal+json, application/json");

    let response = with_retries(self.retries, request_builder).await
      .map_err(|err| {
          PactBrokerError::IoError(format!("Failed to access pact broker path '{}' - {}. URL: '{}'",
              &path,
              err,
              &self.url,
          ))
      })?;

    self.parse_broker_response(path.to_string(), response)
        .await
  }

  fn resolve_path(&self, path: &str) -> Result<Url, PactBrokerError> {
    let broker_url = self.url.parse::<Url>()?;
    let context_path = broker_url.path();
    let url = if path.is_empty() {
      broker_url
    } else if !context_path.is_empty() && context_path != "/" {
      if path.starts_with(context_path) {
        let mut base_url = broker_url.clone();
        base_url.set_path("/");
        base_url.join(path)?
      } else if path.starts_with("/") {
        let mut base_url = broker_url.clone();
        base_url.set_path(path);
        base_url
      } else {
        let mut base_url = broker_url.clone();
        let mut cp = context_path.to_string();
        cp.push('/');
        base_url.set_path(cp.as_str());
        base_url.join(path)?
      }
    } else {
      broker_url.join(path)?
    };
    Ok(url)
  }

  async fn parse_broker_response(
        &self,
        path: String,
        response: reqwest::Response,
    ) -> Result<Value, PactBrokerError> {
      let is_json_content_type = json_content_type(&response);
      let content_type = content_type(&response);
      let status_code = response.status();

      if status_code.is_success() {
        if is_json_content_type {
          response.json::<Value>()
            .await
            .map_err(|err| PactBrokerError::ContentError(
              format!("Did not get a valid HAL response body from pact broker path '{}' - {}. URL: '{}'",
                      path, err, self.url)
            ))
        } else {
          debug!("Request from broker was a success, but the response body was not JSON");
          Err(PactBrokerError::ContentError(
            format!("Did not get a valid HAL response body from pact broker path '{}', content type is '{}'. URL: '{}'",
              path, content_type, self.url)
          ))
        }
      } else if status_code.as_u16() == 404 {
          Err(PactBrokerError::NotFound(
              format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                      status_code, self.url
              )
          ))
      } else if status_code.as_u16() == 400 {
        let body = response.bytes()
          .await
          .map_err(|_| PactBrokerError::IoError(
            format!("Failed to download response body for path '{}'. URL: '{}'", &path, self.url)
          ))?;

        if is_json_content_type {
          let errors = serde_json::from_slice(&body)
            .map_err(|err| PactBrokerError::ContentError(
              format!("Did not get a valid HAL response body from pact broker path '{}' - {}. URL: '{}'",
                      path, err, self.url)
            ))?;
          Err(handle_validation_errors(errors))
        } else {
          let body = from_utf8(&body)
            .map(|b| b.to_string())
            .unwrap_or_else(|err| format!("could not read body: {}", err));
          error!("Request to pact broker path '{}' failed: {}", path, body);
          Err(PactBrokerError::IoError(
            format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                    status_code, self.url
            )
          ))
        }
      } else {
        Err(PactBrokerError::IoError(
          format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
            status_code, self.url)
        ))
      }
    }

    fn parse_link_url(&self, link: &Link, values: &HashMap<String, String>) -> Result<String, PactBrokerError> {
      match link.href {
        Some(ref href) => {
          debug!("templated URL = {}", href);
          let re = Regex::new(r"\{(\w+)}").unwrap();
          let final_url = re.replace_all(href, |caps: &Captures| {
            let lookup = caps.get(1).unwrap().as_str();
            trace!("Looking up value for key '{}'", lookup);
            match values.get(lookup) {
              Some(val) => urlencoding::encode(val.as_str()).to_string(),
              None => {
                warn!("No value was found for key '{}', mapped values are {:?}", lookup, values);
                format!("{{{}}}", lookup)
              }
            }
          });
          debug!("final URL = {}", final_url);
          Ok(final_url.to_string())
        },
        None => Err(PactBrokerError::LinkError(
          format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{}'",
          self.url, link.name)))
      }
    }

  /// Iterate over all the links by name
  pub fn iter_links(&self, link: &str) -> Result<Vec<Link>, PactBrokerError> {
    match self.path_info {
      None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
        self.url, link))),
      Some(ref json) => match json.get("_links") {
        Some(json) => match json.get(&link) {
          Some(link_data) => link_data.as_array()
              .map(|link_data| link_data.iter().map(|link_json| match link_json {
                Value::Object(data) => Link::from_json(&link, data),
                Value::String(s) => Link { name: link.to_string(), href: Some(s.clone()), templated: false, title: None },
                _ => Link { name: link.to_string(), href: Some(link_json.to_string()), templated: false, title: None }
              }).collect())
              .ok_or_else(|| PactBrokerError::LinkError(format!("Link is malformed, expected an object but got {}. URL: '{}', LINK: '{}'",
                  link_data, self.url, link))),
          None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
            link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
        },
        None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
          self.url, link)))
      }
    }
  }

  async fn post_json(&self, url: &str, body: &str) -> Result<serde_json::Value, PactBrokerError> {
    trace!("post_json(url='{}', body='{}')", url, body);

    self.send_document(url, body, Method::POST).await
  }

  async fn put_json(&self, url: &str, body: &str) -> Result<serde_json::Value, PactBrokerError> {
    trace!("put_json(url='{}', body='{}')", url, body);

    self.send_document(url, body, Method::PUT).await
  }

  async fn send_document(&self, url: &str, body: &str, method: Method) -> Result<Value, PactBrokerError> {
    debug!("Sending JSON to {} using {}: {}", url, method, body);

    let base_url = &self.url.parse::<Url>()?;
    let url = if url.starts_with("/") {
      base_url.join(url)?
    } else {
      let url = url.parse::<Url>()?;
      base_url.join(&url.path())?
    };

    let request_builder = match self.auth {
      Some(ref auth) => match auth {
        HttpAuth::User(username, password) => self.client
          .request(method, url.clone())
          .basic_auth(username, password.clone()),
        HttpAuth::Token(token) => self.client
          .request(method, url.clone())
          .bearer_auth(token),
        _ => self.client.request(method, url.clone())
      },
      None => self.client.request(method, url.clone())
    }
      .header("Content-Type", "application/json")
      .header("Accept", "application/hal+json")
      .header("Accept", "application/json")
      .body(body.to_string());

    let response = with_retries(self.retries, request_builder).await;
    match response {
      Ok(res) => self.parse_broker_response(url.path().to_string(), res).await,
      Err(err) => Err(PactBrokerError::IoError(
        format!("Failed to send JSON to the pact broker URL '{}' - IoError {}", url, err)
      ))
    }
  }

  fn with_doc_context(&self, doc_attributes: &[Link]) -> Result<HALClient, PactBrokerError> {
    let links: serde_json::Map<String, serde_json::Value> = doc_attributes.iter()
      .map(|link| (link.name.clone(), link.as_json())).collect();
    let links_json = json!({
      "_links": json!(links)
    });
    Ok(self.update_path_info(links_json))
  }
}

fn handle_validation_errors(body: Value) -> PactBrokerError {
  match &body {
    Value::Object(attrs) => if let Some(errors) = attrs.get("errors") {
      match errors {
        Value::Array(values) => PactBrokerError::ValidationError(values.iter().map(|v| json_to_string(v)).collect()),
        Value::Object(errors) => PactBrokerError::ValidationError(
          errors.iter().map(|(field, errors)| {
            match errors {
              Value::String(error) => format!("{}: {}", field, error),
              Value::Array(errors) => format!("{}: {}", field, errors.iter().map(|err| json_to_string(err)).join(", ")),
              _ => format!("{}: {}", field, errors),
            }
          })
          .collect()
        ),
        Value::String(s) => PactBrokerError::ValidationError(vec![s.clone()]),
        _ => PactBrokerError::ValidationError(vec![errors.to_string()])
      }
    } else {
      PactBrokerError::ValidationError(vec![body.to_string()])
    },
    Value::String(s) => PactBrokerError::ValidationError(vec![s.clone()]),
    _ => PactBrokerError::ValidationError(vec![body.to_string()])
  }
}

impl Default for HALClient {
  fn default() -> Self {
    HALClient {
      client: reqwest::ClientBuilder::new()
        .user_agent(format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
        .tcp_keepalive(None)
        .build()
        .unwrap(),
      url: "".to_string(),
      path_info: None,
      auth: None,
      retries: 3
    }
  }
}

fn links_from_json(json: &Value) -> Vec<Link> {
   match json.get("_links") {
    Some(json) => match json {
      Value::Object(v) => {
        v.iter().map(|(link, json)| match json {
          Value::Object(attr) => Link::from_json(link, attr),
          _ => Link { name: link.clone(), .. Link::default() }
        }).collect()
      },
      _ => vec![]
    },
    None => vec![]
  }
}

/// Fetches the pacts from the broker that match the provider name
pub async fn fetch_pacts_from_broker(
  broker_url: &str,
  provider_name: &str,
  auth: Option<HttpAuth>
) -> anyhow::Result<Vec<anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Option<PactVerificationContext>, Vec<Link>)>>> {
  trace!("fetch_pacts_from_broker(broker_url='{}', provider_name='{}', auth={})", broker_url,
    provider_name, auth.clone().unwrap_or_default());

    let mut hal_client = HALClient::with_url(broker_url, auth);
    let template_values = hashmap!{ "provider".to_string() => provider_name.to_string() };

    hal_client = hal_client.navigate("pb:latest-provider-pacts", &template_values)
        .await
        .map_err(move |err| {
            match err {
                PactBrokerError::NotFound(_) =>
                    PactBrokerError::NotFound(
                        format!("No pacts for provider '{}' where found in the pact broker. URL: '{}'",
                            provider_name, broker_url)),
                _ => err
            }
        })?;

    let pact_links = hal_client.clone().iter_links("pacts")?;

    let results: Vec<_> = futures::stream::iter(pact_links)
        .map(|ref pact_link| {
          match pact_link.href {
            Some(_) => Ok((hal_client.clone(), pact_link.clone())),
            None => Err(
              PactBrokerError::LinkError(
                format!(
                  "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{:?}'",
                  &hal_client.url,
                  pact_link
                )
              )
            )
          }
        })
        .and_then(|(hal_client, pact_link)| {
          let value = template_values.clone();
          async move {
            let pact_json = hal_client.fetch_url(&pact_link, &value).await?;
            Ok((pact_link, pact_json))
          }
        })
        .map(|result| {
          match result {
            Ok((pact_link, pact_json)) => {
              let href = pact_link.href.unwrap_or_default();
              let links = links_from_json(&pact_json);
              load_pact_from_json(href.as_str(), &pact_json)
                .map(|pact| (pact, None, links))
            },
            Err(err) => Err(err.into())
          }
        })
        .into_stream()
        .collect()
        .await;

    Ok(results)
}

/// Fetch Pacts from the broker using the "provider-pacts-for-verification" endpoint
pub async fn fetch_pacts_dynamically_from_broker(
  broker_url: &str,
  provider_name: String,
  pending: bool,
  include_wip_pacts_since: Option<String>,
  provider_tags: Vec<String>,
  provider_branch: Option<String>,
  consumer_version_selectors: Vec<ConsumerVersionSelector>,
  auth: Option<HttpAuth>
) -> anyhow::Result<Vec<Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Option<PactVerificationContext>, Vec<Link>), PactBrokerError>>> {
  trace!("fetch_pacts_dynamically_from_broker(broker_url='{}', provider_name='{}', pending={}, \
    include_wip_pacts_since={:?}, provider_tags: {:?}, consumer_version_selectors: {:?}, auth={})",
    broker_url, provider_name, pending, include_wip_pacts_since, provider_tags,
    consumer_version_selectors, auth.clone().unwrap_or_default());

    let mut hal_client = HALClient::with_url(broker_url, auth);
    let template_values = hashmap!{ "provider".to_string() => provider_name.clone() };

    hal_client = hal_client.navigate("pb:provider-pacts-for-verification", &template_values)
    .await
    .map_err(move |err| {
      match err {
        PactBrokerError::NotFound(_) =>
        PactBrokerError::NotFound(
          format!("No pacts for provider '{}' were found in the pact broker. URL: '{}'",
          provider_name.clone(), broker_url)),
          _ => err
        }
      })?;

    // Construct the Pacts for verification payload
    let pacts_for_verification = PactsForVerificationRequest {
      provider_version_tags: provider_tags,
      provider_version_branch: provider_branch,
      include_wip_pacts_since,
      consumer_version_selectors,
      include_pending_status: pending,
    };
    let request_body = serde_json::to_string(&pacts_for_verification).unwrap();

    // Post the verification request
    let response = match hal_client.find_link("self") {
      Ok(link) => {
        let link = hal_client.clone().parse_link_url(&link, &hashmap!{})?;
        match hal_client.clone().post_json(link.as_str(), request_body.as_str()).await {
          Ok(res) => Some(res),
          Err(err) => {
            info!("error response for pacts for verification: {} ", err);
            return Err(anyhow!(err))
          }
        }
      },
      Err(e) => return Err(anyhow!(e))
    };

    // Find all of the Pact links
    let pact_links = match response {
      Some(v) => {
        let pfv: PactsForVerificationResponse = serde_json::from_value(v)
          .map_err(|err| {
            trace!("Failed to deserialise PactsForVerificationResponse: {}", err);
            err
          })
          .unwrap_or(PactsForVerificationResponse { embedded: PactsForVerificationBody { pacts: vec!() } });
        trace!(?pfv, "got pacts for verification response");

        if pfv.embedded.pacts.len() == 0 {
          return Err(anyhow!(PactBrokerError::NotFound(format!("No pacts were found for this provider"))))
        };

        let links: Result<Vec<(Link, PactVerificationContext)>, PactBrokerError> = pfv.embedded.pacts.iter().map(| p| {
          match p.links.get("self") {
            Some(l) => Ok((l.clone(), p.into())),
            None => Err(
              PactBrokerError::LinkError(
                format!(
                  "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', PATH: '{:?}'",
                  &hal_client.url,
                  &p.links,
                )
              )
            )
          }
        }).collect();

        links
      },
      None => Err(PactBrokerError::NotFound(format!("No pacts were found for this provider")))
    }?;

    let results: Vec<_> = futures::stream::iter(pact_links)
      .map(|(ref pact_link, ref context)| {
        match pact_link.href {
          Some(_) => Ok((hal_client.clone(), pact_link.clone(), context.clone())),
          None => Err(
            PactBrokerError::LinkError(
              format!(
                "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{:?}'",
                &hal_client.url,
                pact_link
              )
            )
          )
        }
      })
      .and_then(|(hal_client, pact_link, context)| {
        let value = template_values.clone();
        async move {
          let pact_json = hal_client.fetch_url(
            &pact_link.clone(),
            &value
          ).await?;
          Ok((pact_link, pact_json, context))
        }
      })
      .map(|result| {
        match result {
          Ok((pact_link, pact_json, context)) => {
            let href = pact_link.href.unwrap_or_default();
            let links = links_from_json(&pact_json);
            load_pact_from_json(href.as_str(), &pact_json)
              .map(|pact| (pact, Some(context), links))
              .map_err(|err| PactBrokerError::ContentError(format!("{}", err)))
          },
          Err(err) => Err(err)
        }
      })
      .into_stream()
      .collect()
      .await;

    Ok(results)
}

/// Fetch the Pact from the given URL, using any required authentication. This will use a GET
/// request to the given URL and parse the result into a Pact model. It will also look for any HAL
/// links in the response, returning those if found.
pub async fn fetch_pact_from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<(Box<dyn Pact + Send + Sync + RefUnwindSafe>, Vec<Link>)> {
  let url = url.to_string();
  let auth = auth.clone();
  let (url, pact_json) = tokio::task::spawn_blocking(move || {
    http_utils::fetch_json_from_url(&url, &auth)
  }).await??;
  let pact = load_pact_from_json(&url, &pact_json)?;
  let links = links_from_json(&pact_json);
  Ok((pact, links))
}

/// Struct that wraps the result of a verification test
pub enum TestResult {
  /// Test was OK
  Ok(Vec<Option<String>>),
  /// Test failed verification. Each value is a tuple of interaction ID, interaction description,
  /// mismatches
  Failed(Vec<(Option<String>, Option<String>, Option<MismatchResult>)>)
}

impl TestResult {
  /// Convert this test result to a boolean value
  pub fn to_bool(&self) -> bool {
    match self {
      TestResult::Ok(_) => true,
      _ => false
    }
  }
}

/// Publishes the result to the "pb:publish-verification-results" link in the links associated with the pact
pub async fn publish_verification_results(
  links: Vec<Link>,
  broker_url: &str,
  auth: Option<HttpAuth>,
  result: TestResult,
  version: String,
  build_url: Option<String>,
  provider_tags: Vec<String>,
  branch: Option<String>,
  metrics_data: Option<&VerificationMetrics>
) -> Result<serde_json::Value, PactBrokerError> {
  let hal_client = HALClient::with_url(broker_url, auth.clone());

  if branch.is_some() {
    publish_provider_branch(&hal_client, &links, &branch.unwrap(), &version).await?;
  }

  if !provider_tags.is_empty() {
    publish_provider_tags(&hal_client, &links, provider_tags, &version).await?;
  }

  let publish_link = links
      .iter()
      .cloned()
      .find(|item| item.name.to_ascii_lowercase() == "pb:publish-verification-results")
      .ok_or_else(|| PactBrokerError::LinkError(
          "Response from the pact broker has no 'pb:publish-verification-results' link".into()
      ))?;

  let json = build_payload(result, version, build_url, metrics_data);
  hal_client.post_json(publish_link.href.unwrap_or_default().as_str(), json.to_string().as_str()).await
}

fn build_payload(
  result: TestResult,
  version: String,
  build_url: Option<String>,
  metrics_data: Option<&VerificationMetrics>
) -> serde_json::Value {
  let mut json = json!({
    "success": result.to_bool(),
    "providerApplicationVersion": version,
    "verifiedBy": {
      "implementation": "Pact-Rust",
      "version": VERIFIER_VERSION
    }
  });
  let json_obj = json.as_object_mut().unwrap();

  if let Some(build_url) = build_url {
    json_obj.insert("buildUrl".into(), build_url.into());
  }

  if let Some(metrics_data) = metrics_data {
    json_obj.get_mut("verifiedBy").unwrap()["clientLanguage"] = json!({
      "testFramework": metrics_data.test_framework,
      "name": metrics_data.app_name,
      "version": metrics_data.app_version
    });
  }

  match result {
    TestResult::Failed(mismatches) => {
      let values = mismatches.iter()
        .chunk_by(|(id, _, _)| id.clone().unwrap_or_default())
        .into_iter()
        .map(|(key, mismatches)| {
          let acc: (Vec<serde_json::Value>, Vec<serde_json::Value>, Option<String>) = (vec![], vec![], None);
          let (mismatches, errors, desc) = mismatches.fold(acc, |mut acc, (_, desc, result)| {
            if let Some(mismatch) = result {
              match mismatch {
                MismatchResult::Mismatches { mismatches, .. } => {
                  for mismatch in mismatches {
                    match mismatch {
                      Mismatch::MethodMismatch { expected, actual } => acc.0.push(json!({
                        "attribute": "method",
                        "description": format!("Expected method of {} but received {}", expected, actual)
                      })),
                      Mismatch::PathMismatch { mismatch, .. } => acc.0.push(json!({
                        "attribute": "path",
                        "description": mismatch
                      })),
                      Mismatch::StatusMismatch { mismatch, .. } => acc.0.push(json!({
                        "attribute": "status",
                        "description": mismatch
                      })),
                      Mismatch::QueryMismatch { parameter, mismatch, .. } => acc.0.push(json!({
                        "attribute": "query",
                        "identifier": parameter,
                        "description": mismatch
                      })),
                      Mismatch::HeaderMismatch { key, mismatch, .. } => acc.0.push(json!({
                        "attribute": "header",
                        "identifier": key,
                        "description": mismatch
                      })),
                      Mismatch::BodyTypeMismatch { expected, actual, .. } => acc.0.push(json!({
                        "attribute": "body",
                        "identifier": "$",
                        "description": format!("Expected body type of '{}' but received '{}'", expected, actual)
                      })),
                      Mismatch::BodyMismatch { path, mismatch, .. } => acc.0.push(json!({
                        "attribute": "body",
                        "identifier": path,
                        "description": mismatch
                      })),
                      Mismatch::MetadataMismatch { key, mismatch, .. } => acc.0.push(json!({
                        "attribute": "metadata",
                        "identifier": key,
                        "description": mismatch
                      }))
                    }
                  }
                },
                MismatchResult::Error(err, _) => acc.1.push(json!({ "message": err }))
              };
            };

            if acc.2.is_none() {
              acc.2 = desc.clone();
            }

            acc
          });

          let mut json = json!({
            "interactionId": key,
            "success": mismatches.is_empty() && errors.is_empty()
          });

          if !mismatches.is_empty() {
            json["mismatches"] = json!(mismatches);
          }

          if !errors.is_empty() {
            json["exceptions"] = json!(errors);
          }

          if let Some(desc) = desc {
            json["interactionDescription"] = json!(desc);
          }

          json
        }).collect::<Vec<serde_json::Value>>();

      json_obj.insert("testResults".into(), serde_json::Value::Array(values));
    }
    TestResult::Ok(ids) => {
      let values = ids.iter().filter(|id| id.is_some())
        .map(|id| json!({
        "interactionId": id.clone().unwrap_or_default(),
        "success": true
      })).collect();
      json_obj.insert("testResults".into(), serde_json::Value::Array(values));
    }
  }
  json
}

async fn publish_provider_tags(
  hal_client: &HALClient,
  links: &[Link],
  provider_tags: Vec<String>,
  version: &str) -> Result<(), PactBrokerError> {
  let hal_client = hal_client.clone().with_doc_context(links)?
    .navigate("pb:provider", &hashmap!{}).await?;
  match hal_client.find_link("pb:version-tag") {
    Ok(link) => {
      for tag in &provider_tags {
        let template_values = hashmap! {
          "version".to_string() => version.to_string(),
          "tag".to_string() => tag.clone()
        };
        match hal_client.clone().put_json(hal_client.clone().parse_link_url(&link, &template_values)?.as_str(), "{}").await {
          Ok(_) => debug!("Pushed tag {} for provider version {}", tag, version),
          Err(err) => {
            error!("Failed to push tag {} for provider version {}", tag, version);
            return Err(err);
          }
        }
      };
      Ok(())
    },
    Err(_) => Err(PactBrokerError::LinkError("Can't publish provider tags as there is no 'pb:version-tag' link".to_string()))
  }
}

async fn publish_provider_branch(
  hal_client: &HALClient,
  links: &[Link],
  branch: &str,
  version: &str
) -> Result<(), PactBrokerError> {
  let hal_client = hal_client.clone().with_doc_context(links)?
    .navigate("pb:provider", &hashmap!{}).await?;

    match hal_client.find_link("pb:branch-version") {
    Ok(link) => {
      let template_values = hashmap! {
        "branch".to_string() => branch.to_string(),
        "version".to_string() => version.to_string(),
      };
      match hal_client.clone().put_json(hal_client.clone().parse_link_url(&link, &template_values)?.as_str(), "{}").await {
        Ok(_) => debug!("Pushed branch {} for provider version {}", branch, version),
        Err(err) => {
          error!("Failed to push branch {} for provider version {}", branch, version);
          return Err(err);
        }
      }
      Ok(())
    },
    Err(_) => Err(PactBrokerError::LinkError("Can't publish provider branch as there is no 'pb:branch-version' link. Please ugrade to Pact Broker version 2.86.0 or later for branch support".to_string()))
  }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Structure to represent a HAL link
pub struct ConsumerVersionSelector {
  /// Application name to filter the results on
  /// allows a selector to only be applied to a certain consumer.
  pub consumer: Option<String>,
  /// Tag
  /// the tag name(s) of the consumer versions to get the pacts for. 
  /// *This field is still supported but it is recommended to use the `branch` in preference now.*
  pub tag: Option<String>,
  /// Fallback tag if Tag doesn't exist
  /// the name of the tag to fallback to if the specified `tag` does not exist. 
  /// This is useful when the consumer and provider use matching branch names to coordinate the development of new features.
  /// *This field is still supported but it is recommended to use two separate selectors - one with the main branch name and one with the feature branch name.*
  pub fallback_tag: Option<String>,
  /// Only select the latest (if false, this selects all pacts for a tag)
  /// Used in conjuction with the tag property. 
  /// If a tag is specified, and latest is true, then the latest pact for each of the consumers with that tag will be returned. 
  /// If a tag is specified and the latest flag is not set to true, all the pacts with the specified tag will be returned. 
  /// (This might seem a bit weird, but it's done this way to match the syntax used for the matrix query params. See https://docs.pact.io/selectors).
  pub latest: Option<bool>,
  /// Applications that have been deployed or released
  /// if the key is specified, can only be set to `true`. 
  /// Returns the pacts for all versions of the consumer that are currently deployed or released and currently supported in any environment. 
  /// Use of this selector requires that the deployment of the consumer application is recorded in the Pact Broker using the `pact-broker record-deployment` or `record-release` CLI.
  pub deployed_or_released: Option<bool>,
  /// Applications that have been deployed
  /// if the key is specified, can only be set to `true`. 
  /// Returns the pacts for all versions of the consumer that are currently deployed to any environment.
  /// Use of this selector requires that the deployment of the consumer application is recorded in the Pact Broker using the `pact-broker record-deployment` CLI.
  pub deployed: Option<bool>,
  /// Applications that have been released
  /// if the key is specified, can only be set to `true`. 
  /// Returns the pacts for all versions of the consumer that are released and currently supported in any environment. 
  /// Use of this selector requires that the deployment of the consumer application is recorded in the Pact Broker using the `pact-broker record-release` CLI.
  pub released: Option<bool>,
  /// Applications in a given environment
  /// the name of the environment containing the consumer versions for which to return the pacts. 
  /// Used to further qualify `{ "deployed": true }` or `{ "released": true }`.
  /// Normally, this would not be needed, as it is recommended to verify the pacts for all currently deployed/currently supported released versions.
  pub environment: Option<String>,
  /// Applications with the default branch set in the broker
  /// if the key is specified, can only be set to `true`. 
  /// Return the pacts for the configured `mainBranch` of each consumer. 
  /// Use of this selector requires that the consumer has configured the `mainBranch` property, and has set a branch name when publishing the pacts.
  pub main_branch: Option<bool>,
  /// Applications with the given branch
  /// the branch name of the consumer versions to get the pacts for.
  /// Use of this selector requires that the consumer has configured a branch name when publishing the pacts.
  pub branch: Option<String>,
  /// Applications that match the the provider version branch sent during verification
  /// if the key is specified, can only be set to `true`. 
  /// When true, returns the latest pact for any branch with the same name as the specified `provider_version_branch`.
  pub matching_branch: Option<bool>,
  /// Fallback branch if branch doesn't exist
  /// the name of the branch to fallback to if the specified branch does not exist. 
  /// Use of this property is discouraged as it may allow a pact to pass on a feature branch while breaking backwards compatibility with the main branch, which is generally not desired. 
  /// It is better to use two separate consumer version selectors, one with the main branch name, and one with the feature branch name, rather than use this property
  pub fallback_branch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PactsForVerificationResponse {
  #[serde(rename(deserialize = "_embedded"))]
  pub embedded: PactsForVerificationBody
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PactsForVerificationBody {
  pub pacts: Vec<PactForVerification>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PactForVerification {
  pub short_description: String,
  #[serde(rename(deserialize = "_links"))]
  pub links: HashMap<String, Link>,
  pub verification_properties: Option<PactVerificationProperties>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Request to send to determine the pacts to verify
pub struct PactsForVerificationRequest {
  /// Provider tags to use for determining pending pacts (if enabled)
  /// the tag name(s) for the provider application version that will be published with the verification results. 
  /// This is used by the Broker to determine whether or not a particular pact is in pending state or not. 
  /// This parameter can be specified multiple times. *This field is still supported but it is recommended to use the `provider_version_branch` in preference now.*
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub provider_version_tags: Vec<String>,
  /// Enable pending pacts feature
  /// When true, a pending boolean will be added to the verificationProperties in the response, 
  /// and an extra message will appear in the notices array to indicate why this pact is/is not in pending state.
  pub include_pending_status: bool,
  /// Find WIP pacts after given date
  /// The date from which to include the "work in progress" pacts. 
  /// See https://docs.pact.io/wip for more information on work in progress pacts.
  pub include_wip_pacts_since: Option<String>,
  /// Detailed pact selection criteria , see https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/
  pub consumer_version_selectors: Vec<ConsumerVersionSelector>,
  /// Current provider version branch if used
  /// the repository branch name for the provider application version that will be published with the verification results. 
  /// This is used by the Broker to determine whether or not a particular pact is in pending state or not.
  pub provider_version_branch: Option<String>
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Provides the context on why a Pact was included
pub struct PactVerificationContext {
  /// Description
  pub short_description: String,
  /// Properties
  pub verification_properties: PactVerificationProperties,
}

impl From<&PactForVerification> for PactVerificationContext {
  fn from(value: &PactForVerification) -> Self {
    PactVerificationContext {
      short_description: value.short_description.clone(),
      verification_properties: value.verification_properties.clone().unwrap_or_default()
    }
  }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// Properties associated with the verification context
pub struct PactVerificationProperties {
  #[serde(default)]
  /// If the Pact is pending
  pub pending: bool,
  /// Notices provided by the Pact Broker
  pub notices: Vec<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use pact_mock_server::mock_server::MockServerConfig;
  use pact_models::{Consumer, PactSpecification, Provider};
  use pact_models::prelude::RequestResponsePact;
  use pact_models::sync_interaction::RequestResponseInteraction;
  use pretty_assertions::assert_eq;

  use pact_consumer::*;
  use pact_consumer::mock_server::StartMockServerAsync;
  use pact_consumer::prelude::*;
  use pact_matching::Mismatch::MethodMismatch;

  use super::*;
  use super::{content_type, json_content_type};

  #[test_log::test(tokio::test)]
  async fn fetch_returns_an_error_if_there_is_no_pact_broker() {
    let client = HALClient::with_url("http://idont.exist:6666", None);
    expect!(client.fetch("/").await).to(be_err());
  }

  #[test_log::test(tokio::test)]
  async fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
        .interaction("a request to a non-existant path", "", |mut i| {
            i.given("the pact broker has a valid pact");
            i.request.path("/hello");
            i.response.status(404);
            i
        })
        .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.fetch("/hello").await;
    expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
        pact_broker.url())));
  }

  #[test_log::test(tokio::test)]
  async fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a non-json resource", "", |mut i| {
          i.request.path("/nonjson");
          i.response
              .header("Content-Type", "text/html")
              .body("<html></html>");
          i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.fetch("/nonjson").await;
    pretty_assertions::assert_eq!(format!("Error with the content of a HAL resource - Did not get a valid HAL response body from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
        pact_broker.url()), result.unwrap_err().to_string());
  }

    #[test]
    fn content_type_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json; charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(content_type(&response)).to(be_equal_to("application/hal+json; charset=utf-8".to_string()));
    }

    #[test]
    fn json_content_type_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/json")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test_log::test(tokio::test)]
    async fn user_agent_test() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to the broker includes a user-agent", "", |mut i| {
                i.request
                  .path("/user-agent")
                  .header("user-agent", format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

                i.response
                  .status(200)
                  .header("Content-Type", "application/hal+json")
                  .body("{\"_links\":{}}");
                i
            })
            .start_mock_server(None, None);

        let client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/user-agent").await;
        expect!(result).to(be_ok());
    }

    #[test]
    fn json_content_type_utf8_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json;charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test_log::test(tokio::test)]
    async fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal resource", "", |mut i| {
                i.request.path("/nonhal");
                i.response.header("Content-Type", "application/hal+json");
                i
            })
            .interaction("a request to a non-hal resource 2", "", |mut i| {
                i.request.path("/nonhal2");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("<html>This is not JSON</html>");
                i
            })
            .start_mock_server(None, None);

        let client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/nonhal").await;
        pretty_assertions::assert_eq!(
          format!("Error with the content of a HAL resource - Did not get a valid HAL response body from pact broker path \'/nonhal\' - error decoding response body. URL: '{}'",
            pact_broker.url()), result.unwrap_err().to_string());

        let result = client.fetch("/nonhal2").await;
        pretty_assertions::assert_eq!(
          format!("Error with the content of a HAL resource - Did not get a valid HAL response body from pact broker path \'/nonhal2\' - error decoding response body. URL: '{}'",
            pact_broker.url()), result.unwrap_err().to_string());
    }

  #[test_log::test(tokio::test)]
  async fn fetch_retries_the_request_on_50x_errors() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource", "", |mut i| {
        i.given("server returns a gateway error");
        i.request.path("/");
        i.response.status(503);
        i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let expected_requests = client.retries as usize;
    let result = client.fetch("/").await;
    expect!(result).to(be_err());
    expect!(pact_broker.metrics().requests).to(be_equal_to(expected_requests ));
  }

  #[test_log::test(tokio::test)]
  async fn fetch_supports_broker_urls_with_context_paths() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a resource from a base URL with a context path", "", |mut i| {
        i.request.path("/path/a/b/c");
        i.response
          .status(200)
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{}}");
        i
      })
      .interaction("a request to a root URL", "", |mut i| {
        i.request.path("/other/a/b/c");
        i.response
          .status(200)
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{}}");
        i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().join("/path").unwrap().as_str(), None);

    let result = client.fetch("/path/a/b/c").await;
    expect!(result).to(be_ok());

    let result = client.fetch("/other/a/b/c").await;
    expect!(result).to(be_ok());

    let result = client.fetch("a/b/c").await;
    expect!(result).to(be_ok());
  }

  #[test_log::test(tokio::test)]
  async fn post_json_retries_the_request_on_50x_errors() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a POST request", "", |mut i| {
        i.given("server returns a gateway error");
        i.request.path("/").method("POST");
        i.response.status(503);
        i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let expected_requests = client.retries as usize;
    let result = client.post_json(pact_broker.url().as_str(), "{}").await;
    expect!(result.clone()).to(be_err());
    expect!(pact_broker.metrics().requests).to(be_equal_to(expected_requests ));
  }

  #[test_log::test(tokio::test)]
  async fn put_json_retries_the_request_on_50x_errors() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a PUT request", "", |mut i| {
        i.given("server returns a gateway error");
        i.request.path("/").method("PUT");
        i.response.status(503);
        i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let expected_requests = client.retries as usize;
    let result = client.put_json(pact_broker.url().as_str(), "{}").await;
    expect!(result.clone()).to(be_err());
    expect!(pact_broker.metrics().requests).to(be_equal_to(expected_requests ));
  }

  #[test]
  fn parse_link_url_returns_error_if_there_is_no_href() {
    let client = HALClient::default();
    let link = Link { name: "link".to_string(), href: None, templated: false, title: None };
    expect!(client.parse_link_url(&link, &hashmap!{})).to(be_err().value(
      "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '', LINK: 'link'"));
  }

  #[test]
  fn parse_link_url_replaces_all_tokens_in_href() {
    let client = HALClient::default();
    let values = hashmap!{ "valA".to_string() => "A".to_string(), "valB".to_string() => "B".to_string() };

    let link = Link { name: "link".to_string(), href: Some("http://localhost".to_string()), templated: false, title: None };
    expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://localhost"));

    let link = Link { name: "link".to_string(), href: Some("http://{valA}/{valB}".to_string()), templated: false, title: None };
    expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/B"));

    let link = Link { name: "link".to_string(), href: Some("http://{valA}/{valC}".to_string()), templated: false, title: None };
    expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/{valC}"));
  }

  #[test]
  fn parse_link_url_encodes_the_tokens_in_href() {
    let client = HALClient::default();
    let values = hashmap!{ "valA".to_string() => "A".to_string(), "valB".to_string() => "B/C".to_string() };

    let link = Link { name: "link".to_string(), href: Some("http://{valA}/{valB}".to_string()), templated: false, title: None };
    expect!(client.clone().parse_link_url(&link, &values)).to(be_ok().value("http://A/B%2FC"));
  }

    #[test_log::test(tokio::test)]
    async fn fetch_link_returns_an_error_if_a_previous_resource_has_not_been_fetched() {
        let client = HALClient::with_url("http://localhost", None);
        let result = client.fetch_link("anything_will_do", &hashmap!{}).await;
        expect!(result).to(be_err().value("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'".to_string()));
    }

    #[test_log::test(tokio::test)]
    async fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", "", |mut i| async move {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
                i
            })
            .await
            .start_mock_server(None, None);

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link("hal2", &hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[test_log::test(tokio::test)]
    async fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", "", |mut i| async move {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
                i
            })
            .await
            .start_mock_server(None, None);

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link("any", &hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

  #[test_log::test(tokio::test)]
  async fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
        .interaction("a request to a hal resource", "", |mut i| async move {
            i.request.path("/");
            i.response
                .header("Content-Type", "application/hal+json")
                .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            i
        })
        .await
        .start_mock_server(None, None);

    let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.fetch("/").await;
    expect!(result.clone()).to(be_ok());
    client.path_info = result.ok();
    let result = client.fetch_link("any", &hashmap!{}).await;
    expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
        pact_broker.url())));
  }

    #[test_log::test(tokio::test)]
    async fn fetch_link_returns_the_resource_for_the_link() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", "", |mut i| async move {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
                i
            })
            .await
            .interaction("a request to next", "", |mut i| async move {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
                i
            })
            .await
            .start_mock_server(None, Some(MockServerConfig::with_keep_alive(true)));

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link("next", &hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

    #[test_log::test(tokio::test)]
    async fn fetch_link_handles_absolute_resource_links() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with absolute paths", "", |mut i| async move {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"http://localhost/abc\"},\"prev\":{\"href\":\"http://localhost/def\"}}}");
                i
            })
            .await
            .interaction("a request to next", "", |mut i| async move {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
                i
            })
            .await
            .start_mock_server_async(None, None)
            .await;

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        client.path_info = Some(client.fetch("/").await.unwrap());
        let result = client.fetch_link("next", &hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

    #[test_log::test(tokio::test)]
    async fn fetch_link_returns_the_resource_for_the_templated_link() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a templated hal resource", "", |mut i| async move {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"document\":{\"href\":\"/doc/{id}\",\"templated\":true}}}");
                i
            })
            .await
            .interaction("a request for a document", "", |mut i| async move {
                i.request.path("/doc/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
                i
            })
            .await
            .start_mock_server(None, None);

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.fetch_link("document", &hashmap!{ "id".to_string() => "abc".to_string() }).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

  #[test_log::test(tokio::test)]
  async fn fetch_link_supports_broker_urls_with_context_paths() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource from a base URL with a context path", "", |mut i| {
        i.request.path("/path");
        i.response
          .status(200)
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{\"document\":{\"href\":\"/path/doc/abc\",\"templated\":false}}}");
        i
      })
      .interaction("a request for a document from a base URL with a context path", "", |mut i| {
        i.request.path("/path/doc/abc");
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!("Yay! You found your way here"));
        i
      })
      .start_mock_server(None, None);

    let mut client = HALClient::with_url(pact_broker.url().join("/path").unwrap().as_str(), None);
    let result = client.fetch("/path").await.unwrap();
    client.path_info = Some(result);
    let result = client.fetch_link("document", &hashmap!{}).await;
    expect!(result).to(be_ok().value(Value::String("Yay! You found your way here".to_string())));
  }

  #[test_log::test(tokio::test)]
  async fn fetch_link_supports_broker_urls_with_context_paths_with_absolute_links() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource from a base URL with a context path", "", |mut i| {
        i.request.path("/path");
        i.response
          .status(200)
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{\"document\":{\"href\":\"http://localhost/path/doc/abc\",\"templated\":false}}}");
        i
      })
      .interaction("a request for a document from a base URL with a context path", "", |mut i| {
        i.request.path("/path/doc/abc");
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!("Yay! You found your way here"));
        i
      })
      .start_mock_server(None, None);

    let mut client = HALClient::with_url(pact_broker.url().join("/path").unwrap().as_str(), None);
    let result = client.fetch("/path").await.unwrap();
    client.path_info = Some(result);
    let result = client.fetch_link("document", &hashmap!{}).await;
    expect!(result).to(be_ok().value(Value::String("Yay! You found your way here".to_string())));
  }

    #[test_log::test(tokio::test)]
    async fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", "", |mut i| async move {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
                i
            })
            .await
            .interaction("a request for a providers pacts", "", |mut i| async move {
                i.given("There are no pacts in the pact broker");
                i.request
                    .path("/pacts/provider/sad_provider/latest")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response.status(404);
                i
            })
            .await
            .start_mock_server(None, Some(MockServerConfig::with_keep_alive(true)));

        let result = fetch_pacts_from_broker(pact_broker.url().as_str(),
                                             "sad_provider", None).await;
        match result {
          Ok(_) => {
            panic!("Expected an error result, but got OK");
          },
          Err(err) => {
            println!("err: {}", err);
            expect!(err.to_string().starts_with("Link/Resource was not found - No pacts for provider 'sad_provider' where found in the pact broker")).to(be_true());
          }
        }
    }

    #[test_log::test(tokio::test)]
    async fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
        let pact = RequestResponsePact { consumer: Consumer { name: "Consumer".to_string() },
            provider: Provider { name: "happy_provider".to_string() },
            .. RequestResponsePact::default() }
            .to_json(PactSpecification::V3).unwrap().to_string();
        let pact2 = RequestResponsePact { consumer: Consumer { name: "Consumer2".to_string() },
            provider: Provider { name: "happy_provider".to_string() },
            interactions: vec![ RequestResponseInteraction { description: "a request friends".to_string(), .. RequestResponseInteraction::default() } ],
            .. RequestResponsePact::default() }
            .to_json(PactSpecification::V3).unwrap().to_string();
        let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", "", |mut i| async move {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
                i
            })
            .await
            .interaction("a request for a providers pacts", "", |mut i| async move {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/latest")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links":{
                            "pacts":[
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0"},
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0"}
                            ]
                        }
                    }));
                i
            })
            .await
            .interaction("a request for the first provider pact", "", |mut i| async move {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact.clone());
                i
            })
            .await
            .interaction("a request for the second provider pact", "", |mut i| async move {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact2.clone());
                i
            })
            .await
            .start_mock_server(None, Some(MockServerConfig::with_keep_alive(true)));

        let result = fetch_pacts_from_broker(pact_broker.url().as_str(),
          "happy_provider", None).await;
        match &result {
          Ok(_) => (),
          Err(err) => panic!("Expected an Ok result, got a error {}", err)
        }
        let pacts = &result.unwrap();
        expect!(pacts.len()).to(be_equal_to(2));
        for pact in pacts {
          match pact {
            Ok(_) => (),
            Err(err) => panic!("Expected an Ok result, got a error {}", err)
          }
        }
    }

    #[test_log::test(tokio::test)]
    async fn fetch_pacts_for_verification_from_broker_returns_a_list_of_pacts() {
      let pact = RequestResponsePact { consumer: Consumer { name: "Consumer".to_string() },
        provider: Provider { name: "happy_provider".to_string() },
        .. RequestResponsePact::default() }
        .to_json(PactSpecification::V3).unwrap().to_string();

      let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
          .interaction("a request to the pact broker root", "", |mut i| async move {
            i.given("Pacts for verification is enabled");
            i.request
              .path("/")
              .header("Accept", "application/hal+json")
              .header("Accept", "application/json");
            i.response
              .header("Content-Type", "application/hal+json")
              .json_body(json_pattern!({
                  "_links": {
                    "pb:provider-pacts-for-verification": {
                      "href": like!("http://localhost/pacts/provider/{provider}/for-verification"),
                      "title": like!("Pact versions to be verified for the specified provider"),
                      "templated": like!(true)
                    }
                  }
              }));
            i
          })
          .await
          .interaction("a request to the pacts for verification endpoint", "", |mut i| async move {
            i.given("There are pacts to be verified");
            i.request
              .get()
              .path("/pacts/provider/happy_provider/for-verification")
              .header("Accept", "application/hal+json")
              .header("Accept", "application/json");
            i.response
              .header("Content-Type", "application/hal+json")
              .json_body(json_pattern!({
                "_links": {
                    "self": {
                      "href": like!("http://localhost/pacts/provider/happy_provider/for-verification"),
                      "title": like!("Pacts to be verified")
                    }
                }
            }));
            i
          })
          .await
          .interaction("a request to fetch pacts to be verified", "", |mut i| async move {
            i.given("There are pacts to be verified");
            i.request
              .post()
              .path("/pacts/provider/happy_provider/for-verification")
              .header("Accept", "application/hal+json")
              .header("Accept", "application/json")
              .json_body(json_pattern!({
                "consumerVersionSelectors": each_like!({
                    "tag": "prod"
                }),
                "providerVersionTags": each_like!("master"),
                "includePendingStatus": like!(false),
                "providerVersionBranch": like!("main")
              }));
            i.response
              .header("Content-Type", "application/hal+json")
              .json_body(json_pattern!({
                "_embedded": {
                  "pacts": each_like!({
                    "shortDescription": "latest prod",
                    "verificationProperties": {
                      "pending": false,
                      "notices": [
                        {
                          "when": "before_verification",
                          "text": "The pact at http://localhost/pacts/provider/happy_provider/consumer/Consumer/pact-version/12345678 is being verified because it matches the following configured selection criterion: latest pact for a consumer version tagged 'prod'"
                        },
                        {
                          "when": "before_verification",
                          "text": "This pact has previously been successfully verified by a version of happy_provider with tag 'master'. If this verification fails, it will fail the build. Read more at https://pact.io/pending"
                        }
                      ]
                    },
                    "_links": {
                      "self": {
                        "href": "http://localhost/pacts/provider/happy_provider/consumer/Consumer/pact-version/12345678",
                        "name": "Pact between Consumer (239aa5048a7de54fe5f231116c6d603eab0c6fde) and happy_provider"
                      }
                    }
                  })
                },
                "_links": {
                  "self": {
                    "href": like!("http://localhost/pacts/provider/happy_provider/for-verification"),
                    "title":like!("Pacts to be verified")
                  }
                }
              }));
            i
        })
        .await
        .interaction("a request for a pact by version", "", |mut i| async move {
          i.given("There is a pact with version 12345678");
          i.request
            .path("/pacts/provider/happy_provider/consumer/Consumer/pact-version/12345678")
            .header("Accept", "application/hal+json")
            .header("Accept", "application/json");
          i.response
            .header("Content-Type", "application/json")
            .body(pact.clone());
          i
      })
      .await
      .start_mock_server(None, None);

      let result = fetch_pacts_dynamically_from_broker(pact_broker.url().as_str(), "happy_provider".to_string(), false, None, vec!("master".to_string()), Some("main".to_string()), vec!(ConsumerVersionSelector {
        consumer: None,
        tag: Some("prod".to_string()),
        fallback_tag: None,
        latest: None,
        branch: None,
        deployed_or_released: None,
        deployed: None,
        released: None,
        main_branch: None,
        matching_branch: None,
        environment: None,
        fallback_branch: None,
      }), None).await;

      match &result {
        Ok(_) => (),
        Err(err) => panic!("Expected an Ok result, got a error {}", err)
      }

      let pacts = &result.unwrap();
      expect!(pacts.len()).to(be_equal_to(1));

      for pact in pacts {
        match pact {
          Ok(_) => (),
          Err(err) => panic!("Expected an Ok result, got a error {}", err)
        }
      }
  }

  #[test_log::test(tokio::test)]
  async fn fetch_pacts_for_verification_from_broker_returns_empty_list_if_there_are_no_pacts() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
      .interaction("a request to the pact broker root", "", |mut i| async move {
        i.given("Pacts for verification is enabled");
        i.request
          .path("/")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json");
        i.response
          .header("Content-Type", "application/hal+json")
          .json_body(json_pattern!({
              "_links": {
                "pb:provider-pacts-for-verification": {
                  "href": like!("http://localhost/pacts/provider/{provider}/for-verification"),
                  "title": like!("Pact versions to be verified for the specified provider"),
                  "templated": like!(true)
                }
              }
          }));
        i
      })
      .await
      .interaction("a request to the pacts for verification endpoint", "", |mut i| async move {
        i.request
          .get()
          .path("/pacts/provider/sad_provider/for-verification")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json");
        i.response
          .header("Content-Type", "application/hal+json")
          .json_body(json_pattern!({
            "_links": {
                "self": {
                  "href": like!("http://localhost/pacts/provider/sad_provider/for-verification"),
                  "title": like!("Pacts to be verified")
                }
            }
        }));
        i
      })
      .await
      .interaction("a request to fetch pacts to be verified", "", |mut i| async move {
        i.given("There are no pacts to be verified");
        i.request
          .post()
          .path("/pacts/provider/sad_provider/for-verification")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json")
          .json_body(json_pattern!({
            "consumerVersionSelectors": each_like!({
                "tag": "prod"
            }),
            "providerVersionTags": each_like!("master"),
            "includePendingStatus": like!(false),
            "providerVersionBranch": like!("main")
          }));
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!({
              "_embedded": {
                "pacts": []
              }
            }));
        i
      })
    .await
    .start_mock_server(None, None);

    let result = fetch_pacts_dynamically_from_broker(pact_broker.url().as_str(), "sad_provider".to_string(), false, None, vec!("master".to_string()), Some("main".to_string()), vec!(ConsumerVersionSelector {
      consumer: None,
      tag: Some("prod".to_string()),
      fallback_tag: None,
      latest: None,
      branch: None,
      deployed_or_released: None,
      deployed: None,
      released: None,
      main_branch: None,
      matching_branch: None,
      environment: None,
      fallback_branch: None,
    }), None).await;

    match result {
      Ok(_) => {
        panic!("Expected an error result, but got OK");
      },
      Err(err) => {
        println!("err: {}", err);
        expect!(err.to_string().starts_with("Link/Resource was not found - No pacts were found for this provider")).to(be_true());
      }
    }
  }

  #[test_log::test(tokio::test)]
  async fn fetch_pacts_for_verification_handles_validation_errors() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
      .interaction("a request to the pact broker root", "", |mut i| async move {
        i.given("Pacts for verification is enabled");
        i.request
          .path("/")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json");
        i.response
          .header("Content-Type", "application/hal+json")
          .json_body(json_pattern!({
              "_links": {
                "pb:provider-pacts-for-verification": {
                  "href": like!("http://localhost/pacts/provider/{provider}/for-verification"),
                  "title": like!("Pact versions to be verified for the specified provider"),
                  "templated": like!(true)
                }
              }
          }));
        i
      })
      .await
      .interaction("a request to the pacts for verification endpoint", "", |mut i| async move {
        i.request
          .get()
          .path("/pacts/provider/sad_provider/for-verification")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json");
        i.response
          .header("Content-Type", "application/hal+json")
          .json_body(json_pattern!({
            "_links": {
                "self": {
                  "href": like!("http://localhost/pacts/provider/sad_provider/for-verification"),
                  "title": like!("Pacts to be verified")
                }
            }
        }));
        i
      })
      .await
      .interaction("a request to fetch pacts to be verified", "", |mut i| async move {
        i.request
          .post()
          .path("/pacts/provider/sad_provider/for-verification")
          .header("Accept", "application/hal+json")
          .header("Accept", "application/json")
          .json_body(json_pattern!({
            "consumerVersionSelectors": each_like!({
                "tag": "prod"
            }),
            "includePendingStatus": like!(true)
          }));
        i.response
          .status(400)
          .content_type("application/json")
          .json_body(json_pattern!({
              "errors": {
                "providerVersionBranch": [
                  "when pending or WIP pacts are enabled and there are no tags provided, the provider version branch must not be an empty string, as it is used in the calculations for WIP/pending. A value must be provided (recommended), or it must not be set at all."
                ]
              }
            }));
        i
      })
      .await
      .start_mock_server(None, None);

    let result = fetch_pacts_dynamically_from_broker(
      pact_broker.url().as_str(),
      "sad_provider".to_string(),
      false,
      None,
      vec![],
      None,
      vec!(ConsumerVersionSelector {
        consumer: None,
        tag: Some("prod".to_string()),
        fallback_tag: None,
        latest: None,
        branch: None,
        deployed_or_released: None,
        deployed: None,
        released: None,
        main_branch: None,
        matching_branch: None,
        environment: None,
        fallback_branch: None,
      }),
      None
    ).await;

    match result {
      Ok(_) => panic!("Expected an error result, but got OK"),
      Err(err) => {
        println!("err: {}", err);
        expect!(err.to_string()).to(be_equal_to(
          "failed validation - [\"providerVersionBranch: when pending or WIP pacts are enabled and there are no tags provided, the provider version branch must not be an empty string, as it is used in the calculations for WIP/pending. A value must be provided (recommended), or it must not be set at all.\"]"
        ));
      }
    }
  }

  #[test]
  fn test_build_payload_with_success() {
    let result = TestResult::Ok(vec![]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": true,
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_adds_the_build_url_if_provided() {
    let result = TestResult::Ok(vec![]);
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()), None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": true,
      "buildUrl": "http://build-url",
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_adds_a_result_for_each_interaction() {
    let result = TestResult::Ok(vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string()), None]);
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()), None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": true,
      "buildUrl": "http://build-url",
      "testResults": [
        { "interactionId": "1", "success": true },
        { "interactionId": "2", "success": true },
        { "interactionId": "3", "success": true }
      ],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_with_failure() {
    let result = TestResult::Failed(vec![]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_with_failure_with_mismatches() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), None, Some(MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Box::new(RequestResponseInteraction::default()),
        actual: Box::new(RequestResponseInteraction::default()),
        interaction_id: Some("1234abc".to_string())
      }))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "interactionId": "1234abc",
          "mismatches": [
            {
              "attribute": "method", "description": "Expected method of PUT but received POST"
            }
          ],
          "success": false
        }
      ],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_with_failure_with_exception() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), None, Some(MismatchResult::Error("Bang".to_string(), Some("1234abc".to_string()))))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "exceptions": [
            {
              "message": "Bang"
            }
          ],
          "interactionId": "1234abc",
          "success": false
        }
      ],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_with_mixed_results() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), None, Some(MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Box::new(RequestResponseInteraction::default()),
        actual: Box::new(RequestResponseInteraction::default()),
        interaction_id: Some("1234abc".to_string())
      })),
      (Some("12345678".to_string()), None, Some(MismatchResult::Error("Bang".to_string(), Some("1234abc".to_string())))),
      (Some("abc123".to_string()), None, None)
    ]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "interactionId": "1234abc",
          "mismatches": [
            {
              "attribute": "method", "description": "Expected method of PUT but received POST"
            }
          ],
          "success": false
        },
        {
          "exceptions": [
            {
              "message": "Bang"
            }
          ],
          "interactionId": "12345678",
          "success": false
        },
        {
          "interactionId": "abc123",
          "success": true
        }
      ],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn test_build_payload_adds_client_language_version_if_provided() {
    let result = TestResult::Ok(vec![]);
    let metrics = VerificationMetrics {
      test_framework: "TEST".to_string(),
      app_name: "TESTER".to_string(),
      app_version: "1.2.3".to_string()
    };
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()), Some(&metrics));
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": true,
      "buildUrl": "http://build-url",
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION,
        "clientLanguage": {
          "testFramework": "TEST",
          "name": "TESTER",
          "version": "1.2.3"
        }
      }
    }));
  }

  #[test]
  fn test_build_payload_with_failure_adds_the_interaction_desc_if_set() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), Some("int_desc".to_string()), Some(MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Box::new(RequestResponseInteraction::default()),
        actual: Box::new(RequestResponseInteraction::default()),
        interaction_id: Some("1234abc".to_string())
      }))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None, None);
    assert_eq!(payload, json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "interactionId": "1234abc",
          "interactionDescription": "int_desc",
          "mismatches": [
            {
              "attribute": "method", "description": "Expected method of PUT but received POST"
            }
          ],
          "success": false
        }
      ],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": VERIFIER_VERSION
      }
    }));
  }

  #[test]
  fn build_link_from_json() {
    let json = json!({
      "href": "localhost"
    });
    let link = Link::from_json(&"link name".to_string(), json.as_object().unwrap());
    expect!(link.name).to(be_equal_to("link name"));
    expect!(link.href).to(be_some().value("localhost"));
    expect!(link.templated).to(be_false());

    let json2 = json!({
      "templated": true
    });
    let link2 = Link::from_json(&"link name".to_string(), json2.as_object().unwrap());
    expect!(link2.name).to(be_equal_to("link name"));
    expect!(link2.href).to(be_none());
    expect!(link2.templated).to(be_true());
  }

  #[test]
  fn build_json_from_link() {
    let link = Link {
      name: "Link Name".to_string(),
      href: Some("1234".to_string()),
      templated: true,
      title: None
    };
    let json = link.as_json();
    expect!(json.to_string()).to(be_equal_to(
      "{\"href\":\"1234\",\"templated\":true}"));

    let link = Link {
      name: "Link Name".to_string(),
      href: Some("1234".to_string()),
      templated: true,
      title: Some("title".to_string())
    };
    let json = link.as_json();
    expect!(json.to_string()).to(be_equal_to(
      "{\"href\":\"1234\",\"templated\":true,\"title\":\"title\"}"));
  }

  #[test_log::test(tokio::test)]
  async fn publish_provider_branch_with_normal_broker_source() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBroker")
      .interaction("a request to the pact broker pacticipant", "", |mut i| async move {
        i.request
          .path("/pacticipants/Pact%20Broker");
        i.response
          .header("Content-Type", "application/hal+json")
          .json_body(json_pattern!({
            "name": "Pact Broker",
            "displayName": "Pact Broker",
            "updatedAt": "2019-05-04T06:20:15+00:00",
            "createdAt": "2019-05-04T06:20:15+00:00",
            "_embedded": {
              "labels": []
            },
            "_links": {
              "self": {
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker"
              },
              "pb:versions": {
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions"
              },
              "pb:version": {
                "title": "Get, create or delete a pacticipant version",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions/{version}",
                "templated": true
              },
              "pb:version-tag": {
                "title": "Get, create or delete a tag for a version of Pact Broker",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions/{version}/tags/{tag}",
                "templated": true
              },
              "pb:branch-version": {
                "title": "Get or add/create a version for a branch of Pact Broker",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/branches/{branch}/versions/{version}",
                "templated": true
              },
              "pb:label": {
                "title": "Get, create or delete a label for Pact Broker",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/labels/{label}",
                "templated": true
              },
              "versions": {
                "title": "Deprecated - use pb:versions",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/versions"
              },
              "pb:can-i-deploy-badge": {
                "title": "Can I Deploy Pact Broker badge",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/latest-version/{tag}/can-i-deploy/to/{environmentTag}/badge",
                "templated": true
              },
              "pb:can-i-deploy-branch-to-environment-badge": {
                "title": "Can I Deploy Pact Broker from branch to environment badge",
                "href": "https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker/branches/{branch}/latest-version/can-i-deploy/to-environment/{environment}/badge",
                "templated": true
              },
              "curies": [
                {
                  "name": "pb",
                  "href": "https://pact-foundation.pactflow.io/doc/{rel}?context=pacticipant",
                  "templated": true
                }
              ]
            }
          }));
        i
      })
      .await
      .interaction("a request to publish the provider branch", "", |mut i| async move {
        i.request
          .method("PUT")
          .path("/pacticipants/Pact%20Broker/branches/feat%2F1234/versions/1234")
          .json_body(json!({}));
        i.response
          .status(200)
          .content_type("application/json")
          .json_body(json!({}));
        i
      })
      .await
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let links = vec![
      Link {
        name: "pb:provider".to_string(),
        href: Some("https://pact-foundation.pactflow.io/pacticipants/Pact%20Broker".to_string()),
        templated: false,
        title: Some("Provider".to_string())
      }
    ];
    let result = publish_provider_branch(&client, &links, "feat/1234", "1234").await;
    expect!(result).to(be_ok());
  }

  #[test_log::test(tokio::test)]
  async fn send_document_supports_broker_urls_with_context_paths() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to send a document from a base URL with a context path", "", |mut i| {
        i.request
          .method("PUT")
          .path("/path/a/b/c")
          .header("Content-Type", "application/json")
          .body("{}");
        i.response
          .status(200)
          .header("Content-Type", "application/json")
          .body("{}");
        i
      })
      .start_mock_server(None, None);

    let client = HALClient::with_url(pact_broker.url().join("/path").unwrap().as_str(), None);
    let result = client.send_document("/path/a/b/c", "{}", Method::PUT).await;
    expect!(result).to(be_ok());
  }

  #[test]
  fn resolve_path_test() {
    let client = HALClient::with_url("not a URL", None);
    expect!(client.resolve_path("/any")).to(be_err());

    let client = HALClient::with_url("http://localhost-ip4:1234", None);
    expect!(client.resolve_path("")).to(be_ok().value(Url::parse("http://localhost-ip4:1234").unwrap()));
    expect!(client.resolve_path("/")).to(be_ok().value(Url::parse("http://localhost-ip4:1234").unwrap()));
    expect!(client.resolve_path("/any")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/any").unwrap()));
    expect!(client.resolve_path("any")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/any").unwrap()));
    expect!(client.resolve_path("any/sub-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/any/sub-path").unwrap()));
    expect!(client.resolve_path("/base-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path").unwrap()));
    expect!(client.resolve_path("/base-path/")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/").unwrap()));
    expect!(client.resolve_path("/base-path/sub-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/sub-path").unwrap()));

    let client = HALClient::with_url("http://localhost-ip4:1234/base-path", None);
    expect!(client.resolve_path("")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path").unwrap()));
    expect!(client.resolve_path("/")).to(be_ok().value(Url::parse("http://localhost-ip4:1234").unwrap()));
    expect!(client.resolve_path("/any")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/any").unwrap()));
    expect!(client.resolve_path("any")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/any").unwrap()));
    expect!(client.resolve_path("any/sub-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/any/sub-path").unwrap()));
    expect!(client.resolve_path("/base-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path").unwrap()));
    expect!(client.resolve_path("/base-path/")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/").unwrap()));
    expect!(client.resolve_path("/base-path/sub-path")).to(be_ok().value(Url::parse("http://localhost-ip4:1234/base-path/sub-path").unwrap()));
  }

  #[test_log::test(tokio::test)]
  async fn navigate_first_retrieves_the_root_resource() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource", "", |mut i| async move {
        i.request.path("/");
        i.response
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
        i
      })
      .await
      .interaction("a request to next", "", |mut i| async move {
        i.request.path("/abc");
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!("Yay! You found your way here"));
        i
      })
      .await
      .start_mock_server(None);

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.navigate("next", &hashmap!{}).await.unwrap();
    expect!(result.path_info).to(be_some().value(serde_json::Value::String("Yay! You found your way here".to_string())));
  }

  #[test_log::test(tokio::test)]
  async fn navigate_will_not_retrieve_the_root_resource_if_a_path_is_already_set() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to next", "", |mut i| async move {
        i.request.path("/abc");
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!("Yay! You found your way here"));
        i
      })
      .await
      .start_mock_server(None);

    let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
    client.path_info = Some(json!({
      "_links": {
        "next": { "href": "/abc" },
        "prev": { "href": "/def" }
      }
    }));
    let result = client.navigate("next", &hashmap!{}).await.unwrap();
    expect!(result.path_info).to(be_some().value(serde_json::Value::String("Yay! You found your way here".to_string())));
  }

  #[test_log::test(tokio::test)]
  async fn navigate_takes_context_paths_into_account() {
    let pact_broker = PactBuilderAsync::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource with base path", "", |mut i| async move {
        i.request.path("/base-path");
        i.response
          .header("Content-Type", "application/hal+json")
          .body("{\"_links\":{\"next\":{\"href\":\"/base-path/abc\"},\"prev\":{\"href\":\"/base-path/def\"}}}");
        i
      })
      .await
      .interaction("a request to next with a base path", "", |mut i| async move {
        i.request.path("/base-path/abc");
        i.response
          .header("Content-Type", "application/json")
          .json_body(json_pattern!("Yay! You found your way here"));
        i
      })
      .await
      .start_mock_server(None);

    let client = HALClient::with_url(pact_broker.url().join("/base-path").unwrap().as_str(), None);
    let result = client.navigate("next", &hashmap!{}).await.unwrap();
    expect!(result.path_info).to(be_some().value(serde_json::Value::String("Yay! You found your way here".to_string())));
  }
}
