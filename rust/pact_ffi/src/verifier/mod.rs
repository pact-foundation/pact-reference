//! The `verifier` module provides a number of exported functions using C bindings for
//! controlling the pact verification process. These can be used in any language that supports C bindings.

#![warn(missing_docs)]

use std::env;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::panic::catch_unwind;
use std::str::from_utf8;

use clap::ArgSettings;
use lazy_static::lazy_static;
use libc::{c_char, c_int, c_uchar, c_ulong, c_ushort, EXIT_FAILURE, EXIT_SUCCESS};
use log::*;
use pact_models::prelude::HttpAuth;
use regex::Regex;
use serde::{Deserialize, Serialize};

use pact_matching::logging::fetch_buffer_contents;
use pact_verifier::selectors::{consumer_tags_to_selectors, json_to_selectors};

use crate::{as_mut, as_ref, ffi_fn, RUNTIME, safe_str};
use crate::ptr;
use crate::util::string::{if_null, optional_str};

mod args;
pub mod verifier;
pub mod handle;

/// External interface to verifier a provider
///
/// * `args` - the same as the CLI interface, except newline delimited
///
/// # Errors
///
/// Errors are returned as non-zero numeric values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | The verification process failed, see output for errors |
/// | 2 | A null pointer was received |
/// | 3 | The method panicked |
/// | 4 | Invalid arguments were provided to the verification process |
///
/// # Safety
///
/// Exported functions are inherently unsafe. Deal.
#[no_mangle]
#[deprecated(since = "0.1.0", note = "use the handle based interface instead. See pact_ffi/src/verifier/handle.rs")]
pub unsafe extern fn pactffi_verify(args: *const c_char) -> i32 {
  if args.is_null() {
    return 2;
  }

  let result = catch_unwind(|| {
    RUNTIME.block_on(async {
      let args_raw = CStr::from_ptr(args).to_string_lossy().into_owned();
      let args: Vec<String> = args_raw.lines().map(|s| s.to_string()).collect();
      #[allow(deprecated)]
      let result = verifier::handle_args(args).await;

      match result {
        Ok(_) => 0,
        Err(e) => e
      }
    })
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      3
    }
  }
}

ffi_fn! {
    /// Get a Handle to a newly created verifier. You should call `pactffi_verifier_shutdown` when
    /// done with the verifier to free all allocated resources.
    ///
    /// By default, verification results will not be published. To enable publishing, use
    /// `pactffi_verifier_set_publish_options` to set the required values and enable it.
    ///
    /// Deprecated: This function is deprecated. Use `pactffi_verifier_new_for_application` which allows the
    /// calling application/framework name and version to be specified.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// Returns NULL on error.
    #[deprecated(since = "0.1.4", note = "Use pactffi_verifier_new_for_application instead")]
    fn pactffi_verifier_new() -> *mut handle::VerifierHandle {
        #[allow(deprecated)]
        let handle = handle::VerifierHandle::new();
        ptr::raw_to(handle)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
  /// Get a Handle to a newly created verifier. You should call `pactffi_verifier_shutdown` when
  /// done with the verifier to free all allocated resources.
  ///
  /// By default, verification results will not be published. To enable publishing, use
  /// `pactffi_verifier_set_publish_options` to set the required values and enable it.
  ///
  /// # Safety
  ///
  /// This function is safe.
  ///
  /// # Error Handling
  ///
  /// Returns NULL on error.
  fn pactffi_verifier_new_for_application(
    name: *const c_char,
    version: *const c_char
  ) -> *mut handle::VerifierHandle {
    let name = if_null(name, "unknown");
    let version = if_null(version, "unknown");
    let handle = handle::VerifierHandle::new_for_application(name.as_str(), version.as_str());
    ptr::raw_to(handle)
  } {
    std::ptr::null_mut()
  }
}

ffi_fn! {
    /// Shutdown the verifier and release all resources
    fn pactffi_verifier_shutdown(handle: *mut handle::VerifierHandle) {
        ptr::drop_raw(handle);
    }
}

ffi_fn! {
    /// Set the provider details for the Pact verifier. Passing a NULL for any field will
    /// use the default value for that field.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_provider_info(
      handle: *mut handle::VerifierHandle,
      name: *const c_char,
      scheme: *const c_char,
      host: *const c_char,
      port: c_ushort,
      path: *const c_char
    ) {
      let handle = as_mut!(handle);
      let name = if_null(name, "provider");
      let scheme = if_null(scheme, "http");
      let host = if_null(host, "localhost");
      let path = if_null(path, "/");

      handle.update_provider_info(name, scheme, host, port as u16, path);
    }
}

