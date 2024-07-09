//! # Standalone Pact Verifier
//!
//! This project provides a command line interface to verify pact files against a running provider. It is a single 
//! executable binary. It implements all the Pact specifications up to [V4](https://github.com/pact-foundation/pact-specification/tree/version-4).
//!
//! [Online rust docs](https://docs.rs/pact_verifier_cli/)
//!
//! The Pact Verifier works by taking all the interactions (requests and responses) from a number of pact files. For each 
//! interaction, it will make the request defined in the pact to a running service provider and check the response received
//! back against the one defined in the pact file. All mismatches will then be reported.
//!
//! ## Command line interface
//!
//! The pact verifier is bundled as a single binary executable `pact_verifier_cli`. Running this without any options 
//! displays the standard help.
//!
//! ```console
//! $ pact_verifier_cli,ignore
//! Standalone pact verifier for provider pact verification
//!
//! Usage: pact_verifier_cli [OPTIONS]
//!
//! Options:
//!       --help     Print help and exit
//!   -v, --version  Print version information and exit
//!
//! Logging options:
//!   -l, --loglevel <loglevel>  Log level to emit log events at (defaults to warn) [possible values: error, warn, info, debug, trace, none]
//!       --pretty-log           Emits excessively pretty, multi-line logs, optimized for human readability.
//!       --full-log             This emits human-readable, single-line logs for each event that occurs, with the current span context displayed before the formatted representation of the event.
//!       --compact-log          Emit logs optimized for short line lengths.
//!   -j, --json <json-file>     Generate a JSON report of the verification [env: PACT_VERIFIER_JSON_REPORT=]
//!   -x, --junit <junit-file>   Generate a JUnit XML report of the verification (requires the junit feature) [env: PACT_VERIFIER_JUNIT_REPORT=]
//!       --no-colour            Disables ANSI escape codes in the output [aliases: no-color]
//!
//! Loading pacts options:
//!   -f, --file <file>
//!           Pact file to verify (can be repeated)
//!   -d, --dir <dir>
//!           Directory of pact files to verify (can be repeated)
//!   -u, --url <url>
//!           URL of pact file to verify (can be repeated)
//!   -b, --broker-url <broker-url>
//!           URL of the pact broker to fetch pacts from to verify (requires the provider name parameter) [env: PACT_BROKER_BASE_URL=]
//!       --webhook-callback-url <webhook-callback-url>
//!           URL of a Pact to verify via a webhook callback. Requires the broker-url to be set. [env: PACT_WEBHOOK_CALLBACK_URL=]
//!       --ignore-no-pacts-error
//!           Do not fail if no pacts are found to verify
//!
//! Authentication options:
//!       --user <user>          Username to use when fetching pacts from URLS [env: PACT_BROKER_USERNAME=]
//!       --password <password>  Password to use when fetching pacts from URLS [env: PACT_BROKER_PASSWORD=]
//!   -t, --token <token>        Bearer token to use when fetching pacts from URLS [env: PACT_BROKER_TOKEN=]
//!
//! Provider options:
//!   -h, --hostname <hostname>
//!           Provider hostname (defaults to localhost) [env: PACT_PROVIDER_HOSTNAME=]
//!   -p, --port <port>
//!           Provider port (defaults to protocol default 80/443) [env: PACT_PROVIDER_PORT=]
//!       --transport <transport>
//!           Provider protocol transport to use (http, https, grpc, etc.) [env: PACT_PROVIDER_TRANSPORT=] [default: http]
//!       --transports <transports>
//!           Allows multiple protocol transports to be configured (http, https, grpc, etc.) with their associated port numbers separated by a colon. For example, use --transports http:8080 grpc:5555 to configure both.
//!   -n, --provider-name <provider-name>
//!           Provider name (defaults to provider) [env: PACT_PROVIDER_NAME=]
//!       --base-path <base-path>
//!           Base path to add to all requests [env: PACT_PROVIDER_BASE_PATH=]
//!       --request-timeout <request-timeout>
//!           Sets the HTTP request timeout in milliseconds for requests to the target API and for state change requests. [env: PACT_PROVIDER_REQUEST_TIMEOUT=]
//!   -H, --header <custom-header>
//!           Add a custom header to be included in the calls to the provider. Values must be in the form KEY=VALUE, where KEY and VALUE contain ASCII characters (32-127) only. Can be repeated.
//!       --disable-ssl-verification
//!           Disables validation of SSL certificates
//!
//! Provider state options:
//!   -s, --state-change-url <state-change-url>
//!           URL to post state change requests to [env: PACT_PROVIDER_STATE_CHANGE_URL=]
//!       --state-change-as-query
//!           State change request data will be sent as query parameters instead of in the request body [env: PACT_PROVIDER_STATE_CHANGE_AS_QUERY=]
//!       --state-change-teardown
//!           State change teardown requests are to be made after each interaction [env: PACT_PROVIDER_STATE_CHANGE_TEARDOWN=]
//!
//! Filtering interactions:
//!       --filter-description <filter-description>
//!           Only validate interactions whose descriptions match this filter (regex format) [env: PACT_DESCRIPTION=]
//!       --filter-state <filter-state>
//!           Only validate interactions whose provider states match this filter (regex format) [env: PACT_PROVIDER_STATE=]
//!       --filter-no-state
//!           Only validate interactions that have no defined provider state [env: PACT_PROVIDER_NO_STATE=]
//!   -c, --filter-consumer <filter-consumer>
//!           Consumer name to filter the pacts to be verified (can be repeated)
//!
//! Publishing options:
//!       --publish
//!           Enables publishing of verification results back to the Pact Broker. Requires the broker-url and provider-version parameters.
//!       --provider-version <provider-version>
//!           Provider version that is being verified. This is required when publishing results.
//!       --build-url <build-url>
//!           URL of the build to associate with the published verification results.
//!       --provider-tags <provider-tags>
//!           Provider tags to use when publishing results. Accepts comma-separated values.
//!       --provider-branch <provider-branch>
//!           Provider branch to use when publishing results
//!
//! Pact Broker options:
//!       --consumer-version-tags <consumer-version-tags>
//!           Consumer tags to use when fetching pacts from the Broker. Accepts comma-separated values.
//!       --consumer-version-selectors <consumer-version-selectors>
//!           Consumer version selectors to use when fetching pacts from the Broker. Accepts a JSON string as per https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/. Can be repeated.
//!       --enable-pending
//!           Enables Pending Pacts
//!       --include-wip-pacts-since <include-wip-pacts-since>
//!           Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall task to fail. For more information, see https://pact.io/wip
//! ```
//!
//! ## Options
//!
//! ### Log Level
//!
//! You can control the log level with the `-l, --loglevel <loglevel>` option. It defaults to warn, and the options that 
//! you can specify are: error, warn, info, debug, trace, none.
//!
//! ### Pact File Sources
//!
//! You can specify the pacts to verify with the following options. They can be repeated to set multiple sources.
//!
//! | Option                          | Type        | Description                                                                                                          |
//! |---------------------------------|-------------|----------------------------------------------------------------------------------------------------------------------|
//! | `-f, --file <file>`             | File        | Loads a pact from the given file                                                                                     |
//! | `-u, --url <url>`               | URL         | Loads a pact from a URL resource                                                                                     |
//! | `-d, --dir <dir>`               | Directory   | Loads all the pacts from the given directory                                                                         |
//! | `-b, --broker-url <broker-url>` | Pact Broker | Loads all the pacts for the provider from the pact broker. Requires the `-n, --provider-name <provider-name>` option |
//!
//! #### Verifying a Pact via a webhook callback
//!
//! The Pact Broker allows for Pacts to be verified via a callback that supplies the URL to the Pact to verify. To verify
//! just the Pact from the webhook call, use the `--webhook-callback-url` set to the supplied URL in conjunction with the 
//! `--broker-url` option.
//!
//! ### Provider Options
//!
//! The running provider can be specified with the following options:
//!
//! | Option                                | Description                                                                                                   |
//! |---------------------------------------|---------------------------------------------------------------------------------------------------------------|
//! | `-h, --hostname <hostname>`           | The provider hostname, defaults to `localhost`                                                                |
//! | `-p, --port <port>`                   | The provider port (defaults to protocol default 80/443)                                                       |
//! | `-n, --provider-name <provider-name>` | The name of the provider. Required if you are loading pacts from a pact broker                                |
//! | `--base-path <base-path>`             | If the provider is mounted on a sub-path, you can use this option to set the base path to add to all requests |
//! | `--transport <transport>`             | Protocol transport to use. Defaults to HTTP.                                                                  |
//!
//! ### Filtering the interactions
//!
//! The interactions that are verified can be filtered by the following options:
//!
//! #### `-c, --filter-consumer <filter-consumer>`
//!
//! This will only verify the interactions of matching consumers. You can specify multiple consumers by either separating
//! the names with a comma, or repeating the option.
//!
//! #### `--filter-description <filter-description>`
//!
//! This option will filter the interactions that are verified that match by description. You can use a regular expression
//! to match.
//!
//! #### `--filter-state <filter-state>`
//!
//! This option will filter the interactions that are verified that match by provider state. You can use a regular
//! expression to match. Can't be used with the `--filter-no-state` option.
//!
//! #### `--filter-no-state`
//!
//! This option will filter the interactions that are verified that don't have a defined provider state. Can't be used
//! with the `--filter-state` option.
//!
//! ### State change requests
//!
//! [Provider states](https://docs.pact.io/getting_started/provider_states) are a mechanism to define the state that the 
//! provider needs to be in to be able to verify a particular request. This is achieved by setting a state change URL that
//! will receive a POST request with the provider state before the actual request is made.
//!
//! *NOTE:* For verifying messages fetched via HTTP, the provider state is also passed in the request to fetch the message,
//! so the state change URL is not required.
//!
//! For example, if a Pact file being verified has a provider state *"a user exists in the database"* and the provider state 
//! URL is set to `http://localhost:8080/provider-state`, then the following request would be made before the interaction
//! is verified:
//!
//! ```http request
//! POST /provider-state HTTP/1.1
//! Host: localhost:8080
//! content-type: application/json
//!
//! {
//!     "state": "a user exists in the database",
//!     "params": {},
//!     "action": "setup"
//! }
//! ```
//!
//! If any parameters are configured for the provider state, they will be passed in the *"params"* attribute.
//!
//! #### `-s, --state-change-url <state-change-url>`
//!
//! This sets the absolute URL that the POST requests will be made to before each actual request. If this value is not
//! set, the state change request will not be made. 
//!
//! #### `--state-change-as-query`
//!
//! By default, the state for the state change request will be sent as a JSON document in the body of the request. This 
//! option forces it to be sent as query parameters instead.
//!
//! #### `--state-change-teardown`
//!
//! This option will cause the verifier to also make a tear down request after the main request is made. It will receive a 
//! field in the body or a query parameter named `action` with the value `teardown`.
//!
//! #### `--consumer-version-selectors`
//!
//! Accepts a set of [Consumer Version Selectors](https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/) encoded as JSON.
//!
//! An example of a well-formed argument value might be:
//!
//! ```sh
//! --consumer-version-selectors '{"branch": "master"}'
//! ```
//!
//! ## Example run
//!
//! This will verify all the pacts for the `happy_provider` found in the pact broker (running on localhost) against the provider running on localhost port 5050. Only the pacts for the consumers `Consumer` and `Consumer2` will be verified.
//!
//! ```console,ignore
//! $ pact_verifier_cli -b http://localhost -n 'happy_provider' -p 5050 --filter-consumer Consumer --filter-consumer Consumer2
//! 21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0", assuming V1.1 specification
//! 21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0", assuming V1.1 specification
//!
//! Verifying a pact between Consumer and happy_provider
//!   Given I am friends with Fred
//!     WARNING: State Change ignored as there is no state change URL
//!   Given I have no friends
//!     WARNING: State Change ignored as there is no state change URL
//!   a request to unfriend but no friends
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!       has a matching body (OK)
//!   a request friends
//!     returns a response which
//!       has status code 200 (FAILED)
//!       includes headers
//!         "Content-Type" with value "application/json" (FAILED)
//!       has a matching body (FAILED)
//!   a request to unfriend
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!         "Content-Type" with value "application/json" (OK)
//!       has a matching body (FAILED)
//!
//!
//! Verifying a pact between Consumer2 and happy_provider
//!   Given I am friends with Fred
//!     WARNING: State Change ignored as there is no state change URL
//!   Given I have no friends
//!     WARNING: State Change ignored as there is no state change URL
//!   a request to unfriend but no friends
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!       has a matching body (OK)
//!   a request friends
//!     returns a response which
//!       has status code 200 (FAILED)
//!       includes headers
//!         "Content-Type" with value "application/json" (FAILED)
//!       has a matching body (FAILED)
//!   a request to unfriend
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!         "Content-Type" with value "application/json" (OK)
//!       has a matching body (FAILED)
//!
//!
//! Failures:
//!
//! 0) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has a matching body
//!     expected 'application/json' body but was 'text/plain'
//!
//! 1) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has status code 200
//!     expected 200 but was 404
//!
//! 2) Verifying a pact between Consumer and happy_provider - a request friends returns a response which includes header 'Content-Type' with value 'application/json'
//!     Expected header 'Content-Type' to have value 'application/json' but was 'text/plain'
//!
//! 3) Verifying a pact between Consumer and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
//!     $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"
//!
//!
//! 4) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has a matching body
//!     expected 'application/json' body but was 'text/plain'
//!
//! 5) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has status code 200
//!     expected 200 but was 404
//!
//! 6) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which includes header 'Content-Type' with value 'application/json'
//!     Expected header 'Content-Type' to have value 'application/json' but was 'text/plain'
//!
//! 7) Verifying a pact between Consumer2 and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
//!     $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"
//!
//!
//!
//! There were 8 pact failures
//!
//! ```
//!
//! ## Verifying message pacts
//!
//! Message pacts can be verified, the messages just need to be fetched from an HTTP endpoint. The verifier will send a
//! POST request to the configured provider and expect the message payload in the response. The POST request will include
//! the description and any provider states configured in the Pact file for the message, formatted as JSON.
//!
//! Example POST request:
//!
//! ```json
//! {
//!     "description": "Test Message",
//!     "providerStates":[ {"name": "message exists"} ]
//! }
//! ```
//!
//! ### Verifying metadata
//!
//! Message metadata can be included as base64 encoded key/value pairs in the response, packed into the `Pact-Message-Metadata` HTTP header, and will be compared against any expected metadata in the pact file.
//!
//! The values may contain any valid JSON.
//!
//! For example, given this metadata:
//!
//! ```json
//! {
//!   "Content-Type": "application/json",
//!   "topic": "baz",
//!   "number": 27,
//!   "complex": {
//!     "foo": "bar"
//!   }
//! }
//! ```
//!
//! we would encode it into a base64 string, giving us `ewogICJDb250ZW50LVR5cGUiOiAiYXBwbGljYXRpb24vanNvbiIsCiAgInRvcGljIjogImJheiIsCiAgIm51bWJlciI6IDI3LAogICJjb21wbGV4IjogewogICAgImZvbyI6ICJiYXIiCiAgfQp9Cg==`.
//!
//! ## TLS and Certificate Management
//!
//! Pact uses the [rustls-native-certs](https://lib.rs/crates/rustls-native-certs) crate, which will respect the platform's native certificate store when operating as a TLS client:
//!
//! This is supported on Windows, macOS and Linux:
//!
//! * On Windows, certificates are loaded from the system certificate store. The schannel crate is used to access the Windows certificate store APIs.
//! * On macOS, certificates are loaded from the keychain. The user, admin and system trust settings are merged together as documented by Apple. The security-framework crate is used to access the keystore APIs.
//! * On Linux and other UNIX-like operating systems, the openssl-probe crate is used to discover the filename of the system CA bundle.
//!
//! On Linux the standard OpenSSL environment variables `SSL_CERT_FILE` and `SSL_CERT_DIR` will also be respected.
//!
//! ## Verifying V4 Pact files
//!
//! ### Pact files that require plugins
//!
//! Pact files that require plugins can be verified with version 0.9.0-beta.0+. For details on how plugins work, see the
//! [Pact plugin project](https://github.com/pact-foundation/pact-plugins).
//!
//! Each required plugin is defined in the `plugins` section in the Pact metadata in the Pact file. The plugins will be 
//! loaded from the plugin directory. By default, this is `~/.pact/plugins` or the value of the `PACT_PLUGIN_DIR` environment 
//! variable. Each plugin required by the Pact file must be installed there. You will need to follow the installation 
//! instructions for each plugin, but the default is to unpack the plugin into a sub-directory `<plugin-name>-<plugin-version>`
//! (i.e., for the Protobuf plugin 0.0.0 it will be `protobuf-0.0.0`). The plugin manifest file must be present for the
//! plugin to be able to be loaded.
//!
//! ### Verifying both HTTP and message interactions
//!
//! V4 Pact files can support both HTTP and message-based interactions in the same file. In this case, the be able to 
//! handle the verification for both types of interactions you need to use the `--transports <transports>` option. This will
//! allow configuring different ports to send the different requests to.
//!
//! For example, `--transports http:8080 message:8081` will send HTTP requests to port 8080 and message requests to port 8081.
//! ```

