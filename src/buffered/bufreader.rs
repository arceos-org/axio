use crate::{BufRead, Read, Result};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Default buffer size used by `BufReader` (1 KB)
const DEFAULT_BUF_SIZE: usize = 1024;

/// The `BufReader<R>` struct adds buffering to any reader.
pub struct BufReader<R> {
    inner: R,
    pos: usize,
    filled: usize,
    buf: [u8; DEFAULT_BUF_SIZE],
}

impl<R: Read> BufReader<R> {
    /// Creates a new `BufReader<R>` with a default buffer capacity (1 KB).
    ///
    /// # Examples
    /// ```
    /// use axio::BufReader;
    /// use axio::Read;
    ///
    /// fn example() -> impl Read {
    ///     "test".as_bytes()
    /// }
    ///
    /// let reader = example();
    /// let buf_reader = BufReader::new(reader);
    /// ```
    pub const fn new(inner: R) -> BufReader<R> {
        Self {
            inner,
            pos: 0,
            filled: 0,
            buf: [0; DEFAULT_BUF_SIZE],
        }
    }
}

impl<R> BufReader<R> {
    /// Gets a reference to the underlying reader.
    pub const fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike [`fill_buf`], this will not attempt to fill the buffer if it is empty.
    ///
    /// [`fill_buf`]: BufRead::fill_buf
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.filled]
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    pub const fn capacity(&self) -> usize {
        DEFAULT_BUF_SIZE
    }

    /// Unwraps this `BufReader<R>`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }

    const fn is_empty(&self) -> bool {
        self.pos >= self.filled
    }
}

impl<R: Read> Read for BufReader<R> {
    /// Reads data into the provided buffer, using the internal buffer to
    /// minimize direct reads from the underlying reader
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.is_empty() && buf.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }

    /// Reads exactly enough bytes to fill the buffer, using buffered data first
    /// Small read_exacts from a BufReader are extremely common when used with a deserializer.
    /// The default implementation calls read in a loop, which results in surprisingly poor code
    /// generation for the common path where the buffer has enough bytes to fill the passed-in
    /// buffer.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let amt = buf.len();
        if let Some(claimed) = self.buffer().get(..amt) {
            buf.copy_from_slice(claimed);
            self.pos += amt;
            return Ok(());
        }
        self.inner.read_exact(buf)
    }

    /// Reads all bytes until EOF, appending them to the provided vector
    /// The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    /// delegate to the inner implementation.
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let inner_buf = self.buffer();
        buf.extend_from_slice(inner_buf);
        let nread = inner_buf.len();
        self.discard_buffer();
        Ok(nread + self.inner.read_to_end(buf)?)
    }

    /// Reads all bytes until EOF as UTF-8, appending them to the string
    /// The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    /// delegate to the inner implementation.
    #[cfg(feature = "alloc")]
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        // In the general `else` case below we must read bytes into a side buffer, check
        // that they are valid UTF-8, and then append them to `buf`. This requires a
        // potentially large memcpy.
        //
        // If `buf` is empty--the most common case--we can leverage `append_to_string`
        // to read directly into `buf`'s internal byte buffer, saving an allocation and
        // a memcpy.
        if buf.is_empty() {
            // `append_to_string`'s safety relies on the buffer only being appended to since
            // it only checks the UTF-8 validity of new data. If there were existing content in
            // `buf` then an untrustworthy reader (i.e. `self.inner`) could not only append
            // bytes but also modify existing bytes and render them invalid. On the other hand,
            // if `buf` is empty then by definition any writes must be appends and
            // `append_to_string` will validate all of the new bytes.
            unsafe { crate::append_to_string(buf, |b| self.read_to_end(b)) }
        } else {
            // We cannot append our byte buffer directly onto the `buf` String as there could
            // be an incomplete UTF-8 sequence that has only been partially read. We must read
            // everything into a side buffer first and then call `from_utf8` on the complete
            // buffer.
            let mut bytes = Vec::new();
            self.read_to_end(&mut bytes)?;
            let string = core::str::from_utf8(&bytes).map_err(|_| {
                axerrno::ax_err_type!(InvalidData, "stream did not contain valid UTF-8")
            })?;
            *buf += string;
            Ok(string.len())
        }
    }
}

