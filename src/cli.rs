use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Experimental native package manager with Lua recipes"
)]
pub struct Cli {
    /// Dedicated, user-owned Shadow prefix. Never use a host-managed directory.
    #[arg(long, global = true, default_value = ".shadow-dev")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Execute a TRUSTED local Lua recipe (not sandboxed).
    Build {
        recipe: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Install {
        package: PathBuf,
    },
    Remove {
        name: String,
    },
    List,
    /// Run an installed executable; arguments follow --.
    Run {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
}
