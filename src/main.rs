// vim:foldmethod=marker
// imports{{{
use std::{
    io, process, mem
};
use clap::{CommandFactory, Parser};
use bw::{
    serie::PrintMode,
    utils,
    episodate,
    args::{Args, Commands::*, OnlineCommands::*},
};

//}}}

fn main() -> io::Result<()> {
    let mut cli = Args::parse();
    cli.execute();
    Ok(())
}

