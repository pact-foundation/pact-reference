//! FFI plugin log sink — buffers plugin log entries and forwards them to a registered C callback.

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Mutex, OnceLock};

use libc::c_char;
use pact_plugin_driver::plugin_log_sink::{PluginLogEntry, PluginLogSink};

/// C callback type invoked for each plugin log entry received via the Log RPC or stderr.
///
/// All pointer arguments are valid only for the duration of the call. The callback must
/// not retain them or call back into pact_ffi from within the callback.
pub type PluginLogCallback = unsafe extern "C" fn(
  plugin_instance_id: *const c_char,
  test_run_id: *const c_char,
  level: *const c_char,
  target: *const c_char,
  message: *const c_char,
);

static LOG_CALLBACK: OnceLock<Mutex<Option<PluginLogCallback>>> = OnceLock::new();
static LOG_BUFFER: OnceLock<Mutex<HashMap<String, Vec<PluginLogEntry>>>> = OnceLock::new();

fn callback_cell() -> &'static Mutex<Option<PluginLogCallback>> {
  LOG_CALLBACK.get_or_init(|| Mutex::new(None))
}

pub(crate) fn buffer_cell() -> &'static Mutex<HashMap<String, Vec<PluginLogEntry>>> {
  LOG_BUFFER.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a C callback to be invoked for each plugin log entry.
pub(crate) fn register_callback(cb: PluginLogCallback) {
  *callback_cell().lock().unwrap() = Some(cb);
}

/// Return all buffered log entries for the given plugin instance ID.
pub(crate) fn get_logs(instance_id: &str) -> Vec<PluginLogEntry> {
  buffer_cell()
    .lock()
    .unwrap()
    .get(instance_id)
    .cloned()
    .unwrap_or_default()
}

/// FFI-facing `PluginLogSink` that buffers every entry and optionally forwards to a C callback.
pub(crate) struct FfiPluginLogSink;

impl PluginLogSink for FfiPluginLogSink {
  fn log(&self, entry: &PluginLogEntry) {
    // Always buffer unconditionally so post-test retrieval works even without a callback.
    buffer_cell()
      .lock()
      .unwrap()
      .entry(entry.plugin_instance_id.clone())
      .or_default()
      .push(entry.clone());

    // Forward to the registered callback if one is set.
    if let Some(cb) = *callback_cell().lock().unwrap() {
      let to_c = |s: &str| CString::new(s).unwrap_or_default();
      let instance_id = to_c(&entry.plugin_instance_id);
      let test_run_id = to_c(entry.test_run_id.as_deref().unwrap_or(""));
      let level = to_c(&entry.level);
      let target = to_c(entry.target.as_deref().unwrap_or(""));
      let message = to_c(&entry.message);
      unsafe {
        cb(
          instance_id.as_ptr(),
          test_run_id.as_ptr(),
          level.as_ptr(),
          target.as_ptr(),
          message.as_ptr(),
        );
      }
    }
  }
}
