//! Module for fetching documents via HTTP

use std::fmt::{Display, Formatter};

use anyhow::anyhow;
use reqwest::blocking::Client;
use reqwest::Error;
use serde_json::Value;

/// Type of authentication to use
#[derive(Debug, Clone)]
pub enum HttpAuth {
  /// Username and Password
  User(String, Option<String>),
  /// Bearer token
  Token(String),
  /// No authentication
  None
}

impl HttpAuth {
  /// If no authentication is set
  pub fn is_none(&self) -> bool {
    match self {
      HttpAuth::None => true,
      _ => false
    }
  }
}

/// Fetches the JSON from a URL
pub fn fetch_json_from_url(url: &String, auth: &Option<HttpAuth>) -> anyhow::Result<(String, Value)> {
  let client = Client::builder()
    .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
    .build()
    .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;
  let request = match auth {
    &Some(ref auth) => {
      match auth {
        &HttpAuth::User(ref username, ref password) => client.get(url).basic_auth(username.clone(), password.clone()),
        &HttpAuth::Token(ref token) => client.get(url).bearer_auth(token.clone()),
        _ => client.get(url)
      }
    },
    &None => client.get(url)
  };

  match request.send() {
    Ok(res) => if res.status().is_success() {
      let pact_json: Result<Value, Error> = res.json();
      match pact_json {
        Ok(ref json) => Ok((url.clone(), json.clone())),
        Err(err) => Err(anyhow!("Failed to parse JSON - {}", err))
      }
    } else {
      Err(anyhow!("Request failed with status - {}", res.status()))
    },
    Err(err) => Err(anyhow!("Request failed - {}", err))
  }
}

impl Display for HttpAuth {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match *self {
      HttpAuth::Token(ref t) =>
        write!(f, "Token({:*<width$})", t.get(0..4).unwrap_or(""), width = t.len()),
      HttpAuth::User(ref u, ref p) => {
        let username = if u.is_empty() { "none" } else { u.as_str() };
        if let Some(pass) = p {
          write!(f, "User({}, {:*<width$})", username, pass.get(0..4).unwrap_or(""), width = pass.len())
        } else {
          write!(f, "User({}, [no password])", username)
        }
      }
      _ => write!(f, "None")
    }
  }
}

impl Default for HttpAuth {
  fn default() -> Self {
    HttpAuth::User("".to_string(), None)
  }
}
