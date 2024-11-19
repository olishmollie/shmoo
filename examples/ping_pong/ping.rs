#[path = "../shmbuf/lib.rs"]
pub mod shmbuf;

use shmbuf::Shmbuf;
use shmoo::Mmap;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn main() {
    let mut mem = Mmap::options()
        .read(true)
        .write(true)
        .with_capacity("/shmoo", std::mem::size_of::<Shmbuf<4>>())
        .unwrap();

    let shmbuf = Shmbuf::<4>::from_shm_mut(&mut mem).unwrap();
    let mut buf = vec![0u8; 4];

    loop {
        // Send a ping.
        shmbuf.write(PING);
        shmbuf.sem1.post().unwrap();

        // Wait for pong to post.
        shmbuf.sem2.wait().unwrap();

        // Check for pong.
        shmbuf.read(&mut buf);
        if buf == DONE {
            break;
        }
        assert_eq!(buf, PONG);
    }
}
