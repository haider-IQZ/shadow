//! Explicit reuse of immutable locally built revisions (not automatic recipe caching).
use crate::{
    dependency::{self, Dependency, PackageId},
    inventory,
};
use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub fn existing(
    root: &Path,
    artifacts: &Path,
    id: &PackageId,
    dependencies: &[Dependency],
    recipe: &Path,
) -> Result<Option<Dependency>> {
    let package = inventory::directory(root, id);
    if !package.try_exists()? {
        return Ok(None);
    }
    let manifest = inventory::read(root, id)?;
    let provenance = package
        .join("payload/share/licenses")
        .join(&id.name)
        .join("SHADOW-SOURCE.json");
    if provenance.try_exists()? {
        let metadata: serde_json::Value = serde_json::from_slice(&fs::read(provenance)?)?;
        ensure!(
            metadata["recipe_sha256"].as_str() == Some(dependency::checksum(recipe)?.as_str()),
            "recipe changed for {}; bump its revision before reuse",
            id.name
        );
    }
    ensure!(
        serde_json::to_value(&manifest.dependencies)? == serde_json::to_value(dependencies)?,
        "dependency closure changed for {}; bump its revision and rebuild",
        id.name
    );
    let hash = dependency::checksum(&artifacts.join(id.artifact()))?;
    ensure!(
        fs::read_to_string(package.join(".archive-sha256"))? == hash,
        "reused artifact checksum changed for {}",
        id.name
    );
    Ok(Some(Dependency {
        id: id.clone(),
        sha256: hash,
    }))
}
