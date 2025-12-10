use alloc::vec::Vec;
use core::mem::MaybeUninit;

use crate::DEFAULT_BUF_SIZE;

pub struct Buffer {
    buf: Vec<u8>,
}

impl Buffer {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn spare_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_slice()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.buf.set_len(new_len);
    }

    #[inline]
    pub fn consume(&mut self, amt: usize) {
        self.buf.drain(..amt);
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.buf.spare_capacity_mut()
    }
}
