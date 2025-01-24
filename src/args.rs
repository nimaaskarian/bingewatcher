// vim:foldmethod=marker
// imports{{{
use clap::{Parser, ValueEnum};
use clap_complete::Shell;
use std::{
    cmp::Ordering, fs, io::{self, Write}, path::PathBuf, process
};
use crate::{
    serie::{Serie, PrintMode},
    utils, episodate,
};
//}}}

// Args {{{
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Search a query among the series
    #[arg(short, long, default_value_t=String::new())]
    pub search: String,

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

    /// Whether to include finished shows in searchs or not
    #[arg(short, long, default_value="no-finished")]
    include: Include,

    /// Print shell completion
    #[arg(short='c', long)]
    pub completion: Option<Shell>,

    /// Perform a trial run with no changes made
    #[arg(short='n', long)]
    pub dry_run: bool,
    
    /// Convert a serie name to a serie path and print
    #[arg(long, default_value_t=String::new())]
    pub name_to_path: String,

    /// Path to todo file (and notes sibling directory)
    #[arg(default_value=utils::append_home_dir(&[".cache", "bingewatcher"]).into_os_string())]
    pub dir: PathBuf,

    /// Files to manipulate (overrides --dir and --include)
    #[arg(last=true)]
    pub files: Vec<PathBuf>,
}
//}}}

#[derive(Debug, Clone, ValueEnum)]
enum Include {
    #[value(alias="n")]
    NoFinished,
    #[value(alias="a")]
    All,
    #[value(alias="f")]
    Finished,
}

pub enum AppMode {
    PrintCompletions(Shell),
    SearchOnline,
    DetailOnline,
    PrintPath,
    MainDoNothing,
}

macro_rules! do_for_paths_or_dir {
    ($self:expr, $func:ident, $series:expr) => {
        do_for_paths_or_dir!($self, $func, $series, $func);
    };
    ($self:expr, $func:ident, $series:expr, $pathsfn:ident) => {
        if $self.files.is_empty() {
            $self.$func($series)
        } else {
            let paths = std::mem::take(&mut $self.files);
            $self.$pathsfn(paths.iter().flat_map(|entry| Serie::from_file(entry)))
        }
    };
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
        let series = utils::series_dir_reader(&self.dir).expect("Couldn't open dir");

        if !self.add_online.is_empty() {
            do_for_paths_or_dir!(self,add_online,series);
            return AppMode::MainDoNothing;
        }
        match self.include {
            Include::NoFinished => do_for_paths_or_dir!(self,list_or_manipulate_series,series.filter(Serie::is_not_finished),manipulate_series),
            Include::Finished => do_for_paths_or_dir!(self,list_or_manipulate_series,series.filter(Serie::is_finished),manipulate_series),
            Include::All => do_for_paths_or_dir!(self,list_or_manipulate_series,series,manipulate_series),
        }
        AppMode::MainDoNothing
    }
    
    #[inline(always)]
    fn list_or_manipulate_series(&self, series: impl Iterator<Item = Serie>) {
        if !self.search.is_empty() {
            self.manipulate_series(series.filter(|s| s.matches(&self.search)))
        } else {
            self.list_series(series)
        }

    }

    #[inline(always)]
    fn add_online(&mut self, mut series: impl Iterator<Item = Serie>) {
        let serie = episodate::request_detail(&self.add_online);
        if let Some(mut old_serie) = series.find(|s| s.name == serie.name)  {
            if self.update_online {
                eprintln!("The serie \"{}\" already exists. Updating it...", serie.name);
                old_serie.merge_serie(&serie);
                if !self.dry_run {
                    old_serie.write_in_dir(&self.dir);
                    old_serie.print(&self.print_mode, Some(&self.dir))
                }
            } else {
                eprintln!("ERROR: The serie \"{}\" already exists.", serie.name);
                process::exit(1);
            }
        } else {
            serie.write_in_dir(&self.dir);
            serie.print(&self.print_mode, Some(&self.dir))
        }
    }

    #[inline(always)]
    fn manipulate_series(&self, series: impl Iterator<Item = Serie>) {
        for mut serie in series {
            match self.watch.cmp(&self.unwatch) {
                Ordering::Less => {
                    let unwatch_count = self.unwatch-self.watch;
                    serie.unwatch(unwatch_count);
                    println!("Unwatched {} episode(s) from {}.",serie.name, unwatch_count);
                }
                Ordering::Greater => {
                    serie.watch(self.watch-self.unwatch);
                    let watched_count = self.watch-self.unwatch;
                    println!("Watched {watched_count} episode(s) from {}.", serie.name);
                }
                Ordering::Equal => { }
            }
            serie.print(&self.print_mode, Some(&self.dir));
            if !self.dry_run {
                serie.write_in_dir(&self.dir).expect("Write failed");
            }
            if self.delete || self.delete_noask {
                if !self.delete_noask {
                    let mut input = String::from("y");
                    eprint!("Do you want to delete \"{}\" [Y/n] ", serie.name);
                    if self.dry_run {
                        eprint!("(dry-run) ")
                    }
                    io::stderr().flush().expect("Flushing stdout failed.");
                    io::stdin().read_line(&mut input).expect("Reading input failed");
                    if input.trim() == "n" {
                        continue;
                    }
                }
                let path = &self.dir.join(serie.filename());
                if self.dry_run {
                    eprintln!("Deleted {} (dry-run)", path.to_str().unwrap());
                } else if let Err(e) = fs::remove_file(path) {
                    eprintln!("ERROR: Couldn't delete {}. Produced the following error:\n{}", path.to_str().unwrap(), e);
                    process::exit(1);
                } else {
                    eprintln!("Deleted {}", path.to_str().unwrap());
                }
            }
        }
    }

    #[inline(always)]
    fn list_series(&self, series: impl Iterator<Item = Serie>) {
        for serie in series {
            serie.print(&self.print_mode, Some(&self.dir));
        }
    }
}

