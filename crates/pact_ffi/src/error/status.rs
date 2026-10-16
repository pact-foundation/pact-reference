//! The possible status codes which error-FFI may return to the C caller.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use crate::util::write::WriteBufError;

/// The status code returned by `get_error_message` to the C caller.
pub(crate) enum Status {
   /// The buffer passed in was a null pointer.
    NullBuffer = -1,

    /// The buffer was too small for the error message.
    BufferTooSmall = -2,

    /// The error message failed to write to the buffer.
    FailedWrite = -3,

    /// The error message contained an interior null terminator.
    InteriorNul = -4,
}

impl From<WriteBufError> for Status {
    fn from(err: WriteBufError) -> Status {
        match err {
            WriteBufError::DstTooShort { .. } => Status::BufferTooSmall,
            WriteBufError::FailedWrite(_) => Status::FailedWrite,
            WriteBufError::InteriorNul(_) => Status::InteriorNul,
        }
    }
}
