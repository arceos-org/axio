#[cfg(feature = "alloc")]
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::io::BorrowedCursor;

use crate::IoBuf;

// =============================================================================
// Forwarding implementations

impl<R: IoBuf + ?Sized> IoBuf for &mut R {
    #[inline]
    fn remaining(&self) -> usize {
        (**self).remaining()
    }
}

#[cfg(feature = "alloc")]
impl<R: IoBuf + ?Sized> IoBuf for Box<R> {
    #[inline]
    fn remaining(&self) -> usize {
        (**self).remaining()
    }
}

// =============================================================================
// In-memory buffer implementations

impl IoBuf for &[u8] {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }
}

impl IoBuf for &mut [u8] {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "alloc")]
impl IoBuf for Vec<u8> {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "alloc")]
impl IoBuf for VecDeque<u8> {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }
}

impl IoBuf for BorrowedCursor<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.capacity()
    }
}
