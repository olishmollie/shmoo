use shm_derive::{FromShm, ShmInit};
use shmoo::sync::BinarySemaphore;

pub const BUF_SIZE: usize = 1024;

#[derive(ShmInit, FromShm)]
#[repr(C)]
pub struct Shmbuf<const N: usize> {
    pub sem1: BinarySemaphore,
    pub sem2: BinarySemaphore,
    pub buf: [u8; N],
}

impl<const N: usize> Shmbuf<N> {
    pub fn read(&self, buf: &mut [u8]) {
        assert!(buf.len() <= N);
        buf.copy_from_slice(&self.buf[..buf.len()]);
    }

    pub fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= N);
        self.buf[..buf.len()].copy_from_slice(buf);
    }
}

impl<const N: usize> Default for Shmbuf<N> {
    fn default() -> Self {
        Self {
            sem1: BinarySemaphore::new(),
            sem2: BinarySemaphore::new(),
            buf: [0; N],
        }
    }
}
