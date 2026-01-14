//! The `mock_server` module provides a number of exported functions using C bindings for
//! controlling a mock server. These can be used in any language that supports C bindings.
//!
//! ## [create_mock_server](fn.create_mock_server_ffi.html)
//!
//! External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
//! as well as the port for the mock server to run on. A value of 0 for the port will result in a
//! port being allocated by the operating system. The port of the mock server is returned.
//!
//! ## [mock_server_matched](fn.mock_server_matched_ffi.html)
//!
//! Simple function that returns a boolean value given the port number of the mock service. This value will be true if all
//! the expectations of the pact that the mock server was created with have been met. It will return false if any request did
//! not match, an un-recognised request was received or an expected request was not received.
//!
//! ## [mock_server_mismatches](fn.mock_server_mismatches_ffi.html)
//!
//! This returns all the mismatches, un-expected requests and missing requests in JSON format, given the port number of the
//! mock server.
//!
//! **IMPORTANT NOTE:** The JSON string for the result is allocated on the Rust heap, and will have to be freed once the
//! code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is provided for this purpose.
//! If the mock server is not cleaned up properly, this will result in memory leaks as the Rust heap will not be reclaimed.
//!
//! ## [cleanup_mock_server](fn.cleanup_mock_server.html)
//!
//! This function will try terminate the mock server with the given port number and cleanup any memory allocated for it by
//! the [`mock_server_mismatches`](fn.mock_server_mismatches.html) function. Returns `true`, unless
//! a mock server with the given port number does not exist, or the function fails in some way.
//!
//! **NOTE:** Although `close()` on the listener for the mock server is called, this does not currently work and the
//! listener will continue handling requests. In this case, it will always return a 501 once the mock server has been
//! cleaned up.
//!
//! ## [write_pact_file](fn.write_pact_file.html)
//!
//! External interface to trigger a mock server to write out its pact file. This function should
//! be called if all the consumer tests have passed. The directory to write the file to is passed
//! as the second parameter. If a NULL pointer is passed, the current working directory is used.
//!
//! Returns 0 if the pact file was successfully written. Returns a positive code if the file can
//! not be written, or there is no mock server running on that port or the function panics.

#![warn(missing_docs)]

use std::ffi::{CStr, CString};
use std::net::ToSocketAddrs;
use std::ptr;
use std::str::{self, from_utf8};
use std::sync::Mutex;

use chrono::Local;
use either::Either;
use libc::c_char;
use onig::Regex;
use rand::Rng;
use serde_json::{json, Value};
use tracing::{error, info, warn};
use uuid::Uuid;

use pact_matching::metrics::{MetricEvent, send_metrics};
use pact_mock_server::{
    WritePactFileErr,
    builder::MockServerBuilder,
    mock_server::{MockServer, MockServerConfig},
    server_manager::ServerManager
};
use pact_models::{
    generators::GeneratorCategory,
    matchingrules::{Category, MatchingRuleCategory},
    pact::{Pact, ReadWritePact},
    time_utils::{parse_pattern, to_chrono_pattern}
};
use pact_plugin_driver::plugin_manager::get_mock_server_results;

use crate::{convert_cstr, ffi_fn, safe_str};
use crate::log::fetch_buffer_contents;
use crate::mock_server::handles::{PactHandle, path_from_dir};
use crate::string::optional_str;

pub mod handles;
pub mod bodies;
mod xml;
mod form_urlencoded;

/// Mutex to protect access to the server manager
static MANAGER: Mutex<Option<ServerManager>> = Mutex::new(None);

/// Attach the mock server builder to the global server manager and spawn the
/// mock server.
fn attach_to_manager(builder: MockServerBuilder) -> anyhow::Result<Either<MockServer, (String, u16)>> {
  let mut guard = MANAGER.lock().unwrap();
  let manager = guard.get_or_insert_with(ServerManager::new);
  manager.spawn_mock_server(builder)
}

/// Fetch the CA Certificate used to generate the self-signed certificate for the TLS mock server.
///
/// **NOTE:** The string for the result is allocated on the heap, and will have to be freed
/// by the caller using pactffi_string_delete.
///
/// # Errors
///
/// An empty string indicates an error reading the pem file.
#[no_mangle]
pub extern "C" fn pactffi_get_tls_ca_certificate() -> *mut c_char  {
  let cert_file = include_str!("ca.pem");
  let cert_str = CString::new(cert_file).unwrap_or_default();

  cert_str.into_raw()
}

