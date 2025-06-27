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
    let mut args = bw::args::Cli::parse_from(&[""]);
    c.bench_function("read system series", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

fn read_system_series_finished(c: &mut Criterion) {
    let mut args = bw::args::Cli::parse_from(["","-i","f"]);
    c.bench_function("read system series finished", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

fn read_system_series_all(c: &mut Criterion) {
    let mut args = bw::args::Cli::parse_from(["","-i", "a"]);
    c.bench_function("read system series all", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

fn search_system_series(c: &mut Criterion) {
    let mut args = bw::args::Cli::parse_from(["","-s", "invincible"]);
    c.bench_function("search system series", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

fn path_system_series(c: &mut Criterion) {
    let mut args = bw::args::Cli::parse_from(["bw","/home/nima/.cache/bingewatcher/Rick and Morty.bw"]);
    assert!(!args.files.is_empty());
    c.bench_function("path system series", |b| b.iter(||{
        black_box(&mut args).app_mode()
    }));
}

criterion_group!(benches,write, read_system_series, read_system_series_finished, read_system_series_all, search_system_series, path_system_series);
criterion_main!(benches);
