//! Tracing layer which forwards log events to a registered C callback.

use std::ffi::CString;
use std::fmt::Write;
use std::sync::{OnceLock, RwLock};

use libc::c_char;
use pact_mock_server::LOG_ID;
use pact_plugin_driver::test_context::current_test_run_id;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_core::LevelFilter;
use tracing_subscriber::layer::{Context, Layer};

/// C callback type invoked for each log event emitted by the Pact libraries.
///
/// All pointer arguments are valid only for the duration of the call. The callback must
/// not retain them, and must not call back into pact_ffi from within the callback. It is
/// invoked on the thread which emitted the event, which may be a runtime worker thread, so
/// it must be thread-safe.
///
/// `log_id` identifies the operation which emitted the event: the mock server ID while
/// handling a mock server request, `verify:<provider name>` while verifying a provider, or an
/// empty string. `test_run_id` is the value set with `pactffi_set_test_run_id` on the emitting
/// thread, or an empty string. The argument order matches `PluginLogCallback`.
pub type LogCallback = unsafe extern "C" fn(
  log_id: *const c_char,
  test_run_id: *const c_char,
  level: *const c_char,
  target: *const c_char,
  message: *const c_char,
);

struct CallbackState {
  callback: Option<LogCallback>,
  level: LevelFilter,
}

// `CallbackLayer` holds a read guard while the callback runs, so taking the write guard waits
// for every invocation in progress. A callback which calls back into pact_ffi can deadlock.
static STATE: OnceLock<RwLock<CallbackState>> = OnceLock::new();

fn state() -> &'static RwLock<CallbackState> {
  STATE.get_or_init(|| RwLock::new(CallbackState { callback: None, level: LevelFilter::OFF }))
}

/// Register the C callback, or deregister it with `None`.
///
/// Returns once no thread is running the previously registered callback, so the caller may
/// release it.
pub(crate) fn register_callback(callback: Option<LogCallback>) {
  state().write().unwrap().callback = callback;
}

/// Set the most verbose level forwarded to the callback.
pub(crate) fn set_callback_level(level: LevelFilter) {
  state().write().unwrap().level = level;
}

/// The most verbose level forwarded to the callback.
pub(crate) fn callback_level() -> LevelFilter {
  state().read().unwrap().level
}

/// Serialises tests which mutate the global callback state, as the test harness runs them
/// concurrently.
#[cfg(test)]
pub(crate) static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Collects the `message` field and any other fields of an event into a single string.
#[derive(Default)]
struct MessageVisitor {
  message: String,
  fields: String,
}

impl Visit for MessageVisitor {
  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    if field.name() == "message" {
      let _ = write!(self.message, "{:?}", value);
    } else {
      if !self.fields.is_empty() {
        self.fields.push(' ');
      }
      let _ = write!(self.fields, "{}={:?}", field.name(), value);
    }
  }

  fn record_str(&mut self, field: &Field, value: &str) {
    if field.name() == "message" {
      self.message.push_str(value);
    } else {
      if !self.fields.is_empty() {
        self.fields.push(' ');
      }
      let _ = write!(self.fields, "{}={}", field.name(), value);
    }
  }
}

impl MessageVisitor {
  fn into_message(self) -> String {
    match (self.message.is_empty(), self.fields.is_empty()) {
      (_, true) => self.message,
      (true, false) => self.fields,
      (false, false) => format!("{} {}", self.message, self.fields),
    }
  }
}

/// Layer which forwards every event at or below the configured level to the registered
/// callback. Does nothing when no callback is registered.
#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct CallbackLayer;

impl<S: Subscriber> Layer<S> for CallbackLayer {
  fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
    let state = state().read().unwrap();
    let Some(callback) = state.callback else { return };
    let metadata = event.metadata();
    if state.level < *metadata.level() {
      return;
    }

    let mut visitor = MessageVisitor::default();
    event.record(&mut visitor);

    let to_c = |s: &str| CString::new(s).unwrap_or_default();
    let test_run_id = to_c(current_test_run_id().as_deref().unwrap_or(""));
    let log_id = to_c(LOG_ID.try_with(|id| id.clone()).as_deref().unwrap_or(""));
    let level_str = to_c(metadata.level().as_str());
    let target = to_c(metadata.target());
    let message = to_c(&visitor.into_message());
    unsafe {
      callback(
        log_id.as_ptr(),
        test_run_id.as_ptr(),
        level_str.as_ptr(),
        target.as_ptr(),
        message.as_ptr(),
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CStr;
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::Mutex;
  use std::thread;
  use std::time::Duration;

  use expectest::prelude::*;
  use tracing_subscriber::layer::SubscriberExt;

  use super::*;

  static RECEIVED: Mutex<Vec<(String, String, String, String, String)>> = Mutex::new(Vec::new());

  unsafe extern "C" fn capture(
    log_id: *const c_char,
    test_run_id: *const c_char,
    level: *const c_char,
    target: *const c_char,
    message: *const c_char,
  ) {
    let s = |p: *const c_char| CStr::from_ptr(p).to_string_lossy().into_owned();
    RECEIVED.lock().unwrap().push((s(log_id), s(test_run_id), s(level), s(target), s(message)));
  }

  #[test]
  fn forwards_events_at_or_below_the_callback_level() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let subscriber = tracing_subscriber::registry().with(CallbackLayer);
    register_callback(Some(capture));
    set_callback_level(LevelFilter::INFO);
    pact_plugin_driver::test_context::set_test_run_id(Some("run-1".to_string()));

    tracing::subscriber::with_default(subscriber, || {
      tracing::info!(target: "pact_ffi::tests", answer = 42, "hello {}", "world");
      tracing::debug!(target: "pact_ffi::tests", "hidden");
    });

    pact_plugin_driver::test_context::set_test_run_id(None);
    register_callback(None);

    let received = RECEIVED.lock().unwrap().clone();
    expect!(received.len()).to(be_equal_to(1));
    let (log_id, test_run_id, level, target, message) = &received[0];
    expect!(log_id.as_str()).to(be_equal_to(""));
    expect!(test_run_id.as_str()).to(be_equal_to("run-1"));
    expect!(level.as_str()).to(be_equal_to("INFO"));
    expect!(target.as_str()).to(be_equal_to("pact_ffi::tests"));
    expect!(message.as_str()).to(be_equal_to("hello world answer=42"));
  }

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
  fn deregistering_waits_for_a_running_callback() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    register_callback(Some(blocking));
    set_callback_level(LevelFilter::INFO);

    let emitter = thread::spawn(|| {
      let subscriber = tracing_subscriber::registry().with(CallbackLayer);
      tracing::subscriber::with_default(subscriber, || tracing::info!("blocked"));
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
    set_callback_level(LevelFilter::OFF);
    expect!(finished_before_return).to(be_true());
  }

  #[test]
  fn message_visitor_joins_message_and_fields() {
    let mut visitor = MessageVisitor::default();
    visitor.message = "msg".to_string();
    expect!(visitor.into_message()).to(be_equal_to("msg"));

    let mut visitor = MessageVisitor::default();
    visitor.fields = "a=1".to_string();
    expect!(visitor.into_message()).to(be_equal_to("a=1"));
  }
}
