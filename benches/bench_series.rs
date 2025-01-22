use std::{hint::black_box, path::PathBuf};

use clap::Parser;
use criterion::{criterion_group, criterion_main, Criterion};


fn write(c: &mut Criterion) {
    let serie = bw::episodate::request_detail("breaking-bad");
    c.bench_function("write breaking bad", |b| b.iter(||{
        black_box(&serie).write(PathBuf::from("./breaking-bad.bw"))
    }));
}

fn read_system_series(c: &mut Criterion) {
    let mut args = bw::args::Args::parse_from(["";0]);
    c.bench_function("read system series", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

criterion_group!(benches,write, read_system_series);
criterion_main!(benches);