ffi_fn! {
    /// Adds a new transport for the given provider. Passing a NULL for any field will
    /// use the default value for that field.
    ///
    /// For non-plugin based message interactions, set protocol to "message" and set scheme
    /// to an empty string or "https" if secure HTTP is required. Communication to the calling
    /// application will be over HTTP to the default provider hostname.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_add_provider_transport(
      handle: *mut handle::VerifierHandle,
      protocol: *const c_char,
      port: c_ushort,
      path: *const c_char,
      scheme: *const c_char
    ) {
      let handle = as_mut!(handle);
      let protocol = if_null(protocol, "");
      let scheme = optional_str(scheme);
      let path = if_null(path, "/");

      handle.add_transport(protocol, port as u16, path, scheme);
    }
}

ffi_fn! {
    /// Set the filters for the Pact verifier.
    ///
    /// If `filter_description` is not empty, it needs to be as a regular expression.
    ///
    /// `filter_no_state` is a boolean value. Set it to greater than zero to turn the option on.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_filter_info(
      handle: *mut handle::VerifierHandle,
      filter_description: *const c_char,
      filter_state: *const c_char,
      filter_no_state: c_uchar
    ) {
      let handle = as_mut!(handle);
      let filter_description = if_null(filter_description, "");
      let filter_state = if_null(filter_state, "");

      handle.update_filter_info(filter_description, filter_state, filter_no_state > 0);
    }
}

ffi_fn! {
    /// Set the provider state URL for the Pact verifier.
    ///
    /// `teardown` is a boolean value. If teardown state change requests should be made after an
    /// interaction is validated (default is false). Set it to greater than zero to turn the
    /// option on.
    /// `body` is a boolean value. Sets if state change request data should be sent in the body
    /// (> 0, true) or as query parameters (== 0, false). Set it to greater than zero to turn the
    /// option on.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_provider_state(
      handle: *mut handle::VerifierHandle,
      url: *const c_char,
      teardown: c_uchar,
      body: c_uchar
    ) {
      let handle = as_mut!(handle);
      let url = if_null(url, "");

      let url = if !url.is_empty() {
        Some(url)
      } else {
        None
      };

      handle.update_provider_state(url, teardown > 0, body > 0);
    }
}

ffi_fn! {
    /// Set the options used by the verifier when calling the provider
    ///
    /// `disable_ssl_verification` is a boolean value. Set it to greater than zero to turn the option on.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_verification_options(
      handle: *mut handle::VerifierHandle,
      disable_ssl_verification: c_uchar,
      request_timeout: c_ulong
    ) -> c_int {
      let handle = as_mut!(handle);

      handle.update_verification_options(disable_ssl_verification > 0, request_timeout as u64);

      EXIT_SUCCESS
    } {
      EXIT_FAILURE
    }
}

ffi_fn! {
    /// Enables or disables coloured output using ANSI escape codes in the verifier output. By default,
    /// coloured output is enabled.
    ///
    /// `coloured_output` is a boolean value. Set it to greater than zero to turn the option on.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the handle pointer points to a valid handle.
    ///
    fn pactffi_verifier_set_coloured_output(
      handle: *mut handle::VerifierHandle,
      coloured_output: c_uchar
    ) -> c_int {
      let handle = as_mut!(handle);

      handle.set_use_coloured_output(coloured_output > 0);

      EXIT_SUCCESS
    } {
      EXIT_FAILURE
    }
}

ffi_fn! {
    /// Enables or disables if no pacts are found to verify results in an error.
    ///
    /// `is_error` is a boolean value. Set it to greater than zero to enable an error when no pacts
    /// are found to verify, and set it to zero to disable this.
    ///
    /// # Safety
    ///
    /// This function is safe as long as the handle pointer points to a valid handle.
    ///
    fn pactffi_verifier_set_no_pacts_is_error(
      handle: *mut handle::VerifierHandle,
      is_error: c_uchar
    ) -> c_int {
      let handle = as_mut!(handle);

      handle.set_no_pacts_is_error(is_error > 0);

      EXIT_SUCCESS
    } {
      EXIT_FAILURE
    }
}

