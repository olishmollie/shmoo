pub mod shmbuf;

use std::error::Error;

use shmbuf::{Shmbuf, BUF_SIZE};
use shmoo::Shm;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    let shmpath = &args[1];

    let mut shm = Shm::options()
        .read(true)
        .write(true)
        .create(true)
        .map(shmpath, 4096)?;

    let shmbuf = shm.construct_mut::<Shmbuf<BUF_SIZE>>()?;

    // Wait for sem1 to post before touching shared memory.
    shmbuf.sem1.wait()?;

    shmbuf.buf.make_ascii_uppercase();

    // Post sem2 to tell the peer that it can access data in shared memory.
    shmbuf.sem2.post()?;

    Ok(())
}
