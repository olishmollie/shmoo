#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use std::{io::Result, process::Command};

use shmbuf::Shmbuf;
use shmoo::Shm;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse::<u32>().unwrap();

    let mut shm = Shm::options()
        .read(true)
        .write(true)
        .create(true)
        .new("/shmoo", std::mem::size_of::<Shmbuf<4>>())?;

    let shmbuf = shm.construct_mut::<Shmbuf<4>>();
    let mut buf = vec![0u8; 4];

    let mut ping = Command::new("target/debug/examples/ping").spawn()?;

    for _ in 0..n {
        // Wait for ping to post.
        shmbuf.sem1.wait()?;

        // Check for ping.
        shmbuf.read(&mut buf);
        debug_assert_eq!(buf, PING);

        // Send a pong.
        shmbuf.write(PONG);
        shmbuf.sem2.post()?;
    }

    shmbuf.sem1.wait()?;
    shmbuf.write(DONE);
    shmbuf.sem2.post()?;

    ping.wait()?;

    Ok(())
}
