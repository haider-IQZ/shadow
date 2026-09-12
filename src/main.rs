mod archive;
mod build;
mod catalog;
mod cli;
mod dependency;
mod download;
mod installation;
mod inventory;
mod manifest;
mod recipe;
mod root;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Build { recipe, output } => recipe::build(&recipe, &output),
        Command::Plan { recipe } => build::plan(&recipe),
        Command::BuildClosure {
            recipe,
            output_dir,
            build_root,
            resume,
        } => build::build(&recipe, &output_dir, &build_root, resume),
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
                        let archive = catalog::fetch(&package, temporary.path())?;
                        root.install_named(&archive, Some(&package))
                    }
                }
                Command::Remove { name } => root.remove(&name),
                Command::List => root.list(),
                Command::Run { name, args } => root.run(&name, &args),
                Command::Build { .. } | Command::Plan { .. } | Command::BuildClosure { .. } => {
                    unreachable!()
                }
            }
        }
    }
}
