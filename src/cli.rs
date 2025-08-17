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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A minimal, file based cli tool that keeps track of the series you watch
pub struct Cli {
    /// Print current season of selected series
    #[arg(short = 'p', long, default_value = "normal", global=true)]
    pub print_mode: PrintMode,

    /// Whether to include finished shows in searchs or not
    #[arg(short, long, default_value = "no-finished", global=true)]
    include: Include,

    /// Whether to include finished shows in searchs or not
    #[arg(short, long, global=true)]
    hidden: bool,

    /// Print shell completion
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Perform a trial run with no changes made
    #[arg(short = 'n', long, global=true)]
    pub dry_run: bool,

    /// Read all series from a directory (respects the BW_DIR variable)
    #[arg(long, short, default_value=std::env::var_os("BW_DIR").unwrap_or(utils::append_home_dir(&[".cache", "bingewatcher"]).into_os_string()), global=true)]
    pub directory: PathBuf,

    /// Files to manipulate (overrides --directory and --include)
    #[arg(global=true)]
    pub files: Vec<PathBuf>,

    /// Force all prompts asking yes/no
    #[arg(short, long)]
    force: bool
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Delete selected series
    #[command(alias = "ls")]
    List {},
    /// Delete selected series
    #[command(alias = "rm", alias = "del")]
    Delete,
    /// Add to watched episodes by a count
    #[command(alias = "add", alias = "w")]
    Watch {
        #[arg()]
        count: usize,
    },
    /// Remove from watched episodes by a count
    #[command(alias = "sub", alias = "u")]
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
    #[command(alias = "online", alias = "o")]
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
        query: Option<String>,
    },
    /// Add a series from online source (needs internet)
    Add {
        #[arg(required=true)]
        name: String,
        /// Whether to update existing serie or not
        #[arg(short, long)]
        update: bool,
    },
    /// Print details of an online show, without adding it
    Detail {
        #[arg(required=true)]
        name: String,
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
            if $self.hidden {
                match $self.include {
                    Include::NoFinished => $self.$func(&mut $series.by_ref().filter(|s_p| s_p.0.is_not_finished()) $(, $arg)*),
                    Include::Finished => $self.$func(&mut $series.by_ref().filter(|s_p| s_p.0.is_finished()) $(, $arg)*),
                    Include::All => $self.$func(&mut $series $(, $arg)*),
                }
            } else {
                match $self.include {
                    Include::NoFinished => $self.$func(&mut $series.by_ref().filter(|s_p| !s_p.1.file_name().unwrap().to_str().unwrap().starts_with('.') && s_p.0.is_not_finished()) $(, $arg)*),
                    Include::Finished => $self.$func(&mut $series.by_ref().filter(|s_p| !s_p.1.file_name().unwrap().to_str().unwrap().starts_with('.') && s_p.0.is_finished()) $(, $arg)*),
                    Include::All => $self.$func(&mut $series.by_ref().filter(|s_p| !s_p.1.file_name().unwrap().to_str().unwrap().starts_with('.')) $(, $arg)*),
                }
            }
        } else {
            let mut paths = std::mem::take(&mut $self.files);
            $self.$func(paths.iter_mut().flat_map(|entry| Serie::from_file(entry).map(|serie| (serie, std::mem::take(entry)))) $(, $arg)*)
        }
    }
}

impl Cli {
    pub fn execute(&mut self) {
        let mut series = utils::series_dir_reader(&self.directory).expect("Couldn't open dir");
        let files_empty = self.files.is_empty();

        match self.command {
            Some(Commands::Completions { shell }) => {
                utils::print_completions(shell, &mut Cli::command());
                return;
            }
            Some(Commands::Episodate { ref mut command }) => match command {
                OnlineCommands::Search { ref query } => {
                    let query = query.as_ref().map_or("", |v| v);
                    episodate::search_write_to_stdout(query);
                }
                OnlineCommands::Add {
                    ref mut name,
                    update,
                } => {
                    let name = mem::take(name);
                    let update = *update;
                    call_series!(
                        self,
                        series,
                        add_online,
                        episodate::request_detail,
                        name,
                        update,
                        files_empty
                    );
                }
                OnlineCommands::Detail { name } => {
                    let serie = episodate::request_detail(&name);
                    serie.print(&PrintMode::Extended, None);
                }
            },
            Some(Commands::Delete ) => {
                call_series!(self, series, delete_series);
            }
            Some(Commands::Watch { count }) => {
                call_series!(self, series, watch_series, count, !files_empty || self.force);
            }
            Some(Commands::Unwatch { count }) => {
                call_series!(self, series, unwatch_series, count, !files_empty || self.force);
            }
            Some(Commands::Wikipedia { .. }) => todo!(),
            None | Some(Commands::List {}) => {
                call_series!(self, series, list_series);
            }
        }
    }

    #[inline(always)]
    pub fn add_online(
        &mut self,
        mut series: impl Iterator<Item = (Serie, PathBuf)>,
        fetch_function: fn(&str) -> Serie,
        name: String,
        update: bool,
        files_empty: bool,
    ) {
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
                if !self.dry_run {
                    serie.write(path);
                }
            } else {
                eprintln!("WARNING: Can't detect the file to write on. Writing on stdout...");
                serie.print(&PrintMode::Content, None);
            }
        }
    }

    #[inline(always)]
    fn write_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>) {
        for (mut serie, path) in series {
            serie.print(&self.print_mode, Some(&path));
            if !self.dry_run {
                serie.write(path).expect("Write failed");
            }
        }
    }

    // !files_empty is used as the force argument. this way we confirm only if files are empty
    fn watch_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>, count: usize, force: bool) {
        for (mut serie, path) in series {
            if !force {
                let prompt = format!("Do you want to watch {count} episodes from \"{}\" [Y/n] ", serie.name);
                if !yes_no_confirmation(prompt) {
                    continue;
                }
            }
            serie.watch(count);
            serie.print(&self.print_mode, Some(&path));
            if !self.dry_run {
                serie.write(path).expect("Write failed");
            }
        }
    }

    fn unwatch_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>, count: usize, force: bool) {
        for (mut serie, path) in series {
            if !force {
                let prompt = format!("Do you want to unwatch {count} episodes from \"{}\" [Y/n] ", serie.name);
                if !yes_no_confirmation(prompt) {
                    continue;
                }
            }
            serie.unwatch(count);
            serie.print(&self.print_mode, Some(&path));
            if !self.dry_run {
                serie.write(path).expect("Write failed");
            }
        }
    }

    #[inline(always)]
    fn delete_series(&self, series: impl Iterator<Item = (Serie, PathBuf)>) {
        for (mut serie, path) in series {
            if !self.force {
                let prompt = format!("Do you want to delete \"{}\" [Y/n] ", serie.name) + if self.dry_run {
                    "(dry-run) "
                } else {
                    ""
                };
                if !yes_no_confirmation(prompt) {
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

fn yes_no_confirmation(prompt: String) -> bool {
    eprint!("{}", prompt);
    io::stderr().flush().expect("Flushing stdout failed.");
    let mut input = String::with_capacity(2);
    io::stdin()
        .read_line(&mut input)
        .expect("Reading input failed");
    let input = input.trim().to_lowercase();
    if input == "n" {
        return false
    }
    return true
}
