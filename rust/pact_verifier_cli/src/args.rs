use clap::{Arg, ArgAction, ArgGroup, Command, command};
use clap::builder::{FalseyValueParser, NonEmptyStringValueParser, PossibleValuesParser};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

fn port_value(v: &str) -> Result<u16, String> {
  v.parse::<u16>().map_err(|e| format!("'{}' is not a valid port value: {}", v, e) )
}

fn integer_value(v: &str) -> Result<u64, String> {
  v.parse::<u64>().map_err(|e| format!("'{}' is not a valid integer value: {}", v, e) )
}

fn validate_regex(val: &str) -> Result<String, String> {
  if val.is_empty() {
    Err("filter value can not be empty".to_string())
  } else {
    Regex::new(val)
      .map(|_| val.to_string())
      .map_err(|err| format!("'{}' is an invalid filter value: {}", val, err))
  }
}

fn json_value(v: &str) -> Result<Value, String> {
  serde_json::from_str(v).map_err(|err| format!("'{}' is not valid JSON: {}", v, err))
}

lazy_static! {
  static ref TRANSPORT_VALUE_RE: Regex = Regex::new(r#"^(\w+):(\d+)(\/[^\s]*)?$"#).unwrap();
}

fn transport_value(v: &str) -> Result<(String, u16, Option<String>), String> {
  if let Some(result) = TRANSPORT_VALUE_RE.captures(v) {
    let transport = if let Some(transport) = result.get(1) {
      transport.as_str().to_string()
    } else {
      return Err(format!("'{}' is not a valid transport, the transport part is empty", v));
    };
    let port = if let Some(port) = result.get(2) {
      port.as_str().parse::<u16>().unwrap() // Ok to unwrap, the regex will only allow digits
    } else {
      return Err(format!("'{}' is not a valid transport, the port part is empty", v));
    };
    Ok((transport, port, result.get(3).map(|v| v.as_str().to_string())))
  } else {
    Err(format!("'{}' is not a valid transport, it must be in the form TRANSPORT:PORT[/path]", v))
  }
}

pub(crate) fn setup_app() -> Command {
  command!()
    .disable_version_flag(true)
    .disable_help_flag(true)
    .arg(Arg::new("help")
      .long("help")
      .action(ArgAction::Help)
      .help("Print help and exit"))
    .arg(Arg::new("version")
      .short('v')
      .long("version")
      .action(ArgAction::Version)
      .help("Print version information and exit"))

    .group(ArgGroup::new("logging").multiple(true))
    .next_help_heading("Logging options")
    .arg(Arg::new("loglevel")
      .short('l')
      .long("loglevel")
      .action(ArgAction::Set)
      .value_parser(PossibleValuesParser::new(["error", "warn", "info", "debug", "trace", "none"]))
      .help("Log level to emit log events at (defaults to warn)"))
    .arg(Arg::new("pretty-log")
      .long("pretty-log")
      .action(ArgAction::SetTrue)
      .conflicts_with_all(&["compact-log", "full-log"])
      .help("Emits excessively pretty, multi-line logs, optimized for human readability."))
    .arg(Arg::new("full-log")
      .long("full-log")
      .conflicts_with_all(&["compact-log", "pretty-log"])
      .action(ArgAction::SetTrue)
      .help("This emits human-readable, single-line logs for each event that occurs, with the current span context displayed before the formatted representation of the event."))
    .arg(Arg::new("compact-log")
      .long("compact-log")
      .conflicts_with_all(&["full-log", "pretty-log"])
      .action(ArgAction::SetTrue)
      .help("Emit logs optimized for short line lengths."))
    .arg(Arg::new("json-file")
      .short('j')
      .long("json")
      .env("PACT_VERIFIER_JSON_REPORT")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Generate a JSON report of the verification"))
    .arg(Arg::new("junit-file")
      .short('x')
      .long("junit")
      .env("PACT_VERIFIER_JUNIT_REPORT")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Generate a JUnit XML report of the verification (requires the junit feature)"))
    .arg(Arg::new("no-colour")
      .long("no-colour")
      .action(ArgAction::SetTrue)
      .visible_alias("no-color")
      .help("Disables ANSI escape codes in the output"))

    .group(ArgGroup::new("source").multiple(true))
    .next_help_heading("Loading pacts options")
    .arg(Arg::new("file")
      .short('f')
      .long("file")
      .required_unless_present_any(&["dir", "url", "broker-url"])
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Pact file to verify (can be repeated)"))
    .arg(Arg::new("dir")
      .short('d')
      .long("dir")
      .required_unless_present_any(&["file", "url", "broker-url"])
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Directory of pact files to verify (can be repeated)"))
    .arg(Arg::new("url")
      .short('u')
      .long("url")
      .required_unless_present_any(&["file", "dir", "broker-url"])
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .help("URL of pact file to verify (can be repeated)"))
    .arg(Arg::new("broker-url")
      .short('b')
      .long("broker-url")
      .env("PACT_BROKER_BASE_URL")
      .required_unless_present_any(&["file", "dir", "url"])
      .requires("provider-name")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("URL of the pact broker to fetch pacts from to verify (requires the provider name parameter)"))
    .arg(Arg::new("webhook-callback-url")
      .long("webhook-callback-url")
      .env("PACT_WEBHOOK_CALLBACK_URL")
      .requires("broker-url")
      .conflicts_with_all(&["file", "dir", "url"])
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("URL of a Pact to verify via a webhook callback. Requires the broker-url to be set."))
    .arg(Arg::new("ignore-no-pacts-error")
      .long("ignore-no-pacts-error")
      .action(ArgAction::SetTrue)
      .help("Do not fail if no pacts are found to verify"))

    .group(ArgGroup::new("auth").multiple(true))
    .next_help_heading("Authentication options")
    .arg(Arg::new("user")
      .long("user")
      .env("PACT_BROKER_USERNAME")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .conflicts_with("token")
      .help("Username to use when fetching pacts from URLS"))
    .arg(Arg::new("password")
      .long("password")
      .env("PACT_BROKER_PASSWORD")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .conflicts_with("token")
      .help("Password to use when fetching pacts from URLS"))
    .arg(Arg::new("token")
      .short('t')
      .long("token")
      .env("PACT_BROKER_TOKEN")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .conflicts_with("user")
      .help("Bearer token to use when fetching pacts from URLS"))

    .group(ArgGroup::new("provider").multiple(true))
    .next_help_heading("Provider options")
    .arg(Arg::new("hostname")
      .short('h')
      .long("hostname")
      .env("PACT_PROVIDER_HOSTNAME")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Provider hostname (defaults to localhost)"))
    .arg(Arg::new("port")
      .short('p')
      .env("PACT_PROVIDER_PORT")
      .long("port")
      .action(ArgAction::Set)
      .help("Provider port (defaults to protocol default 80/443)")
      .value_parser(port_value))
    .arg(Arg::new("transport")
      .long("transport")
      .env("PACT_PROVIDER_TRANSPORT")
      .alias("scheme")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .default_value("http")
      .help("Provider protocol transport to use (http, https, grpc, etc.)"))
    .arg(Arg::new("transports")
      .long("transports")
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .value_delimiter(' ')
      .help("Allows multiple protocol transports to be configured (http, https, grpc, etc.) with their associated port numbers separated by a colon. For example, use --transports http:8080 grpc:5555 to configure both.")
      .value_parser(transport_value))
    .arg(Arg::new("provider-name")
      .short('n')
      .long("provider-name")
      .env("PACT_PROVIDER_NAME")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Provider name (defaults to provider)"))
    .arg(Arg::new("base-path")
      .long("base-path")
      .env("PACT_PROVIDER_BASE_PATH")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Base path to add to all requests"))
    .arg(Arg::new("request-timeout")
      .long("request-timeout")
      .env("PACT_PROVIDER_REQUEST_TIMEOUT")
      .action(ArgAction::Set)
      .value_parser(integer_value)
      .help("Sets the HTTP request timeout in milliseconds for requests to the target API and for state change requests."))
    .arg(Arg::new("custom-header")
      .long("header")
      .short('H')
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Add a custom header to be included in the calls to the provider. Values must be in the form KEY=VALUE, where KEY and VALUE contain ASCII characters (32-127) only. Can be repeated."))
    .arg(Arg::new("disable-ssl-verification")
      .long("disable-ssl-verification")
      .action(ArgAction::SetTrue)
      .help("Disables validation of SSL certificates"))

    .group(ArgGroup::new("states").multiple(true))
    .next_help_heading("Provider state options")
    .arg(Arg::new("state-change-url")
      .short('s')
      .long("state-change-url")
      .env("PACT_PROVIDER_STATE_CHANGE_URL")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("URL to post state change requests to"))
    .arg(Arg::new("state-change-as-query")
      .long("state-change-as-query")
      .env("PACT_PROVIDER_STATE_CHANGE_AS_QUERY")
      .action(ArgAction::SetTrue)
      .value_parser(FalseyValueParser::new())
      .help("State change request data will be sent as query parameters instead of in the request body"))
    .arg(Arg::new("state-change-teardown")
      .long("state-change-teardown")
      .env("PACT_PROVIDER_STATE_CHANGE_TEARDOWN")
      .action(ArgAction::SetTrue)
      .value_parser(FalseyValueParser::new())
      .help("State change teardown requests are to be made after each interaction"))

    .group(ArgGroup::new("filtering").multiple(true))
    .next_help_heading("Filtering interactions")
    .arg(Arg::new("filter-description")
      .long("filter-description")
      .env("PACT_DESCRIPTION")
      .action(ArgAction::Set)
      .value_parser(validate_regex)
      .help("Only validate interactions whose descriptions match this filter (regex format)"))
    .arg(Arg::new("filter-state")
      .long("filter-state")
      .env("PACT_PROVIDER_STATE")
      .action(ArgAction::Set)
      .conflicts_with("filter-no-state")
      .value_parser(validate_regex)
      .help("Only validate interactions whose provider states match this filter (regex format)"))
    .arg(Arg::new("filter-no-state")
      .long("filter-no-state")
      .action(ArgAction::SetTrue)
      .env("PACT_PROVIDER_NO_STATE")
      .conflicts_with("filter-state")
      .help("Only validate interactions that have no defined provider state"))
    .arg(Arg::new("filter-consumer")
      .short('c')
      .long("filter-consumer")
      .action(ArgAction::Append)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Consumer name to filter the pacts to be verified (can be repeated)"))

    .group(ArgGroup::new("publish-options").multiple(true))
    .next_help_heading("Publishing options")
    .arg(Arg::new("publish")
      .long("publish")
      .action(ArgAction::SetTrue)
      .requires("broker-url")
      .requires("provider-version")
      .help("Enables publishing of verification results back to the Pact Broker. Requires the broker-url and provider-version parameters."))
    .arg(Arg::new("provider-version")
      .long("provider-version")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Provider version that is being verified. This is required when publishing results."))
    .arg(Arg::new("build-url")
      .long("build-url")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("URL of the build to associate with the published verification results."))
    .arg(Arg::new("provider-tags")
      .long("provider-tags")
      .action(ArgAction::Set)
      .use_value_delimiter(true)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Provider tags to use when publishing results. Accepts comma-separated values."))
    .arg(Arg::new("provider-branch")
      .long("provider-branch")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .help("Provider branch to use when publishing results"))

    .group(ArgGroup::new("broker").multiple(true))
    .next_help_heading("Pact Broker options")
    .arg(Arg::new("consumer-version-tags")
      .long("consumer-version-tags")
      .action(ArgAction::Set)
      .use_value_delimiter(true)
      .value_parser(NonEmptyStringValueParser::new())
      .requires("broker-url")
      .conflicts_with("consumer-version-selectors")
      .help("Consumer tags to use when fetching pacts from the Broker. Accepts comma-separated values."))
    .arg(Arg::new("consumer-version-selectors")
      .long("consumer-version-selectors")
      .action(ArgAction::Append)
      .value_parser(json_value)
      .requires("broker-url")
      .conflicts_with("consumer-version-tags")
      .help("Consumer version selectors to use when fetching pacts from the Broker. Accepts a JSON string as per https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/. Can be repeated."))
    .arg(Arg::new("enable-pending")
      .long("enable-pending")
      .action(ArgAction::SetTrue)
      .requires("broker-url")
      .help("Enables Pending Pacts"))
    .arg(Arg::new("include-wip-pacts-since")
      .long("include-wip-pacts-since")
      .action(ArgAction::Set)
      .value_parser(NonEmptyStringValueParser::new())
      .requires("broker-url")
      .help("Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip"))

    .group(ArgGroup::new("development").multiple(true))
    .next_help_heading("Development options")
    .arg(Arg::new("exit-first")
      .long("exit-on-first-error")
      .action(ArgAction::SetTrue)
      .conflicts_with("publish")
      .help("Stops the verifier at the first failure"))
    .arg(Arg::new("last-failed")
      .long("last-failed")
      .action(ArgAction::SetTrue)
      .conflicts_with("publish")
      .help("Only runs the interactions that failed on the previous verifier run"))
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use rstest::rstest;

  use crate::args::setup_app;

  use super::{integer_value, port_value, transport_value, validate_regex};

  #[test]
  fn validates_port_value() {
    expect!(port_value("1234")).to(be_ok().value(1234));
    expect!(port_value("1234x")).to(be_err());
    expect!(port_value("3000000")).to(be_err());
  }

  #[test]
  fn validates_integer_value() {
    expect!(integer_value("3000000")).to(be_ok().value(3000000));
    expect!(integer_value("1234x")).to(be_err());
  }

  #[test]
  fn validates_transport_value() {
    expect!(transport_value("http:1234")).to(be_ok().value(("http".to_string(), 1234, None)));
    expect!(transport_value("1234x")).to(be_err());
    expect!(transport_value(":1234")).to(be_err());
    expect!(transport_value("x:")).to(be_err());
    expect!(transport_value("x:x")).to(be_err());
    expect!(transport_value("x:1234x")).to(be_err());
    expect!(transport_value("x:1234/x")).to(be_ok());
    expect!(transport_value("x:1234/p a t h")).to(be_err());
    expect!(transport_value("x:1234/p-a%20t%20h")).to(be_ok());
  }

  #[rstest(
    value,                          expected_value,
    case("http:1234/",              ("http".to_string(), 1234, Some("/".to_string()))),
    case("http:1234/p",             ("http".to_string(), 1234, Some("/p".to_string()))),
    case("http:1234/p/",            ("http".to_string(), 1234, Some("/p/".to_string()))),
    case("http:1234/path/2",        ("http".to_string(), 1234, Some("/path/2".to_string()))),
    case("http:1234/path/2/s%20s",  ("http".to_string(), 1234, Some("/path/2/s%20s".to_string())))
  )]
  fn validates_transport_value_with_path(value: &str, expected_value: (String, u16, Option<String>)) {
    expect!(transport_value(value)).to(be_ok().value(expected_value));
  }

  #[test]
  fn validates_regex_value() {
    expect!(validate_regex("\\d+")).to(be_ok().value("\\d+".to_string()));
    expect!(validate_regex("[a-z")).to(be_err());
    expect!(validate_regex("")).to(be_err());
  }

  #[test]
  fn verify_cli() {
    setup_app().debug_assert();
  }
}
