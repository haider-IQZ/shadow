mod archive;
mod cli;
mod download;
mod manifest;
mod recipe;
mod repository;
mod root;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Build { recipe, output } => recipe::build(&recipe, &output),
        command => {
            let path = match args.root {
                Some(path) => path,
                None => std::path::PathBuf::from(
                    std::env::var_os("HOME")
                        .ok_or_else(|| anyhow::anyhow!("HOME is unset; specify --root"))?,
                )
                .join(".local/share/shadow"),
            };
            let root = root::Root::open(&path)?;
            match command {
                Command::Install { package } => {
                    if package.contains('/') || package.ends_with(".shadow") {
                        root.install(std::path::Path::new(&package))
                    } else {
                        let temporary = tempfile::tempdir()?;
                        let archive = repository::fetch(&package, temporary.path())?;
                        root.install_named(&archive, Some(&package))
                    }
                }
                Command::Remove { name } => root.remove(&name),
                Command::List => root.list(),
                Command::Run { name, args } => root.run(&name, &args),
                Command::Build { .. } => unreachable!(),
            }
        }
    }
}
