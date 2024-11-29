use std::{io, ptr::NonNull};

use shmoo::{sync::Spinlock, FromShm, Shm, ShmInit};

pub struct MsgQueue<T: Sized + Copy> {
    shm: Shm,
    data: NonNull<T>,
}

impl<T: Sized + Copy> MsgQueue<T> {
    pub fn new(name: &str, cap: usize) -> Result<Self> {
        if size_of::<T>() == 0 {
            unimplemented!("ZSTs not yet supported");
        }
        if cap == 0 {
            return Err(Error::CapZero);
        }
        let size = size_of::<Header>() + cap * size_of::<T>();
        let mut shm = Shm::new(name, size)?;
        let hdr = Header::shm_init_mut(&mut shm)?;
        hdr.cap = cap;
        let data = NonNull::new(shm[size_of::<Header>()..size].as_mut_ptr() as *mut T).unwrap();
        Ok(MsgQueue { shm, data })
    }

    pub fn open(name: &str) -> Result<Self> {
        let mut shm = Shm::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .open(name)?;
        let hdr = Header::from_shm(&shm)?;
        let size = size_of::<Header>() + hdr.cap * size_of::<T>();
        let data = NonNull::new(shm[size_of::<Header>()..size].as_mut_ptr() as *mut T).unwrap();
        Ok(MsgQueue { shm, data })
    }

    pub fn capacity(&self) -> usize {
        let hdr = Header::from_shm(&self.shm).unwrap();
        hdr.cap
    }

    pub fn len(&self) -> usize {
        let hdr = Header::from_shm(&self.shm).unwrap();
        hdr.len
    }

    pub fn try_send(&mut self, val: T) -> Result<()> {
        let hdr = Header::from_shm_mut(&mut self.shm)?;
        // TODO: Use Rust's standard library mutex, if possible.
        hdr.wr_lock.lock()?;
        if hdr.len == hdr.cap {
            hdr.wr_lock.unlock()?;
            return Err(Error::QueueFull);
        }
        let wrp = hdr.wrp;
        unsafe {
            self.data.as_ptr().add(wrp).write(val);
        }
        hdr.wrp = (hdr.wrp + 1) % hdr.cap;
        hdr.len += 1;
        hdr.wr_lock.unlock()?;
        Ok(())
    }

    pub fn send(&mut self, val: T) -> Result<()> {
        loop {
            match self.try_send(val) {
                Err(Error::QueueFull) => (),
                Err(e) => return Err(e),
                _ => break,
            }
        }
        Ok(())
    }

    pub fn try_recv(&mut self) -> Result<T> {
        let hdr = Header::from_shm_mut(&mut self.shm)?;
        // TODO: Use Rust's standard library mutex, if possible.
        hdr.rd_lock.lock()?;
        if hdr.len == 0 {
            hdr.rd_lock.unlock()?;
            return Err(Error::QueueEmpty);
        }
        let rdp = hdr.rdp;
        let val = unsafe { self.data.as_ptr().add(rdp).read() };
        hdr.rdp = (hdr.rdp + 1) % hdr.cap;
        hdr.len -= 1;
        hdr.rd_lock.unlock()?;
        Ok(val)
    }

    pub fn recv(&mut self) -> Result<T> {
        let val = loop {
            match self.try_recv() {
                Err(Error::QueueEmpty) => (),
                Err(e) => return Err(e),
                Ok(val) => break val,
            }
        };
        Ok(val)
    }

    pub fn is_empty(&self) -> bool {
        let hdr = Header::from_shm(&self.shm).unwrap();
        hdr.len == 0
    }

    pub fn is_full(&self) -> bool {
        let hdr = Header::from_shm(&self.shm).unwrap();
        hdr.len == hdr.cap
    }
}

#[derive(ShmInit, FromShm)]
#[repr(C)]
struct Header {
    cap: usize,
    len: usize,
    rdp: usize,
    wrp: usize,
    rd_lock: Spinlock,
    wr_lock: Spinlock,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            cap: 0,
            len: 0,
            rdp: 0,
            wrp: 0,
            rd_lock: Spinlock::new(),
            wr_lock: Spinlock::new(),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    QueueEmpty,
    QueueFull,
    CapZero,
    ShmError(shmoo::error::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            Self::QueueEmpty => String::from("queue is empty"),
            Self::QueueFull => String::from("queue is full"),
            Self::CapZero => String::from("capacity must be greater than zero"),
            Self::ShmError(e) => e.to_string(),
        };
        write!(f, "{}", s)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::ShmError(value.into())
    }
}

impl From<shmoo::Error> for Error {
    fn from(value: shmoo::Error) -> Self {
        Self::ShmError(value)
    }
}
