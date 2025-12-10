use core::mem::MaybeUninit;

use crate::DEFAULT_BUF_SIZE;

pub struct Buffer {
    buf: [MaybeUninit<u8>; DEFAULT_BUF_SIZE],
    pos: usize,
}

impl Buffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; DEFAULT_BUF_SIZE],
            pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn spare_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { self.buf[..self.pos].assume_init_ref() }
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.pos = new_len;
    }

    #[inline]
    pub fn consume(&mut self, amt: usize) {
        assert!(amt <= self.pos);
        let remaining = self.pos - amt;
        for i in 0..remaining {
            self.buf[i] = core::mem::replace(&mut self.buf[i + amt], MaybeUninit::uninit());
        }
        self.pos = remaining;
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr().cast()
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buf[self.pos..]
    }
}
