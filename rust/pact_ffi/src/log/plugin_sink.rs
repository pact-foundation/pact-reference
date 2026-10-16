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

pub(crate) fn callback_cell() -> &'static Mutex<Option<PluginLogCallback>> {
  LOG_CALLBACK.get_or_init(|| Mutex::new(None))
}

pub(crate) fn buffer_cell() -> &'static Mutex<HashMap<String, Vec<PluginLogEntry>>> {
  LOG_BUFFER.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register the C callback invoked for each plugin log entry, or deregister it with `None`.
///
/// `FfiPluginLogSink` holds the lock while the callback runs, so this returns once no thread is
/// running the previously registered callback and the caller may release it.
pub(crate) fn register_callback(cb: Option<PluginLogCallback>) {
  *callback_cell().lock().unwrap() = cb;
}

/// Serialises tests which mutate the global callback, as the test harness runs them
/// concurrently.
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

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

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::thread;
  use std::time::Duration;

  use expectest::prelude::*;
  use pact_plugin_driver::plugin_log_sink::PluginLogSource;

  use super::*;

  unsafe extern "C" fn noop(
    _: *const c_char, _: *const c_char, _: *const c_char, _: *const c_char, _: *const c_char,
  ) {}

  static ENTERED: AtomicBool = AtomicBool::new(false);
  static RELEASED: AtomicBool = AtomicBool::new(false);
  static FINISHED: AtomicBool = AtomicBool::new(false);

  unsafe extern "C" fn blocking(
    _: *const c_char, _: *const c_char, _: *const c_char, _: *const c_char, _: *const c_char,
  ) {
    ENTERED.store(true, Ordering::SeqCst);
    while !RELEASED.load(Ordering::SeqCst) {
      thread::sleep(Duration::from_millis(1));
    }
    FINISHED.store(true, Ordering::SeqCst);
  }

  #[test]
  fn register_callback_with_none_deregisters() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    register_callback(Some(noop));
    expect!(callback_cell().lock().unwrap().is_some()).to(be_true());
    register_callback(None);
    expect!(callback_cell().lock().unwrap().is_none()).to(be_true());
  }

  #[test]
  fn deregistering_waits_for_a_running_callback() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    register_callback(Some(blocking));

    let emitter = thread::spawn(|| {
      FfiPluginLogSink.log(&PluginLogEntry {
        plugin_name: "test".to_string(),
        plugin_instance_id: "deregistering_waits_for_a_running_callback".to_string(),
        test_run_id: None,
        level: "INFO".to_string(),
        message: "blocked".to_string(),
        target: None,
        timestamp_ms: 0,
        source: PluginLogSource::LogRpc,
      });
    });
    while !ENTERED.load(Ordering::SeqCst) {
      thread::sleep(Duration::from_millis(1));
    }
    let deregister = thread::spawn(|| {
      register_callback(None);
      FINISHED.load(Ordering::SeqCst)
    });
    thread::sleep(Duration::from_millis(50));
    RELEASED.store(true, Ordering::SeqCst);

    let finished_before_return = deregister.join().unwrap();
    emitter.join().unwrap();
    expect!(finished_before_return).to(be_true());
  }
}
