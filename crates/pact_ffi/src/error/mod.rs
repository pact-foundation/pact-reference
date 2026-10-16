//! Tools for FFI error reporting and handling.
//!
//! This error handling system is based on the one described in the [unofficial Rust FFI book][book].
//!
//! # How It Works (User's Perspective)
//!
//! FFI code needs to be wrapped in `catch_unwind`, to make sure no panics propogate to the C code calling
//! the FFI. The question then is what to do if an error occurs. The mechanism here is similar to the standard
//! C `errno` pattern, and should be familiar and comfortable for C users of the FFI.
//!
//! The idea is that the FFI functions will return a sentinel value to indicate some error has occurred. If the
//! function is returning a pointer, a null pointer is a sensible sentinel. If the function is returning an
//! integer type, some sensible error-representing integers should be selected and documented for that function.
//!
//! If the C code sees the error sentinel, they then have the option of checking for the last error message
//! using the `get_error_message` function provided in this module. This pulls the last error message string
//! from a thread-local global variable called `LAST_ERROR`, and attempts to put it into a user-provided
//! buffer. If the buffer is a null pointer, or not long enough, or there is no `LAST_ERROR` message, the
//! `get_error_message` function returns an appropriate error sentinel value, which can be checked and handled
//! by the C code.
//!
//! # How It Works (Crate Internal Perspective)
//!
//! All code used for the FFI operations needs to be wrapped in a `catch_unwind`, which captures panics and
//! converts their messages into an error of the type `Box<dyn Any + Send + 'static>`. The error handling code
//! then downcasts that to a string, which is stored in the `LAST_ERROR` global. When the C user requests the
//! last error, that string is written to the buffer they provided if possible.
//!
//! What this means for user code is that errors in `catch_unwind` should be converted into panics with a useful
//! error message so they can be handled by this mechanism.
//!
//! [book]: https://michael-f-bryan.github.io/rust-ffi-guide/errors/index.html "Better Error Handling chapter of the Unofficial Rust FFI book"

mod any_error;
mod error_msg;
mod ffi;
mod last_error;
mod panic;
mod status;

// Function for the C program to read the last error message.
pub use crate::error::ffi::pactffi_get_error_message;

// Utility function for setting the last error message.
pub(crate) use crate::error::last_error::set_error_msg;

// Utility function for convenient panic-catching and error-reporting.
pub(crate) use crate::error::panic::catch_panic;
