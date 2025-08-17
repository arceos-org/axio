use core::{cmp, mem};

use axerrno::ax_bail;

use crate::{
    buf::{Buf, BufMut},
    Read, Result, Write,
};

impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

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

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.len() {
            ax_bail!(Io, "failed to fill whole buffer");
        }
        let amt = buf.len();
        let (a, b) = self.split_at(amt);

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

    #[inline]
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize> {
        buf.extend_from_slice(self);
        let len = self.len();
        *self = &self[len..];
        Ok(len)
    }
}

impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);

        // First check if the amount of bytes we want to write is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            a[0] = buf[0];
        } else {
            a.copy_from_slice(&buf[..amt]);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Buf for &[u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn consume(&mut self, mut f: impl FnMut(&[u8]) -> Result<usize>) -> Result<usize> {
        let consumed = f(self)?;
        *self = &self[consumed..];
        Ok(consumed)
    }
}

impl BufMut for &mut [u8] {
    fn remaining_mut(&self) -> usize {
        self.len()
    }

    fn fill(&mut self, mut f: impl FnMut(&mut [u8]) -> Result<usize>) -> Result<usize> {
        let filled = f(self)?;
        *self = mem::take(self).split_at_mut(filled).1;
        Ok(filled)
    }
}
