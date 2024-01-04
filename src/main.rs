mod serie;
use std::{path::PathBuf, io::{self, Write}, fs::{self, File}};
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

    #[arg(short='a', long, default_value_t=0)]
    watch: u32,

    #[arg(short='r', long, default_value_t=0)]
    unwatch: u32,

    #[arg(short='E', long, default_value_t=false)]
    extended: bool,

    #[arg(short='e', long, default_value_t=false)]
    episode: bool,

    /// Selected indexes
    #[arg(last=true)]
    indexes: Vec<usize>,
}

fn main() -> io::Result<()>{
    let args = Args::parse();
    // println!("{:?}",args.search);
    let mut series:Vec<Serie> = vec![];
    let dir = append_home_dir(".cache/bingewatcher");
    std::fs::create_dir_all(&dir)?;

    for entry in fs::read_dir(&dir)? {
        match Serie::from_file(&entry?.path()) {
            Some(serie)=>series.push(serie),
            None => {
                // println!("{:?}", path);
            },
        }
    }
    let mut selected_indexes: Vec<usize> = args.indexes;

    match args.search {
        Some(search) => {
            for (index, serie) in (&series).into_iter().enumerate() {
                if serie.matches(&search) {
                    selected_indexes.push(index);
                }
            }
        }
        _ =>{},
    }
    let print_type = if args.extended {
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

        current_serie.print(&print_type);
        current_serie.write_in_dir(&dir)?;
    }

    if selected_indexes.is_empty() {
        for (index, serie) in (&series).into_iter().enumerate() {
            // println!("{index}: {}", &serie.display());
            print!("{index}: ");
            serie.print(&print_type);
        }
    }
    Ok(())
}
