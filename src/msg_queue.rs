use std::{io::Result, marker::PhantomData};

use crate::{sync::Spinlock, FromShm, Shm, ToShm};

#[repr(C)]
pub struct MsgQueue<T: Sized + Copy> {
    shm: Shm,
    _marker: PhantomData<T>,
}

impl<T: Sized + Copy> MsgQueue<T> {
    pub fn new(name: &str, cap: usize) -> Result<Self> {
        if size_of::<T>() == 0 {
            unimplemented!("ZSTs not yet supported");
        }
        let size = size_of::<Header>() + cap * size_of::<T>();
        let mut shm = Shm::new(name, size)?;
        let hdr = Header::to_shm_mut(&mut shm);
        hdr.cap = cap;
        Ok(MsgQueue {
            shm,
            _marker: PhantomData,
        })
    }

    pub fn open(name: &str) -> Result<Self> {
        let shm = Shm::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .open(name)?;
        Ok(MsgQueue {
            shm,
            _marker: PhantomData,
        })
    }

    pub fn capacity(&self) -> usize {
        let hdr = Header::from_shm(&self.shm);
        hdr.cap
    }

    pub fn len(&self) -> usize {
        let hdr = Header::from_shm(&self.shm);
        hdr.len
    }

    pub fn send(&mut self, val: T) -> Result<()> {
        let hdr = self.header_mut();
        unsafe {
            (*hdr).wr_lock.lock()?;
            while (*hdr).len == (*hdr).cap {
                std::hint::spin_loop();
            }
            let wrp = (*hdr).wrp;
            let ptr = self.shm.as_mut_ptr().add(size_of::<Header>()) as *mut T;
            ptr.add((*hdr).len).write(val);
            (*hdr).len += 1;
            (*hdr).wrp = (wrp + 1) % self.capacity();
            (*hdr).wr_lock.unlock()?;
            Ok(())
        }
    }

    pub fn recv(&mut self) -> Result<T> {
        let hdr = self.header_mut();
        unsafe {
            (*hdr).rd_lock.lock()?;
            while (*hdr).len == 0 {
                std::hint::spin_loop();
            }
            let rdp = (*hdr).rdp;
            let ptr = self.shm.as_ptr().add(size_of::<Header>()) as *const T;
            let val = ptr.add((*hdr).len - 1).read();
            (*hdr).rdp = (rdp + 1) % self.capacity();
            (*hdr).len -= 1;
            (*hdr).rd_lock.unlock()?;
            Ok(val)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    fn header_mut(&mut self) -> *mut Header {
        self.shm[..size_of::<Header>()].as_mut_ptr() as *mut Header
    }
}

#[repr(C)]
struct Header {
    cap: usize,
    len: usize,
    rdp: usize,
    wrp: usize,
    rd_lock: Spinlock,
    wr_lock: Spinlock,
}

unsafe impl ToShm for Header {
    fn to_shm(shm: &mut Shm) -> &Self {
        unsafe {
            let hdr = &mut *(shm[..size_of::<Header>()].as_mut_ptr() as *mut Header);
            hdr.cap = 0;
            hdr.len = 0;
            hdr.rdp = 0;
            hdr.wrp = 0;
            hdr.rd_lock = Spinlock::new();
            hdr.wr_lock = Spinlock::new();
            hdr
        }
    }

    fn to_shm_mut(shm: &mut Shm) -> &mut Self {
        unsafe {
            let hdr = &mut *(shm[..size_of::<Header>()].as_mut_ptr() as *mut Header);
            hdr.cap = 0;
            hdr.len = 0;
            hdr.rdp = 0;
            hdr.wrp = 0;
            hdr.rd_lock = Spinlock::new();
            hdr.wr_lock = Spinlock::new();
            hdr
        }
    }
}
