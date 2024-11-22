use core::slice;
use std::{fmt::Debug, io::Result, marker::PhantomData};

use crate::{
    sync::{PosixCondition, PosixMutex},
    Mmap,
};

#[repr(C)]
pub struct MsgQueue<T: Sized + Copy> {
    mem: MemWrapper<T>,
    cap: usize,
}

impl<T: Sized + Copy + Debug> MsgQueue<T> {
    pub fn new(name: &str, cap: usize) -> Result<Self> {
        if size_of::<T>() == 0 {
            unimplemented!("ZSTs not yet supported");
        }
        let size = size_of::<Header>() + cap * size_of::<T>();
        let mmap = Mmap::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .create(true)
            .with_capacity(&name, size)?;
        let mem = MemWrapper::new(mmap, cap)?;
        Ok(MsgQueue { mem, cap })
    }

    pub fn open(name: &str) -> Result<Self> {
        let mmap = Mmap::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .open(name)?;
        let mem = MemWrapper::from_shm(mmap);
        let cap = mem.cap();
        Ok(MsgQueue { mem, cap })
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn send(&mut self, val: T) -> Result<()> {
        self.mem.mtx().lock()?;
        while self.is_full() {
            self.mem.wr_cond().wait(self.mem.mtx())?;
        }
        let wrp = self.mem.wrp() % self.mem.cap();
        self.mem.data_mut()[wrp] = val;
        self.mem.inc_wrp();
        self.mem.rd_cond().signal()?;
        self.mem.mtx().unlock()?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<T> {
        self.mem.mtx().lock()?;
        while self.is_empty() {
            self.mem.rd_cond().wait(self.mem.mtx())?;
        }
        let rdp = self.mem.rdp() % self.mem.cap();
        let data = self.mem.data();
        let val = data[rdp];
        self.mem.inc_rdp();
        self.mem.wr_cond().signal()?;
        self.mem.mtx().unlock()?;
        Ok(val)
    }

    pub fn is_empty(&self) -> bool {
        self.mem.rdp() == self.mem.wrp()
    }

    pub fn is_full(&self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.mem.rdp() % self.mem.cap() == self.mem.wrp() - 1
        }
    }
}

struct MemWrapper<T: Sized> {
    mmap: Mmap,
    hdr: *mut Header,
    _marker: PhantomData<T>,
}

impl<T: Sized> MemWrapper<T> {
    fn new(mut mmap: Mmap, cap: usize) -> Result<Self> {
        let hdr = unsafe { Header::new(mmap.as_mut_ptr() as *mut u8, cap)? };
        Ok(MemWrapper {
            mmap,
            hdr,
            _marker: PhantomData,
        })
    }

    fn from_shm(mut mmap: Mmap) -> Self {
        let hdr = Header::from_shm(mmap.as_mut_ptr() as *mut u8);
        MemWrapper {
            mmap,
            hdr,
            _marker: PhantomData,
        }
    }

    fn cap(&self) -> usize {
        unsafe { (*self.hdr).cap }
    }

    fn rdp(&self) -> usize {
        unsafe { (*self.hdr).rdp }
    }

    fn inc_rdp(&mut self) {
        unsafe {
            *(&raw mut (*self.hdr).rdp) += 1;
        }
    }

    fn wrp(&self) -> usize {
        unsafe { (*self.hdr).wrp }
    }

    fn inc_wrp(&mut self) {
        unsafe {
            *(&raw mut (*self.hdr).wrp) += 1;
        }
    }

    fn mtx(&self) -> &mut PosixMutex {
        unsafe { &mut *(&raw mut (*self.hdr).mtx) }
    }

    fn rd_cond(&self) -> &mut PosixCondition {
        unsafe { &mut *(&raw mut (*self.hdr).rd_cond) }
    }

    fn wr_cond(&self) -> &mut PosixCondition {
        unsafe { &mut *(&raw mut (*self.hdr).wr_cond) }
    }

    fn data(&self) -> &[T] {
        unsafe {
            let data = self.mmap.as_ptr().add(size_of::<Header>()) as *mut T;
            slice::from_raw_parts(data, self.cap())
        }
    }

    fn data_mut(&mut self) -> &mut [T] {
        unsafe {
            let data = self.mmap.as_mut_ptr().add(size_of::<Header>()) as *mut T;
            slice::from_raw_parts_mut(data, self.cap())
        }
    }
}

#[repr(C)]
struct Header {
    cap: usize,
    rdp: usize,
    wrp: usize,
    mtx: PosixMutex,
    rd_cond: PosixCondition,
    wr_cond: PosixCondition,
}

impl Header {
    unsafe fn new(mem: *mut u8, cap: usize) -> Result<*mut Self> {
        let ptr = mem as *mut usize;
        ptr.write(cap);
        ptr.add(1).write(0); // rdp
        ptr.add(2).write(0); // wrp
        let mtx_ptr = ptr.add(3) as *mut PosixMutex;
        debug_assert!(
            mtx_ptr.is_aligned(),
            "*mut PosixMutex is unaligned: {:?}",
            mtx_ptr
        );
        mtx_ptr.write(PosixMutex::new()?);
        let cond_ptr = mtx_ptr.add(1) as *mut PosixCondition;
        debug_assert!(
            cond_ptr.is_aligned(),
            "*mut PosixCondition is unaligned: {:?}",
            cond_ptr
        );
        cond_ptr.write(PosixCondition::new()?);
        cond_ptr.add(1).write(PosixCondition::new()?);
        Ok(mem as *mut Header)
    }

    fn from_shm(mem: *const u8) -> *mut Self {
        mem as *mut Header
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_metadata() {
        let mq = MsgQueue::<u8>::new("/shmoo", 1).unwrap();
        assert_eq!(mq.mem.cap(), 1);
        assert_eq!(mq.mem.rdp(), 0);
        assert_eq!(mq.mem.wrp(), 0);
    }

    #[test]
    fn test_pointer_writes() {
        let mut mq = MsgQueue::<u8>::new("/shmoo", 2).unwrap();
        mq.mem.inc_rdp();
        mq.mem.inc_wrp();
        assert_eq!(mq.mem.rdp(), 1);
        assert_eq!(mq.mem.wrp(), 1);
    }
}
