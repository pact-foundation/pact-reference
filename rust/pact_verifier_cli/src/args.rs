#![allow(missing_docs)]

use std::fmt;

use clap::{ArgAction, Args, Parser, ValueEnum};
use clap::builder::{FalseyValueParser, NonEmptyStringValueParser};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

pub(crate) fn port_value(v: &str) -> Result<u16, String> {
  v.parse::<u16>().map_err(|e| format!("'{}' is not a valid port value: {}", v, e))
}

pub(crate) fn integer_value(v: &str) -> Result<u64, String> {
  v.parse::<u64>().map_err(|e| format!("'{}' is not a valid integer value: {}", v, e))
}

pub(crate) fn validate_regex(val: &str) -> Result<String, String> {
  if val.is_empty() {
    Err("filter value can not be empty".to_string())
  } else {
    Regex::new(val)
      .map(|_| val.to_string())
      .map_err(|err| format!("'{}' is an invalid filter value: {}", val, err))
  }
}

pub(crate) fn json_value(v: &str) -> Result<Value, String> {
  serde_json::from_str(v).map_err(|err| format!("'{}' is not valid JSON: {}", v, err))
}

lazy_static! {
  static ref TRANSPORT_VALUE_RE: Regex = Regex::new(r#"^(\w+):(\d+)(\/[^\s]*)?$"#).unwrap();
}

pub(crate) fn transport_value(v: &str) -> Result<(String, u16, Option<String>), String> {
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

/// Log level to use
#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum LogLevel {
  /// error
  Error,
  /// warn
  Warn,
  /// info
  Info,
  /// debug
  Debug,
  /// trace
  Trace,
  /// none
  None,
}

impl fmt::Display for LogLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.to_possible_value().expect("no skipped values").get_name().fmt(f)
  }
}

/// Standalone pact verifier CLI arguments
#[derive(Parser, Debug)]
#[command(
  author,
  version,
  about = "Standalone pact verifier for provider pact verification",
  disable_version_flag = true,
  disable_help_flag = true,
  arg_required_else_help = true,
)]
pub struct CliArgs {
  /// Print help and exit
  #[arg(long, action = ArgAction::Help)]
  pub help: Option<bool>,

  /// Print version information and exit
  #[arg(short = 'v', long, action = ArgAction::Version)]
  pub version: Option<bool>,

  #[command(flatten)]
  pub logging: LoggingArgs,

  #[command(flatten)]
  pub source: SourceArgs,

  #[command(flatten)]
  pub auth: AuthArgs,

  #[command(flatten)]
  pub provider: ProviderArgs,

  #[command(flatten)]
  pub states: StateArgs,

  #[command(flatten)]
  pub filtering: FilteringArgs,

  #[command(flatten)]
  pub publishing: PublishingArgs,

  #[command(flatten)]
  pub broker: BrokerArgs,

  #[command(flatten)]
  pub development: DevelopmentArgs,
}

/// Logging-related options
#[derive(Args, Debug)]
#[command(next_help_heading = "Logging options")]
pub struct LoggingArgs {
  /// Log level to emit log events at (defaults to warn)
  #[arg(short = 'l', long, value_name = "loglevel", value_enum)]
  pub loglevel: Option<LogLevel>,

  /// Emits excessively pretty, multi-line logs, optimized for human readability.
  #[arg(long, conflicts_with_all = ["compact_log", "full_log"])]
  pub pretty_log: bool,

  /// This emits human-readable, single-line logs for each event that occurs, with the current span context displayed before the formatted representation of the event.
  #[arg(long, conflicts_with_all = ["compact_log", "pretty_log"])]
  pub full_log: bool,

  /// Emit logs optimized for short line lengths.
  #[arg(long, conflicts_with_all = ["full_log", "pretty_log"])]
  pub compact_log: bool,

  /// Generate a JSON report of the verification
  #[arg(
    short = 'j',
    long = "json",
    env = "PACT_VERIFIER_JSON_REPORT",
    value_name = "json-file",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub json_file: Option<String>,

  /// Generate a JUnit XML report of the verification (requires the junit feature)
  #[arg(
    short = 'x',
    long = "junit",
    env = "PACT_VERIFIER_JUNIT_REPORT",
    value_name = "junit-file",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub junit_file: Option<String>,

  /// Disables ANSI escape codes in the output
  #[arg(long = "no-colour", visible_alias = "no-color")]
  pub no_colour: bool,
}

/// Options for loading pact files
#[derive(Args, Debug)]
#[command(next_help_heading = "Loading pacts options")]
pub struct SourceArgs {
  /// Pact file to verify (can be repeated)
  #[arg(
    short = 'f',
    long,
    value_name = "file",
    required_unless_present_any = ["dir", "url", "broker_url"],
    action = ArgAction::Append,
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub file: Vec<String>,

  /// Directory of pact files to verify (can be repeated)
  #[arg(
    short = 'd',
    long,
    value_name = "dir",
    required_unless_present_any = ["file", "url", "broker_url"],
    action = ArgAction::Append,
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub dir: Vec<String>,

