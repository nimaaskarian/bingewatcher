// vim:foldmethod=marker
// imports{{{
use std::{
    io, process
};
mod utils;
mod args;
mod episodate;
mod serie;
use serie::{Serie, PrintMode};
use args::{AppMode::*, Args};
use clap::{CommandFactory, Parser};
//}}}

fn main() -> io::Result<()> {
    let mut args = Args::parse();
    match args.app_mode() {
        PrintCompletions(shell) => {
            utils::print_completions(shell, &mut Args::command());
        }
        SearchOnline => {
            episodate::search_write_to_stdout(args.search_online);
        }
        PrintPath => {
            if !args.name_to_path.ends_with(".bw") {
                args.name_to_path+=".bw";
            }
            println!("{}", args.dir.join(&args.name_to_path).to_str().unwrap());
        }
        DetailOnline => {
            let serie = episodate::request_detail(&args.detail_online);
            serie.print(&PrintMode::Extended, None);
            process::exit(0);
        }
        MainDoNothing => {}
    }
    Ok(())
}
