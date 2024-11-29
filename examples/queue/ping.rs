pub mod msg_queue;

use msg_queue::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn main() {
    let mut tx = MsgQueue::<Msg>::open("/ping").unwrap();
    let mut rx = MsgQueue::<Msg>::open("/pong").unwrap();

    loop {
        tx.send(PING).unwrap();
        let msg = rx.recv().unwrap();
        if msg == DONE {
            break;
        }
        assert_eq!(msg, PONG);
    }
}
