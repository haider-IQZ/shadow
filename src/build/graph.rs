//! Deterministic dependency-first traversal with cycles and version conflicts rejected.
use super::recipe::{self, Recipe};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub fn resolve(root: &Path) -> Result<Vec<Recipe>> {
    let mut result = Vec::new();
    visit(
        root,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
        &mut result,
    )?;
    Ok(result)
}

fn visit(
    path: &Path,
    visiting: &mut BTreeSet<PathBuf>,
    visited: &mut BTreeSet<PathBuf>,
    names: &mut BTreeMap<String, PathBuf>,
    result: &mut Vec<Recipe>,
) -> Result<()> {
    let path = path.canonicalize()?;
    ensure!(
        !visiting.contains(&path),
        "dependency cycle at {}",
        path.display()
    );
    if visited.contains(&path) {
        return Ok(());
    }
    let recipe = recipe::load(&path)?;
    if let Some(other) = names.insert(recipe.id.name.clone(), path.clone()) {
        ensure!(
            other == path,
            "conflicting recipes for {} in one application closure",
            recipe.id.name
        );
    }
    visiting.insert(path.clone());
    for dependency in &recipe.dependencies {
        visit(dependency, visiting, visited, names, result)?;
    }
    visiting.remove(&path);
    visited.insert(path);
    result.push(recipe);
    Ok(())
}
