use std::{
    io::Result,
    sync::atomic::{AtomicU32, Ordering},
};

#[derive(Debug)]
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
        self.inner.store(1, Ordering::Release);
        Ok(())
    }

    pub fn wait(&mut self) -> Result<()> {
        if let Ok(_) = self
            .inner
            .compare_exchange(1, 0, Ordering::Acquire, Ordering::Acquire)
        {
            return Ok(());
        }
        while let Err(_) = self
            .inner
            .compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed)
        {
            std::hint::spin_loop();
        }
        Ok(())
    }
}
