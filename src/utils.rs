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
-> io::Result<FlatMap<fs::ReadDir, Option<(Serie, PathBuf)>, impl FnMut(Result<fs::DirEntry, io::Error>) -> Option<(Serie, PathBuf)>>>
{
    let _ = std::fs::create_dir_all(dir);
    let dir = fs::read_dir(dir)?;
    Ok(dir.flat_map(|entry| {
        let path = entry.expect("File error").path();
        Serie::from_file(&path).map(|serie| (serie,path))
    }))
}

// pub fn series_paths_reader<'a>(dir: &'a[PathBuf]) -> FlatMap<std::slice::Iter<'a, PathBuf>, Option<Serie>, impl FnMut(&PathBuf) -> Option<Serie>>
// {
//     // let _ = std::fs::create_dir_all(dir);
//     // let dir = fs::read_dir(dir)?;
//     dir.iter().flat_map(|entry| Serie::from_file(&entry))
// }

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