ffi_fn! {
  /// Create a mock server for the provided Pact handle and transport. If the transport is not
  /// provided (it is a NULL pointer or an empty string), will default to an HTTP transport. The
  /// address is the interface bind to, and will default to the loopback adapter if not specified.
  /// Specifying a value of zero for the port will result in the operating system allocating the port.
  ///
  /// Parameters:
  /// * `pact` - Handle to a Pact model created with created with `pactffi_new_pact`.
  /// * `addr` - Address to bind to (i.e. `127.0.0.1` or `[::1]`). Must be a valid UTF-8 NULL-terminated string, or NULL or empty, in which case the loopback adapter is used.
  /// * `port` - Port number to bind to. A value of zero will result in the operating system allocating an available port.
  /// * `transport` - The transport to use (i.e. http, https, grpc). Must be a valid UTF-8 NULL-terminated string, or NULL or empty, in which case http will be used.
  /// * `transport_config` - (OPTIONAL) Configuration for the transport as a valid JSON string. Set to NULL or empty if not required.
  ///
  /// The port of the mock server is returned.
  ///
  /// # Safety
  /// NULL pointers or empty strings can be passed in for the address, transport and transport_config,
  /// in which case a default value will be used. Passing in an invalid pointer will result in undefined behaviour.
  ///
  /// # Errors
  ///
  /// Errors are returned as negative values.
  ///
  /// | Error | Description |
  /// |-------|-------------|
  /// | -1 | An invalid handle was received. Handles should be created with `pactffi_new_pact` |
  /// | -2 | transport_config is not valid JSON |
  /// | -3 | The mock server could not be started |
  /// | -4 | The method panicked |
  /// | -5 | The address is not valid |
  ///
  #[tracing::instrument(level = "trace")]
  async fn pactffi_create_mock_server_for_transport(
    pact: PactHandle,
    addr: *const c_char,
    port: u16,
    transport: *const c_char,
    transport_config: *const c_char
  ) -> i32 {
    let addr = safe_str!(addr);
    let transport = optional_str(transport).unwrap_or_else(|| "http".to_string());

    let transport_config = match optional_str(transport_config).map(|config| str::parse::<Value>(config.as_str())) {
      None => Ok(None),
      Some(result) => match result {
        Ok(value) => Ok(Some(MockServerConfig::from_json(&value))),
        Err(err) => {
          error!("Failed to parse transport_config as JSON - {}", err);
          Err(-2)
        }
      }
    };

    match transport_config {
      Ok(transport_config) => if let Ok(mut socket_addr) = (addr, port).to_socket_addrs() {
        // Seems ok to unwrap this here, as it doesn't make sense that to_socket_addrs will return
        // a success with an iterator that is empty
        let socket_addr = socket_addr.next().unwrap();
        pact.with_pact(&move |_, inner| {
          let transport_config = transport_config.clone();
          let config = MockServerConfig {
            pact_specification: inner.specification_version,
            .. transport_config.unwrap_or_default()
          };

          let builder = MockServerBuilder::new()
            .with_pact(inner.pact.boxed())
            .with_config(config)
            .with_id(Uuid::new_v4().to_string())
            .bind_to(socket_addr.to_string());

          let builder = match builder.with_transport(transport.as_str()) {
            Ok(builder) => builder,
            Err(err) => {
              error!("Failed to configure mock server transport '{}' - {}", transport, err);
              return -3;
            }
          };

          match attach_to_manager(builder) {
            Ok(Either::Left(mock_server)) => {
              inner.mock_server_started = true;
              mock_server.port() as i32
            },
            Ok(Either::Right((_id, port))) => {
              inner.mock_server_started = true;
              port as i32
            },
            Err(err) => {
              error!("Failed to start mock server - {}", err);
              -3
            }
          }
        }).unwrap_or(-1)
      } else {
        error!("Failed to parse '{}', {} as an address", addr, port);
        -5
      }
      Err(err) => err
    }
  } {
    -4
  }
}

