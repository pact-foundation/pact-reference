//! A collection of helper functions to make it easier to write the FFI code correctly
//! and consistently.

#[macro_use]
pub mod ffi;
pub(crate) mod ptr;
pub(crate) mod string;
pub(crate) mod write;