ffi_fn! {
  /// Set the options used when publishing verification results to the Pact Broker. By default,
  /// verification results will not be published unless this function is called.
  ///
  /// # Args
  ///
  /// - `handle` - The pact verifier handle to update
  /// - `provider_version` - Version of the provider to publish
  /// - `build_url` - URL to the build which ran the verification [OPTIONAL]
  /// - `provider_tags` - Collection of tags for the provider [OPTIONAL]
  /// - `provider_tags_len` - Number of provider tags supplied [OPTIONAL]
  /// - `provider_branch` - Name of the branch used for verification [OPTIONAL]
  ///
  /// For optional args, a NULL pointer can be used.
  ///
  /// # Safety
  ///
  /// All string fields must contain valid UTF-8. Invalid UTF-8
  /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
  ///
  fn pactffi_verifier_set_publish_options(
    handle: *mut handle::VerifierHandle,
    provider_version: *const c_char,
    build_url: *const c_char,
    provider_tags: *const *const c_char,
    provider_tags_len: c_ushort,
    provider_branch: *const c_char
  ) -> c_int {
    let handle = as_mut!(handle);
    let provider_version = safe_str!(provider_version);
    let build_url = optional_str(build_url);
    let provider_branch = optional_str(provider_branch);

    let tags = get_vector(provider_tags, provider_tags_len);

    handle.update_publish_options(provider_version, build_url, tags, provider_branch);

    EXIT_SUCCESS
  } {
    EXIT_FAILURE
  }
}

ffi_fn! {
    /// Set the consumer filters for the Pact verifier.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_consumer_filters(
      handle: *mut handle::VerifierHandle,
      consumer_filters: *const *const c_char,
      consumer_filters_len: c_ushort
    ) {
      let handle = as_mut!(handle);

      let consumers = get_vector(consumer_filters, consumer_filters_len);

      handle.update_consumers(consumers);
    }
}

ffi_fn! {
    /// Adds a custom header to be added to the requests made to the provider.
    ///
    /// # Safety
    ///
    /// The header name and value must point to a valid NULL terminated string and must contain
    /// valid UTF-8.
    fn pactffi_verifier_add_custom_header(
      handle: *mut handle::VerifierHandle,
      header_name: *const c_char,
      header_value: *const c_char
    ) {
      let handle = as_mut!(handle);
      let header_name = safe_str!(header_name);
      let header_value = safe_str!(header_value);

      handle.add_custom_header(header_name, header_value);
    }
}

ffi_fn! {
    /// Adds a Pact file as a source to verify.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_add_file_source(
      handle: *mut handle::VerifierHandle,
      file: *const c_char
    ) {
      let handle = as_mut!(handle);
      let file = safe_str!(file);

      handle.add_file_source(file);
    }
}

ffi_fn! {
    /// Adds a Pact directory as a source to verify. All pacts from the directory that match the
    /// provider name will be verified.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_add_directory_source(
      handle: *mut handle::VerifierHandle,
      directory: *const c_char
    ) {
      let handle = as_mut!(handle);
      let directory = safe_str!(directory);

      handle.add_directory_source(directory);
    }
}

ffi_fn! {
    /// Adds a URL as a source to verify. The Pact file will be fetched from the URL.
    ///
    /// If a username and password is given, then basic authentication will be used when fetching
    /// the pact file. If a token is provided, then bearer token authentication will be used.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_url_source(
      handle: *mut handle::VerifierHandle,
      url: *const c_char,
      username: *const c_char,
      password: *const c_char,
      token: *const c_char
    ) {
      let handle = as_mut!(handle);
      let url = safe_str!(url);
      let username = if_null(username, "");
      let password = if_null(password, "");
      let token = if_null(token, "");

      let auth = if !username.is_empty() {
        if !password.is_empty() {
          HttpAuth::User(username, Some(password))
        } else {
          HttpAuth::User(username, None)
        }
      } else if !token.is_empty() {
        HttpAuth::Token(token)
      } else {
        HttpAuth::None
      };

      handle.add_url_source(url, &auth);
    }
}

