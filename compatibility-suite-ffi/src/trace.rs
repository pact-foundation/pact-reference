//! Per-scenario log of Gherkin steps and the pact_ffi calls each one made. Scenarios run one at a
//! time, so a single global log is enough.

use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

enum Entry {
  Step(String),
  Call(String),
  Note(String),
}

static LOG: Mutex<Vec<Entry>> = Mutex::new(Vec::new());

pub fn reset() {
  LOG.lock().unwrap().clear();
}

pub fn step(keyword: &str, text: &str) {
  LOG.lock().unwrap().push(Entry::Step(format!("{}{}", keyword, text)));
}

pub fn call(call: String, result: impl Display) {
  LOG.lock().unwrap().push(Entry::Call(format!("{call}; // -> {result}")));
}

/// Something the client does outside the FFI, e.g. the HTTP request it sends to the mock server.
pub fn note(note: impl Into<String>) {
  LOG.lock().unwrap().push(Entry::Note(note.into()));
}

/// Writes the log as Markdown. Steps that made no FFI calls are kept so the scenario still reads in full.
pub fn write(path: &Path, feature: &str, scenario: &str) -> std::io::Result<()> {
  let log = LOG.lock().unwrap();
  let mut out = format!("# {feature}\n\n## {scenario}\n");
  let mut open = false;
  for entry in log.iter() {
    match entry {
      Entry::Step(s) => {
        if open { out.push_str("```\n"); open = false; }
        out.push_str(&format!("\n**{}**\n", s.trim()));
      }
      Entry::Call(s) | Entry::Note(s) => {
        if !open { out.push_str("\n```c\n"); open = true; }
        if let Entry::Note(_) = entry { out.push_str("// "); }
        out.push_str(s);
        out.push('\n');
      }
    }
  }
  if open { out.push_str("```\n"); }
  if let Some(dir) = path.parent() { fs::create_dir_all(dir)?; }
  fs::write(path, out)
}
