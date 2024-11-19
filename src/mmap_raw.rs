use std::io::{Error, Result};
use std::num::NonZero;
use std::ops::{Deref, DerefMut};
use std::os::fd::AsFd;
use std::ptr::NonNull;

use nix::libc::{c_void, off_t};
use nix::sys::mman::{mmap, msync, munmap, MapFlags, MsFlags, ProtFlags};

#[derive(Debug)]
pub(crate) struct MmapRaw {
    pub ptr: NonNull<c_void>,
    pub len: usize,
}

impl<'l> MmapRaw {
    pub fn new<F: AsFd>(
        addr: Option<NonZero<usize>>,
        len: NonZero<usize>,
        prot: ProtFlags,
        flgs: MapFlags,
        file: &F,
        off: off_t,
    ) -> Result<Self> {
        let ptr = unsafe { mmap(addr, len, prot, flgs, file, off)? };
        Ok(MmapRaw {
            ptr,
            len: len.into(),
        })
    }

    pub fn flush(&mut self) -> Result<()> {
        unsafe { msync(self.ptr, self.len, MsFlags::MS_SYNC).map_err(|_| Error::last_os_error()) }
    }
}

impl Drop for MmapRaw {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr, self.len).unwrap();
        }
    }
}

impl Deref for MmapRaw {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) }
    }
}

impl DerefMut for MmapRaw {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, self.len) }
    }
}
