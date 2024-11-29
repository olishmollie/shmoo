pub mod msg_queue;

use msg_queue::MsgQueue;
use std::{error::Error, process::Command};

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() -> Result<(), Box<dyn Error>> {
    let n = std::env::args().collect::<Vec<String>>()[1]
        .parse::<u32>()
        .unwrap();

    let mut tx = MsgQueue::<Msg>::new("/pong", 1)?;
    let mut rx = MsgQueue::<Msg>::new("/ping", 1)?;

    #[cfg(debug_assertions)]
    let target = "target/debug/examples/queue_ping";
    #[cfg(not(debug_assertions))]
    let target = "target/release/examples/queue_ping";

    let mut peer = Command::new(target).spawn().unwrap();

    for _ in 0..n {
        let msg = rx.recv()?;
        assert_eq!(msg, PING);
        tx.send(PONG)?;
    }

    tx.send(DONE)?;
    peer.wait()?;

    Ok(())
}
