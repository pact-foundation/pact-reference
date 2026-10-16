//! FFI wrapper code for iterating over Pact interactions

use std::panic::RefUnwindSafe;

use tracing::trace;

use pact_models::message::Message;
use pact_models::message_pact::MessagePact;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::synch_http::SynchronousHttp;
use pact_models::v4::V4InteractionType;

use crate::{as_mut, ffi_fn};
use crate::models::PactInteraction;
use crate::ptr;

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_message_iter_delete(iter: *mut PactMessageIterator) {
        ptr::drop_raw(iter);
    }
}

/// An iterator over messages in a pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactMessageIterator {
  current: usize,
  message_pact: MessagePact,
}

impl PactMessageIterator {
  /// Create a new iterator given a message pact
  pub fn new(message_pact: MessagePact) -> Self {
    PactMessageIterator {
      current: 0,
      message_pact
    }
  }

  /// Get the next message in the message pact.
  fn next(&mut self) -> Option<&mut Message> {
    let idx = self.current;
    self.current += 1;
    self.message_pact.messages.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next message from the message pact. As the messages returned are owned by the
    /// iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting a message returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_message_iter_next(iter: *mut PactMessageIterator) -> *mut Message {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(message) => message as *mut Message,
          None => {
            trace!("iter past the end of messages");
            std::ptr::null_mut()
          }
      }
    } {
        std::ptr::null_mut()
    }
}

/// An iterator over asynchronous messages in a V4 pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactAsyncMessageIterator {
  current: usize,
  messages: Vec<AsynchronousMessage>
}

impl PactAsyncMessageIterator {
  /// Create a new iterator over all asynchronous messages in the pact
  pub fn new(pact: V4Pact) -> Self {
    PactAsyncMessageIterator {
      current: 0,
      messages: pact.filter_interactions(V4InteractionType::Asynchronous_Messages)
        .iter()
        .map(|i| i.as_v4_async_message().unwrap())
        .collect()
    }
  }

  /// Get the next message in the pact.
  fn next(&mut self) -> Option<&mut AsynchronousMessage> {
    let idx = self.current;
    self.current += 1;
    self.messages.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next asynchronous from the V4 pact. As the messages returned are
    /// owned by the iterator, they do not need to be deleted but will be
    /// cleaned up when the iterator is deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end
    /// of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting a message returned by the iterator can lead to undefined
    /// behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if
    /// an error occurs.
    fn pactffi_pact_async_message_iter_next(iter: *mut PactAsyncMessageIterator) -> *mut AsynchronousMessage {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(message) => message as *mut AsynchronousMessage,
          None => {
            trace!("iter past the end of messages");
            std::ptr::null_mut()
          }
      }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_async_message_iter_delete(iter: *mut PactAsyncMessageIterator) {
        ptr::drop_raw(iter);
    }
}

/// An iterator over synchronous request/response messages in a V4 pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactSyncMessageIterator {
  current: usize,
  messages: Vec<SynchronousMessage>
}

impl PactSyncMessageIterator {
  /// Create a new iterator over all synchronous request/response messages in the pact
  pub fn new(pact: V4Pact) -> Self {
    PactSyncMessageIterator {
      current: 0,
      messages: pact.filter_interactions(V4InteractionType::Synchronous_Messages)
        .iter()
        .map(|i| i.as_v4_sync_message().unwrap())
        .collect()
    }
  }

  /// Get the next message in the pact.
  fn next(&mut self) -> Option<&mut SynchronousMessage> {
    let idx = self.current;
    self.current += 1;
    self.messages.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next synchronous request/response message from the V4 pact. As the messages returned are owned by the
    /// iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting a message returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_sync_message_iter_next(iter: *mut PactSyncMessageIterator) -> *mut SynchronousMessage {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(message) => message as *mut SynchronousMessage,
          None => {
            trace!("iter past the end of messages");
            std::ptr::null_mut()
          }
      }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_sync_message_iter_delete(iter: *mut PactSyncMessageIterator) {
        ptr::drop_raw(iter);
    }
}

/// An iterator over synchronous HTTP request/response interactions in a V4 pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactSyncHttpIterator {
  current: usize,
  interactions: Vec<SynchronousHttp>
}

impl PactSyncHttpIterator {
  /// Create a new iterator over all synchronous HTTP request/response interactions in the V4 pact
  pub fn new(pact: V4Pact) -> Self {
    PactSyncHttpIterator {
      current: 0,
      interactions: pact.filter_interactions(V4InteractionType::Synchronous_HTTP)
        .iter()
        .map(|i| i.as_v4_http().unwrap())
        .collect()
    }
  }

  /// Get the next interaction in the pact.
  fn next(&mut self) -> Option<&mut SynchronousHttp> {
    let idx = self.current;
    self.current += 1;
    self.interactions.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next synchronous HTTP request/response interaction from the V4 pact. As the
    /// interactions returned are owned by the iterator, they do not need to be deleted but
    /// will be cleaned up when the iterator is deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting an interaction returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_sync_http_iter_next(iter: *mut PactSyncHttpIterator) -> *mut SynchronousHttp {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(interaction) => interaction as *mut SynchronousHttp,
          None => {
            trace!("iter past the end of the list");
            std::ptr::null_mut()
          }
        }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_sync_http_iter_delete(iter: *mut PactSyncHttpIterator) {
        ptr::drop_raw(iter);
    }
}

/// An iterator over the interactions in a pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactInteractionIterator {
  current: usize,
  interactions: Vec<PactInteraction>
}

impl PactInteractionIterator {
  /// Create a new iterator over all the interactions in the pact
  pub fn new(pact: Box<dyn pact_models::pact::Pact + Send + Sync + RefUnwindSafe>) -> Self {
    PactInteractionIterator {
      current: 0,
      interactions: pact.interactions().iter()
        .map(|i| PactInteraction::new(i))
        .collect()
    }
  }

  /// Get the next interaction in the pact.
  fn next(&mut self) -> Option<&PactInteraction> {
    let idx = self.current;
    self.current += 1;
    self.interactions.get(idx)
  }
}

ffi_fn! {
    /// Get the next interaction from the pact. As the interactions returned are owned by the
    /// iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// Will return a NULL pointer when the iterator has advanced past the end of the list.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting an interaction returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_interaction_iter_next(iter: *mut PactInteractionIterator) -> *const PactInteraction {
        let iter = as_mut!(iter);
        match iter.next() {
          Some(interaction) => interaction as *const PactInteraction,
          None => {
            trace!("iter past the end of messages");
            std::ptr::null()
          }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_interaction_iter_delete(iter: *mut PactInteractionIterator) {
        ptr::drop_raw(iter);
    }
}
