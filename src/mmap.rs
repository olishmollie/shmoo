use std::io::{Error, ErrorKind, Result};
use std::num::NonZero;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use nix::sys::mman::shm_unlink;
use nix::unistd::ftruncate;
use nix::{
    fcntl::OFlag,
    libc::off_t,
    sys::mman::{shm_open, MapFlags, ProtFlags},
    sys::stat::Mode,
};

use crate::mmap_raw::MmapRaw;

#[derive(Debug)]
pub struct Mmap {
    handle: PathBuf,
    inner: Option<MmapRaw>,
}

impl Mmap {
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }
}

impl Deref for Mmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        if let Some(inner) = &self.inner {
            inner.deref()
        } else {
            unreachable!()
        }
    }
}

impl DerefMut for Mmap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(inner) = &mut self.inner {
            inner.deref_mut()
        } else {
            unreachable!()
        }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        drop(self.inner.take());
        let _ = shm_unlink(&self.handle);
    }
}

pub struct OpenOptions {
    mode: Mode,
    oflg: OFlag,
    prot: ProtFlags,
    flgs: MapFlags,
    offset: off_t,
}

impl OpenOptions {
    pub fn with_capacity(self, handle: &str, len: usize) -> Result<Mmap> {
        let fd = shm_open(handle, self.oflg, self.mode)?;
        ftruncate(fd.try_clone()?, len as i64)?;
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
        Ok(Mmap {
            inner: Some(inner),
            handle: handle.into(),
        })
    }

    pub fn new() -> Self {
        OpenOptions {
            mode: Mode::from_bits(0o600).unwrap(),
            oflg: OFlag::O_RDWR,
            prot: ProtFlags::PROT_NONE,
            flgs: MapFlags::MAP_SHARED,
            offset: 0,
        }
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
