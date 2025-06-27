// vim:foldmethod=marker
// imports{{{
use std::{
    io
};
use clap::Parser;
use bw::{
    cli::{Cli, Commands::*, OnlineCommands::*},
};

//}}}

fn main() -> io::Result<()> {
    let mut cli = Cli::parse();
    cli.execute();
    Ok(())
}

