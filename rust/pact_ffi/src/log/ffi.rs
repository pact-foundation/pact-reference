//! The public FFI functions for initializing, adding sinks to, and applying a logger.

use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::ptr;
use std::str::from_utf8;

use libc::{c_char, c_int};
use log::{error, LevelFilter as LogLevelFilter};

use crate::error::set_error_msg;
use crate::log::inmem_buffer::fetch_buffer_contents;
use crate::log::level_filter::LevelFilter;
use crate::log::logger::{add_sink, apply_logger, init_logger};
use crate::log::sink::Sink;
use crate::log::status::Status;
use crate::util::string::to_c;

/// Convenience function to direct all logging to stdout.
///
/// This function is equivalent to using [`pactffi_logger_init`] followed by the
/// [`pactffi_logger_attach_sink`] with the appropriate sink specifier, and then
/// [`pactffi_logger_apply`].
#[no_mangle]
pub extern "C" fn pactffi_log_to_stdout(level_filter: LevelFilter) -> c_int {
    pactffi_logger_init();

    let spec = match CString::new("stdout") {
        Ok(spec) => spec,
        Err(e) => {
            set_error_msg(e.to_string());
            return Status::CantConstructSink as c_int;
        }
    };

    let status = unsafe { pactffi_logger_attach_sink(spec.as_ptr(), level_filter) };
    if status != 0 {
        return status;
    }

    let status = pactffi_logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

/// Convenience function to direct all logging to stderr.
///
/// This function is equivalent to using [`pactffi_logger_init`] followed by the
/// [`pactffi_logger_attach_sink`] with the appropriate sink specifier, and then
/// [`pactffi_logger_apply`].
#[no_mangle]
pub extern "C" fn pactffi_log_to_stderr(level_filter: LevelFilter) -> c_int {
    pactffi_logger_init();

    let spec = match CString::new("stderr") {
        Ok(spec) => spec,
        Err(e) => {
            set_error_msg(e.to_string());
            return Status::CantConstructSink as c_int;
        }
    };

    let status = unsafe { pactffi_logger_attach_sink(spec.as_ptr(), level_filter) };
    if status != 0 {
        return status;
    }

    let status = pactffi_logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

/// Convenience function to direct all logging to a file.
///
///
/// This function is equivalent to using [`pactffi_logger_init`] followed by the
/// [`pactffi_logger_attach_sink`] with the appropriate sink specifier, and then
/// [`pactffi_logger_apply`].
///
/// # Safety
///
/// This function will fail if the file_name pointer is invalid or does not point to a NULL
/// terminated string.
#[no_mangle]
pub unsafe extern "C" fn pactffi_log_to_file(
    file_name: *const c_char,
    level_filter: LevelFilter,
) -> c_int {
    pactffi_logger_init();

    let spec = {
        if file_name.is_null() {
            return Status::CantConstructSink as c_int;
        }

        let file_name =
            match CStr::from_ptr(file_name).to_str() {
                Ok(file_name) => file_name,
                Err(e) => {
                    set_error_msg(e.to_string());
                    return Status::CantConstructSink as c_int;
                }
            };

        let spec = format!("file {}", file_name);

        match CString::new(spec) {
            Ok(spec) => spec,
            Err(e) => {
                set_error_msg(e.to_string());
                return Status::CantConstructSink as c_int;
            }
        }
    };

    let status = pactffi_logger_attach_sink(spec.as_ptr(), level_filter);
    if status != 0 {
        return status;
    }

    let status = pactffi_logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

/// Convenience function to direct all logging to a task local memory buffer.
///
/// This function is equivalent to using [`pactffi_logger_init`] followed by the
/// [`pactffi_logger_attach_sink`] with the appropriate sink specifier, and then
/// [`pactffi_logger_apply`].
///
/// The contents of the buffer can be fetched using
/// [`pactffi_fetch_log_buffer`].
#[no_mangle]
pub extern "C" fn pactffi_log_to_buffer(level_filter: LevelFilter) -> c_int {
  pactffi_logger_init();

  let spec = match CString::new("buffer") {
    Ok(spec) => spec,
    Err(e) => {
      set_error_msg(e.to_string());
      return Status::CantConstructSink as c_int;
    }
  };

  let status = unsafe { pactffi_logger_attach_sink(spec.as_ptr(), level_filter) };
  if status != 0 {
    return status;
  }

  let status = pactffi_logger_apply();
  if status != 0 {
    return status;
  }

  Status::Success as c_int
}

// C API uses something like the pledge API to select write locations, including:
//
// * stdout (`pactffi_logger_attach_sink("stdout", LevelFilter_Info)`)
// * stderr (`pactffi_logger_attach_sink("stderr", LevelFilter_Debug)`)
// * file w/ file path (`pactffi_logger_attach_sink("file /some/file/path", LevelFilter_Trace)`)
//
// The general flow is:
//
// 1. Call `pactffi_logger_init` to create a `Dispatch` struct.
// 2. Call `pactffi_logger_attach_sink` to add an additional sink, using bitflags to set the metadata.
// 3. Call `pactffi_logger_apply` to finalize the logger and enable logging to the configured sinks.
//
// Once `pactffi_logger_apply` has been called, any additional calls to `logger_attach_sink` will fail
// with an error indicating the logger has been applied already.
//
// ```
// pactffi_logger_init();
//
// int result = pactffi_logger_attach_sink("stderr", FilterLevel_Debug);
// /* handle the error */
//
// int result = pactffi_logger_attach_sink("file /some/file/path", FilterLevel_Info);
// /* handle the error */
//
// int result = pactffi_logger_apply();
// /* handle the error */
// ```

/// Initialize the FFI logger with no sinks.
///
/// This initialized logger does nothing until `pactffi_logger_apply` has been called.
///
/// # Usage
///
/// ```c
/// pactffi_logger_init();
/// ```
///
/// # Safety
///
/// This function is always safe to call.
#[no_mangle]
pub extern "C" fn pactffi_logger_init() {
  init_logger();
}

/// Attach an additional sink to the thread-local logger.
///
/// This logger does nothing until `pactffi_logger_apply` has been called.
///
/// Types of sinks can be specified:
///
/// - stdout (`pactffi_logger_attach_sink("stdout", LevelFilter_Info)`)
/// - stderr (`pactffi_logger_attach_sink("stderr", LevelFilter_Debug)`)
/// - file w/ file path (`pactffi_logger_attach_sink("file /some/file/path", LevelFilter_Trace)`)
/// - buffer (`pactffi_logger_attach_sink("buffer", LevelFilter_Debug)`)
///
/// # Usage
///
/// ```c
/// int result = pactffi_logger_attach_sink("file /some/file/path", LogLevel_Filter);
/// ```
///
/// # Error Handling
///
/// The return error codes are as follows:
///
/// - `-1`: Can't set logger (applying the logger failed, perhaps because one is applied already).
/// - `-2`: No logger has been initialized (call `pactffi_logger_init` before any other log function).
/// - `-3`: The sink specifier was not UTF-8 encoded.
/// - `-4`: The sink type specified is not a known type (known types: "stdout", "stderr", "buffer", or "file /some/path").
/// - `-5`: No file path was specified in a file-type sink specification.
/// - `-6`: Opening a sink to the specified file path failed (check permissions).
///
/// # Safety
///
/// This function checks the validity of the passed-in sink specifier, and errors
/// out if the specifier isn't valid UTF-8. Passing in an invalid or NULL pointer will result in
/// undefined behaviour.
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn pactffi_logger_attach_sink(
    sink_specifier: *const c_char,
    level_filter: LevelFilter,
) -> c_int {
    // Get the specifier from the raw C string.
    let sink_specifier = CStr::from_ptr(sink_specifier);
    let sink_specifier = match sink_specifier.to_str() {
        Ok(sink_specifier) => sink_specifier,
        // TODO: Permit non-UTF8 strings, as some filesystems may have non-UTF8
        //       paths to which the user wants to direct the logging output.
        Err(_) => return Status::SpecifierNotUtf8 as c_int,
    };

    // Attempt to construct a sink from the specifier.
    if let Err(err) = Sink::try_from(sink_specifier) {
        return Status::from(err) as c_int;
    }

    // Convert from our `#[repr(C)]` LevelFilter to the one from the `log` crate.
    let level_filter: LogLevelFilter = level_filter.into();

    // Add the sink specifier and level filter to the current log data
    let status = match add_sink(sink_specifier, level_filter) {
        Ok(_) => Status::Success,
        Err(err) => Status::from(err),
    };

    status as c_int
}

/// Apply the previously configured sinks and levels to the program. If no sinks have been setup,
/// will set the log level to info and the target to standard out.
///
/// This function will install a global tracing subscriber. Any attempts to modify the logger
/// after the call to `logger_apply` will fail.
///
/// # Error Handling
///
/// The return error codes are as follows:
///
/// - `-1`: Can't set logger (applying the logger failed, perhaps because one is applied already).
#[no_mangle]
pub extern "C" fn pactffi_logger_apply() -> c_int {
    let status = match apply_logger() {
        Ok(_) => Status::Success,
        Err(err) => Status::from(err),
    };

    status as c_int
}


/// Fetch the in-memory logger buffer contents. This will only have any contents if the `buffer`
/// sink has been configured to log to. The contents will be allocated on the heap and will need
/// to be freed with `pactffi_string_delete`.
///
/// Fetches the logs associated with the provided identifier, or uses the "global" one if the
/// identifier is not specified (i.e. NULL).
///
/// Returns a NULL pointer if the buffer can't be fetched. This can occur is there is not
/// sufficient memory to make a copy of the contents or the buffer contains non-UTF-8 characters.
///
/// # Safety
///
/// This function will fail if the log_id pointer is invalid or does not point to a NULL
/// terminated string.
#[no_mangle]
pub unsafe extern "C" fn pactffi_fetch_log_buffer(log_id: *const c_char) -> *const c_char {
  let id = if log_id.is_null() {
    "global"
  } else {
    CStr::from_ptr(log_id).to_str().unwrap_or("global")
  };
  match from_utf8(&fetch_buffer_contents(&id.to_string())) {
    Ok(contents) => match to_c(contents) {
      Ok(c_str) => c_str,
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

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;

  use crate::log::level_filter::LevelFilter;
  use crate::log::pactffi_logger_attach_sink;

  #[test]
  fn pactffi_logger_attach_sink_with_log_level_off() {
    let sink = CString::new("stderr").unwrap();
    let result = unsafe {
      pactffi_logger_attach_sink(sink.as_ptr(), LevelFilter::Off)
    };
    expect!(result).to(be_equal_to(0));
  }
}
