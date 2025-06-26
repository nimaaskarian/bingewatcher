// vim:foldmethod=marker
// imports{{{
use crate::{
    episodate,
    serie::{PrintMode, Serie},
    utils,
};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::mem;
use std::{
    cmp::Ordering,
    fs,
    io::{self, Write},
    path::PathBuf,
    process,
};
//}}}

// Args {{{
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Search a query among the series and select matching ("*" matches all the series)
    #[arg(short, long, default_value_t=String::new())]
    pub search: String,

    /// Print current season of selected series
    #[arg(short = 'p', long, default_value = "normal")]
    pub print_mode: PrintMode,

    /// Whether to include finished shows in searchs or not
    #[arg(short, long, default_value = "no-finished")]
    include: Include,

    /// Print shell completion
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Perform a trial run with no changes made
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Read all series from a directory (respects the BW_DIR variable)
    #[arg(long, short, default_value=std::env::var_os("BW_DIR").unwrap_or(utils::append_home_dir(&[".cache", "bingewatcher"]).into_os_string()))]
    pub directory: PathBuf,

    /// Files to manipulate (overrides --directory and --include)
    #[arg()]
    pub files: Vec<PathBuf>,
}
//}}}
//

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Delete selected series
    #[command(alias = "ls")]
    List {},
    /// Delete selected series
    Delete {
        /// Force delete selected series without asking for confirmation
        #[arg(short = 'f', long, default_value_t = false)]
        force: bool,
    },
    /// Add to watched episodes by a count
    Watch {
        #[arg()]
        count: usize,
    },
    /// Remove from watched episodes by a count
    Unwatch {
        #[arg()]
        count: usize,
    },
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Fetching series, from episodate API
    Episodate {
        #[command(subcommand)]
        command: OnlineCommands,
    },
    /// Fetching series, from en.wikipedia.org
    Wikipedia {
        #[command(subcommand)]
        command: OnlineCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum OnlineCommands {
    /// Search online source with a query
    Search {
        /// Query to search with
        #[arg()]
        query: String,
    },
    /// Add a series from online source (needs internet)
    Add {
        #[arg()]
        names: Vec<String>,
        /// Whether to update existing serie or not
        #[arg(short, long)]
        update: bool,
    },
    /// Print details of an online show, without adding it
    Detail {
        #[arg()]
        names: Vec<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum Include {
    #[value(alias = "n")]
    NoFinished,
    #[value(alias = "a")]
    All,
    #[value(alias = "f")]
    Finished,
}

pub enum AppMode {
    PrintCompletions(Shell),
    SearchOnline,
    DetailOnline,
    PrintPath,
    MainDoNothing,
}

macro_rules! call_series {
    ($self: expr, $series:expr, $func:ident $(, $arg:expr)*) => {
        if $self.files.is_empty() {
            match $self.include {
                Include::NoFinished => $self.$func(&mut $series.by_ref().filter(|s_p| s_p.0.is_not_finished()) $(, $arg)*),
                Include::Finished => $self.$func(&mut $series.by_ref().filter(|s_p| s_p.0.is_finished()) $(, $arg)*),
                Include::All => $self.$func(&mut $series $(, $arg)*),
            }
        } else {
            let mut paths = std::mem::take(&mut $self.files);
            $self.$func(paths.iter_mut().flat_map(|entry| Serie::from_file(entry).map(|serie| (serie, std::mem::take(entry)))) $(, $arg)*)
        }
    }
}

impl Args {
    pub fn execute(&mut self) {
        let mut series = utils::series_dir_reader(&self.directory).expect("Couldn't open dir");
        let files_empty = self.files.is_empty();

        match self.command {
            Some(Commands::Completions { shell }) => {
                utils::print_completions(shell, &mut Args::command());
                return;
            }
            Some(Commands::Episodate { ref mut command }) => match command {
                OnlineCommands::Search { ref query } => {
                    episodate::search_write_to_stdout(query);
                }
                OnlineCommands::Add {
                    ref mut names,
                    update,
                } => {
                    let names = mem::take(names);
                    let update = *update;
                    call_series!(
                        self,
                        series,
                        add_online,
                        episodate::request_detail,
                        names,
                        update,
                        files_empty
                    );
                }
                OnlineCommands::Detail { names } => {
                    for name in names {
                        let serie = episodate::request_detail(&name);
                        serie.print(&PrintMode::Extended, None);
                    }
                }
            },
            Some(Commands::Delete { force }) => {
                call_series!(self, series, delete_series, force);
            }
            Some(Commands::Watch { count }) => {
                call_series!(self, series, watch_series, count);
            }
            Some(Commands::Unwatch { count }) => {
                call_series!(self, series, unwatch_series, count);
            }
            Some(Commands::Wikipedia { .. }) => todo!(),
            None | Some(Commands::List {}) => {}
        }
        call_series!(self, series, list_series);
    }

    #[inline(always)]
    fn list_or_manipulate_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>) {
        if !self.search.is_empty() {
            if self.search == "*" {
                self.manipulate_series(series)
            } else {
                self.manipulate_series(series.filter(|s_p| s_p.0.matches(&self.search)))
            }
        } else {
            self.list_series(series)
        }
    }

    #[inline(always)]
    pub fn add_online(
        &mut self,
        mut series: impl Iterator<Item = (Serie, PathBuf)>,
        fetch_function: fn(&str) -> Serie,
        names: Vec<String>,
        update: bool,
        files_empty: bool,
    ) {
        for name in names {
            let serie = fetch_function(&name);
            if let Some((mut old_serie, path)) = series.find(|s_p| s_p.0.name == serie.name) {
                if update {
                    eprintln!(
                        "INFO: The serie \"{}\" already exists. Updating it...",
                        serie.name
                    );
                    old_serie.merge_serie(&serie);
                    if !self.dry_run {
                        old_serie.print(&self.print_mode, Some(&path));
                        old_serie.write(path);
                    }
                } else {
                    eprintln!("ERROR: The serie \"{}\" already exists.", serie.name);
                    process::exit(1);
                }
            } else {
                if files_empty {
                    let path = self.directory.join(serie.filename());
                    serie.print(&self.print_mode, Some(&path));
                    serie.write(path);
                } else {
                    eprintln!("WARNING: Can't detect the file to write on. Writing on stdout...");
                    serie.print(&PrintMode::Content, None);
                }
            }
        }
    }

    #[inline(always)]
    fn manipulate_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>) {
        for (mut serie, path) in series {
            serie.print(&self.print_mode, Some(&path));
            if !self.dry_run {
                serie.write(path).expect("Write failed");
            }
        }
    }

    fn watch_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>, count: usize) {
        for (mut serie, path) in series {
            serie.watch(count);
        }
    }

    fn unwatch_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>, count: usize) {
        for (mut serie, path) in series {
            serie.unwatch(count);
        }
    }

    #[inline(always)]
    fn delete_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>, force: bool) {
        for (mut serie, path) in series {
            if !force {
                let mut input = String::from("y");
                eprint!("Do you want to delete \"{}\" [Y/n] ", serie.name);
                if self.dry_run {
                    eprint!("(dry-run) ")
                }
                io::stderr().flush().expect("Flushing stdout failed.");
                io::stdin()
                    .read_line(&mut input)
                    .expect("Reading input failed");
                if input.trim() == "n" {
                    continue;
                }
            }
            if self.dry_run {
                eprintln!("Deleted {} (dry-run)", path.to_str().unwrap());
            } else if let Err(e) = fs::remove_file(&path) {
                eprintln!(
                    "ERROR: Couldn't delete {}. Produced the following error:\n{}",
                    path.to_str().unwrap(),
                    e
                );
                process::exit(1);
            } else {
                eprintln!("Deleted {}", path.to_str().unwrap());
            }
        }
    }

    #[inline(always)]
    fn list_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>) {
        for (serie, path) in series {
            serie.print(&self.print_mode, Some(&path));
        }
    }
}
