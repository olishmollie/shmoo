#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use std::{io::Result, process::Command};

use shmoo::MsgQueue;

type Msg = [u8; 4];

// const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() -> Result<()> {
    let n = 10;

    let mut mq = MsgQueue::<Msg>::new("/ping", 1)?;

    let mut ping = Command::new("target/debug/examples/ping").spawn()?;

    for _ in 0..n {
        let ping = mq.recv()?;
        // assert_eq!(ping, PING);
        println!("pong msg: {}", String::from_utf8(ping.to_vec()).unwrap());
        mq.send(PONG)?;
    }

    mq.send(DONE)?;
    ping.wait()?;

    Ok(())
}
