//! An alternative to `std::panic::catch_unwind` which does error-reporting.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::panic::{catch_unwind, UnwindSafe};

use crate::error::any_error::ToErrorMsg;
use crate::error::last_error::set_error_msg;

/// Convenient panic-catching and reporting.
///
/// This wraps `std::panic::catch_unwind`, but enables you to write functions which return
/// `Result<T, anyhow::Error>` and have those errors correctly reported out.
pub(crate) fn catch_panic<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Result<T, anyhow::Error> + UnwindSafe,
{
    // The return type is Result<Result<T, anyhow::Error>, AnyError>
    let result = catch_unwind(f);

    match result {
        Ok(Ok(value)) => Some(value),
        Ok(Err(err)) => {
            // We have an `anyhow::Error`
            let err = err.to_string();
            eprintln!("Caught panic with error: {}", err);
            set_error_msg(err);
            None
        }
        Err(err) => {
            // We have an `AnyError`
            if let Some(msg) = panic_message::get_panic_message(&err) {
                eprintln!("Caught panic with error: {}", msg);
                set_error_msg(msg.to_string());
            } else {
                let err = err.into_error_msg();
                eprintln!("Caught panic with error: {}", err);
                set_error_msg(err);
            }
            None
        }
    }
}
