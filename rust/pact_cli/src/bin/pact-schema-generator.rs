//! Pact schema generator

#![warn(missing_docs)]

use std::env;

use clap::{App, AppSettings, Arg, ArgMatches, ErrorKind};
use log::*;
use serde_json::{json, to_string_pretty};

use pact_cli::setup_loggers;
use pact_models::PactSpecification;
use pact_models::sync_pact::RequestResponsePact;

fn setup_app<'a, 'b>(program: &str, version: &'b str) -> App<'a, 'b> {
  App::new(program)
    .version(version)
    .about("Pact schema generator")
    .version_short("v")
    .arg(Arg::with_name("loglevel")
      .short("l")
      .long("loglevel")
      .takes_value(true)
      .use_delimiter(false)
      .possible_values(&["error", "warn", "info", "debug", "trace", "none"])
      .help("Log level (defaults to warn)"))
    .arg(Arg::with_name("spec")
      .long("specification")
      .short("s")
      .takes_value(true)
      .possible_values(&["v1", "v2", "v3", "v4"])
      .default_value("v4")
      .help("Pact specification to generate the schema for."))
}

fn handle_cli() -> Result<(), i32> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let app = setup_app(&program, clap::crate_version!());
  let matches = app
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::ColoredHelp)
    .get_matches_safe();

  match matches {
    Ok(results) => handle_matches(&results),
    Err(ref err) => {
      match err.kind {
        ErrorKind::HelpDisplayed => {
          println!("{}", err.message);
          Ok(())
        },
        ErrorKind::VersionDisplayed => Ok(()),
        _ => err.exit()
      }
    }
  }
}

fn handle_matches(args: &ArgMatches) -> Result<(), i32> {
  let log_level = args.value_of("loglevel");
  if let Err(err) = setup_loggers(log_level.unwrap_or("warn")) {
    eprintln!("ERROR: Could not setup loggers: {}", err);
    eprintln!();
  }

  let spec_version = args.value_of("spec")
    .map(|version| PactSpecification::from(version))
    .unwrap_or(PactSpecification::V4);

  let mut schema = json!({
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": format!("https://pact.io/schema/pact-{}.json", spec_version.to_string()),
    "title": format!("Pact File {} Schema", spec_version.to_string()),
    "description": format!("JSON schema for a {} specification pact file", spec_version.to_string()),
    "type": "object"
  });

  if let Some(map) = schema.as_object_mut() {
    match spec_version {
      PactSpecification::V1_1 |PactSpecification::V1 | PactSpecification::V2 => {
        if let Some(attributes) = RequestResponsePact::schema(spec_version).as_object() {
          for (k, v) in attributes {
            map.insert(k.clone(), v.clone());
          }
        }
      }
      PactSpecification::V3 => {}
      PactSpecification::V4 => {}
      _ => {
        eprintln!("ERROR: Mat a valid Pact specification version: '{}'",
                  args.value_of("spec").unwrap_or_default());
        return Err(2);
      }
    }
  }

  let result = to_string_pretty(&schema);
  match result {
    Ok(str) => {
      println!("{}", str);
      Ok(())
    }
    Err(err) => {
      eprintln!("ERROR: failed to generate schema: {}", err);
      Err(1)
    }
  }
}

fn main() {
  match handle_cli() {
    Ok(_) => (),
    Err(err) => std::process::exit(err)
  }
}