ffi_fn! {
    /// Adds a Pact broker as a source to verify. This will fetch all the pact files from the broker
    /// that match the provider name.
    ///
    /// If a username and password is given, then basic authentication will be used when fetching
    /// the pact file. If a token is provided, then bearer token authentication will be used.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_broker_source(
      handle: *mut handle::VerifierHandle,
      url: *const c_char,
      username: *const c_char,
      password: *const c_char,
      token: *const c_char
    ) {
      let handle = as_mut!(handle);
      let url = safe_str!(url);
      let username = if_null(username, "");
      let password = if_null(password, "");
      let token = if_null(token, "");

      let auth = if !username.is_empty() {
        if !password.is_empty() {
          HttpAuth::User(username, Some(password))
        } else {
          HttpAuth::User(username, None)
        }
      } else if !token.is_empty() {
        HttpAuth::Token(token)
      } else {
        HttpAuth::None
      };

      handle.add_pact_broker_source(url, false, None, vec![], None, vec![], &auth);
    }
}

ffi_fn! {
    /// Adds a Pact broker as a source to verify. This will fetch all the pact files from the broker
    /// that match the provider name and the consumer version selectors
    /// (See `https://docs.pact.io/pact_broker/advanced_topics/consumer_version_selectors/`).
    ///
    /// The consumer version selectors must be passed in in JSON format.
    ///
    /// `enable_pending` is a boolean value. Set it to greater than zero to turn the option on.
    ///
    /// If the `include_wip_pacts_since` option is provided, it needs to be a date formatted in
    /// ISO format (YYYY-MM-DD).
    ///
    /// If a username and password is given, then basic authentication will be used when fetching
    /// the pact file. If a token is provided, then bearer token authentication will be used.
    ///
    /// This function will return zero unless any of the consumer version selectors are not valid
    /// JSON, in which case, it will return -1.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_broker_source_with_selectors(
      handle: *mut handle::VerifierHandle,
      url: *const c_char,
      username: *const c_char,
      password: *const c_char,
      token: *const c_char,
      enable_pending: c_uchar,
      include_wip_pacts_since: *const c_char,
      provider_tags: *const *const c_char,
      provider_tags_len: c_ushort,
      provider_branch: *const c_char,
      consumer_version_selectors: *const *const c_char,
      consumer_version_selectors_len: c_ushort,
      consumer_version_tags: *const *const c_char,
      consumer_version_tags_len: c_ushort
    ) -> c_int {
      let handle = as_mut!(handle);
      let url = safe_str!(url);
      let provider_branch: Option<String> = if provider_branch.is_null() {
        None
      } else {
        Some(safe_str!(provider_branch).to_string())
      };

      let username = if_null(username, "");
      let password = if_null(password, "");
      let token = if_null(token, "");
      let wip_pacts = if_null(include_wip_pacts_since, "");

      let auth = if !username.is_empty() {
        if !password.is_empty() {
          HttpAuth::User(username, Some(password))
        } else {
          HttpAuth::User(username, None)
        }
      } else if !token.is_empty() {
        HttpAuth::Token(token)
      } else {
        HttpAuth::None
      };

      let wip = if !wip_pacts.is_empty() {
        Some(wip_pacts)
      } else {
        None
      };

      let provider_tags_vector = get_vector(provider_tags, provider_tags_len);
      let consumer_version_selectors_vector = get_vector(consumer_version_selectors, consumer_version_selectors_len);
      let consumer_version_tags_vector = get_vector(consumer_version_tags, consumer_version_tags_len);

      let selectors = if consumer_version_selectors_vector.len() > 0 {
        let mut selectors = vec![];
        let mut errors = false;
        for s in consumer_version_selectors_vector {
          match serde_json::from_str(s.as_str()) {
            Ok(cvs) => selectors.push(cvs),
            Err(err) => {
              error!("Failed to parse consumer version selector '{}' as JSON: {}", s, err);
              errors = true;
            }
          }
        }

        if errors {
          return Ok(-1);
        }

        json_to_selectors(selectors)
      } else if consumer_version_tags_vector.len() > 0 {
        consumer_tags_to_selectors(consumer_version_tags_vector.iter().map(|s| &**s).collect())
      } else {
        vec![]
      };

      handle.add_pact_broker_source(url, enable_pending > 0, wip, provider_tags_vector, provider_branch, selectors, &auth);

      0
    } { -2 }
}

