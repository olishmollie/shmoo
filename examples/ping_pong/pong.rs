#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use std::{io::Result, process::Command};

use shmbuf::Shmbuf;
use shmoo::Mmap;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse::<u32>().unwrap();

    let mut mem = Mmap::options()
        .read(true)
        .write(true)
        .mode(0o644)
        .create(true)
        .with_capacity("/shmoo", std::mem::size_of::<Shmbuf<4>>())?;

    let mut ping = Command::new("target/release/examples/ping").spawn()?;

    let shmbuf = Shmbuf::<4>::new(&mut mem).unwrap();
    let mut buf = vec![0u8; 4];

    for _ in 0..n {
        // Wait for ping to post.
        shmbuf.sem1.wait().unwrap();

        // Check for ping.
        shmbuf.read(&mut buf);
        assert_eq!(buf, PING);

        // Send a pong.
        shmbuf.write(PONG);
        shmbuf.sem2.post().unwrap();
    }

    shmbuf.write(DONE);
    shmbuf.sem2.post()?;
    ping.wait()?;

    Ok(())
}
