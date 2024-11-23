#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() {
    let mut tx = MsgQueue::<Msg>::open("/ping").unwrap();
    let mut rx = MsgQueue::<Msg>::open("/pong").unwrap();

    loop {
        //println!("ping");
        tx.send(PING).unwrap();
        let pong = rx.recv().unwrap();
        if pong == DONE {
            break;
        }
        assert_eq!(pong, PONG);
    }
}
