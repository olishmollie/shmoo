#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() {
    let mut mq = MsgQueue::<Msg>::open("shmoo").unwrap();

    loop {
        let ping = mq.recv().unwrap();
        if ping == DONE {
            break;
        }
        mq.send(PONG).unwrap();
        assert_eq!(ping, PING);
    }
}
