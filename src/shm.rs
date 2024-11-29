use std::io::{self, Read, Write};
use std::num::NonZero;
use std::ops::{Deref, DerefMut};
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::slice;

use nix::errno::Errno;
use nix::sys::mman::shm_unlink;
use nix::unistd::ftruncate;
use nix::{
    fcntl::OFlag,
    libc::c_void,
    libc::off_t,
    sys::mman::{mmap, msync, munmap, shm_open, MapFlags, MsFlags, ProtFlags},
    sys::stat::{fstat, Mode},
};

use crate::{error::Result, ShmInit};

pub struct OpenOptions {
    mode: Mode,
    oflg: OFlag,
    prot: ProtFlags,
    flgs: MapFlags,
    offset: off_t,
}

impl OpenOptions {
    pub fn open(self, name: &str) -> Result<Shm> {
        let name = OpenOptions::prepend_slash(name);
        let fd = shm_open(name.as_str(), self.oflg, self.mode)?;
        let statbuf = fstat(fd.as_raw_fd())?;
        let len = statbuf.st_size as usize;
        Self::map_raw(fd, name, len, self.prot, self.flgs, self.offset)
    }

    pub fn map(self, name: &str, len: usize) -> Result<Shm> {
        let name = Self::prepend_slash(name);
        let fd = shm_open(name.as_str(), self.oflg, self.mode)?;
        ftruncate(&fd, len as i64)?;
        Self::map_raw(fd, name, len, self.prot, self.flgs, self.offset)
    }

    pub fn mode(mut self, mode: u32) -> Self {
        self.mode = Mode::from_bits(mode).expect("invalid mode");
        self
    }

    pub fn create(mut self, create: bool) -> Self {
        if create {
            self.oflg |= OFlag::O_CREAT;
        } else {
            self.oflg &= !OFlag::O_CREAT;
        }
        self
    }

    pub fn exclusive(mut self, exclusive: bool) -> Self {
        if exclusive {
            self.oflg |= OFlag::O_EXCL;
        } else {
            self.oflg &= !OFlag::O_EXCL;
        }
        self
    }

    pub fn read(mut self, readable: bool) -> Self {
        if readable {
            self.prot |= ProtFlags::PROT_READ;
        } else {
            self.prot &= !ProtFlags::PROT_READ;
        }
        self
    }

    pub fn write(mut self, writable: bool) -> Self {
        if writable {
            self.prot |= ProtFlags::PROT_WRITE;
        } else {
            self.prot &= !ProtFlags::PROT_WRITE;
        }
        self
    }

    pub fn execute(mut self, executable: bool) -> Self {
        if executable {
            self.prot |= ProtFlags::PROT_EXEC;
        } else {
            self.prot &= !ProtFlags::PROT_EXEC;
        }
        self
    }

    /// Must be aligned to page boundary.
    pub fn offset(mut self, offset: off_t) -> Self {
        self.offset = offset;
        self
    }

    fn map_raw(
        fd: OwnedFd,
        name: String,
        len: usize,
        prot: ProtFlags,
        flgs: MapFlags,
        offset: off_t,
    ) -> Result<Shm> {
        // Since we embed a header, the length will never be zero.
        let actual_len = len + size_of::<Header>();
        let ptr = unsafe {
            mmap(
                None,
                NonZero::new(actual_len).unwrap(),
                prot,
                flgs,
                &fd,
                offset,
            )?
        };
        let mut shm = Shm {
            ptr,
            len,
            name: name.into(),
        };
        Header::init(&mut shm, len)?;
        Ok(shm)
    }

    fn prepend_slash(name: &str) -> String {
        if name.chars().nth(0) != Some('/') {
            String::from("/") + name
        } else {
            String::from(name)
        }
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions {
            mode: Mode::from_bits(0o644).unwrap(),
            oflg: OFlag::O_RDWR,
            prot: ProtFlags::PROT_NONE,
            flgs: MapFlags::MAP_SHARED,
            offset: 0,
        }
    }
}

pub struct Shm {
    name: PathBuf,
    ptr: NonNull<c_void>,
    len: usize,
}

impl Shm {
    pub fn new(name: &str, size: usize) -> Result<Self> {
        Shm::options()
            .read(true)
            .write(true)
            .create(true)
            .exclusive(true)
            .map(name, size)
    }

    pub fn open(name: &str) -> Result<Self> {
        Shm::options().read(true).write(true).open(name)
    }

    pub fn construct<T: ShmInit>(&mut self) -> Result<&T> {
        let hdr_bytes =
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, size_of::<Header>()) };
        let hdr = Header::from_bytes_mut(hdr_bytes);
        let result = T::shm_init_mut(self)?;
        hdr.nxt += size_of::<T>();
        Ok(result)
    }

    pub fn construct_mut<T: ShmInit>(&mut self) -> Result<&mut T> {
        let hdr_bytes =
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, size_of::<Header>()) };
        let hdr = Header::from_bytes_mut(hdr_bytes);
        let result = T::shm_init_mut(self)?;
        hdr.nxt += size_of::<T>();
        Ok(result)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::default()
    }
}

impl Read for Shm {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = std::cmp::min(self.len, buf.len());
        buf.copy_from_slice(&self[..n]);
        Ok(n)
    }
}

impl Write for Shm {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = std::cmp::min(self.len, buf.len());
        self[..n].copy_from_slice(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            msync(self.ptr, self.len, MsFlags::MS_SYNC).map_err(|_| io::Error::last_os_error())
        }
    }
}

impl Deref for Shm {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe {
            let hdr = Header::from_shm(self);
            let ptr = (self.ptr.as_ptr() as *const u8).add(hdr.nxt);
            slice::from_raw_parts(ptr, self.len)
        }
    }
}

impl DerefMut for Shm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let hdr = Header::from_shm(self);
            let ptr = (self.ptr.as_ptr() as *mut u8).add(hdr.nxt);
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr, self.len).unwrap();
        }
        // Ignore ENOENT in case another process already closed the file.
        match shm_unlink(&self.name) {
            Err(Errno::ENOENT) => (),
            r => r.unwrap(),
        }
    }
}

#[repr(C)]
struct Header {
    len: usize,
    nxt: usize,
}

impl Header {
    fn init(shm: &mut Shm, len: usize) -> Result<()> {
        let hdr = Header::from_shm_mut(shm);
        hdr.len = len;
        hdr.nxt = size_of::<Self>();
        Ok(())
    }

    fn from_shm(shm: &Shm) -> &Self {
        unsafe {
            let hdr_bytes = slice::from_raw_parts(shm.ptr.as_ptr() as *const u8, size_of::<Self>());
            &*(hdr_bytes.as_ptr() as *const Header)
        }
    }

    fn from_shm_mut(shm: &mut Shm) -> &mut Self {
        unsafe {
            let hdr_bytes =
                slice::from_raw_parts_mut(shm.ptr.as_ptr() as *mut u8, size_of::<Self>());
            &mut *(hdr_bytes.as_mut_ptr() as *mut Header)
        }
    }

    fn from_bytes_mut(data: &mut [u8]) -> &mut Self {
        unsafe { &mut *(data[..size_of::<Self>()].as_ptr() as *mut Header) }
    }
}
