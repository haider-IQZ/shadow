mod archive;
mod cli;
mod manifest;
mod recipe;
mod root;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Build { recipe, output } => recipe::build(&recipe, &output),
        command => {
            let root = root::Root::open(&args.root)?;
            match command {
                Command::Install { package } => root.install(&package),
                Command::Remove { name } => root.remove(&name),
                Command::List => root.list(),
                Command::Run { name, args } => root.run(&name, &args),
                Command::Build { .. } => unreachable!(),
            }
        }
    }
}
