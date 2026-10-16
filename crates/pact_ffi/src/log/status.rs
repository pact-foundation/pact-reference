//! Status returned to the C caller for log FFI functions.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use crate::log::sink::SinkSpecifierError;

/// An enum representing the status codes which can be returned to the C caller.
pub(crate) enum Status {
    /// Can't construct sink
    CantConstructSink = -7,

    /// Opening a sink to the given file failed.
    CantOpenSinkToFile = -6,

    /// No file path was specified in the sink specification.
    MissingFilePath = -5,

    /// The sink type specified is not a known type.
    UnknownSinkType = -4,

    /// The sink specifier was not UTF-8 encoded.
    SpecifierNotUtf8 = -3,

    /// No logger has been initialized.
    /// Deprecated: Logging is initialised with defaults.
    #[allow(dead_code)]
    NoLogger = -2,

    /// Can't set the logger
    CantSetLogger = -1,

    /// Operation succeeded.
    Success = 0,
}

impl From<anyhow::Error> for Status {
    fn from(_err: anyhow::Error) -> Status {
        Status::CantSetLogger
    }
}

impl From<SinkSpecifierError> for Status {
    fn from(err: SinkSpecifierError) -> Status {
        match err {
            SinkSpecifierError::UnknownSinkType { .. } => {
                Status::UnknownSinkType
            }
            SinkSpecifierError::MissingFilePath { .. } => {
                Status::MissingFilePath
            }
            SinkSpecifierError::CantMakeFile { .. } => {
                Status::CantOpenSinkToFile
            }
        }
    }
}
