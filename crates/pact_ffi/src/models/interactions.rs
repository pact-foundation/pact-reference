//! FFI functions to support Pact interaction models.

use pact_models::message::Message;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::synch_http::SynchronousHttp;

use crate::{as_ref, ffi_fn};
use crate::models::PactInteraction;
use crate::util::ptr;

ffi_fn! {
  /// Casts this interaction to a `SynchronousHttp` interaction. Returns a NULL pointer if the
  /// interaction can not be casted to a `SynchronousHttp` interaction (for instance, it is a
  /// message interaction). The returned pointer must be freed with `pactffi_sync_http_delete`
  /// when no longer required.
  ///
  /// # Safety
  /// This function is safe as long as the interaction pointer is a valid pointer.
  ///
  /// # Errors
  /// On any error, this function will return a NULL pointer.
  fn pactffi_pact_interaction_as_synchronous_http(interaction: *const PactInteraction) -> *const SynchronousHttp {
    let interaction = as_ref!(interaction);
    let inner = interaction.inner.lock().unwrap();
    if let Some(http) = inner.as_v4_http() {
      ptr::raw_to(http)
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Casts this interaction to a `Message` interaction. Returns a NULL pointer if the
  /// interaction can not be casted to a `Message` interaction (for instance, it is a
  /// http interaction). The returned pointer must be freed with `pactffi_message_delete`
  /// when no longer required.
  ///
  /// Note that if the interaction is a V4 `AsynchronousMessage`, it will be converted to a V3
  /// `Message` before being returned.
  ///
  /// # Safety
  /// This function is safe as long as the interaction pointer is a valid pointer.
  ///
  /// # Errors
  /// On any error, this function will return a NULL pointer.
  fn pactffi_pact_interaction_as_message(interaction: *const PactInteraction) -> *const Message {
    let interaction = as_ref!(interaction);
    let inner = interaction.inner.lock().unwrap();
    if let Some(message) = inner.as_message() {
      ptr::raw_to(message)
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Casts this interaction to a `AsynchronousMessage` interaction. Returns a NULL pointer if the
  /// interaction can not be casted to a `AsynchronousMessage` interaction (for instance, it is a
  /// http interaction). The returned pointer must be freed with `pactffi_async_message_delete`
  /// when no longer required.
  ///
  /// Note that if the interaction is a V3 `Message`, it will be converted to a V4
  /// `AsynchronousMessage` before being returned.
  ///
  /// # Safety
  /// This function is safe as long as the interaction pointer is a valid pointer.
  ///
  /// # Errors
  /// On any error, this function will return a NULL pointer.
  fn pactffi_pact_interaction_as_asynchronous_message(interaction: *const PactInteraction) -> *const AsynchronousMessage {
    let interaction = as_ref!(interaction);
    let inner = interaction.inner.lock().unwrap();
    if let Some(message) = inner.as_v4_async_message() {
      ptr::raw_to(message)
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}

ffi_fn! {
  /// Casts this interaction to a `SynchronousMessage` interaction. Returns a NULL pointer if the
  /// interaction can not be casted to a `SynchronousMessage` interaction (for instance, it is a
  /// http interaction). The returned pointer must be freed with `pactffi_sync_message_delete`
  /// when no longer required.
  ///
  /// # Safety
  /// This function is safe as long as the interaction pointer is a valid pointer.
  ///
  /// # Errors
  /// On any error, this function will return a NULL pointer.
  fn pactffi_pact_interaction_as_synchronous_message(interaction: *const PactInteraction) -> *const SynchronousMessage {
    let interaction = as_ref!(interaction);
    let inner = interaction.inner.lock().unwrap();
    if let Some(message) = inner.as_v4_sync_message() {
      ptr::raw_to(message)
    } else {
      std::ptr::null()
    }
  } {
    std::ptr::null()
  }
}
