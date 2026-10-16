//! FFI wrapper code for pact_matching::models::Consumer

use libc::c_char;
pub use pact_models::Consumer;

use crate::{as_ref, ffi_fn};
use crate::util::*;
use crate::models::Pact;

ffi_fn! {
    /// Get a copy of this consumer's name.
    ///
    /// The copy must be deleted with `pactffi_string_delete`.
    ///
    /// # Usage
    ///
    /// ```c
    /// // Assuming `file_name` and `json_str` are already defined.
    ///
    /// MessagePact *message_pact = pactffi_message_pact_new_from_json(file_name, json_str);
    /// if (message_pact == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// Consumer *consumer = pactffi_message_pact_get_consumer(message_pact);
    /// if (consumer == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// char *name = pactffi_consumer_get_name(consumer);
    /// if (name == NULL) {
    ///     // handle error.
    /// }
    ///
    /// printf("%s\n", name);
    ///
    /// pactffi_string_delete(name);
    /// ```
    ///
    /// # Errors
    ///
    /// This function will fail if it is passed a NULL pointer,
    /// or the Rust string contains an embedded NULL byte.
    /// In the case of error, a NULL pointer will be returned.
    fn pactffi_consumer_get_name(consumer: *const Consumer) -> *const c_char {
        let consumer = as_ref!(consumer);
        string::to_c(&consumer.name)? as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get the consumer from a Pact. This returns a copy of the consumer model, and needs to
    /// be cleaned up with `pactffi_pact_consumer_delete` when no longer required.
    ///
    /// # Errors
    ///
    /// This function will fail if it is passed a NULL pointer.
    /// In the case of error, a NULL pointer will be returned.
    fn pactffi_pact_get_consumer(pact: *const Pact) -> *const Consumer {
        let pact = as_ref!(pact);
        let inner = pact.inner.lock().unwrap();
        let consumer = ptr::raw_to(inner.consumer());
        consumer as *const Consumer
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Frees the memory used by the Pact consumer
  fn pactffi_pact_consumer_delete(consumer: *const Consumer) {
    ptr::drop_raw(consumer as *mut Consumer);
  }
}
