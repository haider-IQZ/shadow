use crate::dependency::{Dependency, PackageId};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub name: String,
    pub version: String,
    pub revision: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,
    /// None is the legacy single executable; Some([]) represents a library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executables: Option<Vec<String>>,
}

pub fn component(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 100,
        "invalid component length"
    );
    ensure!(
        value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._+-".contains(&c)),
        "invalid path component: {value}"
    );
    ensure!(
        value != "." && value != ".." && !value.starts_with('.'),
        "invalid component: {value}"
    );
    Ok(())
}

impl Manifest {
    pub fn id(&self) -> PackageId {
        PackageId {
            name: self.name.clone(),
            version: self.version.clone(),
            revision: self.revision,
        }
    }

    pub fn exports(&self) -> Vec<String> {
        self.executables
            .clone()
            .unwrap_or_else(|| vec![self.name.clone()])
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == 1 || self.format == 2,
            "unsupported package format {}",
            self.format
        );
        component(&self.name)?;
        component(&self.version)?;
        ensure!(self.revision > 0, "revision must be positive");
        ensure!(
            self.format == 2 || (self.dependencies.is_empty() && self.executables.is_none()),
            "format 1 cannot declare dependencies or exports"
        );
        let mut dependencies = std::collections::BTreeSet::new();
        for dependency in &self.dependencies {
            dependency.validate()?;
            ensure!(
                dependency.id != self.id(),
                "package cannot depend on itself"
            );
            ensure!(
                dependencies.insert(&dependency.id.name),
                "duplicate dependency name"
            );
        }
        let mut exports = std::collections::BTreeSet::new();
        for name in self.exports() {
            component(&name)?;
            ensure!(exports.insert(name), "duplicate executable export");
        }
        Ok(())
    }
}
