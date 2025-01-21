// vim:foldmethod=marker
// imports{{{
use std::{
    io, process
};
mod utils;
mod args;
mod episodate;
mod serie;
use serie::{Serie, SeriePrint};
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
        DetailOnline => {
            let serie = episodate::request_detail(&args.detail_online);
            serie.print(&SeriePrint::Extended, None);
            process::exit(0);
        }
        ListOrManipulate => {}
    }
    Ok(())
}
