mod msg_queue;
mod shm;

pub mod sync;
pub use msg_queue::MsgQueue;
pub use shm::Shm;

/// # Safety
/// [`Shm`] objects use the Posix function `shm_open` to initialize a shared memory
/// segment, which guarantees the memory (and thus all bytes, including padding,
/// constructed from it) is zero initialized. In addition, the Shm object's [`Deref`]
/// implementation ensures that the pointer is properly aligned
/// (TODO: I think this is the case, need to make sure).
pub unsafe trait FromShm {
    fn from_shm(shm: &Shm) -> &Self;
    fn from_shm_mut(shm: &mut Shm) -> &mut Self;
}

unsafe impl<T: Sized> FromShm for T {
    fn from_shm(shm: &Shm) -> &Self {
        let size = size_of::<Self>();
        assert!(shm.len() >= size);
        let ptr = shm[..size].as_ptr() as *const Self;
        debug_assert!(ptr.is_aligned());
        unsafe { &*ptr }
    }

    fn from_shm_mut(shm: &mut Shm) -> &mut Self {
        let size = size_of::<Self>();
        assert!(shm.len() >= size);
        let ptr = shm[..size].as_mut_ptr() as *mut Self;
        debug_assert!(ptr.is_aligned());
        unsafe { &mut *ptr }
    }
}

pub unsafe trait ToShm: Sized {
    fn to_shm(shm: &mut Shm) -> &Self;
    fn to_shm_mut(shm: &mut Shm) -> &mut Self;
}
