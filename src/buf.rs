use core::mem::MaybeUninit;

use crate::{Read, Result, Write};

const IO_BUF_SIZE: usize = 4096;

pub trait Buf: Read {
    fn remaining(&self) -> usize;

    fn consume(&mut self, mut f: impl FnMut(&[u8]) -> Result<usize>) -> Result<usize> {
        let size = self.remaining();
        let mut buf = [const { MaybeUninit::uninit() }; IO_BUF_SIZE];
        let mut count = 0;
        loop {
            let len = buf.len().min(size - count);
            if len == 0 {
                break;
            }
            self.read(unsafe { buf[..len].assume_init_mut() })?;
            let read = f(unsafe { buf[..len].assume_init_ref() })?;
            count += read;
            if read < len {
                break;
            }
        }
        Ok(count)
    }
}

pub trait BufMut: Write {
    fn remaining_mut(&self) -> usize;

    fn fill(&mut self, mut f: impl FnMut(&mut [u8]) -> Result<usize>) -> Result<usize> {
        let size = self.remaining_mut();
        let mut buf = [const { MaybeUninit::uninit() }; IO_BUF_SIZE];
        let mut count = 0;
        loop {
            let len = buf.len().min(size - count);
            if len == 0 {
                break;
            }
            let written = f(unsafe { buf[..len].assume_init_mut() })?;
            count += written;
            self.write(unsafe { buf[..written].assume_init_ref() })?;
            if written < len {
                break;
            }
        }
        Ok(count)
    }
}
