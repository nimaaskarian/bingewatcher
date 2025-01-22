// vim:foldmethod=marker
// imports{{{
use std::{
    io, process
};
use clap::{CommandFactory, Parser};
use bw::{
    serie::PrintMode,
    args::{AppMode::*, Args},
    utils,
    episodate,
};
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
            let path = args.dir.join(&args.name_to_path);
            if args.name_to_path.ends_with(".bw") {
                println!("{}", path.to_str().unwrap());
            } else {
                println!("{}.bw", path.to_str().unwrap());
            }
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