ffi_fn! {
    /// Runs the verification.
    ///
    /// # Error Handling
    ///
    /// Errors will be reported with a non-zero return value.
    fn pactffi_verifier_execute(handle: *mut handle::VerifierHandle) -> c_int {
      let handle = as_mut!(handle);

      handle.execute()
    } {
      EXIT_FAILURE
    }
}

/// Contain the various attributes of an argument given to the verifier
#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short: Option<String>,
    help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    possible_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
    multiple: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<String>,
}

/// Contain the lists of the two types of argument: options and flags
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsFlags {
    /// Arguments which require a parameter, such as loglevel
    pub options: Vec<Argument>,
    /// Arguments which are a bool, such as publish
    pub flags: Vec<Argument>
}

/// External interface to retrieve the options and arguments available when calling the CLI interface,
/// returning them as a JSON string.
///
/// The purpose is to then be able to use in other languages which wrap the FFI library, to implement
/// the same CLI functionality automatically without manual maintenance of arguments, help descriptions
/// etc.
///
/// # Example structure
/// ```json
/// {
///   "options": [
///     {
///       "long": "scheme",
///       "help": "Provider URI scheme (defaults to http)",
///       "possible_values": [
///         "http",
///         "https"
///       ],
///       "default_value": "http"
///       "multiple": false,
///     },
///     {
///       "long": "file",
///       "short": "f",
///       "help": "Pact file to verify (can be repeated)",
///       "multiple": true
///     },
///     {
///       "long": "user",
///       "help": "Username to use when fetching pacts from URLS",
///       "multiple": false,
///       "env": "PACT_BROKER_USERNAME"
///     }
///   ],
///   "flags": [
///     {
///       "long": "disable-ssl-verification",
///       "help": "Disables validation of SSL certificates",
///       "multiple": false
///     }
///   ]
/// }
/// ```
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub extern "C" fn pactffi_verifier_cli_args() -> *const c_char {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let app = args::setup_app(program, clap::crate_version!());

    // Iterate through the args, extracting info from each to then add to a Vector of args
    let mut options: Vec<Argument> = Vec::new();
    let mut flags: Vec<Argument> = Vec::new();

    for opt in app.p.opts.iter() {
        let arg = parse_argument(opt.s.long, opt.s.short, opt.b.help, opt.v.possible_vals.clone(),opt.v.default_val, opt.b.settings.is_set(ArgSettings::Multiple), opt.v.env.clone());
        options.push(arg);
    }

    for opt in app.p.flags.iter() {
        let arg = parse_argument(opt.s.long, opt.s.short, opt.b.help, None, None,opt.b.settings.is_set(ArgSettings::Multiple), None);
        flags.push(arg);
    }


    let opts_flags = OptionsFlags { options, flags };
    let json = serde_json::to_string(&opts_flags).unwrap();
    let c_str = CString::new(json).unwrap();
    c_str.into_raw() as *const c_char
}

fn parse_argument(long: Option<&str>, short: Option<char>, help: Option<&str>, possible_values: Option<Vec<&str>>, default_value: Option<&OsStr>, multiple: bool, env: Option<(&OsStr, Option<OsString>)>) -> Argument {
    let mut arg = Argument { short: None, long: None, help: None, possible_values: None, default_value: None, multiple: Some(false), env: None };

    // Long
    match long {
        None => {}
        Some(val) => {
            arg.long = Some(val.to_string());
        }
    }

    // Short
    match short {
        None => {}
        Some(val) => {
            arg.short = Some(val.to_string());
        }
    }

    // Help
    match help {
        None => {}
        Some(val) => {
            arg.help = Some(val.to_string());
        }
    }

    // Possible values
    match possible_values {
        None => {}
        Some(val) => {
            let mut possible_vals: Vec<String> = Vec::new();
            for possible_val in val.iter() {
                possible_vals.push(possible_val.to_string())
            }
            arg.possible_values = Some(possible_vals);
        }
    }

    // Default value
    match default_value {
        None => {}
        Some(val) =>
            {
                arg.default_value = Some(val.to_os_string().into_string().unwrap());
            }
    }

    // Multiple
    if multiple {
        arg.multiple = Some(true);
    }

    // Env
    match env {
        None => {}
        Some(val) =>
            {
                arg.env = Some(val.0.to_os_string().into_string().unwrap());
            }
    }

    arg
}

