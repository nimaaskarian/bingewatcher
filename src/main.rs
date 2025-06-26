// vim:foldmethod=marker
// imports{{{
use std::{
    io
};
use clap::{CommandFactory, Parser};
use bw::{
    args::{Args, Commands::*, OnlineCommands::*},
};

//}}}

fn main() -> io::Result<()> {
    let mut cli = Args::parse();
    cli.execute();
    Ok(())
}

