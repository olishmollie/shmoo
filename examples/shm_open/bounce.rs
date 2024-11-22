#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use std::io::Result;

use shmbuf::{Shmbuf, BUF_SIZE};
use shmoo::Mmap;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let shmpath = &args[1];

    let mut mem = Mmap::options()
        .read(true)
        .write(true)
        .mode(0o644)
        .create(true)
        .with_capacity(shmpath, 4096)?;

    let shmbuf = Shmbuf::<BUF_SIZE>::new(&mut mem)?;

    // Wait for sem1 to post before touching shared memory.
    shmbuf.sem1.wait()?;

    shmbuf.buf.make_ascii_uppercase();

    // Post sem2 to tell the peer that it can access data in shared memory.
    shmbuf.sem2.post()?;

    Ok(())
}
