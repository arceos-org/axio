//! Implementation of core I/O traits for basic types

use crate::{prelude::*, Result};
use core::cmp;

/// Implementation of Read trait for byte slices (&[u8])
///
/// Provides efficient reading operations directly from byte slices without additional buffering.
impl Read for &[u8] {
    /// Reads bytes from the slice into the provided buffer
    ///
    /// Returns the number of bytes read.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let a = &self[..amt];
        let b = &self[amt..];

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    /// Reads all remaining bytes until end of slice
    ///
    /// Need enable `alloc` feature to use this function cause its needs heap allocation.
    #[inline]
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize> {
        buf.extend_from_slice(self);
        let len = self.len();
        *self = &self[len..];
        Ok(len)
    }

    /// Reads exactly the requested number of bytes
    ///
    /// Returns an error if not enough bytes are available.
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.len() {
            return axerrno::ax_err!(UnexpectedEof, "failed to fill whole buffer");
        }
        let amt = buf.len();
        let a = &self[..amt];
        let b = &self[amt..];

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let data = b"hello";
        let mut slice = &data[..];
        let mut buf = [0u8; 5];

        // test full read
        let res = slice.read(&mut buf).unwrap();
        assert_eq!(res, 5);
        assert_eq!(buf, *b"hello");
        assert!(slice.is_empty());

        // test partial read
        let data = b"world";
        let mut slice = &data[..];
        let mut buf = [0u8; 3];
        let res = slice.read(&mut buf).unwrap();
        assert_eq!(res, 3);
        assert_eq!(buf, *b"wor");
        assert_eq!(slice, b"ld");

        // test single-byte read optimization
        let data = b"x";
        let mut slice = &data[..];
        let mut buf = [0u8; 1];
        let res = slice.read(&mut buf).unwrap();
        assert_eq!(res, 1);
        assert_eq!(buf[0], b'x');
        assert!(slice.is_empty());

        // test empty read
        let mut empty_reader = &b""[..];
        assert_eq!(empty_reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_read_partial() {
        let data = b"hello world";
        let mut reader = &data[..];
        let mut buf = [0u8; 5];

        // test full read
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"hello");

        // test partial read
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b" worl");

        // test single-byte read
        let mut small_buf = [0u8; 1];
        assert_eq!(reader.read(&mut small_buf).unwrap(), 1);
        assert_eq!(&small_buf, b"d");
    }

    #[test]
    fn test_read_exact() {
        let data = b"heke";
        let mut slice = &data[..];
        let mut buf = [0u8; 4];

        // test exact read
        slice.read_exact(&mut buf).unwrap();
        assert_eq!(buf, *b"heke");
        assert!(slice.is_empty());

        // test insufficient data
        let mut slice = &data[..];
        let mut buf = [0u8; 5];
        let res = slice.read_exact(&mut buf);
        assert!(res.is_err());
        assert_eq!(slice, b"heke");

        // test single-byte exact read
        let data = b"x";
        let mut slice = &data[..];
        let mut buf = [0u8; 1];
        assert!(slice.read_exact(&mut buf).is_ok());
        assert_eq!(buf[0], b'x');
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_read_to_end() {
        use alloc::vec::Vec;

        let data = b"test";
        let mut slice = &data[..];
        let mut buf = Vec::new();

        // test read all data
        let res = slice.read_to_end(&mut buf).unwrap();
        assert_eq!(res, 4);
        assert_eq!(buf, b"test");
        assert!(slice.is_empty());

        // test empty read
        let mut empty_reader = &b""[..];
        let mut empty_buf = Vec::new();
        assert_eq!(empty_reader.read_to_end(&mut empty_buf).unwrap(), 0);
        assert!(empty_buf.is_empty());
    }
}
