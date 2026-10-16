//! Represents the state of providers in a message.

use anyhow::anyhow;
use libc::c_char;
use serde_json::Value as JsonValue;
use tracing::trace;

use pact_models::provider_states::ProviderState;

use crate::{as_mut, as_ref, ffi_fn};
use crate::util::*;

ffi_fn! {
    /// Get the name of the provider state as a string, which needs to be deleted with `pactffi_string_delete`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the provider_state param is NULL, this returns NULL.
    fn pactffi_provider_state_get_name(provider_state: *const ProviderState) -> *const c_char {
        let provider_state = as_ref!(provider_state);
        let name = string::to_c(&provider_state.name)?;
        name as *const c_char
    } {
        std::ptr::null()
    }
}

ffi_fn! {
    /// Get an iterator over the params of a provider state.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the provider state, and must
    /// not outlive the provider state.
    ///
    /// The provider state params also must not be modified during iteration. If it is,
    /// the old iterator must be deleted and a new iterator created.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain
    /// embedded null ('\0') bytes.
    fn pactffi_provider_state_get_param_iter(
        provider_state: *mut ProviderState
    ) -> *mut ProviderStateParamIterator {
        let provider_state = as_mut!(provider_state);

        let iter = ProviderStateParamIterator {
            keys:  provider_state.params.keys().cloned().collect(),
            current: 0,
            provider_state: provider_state as *const ProviderState,
        };

        ptr::raw_to(iter)
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Get the next key and value out of the iterator, if possible
    ///
    /// Returns a pointer to a heap allocated array of 2 elements, the pointer to the
    /// key string on the heap, and the pointer to the value string on the heap.
    ///
    /// # Safety
    ///
    /// The underlying data must not be modified during iteration.
    ///
    /// The user needs to free both the contained strings and the array.
    ///
    /// # Error Handling
    ///
    /// Returns NULL if there's no further elements or the iterator is NULL.
    fn pactffi_provider_state_param_iter_next(
        iter: *mut ProviderStateParamIterator
    ) -> *mut ProviderStateParamPair {
        let iter = as_mut!(iter);
        let provider_state = as_ref!(iter.provider_state);
        match iter.next() {
          Some(key) => {
            let (key, value) = provider_state
                .params
                .get_key_value(key)
                .ok_or(anyhow!("iter provided invalid param key"))?;
            let pair = ProviderStateParamPair::new(key, value)?;
            ptr::raw_to(pair)
          }
          None => {
            trace!("iter past the end of params");
            std::ptr::null_mut()
          }
        }
    } {
        std::ptr::null_mut()
    }
}

ffi_fn! {
    /// Free the provider state when you're done using it.
    fn pactffi_provider_state_delete(provider_state: *mut ProviderState) {
        ptr::drop_raw(provider_state);
    }
}

ffi_fn! {
    /// Free the provider state param iterator when you're done using it.
    fn pactffi_provider_state_param_iter_delete(iter: *mut ProviderStateParamIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Free a pair of key and value returned from `pactffi_provider_state_param_iter_next`.
    fn pactffi_provider_state_param_pair_delete(pair: *mut ProviderStateParamPair) {
        ptr::drop_raw(pair);
    }
}

/// An iterator that enables FFI iteration over provider state params by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying provider state happens while the iterator is live.
#[derive(Debug)]
pub struct ProviderStateParamIterator {
    /// The provider state param keys
    keys: Vec<String>,
    /// The current key
    current: usize,
    /// Pointer to the provider state.
    provider_state: *const ProviderState,
}

impl ProviderStateParamIterator {
    fn next(&mut self) -> Option<&String> {
        let idx = self.current;
        self.current += 1;
        self.keys.get(idx)
    }
}

/// A single key-value pair exported to the C-side.
#[derive(Debug)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct ProviderStateParamPair {
    /// The key of the `ProviderState` parameter.
    key: *const c_char,
    /// The value of the `ProviderState` parameter.
    value: *const c_char,
}

impl ProviderStateParamPair {
    fn new(
        key: &str,
        value: &JsonValue,
    ) -> anyhow::Result<ProviderStateParamPair> {
        let value = value.to_string();

        Ok(ProviderStateParamPair {
            key: string::to_c(key)? as *const c_char,
            value: string::to_c(&value)? as *const c_char,
        })
    }
}
