#[path = "../examples/shmbuf/lib.rs"]
pub mod shmbuf;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::process::Command;

use shmbuf::Shmbuf;
use shmoo::Mmap;

const PING: &[u8] = b"ping";
const PONG: &[u8] = b"pong";
const DONE: &[u8] = b"done";

fn bench(c: &mut Criterion) {
    let n = 1000;
    let mut mem = Mmap::options()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o644)
        .with_capacity("/shmoo", std::mem::size_of::<Shmbuf<4>>())
        .unwrap();

    let mut ping = Command::new("target/release/examples/ping")
        .spawn()
        .unwrap();

    let shmbuf = Shmbuf::<4>::new(&mut mem).unwrap();
    let mut buf = vec![0u8; 4];

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

    shmbuf.write(DONE);
    shmbuf.sem2.post().unwrap();
    ping.wait().unwrap();
}

criterion_group!(benches, bench);
criterion_main!(benches);
