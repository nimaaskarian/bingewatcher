// vim:foldmethod=marker
// imports{{{
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use std::{
    fs, io::{self, Write}, path::PathBuf, process
};

mod utils;
mod onlineserie;
mod serie;
use serie::{Serie, SeriePrint};
use onlineserie::{online_tv_show, request_detail};
//}}}

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

enum AppMode {
    PrintCompletions(Shell),
    SearchOnline,
    DetailOnline,
    ListOrManipulate,
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
        let dir = utils::append_home_dir(&[".cache", "bingewatcher"]);
        let mut series: Vec<Serie> = if self.only_finished {
            utils::read_series_dir(&dir,Some(Serie::is_finished))
        } else if !self.finished {
            utils::read_series_dir(&dir,Some(Serie::is_not_finished))
        } else {
            utils::read_series_dir(&dir,None)
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
            self.manipulate_series(&mut series, dir);
        } else {
            self.list_series(&series);
        }
        AppMode::ListOrManipulate
    }

    #[inline]
    fn manipulate_series(&self, series: &mut Vec<Serie>, dir: PathBuf) {
        for &index in &self.indexes {
            let current_serie = &mut series[index];
            if self.watch > self.unwatch {
                current_serie.watch(self.watch-self.unwatch);
                let watched_count = self.watch-self.unwatch;
                println!("Watched {watched_count} episode(s) from {}.", current_serie.name);
            } else {
                let unwatch_count = self.unwatch-self.watch;
                current_serie.unwatch(unwatch_count);
                println!("Unwatched {} episode(s) from {}.",current_serie.name, unwatch_count);
            }
            current_serie.print(&self.print_mode);
            current_serie.write_in_dir(&dir);
            if self.delete || self.delete_noask {
                let mut input = String::from("y");
                if !self.delete_noask {
                    print!("Do you want to delete \"{}\" [Y/n] ", current_serie.name);
                    io::stdout().flush();
                    io::stdin().read_line(&mut input);
                }
                if input.trim() != "n" {
                    let path = &dir.join(&current_serie.filename());
                    fs::remove_file(path);
                    println!("Deleted {}", path.to_str().unwrap());
                }
            }
        }
    }

    #[inline]
    fn list_series(&self, series: &[Serie]) {
        for (index, serie) in series.iter().enumerate() {
            print!("{index}: ");
            serie.print(&self.print_mode);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    use AppMode::*;
    let mut args = Args::parse();
    match args.app_mode().await {
        PrintCompletions(shell) => {
            utils::print_completions(shell, &mut Args::command());
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
        ListOrManipulate => {}
    }
    Ok(())
}