#![warn(missing_docs)]

// Due to large generated future for async fns
#![type_length_limit="100000000"]

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::ArgMatches;
use clap::error::ErrorKind;
use log::{LevelFilter};
use maplit::hashmap;
use pact_models::{PACT_RUST_VERSION, PactSpecification};
use pact_models::prelude::HttpAuth;
use tokio::time::sleep;
use tracing::{debug, debug_span, error, Instrument, warn};
use tracing_subscriber::FmtSubscriber;

use pact_verifier::{
  FilterInfo,
  NullRequestFilterExecutor,
  PactSource,
  ProviderInfo,
  PublishOptions,
  VerificationOptions,
  verify_provider_async,
  ProviderTransport
};
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;
use pact_verifier::metrics::VerificationMetrics;
use pact_verifier::selectors::{consumer_tags_to_selectors, json_to_selectors};
use tracing_log::LogTracer;

mod args;
mod reports;

/// Handles the command line arguments from the running process
pub async fn handle_cli(version: &'static str) -> Result<(), i32> {
  let app = args::setup_app();
  let matches = app
    .arg_required_else_help(true)
    .try_get_matches();

  match matches {
    Ok(results) => handle_matches(&results).await,
    Err(ref err) => {
      match err.kind() {
        ErrorKind::DisplayHelp => {
          let _ = err.print();
          Ok(())
        },
        ErrorKind::DisplayVersion => {
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

async fn handle_matches(matches: &ArgMatches) -> Result<(), i32> {
  let coloured_output = setup_output(matches);

  let provider = configure_provider(matches);
  let source = pact_source(matches);
  let filter = interaction_filter(matches);
  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: matches.get_one::<String>("state-change-url").cloned(),
    state_change_body: !matches.get_flag("state-change-as-query"),
    state_change_teardown: matches.get_flag("state-change-teardown"),
    .. HttpRequestProviderStateExecutor::default()
  });

  let mut custom_headers = hashmap!{};
  if let Some(headers) = matches.get_many::<String>("custom-header") {
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
    disable_ssl_verification: matches.get_flag("disable-ssl-verification"),
    request_timeout: matches.get_one::<u64>("request-timeout").map(|v| *v).unwrap_or(5000),
    custom_headers,
    coloured_output,
    no_pacts_is_error: !matches.get_flag("ignore-no-pacts-error"),
    .. VerificationOptions::default()
  };

  let publish_options = if matches.get_flag("publish") {
    Some(PublishOptions {
      provider_version: matches.get_one::<String>("provider-version").cloned(),
      build_url: matches.get_one::<String>("build-url").cloned(),
      provider_tags: matches.get_many::<String>("provider-tags")
        .map_or_else(Vec::new, |tags| tags.map(|tag| tag.clone()).collect()),
      provider_branch: matches.get_one::<String>("provider-branch").cloned()
    })
  } else {
    None
  };

  for s in &source {
    debug!("Pact source to verify = {}", s);
  };

  let provider_name = provider.name.clone();
  verify_provider_async(
    provider,
    source,
    filter,
    matches.get_many::<String>("filter-consumer").unwrap_or_default().map(|v| v.to_string()).collect::<Vec<_>>(),
    &verification_options,
    publish_options.as_ref(),
    &provider_state_executor,
    Some(VerificationMetrics {
      test_framework: "pact_verifier_cli".to_string(),
      app_name: "pact_verifier_cli".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    }),
  ).instrument(debug_span!("verify_provider", provider_name = provider_name.as_str())).await
    .map_err(|err| {
      error!("Verification failed with error: {}", err);
      2
    })
    .and_then(|result| {
      if let Some(json_file) = matches.get_one::<String>("json-file") {
        if let Err(err) = reports::write_json_report(&result, json_file.as_str()) {
          error!("Failed to write JSON report to '{json_file}' - {err}");
          return Err(2)
        }
      }

      if let Some(_junit_file) = matches.get_one::<String>("junit-file") {
        #[cfg(feature = "junit")]
        if let Err(err) = reports::write_junit_report(&result, _junit_file.as_str(), &provider_name) {
          error!("Failed to write JUnit report to '{_junit_file}' - {err}");
          return Err(2)
        }

        #[cfg(not(feature = "junit"))]
        warn!("junit feature is not enabled, ignoring junit-file option");
      }

      if result.result { Ok(()) } else { Err(1) }
    })
}

fn setup_output(matches: &ArgMatches) -> bool {
  let coloured_output = !matches.get_flag("no-colour");
  let level = matches.get_one::<String>("loglevel").cloned().unwrap_or("warn".to_string());
  let log_level = match level.as_str() {
    "none" => LevelFilter::Off,
    _ => LevelFilter::from_str(level.as_str()).unwrap()
  };
  let _ = LogTracer::builder()
    .with_max_level(log_level)
    .init();

  if matches.get_flag("pretty-log") {
    setup_pretty_log(level.as_str(), coloured_output);
  } else if matches.get_flag("full-log") {
    setup_default_log(level.as_str(), coloured_output);
  } else if matches.get_flag("compact-log") {
    setup_compact_log(level.as_str(), coloured_output);
  } else {
    setup_default_log(level.as_str(), coloured_output);
  };

  coloured_output
}

fn setup_compact_log(level: &str, coloured_output: bool) {
  let subscriber = FmtSubscriber::builder()
    .compact()
    .with_max_level(tracing_core::LevelFilter::from_str(level)
      .unwrap_or(tracing_core::LevelFilter::INFO))
    .with_thread_names(false)
    .with_ansi(coloured_output)
    .finish();

  if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("WARNING: Failed to initialise global tracing subscriber - {err}");
  };
}

fn setup_default_log(level: &str, coloured_output: bool) {
  let subscriber = FmtSubscriber::builder()
    .with_max_level(tracing_core::LevelFilter::from_str(level)
      .unwrap_or(tracing_core::LevelFilter::INFO))
    .with_thread_names(true)
    .with_ansi(coloured_output)
    .finish();

  if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("WARNING: Failed to initialise global tracing subscriber - {err}");
  };
}

fn setup_pretty_log(level: &str, coloured_output: bool) {
  let subscriber = FmtSubscriber::builder()
    .pretty()
    .with_max_level(tracing_core::LevelFilter::from_str(level)
      .unwrap_or(tracing_core::LevelFilter::INFO))
    .with_thread_names(true)
    .with_ansi(coloured_output)
    .finish();

  if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("WARNING: Failed to initialise global tracing subscriber - {err}");
  };
}

