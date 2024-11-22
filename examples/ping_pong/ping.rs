#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use shmoo::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() {
    let mut mq = MsgQueue::<Msg>::open("/shmoo").unwrap();

    loop {
        println!("PING: Waiting for pong...");
        let pong = mq.recv().unwrap();
        if pong == DONE {
            break;
        }
        assert_eq!(
            pong,
            PONG,
            "PING: expected pong, got {}",
            String::from_utf8(pong.to_vec()).unwrap()
        );
    }
}
