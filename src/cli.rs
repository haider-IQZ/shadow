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
    /// Print dependency-first source build order without running build callbacks.
    Plan {
        recipe: PathBuf,
    },
    /// Build a pinned source closure into versioned package artifacts.
    BuildClosure {
        recipe: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
        #[arg(long)]
        build_root: PathBuf,
        /// Reuse already-built immutable revisions with matching artifact hashes and dependencies.
        #[arg(long)]
        resume: bool,
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
