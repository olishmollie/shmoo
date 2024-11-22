use std::{io::Result, slice, sync::atomic::AtomicU32};

use crate::{BinarySemaphore, Mmap};

#[repr(C)]
pub struct MsgQueue<'l, T: Sized> {
    mem: Mmap,
    lock: &'l mut BinarySemaphore,
    data: &'l [T],
}

impl<'l, T: Sized> MsgQueue<'l, T> {
    /// Max len is number of items.
    pub fn new(name: &str, max_len: usize) -> Result<Self> {
        let size = max_len * size_of::<T>() + size_of::<BinarySemaphore>();
        let mut mmap = Mmap::options()
            .mode(0o644)
            .read(true)
            .write(true)
            .create(true)
            .with_capacity(&name, size)?;
        unsafe {
            let ptr = mmap.as_mut_ptr() as *mut usize;
            ptr.write(max_len);
            ptr.add(size_of::<usize>()).write(0);
            let data_ptr = ptr.add(size_of::<usize>() * 2) as *mut T;
            debug_assert!(ptr.is_aligned(), "*const T is unaligned: {:?}", ptr);
            let lock_ptr = data_ptr.add(size_of::<T>() * size) as *mut BinarySemaphore;
            debug_assert!(
                lock_ptr.is_aligned(),
                "*mut BinarySemaphore is unaligned: {:?}",
                ptr
            );
            (&raw mut (*lock_ptr).inner).write(AtomicU32::new(0));
            Ok(MsgQueue {
                mem: mmap,
                lock: &mut *lock_ptr,
                data: slice::from_raw_parts(data_ptr, max_len),
            })
        }
    }

    pub fn open(name: &str) -> Result<Self> {
        let mut mmap = Mmap::options().mode(0o644).read(true).write(true)
    }

    pub fn capacity(&self) -> usize {
        unsafe { *(self.mem.as_ptr() as *const usize) }
    }

    pub fn len(&self) -> usize {
        unsafe { *(self.mem.as_ptr().add(size_of::<usize>()) as *const usize) }
    }

    pub fn send(&mut self, val: T) -> Result<()> {
        unimplemented!()
    }

    pub fn recv(&mut self) -> Result<T> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_writes_header_to_shm() {
        let cap = 4;
        let mq = MsgQueue::<[u8; 4]>::new("/shmoo", 4).unwrap();
        assert_eq!(mq.capacity(), cap);
        assert_eq!(mq.len(), 0);
    }
}
