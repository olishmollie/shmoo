use std::io::Result;

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() -> Result<()> {
    let mut tx = MsgQueue::<Msg>::open("/ping")?;
    let mut rx = MsgQueue::<Msg>::open("/pong")?;

    loop {
        tx.send(PING)?;
        let msg = rx.recv()?;
        if msg == DONE {
            break;
        }
        assert_eq!(msg, PONG);
    }

    Ok(())
}
