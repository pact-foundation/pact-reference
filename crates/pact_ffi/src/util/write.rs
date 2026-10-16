//! API for safely writing Rust `String`s into C strings.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::ffi::{CString, NulError};
use std::io::{self, Write};
use zeroize::Zeroize;

/// Write a string slice to a C buffer safely.
///
/// This performs a write, including the null terminator and performing zeroization of any
/// excess in the destination buffer.
pub(crate) fn write_to_c_buf(
    src: &str,
    dst: &mut [u8],
) -> Result<usize, WriteBufError> {
    // Ensure the string has the null terminator.
    let src = CString::new(src.as_bytes())?;
    let src = src.as_bytes_with_nul();

    // Make sure the destination buffer is big enough.
    check_len(src, dst)?;

    // Perform a zeroized write to the destination buffer.
    let bytes = dst.zeroized_write(src)?;

    Ok(bytes)
}

/// An error arising out of an attempted safe write to a C buffer.
#[derive(Debug, thiserror::Error)]
pub(crate) enum WriteBufError {
    /// The destination buffer is too short.
    #[error("destination buffer too short (needs {src_len} bytes, has {dst_len} bytes)")]
    DstTooShort { src_len: usize, dst_len: usize },

    /// The write failed.
    #[error(transparent)]
    FailedWrite(#[from] io::Error),

    /// The string contained an interior null byte.
    #[error(transparent)]
    InteriorNul(#[from] NulError),
}

fn check_len(src: &[u8], dst: &[u8]) -> Result<(), WriteBufError> {
    // make room for the null terminator.
    let src_len = src.len();
    let dst_len = dst.len();

    if dst_len < src_len {
        Err(WriteBufError::DstTooShort { src_len, dst_len })
    } else {
        Ok(())
    }
}

/// A convenience trait extending `Write` to zeroize the remainder of the buffer.
///
/// Only implemented for `&mut [u8]`.
trait ZeroizedWrite: Write {
    // Because the write is zeroized, no length written is returned,
    // as it will always be the full length of the buffer being written to.
    fn zeroized_write(self, buf: &[u8]) -> io::Result<usize>;
}

impl<'a> ZeroizedWrite for &'a mut [u8] {
    fn zeroized_write(mut self, buf: &[u8]) -> io::Result<usize> {
        // Write the buffer.
        let bytes = self.write(buf)?;

        // Check if there's a remainder and zeroize if there is.
        if let Some(remainder) = self.get_mut(buf.len()..) {
            remainder.iter_mut().zeroize();
        }

        Ok(bytes)
    }
}
