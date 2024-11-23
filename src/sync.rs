use std::{
    io::{Error, Result},
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering},
};

use nix::libc::{
    pthread_cond_init, pthread_cond_signal, pthread_cond_t, pthread_cond_wait,
    pthread_condattr_init, pthread_condattr_setpshared, pthread_condattr_t, pthread_mutex_init,
    pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, pthread_mutexattr_init,
    pthread_mutexattr_setpshared, pthread_mutexattr_t,
};

macro_rules! check_err {
    ($call:expr) => {
        let err = $call;
        if (err < 0) {
            return Err(Error::from_raw_os_error(err));
        }
    };
}

#[repr(C)]
pub struct PosixMutex {
    attr: pthread_mutexattr_t,
    mtx: pthread_mutex_t,
}

impl PosixMutex {
    pub fn new() -> Result<Self> {
        let mut attr = MaybeUninit::uninit();
        let mut mtx = MaybeUninit::uninit();
        unsafe {
            check_err!(pthread_mutexattr_init(attr.as_mut_ptr()));
            check_err!(pthread_mutexattr_setpshared(attr.as_mut_ptr(), 1));
            check_err!(pthread_mutex_init(mtx.as_mut_ptr(), attr.as_mut_ptr()));
            Ok(PosixMutex {
                attr: attr.assume_init(),
                mtx: mtx.assume_init(),
            })
        }
    }

    pub fn lock(&mut self) -> Result<()> {
        unsafe {
            check_err!(pthread_mutex_lock(&raw mut self.mtx));
        }
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        unsafe {
            check_err!(pthread_mutex_unlock(&raw mut self.mtx));
        }
        Ok(())
    }
}

#[repr(C)]
pub struct PosixCondition {
    attr: pthread_condattr_t,
    cond: pthread_cond_t,
}

impl PosixCondition {
    pub fn new() -> Result<Self> {
        let mut attr = MaybeUninit::uninit();
        let mut cond = MaybeUninit::uninit();
        unsafe {
            check_err!(pthread_condattr_init(attr.as_mut_ptr()));
            check_err!(pthread_condattr_setpshared(attr.as_mut_ptr(), 1));
            check_err!(pthread_cond_init(cond.as_mut_ptr(), attr.as_mut_ptr()));
            Ok(PosixCondition {
                attr: attr.assume_init(),
                cond: cond.assume_init(),
            })
        }
    }

    pub fn wait(&mut self, mtx: &mut PosixMutex) -> Result<()> {
        unsafe {
            check_err!(pthread_cond_wait(&raw mut self.cond, &raw mut mtx.mtx));
        }
        Ok(())
    }

    pub fn signal(&mut self) -> Result<()> {
        unsafe {
            check_err!(pthread_cond_signal(&raw mut self.cond));
        }
        Ok(())
    }
}

#[repr(transparent)]
pub struct BinarySemaphore {
    pub(crate) inner: AtomicU32,
}

impl BinarySemaphore {
    pub fn new() -> Self {
        let inner = AtomicU32::new(0);
        BinarySemaphore { inner }
    }

    pub fn post(&mut self) -> Result<()> {
        self.inner.store(0, Ordering::Release);
        Ok(())
    }

    pub fn wait(&mut self) -> Result<()> {
        if let Ok(_) = self
            .inner
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
        {
            return Ok(());
        }
        while let Err(_) = self
            .inner
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            std::hint::spin_loop();
        }
        Ok(())
    }
}
