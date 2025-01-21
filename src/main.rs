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

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut args = Args::parse();
    match args.app_mode().await {
        PrintCompletions(shell) => {
            utils::print_completions(shell, &mut Args::command());
        }
        SearchOnline => {
            let _ = episodate::search_query(args.search_online).await;
        }
        DetailOnline => {
            if let Ok(serie) = episodate::request_detail(&args.detail_online).await {
                serie.print(&SeriePrint::Extended, None);
                process::exit(0);
            }
        }
        ListOrManipulate => {}
    }
    Ok(())
}
