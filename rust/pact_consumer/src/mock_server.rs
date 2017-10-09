//! Support for mock HTTP servers that verify pacts.

use pact_matching::models::*;
use pact_mock_server::*;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::prelude::*;
use std::thread;
use url::Url;
use uuid::Uuid;

/// This trait is implemented by types which allow us to start a mock server.
pub trait StartMockServer {
    /// Start a mock server running.
    fn start_mock_server(&self) -> ValidatingMockServer;
}

impl StartMockServer for Pact {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::new(self.clone())
    }
}

/// A mock HTTP server that handles the requests described in a `Pact`, intended
/// for use in tests, and validates that the requests made to that server are
/// correct.
///
/// Because this is intended for use in tests, it will panic if something goes
/// wrong.
pub struct ValidatingMockServer {
    // A description of our mock server, for use in error messages.
    description: String,
    // The ID of our mock server.
    port: i32,
    // The URL of our mock server.
    url: Url,
}

impl ValidatingMockServer {
    /// Create a new mock server which handles requests as described in the
    /// pact.
    pub fn new(pact: Pact) -> ValidatingMockServer {
        let description = format!("{}/{}", pact.consumer.name, pact.provider.name);
        let uuid = Uuid::new_v4().simple().to_string();
        let port = start_mock_server(uuid, pact, 0)
            .expect("error starting mock server");
        let url_str = lookup_mock_server_by_port(port, &|ms| ms.url())
            .expect("could not find mock server");
        ValidatingMockServer {
            description: description,
            port: port,
            url: url_str.parse().expect("invalid mock server URL"),
        }
    }

    /// The URL of our mock server. You can make normal HTTP requests using this
    /// as the base URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Given a path string, return a URL pointing to that path on the mock
    /// server. If the `path` cannot be parsed as URL, **this function will
    /// panic**. For a non-panicking version, call `.url()` instead and build
    /// this path yourself.
    pub fn path<P: AsRef<str>>(&self, path: P) -> Url {
        // We panic here because this a _test_ library, the `?` operator is
        // useless in tests, and filling up our test code with piles of `unwrap`
        // calls is ugly.
        self.url.join(path.as_ref()).expect("could not parse URL")
    }

    /// Helper function called by our `drop` implementation. This basically exists
    /// so that it can return `Err(message)` whenever needed without making the
    /// flow control in `drop` ultra-complex.
    fn drop_helper(&mut self) -> Result<(), String> {
        // Look up any mismatches which occurred.
        let mismatches = lookup_mock_server_by_port(self.port, &|ms| ms.mismatches())
            .ok_or_else(|| "unable to find mock server".to_owned())?;

        if mismatches.is_empty() {
            // Success! Write out the generated pact file.
            lookup_mock_server_by_port(self.port, &|ms| {
                ms.write_pact(&Some("target/pacts".to_owned()))
            })
                .ok_or_else(|| "unable to find mock server".to_owned())?
                .map_err(|err| format!("error writing pact: {}", err))?;
            Ok(())
        } else {
            // Failure. Format our errors.
            let mut msg = format!(
                "mock server {} failed verification:\n",
                self.description,
            );
            for mismatch in mismatches {
                match mismatch {
                    MatchResult::RequestMatch(_) => {
                        unreachable!("list of mismatches contains a match");
                    }
                    MatchResult::RequestMismatch(interaction, mismatches) => {
                        let _ = writeln!(
                            &mut msg,
                            "- interaction {:?}:",
                            interaction.description,
                        );
                        for m in mismatches {
                            let _ = writeln!(&mut msg, "  - {}", m.description());
                        }
                    }
                    MatchResult::RequestNotFound(request) => {
                        let _ = writeln!(&mut msg, "- received unexpected request:");
                        let _ = writeln!(&mut msg, "{:#?}", request);
                    }
                    MatchResult::MissingRequest(interaction) => {
                        let _ = writeln!(
                            &mut msg,
                            "- interaction {:?} expected, but never occurred",
                            interaction.description,
                        );
                        let _ = writeln!(&mut msg, "{:#?}", interaction.request);
                    }
                }
            }
            Err(msg)
        }
    }
}

/// Either panic with `msg`, or if we're already in the middle of a panic,
/// just print `msg` to standard error.
fn panic_or_print_error(msg: &str) {
    if thread::panicking() {
        // The current thread is panicking, so don't try to panic again, because
        // double panics don't print useful explanations of why the test failed.
        // Instead, just print to `stderr`. Ignore any errors, because there's
        // not much we can do if we can't panic and we can't write to `stderr`.
        let _ = writeln!(io::stderr(), "{}", msg);
    } else {
        panic!("{}", msg);
    }
}

impl Drop for ValidatingMockServer {
    fn drop(&mut self) {
        let result = self.drop_helper();
        shutdown_mock_server_by_port(self.port);
        if let Err(msg) = result {
            panic_or_print_error(&msg);
        }
    }
}
