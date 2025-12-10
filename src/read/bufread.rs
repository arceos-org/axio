#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use crate::{Read, Result};

/// A `BufRead` is a type of `Read`er which has an internal buffer, allowing it
/// to perform extra ways of reading.
///
/// See [`std::io::BufRead`] for more details.
pub trait BufRead: Read {
    /// Returns the contents of the internal buffer, filling it with more data, via `Read` methods, if empty.
    fn fill_buf(&mut self) -> Result<&[u8]>;

    /// Marks the given `amount` of additional bytes from the internal buffer as having been read.
    /// Subsequent calls to `read` only return bytes that have not been marked as read.
    fn consume(&mut self, amt: usize);

    /// Checks if there is any data left to be `read`.
    fn has_data_left(&mut self) -> Result<bool> {
        self.fill_buf().map(|b| !b.is_empty())
    }

    /// Skips all bytes until the delimiter `byte` or EOF is reached.
    fn skip_until(&mut self, byte: u8) -> Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = self.fill_buf()?;
                match memchr::memchr(byte, available) {
                    Some(i) => (true, i + 1),
                    None => (false, available.len()),
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }

    /// Read all bytes into `buf` until the delimiter `byte` or EOF is reached.
    #[cfg(feature = "alloc")]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            let (done, used) = {
                let available = self.fill_buf()?;
                match memchr::memchr(byte, available) {
                    Some(i) => {
                        buf.extend_from_slice(&available[..=i]);
                        (true, i + 1)
                    }
                    None => {
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                }
            };
            self.consume(used);
            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }

    /// Read all bytes until a newline (the `0xA` byte) is reached, and append
    /// them to the provided `String` buffer.
    #[cfg(feature = "alloc")]
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        unsafe { super::append_to_string(buf, |b| self.read_until(b'\n', b)) }
    }

    /// Returns an iterator over the contents of this reader split on the byte
    /// `byte`.
    #[cfg(feature = "alloc")]
    fn split(self, byte: u8) -> Split<Self>
    where
        Self: Sized,
    {
        Split {
            buf: self,
            delim: byte,
        }
    }

    /// Returns an iterator over the lines of this reader.
    #[cfg(feature = "alloc")]
    fn lines(self) -> Lines<Self>
    where
        Self: Sized,
    {
        Lines { buf: self }
    }
}

/// An iterator over the contents of an instance of `BufRead` split on a
/// particular byte.
///
/// This struct is generally created by calling [`split`] on a `BufRead`.
/// Please see the documentation of [`split`] for more details.
///
/// [`split`]: BufRead::split
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Split<B> {
    buf: B,
    delim: u8,
}

#[cfg(feature = "alloc")]
impl<B: BufRead> Iterator for Split<B> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.buf.read_until(self.delim, &mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf[buf.len() - 1] == self.delim {
                    buf.pop();
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// An iterator over the lines of an instance of `BufRead`.
///
/// This struct is generally created by calling [`lines`] on a `BufRead`.
/// Please see the documentation of [`lines`] for more details.
///
/// [`lines`]: BufRead::lines
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Lines<B> {
    buf: B,
}

#[cfg(feature = "alloc")]
impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
