#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use std::{io::Result, process::Command};

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse::<u32>().unwrap();

    let mut mq = MsgQueue::<Msg>::new("shmoo", 1)?;

    let mut ping = Command::new("target/debug/examples/ping").spawn()?;

    for _ in 0..n {
        mq.send(PING)?;
        let pong = mq.recv()?;
        assert_eq!(pong, PONG);
    }

    mq.send(DONE)?;
    ping.wait()?;

    Ok(())
}
