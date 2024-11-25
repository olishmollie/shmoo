use std::io::Result;

use shmoo::{sync::BinarySemaphore, FromShm, Shm, ToShm};

pub const BUF_SIZE: usize = 1024;

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

unsafe impl<const N: usize> ToShm for Shmbuf<N> {
    fn to_shm(shm: &mut Shm) -> Result<&Self> {
        if shm.len() < size_of::<Self>() {
            todo!()
        }
        let shmbuf = shm.as_mut_ptr() as *mut Self;
        unsafe {
            (&raw mut (*shmbuf).sem1).write(BinarySemaphore::new());
            (&raw mut (*shmbuf).sem2).write(BinarySemaphore::new());
            (&raw mut (*shmbuf).buf).write([0; N]);
            Ok(&*shmbuf)
        }
    }

    fn to_shm_mut(shm: &mut Shm) -> Result<&mut Self> {
        if shm.len() < size_of::<Self>() {
            todo!()
        }
        let shmbuf = shm.as_mut_ptr() as *mut Self;
        unsafe {
            (&raw mut (*shmbuf).sem1).write(BinarySemaphore::new());
            (&raw mut (*shmbuf).sem2).write(BinarySemaphore::new());
            (&raw mut (*shmbuf).buf).write([0; N]);
            Ok(&mut *shmbuf)
        }
    }
}

unsafe impl<const N: usize> FromShm for Shmbuf<N> {
    fn from_shm(shm: &mut Shm) -> Result<&Self> {
        let ptr = shm.as_mut_ptr() as *const Shmbuf<N>;
        unsafe { Ok(&*ptr) }
    }

    fn from_shm_mut(shm: &mut Shm) -> Result<&mut Self> {
        let ptr = shm.as_mut_ptr() as *mut Shmbuf<N>;
        unsafe { Ok(&mut *ptr) }
    }
}
