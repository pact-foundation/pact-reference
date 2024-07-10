//! Exported verifier functions

use std::env;
use std::str;
use std::str::FromStr;
use std::sync::Arc;

use clap::{AppSettings, ArgMatches, ErrorKind};
use maplit::hashmap;
use serde_json::Value;
use pact_models::http_utils::HttpAuth;
use pact_models::PactSpecification;
use tracing::{debug, error, warn};
use tracing_core::LevelFilter;
use tracing_subscriber::FmtSubscriber;

use pact_verifier::*;
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;
use pact_verifier::metrics::VerificationMetrics;
use pact_verifier::selectors::{consumer_tags_to_selectors, json_to_selectors};

use super::args;

#[deprecated(since = "0.1.0-beta.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
fn pact_source(matches: &ArgMatches) -> Vec<PactSource> {
  let mut sources = vec![];

  if let Some(values) = matches.values_of("file") {
    sources.extend(values.map(|v| PactSource::File(v.into())).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("dir") {
    sources.extend(values.map(|v| PactSource::Dir(v.into())).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("url") {
    sources.extend(values.map(|v| {
      if matches.is_present("user") {
        PactSource::URL(v.into(), matches.value_of("user").map(|user| {
          HttpAuth::User(user.to_string(), matches.value_of("password").map(|p| p.to_string()))
        }))
      } else if matches.is_present("token") {
        PactSource::URL(v.into(), matches.value_of("token").map(|token| HttpAuth::Token(token.to_string())))
      } else {
        PactSource::URL(v.into(), None)
      }
    }).collect::<Vec<PactSource>>());
  };

  if let Some(broker_url) = matches.value_of("broker-url") {
    let name = matches.value_of("provider-name").map(|n| n.to_string()).unwrap_or_default();
    let auth = matches.value_of("user").map(|user| {
      HttpAuth::User(user.to_string(), matches.value_of("password").map(|p| p.to_string()))
    }).or_else(|| matches.value_of("token").map(|t| HttpAuth::Token(t.to_string())));

    let source = if matches.is_present("consumer-version-selectors") || matches.is_present("consumer-version-tags") {
      let pending = matches.is_present("enable-pending");
      let wip = matches.value_of("include-wip-pacts-since").map(|wip| wip.to_string());
      let provider_tags = matches.values_of("provider-tags")
        .map_or_else(Vec::new, |tags| tags.map(|tag| tag.to_string()).collect());
      let provider_branch = matches.value_of("provider-branch").map(|v| v.to_string());
      let selectors = if matches.is_present("consumer-version-selectors") {
        matches.values_of("consumer-version-selectors")
          .map_or_else(Vec::new, |s| json_to_selectors(s
            .map(|cvs| serde_json::from_str::<Value>(cvs))
            .flatten()
            .collect::<Vec<_>>()))
      } else if matches.is_present("consumer-version-tags") {
        matches.values_of("consumer-version-tags")
          .map_or_else(Vec::new, |tags| consumer_tags_to_selectors(tags.collect::<Vec<_>>()))
      } else {
        vec![]
      };

      PactSource::BrokerWithDynamicConfiguration {
        provider_name: name,
        broker_url: broker_url.into(),
        enable_pending: pending,
        include_wip_pacts_since: wip,
        provider_tags,
        provider_branch,
        selectors,
        auth,
        links: vec![]
      }
    } else {
      PactSource::BrokerUrl(name, broker_url.to_string(), auth, vec![])
    };
    sources.push(source);
  };
  sources
}

#[deprecated(since = "0.1.0-beta.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
fn interaction_filter(matches: &ArgMatches) -> FilterInfo {
  if matches.is_present("filter-description") &&
      (matches.is_present("filter-state") || matches.is_present("filter-no-state")) {
      if matches.is_present("filter-state") {
          FilterInfo::DescriptionAndState(matches.value_of("filter-description").unwrap().to_string(),
              matches.value_of("filter-state").unwrap().to_string())
      } else {
          FilterInfo::DescriptionAndState(matches.value_of("filter-description").unwrap().to_string(),
              String::new())
      }
  } else if matches.is_present("filter-description") {
      FilterInfo::Description(matches.value_of("filter-description").unwrap().to_string())
  } else if matches.is_present("filter-state") {
      FilterInfo::State(matches.value_of("filter-state").unwrap().to_string())
  } else if matches.is_present("filter-no-state") {
      FilterInfo::State(String::new())
  } else {
      FilterInfo::None
  }
}

/// Handles the command line arguments from the running process
/// Deprecated: This method is now deprecated and duplicated in the verifier CLI module. FFI consumers
/// should use the handle based interface instead.
#[deprecated(since = "0.1.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
pub async fn handle_cli(version: &str) -> Result<(), i32> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let app = args::setup_app(program, version);
  let matches = app
                  .setting(AppSettings::ArgRequiredElseHelp)
                  .setting(AppSettings::ColoredHelp)
                  .get_matches_safe();

  match matches {
    Ok(results) => {
      #[allow(deprecated)]
      handle_matches(&results).await
    },
    Err(ref err) => {
      match err.kind {
          ErrorKind::HelpDisplayed => {
              println!("{}", err.message);
              Ok(())
          },
          ErrorKind::VersionDisplayed => {
              print_version(version);
              println!();
              Ok(())
          },
          _ => {
              err.exit()
          }
      }
    }
  }
}

// Currently, clap prints things out as if it were a CLI call
#[allow(dead_code, missing_docs)]
/// Deprecated: This method is now deprecated and duplicated in the verifier CLI module. FFI consumers
/// should use the handle based interface instead.
#[deprecated(since = "0.1.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
pub async fn handle_args(args: Vec<String>) -> Result<(), i32> {
  let program = "pact_verifier_cli".to_string();
  let version = format!("v{}", clap::crate_version!()).as_str().to_owned();
  let app = args::setup_app(program, &version);
  let matches = app
                  .setting(AppSettings::NoBinaryName)
                  .setting(AppSettings::ColorNever)
                  .get_matches_from_safe(args);

  match matches {
    Ok(results) => {
      #[allow(deprecated)]
      handle_matches(&results).await
    },
    Err(ref err) => {
      log::error!("error verifying Pact: {:?} {:?}", err.message, err);
      Err(4)
    }
  }
}

#[deprecated(since = "0.1.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
async fn handle_matches(matches: &clap::ArgMatches<'_>) -> Result<(), i32> {
    let level = matches.value_of("loglevel").unwrap_or("warn");
    let log_level = match level {
        "none" => LevelFilter::OFF,
        _ => LevelFilter::from_str(level).unwrap()
    };

    let subscriber = FmtSubscriber::builder()
      .with_max_level(log_level)
      .with_thread_names(true)
      .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
      warn!("Failed to initialise global tracing subscriber - {err}");
    };

    #[allow(deprecated)]
    let provider = ProviderInfo {
      host: matches.value_of("hostname").unwrap_or("localhost").to_string(),
      port: matches.value_of("port").map(|port| port.parse::<u16>().unwrap()),
      path: matches.value_of("base-path").unwrap_or("/").into(),
      protocol: matches.value_of("scheme").unwrap_or("http").to_string(),
      .. ProviderInfo::default()
    };
    #[allow(deprecated)]
    let source = pact_source(matches);
    #[allow(deprecated)]
    let filter = interaction_filter(matches);
    let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
      state_change_url: matches.value_of("state-change-url").map(|s| s.to_string()),
      state_change_body: !matches.is_present("state-change-as-query"),
      state_change_teardown: matches.is_present("state-change-teardown"),
      .. HttpRequestProviderStateExecutor::default()
    });

    let mut custom_headers = hashmap!{};
    if let Some(headers) = matches.values_of("custom-header") {
      for header in headers {
        let (key, value) = header.split_once('=').ok_or_else(|| {
          error!("Custom header values must be in the form KEY=VALUE, where KEY and VALUE contain ASCII characters (32-127) only.");
          3
        })?;
        custom_headers.insert(key.to_string(), value.to_string());
      }
    }

    let verification_options = VerificationOptions {
      request_filter: None::<Arc<NullRequestFilterExecutor>>,
      disable_ssl_verification: matches.is_present("disable-ssl-verification"),
      request_timeout: matches.value_of("request-timeout")
        .map(|t| t.parse::<u64>().unwrap_or(5000)).unwrap_or(5000),
      custom_headers,
      .. VerificationOptions::default()
    };

    let publish_options = if matches.is_present("publish") {
      Some(PublishOptions {
        provider_version: matches.value_of("provider-version").map(|v| v.to_string()),
        build_url: matches.value_of("build-url").map(|v| v.to_string()),
        provider_tags: matches.values_of("provider-tags")
          .map_or_else(Vec::new, |tags| tags.map(|tag| tag.to_string()).collect()),
        provider_branch: matches.value_of("provider-branch").map(|v| v.to_string()),
      })
    } else {
      None
    };

    for s in &source {
      debug!("Pact source to verify = {}", s);
    };

    verify_provider_async(
        provider,
        source,
        filter,
        matches.values_of_lossy("filter-consumer").unwrap_or_default(),
        &verification_options,
        publish_options.as_ref(),
        &provider_state_executor,
        Some(VerificationMetrics {
          test_framework: "pact_ffi".to_string(),
          app_name: "unknown".to_string(),
          app_version: "unknown".to_string()
        })
    ).await
      .map_err(|err| {
        error!("Verification failed with error: {}", err);
        2
      })
      .and_then(|result| if result.result { Ok(()) } else { Err(1) })
}

fn print_version(version: &str) {
  println!("\npact verifier version     : v{}", version);
  println!("pact specification version: v{}", PactSpecification::V3.version_str());
}
