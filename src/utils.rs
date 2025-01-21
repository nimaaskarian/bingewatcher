use super::Serie;
use home::home_dir;
use clap::Command;
use clap_complete::{generate, Generator};
use std::{
    fs, io::{self}, path::{Path, PathBuf},
};

#[inline(always)]
pub fn append_home_dir(strs: &[&str]) -> PathBuf {
    let mut out = home_dir().unwrap();
    for str in strs {
        out.push(str);
    }
    out
}

pub fn read_series_dir(dir: &Path, filter: Option<fn(&Serie) -> bool>) -> Vec<Serie> {
    let _ = std::fs::create_dir_all(dir);
    if let Ok(dir) = fs::read_dir(dir) {
        let iter = dir.flat_map(|entry| Serie::from_file(&entry.expect("File error").path()));
        if let Some(filter) = filter {
            iter.filter(filter).collect()
        } else {
            iter.collect()
        }
    } else {
        vec![]
    }
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
