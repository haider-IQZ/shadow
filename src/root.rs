//! Prefix ownership and operation locking; package lifecycle lives in installation.rs.
use crate::{installation, inventory};
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Root {
    path: PathBuf,
    _lock: File,
}

impl Root {
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        ensure!(
            !fs::symlink_metadata(path)?.file_type().is_symlink(),
            "root must not be a symlink"
        );
        let path = path.canonicalize()?;
        ensure!(
            path != Path::new("/")
                && !["/usr", "/etc", "/bin", "/nix", "/var", "/home"]
                    .iter()
                    .any(|p| path == Path::new(p)),
            "use a dedicated Shadow root"
        );
        let marker = path.join(".shadow-root");
        if !marker.exists() {
            ensure!(
                fs::read_dir(&path)?.next().is_none(),
                "refusing nonempty directory without Shadow marker"
            );
            fs::write(&marker, "shadow-root-v1\n")?;
        }
        ensure!(
            fs::read_to_string(&marker)? == "shadow-root-v1\n",
            "unknown root format"
        );
        let lock_path = path.join(".lock");
        if let Ok(meta) = fs::symlink_metadata(&lock_path) {
            ensure!(
                meta.is_file() && !meta.file_type().is_symlink(),
                "invalid lock file"
            );
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        lock.lock_exclusive()?;
        for dir in ["Cellar", "bin", "work"] {
            let dir = path.join(dir);
            fs::create_dir_all(&dir)?;
            ensure!(
                fs::symlink_metadata(&dir)?.file_type().is_dir(),
                "internal directory must not be a symlink"
            );
        }
        Ok(Self { path, _lock: lock })
    }

    pub fn install(&self, package: &Path) -> Result<()> {
        self.install_named(package, None)
    }
    pub fn install_named(&self, package: &Path, name: Option<&str>) -> Result<()> {
        installation::install(&self.path, package, name)
    }
    pub fn remove(&self, name: &str) -> Result<()> {
        installation::remove(&self.path, name)
    }
    pub fn list(&self) -> Result<()> {
        for manifest in inventory::all(&self.path)? {
            println!("{} {}", manifest.name, manifest.id().directory());
        }
        Ok(())
    }
    pub fn run(self, name: &str, args: &[String]) -> Result<()> {
        crate::manifest::component(name)?;
        let link = self.path.join("bin").join(name);
        let executable = fs::read_link(&link)?;
        let owned = inventory::all(&self.path)?.into_iter().any(|m| {
            m.exports().iter().any(|export| export == name)
                && inventory::directory(&self.path, &m.id())
                    .join("payload/bin")
                    .join(name)
                    == executable
        });
        ensure!(owned, "executable is not owned by an installed package");
        ensure!(
            executable.canonicalize()? == executable,
            "executable path contains unexpected symlinks"
        );
        drop(self);
        Err(Command::new(executable).args(args).exec()).context("could not execute package")
    }
}
