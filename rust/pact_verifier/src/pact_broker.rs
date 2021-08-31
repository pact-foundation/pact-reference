//! Structs and functions for interacting with a Pact Broker

use std::collections::HashMap;

use anyhow::anyhow;
use futures::stream::*;
use itertools::Itertools;
use log::*;
use maplit::*;
use regex::{Captures, Regex};
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use tokio::time::{Duration, sleep};

use pact_matching::Mismatch;
use pact_models::http_utils::HttpAuth;
use pact_models::message_pact::MessagePact;
use pact_models::pact::Pact;
use pact_models::PACT_RUST_VERSION;
use pact_models::sync_pact::RequestResponsePact;

use crate::MismatchResult;

use super::provider_client::join_paths;

fn is_true(object: &serde_json::Map<String, serde_json::Value>, field: &str) -> bool {
    match object.get(field) {
        Some(json) => match *json {
            serde_json::Value::Bool(b) => b,
            _ => false
        },
        None => false
    }
}

fn as_string(json: &serde_json::Value) -> String {
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

fn find_entry(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<(String, serde_json::Value)> {
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
  UrlError(String)
}

impl PartialEq<String> for PactBrokerError {
    fn eq(&self, other: &String) -> bool {
        let message = match *self {
            PactBrokerError::LinkError(ref s) => s.clone(),
            PactBrokerError::ContentError(ref s) => s.clone(),
            PactBrokerError::IoError(ref s) => s.clone(),
            PactBrokerError::NotFound(ref s) => s.clone(),
            PactBrokerError::UrlError(ref s) => s.clone()
        };
        message == *other
    }
}

impl <'a> PartialEq<&'a str> for PactBrokerError {
    fn eq(&self, other: &&str) -> bool {
        let message = match *self {
            PactBrokerError::LinkError(ref s) => s.clone(),
            PactBrokerError::ContentError(ref s) => s.clone(),
            PactBrokerError::IoError(ref s) => s.clone(),
            PactBrokerError::NotFound(ref s) => s.clone(),
            PactBrokerError::UrlError(ref s) => s.clone()
        };
        message.as_str() == *other
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
  path_info: Option<serde_json::Value>,
  auth: Option<HttpAuth>,
  retries: u8
}

impl HALClient {
  /// Initialise a client with the URL and authentication
  pub fn with_url(url: &str, auth: Option<HttpAuth>) -> HALClient {
    HALClient { url: url.to_string(), auth, ..HALClient::default() }
  }

  fn update_path_info(self, path_info: serde_json::Value) -> HALClient {
    HALClient {
      client: self.client.clone(),
      url: self.url.clone(),
      path_info: Some(path_info),
      auth: self.auth,
      retries: self.retries
    }
  }

  /// Navigate to the resource from the link name
  pub async fn navigate(
    self,
    link: &'static str,
    template_values: &HashMap<String, String>
  ) -> Result<HALClient, PactBrokerError> {
    trace!("navigate(link='{}', template_values={:?})", link, template_values);

    let client = if self.path_info.is_none() {
      let path_info = self.clone().fetch("/".into()).await?;
      self.update_path_info(path_info)
    } else {
      self
    };

    let path_info = client.clone().fetch_link(link, template_values).await?;
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
        self,
        link: &'static str,
        template_values: &HashMap<String, String>
    ) -> Result<serde_json::Value, PactBrokerError> {
      trace!("fetch_link(link='{}', template_values={:?})", link, template_values);

        let link_data = self.find_link(link)?;

        self.fetch_url(&link_data, template_values).await
    }

  /// Fetch the resource at the Link from the Pact broker
  pub async fn fetch_url(
    self,
    link: &Link,
    template_values: &HashMap<String, String>
  ) -> Result<serde_json::Value, PactBrokerError> {
    trace!("fetch_url(link={:?}, template_values={:?})", link, template_values);

      let link_url = if link.templated {
          log::trace!("Link URL is templated");
          self.clone().parse_link_url(&link, &template_values)
      } else {
          link.href.clone()
              .ok_or_else(|| PactBrokerError::LinkError(
                  format!("Link is malformed, there is no href. URL: '{}', LINK: '{}'",
                      self.url, link.name
                  )
              ))
      }?;

      let base_url = self.url.parse::<reqwest::Url>()
          .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

      let joined_url = base_url.join(&link_url)
          .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

      self.fetch(joined_url.path().into()).await
  }

  async fn fetch(self, path: &str) -> Result<serde_json::Value, PactBrokerError> {
    info!("Fetching path '{}' from pact broker", path);

    let url = join_paths(&self.url, path).parse::<reqwest::Url>()
        .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

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

    async fn parse_broker_response(
        &self,
        path: String,
        response: reqwest::Response,
    ) -> Result<serde_json::Value, PactBrokerError> {
        let is_json_content_type = json_content_type(&response);
        let content_type = content_type(&response);

        if response.status().is_success() {
            let body = response.bytes()
                .await
                .map_err(|_| PactBrokerError::IoError(
                    format!("Failed to download response body for path '{}'. URL: '{}'", &path, self.url)
                ))?;

            if is_json_content_type {
                serde_json::from_slice(&body)
                    .map_err(|err| PactBrokerError::ContentError(
                        format!("Did not get a valid HAL response body from pact broker path '{}' - {}. URL: '{}'",
                            path, err, self.url)
                    ))
            } else {
                Err(PactBrokerError::ContentError(
                    format!("Did not get a HAL response from pact broker path '{}', content type is '{}'. URL: '{}'",
                        path, content_type, self.url
                    )
                ))
            }
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            Err(PactBrokerError::NotFound(
                format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                    response.status(), self.url
                )
            ))
        } else {
            Err(PactBrokerError::IoError(
                format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                    response.status(), self.url
                )
            ))
        }
    }

    fn parse_link_url(self, link: &Link, values: &HashMap<String, String>) -> Result<String, PactBrokerError> {
        match link.href {
            Some(ref href) => {
                log::trace!("templated URL = {}", href);
                let re = Regex::new(r"\{(\w+)\}").unwrap();
                let final_url = re.replace_all(href, |caps: &Captures| {
                    let lookup = caps.get(1).unwrap().as_str();
                    log::trace!("Looking up value for key '{}'", lookup);
                    match values.get(lookup) {
                        Some(val) => val.clone(),
                        None => {
                            log::error!("No value was found for key '{}', mapped values are {:?}. This is probably a bug in the pact broker or pact verifier.",
                                lookup, values);
                            format!("{{{}}}", lookup)
                        }
                    }
                });
                log::trace!("final URL = {}", final_url);
                Ok(final_url.to_string())
            },
            None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{}'",
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

  async fn send_document(&self, url: &str, body: &str, method: Method) -> Result<serde_json::Value, PactBrokerError> {
    debug!("Sending JSON to {} using {}: {}", url, method, body);

    let url = url.parse::<reqwest::Url>()
      .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

    let base_url = self.url.parse::<reqwest::Url>()
      .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

    let url = base_url.join(&url.path())
      .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

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

    let response = with_retries(self.retries, request_builder)
      .await
      .map_err(|err| PactBrokerError::IoError(
        format!("Failed to send JSON to the pact broker URL '{}' - {}", url, err)
      ))?
      .error_for_status()
      .map_err(|err| PactBrokerError::ContentError(
        format!("Request to pact broker URL '{}' failed - {}",  url, err)
      ));

      match response {
        Ok(res) => {
          let res = self.parse_broker_response(url.path().to_string(), res).await;
          Ok(res.unwrap_or_default())
        },
        Err(err) => Err(err)
      }
  }

  fn with_doc_context(self, doc_attributes: &[Link]) -> Result<HALClient, PactBrokerError> {
    let links: serde_json::Map<String, serde_json::Value> = doc_attributes.iter()
      .map(|link| (link.name.clone(), link.as_json())).collect();
    let links_json = json!({
      "_links": json!(links)
    });
    Ok(self.update_path_info(links_json))
  }
}

async fn with_retries(retries: u8, request: RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
  match &request.try_clone() {
    None => {
      warn!("with_retries: Could not retry the request as it is not cloneable");
      request.send().await
    }
    Some(rb) => {
      futures::stream::iter((1..=retries).step_by(1))
        .fold((None::<Result<reqwest::Response, reqwest::Error>>, rb.try_clone()), |(response, request), attempt| {
          async move {
            match request {
              Some(request_builder) => match response {
                None => {
                  let next = request_builder.try_clone();
                  (Some(request_builder.send().await), next)
                },
                Some(response) => {
                  trace!("with_retries: attempt {}/{} is {:?}", attempt, retries, response);
                  match response {
                    Ok(ref res) => if res.status().is_server_error() {
                      match request_builder.try_clone() {
                        None => (Some(response), None),
                        Some(rb) => {
                          sleep(Duration::from_millis(10_u64.pow(attempt as u32))).await;
                          (Some(request_builder.send().await), Some(rb))
                        }
                      }
                    } else {
                      (Some(response), None)
                    },
                    Err(ref err) => if err.is_status() {
                      if err.status().unwrap_or_default().is_server_error() {
                        match request_builder.try_clone() {
                          None => (Some(response), None),
                          Some(rb) => {
                            sleep(Duration::from_millis(10_u64.pow(attempt as u32))).await;
                            (Some(request_builder.send().await), Some(rb))
                          }
                        }
                      } else {
                        (Some(response), None)
                      }
                    } else {
                      (Some(response), None)
                    }
                  }
                }
              }
              None => (response, None)
            }
          }
        }).await.0.unwrap()
    }
  }
}

impl Default for HALClient {
  fn default() -> Self {
    HALClient {
      client: reqwest::ClientBuilder::new()
        .build()
        .unwrap(),
      url: "".to_string(),
      path_info: None,
      auth: None,
      retries: 3
    }
  }
}

fn links_from_json(json: &serde_json::Value) -> Vec<Link> {
   match json.get("_links") {
    Some(json) => match *json {
      serde_json::Value::Object(ref v) => {
        v.iter().map(|(link, json)| match *json {
          serde_json::Value::Object(ref attr) => Link::from_json(link, attr),
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
) -> anyhow::Result<Vec<anyhow::Result<(Box<dyn Pact + Send>, Option<PactVerificationContext>, Vec<Link>)>>> {
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
        .and_then(|(hal_client, pact_link)| async {
          let pact_json = hal_client.fetch_url(
            &pact_link.clone(),
            &template_values.clone()
          ).await?;
          Ok((pact_link, pact_json))
        })
        .map(|result| {
          match result {
            Ok((pact_link, pact_json)) => {
              let href = pact_link.href.unwrap_or_default();
              let links = links_from_json(&pact_json);
              match pact_json {
                Value::Object(ref map) => if map.contains_key("messages") {
                  MessagePact::from_json(&href, &pact_json)
                    .map(|pact| (pact.boxed(), None, links))
                } else {
                  RequestResponsePact::from_json(&href, &pact_json)
                    .map(|pact| (pact.boxed(), None, links))
                },
                _ => Err(anyhow!("Link '{}' does not point to a valid pact file", href))
              }
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
  consumer_version_selectors: Vec<ConsumerVersionSelector>,
  auth: Option<HttpAuth>
) -> Result<Vec<Result<(Box<dyn Pact + Send>, Option<PactVerificationContext>, Vec<Link>), PactBrokerError>>, PactBrokerError> {
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
            debug!("error Response for pacts for verification {:?} ", err);
            return Err(err)
          }
        }
      },
      Err(e) => return Err(e)
    };

    // Find all of the Pact links
    let pact_links = match response {
      Some(v) => {
        let pfv: PactsForVerificationResponse = serde_json::from_value(v).unwrap_or(PactsForVerificationResponse { embedded: PactsForVerificationBody { pacts: vec!() } });

        if pfv.embedded.pacts.len() == 0 {
          return Err(PactBrokerError::NotFound(format!("No pacts were found for this provider")))
        };

        let links: Result<Vec<(Link, PactVerificationContext)>, PactBrokerError> = pfv.embedded.pacts.iter().map(| p| {
          match p.links.get("self") {
            Some(l) => Ok((l.clone(), PactVerificationContext{
              short_description: p.short_description.clone(),
              verification_properties: PactVerificationProperties {
                pending: p.verification_properties.pending,
                notices: p.verification_properties.notices.clone(),
              }
            })),
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
      .and_then(|(hal_client, pact_link, context)| async {
        let pact_json = hal_client.fetch_url(
          &pact_link.clone(),
          &template_values.clone()
        ).await?;
        Ok((pact_link, pact_json, context))
      })
      .map(|result| {
        match result {
          Ok((pact_link, pact_json, context)) => {
            let href = pact_link.href.unwrap_or_default();
            let links = links_from_json(&pact_json);
            match pact_json {
              Value::Object(ref map) => if map.contains_key("messages") {
                match MessagePact::from_json(&href, &pact_json) {
                  Ok(pact) => Ok((pact.boxed(), Some(context), links)),
                  Err(err) => Err(PactBrokerError::ContentError(format!("{}", err)))
                }
              } else {
                match RequestResponsePact::from_json(&href, &pact_json) {
                  Ok(pact) => Ok((pact.boxed(), Some(context), links)),
                  Err(err) => Err(PactBrokerError::ContentError(format!("{}", err)))
                }
              },
              _ => Err(PactBrokerError::ContentError(format!("Link '{}' does not point to a valid pact file", href)))
            }
          },
          Err(err) => Err(err)
        }
      })
      .into_stream()
      .collect()
      .await;

    Ok(results)
}

/// Struct that wraps the result of a verification test
pub enum TestResult {
  /// Test was OK
  Ok(Vec<Option<String>>),
  /// Test failed verification
  Failed(Vec<(Option<String>, Option<MismatchResult>)>)
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
  provider_tags: Vec<String>
) -> Result<serde_json::Value, PactBrokerError> {
  let hal_client = HALClient::with_url(broker_url, auth.clone());

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

  let json = build_payload(result, version, build_url);
  hal_client.post_json(publish_link.href.unwrap_or_default().as_str(), json.to_string().as_str()).await
}

fn build_payload(result: TestResult, version: String, build_url: Option<String>) -> serde_json::Value {
  let mut json = json!({
    "success": result.to_bool(),
    "providerApplicationVersion": version,
    "verifiedBy": {
      "implementation": "Pact-Rust",
      "version": PACT_RUST_VERSION
    }
  });
  let json_obj = json.as_object_mut().unwrap();

  if build_url.is_some() {
    json_obj.insert("buildUrl".into(), json!(build_url.unwrap()));
  }

  match result {
    TestResult::Failed(mismatches) => {
      let values = mismatches.iter()
        .group_by(|(id, _)| id.clone().unwrap_or_default())
        .into_iter()
        .map(|(key, mismatches)| {
          let acc: (Vec<serde_json::Value>, Vec<serde_json::Value>) = (vec![], vec![]);
          let values = mismatches.fold(acc, |mut acc, (_, result)| {
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
            acc
          });

          let mut json = json!({
            "interactionId": key,
            "success": values.0.is_empty() && values.1.is_empty()
          });

          if !values.0.is_empty() {
            json.as_object_mut().unwrap().insert("mismatches".into(), json!(values.0));
          }

          if !values.1.is_empty() {
            json.as_object_mut().unwrap().insert("exceptions".into(), json!(values.1));
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Structure to represent a HAL link
pub struct ConsumerVersionSelector {
  /// Application name to filter the results on
  pub consumer: Option<String>,
  /// Tag
  pub tag: Option<String>,
  /// Fallback tag if Tag doesn't exist
  pub fallback_tag: Option<String>,
  /// Only select the latest (if false, this selects all pacts for a tag)
  pub latest: Option<bool>,
  /// Applications that have been deployed or released
  pub deployed_or_released: Option<bool>,
  /// Applications that have been deployed
  pub deployed: Option<bool>,
  /// Applications that have been released
  pub released: Option<bool>,
  /// Applications in a given environment
  pub environment: Option<String>,
  /// Applications with the default branch set in the broker
  pub main_branch: Option<bool>,
  /// Applications with the given branch
  pub branch: Option<String>,
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
  pub verification_properties: PactVerificationProperties,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Request to send to determine the pacts to verify
pub struct PactsForVerificationRequest {
  /// Provider tags to use for determining pending pacts (if enabled)
  pub provider_version_tags: Vec<String>,
  /// Enable pending pacts feature
  pub include_pending_status: bool,
  /// Find WIP pacts after given date
  pub include_wip_pacts_since: Option<String>,
  /// Detailed pact selection criteria , see https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/
  pub consumer_version_selectors: Vec<ConsumerVersionSelector>
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
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
  use env_logger::*;
  use expectest::expect;
  use expectest::prelude::*;

  use pact_consumer::*;
  use pact_consumer::prelude::*;
  use pact_matching::Mismatch::MethodMismatch;
  use pact_models::{Consumer, PactSpecification, Provider};
  use pact_models::sync_interaction::RequestResponseInteraction;

  use super::*;
  use super::{content_type, json_content_type};

  #[tokio::test]
  async fn fetch_returns_an_error_if_there_is_no_pact_broker() {
    let client = HALClient::with_url("http://idont.exist:6666", None);
    expect!(client.fetch("/").await).to(be_err());
  }

  #[tokio::test]
  async fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
        .interaction("a request to a non-existant path", |i| {
            i.given("the pact broker has a valid pact");
            i.request.path("/hello");
            i.response.status(404);
        })
        .start_mock_server();

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.fetch("/hello").await;
    expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
        pact_broker.url())));
  }

  #[tokio::test]
  async fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a non-json resource", |i| {
          i.request.path("/nonjson");
          i.response
              .header("Content-Type", "text/html")
              .body("<html></html>");
      })
      .start_mock_server();

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.fetch("/nonjson").await;
    expect!(result).to(be_err().value(format!("Did not get a HAL response from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
      pact_broker.url())));
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

    #[tokio::test]
    async fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal resource", |i| {
                i.request.path("/nonhal");
                i.response.header("Content-Type", "application/hal+json");
            })
            .interaction("a request to a non-hal resource 2", |i| {
                i.request.path("/nonhal2");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("<html>This is not JSON</html>");
            })
            .start_mock_server();

        let client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/nonhal").await;
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal\' - EOF while parsing a value at line 1 column 0. URL: '{}'",
            pact_broker.url())));

        let result = client.clone().fetch("/nonhal2").await;
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal2\' - expected value at line 1 column 1. URL: '{}'",
            pact_broker.url())));
    }

  #[tokio::test]
  async fn fetch_retries_the_request_on_50x_errors() {
    let _ = env_logger::try_init();
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a request to a hal resource", |i| {
        i.given("server returns a gateway error");
        i.request.path("/");
        i.response.status(503);
      })
      .start_mock_server();

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let expected_requests = client.retries as usize;
    let result = client.fetch("/").await;
    expect!(result).to(be_err());
    expect!(pact_broker.metrics().requests).to(be_equal_to(expected_requests ));
  }

  #[tokio::test]
  async fn post_json_retries_the_request_on_50x_errors() {
    let _ = env_logger::try_init();
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a POST request", |i| {
        i.given("server returns a gateway error");
        i.request.path("/").method("POST");
        i.response.status(503);
      })
      .start_mock_server();

    let client = HALClient::with_url(pact_broker.url().as_str(), None);
    let expected_requests = client.retries as usize;
    let result = client.post_json(pact_broker.url().as_str(), "{}").await;
    expect!(result.clone()).to(be_err());
    expect!(pact_broker.metrics().requests).to(be_equal_to(expected_requests ));
  }

  #[tokio::test]
  async fn put_json_retries_the_request_on_50x_errors() {
    let _ = env_logger::try_init();
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
      .interaction("a PUT request", |i| {
        i.given("server returns a gateway error");
        i.request.path("/").method("PUT");
        i.response.status(503);
      })
      .start_mock_server();

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
    expect!(client.clone().parse_link_url(&link, &values)).to(be_ok().value("http://localhost"));

    let link = Link { name: "link".to_string(), href: Some("http://{valA}/{valB}".to_string()), templated: false, title: None };
    expect!(client.clone().parse_link_url(&link, &values)).to(be_ok().value("http://A/B"));

    let link = Link { name: "link".to_string(), href: Some("http://{valA}/{valC}".to_string()), templated: false, title: None };
    expect!(client.clone().parse_link_url(&link, &values)).to(be_ok().value("http://A/{valC}"));
  }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_a_previous_resource_has_not_been_fetched() {
        let client = HALClient::with_url("http://localhost", None);
        let result = client.fetch_link("anything_will_do", &hashmap!{}).await;
        expect!(result).to(be_err().value("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'".to_string()));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
      try_init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("hal2", &hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
      try_init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("any", &hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

  #[tokio::test]
  async fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
        .interaction("a request to a hal resource", |i| {
            i.request.path("/");
            i.response
                .header("Content-Type", "application/hal+json")
                .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
        })
        .start_mock_server();

    let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
    let result = client.clone().fetch("/").await;
    expect!(result.clone()).to(be_ok());
    client.path_info = result.ok();
    let result = client.clone().fetch_link("any", &hashmap!{}).await;
    expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
        pact_broker.url())));
  }

    #[tokio::test]
    async fn fetch_link_returns_the_resource_for_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("next", &hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

    #[tokio::test]
    async fn fetch_link_returns_handles_absolute_resource_links() {
      try_init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with absolute paths", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"http://localhost/abc\"},\"prev\":{\"href\":\"http://localhost/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("next", &hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

    #[tokio::test]
    async fn fetch_link_returns_the_resource_for_the_templated_link() {
      try_init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a templated hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"document\":{\"href\":\"/doc/{id}\",\"templated\":true}}}");

            })
            .interaction("a request for a document", |i| {
                i.request.path("/doc/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().as_str(), None);
        let result = client.clone().fetch("/").await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("document", &hashmap!{ "id".to_string() => "abc".to_string() }).await;
        expect!(result).to(be_ok().value(serde_json::Value::String("Yay! You found your way here".to_string())));
    }

    #[tokio::test]
    async fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
      try_init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
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
            })
            .interaction("a request for a providers pacts", |i| {
                i.given("There are no pacts in the pact broker");
                i.request
                    .path("/pacts/provider/sad_provider/latest")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response.status(404);
            })
            .start_mock_server();

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

    #[tokio::test]
    async fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
      try_init().unwrap_or(());
        let pact = RequestResponsePact { consumer: Consumer { name: "Consumer".to_string() },
            provider: Provider { name: "happy_provider".to_string() },
            .. RequestResponsePact::default() }
            .to_json(PactSpecification::V3).unwrap().to_string();
        let pact2 = RequestResponsePact { consumer: Consumer { name: "Consumer2".to_string() },
            provider: Provider { name: "happy_provider".to_string() },
            interactions: vec![ RequestResponseInteraction { description: "a request friends".to_string(), .. RequestResponseInteraction::default() } ],
            .. RequestResponsePact::default() }
            .to_json(PactSpecification::V3).unwrap().to_string();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
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
            })
            .interaction("a request for a providers pacts", |i| {
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
            })
            .interaction("a request for the first provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact.clone());
            })
            .interaction("a request for the second provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0")
                    .header("Accept", "application/hal+json")
                    .header("Accept", "application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact2.clone());
            })
            .start_mock_server();

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

    #[tokio::test]
    async fn fetch_pacts_for_verification_from_broker_returns_a_list_of_pacts() {
      try_init().unwrap_or(());

      let pact = RequestResponsePact { consumer: Consumer { name: "Consumer".to_string() },
        provider: Provider { name: "happy_provider".to_string() },
        .. RequestResponsePact::default() }
        .to_json(PactSpecification::V3).unwrap().to_string();

      let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
          .interaction("a request to the pact broker root", |i| {
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
          })
          .interaction("a request to the pacts for verification endpoint", |i| {
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
          })
          .interaction("a request to fetch pacts to be verified", |i| {
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
        })
        .interaction("a request for a pact by version", |i| {
          i.given("There is a pact with version 12345678");
          i.request
            .path("/pacts/provider/happy_provider/consumer/Consumer/pact-version/12345678")
            .header("Accept", "application/hal+json")
            .header("Accept", "application/json");
          i.response
            .header("Content-Type", "application/json")
            .body(pact.clone());
      })
      .start_mock_server();

    let result = fetch_pacts_dynamically_from_broker(pact_broker.url().as_str(), "happy_provider".to_string(), false, None, vec!("master".to_string()), vec!(ConsumerVersionSelector {
      consumer: None,
      tag: Some("prod".to_string()),
      fallback_tag: None,
      latest: None,
      branch: None,
      deployed_or_released: None,
      deployed: None,
      released: None,
      main_branch: None,
      environment: None,
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

  #[tokio::test]
  async fn fetch_pacts_for_verification_from_broker_returns_empty_list_if_there_are_no_pacts() {
    try_init().unwrap_or(());

    let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
      .interaction("a request to the pact broker root", |i| {
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
      })
      .interaction("a request to the pacts for verification endpoint", |i| {
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
      })
      .interaction("a request to fetch pacts to be verified", |i| {
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
          }));
        i.response
          .json_body(json_pattern!({
              "_embedded": {
                "pacts": []
              }
            }));
      })
    .start_mock_server();

    let result = fetch_pacts_dynamically_from_broker(pact_broker.url().as_str(), "sad_provider".to_string(), false, None, vec!("master".to_string()), vec!(ConsumerVersionSelector {
      consumer: None,
      tag: Some("prod".to_string()),
      fallback_tag: None,
      latest: None,
      branch: None,
      deployed_or_released: None,
      deployed: None,
      released: None,
      main_branch: None,
      environment: None,
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

  #[test]
  fn test_build_payload_with_success() {
    let result = TestResult::Ok(vec![]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": true,
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_adds_the_build_url_if_provided() {
    let result = TestResult::Ok(vec![]);
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()));
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": true,
      "buildUrl": "http://build-url",
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_adds_a_result_for_each_interaction() {
    let result = TestResult::Ok(vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string()), None]);
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()));
    expect!(payload).to(be_equal_to(json!({
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
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_with_failure() {
    let result = TestResult::Failed(vec![]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [],
      "verifiedBy": {
        "implementation": "Pact-Rust",
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_with_failure_with_mismatches() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), Some(MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Box::new(RequestResponseInteraction::default()),
        actual: Box::new(RequestResponseInteraction::default()),
        interaction_id: Some("1234abc".to_string())
      }))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
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
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_with_failure_with_exception() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), Some(MismatchResult::Error("Bang".to_string(), Some("1234abc".to_string()))))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
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
        "version": PACT_RUST_VERSION
      }
    })));
  }

  #[test]
  fn test_build_payload_with_mixed_results() {
    let result = TestResult::Failed(vec![
      (Some("1234abc".to_string()), Some(MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Box::new(RequestResponseInteraction::default()),
        actual: Box::new(RequestResponseInteraction::default()),
        interaction_id: Some("1234abc".to_string())
      })),
      (Some("12345678".to_string()), Some(MismatchResult::Error("Bang".to_string(), Some("1234abc".to_string())))),
      (Some("abc123".to_string()), None)
    ]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
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
        "version": PACT_RUST_VERSION
      }
    })));
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
}
