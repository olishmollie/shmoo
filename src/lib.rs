mod msg_queue;
mod shm;

pub mod sync;
pub use msg_queue::MsgQueue;
pub use shm::Shm;
pub use shm_derive::{FromShm, ShmInit};

// Allows derive macros to use fully qualified trait names (e.g. 'shmoo::ShmInit')
// and still compile within this crate.
extern crate self as shmoo;

/// # Safety
///
/// TODO: Implementers must guarantee provenance, size, and alignment are correct. Can
/// Shm help with that?
pub unsafe trait ShmInit: Sized + Default {
    fn shm_init(shm: &mut Shm) -> &Self;
    fn shm_init_mut(shm: &mut Shm) -> &mut Self;
}

/// # Safety
///
/// The methods of this trait should only be called if `Self` was created using
/// [`construct`](Shm::construct), which uses the [`ShmInit`] methods internally.
/// Implementors must also guarantee that the struct is `repr(C)`, the size of
/// the shared memory segment is greater than or equal to the size of Self, and
/// any pointers created to Self have the proper alignment and provenance.
///
/// Use the [`FromShm`](shm_derive::FromShm) derive macro to assert these invariants
/// at compile time.
///
pub unsafe trait FromShm: Sized {
    fn from_shm(shm: &Shm) -> &Self;
    fn from_shm_mut(shm: &mut Shm) -> &mut Self;
}