ffi_fn! {
    /// External interface to check if a mock server has matched all its requests. The port number is
    /// passed in, and if all requests have been matched, true is returned. False is returned if there
    /// is no mock server on the given port, or if any request has not been successfully matched, or
    /// the method panics.
    #[tracing::instrument(level = "trace")]
    fn pactffi_mock_server_matched(mock_server_port: i32) -> bool {
        let mut guard = MANAGER.lock().unwrap();
        let manager = guard.get_or_insert_with(ServerManager::new);
        manager.mock_server_matched_by_port(mock_server_port as u16)
          .unwrap_or(false)
    }
    {
        false
    }
}

ffi_fn! {
    /// External interface to get all the mismatches from a mock server. The port number of the mock
    /// server is passed in, and a pointer to a C string with the mismatches in JSON format is
    /// returned.
    ///
    /// **NOTE:** The JSON string for the result is allocated on the heap, and will have to be freed
    /// once the code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is
    /// provided for this purpose.
    ///
    /// # Errors
    ///
    /// If there is no mock server with the provided port number, or the function panics, a NULL
    /// pointer will be returned. Don't try to dereference it, it will not end well for you.
    ///
    #[tracing::instrument(level = "trace")]
    fn pactffi_mock_server_mismatches(mock_server_port: i32) -> *mut c_char {
        let mut guard = MANAGER.lock().unwrap();
        let manager = guard.get_or_insert_with(ServerManager::new);
        let mismatches = manager.mock_server_mismatches_by_port(mock_server_port as u16);

        match mismatches {
            Ok(Some(results)) => {
                let str = Value::Array(results).to_string();
                match CString::new(str) {
                  Ok(s) => {
                      let p = s.as_ptr() as *mut _;
                      manager.store_mock_server_resource(mock_server_port as u16, s);
                      p
                  }
                  Err(err) => {
                      error!("Failed to copy mismatches result - {}", err);
                      ptr::null_mut()
                  }
                }
            }
            Ok(None) => ptr::null_mut(),
            Err(err) => {
              error!("Request to plugin to get matching results failed - {}", err);
              ptr::null_mut()
            }
        }
    } {
        ptr::null_mut()
    }
}

ffi_fn! {
    /// External interface to cleanup a mock server. This function will try terminate the mock server
    /// with the given port number and cleanup any memory allocated for it. Returns true, unless a
    /// mock server with the given port number does not exist, or the function panics.
    fn pactffi_cleanup_mock_server(mock_server_port: i32) -> bool {
        let mut guard = MANAGER.lock().unwrap();
        let manager = guard.get_or_insert_with(ServerManager::new);
        let id = manager.find_mock_server_by_port(mock_server_port as u16, &|_, id, mock_server| {
            let interactions = match mock_server {
                Either::Left(ms) => ms.pact.interactions().len(),
                Either::Right(ms) => ms.pact.interactions.len()
            };
            send_metrics(MetricEvent::ConsumerTestRun {
                interactions,
                test_framework: "pact_ffi".to_string(),
                app_name: "pact_ffi".to_string(),
                app_version: env!("CARGO_PKG_VERSION").to_string()
            });
            id.clone()
        });
        if let Some(id) = id {
            manager.shutdown_mock_server_by_id(id)
        } else {
            false
        }
    }
    {
        false
    }
}