impl<R: Read> BufRead for BufReader<R> {
    /// Fills the internal buffer if empty and returns its contents
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.is_empty() {
            let read_len = self.inner.read(&mut self.buf)?;
            self.pos = 0;
            self.filled = read_len;
        }
        Ok(self.buffer())
    }

    /// Consumes the specified number of bytes from the buffer
    fn consume(&mut self, amt: usize) {
        self.pos = core::cmp::min(self.pos + amt, self.filled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    struct TempReader {
        data: &'static [u8],
        pos: usize,
    }

    impl TempReader {
        fn new(data: &'static [u8]) -> Self {
            Self { data, pos: 0 }
        }
    }

    impl Read for TempReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let remaining = self.data.len() - self.pos;
            if remaining == 0 {
                return Ok(0);
            }
            let to_copy = core::cmp::min(buf.len(), remaining);
            buf[..to_copy].copy_from_slice(&self.data[self.pos..self.pos + to_copy]);
            self.pos += to_copy;
            Ok(to_copy)
        }
    }

    #[test]
    fn test_get_ref() {
        let reader = TempReader::new(b"test");
        let buf_reader = BufReader::new(reader);
        assert_eq!(buf_reader.get_ref().data, b"test");
    }

    #[test]
    fn test_get_mut() {
        let reader = TempReader::new(b"test");
        let mut buf_reader = BufReader::new(reader);
        assert_eq!(buf_reader.get_mut().data, b"test");
    }

    #[test]
    fn test_buffer_empty() {
        let reader = TempReader::new(b"");
        let buf_reader = BufReader::new(reader);
        assert!(buf_reader.buffer().is_empty());
    }

    #[test]
    fn test_into_inner() {
        let reader = TempReader::new(b"test");
        let buf_reader = BufReader::new(reader);
        let inner = buf_reader.into_inner();
        assert_eq!(inner.data, b"test");
    }

    #[test]
    fn test_read_small() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        let mut buf = [0; 5];
        assert_eq!(buf_reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"hello");

        assert_eq!(buf_reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b" worl");

        assert_eq!(buf_reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"d");

        assert_eq!(buf_reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_read_large() {
        const DATA: &'static [u8] = &[1u8; DEFAULT_BUF_SIZE * 2];
        let reader = TempReader::new(DATA);
        let mut buf_reader = BufReader::new(reader);

        let mut buf = [0u8; DEFAULT_BUF_SIZE * 2];
        assert_eq!(buf_reader.read(&mut buf).unwrap(), DEFAULT_BUF_SIZE * 2);
        assert_eq!(&buf, DATA);
    }

    #[test]
    fn test_read_exact() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        let mut buf = [0; 5];
        buf_reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"hello");

        buf_reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b" worl");

        let mut buf2 = [0; 1];
        buf_reader.read_exact(&mut buf2).unwrap();
        assert_eq!(&buf2, b"d");

        let mut buf3 = [0; 1];
        assert!(buf_reader.read_exact(&mut buf3).is_err());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_read_to_end() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        let mut buf = Vec::new();
        assert_eq!(buf_reader.read_to_end(&mut buf).unwrap(), 11);
        assert_eq!(buf, b"hello world");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_read_to_string() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        let mut buf = String::new();
        assert_eq!(buf_reader.read_to_string(&mut buf).unwrap(), 11);
        assert_eq!(buf, "hello world");
    }

    #[test]
    fn test_fill_buf() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        let buf = buf_reader.fill_buf().unwrap();
        assert_eq!(buf, b"hello world");

        buf_reader.consume(5);
        let buf = buf_reader.fill_buf().unwrap();
        assert_eq!(buf, b" world");
    }

    #[test]
    fn test_consume() {
        let reader = TempReader::new(b"hello world");
        let mut buf_reader = BufReader::new(reader);

        buf_reader.fill_buf().unwrap();
        assert_eq!(buf_reader.buffer(), b"hello world");

        buf_reader.consume(5);
        assert_eq!(buf_reader.buffer(), b" world");

        buf_reader.consume(6);
        assert!(buf_reader.buffer().is_empty());
    }

    #[test]
    fn test_edge_cases() {
        // Empty reader
        let reader = TempReader::new(b"");
        let mut buf_reader = BufReader::new(reader);
        let mut buf = [0; 1];
        assert_eq!(buf_reader.read(&mut buf).unwrap(), 0);

        // Single byte
        let reader = TempReader::new(b"x");
        let mut buf_reader = BufReader::new(reader);
        let mut buf = [0; 1];
        assert_eq!(buf_reader.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], b'x');

        // Exact buffer size
        const DATA: &'static [u8] = &[1u8; DEFAULT_BUF_SIZE];
        let reader = TempReader::new(DATA);
        let mut buf_reader = BufReader::new(reader);
        let mut buf = [0u8; DEFAULT_BUF_SIZE];
        assert_eq!(buf_reader.read(&mut buf).unwrap(), DEFAULT_BUF_SIZE);
        assert_eq!(&buf, DATA);
    }
}
