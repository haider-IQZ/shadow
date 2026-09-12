use crate::{
    archive,
    manifest::{Manifest, component},
    ui::{self, Progress},
};
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::{
        fs::{PermissionsExt, symlink},
        process::CommandExt,
    },
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

    pub fn install_named(&self, package: &Path, expected_name: Option<&str>) -> Result<()> {
        let progress = Progress::stage("Validating and installing package");
        let stage = tempfile::tempdir_in(self.path.join("work"))?;
        let manifest = archive::unpack(package, stage.path())?;
        if let Some(name) = expected_name {
            ensure!(
                manifest.name == name,
                "downloaded package name does not match request"
            );
        }
        let executable = stage.path().join("payload/bin").join(&manifest.name);
        let meta = fs::symlink_metadata(&executable)
            .context("v1 packages must provide payload/bin/<package-name>")?;
        ensure!(
            meta.is_file() && meta.permissions().mode() & 0o111 != 0,
            "package entry point must be an executable regular file"
        );
        let link = self.path.join("bin").join(&manifest.name);
        ensure!(
            fs::symlink_metadata(&link).is_err(),
            "package already active; remove it before installing another version"
        );
        let name_dir = self.path.join("Cellar").join(&manifest.name);
        fs::create_dir_all(&name_dir)?;
        ensure!(
            fs::symlink_metadata(&name_dir)?.file_type().is_dir(),
            "invalid package directory"
        );
        let destination = name_dir.join(format!("{}-r{}", manifest.version, manifest.revision));
        ensure!(
            fs::symlink_metadata(&destination).is_err(),
            "package revision already exists; use a new revision"
        );
        fs::rename(stage.path(), &destination)?;
        // This single link is the activation point; a crash before it leaves only an inactive directory.
        let target = destination.join("payload/bin").join(&manifest.name);
        symlink(&target, &link)
            .context("package staged but activation failed; inactive files retained in Cellar")?;
        drop(progress);
        ui::success(&format!(
            "Installed {} {}-r{}",
            manifest.name, manifest.version, manifest.revision
        ));
        eprintln!("  Executable: {}", link.display());
        Ok(())
    }

    fn active(&self, name: &str) -> Result<(PathBuf, Manifest)> {
        component(name)?;
        let link = self.path.join("bin").join(name);
        ensure!(
            fs::symlink_metadata(&link)?.file_type().is_symlink(),
            "invalid active entry"
        );
        let target = fs::read_link(&link)?;
        let relative = target
            .strip_prefix(self.path.join("Cellar").join(name))
            .context("active entry is outside package directory")?;
        let parts: Vec<_> = relative.components().collect();
        ensure!(
            parts.len() == 4
                && parts
                    .iter()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "invalid active path"
        );
        ensure!(
            parts[1].as_os_str() == "payload"
                && parts[2].as_os_str() == "bin"
                && parts[3].as_os_str() == name,
            "invalid entry point"
        );
        let package = self
            .path
            .join("Cellar")
            .join(name)
            .join(parts[0].as_os_str());
        ensure!(
            package.canonicalize()? == package,
            "package path contains symlinks"
        );
        let manifest: Manifest = serde_json::from_slice(&fs::read(package.join("manifest.json"))?)?;
        manifest.validate()?;
        ensure!(manifest.name == name, "package name mismatch");
        Ok((package, manifest))
    }

    pub fn list(&self) -> Result<()> {
        let mut names = fs::read_dir(self.path.join("bin"))?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<std::io::Result<Vec<_>>>()?;
        names.sort();
        for name in names {
            let (_, manifest) = self.active(name.to_str().context("non-UTF8 package name")?)?;
            println!(
                "{} {}-r{}",
                manifest.name, manifest.version, manifest.revision
            );
        }
        Ok(())
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let progress = Progress::stage(&format!("Removing {name}"));
        let (package, _) = self.active(name)?;
        fs::remove_file(self.path.join("bin").join(name))?;
        fs::remove_dir_all(&package).context("package deactivated but cleanup failed")?;
        drop(progress);
        ui::success(&format!("Removed {name}"));
        Ok(())
    }

    pub fn run(self, name: &str, args: &[String]) -> Result<()> {
        let (package, _) = self.active(name)?;
        let executable = package.join("payload/bin").join(name);
        // Do not keep the package-manager lock for the lifetime of an application.
        drop(self);
        Err(Command::new(executable).args(args).exec()).context("could not execute package")
    }
}