ffi_fn! {
    /// External interface to trigger a mock server to write out its pact file. This function should
    /// be called if all the consumer tests have passed. The directory to write the file to is passed
    /// as the second parameter. If a NULL pointer is passed, the current working directory is used.
    ///
    /// If overwrite is true, the file will be overwritten with the contents of the current pact.
    /// Otherwise, it will be merged with any existing pact file.
    ///
    /// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
    /// not be written, or there is no mock server running on that port or the function panics.
    ///
    /// # Errors
    ///
    /// Errors are returned as positive values.
    ///
    /// | Error | Description |
    /// |-------|-------------|
    /// | 1 | A general panic was caught |
    /// | 2 | The pact file was not able to be written |
    /// | 3 | A mock server with the provided port was not found |
    fn pactffi_write_pact_file(mock_server_port: i32, directory: *const c_char, overwrite: bool) -> i32 {
        let dir = path_from_dir(directory, None);
        let path = dir.map(|path| path.into_os_string().into_string().unwrap_or_default());

        let mut guard = MANAGER.lock().unwrap();
        let manager = guard.get_or_insert_with(ServerManager::new);
        let directory = path.clone();
        let write_result = manager.find_mock_server_by_port(mock_server_port as u16, &|_, _, mock_server| {
            match mock_server {
                Either::Left(mock_server) => {
                    mock_server.write_pact(&directory, overwrite)
                    .map(|_| ())
                    .map_err(|err| {
                        error!("Failed to write pact to file - {}", err);
                        WritePactFileErr::IOError
                    })
                }
                Either::Right(plugin_mock_server) => {
                    let mut pact = plugin_mock_server.pact.clone();
                    pact.add_md_version("mockserver", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
                    let pact_file_name = ReadWritePact::default_file_name(&pact);
                    let filename = match directory.clone() {
                        Some(path) => {
                            let mut path = std::path::PathBuf::from(path);
                            path.push(pact_file_name);
                            path
                        },
                        None => std::path::PathBuf::from(pact_file_name)
                    };

                    info!("Writing pact out to '{}'", filename.display());
                    match pact_models::pact::write_pact(pact.boxed(), filename.as_path(), pact_models::PactSpecification::V4, overwrite) {
                        Ok(_) => Ok(()),
                        Err(err) => {
                            warn!("Failed to write pact to file - {}", err);
                            Err(WritePactFileErr::IOError)
                        }
                    }
                }
            }
        });

        match write_result {
            None => 3,
            Some(Ok(())) => 0,
            Some(Err(err)) => match err {
                WritePactFileErr::IOError => 2,
                WritePactFileErr::NoMockServer => 3
            }
        }
    }
    {
        1
    }
}

ffi_fn! {
    /// Fetch the logs for the mock server. This needs the memory buffer log sink to be setup before
    /// the mock server is started. Returned string will be freed with the `cleanup_mock_server`
    /// function call.
    ///
    /// Will return a NULL pointer if the logs for the mock server can not be retrieved.
    fn pactffi_mock_server_logs(mock_server_port: i32) -> *const c_char {
        let mut guard = MANAGER.lock().unwrap();
        let manager = guard.get_or_insert_with(ServerManager::new);
        let logs = manager.find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
            fetch_buffer_contents(&mock_server.id)
        });
        match logs {
            Some(bytes) => {
                match from_utf8(&bytes) {
                    Ok(contents) => match CString::new(contents.to_string()) {
                        Ok(c_str) => {
                            let p = c_str.as_ptr();
                            manager.store_mock_server_resource(mock_server_port as u16, c_str);
                            p
                        },
                        Err(err) => {
                            error!("Failed to copy in-memory log buffer - {}", err);
                            ptr::null()
                        }
                    }
                    Err(err) => {
                        error!("Failed to convert in-memory log buffer to UTF-8 = {}", err);
                        ptr::null()
                    }
                }
            }
            None => {
                error!("No mock server found for port {}", mock_server_port);
                ptr::null()
            }
        }
    }
    {
        ptr::null()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Result of wrapping a string value
pub enum StringResult {
  /// Was generated OK
  Ok(*mut c_char),
  /// There was an error generating the string
  Failed(*mut c_char)
}

/// Generates a datetime value from the provided format string, using the current system date and time
/// NOTE: The memory for the returned string needs to be freed with the `pactffi_string_delete` function
///
/// # Safety
///
/// If the format string pointer is NULL or has invalid UTF-8 characters, an error result will be
/// returned. If the format string pointer is not a valid pointer or is not a NULL-terminated string,
/// this will lead to undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn pactffi_generate_datetime_string(format: *const c_char) -> StringResult {
  if format.is_null() {
    let error = CString::new("generate_datetime_string: format is NULL").unwrap();
    StringResult::Failed(error.into_raw())
  } else {
    let c_str = CStr::from_ptr(format);
    match c_str.to_str() {
      Ok(s) => match parse_pattern(s) {
        Ok(pattern_tokens) => {
          let result = Local::now().format(to_chrono_pattern(&pattern_tokens).as_str()).to_string();
          let result_str = CString::new(result.as_str()).unwrap();
          StringResult::Ok(result_str.into_raw())
        },
        Err(err) => {
          let error = format!("Error parsing '{}': {:?}", s, err);
          let error_str = CString::new(error.as_str()).unwrap();
          StringResult::Failed(error_str.into_raw())
        }
      },
      Err(err) => {
        let error = format!("generate_datetime_string: format is not a valid UTF-8 string: {:?}", err);
        let error_str = CString::new(error.as_str()).unwrap();
        StringResult::Failed(error_str.into_raw())
      }
    }
  }
}

