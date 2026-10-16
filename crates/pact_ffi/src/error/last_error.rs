//! The internal API for setting and getting the last error message.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::cell::RefCell;

thread_local! {
    // The last error to have been reported by the FFI code.
    /// cbindgen:ignore
    pub(crate) static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// Update the last error with a new error message.
#[inline]
pub(crate) fn set_error_msg(e: String) {
    LAST_ERROR.with(|last| {
        *last.borrow_mut() = Some(e);
    });
}

/// Get the last error message if there is one.
#[inline]
pub(crate) fn get_error_msg() -> Option<String> {
    LAST_ERROR.with(|last| last.borrow_mut().take())
}
