use std::io::Result;

use shmoo::{Mmap, Semaphore};

pub const BUF_SIZE: usize = 1024;

#[repr(C)]
#[derive(Debug)]
pub struct Shmbuf<const N: usize> {
    pub sem1: Semaphore,
    pub sem2: Semaphore,
    pub buf: [u8; N],
}

impl<const N: usize> Shmbuf<N> {
    pub fn new(mem: &mut Mmap) -> Result<&mut Self> {
        assert!(
            mem.len() >= size_of::<Self>(),
            "size of shared memory segment cannot be smaller than Shmbuf"
        );
        let shmbuf = mem.as_mut_ptr() as *mut Shmbuf<N>;
        unsafe {
            (&raw mut (*shmbuf).sem1).write(Semaphore::new(0)?);
            (&raw mut (*shmbuf).sem2).write(Semaphore::new(0)?);
            (&raw mut (*shmbuf).buf).write([0; N]);
            Ok(&mut *shmbuf)
        }
    }

    pub fn from_shm_mut(mem: &mut Mmap) -> Result<&mut Self> {
        let shmbuf = mem.as_mut_ptr() as *mut Shmbuf<N>;
        unsafe { Ok(&mut *shmbuf) }
    }

    pub fn read(&self, buf: &mut [u8]) {
        assert!(buf.len() <= N);
        buf.copy_from_slice(&self.buf[..buf.len()]);
    }

    pub fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= N);
        self.buf[..buf.len()].copy_from_slice(buf);
    }
}

impl<const N: usize> Drop for Shmbuf<N> {
    fn drop(&mut self) {
        println!("Dropping Shmbuf!");
    }
}
