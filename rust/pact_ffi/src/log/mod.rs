//! Sets up a log sink to view logs from the FFI.

mod ffi;
mod level_filter;
mod logger;
mod sink;
mod status;
mod inmem_buffer;
pub(crate) mod plugin_sink;

pub use crate::log::ffi::{
    pactffi_logger_apply,
    pactffi_logger_attach_sink,
    pactffi_logger_init,
    pactffi_fetch_log_buffer,
    pactffi_log_to_stdout,
    pactffi_log_to_stderr,
    pactffi_log_to_file,
    pactffi_log_to_buffer
};
pub use crate::log::inmem_buffer::{
    fetch_buffer_contents,
    write_to_log_buffer
};
