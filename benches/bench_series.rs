use std::{env, fs::File, hint::black_box, path::PathBuf};

use bw::episodate::request_detail;

use criterion::{criterion_group, criterion_main, Criterion};

fn write(c: &mut Criterion) {
    let serie = request_detail("breaking-bad");
    c.bench_function("write breaking bad", |b| b.iter(||{
        black_box(&serie).write(PathBuf::from("./breaking-bad.bw"))
    }));
}

criterion_group!(benches,write);
criterion_main!(benches);
