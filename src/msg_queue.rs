use std::{io::Result, marker::PhantomData, slice};

use crate::{sync::Spinlock, Shm};

#[repr(C)]
pub struct MsgQueue<T: Sized + Copy> {
    mem: MemWrapper<T>,
}

impl<T: Sized + Copy> MsgQueue<T> {
    pub fn new(name: &str, cap: usize) -> Result<Self> {
        if size_of::<T>() == 0 {
            unimplemented!("ZSTs not yet supported");
        }
        let size = size_of::<Header>() + cap * size_of::<T>();
        let mmap = Shm::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .create(true)
            .exclusive(true)
            .new(&name, size)?;
        let mem = MemWrapper::new(mmap, cap)?;
        Ok(MsgQueue { mem })
    }

    pub fn open(name: &str) -> Result<Self> {
        let mmap = Shm::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .open(name)?;
        let mem = MemWrapper::from_shm(mmap);
        Ok(MsgQueue { mem })
    }

    pub fn capacity(&self) -> usize {
        self.mem.cap()
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }

    pub fn send(&mut self, val: T) -> Result<()> {
        self.mem.wr_lock().lock()?;
        while self.is_full() {
            std::hint::spin_loop();
        }
        let wrp = self.mem.wrp();
        self.mem.data_mut()[wrp] = val;
        self.mem.inc_wrp();
        self.mem.inc_len();
        self.mem.wr_lock().unlock()?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<T> {
        self.mem.rd_lock().lock()?;
        while self.is_empty() {
            std::hint::spin_loop();
        }
        let rdp = self.mem.rdp();
        let data = self.mem.data();
        let val = data[rdp];
        self.mem.inc_rdp();
        self.mem.dec_len();
        self.mem.rd_lock().unlock()?;
        Ok(val)
    }

    pub fn is_empty(&self) -> bool {
        self.mem.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.mem.len() == self.mem.cap()
    }
}

struct MemWrapper<T: Sized> {
    mmap: Shm,
    hdr: *mut Header,
    _marker: PhantomData<T>,
}

impl<T: Sized> MemWrapper<T> {
    fn new(mut mmap: Shm, cap: usize) -> Result<Self> {
        let hdr = unsafe { Header::new(mmap.as_mut_ptr() as *mut u8, cap)? };
        Ok(MemWrapper {
            mmap,
            hdr,
            _marker: PhantomData,
        })
    }

    fn from_shm(mut mmap: Shm) -> Self {
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

    fn len(&self) -> usize {
        unsafe { (*self.hdr).len }
    }

    fn inc_len(&mut self) {
        unsafe {
            (*self.hdr).len += 1;
        }
    }

    fn dec_len(&mut self) {
        unsafe {
            (*self.hdr).len -= 1;
        }
    }

    fn rdp(&self) -> usize {
        unsafe { (*self.hdr).rdp }
    }

    fn inc_rdp(&mut self) {
        unsafe {
            (*self.hdr).rdp = ((*self.hdr).rdp + 1) % self.cap();
        }
    }

    fn wrp(&self) -> usize {
        unsafe { (*self.hdr).wrp }
    }

    fn inc_wrp(&mut self) {
        unsafe {
            (*self.hdr).wrp = ((*self.hdr).wrp + 1) % self.cap();
        }
    }

    fn rd_lock(&self) -> &mut Spinlock {
        unsafe { &mut *(&raw mut (*self.hdr).rd_lock) }
    }

    fn wr_lock(&self) -> &mut Spinlock {
        unsafe { &mut *(&raw mut (*self.hdr).wr_lock) }
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
    len: usize,
    rdp: usize,
    wrp: usize,
    rd_lock: Spinlock,
    wr_lock: Spinlock,
}

impl Header {
    unsafe fn new(mem: *mut u8, cap: usize) -> Result<*mut Self> {
        let ptr = mem as *mut usize;
        ptr.write(cap);
        ptr.add(1).write(0); //len
        ptr.add(2).write(0); //rdp
        ptr.add(3).write(0); //wrp
        let mtx_ptr = ptr.add(3) as *mut Spinlock;
        debug_assert!(
            mtx_ptr.is_aligned(),
            "*mut BinarySemaphore is unaligned: {:?}",
            mtx_ptr
        );
        mtx_ptr.write(Spinlock::new());
        mtx_ptr.add(1).write(Spinlock::new());
        Ok(mem as *mut Header)
    }

    fn from_shm(mem: *const u8) -> *mut Self {
        mem as *mut Header
    }
}
