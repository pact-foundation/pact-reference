//! The FFI-friendly level filter to be applied on a per-sink basis.

use log::LevelFilter as NonCLevelFilter;

// This is exactly equivalent to `LevelFilter` from the `log` crate,
// except that it's `#[repr(C)]`, meaning it's safe to put in the signature
// of a C-exposed FFI function.

/// An enum representing the log level to use.
///
/// This enum is passed to `log_attach_sink`, which defines where to direct
/// log output at the specified level or lower.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<NonCLevelFilter> for LevelFilter {
    fn from(filter: NonCLevelFilter) -> LevelFilter {
        match filter {
            NonCLevelFilter::Off => LevelFilter::Off,
            NonCLevelFilter::Error => LevelFilter::Error,
            NonCLevelFilter::Warn => LevelFilter::Warn,
            NonCLevelFilter::Info => LevelFilter::Info,
            NonCLevelFilter::Debug => LevelFilter::Debug,
            NonCLevelFilter::Trace => LevelFilter::Trace,
        }
    }
}

impl From<LevelFilter> for NonCLevelFilter {
    fn from(filter: LevelFilter) -> NonCLevelFilter {
        match filter {
            LevelFilter::Off => NonCLevelFilter::Off,
            LevelFilter::Error => NonCLevelFilter::Error,
            LevelFilter::Warn => NonCLevelFilter::Warn,
            LevelFilter::Info => NonCLevelFilter::Info,
            LevelFilter::Debug => NonCLevelFilter::Debug,
            LevelFilter::Trace => NonCLevelFilter::Trace,
        }
    }
}
