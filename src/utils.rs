use crate::serie::Serie;
use home::home_dir;
use clap::Command;
use clap_complete::{generate, Generator};
use std::{
    fs, io::{self}, iter::FlatMap, path::{Path, PathBuf}
};

#[inline(always)]
pub fn append_home_dir(strs: &[&str]) -> PathBuf {
    let mut out = home_dir().unwrap();
    for str in strs {
        out.push(str);
    }
    out
}

pub fn series_dir_reader(dir: &Path) 
-> io::Result<FlatMap<fs::ReadDir, Option<Serie>, impl FnMut(Result<fs::DirEntry, io::Error>) -> Option<Serie>>>
{
    let _ = std::fs::create_dir_all(dir);
    let dir = fs::read_dir(dir)?;
    Ok(dir.flat_map(|entry| Serie::from_file(&entry.expect("File error").path())))
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
