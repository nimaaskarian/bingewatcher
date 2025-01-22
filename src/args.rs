// vim:foldmethod=marker
// imports{{{
use clap::Parser;
use clap_complete::Shell;
use std::{
    cmp::Ordering, fs, io::{self, Write}, path::PathBuf, process
};
use crate::{PrintMode, Serie, utils, episodate};
//}}}

// Args {{{
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Search a query among the series
    #[arg(short, long)]
    pub search: Option<String>,

    /// Add watched
    #[arg(short = 'a', long, default_value_t = 0)]
    pub watch: usize,

    /// Remove watched
    #[arg(short = 'r', long, default_value_t = 0)]
    pub unwatch: usize,

    /// Delete selected series
    #[arg(short, long, default_value_t = false)]
    pub delete: bool,

    /// Delete selected series without asking for confirmation
    #[arg(short = 'D', long, default_value_t = false)]
    pub delete_noask: bool,

    /// Print current season of selected series
    #[arg(short = 'p', long, default_value="normal")]
    pub print_mode: PrintMode,

    /// Add an series from episodate API (needs internet)
    #[arg(short='o', long, default_value_t=String::new())]
    pub add_online: String,

    /// Update the series with the same watched values, but append new seasons to it.
    /// use with --add-online
    #[arg(short='u', long)]
    pub update_online: bool,

    /// Show details of a series from episodate API (needs internet)
    #[arg(long, default_value_t=String::new())]
    pub detail_online: String,

    /// Search series from episodate API (needs internet)
    #[arg(short='O', long, default_value_t=String::new())]
    pub search_online: String,

    /// Show finished too
    #[arg(short, long, default_value_t = false)]
    pub finished: bool,

    /// Show finished only
    #[arg(short = 'F', long, default_value_t = false)]
    pub only_finished: bool,

    /// Print shell completion
    #[arg(short='c', long)]
    pub completion: Option<Shell>,

    /// Perform a trial run with no changes made
    #[arg(short='n', long)]
    pub dry_run: bool,
    
    /// Convert a serie name to a serie path and print
    #[arg(long, default_value_t=String::new())]
    pub name_to_path: String,

    /// Selected indexes
    #[arg(last = true)]
    pub indexes: Vec<usize>,

    /// Path to todo file (and notes sibling directory)
    #[arg(default_value=utils::append_home_dir(&[".cache", "bingewatcher"]).into_os_string())]
    pub dir: PathBuf,
}
//}}}

pub enum AppMode {
    PrintCompletions(Shell),
    SearchOnline,
    DetailOnline,
    PrintPath,
    MainDoNothing,
}

impl Args {
    pub fn app_mode(&mut self) -> AppMode {
        if let Some(generator) = self.completion {
            return AppMode::PrintCompletions(generator);
        }
        if !self.search_online.is_empty() {
            return AppMode::SearchOnline
        }
        if !self.detail_online.is_empty() {
            return AppMode::DetailOnline
        }
        if self.update_online && self.add_online.is_empty() {
            eprintln!("Warning: you used update-online with no add-online. Ignoring it.");
        }
        if !self.name_to_path.is_empty() {
            return AppMode::PrintPath;
        }
        let mut series: Vec<Serie> = match (!self.add_online.is_empty(), self.only_finished, self.finished)  {
            (false, false, true) |
            (true, _, _) => utils::read_series_dir(&self.dir,None),
            (_, true, _) => utils::read_series_dir(&self.dir,Some(Serie::is_finished)),
            _ => utils::read_series_dir(&self.dir,Some(Serie::is_not_finished)),

        };
        if !self.add_online.is_empty() {
            self.add_online(&mut series);
        }
        if let Some(search) = self.search.as_ref() {
            if let Some(index) = series.iter().position(|serie| serie.matches(search)) {
                self.indexes.push(index);
            } else {
                eprintln!("ERROR: search with query \"{}\" had no results.", search);
                process::exit(1);
            }
        }
        if !self.indexes.is_empty() {
            self.manipulate_series(series);
        } else {
            self.list_series(series);
        }
        AppMode::MainDoNothing
    }

    #[inline(always)]
    fn add_online(&mut self, series: &mut Vec<Serie>) {
        let serie = episodate::request_detail(&self.add_online);
        if let Some(index) = series.iter().position(|s| s.name == serie.name)  {
            if self.update_online {
                eprintln!("The serie \"{}\" already exists. Updating it...", serie.name);
                let old_serie = &mut series[index];
                old_serie.merge_serie(&serie);
                self.indexes.push(index);
            } else {
                eprintln!("ERROR: The serie \"{}\" already exists.", serie.name);
                process::exit(1);
            }
        } else {
            self.indexes.push(series.len());
            series.push(serie);
        }
    }

    #[inline(always)]
    fn manipulate_series(&self, mut series: Vec<Serie>) {
        for &index in &self.indexes {
            let current_serie = &mut series[index];
            match self.watch.cmp(&self.unwatch) {
                Ordering::Less => {
                    let unwatch_count = self.unwatch-self.watch;
                    current_serie.unwatch(unwatch_count);
                    println!("Unwatched {} episode(s) from {}.",current_serie.name, unwatch_count);
                }
                Ordering::Greater => {
                    current_serie.watch(self.watch-self.unwatch);
                    let watched_count = self.watch-self.unwatch;
                    println!("Watched {watched_count} episode(s) from {}.", current_serie.name);
                }
                Ordering::Equal => { }
            }
            current_serie.print(&self.print_mode, Some(&self.dir));
            if !self.dry_run {
                current_serie.write_in_dir(&self.dir).expect("Write failed");
            }
            if self.delete || self.delete_noask {
                if !self.delete_noask {
                    let mut input = String::from("y");
                    eprint!("Do you want to delete \"{}\" [Y/n] ", current_serie.name);
                    if self.dry_run {
                        eprint!("(dry-run) ")
                    }
                    io::stderr().flush().expect("Flushing stdout failed.");
                    io::stdin().read_line(&mut input).expect("Reading input failed");
                    if input.trim() == "n" {
                        continue;
                    }
                }
                let path = &self.dir.join(current_serie.filename());
                if self.dry_run {
                    eprintln!("Deleted {} (dry-run)", path.to_str().unwrap());
                } else {
                    if let Err(e) = fs::remove_file(path) {
                        eprintln!("ERROR: Couldn't delete {}. Produced the following error:\n{}", path.to_str().unwrap(), e);
                        process::exit(1);
                    } else {
                        eprintln!("Deleted {}", path.to_str().unwrap());
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn list_series(&self, series: Vec<Serie>) {
        for serie in series {
            serie.print(&self.print_mode, Some(&self.dir));
        }
    }
}