  /// URL of pact file to verify (can be repeated)
  #[arg(
    short = 'u',
    long,
    value_name = "url",
    required_unless_present_any = ["file", "dir", "broker_url"],
    action = ArgAction::Append,
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub url: Vec<String>,

  /// URL of the pact broker to fetch pacts from to verify (requires the provider name parameter)
  #[arg(
    short = 'b',
    long = "broker-url",
    env = "PACT_BROKER_BASE_URL",
    value_name = "broker-url",
    required_unless_present_any = ["file", "dir", "url"],
    requires = "provider_name",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub broker_url: Option<String>,

  /// URL of a Pact to verify via a webhook callback. Requires the broker-url to be set.
  #[arg(
    long = "webhook-callback-url",
    env = "PACT_WEBHOOK_CALLBACK_URL",
    value_name = "webhook-callback-url",
    requires = "broker_url",
    conflicts_with_all = ["file", "dir", "url"],
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub webhook_callback_url: Option<String>,

  /// Do not fail if no pacts are found to verify
  #[arg(long = "ignore-no-pacts-error")]
  pub ignore_no_pacts_error: bool,
}

/// Authentication options for fetching pacts
#[derive(Args, Debug)]
#[command(next_help_heading = "Authentication options")]
pub struct AuthArgs {
  /// Username to use when fetching pacts from URLS
  #[arg(
    long,
    env = "PACT_BROKER_USERNAME",
    value_name = "user",
    value_parser = NonEmptyStringValueParser::new(),
    conflicts_with = "token"
  )]
  pub user: Option<String>,

  /// Password to use when fetching pacts from URLS
  #[arg(
    long,
    env = "PACT_BROKER_PASSWORD",
    value_name = "password",
    value_parser = NonEmptyStringValueParser::new(),
    conflicts_with = "token"
  )]
  pub password: Option<String>,

  /// Bearer token to use when fetching pacts from URLS
  #[arg(
    short = 't',
    long,
    env = "PACT_BROKER_TOKEN",
    value_name = "token",
    value_parser = NonEmptyStringValueParser::new(),
    conflicts_with = "user"
  )]
  pub token: Option<String>,
}

/// Options for the provider under test
#[derive(Args, Debug)]
#[command(next_help_heading = "Provider options")]
pub struct ProviderArgs {
  /// Provider hostname (defaults to localhost)
  #[arg(
    short = 'h',
    long,
    env = "PACT_PROVIDER_HOSTNAME",
    value_name = "hostname",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub hostname: Option<String>,

  /// Provider port (defaults to protocol default 80/443)
  #[arg(
    short = 'p',
    long,
    env = "PACT_PROVIDER_PORT",
    value_name = "port",
    value_parser = port_value
  )]
  pub port: Option<u16>,

  /// Provider protocol transport to use (http, https, grpc, etc.)
  #[arg(
    long,
    env = "PACT_PROVIDER_TRANSPORT",
    value_name = "transport",
    alias = "scheme",
    value_parser = NonEmptyStringValueParser::new(),
    default_value = "http"
  )]
  pub transport: String,

  /// Allows multiple protocol transports to be configured (http, https, grpc, etc.) with their associated port numbers separated by a colon. For example, use --transports http:8080 grpc:5555 to configure both.
  #[arg(
    long,
    value_name = "transports",
    action = ArgAction::Append,
    value_delimiter = ' ',
    value_parser = transport_value
  )]
  pub transports: Vec<(String, u16, Option<String>)>,

  /// Provider name (defaults to provider)
  #[arg(
    short = 'n',
    long = "provider-name",
    env = "PACT_PROVIDER_NAME",
    value_name = "provider-name",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub provider_name: Option<String>,

  /// Base path to add to all requests
  #[arg(
    long = "base-path",
    env = "PACT_PROVIDER_BASE_PATH",
    value_name = "base-path",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub base_path: Option<String>,

  /// Sets the HTTP request timeout in milliseconds for requests to the target API and for state change requests.
  #[arg(
    long = "request-timeout",
    env = "PACT_PROVIDER_REQUEST_TIMEOUT",
    value_name = "request-timeout",
    value_parser = integer_value
  )]
  pub request_timeout: Option<u64>,

  /// Add a custom header to be included in the calls to the provider. Values must be in the form KEY=VALUE, where KEY and VALUE contain ASCII characters (32-127) only. Can be repeated.
  #[arg(
    long = "header",
    short = 'H',
    value_name = "custom-header",
    action = ArgAction::Append,
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub custom_header: Vec<String>,

  /// Disables validation of SSL certificates
  #[arg(long = "disable-ssl-verification")]
  pub disable_ssl_verification: bool,
}

/// Options for provider state management
#[derive(Args, Debug)]
#[command(next_help_heading = "Provider state options")]
pub struct StateArgs {
  /// URL to post state change requests to
  #[arg(
    short = 's',
    long = "state-change-url",
    env = "PACT_PROVIDER_STATE_CHANGE_URL",
    value_name = "state-change-url",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub state_change_url: Option<String>,