#[allow(deprecated)]
pub(crate) fn configure_provider(matches: &ArgMatches) -> ProviderInfo {
  // It is ok to unwrap values here, as they have all been validated by the CLI parser
  let transports = matches.get_many::<(String, u16, Option<String>)>("transports")
    .map(|values| {
      values.map(|(transport, port, base_path)| {
        ProviderTransport {
          transport: transport.to_string(),
          port: Some(*port),
          path: base_path.clone(),
          scheme: None
        }
      }).collect()
    }).unwrap_or_default();
  ProviderInfo {
    host: matches.get_one::<String>("hostname").cloned().unwrap_or("localhost".to_string()),
    port: matches.get_one::<u16>("port").map(|p| *p),
    path: matches.get_one::<String>("base-path").cloned().unwrap_or_default(),
    protocol: matches.get_one::<String>("transport").cloned().unwrap_or("http".to_string()),
    name: matches.get_one::<String>("provider-name").cloned().unwrap_or("provider".to_string()),
    transports,
    ..ProviderInfo::default()
  }
}

fn print_version(version: &str) {
  println!("pact verifier version   : v{}", version);
  println!("pact specification      : v{}", PactSpecification::V4.version_str());
  println!("models version          : v{}", PACT_RUST_VERSION.unwrap_or_default());
}

