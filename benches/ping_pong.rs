pub mod shmbuf;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::process::Command;

use shmbuf::Shmbuf;
use shmoo::Shm;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn bench(c: &mut Criterion) {
    let n = 1000;
    let mut shm = Shm::options()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o644)
        .map("/shmoo", std::mem::size_of::<Shmbuf<4>>())
        .unwrap();

    let shmbuf = shm.construct_mut::<Shmbuf<4>>().unwrap();
    let mut buf = vec![0u8; 4];

    let mut peer = Command::new("target/release/examples/ping")
        .spawn()
        .unwrap();

    let mut group = c.benchmark_group("ping_pong_throughput");
    group.throughput(Throughput::Elements(n));
    group.bench_function("ping_pong", |b| {
        b.iter(|| {
            for _ in 0..n {
                // Wait for ping to post.
                shmbuf.sem1.wait().unwrap();

                // Check for ping.
                shmbuf.read(&mut buf);
                assert_eq!(buf, PING);

                // Send a pong.
                shmbuf.write(PONG);
                shmbuf.sem2.post().unwrap();
            }
        })
    });

    shmbuf.sem1.wait().unwrap();
    shmbuf.write(DONE);
    shmbuf.sem2.post().unwrap();
    peer.wait().unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);
