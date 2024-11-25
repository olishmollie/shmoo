mod msg_queue;
mod shm;

pub mod sync;
pub use msg_queue::MsgQueue;
pub use shm::Shm;

use std::io::Result;

/// # Safety
/// [`Shm`] objects use the Posix function `shm_open` to initialize a shared memory
/// segment, which guarantees the memory (and thus all bytes, including padding,
/// constructed from it) is zero initialized.
pub unsafe trait FromShm: Sized {
    fn from_shm(shm: &mut Shm) -> Result<&Self>;
    fn from_shm_mut(shm: &mut Shm) -> Result<&mut Self>;
}

pub unsafe trait ToShm: Sized {
    fn to_shm(shm: &mut Shm) -> Result<&Self>;
    fn to_shm_mut(shm: &mut Shm) -> Result<&mut Self>;
}
