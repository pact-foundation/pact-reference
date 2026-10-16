//! The FFI functions exposed for getting the last error.

use std::slice;

use libc::{c_char, c_int};

use crate::error::last_error::get_error_msg;
use crate::error::status::Status;
use crate::util::write::write_to_c_buf;

/// Provide the error message from `LAST_ERROR` to the calling C code.
///
/// This function should be called after any other function in the pact_matching FFI
/// indicates a failure with its own error message, if the caller wants to get more context
/// on why the error happened.
///
/// Do note that this error-reporting mechanism only reports the top-level error message,
/// not any source information embedded in the original Rust error type. If you want more
/// detailed information for debugging purposes, use the logging interface.
///
/// # Params
///
/// * `buffer`: a pointer to an array of `char` of sufficient length to hold the error message.
/// * `length`: an int providing the length of the `buffer`.
///
/// # Return Codes
///
/// * The number of bytes written to the provided buffer, which may be zero if there is no last error.
/// * `-1` if the provided buffer is a null pointer.
/// * `-2` if the provided buffer length is too small for the error message.
/// * `-3` if the write failed for some other reason.
/// * `-4` if the error message had an interior NULL
///
/// # Notes
///
/// Note that this function zeroes out any excess in the provided buffer.
///
/// # Error Handling
///
/// The return code must be checked for one of the negative number error codes before the buffer
/// is used. If an error code is present, the buffer may not be in a usable state.
///
/// If the buffer is longer than needed for the error message, the excess space will be zeroed
/// as a safety mechanism. This is slightly less efficient than leaving the contents of the buffer
/// alone, but the difference is expected to be negligible in practice.
#[no_mangle]
pub extern "C" fn pactffi_get_error_message(
    buffer: *mut c_char,
    length: c_int,
) -> c_int {
    // Make sure the buffer isn't null.
    if buffer.is_null() {
        return Status::NullBuffer as c_int;
    }

    // Convert the buffer raw pointer into a byte slice.
    let buffer = unsafe {
        slice::from_raw_parts_mut(buffer as *mut u8, length as usize)
    };

    // Get the last error, possibly empty if there isn't one.
    let last_err = get_error_msg().unwrap_or_else(String::new);

    // Try to write the error to the buffer.
    match write_to_c_buf(&last_err, buffer) {
      Ok(bytes) => bytes as c_int,
      Err(err) => Status::from(err) as c_int
    }
}

#[cfg(test)]
mod tests {
  use std::ffi::CStr;
  use expectest::prelude::*;

  use crate::error::{pactffi_get_error_message, set_error_msg};
  use crate::error::last_error::LAST_ERROR;

  #[test]
  fn get_error_message_test() {
    let mut buffer = Vec::with_capacity(16);
    let pointer = buffer.as_mut_ptr();

    set_error_msg("".into());
    let result = pactffi_get_error_message(pointer, 16);
    expect!(result).to(be_equal_to(1));
    let c_str = unsafe { CStr::from_ptr(pointer) };
    expect!(c_str.to_str().unwrap()).to(be_equal_to(""));

    set_error_msg("error".into());
    let result = pactffi_get_error_message(pointer, 16);
    expect!(result).to(be_equal_to(6));
    let c_str = unsafe { CStr::from_ptr(pointer) };
    expect!(c_str.to_str().unwrap()).to(be_equal_to("error"));

    LAST_ERROR.with(|v| v.replace(None));
    let result = pactffi_get_error_message(pointer, 16);
    expect!(result).to(be_equal_to(1));
    let c_str = unsafe { CStr::from_ptr(pointer) };
    expect!(c_str.to_str().unwrap()).to(be_equal_to(""));
  }
}