fn get_vector(items_ptr: *const *const c_char, items_len: c_ushort) -> Vec<String> {
  if !items_ptr.is_null() && items_len > 0 {
    let mut items = Vec::with_capacity(items_len as usize);
    for index in 0..items_len {
      let item_ptr: *const c_char = unsafe { *(items_ptr.offset(index as isize)) };
      let item = if_null(item_ptr, "");
      if !item.is_empty() {
        items.push(item.to_string());
      }
    }
    items
  } else {
    vec![]
  }
}

fn extract_verifier_logs(name: &str) -> *const c_char {
  let key = format!("verify:{}", name);
  let buffer = fetch_buffer_contents(&key);
  match from_utf8(&buffer) {
    Ok(contents) => {
      match CString::new(contents.to_string()) {
        Ok(c_str) => c_str.into_raw(),
        Err(err) => {
          eprintln!("Failed to copy in-memory log buffer - {}", err);
          std::ptr::null()
        }
      }
    }
    Err(err) => {
      eprintln!("Failed to convert in-memory log buffer to UTF-8 - {}", err);
      std::ptr::null()
    }
  }
}

ffi_fn! {
    /// Extracts the logs for the verification run. This needs the memory buffer log sink to be
    /// setup before the verification is executed. The returned string will need to be freed with
    /// the `free_string` function call to avoid leaking memory.
    ///
    /// Will return a NULL pointer if the logs for the verification can not be retrieved.
    fn pactffi_verifier_logs(handle: *const handle::VerifierHandle) -> *const c_char {
      let handle = as_ref!(handle);
      extract_verifier_logs(&handle.provider_info().name)
    } {
      std::ptr::null()
    }
}

ffi_fn! {
    /// Extracts the logs for the verification run for the provider name. This needs the memory
    /// buffer log sink to be setup before the verification is executed. The returned string will
    /// need to be freed with the `free_string` function call to avoid leaking memory.
    ///
    /// Will return a NULL pointer if the logs for the verification can not be retrieved.
    fn pactffi_verifier_logs_for_provider(provider_name: *const c_char) -> *const c_char {
      let name = safe_str!(provider_name);
      extract_verifier_logs(name)
    } {
      std::ptr::null()
    }
}

lazy_static! {
  static ref ANSI_CODE_RE: Regex = Regex::new("\\x1B\\[(?:;?[0-9]{1,3})+[mGK]").unwrap();
}

ffi_fn! {
    /// Extracts the standard output for the verification run. The returned string will need to be
    /// freed with the `free_string` function call to avoid leaking memory.
    ///
    /// * `strip_ansi` - This parameter controls ANSI escape codes. Setting it to a non-zero value
    /// will cause the ANSI control codes to be stripped from the output.
    ///
    /// Will return a NULL pointer if the handle is invalid.
    fn pactffi_verifier_output(handle: *const handle::VerifierHandle, strip_ansi: c_uchar) -> *const c_char {
      let handle = as_ref!(handle);
      let mut raw_output = handle.output();

      if strip_ansi > 0 {
        raw_output = ANSI_CODE_RE.replace_all(raw_output.as_str(), "").to_string();
      }

      let output = CString::new(raw_output).unwrap();
      output.into_raw() as *const c_char
    } {
      std::ptr::null()
    }
}