  /// State change request data will be sent as query parameters instead of in the request body
  #[arg(
    long = "state-change-as-query",
    env = "PACT_PROVIDER_STATE_CHANGE_AS_QUERY",
    action = ArgAction::SetTrue,
    value_parser = FalseyValueParser::new()
  )]
  pub state_change_as_query: bool,

  /// State change teardown requests are to be made after each interaction
  #[arg(
    long = "state-change-teardown",
    env = "PACT_PROVIDER_STATE_CHANGE_TEARDOWN",
    action = ArgAction::SetTrue,
    value_parser = FalseyValueParser::new()
  )]
  pub state_change_teardown: bool,
}

/// Options for filtering which interactions to verify
#[derive(Args, Debug)]
#[command(next_help_heading = "Filtering interactions")]
pub struct FilteringArgs {
  /// Only validate interactions whose descriptions match this filter (regex format)
  #[arg(
    long = "filter-description",
    env = "PACT_DESCRIPTION",
    value_name = "filter-description",
    value_parser = validate_regex
  )]
  pub filter_description: Option<String>,

  /// Only validate interactions whose provider states match this filter (regex format)
  #[arg(
    long = "filter-state",
    env = "PACT_PROVIDER_STATE",
    value_name = "filter-state",
    conflicts_with = "filter_no_state",
    value_parser = validate_regex
  )]
  pub filter_state: Option<String>,

  /// Only validate interactions that have no defined provider state
  #[arg(
    long = "filter-no-state",
    env = "PACT_PROVIDER_NO_STATE",
    conflicts_with = "filter_state"
  )]
  pub filter_no_state: bool,

  /// Consumer name to filter the pacts to be verified (can be repeated)
  #[arg(
    short = 'c',
    long = "filter-consumer",
    value_name = "filter-consumer",
    action = ArgAction::Append,
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub filter_consumer: Vec<String>,
}

/// Options for publishing verification results
#[derive(Args, Debug)]
#[command(next_help_heading = "Publishing options")]
pub struct PublishingArgs {
  /// Enables publishing of verification results back to the Pact Broker. Requires the broker-url and provider-version parameters.
  #[arg(long, requires = "broker_url", requires = "provider_version")]
  pub publish: bool,

  /// Provider version that is being verified. This is required when publishing results.
  #[arg(
    long = "provider-version",
    value_name = "provider-version",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub provider_version: Option<String>,

  /// URL of the build to associate with the published verification results.
  #[arg(
    long = "build-url",
    value_name = "build-url",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub build_url: Option<String>,

  /// Provider tags to use when publishing results. Accepts comma-separated values.
  #[arg(
    long = "provider-tags",
    value_name = "provider-tags",
    value_delimiter = ',',
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub provider_tags: Vec<String>,

  /// Provider branch to use when publishing results
  #[arg(
    long = "provider-branch",
    value_name = "provider-branch",
    value_parser = NonEmptyStringValueParser::new()
  )]
  pub provider_branch: Option<String>,
}

/// Options for fetching pacts from a Pact Broker
#[derive(Args, Debug)]
#[command(next_help_heading = "Pact Broker options")]
pub struct BrokerArgs {
  /// Consumer tags to use when fetching pacts from the Broker. Accepts comma-separated values.
  #[arg(
    long = "consumer-version-tags",
    value_name = "consumer-version-tags",
    value_delimiter = ',',
    value_parser = NonEmptyStringValueParser::new(),
    requires = "broker_url",
    conflicts_with = "consumer_version_selectors"
  )]
  pub consumer_version_tags: Vec<String>,

  /// Consumer version selectors to use when fetching pacts from the Broker. Accepts a JSON string as per https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/. Can be repeated.
  #[arg(
    long = "consumer-version-selectors",
    value_name = "consumer-version-selectors",
    action = ArgAction::Append,
    value_parser = json_value,
    requires = "broker_url",
    conflicts_with = "consumer_version_tags"
  )]
  pub consumer_version_selectors: Vec<Value>,

  /// Enables Pending Pacts
  #[arg(long = "enable-pending", requires = "broker_url")]
  pub enable_pending: bool,

  /// Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip
  #[arg(
    long = "include-wip-pacts-since",
    value_name = "include-wip-pacts-since",
    value_parser = NonEmptyStringValueParser::new(),
    requires = "broker_url"
  )]
  pub include_wip_pacts_since: Option<String>,
}

/// Development and debugging options
#[derive(Args, Debug)]
#[command(next_help_heading = "Development options")]
pub struct DevelopmentArgs {
  /// Stops the verifier at the first failure
  #[arg(long = "exit-on-first-error", conflicts_with = "publish")]
  pub exit_first: bool,

  /// Only runs the interactions that failed on the previous verifier run. Requires --json-file to have been set
  #[arg(
    long = "last-failed",
    conflicts_with = "publish",
    conflicts_with_all = ["filter_description", "filter_state", "filter_no_state", "filter_consumer"],
    requires = "json_file"
  )]
  pub last_failed: bool,
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use rstest::rstest;

  use crate::args::CliArgs;

  use super::{integer_value, port_value, transport_value, validate_regex};
  use clap::{CommandFactory, Parser};

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
    CliArgs::command().debug_assert();
  }
}
