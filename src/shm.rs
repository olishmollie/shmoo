use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::num::NonZero;
use std::ops::{Deref, DerefMut};
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::PathBuf;

use nix::sys::mman::shm_unlink;
use nix::unistd::ftruncate;
use nix::{
    fcntl::OFlag,
    libc::off_t,
    sys::mman::{shm_open, MapFlags, ProtFlags},
    sys::stat::{fstat, Mode},
};

use crate::mmap_raw::MmapRaw;

/// Options to create an [`Shm`].
pub struct OpenOptions {
    mode: Mode,
    oflg: OFlag,
    prot: ProtFlags,
    flgs: MapFlags,
    offset: off_t,
    file: Option<File>,
}

impl OpenOptions {
    pub fn open(self, handle: &str) -> Result<Shm> {
        let fd = shm_open(handle, self.oflg, self.mode)?;
        let statbuf = fstat(fd.as_raw_fd())?;
        let size = statbuf.st_size as usize;
        let inner = MmapRaw::new(
            None,
            NonZero::new(size).ok_or(Error::new(
                ErrorKind::InvalidData,
                "file size cannot be zero",
            ))?,
            self.prot,
            self.flgs,
            &fd,
            self.offset,
        )?;
        Ok(Shm {
            inner: Some(inner),
            handle: handle.into(),
        })
    }

    pub fn with_capacity(self, handle: &str, len: usize) -> Result<Shm> {
        if handle.chars().nth(0) != Some('/') {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Mmap handle must start with '/'",
            ));
        }
        let fd = if let Some(file) = self.file {
            OwnedFd::from(file)
        } else {
            let fd = shm_open(handle, self.oflg, self.mode)?;
            ftruncate(&fd, len as i64)?;
            fd
        };
        let inner = MmapRaw::new(
            None,
            NonZero::new(len).ok_or(Error::new(
                ErrorKind::InvalidData,
                "file size cannot be zero",
            ))?,
            self.prot,
            self.flgs,
            &fd,
            self.offset,
        )?;
        Ok(Shm {
            inner: Some(inner),
            handle: handle.into(),
        })
    }

    pub fn new() -> Self {
        OpenOptions {
            mode: Mode::from_bits(0o644).unwrap(),
            oflg: OFlag::O_RDWR,
            prot: ProtFlags::PROT_NONE,
            flgs: MapFlags::MAP_SHARED,
            offset: 0,
            file: None,
        }
    }

    pub fn from_file(mut self, file: File) -> Self {
        self.file = Some(file);
        self
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
}

#[derive(Debug)]
pub struct Shm {
    handle: PathBuf,
    inner: Option<MmapRaw>,
}

impl Shm {
    pub fn new(name: &str, size: usize) -> Result<Self> {
        Shm::options()
            .read(true)
            .write(true)
            .create(true)
            .exclusive(true)
            .with_capacity(name, size)
    }

    pub fn open(name: &str) -> Result<Self> {
        Shm::options().read(true).write(true).open(name)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }
}

impl Read for Shm {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let shm = self.inner.as_ref().unwrap();
        let n = std::cmp::min(shm.len(), buf.len());
        buf.copy_from_slice(&shm[..n]);
        Ok(n)
    }
}

impl Write for Shm {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let shm = self.inner.as_mut().unwrap();
        let n = std::cmp::min(shm.len(), buf.len());
        shm[..n].copy_from_slice(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.as_mut().unwrap().flush()
    }
}

impl Deref for Shm {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        if let Some(inner) = &self.inner {
            inner.deref()
        } else {
            unreachable!()
        }
    }
}

impl DerefMut for Shm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(inner) = &mut self.inner {
            inner.deref_mut()
        } else {
            unreachable!()
        }
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        drop(self.inner.take());
        let _ = shm_unlink(&self.handle);
    }
}
