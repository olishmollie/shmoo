use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug)]
pub struct BinarySemaphore {
    inner: AtomicU8,
}

impl BinarySemaphore {
    pub fn new() -> Self {
        let inner = AtomicU8::new(0);
        BinarySemaphore { inner }
    }

    pub fn post(&mut self) {
        self.inner.store(1, Ordering::Release);
    }

    pub fn wait(&mut self) {
        while let Err(_) = self
            .inner
            .compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed)
        {
            std::hint::spin_loop();
        }
    }
}
