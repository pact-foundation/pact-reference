//! Exported verifier functions

use std::env;
use std::str;
use std::str::FromStr;
use std::sync::Arc;

use clap::{AppSettings, ArgMatches, ErrorKind};
use log::{debug, LevelFilter};
use simplelog::{Config, TerminalMode, TermLogger};

use pact_matching::s;
use pact_models::http_utils::HttpAuth;
use pact_models::PactSpecification;
use pact_verifier::*;
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;

use super::args;

fn pact_source(matches: &ArgMatches) -> Vec<PactSource> {
  let mut sources = vec![];

  if let Some(values) = matches.values_of("file") {
    sources.extend(values.map(|v| PactSource::File(s!(v))).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("dir") {
    sources.extend(values.map(|v| PactSource::Dir(s!(v))).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("url") {
    sources.extend(values.map(|v| {
      if matches.is_present("user") {
        PactSource::URL(s!(v), matches.value_of("user").map(|user| {
          HttpAuth::User(user.to_string(), matches.value_of("password").map(|p| p.to_string()))
        }))
      } else if matches.is_present("token") {
        PactSource::URL(s!(v), matches.value_of("token").map(|token| HttpAuth::Token(token.to_string())))
      } else {
        PactSource::URL(s!(v), None)
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

      let selectors = if matches.is_present("consumer-version-selectors") {
        matches.values_of("consumer-version-selectors")
          .map_or_else(Vec::new, |s| json_to_selectors(s.collect::<Vec<_>>()))
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

fn consumer_tags_to_selectors(tags: Vec<&str>) -> Vec<pact_verifier::ConsumerVersionSelector> {
  tags.iter().map(|t| {
    pact_verifier::ConsumerVersionSelector {
      consumer: None,
      fallback_tag: None,
      tag: Some(t.to_string()),
      latest: Some(true),
      branch: None,
      deployed_or_released: None,
      deployed: None,
      released: None,
      main_branch: None,
      environment: None,
    }
  }).collect()
}

fn json_to_selectors(tags: Vec<&str>) -> Vec<pact_verifier::ConsumerVersionSelector> {
  tags.iter().map(|t| serde_json::from_str(t))
  .flatten()
  .collect()
}

fn interaction_filter(matches: &ArgMatches) -> FilterInfo {
  if matches.is_present("filter-description") &&
      (matches.is_present("filter-state") || matches.is_present("filter-no-state")) {
      if matches.is_present("filter-state") {
          FilterInfo::DescriptionAndState(s!(matches.value_of("filter-description").unwrap()),
              s!(matches.value_of("filter-state").unwrap()))
      } else {
          FilterInfo::DescriptionAndState(s!(matches.value_of("filter-description").unwrap()),
              s!(""))
      }
  } else if matches.is_present("filter-description") {
      FilterInfo::Description(s!(matches.value_of("filter-description").unwrap()))
  } else if matches.is_present("filter-state") {
      FilterInfo::State(s!(matches.value_of("filter-state").unwrap()))
  } else if matches.is_present("filter-no-state") {
      FilterInfo::State(s!(""))
  } else {
      FilterInfo::None
  }
}

/// Handles the command line arguments from the running process
pub async fn handle_cli(version: &str) -> Result<(), i32> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let app = args::setup_app(program, version);
  let matches = app
                  .setting(AppSettings::ArgRequiredElseHelp)
                  .setting(AppSettings::ColoredHelp)
                  .get_matches_safe();

  match matches {
    Ok(results) => handle_matches(&results).await,
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

// TODO: it's possible to introspect the clap::Error and return it or wrapped error type
// so that the caller could have more control over the error output.
//
// Currently, clap prints things out as if it were a CLI call
#[allow(dead_code, missing_docs)]
pub async fn handle_args(args: Vec<String>) -> Result<(), i32> {
  let program = "pact_verifier_cli".to_string();
  let version = format!("v{}", clap::crate_version!()).as_str().to_owned();
  let app = args::setup_app(program, &version);
  let matches = app
                  .setting(AppSettings::NoBinaryName)
                  .setting(AppSettings::ColorNever)
                  .get_matches_from_safe(args);

  match matches {
    Ok(results) => handle_matches(&results).await,
    Err(ref err) => {
      log::error!("Error verifying Pact: {:?} {:?}", err.message, err);
      Err(4)
    }
  }
}

async fn handle_matches(matches: &clap::ArgMatches<'_>) -> Result<(), i32> {
    let level = matches.value_of("loglevel").unwrap_or("warn");
    let log_level = match level {
        "none" => LevelFilter::Off,
        _ => LevelFilter::from_str(level).unwrap()
    };
    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed).unwrap_or_default();
    let provider = ProviderInfo {
      host: s!(matches.value_of("hostname").unwrap_or("localhost")),
      port: matches.value_of("port").map(|port| port.parse::<u16>().unwrap()),
      path: matches.value_of("base-path").unwrap_or("/").into(),
      protocol: s!(matches.value_of("scheme").unwrap_or("http")),
      .. ProviderInfo::default()
    };
    let source = pact_source(matches);
    let filter = interaction_filter(matches);
    let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
      state_change_url: matches.value_of("state-change-url").map(|s| s.to_string()),
      state_change_body: !matches.is_present("state-change-as-query"),
      state_change_teardown: matches.is_present("state-change-teardown")
    });

    let options = VerificationOptions {
      publish: matches.is_present("publish"),
      provider_version: matches.value_of("provider-version").map(|v| v.to_string()),
      build_url: matches.value_of("build-url").map(|v| v.to_string()),
      request_filter: None::<Arc<NullRequestFilterExecutor>>,
      provider_tags: matches.values_of("provider-tags").map_or_else(Vec::new, |tags| tags.map(|tag| tag.to_string()).collect()),
      disable_ssl_verification: matches.is_present("disable-ssl-verification"),
      request_timeout: matches.value_of("request-timeout").map(|t| t.parse::<u64>().unwrap_or(5000)).unwrap_or(5000),
      .. VerificationOptions::default()
    };

    for s in &source {
      debug!("Pact source to verify = {}", s);
    };

    if verify_provider_async(
        provider,
        source,
        filter,
        matches.values_of_lossy("filter-consumer").unwrap_or_default(),
        options,
        &provider_state_executor
    ).await {
        Ok(())
    } else {
        Err(1)
    }
}

fn print_version(version: &str) {
  println!("\npact verifier version     : v{}", version);
  println!("pact specification version: v{}", PactSpecification::V3.version_str());
}
