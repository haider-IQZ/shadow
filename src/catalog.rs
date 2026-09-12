//! Public repository metadata: aliases select immutable artifacts, dependencies pin their hashes.
use crate::{
    dependency::{Dependency, checksum},
    download,
    manifest::component,
};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const RELEASES: &str = "https://github.com/haider-IQZ/shadow/releases";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    format: u32,
    release: String,
    packages: BTreeMap<String, String>,
    artifacts: BTreeMap<String, Artifact>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    sha256: String,
    bytes: u64,
    dependencies: Vec<Dependency>,
}

pub fn fetch(name: &str, directory: &Path) -> Result<PathBuf> {
    component(name)?;
    ensure!(
        std::env::consts::OS == "linux" && std::env::consts::ARCH == "x86_64",
        "catalog supports Linux x86_64 only"
    );
    let path = directory.join("catalog.json");
    download::fetch(
        "https://raw.githubusercontent.com/haider-IQZ/shadow/main/repository/catalog-v3.json",
        &path,
        1024 * 1024,
        "Fetching catalog",
        None,
    )?;
    let catalog: Catalog = serde_json::from_slice(&fs::read(path)?)?;
    ensure!(catalog.format == 3, "unsupported catalog format");
    component(&catalog.release)?;
    let artifact = catalog.packages.get(name).with_context(|| {
        format!(
            "unknown package {name}; available: {}",
            catalog
                .packages
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    fetch_artifact(
        &catalog,
        artifact,
        None,
        directory,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
    )?;
    Ok(directory.join(artifact))
}

fn fetch_artifact(
    catalog: &Catalog,
    filename: &str,
    expected: Option<&str>,
    directory: &Path,
    visiting: &mut BTreeSet<String>,
    complete: &mut BTreeSet<String>,
) -> Result<()> {
    ensure!(
        !filename.is_empty()
            && filename.len() <= 255
            && !filename.starts_with('.')
            && filename
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b)),
        "invalid artifact filename"
    );
    ensure!(filename.ends_with(".shadow"), "invalid artifact filename");
    let record = catalog
        .artifacts
        .get(filename)
        .context("dependency missing from repository catalog")?;
    ensure!(
        record.sha256.len() == 64 && record.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid checksum"
    );
    if let Some(expected) = expected {
        ensure!(
            record.sha256.eq_ignore_ascii_case(expected),
            "repository dependency revision changed content"
        );
    }
    if complete.contains(filename) {
        return Ok(());
    }
    ensure!(
        visiting.len() < 128 && complete.len() < 512,
        "repository dependency graph exceeds limits"
    );
    ensure!(
        visiting.insert(filename.to_owned()),
        "cycle in repository dependencies"
    );
    for dependency in &record.dependencies {
        dependency.validate()?;
        fetch_artifact(
            catalog,
            &dependency.id.artifact(),
            Some(&dependency.sha256),
            directory,
            visiting,
            complete,
        )?;
    }
    let path = directory.join(filename);
    download::fetch(
        &format!("{RELEASES}/download/{}/{filename}", catalog.release),
        &path,
        512 * 1024 * 1024,
        &format!("Downloading {filename}"),
        Some(record.bytes),
    )?;
    let _progress = crate::ui::Progress::stage("Verifying SHA-256");
    ensure!(
        checksum(&path)?.eq_ignore_ascii_case(&record.sha256),
        "downloaded artifact checksum mismatch"
    );
    visiting.remove(filename);
    complete.insert(filename.to_owned());
    Ok(())
}
