//! Installed-version records; no network or build behavior.
use crate::{dependency::PackageId, manifest::Manifest};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn directory(root: &Path, id: &PackageId) -> PathBuf {
    root.join("Cellar").join(&id.name).join(id.directory())
}

pub fn read(root: &Path, id: &PackageId) -> Result<Manifest> {
    id.validate()?;
    let package = directory(root, id);
    ensure!(
        package.canonicalize()? == package,
        "installed package path contains symlinks"
    );
    let manifest: Manifest = serde_json::from_slice(&fs::read(package.join("manifest.json"))?)?;
    manifest.validate()?;
    ensure!(manifest.id() == *id, "installed identity mismatch");
    Ok(manifest)
}

pub fn all(root: &Path) -> Result<Vec<Manifest>> {
    let mut manifests = Vec::new();
    for name in fs::read_dir(root.join("Cellar"))? {
        let name = name?;
        ensure!(name.file_type()?.is_dir(), "invalid Cellar entry");
        for version in fs::read_dir(name.path())? {
            let version = version?;
            ensure!(version.file_type()?.is_dir(), "invalid version entry");
            let manifest: Manifest =
                serde_json::from_slice(&fs::read(version.path().join("manifest.json"))?)?;
            ensure!(
                directory(root, &manifest.id()) == version.path(),
                "misplaced package record"
            );
            manifests.push(read(root, &manifest.id())?);
        }
    }
    manifests.sort_by_key(Manifest::id);
    Ok(manifests)
}

pub fn select(root: &Path, name: &str) -> Result<Manifest> {
    let (name, version) = name
        .split_once('@')
        .map_or((name, None), |(n, v)| (n, Some(v)));
    crate::manifest::component(name)?;
    let mut matches = all(root)?
        .into_iter()
        .filter(|m| m.name == name && version.is_none_or(|v| m.id().directory() == v));
    let selected = matches.next().context("package is not installed")?;
    ensure!(
        matches.next().is_none(),
        "multiple versions installed; specify name@version-rN"
    );
    Ok(selected)
}
