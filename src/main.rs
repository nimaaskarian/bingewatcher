mod serie;
mod onlineserie;

use scanf::scanf;
use onlineserie::{online_tv_show, request_detail};
use std::{fs::{self, remove_file}, io, path::PathBuf, process::exit};
use serie::{Serie, SeriePrint};
use clap::Parser;

use home::home_dir;

#[inline(always)]
pub fn append_home_dir(str:&str) -> PathBuf {
    PathBuf::from(format!("{}/{}", home_dir().unwrap().to_str().unwrap(), str))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Search a query among the series
    #[arg(short, long)]
    search: Option<String>,

    /// Add watched
    #[arg(short='a', long, default_value_t=0)]
    watch: usize,

    /// Remove watched
    #[arg(short='r', long, default_value_t=0)]
    unwatch: usize,

    /// Extended series view, including each episode
    #[arg(short='E', long, default_value_t=false)]
    extended: bool,

    /// Show next episode when printing
    #[arg(short='e', long, default_value_t=false)]
    episode: bool,

    /// Delete selected series
    #[arg(short, long, default_value_t=false)]
    delete: bool,

    /// Delete selected series without asking for confirmation
    #[arg(short='D', long, default_value_t=false)]
    delete_noask: bool,

    /// Print current season of selected series
    #[arg(short='S', long, default_value_t=false)]
    print_season: bool,

    /// Add an series from episodate API (needs internet)
    #[arg(short='o', long, default_value_t=String::new())]
    add_online: String,

    /// Search series from episodate API (needs internet)
    #[arg(short='O', long, default_value_t=String::new())]
    search_online: String,

    /// Show finished too
    #[arg(short, long, default_value_t=false)]
    finished: bool,

    /// Show finished too
    #[arg(short='F', long, default_value_t=false)]
    only_finished: bool,

    /// Selected indexes
    #[arg(last=true)]
    indexes: Vec<usize>,
}

#[inline]
fn read_dir_for_series(dir:&PathBuf) -> io::Result<Vec<Serie>> {
    let mut series:Vec<Serie> = vec![];

    std::fs::create_dir_all(dir)?;

    for entry in fs::read_dir(dir)? {
        match Serie::from_file(&entry?.path()) {
            Some(serie)=>series.push(serie),
            None => {},
        }
    }
    Ok(series)
}

#[inline]
fn print_watched_count(watch: usize, unwatch:usize, name:&String) {
    let watched_count: isize = watch as isize - unwatch as isize;
    match watched_count {
        ..=-1 => println!("Unwatched {} episode(s) from {name}.", -watched_count),
        1.. => println!("Watched {watched_count} episode(s) from {name}."),
        _ => {},
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    if !args.search_online.is_empty() {
        let _ = online_tv_show(args.search_online).await;
        return Ok(())
    }

    let dir = append_home_dir(".cache/bingewatcher");
    let mut series:Vec<Serie> = read_dir_for_series(&dir)?;

    if args.only_finished {
        series = series.into_iter().filter(|serie| serie.is_finished()).collect()
    }
    else if !args.finished {
        series = series.into_iter().filter(|serie| !serie.is_finished()).collect()
    }

    let mut selected_indexes: Vec<usize> = args.indexes;
    if !args.add_online.is_empty() {
        if let Some(serie) = request_detail(args.add_online).await.ok() {
            series.push(serie);
            selected_indexes.push(series.len()-1);
        }
    }

    match args.search {
        Some(search) => {
            let mut not_found = true;
            for (index, serie) in (&series).into_iter().enumerate() {
                if serie.matches(&search) {
                    selected_indexes.push(index);
                    not_found = false;
                }
            }
            if not_found {
                println!("Error: search with query \"{}\" had no results.", search);
            }
        }
        _ =>{},
    }
    let print_type =  if args.print_season {
        SeriePrint::Season
    } else if args.extended {
        SeriePrint::Extended
    } else if args.episode {
        SeriePrint::NextEpisode
    } else {
        SeriePrint::Normal
    };

    for index in &selected_indexes {
        let current_serie = &mut series[*index];
        current_serie.watch(args.watch);
        current_serie.unwatch(args.unwatch);
        print_watched_count(args.watch, args.unwatch, &current_serie.name);

        current_serie.print(&print_type);
        current_serie.write_in_dir(&dir)?;
        if args.delete || args.delete_noask {
            let mut ch = 'y';
            if !args.delete_noask {
                print!("Do you want to delete \"{}\" [Y/n] ", current_serie.name);
                let _ = scanf!("{}", ch);
            }
            if ch != 'n'{
                let path = &dir.join(&current_serie.filename());
                remove_file(path)?;
                println!("Deleted {}", path.to_str().unwrap());
            }
        }
    }

    if selected_indexes.is_empty() {
        for (index, serie) in (&series).into_iter().enumerate() {
            print!("{index}: ");
            serie.print(&print_type);
        }
    }
    Ok(())
}
