mod onlineserie;
mod serie;

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use onlineserie::{online_tv_show, request_detail};
use serie::{Serie, SeriePrint};
use std::{
    cell::RefCell, fs, io::{self, Write}, path::{Path, PathBuf}, process
};

use home::home_dir;

#[inline(always)]
pub fn append_home_dir(strs: &[&str]) -> PathBuf {
    let mut out = home_dir().unwrap();
    for str in strs {
        out.push(str);
    }
    out
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Search a query among the series
    #[arg(short, long)]
    search: Option<String>,

    /// Add watched
    #[arg(short = 'a', long, default_value_t = 0)]
    watch: usize,

    /// Remove watched
    #[arg(short = 'r', long, default_value_t = 0)]
    unwatch: usize,

    /// Extended series view, including each episode
    #[arg(short = 'E', long, default_value_t = false)]
    extended: bool,

    #[arg(short='c', long)]
    completion: Option<Shell>,

    /// Show next episode when printing
    #[arg(short = 'e', long, default_value_t = false)]
    episode: bool,

    /// Delete selected series
    #[arg(short, long, default_value_t = false)]
    delete: bool,

    /// Delete selected series without asking for confirmation
    #[arg(short = 'D', long, default_value_t = false)]
    delete_noask: bool,

    /// Print current season of selected series
    #[arg(short = 'p', long, default_value="normal")]
    print_mode: SeriePrint,

    /// Add an series from episodate API (needs internet)
    #[arg(short='o', long, default_value_t=String::new())]
    add_online: String,

    /// Show details of a series from episodate API (needs internet)
    #[arg(long, default_value_t=String::new())]
    detail_online: String,

    /// Search series from episodate API (needs internet)
    #[arg(short='O', long, default_value_t=String::new())]
    search_online: String,

    /// Show finished too
    #[arg(short, long, default_value_t = false)]
    finished: bool,

    /// Show finished too
    #[arg(short = 'F', long, default_value_t = false)]
    only_finished: bool,

    /// Selected indexes
    #[arg(last = true)]
    indexes: Vec<usize>,
}

impl Args {
    async fn app_mode(&mut self) -> AppMode {
        if let Some(generator) = self.completion {
            return AppMode::PrintCompletions(generator);
        }
        if !self.search_online.is_empty() {
            return AppMode::SearchOnline
        }
        if !self.detail_online.is_empty() {
            return AppMode::DetailOnline
        }
        let dir = append_home_dir(&[".cache", "bingewatcher"]);
        let mut series: Vec<Serie> = if self.only_finished {
            read_dir_for_series(&dir,Some(Serie::is_finished))
        } else if !self.finished {
            read_dir_for_series(&dir,Some(Serie::is_not_finished))
        } else {
            read_dir_for_series(&dir,None)
        };
        if !self.add_online.is_empty() {
            if let Ok(serie) = request_detail(&self.add_online).await {
                if series.iter().any(|s| s.name == serie.name)  {
                    eprintln!("The serie \"{}\" already exists.", serie.name);
                    process::exit(1);
                }
                self.indexes.push(series.len());
                series.push(serie);
            }
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
            return AppMode::ManipulateSeries(RefCell::new(series), RefCell::new(dir));
        } else {
            self.list_series(&series);
        }
        AppMode::Nothing
    }

    fn list_series(&self, series: &[Serie]) {
        for (index, serie) in series.iter().enumerate() {
            print!("{index}: ");
            serie.print(&self.print_mode);
        }
    }
}

enum AppMode {
    PrintCompletions(Shell),
    SearchOnline,
    DetailOnline,
    ManipulateSeries(RefCell<Vec<Serie>>, RefCell<PathBuf>),
    Nothing,
}

#[inline]
fn read_dir_for_series(dir: &Path, filter: Option<fn(&Serie) -> bool>) -> Vec<Serie> {
    std::fs::create_dir_all(dir);
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

#[inline]
fn print_watched_count(watch: usize, unwatch: usize, name: &str) {
    let watched_count: isize = watch as isize - unwatch as isize;
    match watched_count {
        ..=-1 => println!("Unwatched {} episode(s) from {name}.", -watched_count),
        1.. => println!("Watched {watched_count} episode(s) from {name}."),
        _ => {}
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

use AppMode::*;
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut args = Args::parse();
    match args.app_mode().await {
        PrintCompletions(shell) => {
            print_completions(shell, &mut Args::command());
        }
        SearchOnline => {
            let _ = online_tv_show(args.search_online).await;
        }
        DetailOnline => {
            if let Ok(serie) = request_detail(&args.detail_online).await {
                serie.print(&SeriePrint::Extended);
                process::exit(0);
            }
        }
        ManipulateSeries(series, dir) => {
            for index in args.indexes {
                let current_serie = &mut series.borrow_mut()[index];
                current_serie.watch(args.watch);
                current_serie.unwatch(args.unwatch);
                print_watched_count(args.watch, args.unwatch, &current_serie.name);

                current_serie.print(&args.print_mode);
                current_serie.write_in_dir(&dir.borrow())?;
                if args.delete || args.delete_noask {
                    let mut input = String::from("y");
                    if !args.delete_noask {
                        print!("Do you want to delete \"{}\" [Y/n] ", current_serie.name);
                        io::stdout().flush();
                        io::stdin().read_line(&mut input)?;
                    }
                    if input.trim() != "n" {
                        let path = &dir.borrow().join(&current_serie.filename());
                        fs::remove_file(path)?;
                        println!("Deleted {}", path.to_str().unwrap());
                    }
                }
            }
        }
        Nothing => {}
    }
    Ok(())
}
