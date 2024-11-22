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

    let mut mq = MsgQueue::<Msg>::new("/shmoo", 1)?;

    println!("Spawning ping process...");
    let mut ping = Command::new("target/debug/examples/ping").spawn()?;

    for _ in 0..n {
        //println!("PONG: Waiting for ping...");
        //let ping = mq.recv()?;
        //assert_eq!(
        //    ping,
        //    PING,
        //    "PONG: expected ping, got {}",
        //    String::from_utf8(ping.to_vec()).unwrap()
        //);
        println!("Sending pong...");
        mq.send(PONG)?;
    }

    mq.send(DONE)?;
    ping.wait()?;

    Ok(())
}