ffi_fn! {
    /// Extracts the verification result as a JSON document. The returned string will need to be
    /// freed with the `free_string` function call to avoid leaking memory.
    ///
    /// Will return a NULL pointer if the handle is invalid.
    fn pactffi_verifier_json(handle: *const handle::VerifierHandle) -> *const c_char {
      let handle = as_ref!(handle);
      let output = CString::new(handle.json()).unwrap();
      output.into_raw() as *const c_char
    } {
      std::ptr::null()
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;
  use std::ptr::null;

  use expectest::prelude::*;
  use libc::c_char;

  use crate::verifier::handle::VerifierHandle;
  use crate::verifier::pactffi_verifier_output;

  #[test]
  fn pactffi_verifier_output_test() {
    let output = "  Given [1mtest state[0m
    [33mWARNING: State Change ignored as there is no state change URL provided[0m
00:48:03 [0m[33m[WARN] [0m

Please note:
We are tracking events anonymously to gather important usage statistics like Pact version and operating system. To disable tracking, set the 'PACT_DO_NOT_TRACK' environment variable to 'true'.



Verifying a pact between [1mtest_consumer[0m and [1mtest_provider[0m

  test interaction
      [31mRequest Failed - error sending request for url (http://localhost/): error trying to connect: tcp connect error: Connection refused (os error 111)[0m


Failures:

1) Verifying a pact between test_consumer and test_provider Given test state - test interaction - error sending request for url (http://localhost/): error trying to connect: tcp connect error: Connection refused (os error 111)


There were 1 pact failures

";
    let mut handle = VerifierHandle::new_for_application("tests", "1.0.0");
    handle.set_output(output);

    let result = pactffi_verifier_output(&handle, 0);
    let result2 = pactffi_verifier_output(&handle, 1);

    let out1 = unsafe { CString::from_raw(result as *mut c_char) };
    let out2 = unsafe { CString::from_raw(result2 as *mut c_char) };

    let raw_output = out1.into_string().unwrap();
    let stripped_output = out2.into_string().unwrap();

    expect!(raw_output.as_str()).to(be_equal_to(output));
    expect!(stripped_output.as_str()).to(be_equal_to("  Given test state\n    WARNING: State Change ignored as there is no state change URL provided\n\
00:48:03 [WARN] \n\nPlease note:\n\
We are tracking events anonymously to gather important usage statistics like Pact version and operating system. To disable tracking, set the 'PACT_DO_NOT_TRACK' environment variable to 'true'.\n\
\n\n\nVerifying a pact between test_consumer and test_provider\n\n  test interaction\n      \
      Request Failed - error sending request for url (http://localhost/): error trying to connect: tcp connect error: Connection refused (os error 111)\n\
\n\nFailures:\n\n\
1) Verifying a pact between test_consumer and test_provider Given test state - test interaction - error sending request for url (http://localhost/): error trying to connect: tcp connect error: Connection refused (os error 111)\n\
\n\nThere were 1 pact failures\n\n"));
  }

  #[test]
  fn pactffi_verifier_broker_source_with_selectors_test() {
    let mut handle = VerifierHandle::new_for_application("test", "0.0.0");
    let url = CString::new("http://127.0.0.1:8080").unwrap();
    let provider_branch = CString::new("main").unwrap();
    let cvs_1 = CString::new(r#"{"mainBranch":true}"#).unwrap();
    let cvs_2 = CString::new(r#"{"deployedOrReleased":true}"#).unwrap();
    let consumer_version_selectors = [ cvs_1.as_ptr(), cvs_2.as_ptr() ];
    let result = super::pactffi_verifier_broker_source_with_selectors(
      &mut handle,
      url.as_ptr(),
      null(),
      null(),
      null(),
      0,
      null(),
      null(),
      0,
      provider_branch.as_ptr(),
      consumer_version_selectors.as_ptr(),
      2,
      null(),
      0
    );
    expect!(result).to(be_equal_to(0));
  }

  #[test_log::test]
  fn pactffi_verifier_broker_source_with_selectors_error_test() {
    let mut handle = VerifierHandle::new_for_application("test", "0.0.0");
    let url = CString::new("http://127.0.0.1:8080").unwrap();
    let provider_branch = CString::new("main").unwrap();
    let cvs_1 = CString::new(r#"{"mainBranch":true"#).unwrap();
    let cvs_2 = CString::new(r#"{"deployedOrReleased:true}"#).unwrap();
    let consumer_version_selectors = [ cvs_1.as_ptr(), cvs_2.as_ptr() ];
    let result = super::pactffi_verifier_broker_source_with_selectors(
      &mut handle,
      url.as_ptr(),
      null(),
      null(),
      null(),
      0,
      null(),
      null(),
      0,
      provider_branch.as_ptr(),
      consumer_version_selectors.as_ptr(),
      2,
      null(),
      0
    );
    expect!(result).to(be_equal_to(-1));
  }
}
