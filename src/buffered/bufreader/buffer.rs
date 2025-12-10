#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{cmp, io::BorrowedBuf, mem::MaybeUninit};

use crate::{Read, Result, DEFAULT_BUF_SIZE};

pub struct Buffer {
    // The buffer.
    #[cfg(feature = "alloc")]
    buf: Box<[MaybeUninit<u8>]>,
    // The buffer.
    #[cfg(not(feature = "alloc"))]
    buf: [MaybeUninit<u8>; DEFAULT_BUF_SIZE],

    // The current seek offset into `buf`, must always be <= `filled`.
    pos: usize,
    // Each call to `fill_buf` sets `filled` to indicate how many bytes at the start of `buf` are
    // initialized with bytes from a read.
    filled: usize,
    // This is the max number of bytes returned across all `fill_buf` calls. We track this so that we
    // can accurately tell `read_buf` how many bytes of buf are initialized, to bypass as much of its
    // defensive initialization as possible. Note that while this often the same as `filled`, it
    // doesn't need to be. Calls to `fill_buf` are not required to actually fill the buffer, and
    // omitting this is a huge perf regression for `Read` impls that do not.
    initialized: usize,
}

impl Buffer {
    #[inline]
    pub fn new() -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::with_capacity(DEFAULT_BUF_SIZE)
        }
        #[cfg(not(feature = "alloc"))]
        Self {
            buf: [const { MaybeUninit::uninit() }; DEFAULT_BUF_SIZE],
            pos: 0,
            filled: 0,
            initialized: 0,
        }
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub fn with_capacity(capacity: usize) -> Self {
        let buf = Box::new_uninit_slice(capacity);
        Self {
            buf,
            pos: 0,
            filled: 0,
            initialized: 0,
        }
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        // SAFETY: self.pos and self.cap are valid, and self.cap => self.pos, and
        // that region is initialized because those are all invariants of this type.
        unsafe {
            self.buf
                .get_unchecked(self.pos..self.filled)
                .assume_init_ref()
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn filled(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn initialized(&self) -> usize {
        self.initialized
    }

    #[inline]
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }

    #[inline]
    pub fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.filled);
    }

    /// If there are `amt` bytes available in the buffer, pass a slice containing those bytes to
    /// `visitor` and return true. If there are not enough bytes available, return false.
    #[inline]
    pub fn consume_with<V>(&mut self, amt: usize, mut visitor: V) -> bool
    where
        V: FnMut(&[u8]),
    {
        if let Some(claimed) = self.buffer().get(..amt) {
            visitor(claimed);
            // If the indexing into self.buffer() succeeds, amt must be a valid increment.
            self.pos += amt;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn unconsume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_sub(amt);
    }

    /// Read more bytes into the buffer without discarding any of its contents
    pub fn read_more(&mut self, mut reader: impl Read) -> Result<usize> {
        let mut buf = BorrowedBuf::from(&mut self.buf[self.filled..]);
        let old_init = self.initialized - self.filled;
        unsafe {
            buf.set_init(old_init);
        }
        reader.read_buf(buf.unfilled())?;
        self.filled += buf.len();
        self.initialized += buf.init_len() - old_init;
        Ok(buf.len())
    }

    /// Remove bytes that have already been read from the buffer.
    pub fn backshift(&mut self) {
        self.buf.copy_within(self.pos.., 0);
        self.filled -= self.pos;
        self.pos = 0;
    }

    #[inline]
    pub fn fill_buf(&mut self, mut reader: impl Read) -> Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.filled {
            debug_assert!(self.pos == self.filled);

            #[cfg(feature = "alloc")]
            let mut buf = BorrowedBuf::from(&mut *self.buf);
            #[cfg(not(feature = "alloc"))]
            let mut buf = BorrowedBuf::from(self.buf.as_mut_slice());
            // SAFETY: `self.filled` bytes will always have been initialized.
            unsafe {
                buf.set_init(self.initialized);
            }

            let result = reader.read_buf(buf.unfilled());

            self.pos = 0;
            self.filled = buf.len();
            self.initialized = buf.init_len();

            result?;
        }
        Ok(self.buffer())
    }
}
