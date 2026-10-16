//! The sinks to which logs may be sent.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Stderr, Stdout};
use std::ops::Not;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::debug;

use crate::log::inmem_buffer::InMemBuffer;

/// A sink for logs to be written to, based on a provider specifier.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Sink {
    /// Write logs to stdout.
    Stdout(Stdout),

    /// Write logs to stderr.
    Stderr(Stderr),

    /// Write logs to a file.
    File(File),

    /// Write logs to a thread local memory buffer
    Buffer(InMemBuffer)
}

impl<'a> TryFrom<&'a str> for Sink {
  type Error = SinkSpecifierError;

  fn try_from(s: &'a str) -> Result<Sink, Self::Error> {
    let s = s.trim();

    match s {
      "stdout" => Ok(Sink::Stdout(io::stdout())),
      "stderr" => Ok(Sink::Stderr(io::stderr())),
      "buffer" => Ok(Sink::Buffer(InMemBuffer{})),
      _ => {
        let pat = "file ";
        if s.starts_with(pat).not() {
          Err(SinkSpecifierError::UnknownSinkType {
            name: s.to_owned(),
          })
        } else {
          match s.get(pat.len()..) {
            None => Err(SinkSpecifierError::MissingFilePath),
            Some(remainder) => {
              // PANIC SAFETY: This `unwrap` is fine because the `PathBuf` impl of `FromStr` has an associated
              // `Self::Error` type of `Infallible`, indicating the operation always succeeds.
              let path = PathBuf::from_str(remainder).unwrap();
              if let Some(path_dir) = path.parent() {
                if !path_dir.exists() {
                  debug!("Creating log directory '{}'", path_dir.to_string_lossy());
                  if let Err(source) = fs::create_dir_all(path_dir) {
                    debug!("Creating log directory failed - {}", source);
                    return Err(SinkSpecifierError::CantMakeFile {
                      path,
                      source,
                    });
                  }
                }
              }
              let file = OpenOptions::new().append(true).create(true).open(remainder);
              match file {
                Ok(file) => Ok(Sink::File(file)),
                Err(source) => {
                  Err(SinkSpecifierError::CantMakeFile {
                    path,
                    source,
                  })
                }
              }
            }
          }
        }
      }
    }
  }
}

/// An error arising from attempting to parse a sink specifier string.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SinkSpecifierError {
    #[error("unknown logger sink type (was '{name}', should be \"stdout\", /\"stderr\", /\"buffer\"/or \"file <file path>\")")]
    UnknownSinkType { name: String },

    #[error("missing path in file sink specifier")]
    MissingFilePath,

    #[error("can't make log sink file at path: '{path}'")]
    CantMakeFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use tempfile::tempdir;

  use crate::log::sink::Sink;

  #[test]
  fn try_from_for_file_should_create_any_required_directory() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().join("logs");
    let file_path = dir_path.join("test.log");
    let sink_str = format!("file {}", file_path.to_string_lossy());
    let result = Sink::try_from(sink_str.as_str());
    expect!(result).to(be_ok());
    expect!(dir_path.exists()).to(be_true());
  }
}
