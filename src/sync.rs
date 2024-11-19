use std::io::{Error, Result};
use std::mem::MaybeUninit;

use nix::libc::sem_init;
use nix::libc::{sem_post, sem_t, sem_wait};

#[derive(Debug)]
pub struct Semaphore {
    inner: sem_t,
}

impl Semaphore {
    pub fn new(value: u32) -> Result<Self> {
        let mut sem = MaybeUninit::uninit();
        let fd = unsafe { sem_init(sem.as_mut_ptr(), 1, value) };
        if fd < 0 {
            Err(Error::last_os_error())
        } else {
            let inner = unsafe { sem.assume_init() };
            Ok(Semaphore { inner })
        }
    }

    pub fn post(&mut self) -> Result<()> {
        let err = unsafe { sem_post(&mut self.inner as *mut sem_t) };
        if err != 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn wait(&mut self) -> Result<()> {
        let err = unsafe { sem_wait(&mut self.inner as *mut sem_t) };
        if err != 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

// impl Drop for Semaphore {
//     fn drop(&mut self) {
//         println!("Dropping {:?}...", self);
//         unsafe {
//             sem_destroy(&mut self.inner as *mut sem_t);
//         }
//     }
// }
