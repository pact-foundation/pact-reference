//! In-memory buffer for logging output.

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::sync::Mutex;

use bytes::{BufMut, Bytes, BytesMut};
use lazy_static::lazy_static;
use pact_mock_server::LOG_ID;
use tracing_subscriber::fmt::MakeWriter;

/// In-memory buffer for logging output. Sends output to global static `LOG_BUFFER` in the pact_matching
/// crate. If there is a task local ID found, will accumulate against that ID, otherwise will
/// accumulate against the "global" ID.
#[derive(Debug, Copy, Clone)]
pub(crate) struct InMemBuffer { }

impl Write for InMemBuffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    write_to_log_buffer(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    // no-op
    Ok(())
  }
}

impl <'a> MakeWriter<'a> for InMemBuffer {
  type Writer = InMemBuffer;

  fn make_writer(&'a self) -> Self::Writer {
    *self
  }
}

lazy_static! {
  /// Memory buffer for the buffer logger. This is needed here because there is no
  /// way to get the logger sync from the Dispatch struct. The buffer will be emptied
  /// when the contents is fetched via an FFI call.
  ///
  /// Accumulates the log entries against a task local ID. If the ID is not set, accumulates against
  /// the "global" ID.
  /// cbindgen:ignore
  static ref LOG_BUFFER: Mutex<HashMap<String, BytesMut>> = Mutex::new(HashMap::new());
}

// TODO: This needs to be moved from pact_mock_server, but at the moment the mock server crate
//       relies on it
// task_local! {
//   /// Log ID to accumulate logs against
//   #[allow(missing_docs)]
//   pub static LOG_ID: String;
// }

/// Fetches the contents from the id scoped in-memory buffer and empties the buffer.
pub fn fetch_buffer_contents(id: &str) -> Bytes {
  let mut inner = LOG_BUFFER.lock().unwrap();
  let buffer = inner.entry(id.to_string())
    .or_insert_with(|| BytesMut::with_capacity(256));
  buffer.split().freeze()
}

/// Writes the provided bytes to the task local ID scoped in-memory buffer. If there is no
/// task local ID set, will write to the "global" buffer.
pub fn write_to_log_buffer(buf: &[u8]) {
  let id = LOG_ID
    .try_with(|id| id.clone())
    .unwrap_or_else(|_| "global".into());
  let mut inner = LOG_BUFFER.lock().unwrap();
  let buffer = inner.entry(id)
    .or_insert_with(|| BytesMut::with_capacity(256));
  buffer.put(buf);
}
