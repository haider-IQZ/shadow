//! Local dependency-closure installation and reference-aware removal.
use crate::{
    archive,
    dependency::{self, Dependency, PackageId},
    inventory,
    ui::{self, Progress},
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};

pub fn install(root: &Path, artifact: &Path, expected_name: Option<&str>) -> Result<()> {
    install_one(root, artifact, expected_name, None, &mut BTreeSet::new())
}

fn install_one(
    root: &Path,
    artifact: &Path,
    expected_name: Option<&str>,
    expected: Option<&Dependency>,
    visiting: &mut BTreeSet<PackageId>,
) -> Result<()> {
    ensure!(visiting.len() < 128, "package dependency graph is too deep");
    let progress = Progress::stage("Validating package and exact dependencies");
    let checksum = dependency::checksum(artifact)?;
    if let Some(expected) = expected {
        ensure!(
            checksum.eq_ignore_ascii_case(&expected.sha256),
            "dependency artifact checksum mismatch"
        );
    }
    let stage = tempfile::tempdir_in(root.join("work"))?;
    let manifest = archive::unpack(artifact, stage.path())?;
    let id = manifest.id();
    if let Some(name) = expected_name {
        ensure!(manifest.name == name, "package name does not match request");
    }
    if let Some(expected) = expected {
        ensure!(id == expected.id, "dependency artifact identity mismatch");
    }
    ensure!(
        visiting.insert(id.clone()),
        "dependency cycle in package manifests"
    );
    let destination = inventory::directory(root, &id);
    if destination.try_exists()? {
        ensure!(expected.is_some(), "package revision already installed");
        inventory::read(root, &id)?;
        ensure!(
            fs::read_to_string(destination.join(".archive-sha256"))? == checksum,
            "installed revision has different content"
        );
        visiting.remove(&id);
        return Ok(());
    }
    for name in manifest.exports() {
        let executable = stage.path().join("payload/bin").join(&name);
        let metadata = fs::symlink_metadata(&executable).context("missing exported executable")?;
        ensure!(
            metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
            "export must be executable regular file"
        );
        match fs::symlink_metadata(root.join("bin").join(&name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => anyhow::bail!(
                "executable {name} already exists; remove its owner before activating another version"
            ),
        }
    }
    drop(progress);
    for dep in &manifest.dependencies {
        let installed = inventory::directory(root, &dep.id);
        if installed.try_exists()? {
            inventory::read(root, &dep.id)?;
            ensure!(
                fs::read_to_string(installed.join(".archive-sha256"))?
                    .eq_ignore_ascii_case(&dep.sha256),
                "installed dependency content does not match {}",
                dep.id.name
            );
        } else {
            let artifact = artifact
                .parent()
                .context("artifact parent missing")?
                .join(dep.id.artifact());
            ensure!(
                artifact.is_file(),
                "missing dependency artifact {}; place exact dependency archives alongside the application archive",
                artifact.display()
            );
            install_one(root, &artifact, Some(&dep.id.name), Some(dep), visiting)?;
        }
    }
    let progress = Progress::stage(&format!("Installing {}", id.name));
    let parent = destination.parent().context("package parent missing")?;
    fs::create_dir_all(parent)?;
    ensure!(
        fs::symlink_metadata(parent)?.file_type().is_dir(),
        "invalid package directory"
    );
    fs::write(stage.path().join(".archive-sha256"), checksum)?;
    fs::rename(stage.path(), &destination)?;
    let mut activated = Vec::new();
    for name in manifest.exports() {
        let link = root.join("bin").join(&name);
        if let Err(error) = symlink(destination.join("payload/bin").join(&name), &link) {
            for link in activated {
                fs::remove_file(link).context("activation rollback failed")?;
            }
            fs::remove_dir_all(&destination).context("activation rollback cleanup failed")?;
            return Err(error)
                .context("activation failed; package rolled back (dependencies retained)");
        }
        activated.push(link);
    }
    visiting.remove(&id);
    drop(progress);
    ui::success(&format!("Installed {} {}", id.name, id.directory()));
    Ok(())
}

pub fn remove(root: &Path, selector: &str) -> Result<()> {
    let selected = inventory::select(root, selector)?;
    for other in inventory::all(root)? {
        ensure!(
            !other.dependencies.iter().any(|d| d.id == selected.id()),
            "{} {} is required by {} {}",
            selected.name,
            selected.id().directory(),
            other.name,
            other.id().directory()
        );
    }
    let progress = Progress::stage(&format!("Removing {}", selected.name));
    let directory = inventory::directory(root, &selected.id());
    // Validate ownership of every link before deleting any of them.
    for name in selected.exports() {
        ensure!(
            fs::read_link(root.join("bin").join(&name))?
                == directory.join("payload/bin").join(&name),
            "executable ownership mismatch"
        );
    }
    for name in selected.exports() {
        fs::remove_file(root.join("bin").join(name))?;
    }
    fs::remove_dir_all(directory).context("package deactivated but cleanup failed")?;
    drop(progress);
    ui::success(&format!("Removed {}", selected.name));
    Ok(())
}