fn pact_source(matches: &ArgMatches) -> Vec<PactSource> {
  let mut sources = vec![];

  if let Some(webhook_url) = matches.get_one::<String>("webhook-callback-url") {
    let broker_url = matches.get_one::<String>("broker-url").unwrap();
    let auth = matches.get_one::<String>("user").map(|user| {
      HttpAuth::User(user.clone(), matches.get_one::<String>("password").cloned())
    }).or_else(|| matches.get_one::<String>("token").map(|t| HttpAuth::Token(t.clone())));
    sources.push(PactSource::WebhookCallbackUrl {
      pact_url: webhook_url.clone(),
      broker_url: broker_url.clone(),
      auth
    });
  } else {
    if let Some(values) = matches.get_many::<String>("file") {
      sources.extend(values.map(|v| PactSource::File(v.clone())).collect::<Vec<PactSource>>());
    };

    if let Some(values) = matches.get_many::<String>("dir") {
      sources.extend(values.map(|v| PactSource::Dir(v.clone())).collect::<Vec<PactSource>>());
    };

    if let Some(values) = matches.get_many::<String>("url") {
      sources.extend(values.map(|v| {
        if let Some(user) = matches.get_one::<String>("user") {
          PactSource::URL(v.clone(), Some(HttpAuth::User(user.clone(),
                                                         matches.get_one::<String>("password").map(|p| p.clone()))))
        } else if let Some(token) = matches.get_one::<String>("token") {
          PactSource::URL(v.clone(), Some(HttpAuth::Token(token.clone())))
        } else {
          PactSource::URL(v.clone(), None)
        }
      }).collect::<Vec<PactSource>>());
    };

    if let Some(broker_url) = matches.get_one::<String>("broker-url") {
      let name = matches.get_one::<String>("provider-name").cloned().unwrap_or_default();
      let auth = matches.get_one::<String>("user").map(|user| {
        HttpAuth::User(user.clone(), matches.get_one::<String>("password").cloned())
      }).or_else(|| matches.get_one::<String>("token").map(|t| HttpAuth::Token(t.clone())));

      let source = if matches.contains_id("consumer-version-selectors") || matches.contains_id("consumer-version-tags") {
        let pending = matches.get_flag("enable-pending");
        let wip = matches.get_one::<String>("include-wip-pacts-since").cloned();
        let provider_tags = matches.get_many::<String>("provider-tags")
          .map_or_else(Vec::new, |tags| tags.map(|tag| tag.clone()).collect());
        let provider_branch = matches.get_one::<String>("provider-branch").cloned();

        let selectors = if matches.contains_id("consumer-version-selectors") {
          matches.get_many::<String>("consumer-version-selectors")
            .map_or_else(Vec::new, |s| json_to_selectors(s.map(|v| v.as_str()).collect::<Vec<_>>()))
        } else if matches.contains_id("consumer-version-tags") {
          matches.get_many::<String>("consumer-version-tags")
            .map_or_else(Vec::new, |tags| consumer_tags_to_selectors(tags.map(|v| v.as_str()).collect::<Vec<_>>()))
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
  }

  sources
}

fn interaction_filter(matches: &ArgMatches) -> FilterInfo {
  if matches.contains_id("filter-description") &&
    (matches.contains_id("filter-state") || matches.get_flag("filter-no-state")) {
    if let Some(state) = matches.get_one::<String>("filter-state") {
      FilterInfo::DescriptionAndState(matches.get_one::<String>("filter-description").unwrap().clone(),
                                      state.clone())
    } else {
      FilterInfo::DescriptionAndState(matches.get_one::<String>("filter-description").unwrap().clone(),
                                      String::new())
    }
  } else if let Some(desc) = matches.get_one::<String>("filter-description") {
    FilterInfo::Description(desc.clone())
  } else if let Some(state) = matches.get_one::<String>("filter-state") {
    FilterInfo::State(state.clone())
  } else if matches.get_flag("filter-no-state") {
    FilterInfo::State(String::new())
  } else {
    FilterInfo::None
  }
}

fn main() {
  init_windows();

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Could not start a Tokio runtime for running async tasks");

  let result = runtime.block_on(async {
    let result = handle_cli(clap::crate_version!()).await;

    // Add a small delay to let asynchronous tasks to complete
    sleep(Duration::from_millis(500)).await;

    result
  });

  runtime.shutdown_timeout(Duration::from_millis(500));

  if let Err(err) = result {
    std::process::exit(err);
  }
}

#[cfg(windows)]
fn init_windows() {
  if let Err(err) = ansi_term::enable_ansi_support() {
    warn!("Could not enable ANSI console support - {err}");
  }
}

#[cfg(not(windows))]
fn init_windows() { }

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use crate::{args, configure_provider};

  #[test]
  #[allow(deprecated)]
  fn parse_provider_args_defaults() {
    let args = args::setup_app();
    let matches = args.get_matches_from(vec!["test", "-f", "test"]);
    let provider = configure_provider(&matches);

    expect!(provider.host).to(be_equal_to("localhost"));
    expect!(provider.port).to(be_none());
    expect!(provider.name).to(be_equal_to("provider"));
    expect!(provider.path).to(be_equal_to(""));
    expect!(provider.protocol).to(be_equal_to("http"));
  }

  #[test]
  #[allow(deprecated)]
  fn parse_provider_args() {
    let args = args::setup_app();
    let matches = args.get_matches_from(vec![
      "test", "-f", "test", "-h", "test.com", "-p", "1234", "-n", "test", "--transport", "https",
      "--base-path", "/base/path"
    ]);
    let provider = configure_provider(&matches);

    expect!(provider.host).to(be_equal_to("test.com"));
    expect!(provider.port).to(be_some().value(1234));
    expect!(provider.name).to(be_equal_to("test"));
    expect!(provider.path).to(be_equal_to("/base/path"));
    expect!(provider.protocol).to(be_equal_to("https"));
  }

  #[test]
  #[allow(deprecated)]
  fn parse_provider_args_with_old_alias() {
    let args = args::setup_app();
    let matches = args.get_matches_from(vec![
      "test", "-f", "test", "--scheme", "https"
    ]);
    let provider = configure_provider(&matches);

    expect!(provider.protocol).to(be_equal_to("https"));
  }
}
