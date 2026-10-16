//! FFI wrapper code for pact_matching::models::Provider
use libc::c_char;
pub use pact_models::Provider;

use crate::{as_ref, ffi_fn};
use crate::models::Pact;
use crate::util::*;

ffi_fn! {
    /// Get a copy of this provider's name.
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
    /// Provider *provider = pactffi_message_pact_get_provider(message_pact);
    /// if (provider == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// char *name = pactffi_provider_get_name(provider);
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
    fn pactffi_provider_get_name(provider: *const Provider) -> *const c_char {
        let provider = as_ref!(provider);
        string::to_c(&provider.name)? as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get the provider from a Pact. This returns a copy of the provider model, and needs to
    /// be cleaned up with `pactffi_pact_provider_delete` when no longer required.
    ///
    /// # Errors
    ///
    /// This function will fail if it is passed a NULL pointer.
    /// In the case of error, a NULL pointer will be returned.
    fn pactffi_pact_get_provider(pact: *const Pact) -> *const Provider {
        let pact = as_ref!(pact);
        let inner = pact.inner.lock().unwrap();
        let provider = ptr::raw_to(inner.provider());
        provider as *const Provider
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Frees the memory used by the Pact provider
  fn pactffi_pact_provider_delete(provider: *const Provider) {
    ptr::drop_raw(provider as *mut Provider);
  }
}
