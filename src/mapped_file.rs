use nix::{
    libc::off_t,
    sys::mman::{MapFlags, ProtFlags},
};
use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write},
    num::NonZero,
};

use crate::mmap_raw::MmapRaw;

pub struct MappedFile {
    inner: MmapRaw,
    pos: usize,
}

impl MappedFile {
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }
}

impl Seek for MappedFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(p) => p as usize,
            SeekFrom::End(p) => {
                if p < 0 && -p > self.pos as i64 {
                    self.pos
                } else {
                    (self.inner.len as i64 - p) as usize
                }
            }
            SeekFrom::Current(p) => (self.pos as i64 + p) as usize,
        };
        Ok(self.pos as u64)
    }
}

impl Read for MappedFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.pos >= self.inner.len {
            return Ok(0);
        }
        let to_read = std::cmp::min(self.inner.len - self.pos, buf.len());
        buf[..to_read].copy_from_slice(&self.inner[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl Write for MappedFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.pos >= self.inner.len {
            return Ok(0);
        }
        let to_write = std::cmp::min(self.inner.len - self.pos, buf.len());
        self.inner[self.pos..self.pos + to_write].copy_from_slice(&buf[..to_write]);
        Ok(to_write)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

pub struct OpenOptions {
    prot: ProtFlags,
    flgs: MapFlags,
    offset: off_t,
}

impl OpenOptions {
    pub fn open(self, f: &File) -> Result<MappedFile> {
        let len = f.metadata()?.len() as usize;
        let inner = MmapRaw::new(
            None,
            NonZero::new(len).ok_or(Error::new(
                ErrorKind::InvalidData,
                "file size cannot be zero",
            ))?,
            self.prot,
            self.flgs,
            f,
            self.offset,
        )?;
        Ok(MappedFile { inner, pos: 0 })
    }

    pub fn new() -> Self {
        OpenOptions {
            prot: ProtFlags::PROT_NONE,
            flgs: MapFlags::MAP_SHARED,
            offset: 0,
        }
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
