pub mod shmbuf;

use std::{error::Error, process::Command};

use shmbuf::Shmbuf;
use shmoo::Shm;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse::<u32>().unwrap();

    let mut shm = Shm::options()
        .read(true)
        .write(true)
        .create(true)
        .map("/shmoo", std::mem::size_of::<Shmbuf<4>>())?;

    let shmbuf = shm.construct_mut::<Shmbuf<4>>()?;
    let mut buf = vec![0u8; 4];

    #[cfg(debug_assertions)]
    let target = "target/debug/examples/ping";
    #[cfg(not(debug_assertions))]
    let target = "target/release/examples/ping";

    let mut peer = Command::new(target).spawn()?;

    for _ in 0..n {
        // Wait for ping to post.
        shmbuf.sem1.wait()?;

        // Check for ping.
        shmbuf.read(&mut buf);
        assert_eq!(buf, PING);

        // Send a pong.
        shmbuf.write(PONG);
        shmbuf.sem2.post()?;
    }

    shmbuf.sem1.wait()?;
    shmbuf.write(DONE);
    shmbuf.sem2.post()?;

    peer.wait()?;

    Ok(())
}
