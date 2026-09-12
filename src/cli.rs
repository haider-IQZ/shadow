use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Experimental native package manager with Lua recipes"
)]
pub struct Cli {
    /// Dedicated, user-owned Shadow prefix. Never use a host-managed directory.
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,
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
    /// Install a catalog name (hello) or an explicit local path (./hello.shadow).
    Install {
        package: String,
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
