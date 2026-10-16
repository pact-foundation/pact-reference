//! Defines an error type representing the message extracted from `AnyError`.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use crate::error::any_error::AnyError;

/// An error message extracted from an `AnyError`.
///
/// This is part of the implement of the LAST_ERROR mechanism, which takes any `AnyError`,
/// attempts to extract an `ErrorMsg` out of it, and then stores the resulting string
/// (from the `ToString` impl implies by `Display`) as the LAST_ERROR message.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorMsg {
    /// A successfully-extracted message.
    #[error("{0}")]
    Message(String),

    /// Could not extract a message, so the error is unknown.
    #[error("an unknown error occured")]
    Unknown,
}

impl From<AnyError> for ErrorMsg {
    fn from(other: AnyError) -> ErrorMsg {
        if let Some(s) = other.downcast_ref::<String>() {
            ErrorMsg::Message(s.clone())
        } else if let Some(s) = other.downcast_ref::<&str>() {
            ErrorMsg::Message(s.to_string())
        } else {
            ErrorMsg::Unknown
        }
    }
}
