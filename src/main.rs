mod onlineserie;
mod serie;

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use onlineserie::{online_tv_show, request_detail};
use serie::{Serie, SeriePrint};
use std::{
    process, fs, io, path::{Path, PathBuf}
};

use home::home_dir;

#[inline(always)]
pub fn append_home_dir(str: &str) -> PathBuf {
    PathBuf::from(format!("{}/{}", home_dir().unwrap().to_str().unwrap(), str))
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

#[inline]
fn read_dir_for_series(dir: &Path) -> io::Result<Vec<Serie>> {
    let mut series: Vec<Serie> = vec![];

    std::fs::create_dir_all(dir)?;

    for entry in fs::read_dir(dir)? {
        if let Some(serie) = Serie::from_file(&entry?.path()) {
            series.push(serie)
        }
    }
    Ok(series)
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    if let Some(generator) = args.completion {
        print_completions(generator, &mut Args::command());
        return Ok(())
    }

    if !args.search_online.is_empty() {
        let _ = online_tv_show(args.search_online).await;
        return Ok(());
    }

    let dir = append_home_dir(".cache/bingewatcher");
    let mut series: Vec<Serie> = read_dir_for_series(&dir)?;

    if args.only_finished {
        series.retain(|serie| serie.is_finished())
    } else if !args.finished {
        series.retain(|serie| !serie.is_finished())
    }

    let mut selected_indexes: Vec<usize> = args.indexes;
    if !args.add_online.is_empty() {
        if let Ok(serie) = request_detail(args.add_online).await {
            series.push(serie);
            selected_indexes.push(series.len() - 1);
        }
    }

    if let Some(search) = args.search {
        let mut not_found = true;
        for (index, serie) in series.iter().enumerate() {
            if serie.matches(&search) {
                selected_indexes.push(index);
                not_found = false;
            }
        }
        if not_found {
            eprintln!("ERROR: search with query \"{}\" had no results.", search);
            process::exit(1);
        }
    }

    for &index in &selected_indexes {
        let current_serie = &mut series[index];
        current_serie.watch(args.watch);
        current_serie.unwatch(args.unwatch);
        print_watched_count(args.watch, args.unwatch, &current_serie.name);

        current_serie.print(&args.print_mode);
        current_serie.write_in_dir(&dir)?;
        if args.delete || args.delete_noask {
            let mut input = String::from("y");
            if !args.delete_noask {
                print!("Do you want to delete \"{}\" [Y/n] ", current_serie.name);
                io::stdin().read_line(&mut input)?;
            }
            if input.trim() != "n" {
                let path = &dir.join(&current_serie.filename());
                fs::remove_file(path)?;
                println!("Deleted {}", path.to_str().unwrap());
            }
        }
    }

    if selected_indexes.is_empty() {
        for (index, serie) in series.iter().enumerate() {
            print!("{index}: ");
            serie.print(&args.print_mode);
        }
    }
    Ok(())
}
