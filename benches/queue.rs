pub mod msg_queue;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::process::Command;

use msg_queue::MsgQueue;

type Msg = [u8; 4];

const PING: Msg = *b"ping";
const PONG: Msg = *b"pong";
const DONE: Msg = *b"done";

fn bench(c: &mut Criterion) {
    let n = 1000;

    let mut tx = MsgQueue::<Msg>::new("/pong", 1).unwrap();
    let mut rx = MsgQueue::<Msg>::new("/ping", 1).unwrap();

    let mut peer = Command::new("target/release/examples/queue_ping")
        .spawn()
        .unwrap();

    let mut group = c.benchmark_group("ping_pong_throughput");
    group.throughput(Throughput::Elements(n));
    group.bench_function("ping_pong", |b| {
        b.iter(|| {
            for _ in 0..n {
                let msg = rx.recv().unwrap();
                assert_eq!(msg, PING);
                tx.send(PONG).unwrap();
            }
        })
    });

    tx.send(DONE).unwrap();
    peer.wait().unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);