/// Checks that the example string matches the given regex.
///
/// # Safety
///
/// Both the regex and example pointers must be valid pointers to NULL-terminated strings. Invalid
/// pointers will result in undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn pactffi_check_regex(regex: *const c_char, example: *const c_char) -> bool {
  if regex.is_null() {
    false
  } else {
    let c_str = CStr::from_ptr(regex);
    match c_str.to_str() {
      Ok(regex) => {
        let example = convert_cstr("example", example).unwrap_or_default();
        match Regex::new(regex) {
          Ok(re) => re.is_match(example),
          Err(err) => {
            error!("check_regex: '{}' is not a valid regular expression - {}", regex, err);
            false
          }
        }
      },
      Err(err) => {
        error!("check_regex: regex is not a valid UTF-8 string: {:?}", err);
        false
      }
    }
  }
}

/// Generates an example string based on the provided regex.
pub fn generate_regex_value_internal(regex: &str) -> Result<String, String> {
  let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
  match parser.parse(regex) {
    Ok(hir) => {
      let mut rnd = rand::rng();
      match rand_regex::Regex::with_hir(hir, 20) {
        Ok(re) => Ok(rnd.sample(re)),
        Err(err) => Err(format!("generate_regex_value: '{}' is not a valid regular expression - {}", regex, err))
      }
    },
    Err(err) => Err(format!("generate_regex_value: '{}' is not a valid regular expression - {}", regex, err))
  }
}

/// Generates an example string based on the provided regex.
/// NOTE: The memory for the returned string needs to be freed with the `pactffi_string_delete` function.
///
/// # Safety
///
/// The regex pointer must be a valid pointer to a NULL-terminated string. Invalid pointers will
/// result in undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn pactffi_generate_regex_value(regex: *const c_char) -> StringResult {
  if regex.is_null() {
    let error = CString::new("generate_regex_value: regex is NULL").unwrap();
    StringResult::Failed(error.into_raw())
  } else {
    let c_str = CStr::from_ptr(regex);
    match c_str.to_str() {
      Ok(regex) => match generate_regex_value_internal(regex) {
        Ok(val) => {
          let result_str = CString::new(val.as_str()).unwrap();
          StringResult::Ok(result_str.into_raw())
        },
        Err(err) => {
          let error = CString::new(err).unwrap();
          StringResult::Failed(error.into_raw())
        }
      },
      Err(err) => {
        let error = CString::new(format!("generate_regex_value: regex is not a valid UTF-8 string: {:?}", err)).unwrap();
        StringResult::Failed(error.into_raw())
      }
    }
  }
}

/// [DEPRECATED] Frees the memory allocated to a string by another function
///
/// This function is deprecated. Use `pactffi_string_delete` instead.
///
/// # Safety
///
/// The string pointer can be NULL (which is a no-op), but if it is not a valid pointer the call
/// will result in undefined behaviour.
#[no_mangle]
#[deprecated(since = "0.1.0", note = "Use pactffi_string_delete instead")]
pub unsafe extern "C" fn pactffi_free_string(s: *mut c_char) {
  if s.is_null() {
    return;
  }
  drop(CString::from_raw(s));
}

pub(crate) fn generator_category(matching_rules: &mut MatchingRuleCategory) -> &GeneratorCategory {
  match matching_rules.name {
    Category::BODY => &GeneratorCategory::BODY,
    Category::HEADER => &GeneratorCategory::HEADER,
    Category::PATH => &GeneratorCategory::PATH,
    Category::QUERY => &GeneratorCategory::QUERY,
    Category::METADATA => &GeneratorCategory::METADATA,
    Category::STATUS => &GeneratorCategory::STATUS,
    _ => {
      warn!("invalid generator category {} provided, defaulting to body", matching_rules.name);
      &GeneratorCategory::BODY
    }
  }
}
