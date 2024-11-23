#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
// const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() {
    let mut mq = MsgQueue::<Msg>::open("/ping").unwrap();

    loop {
        mq.send(PING).unwrap();
        let pong = mq.recv().unwrap();
        println!("ping msg: {}", String::from_utf8(pong.to_vec()).unwrap());
        if pong == DONE {
            break;
        }
        // assert_eq!(pong, PONG);
    }
}
