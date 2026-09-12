//! Exact package identities shared by recipes, manifests, and installed dependencies.
use crate::manifest::component;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageId {
    pub name: String,
    pub version: String,
    pub revision: u32,
}

impl PackageId {
    pub fn validate(&self) -> Result<()> {
        component(&self.name)?;
        component(&self.version)?;
        ensure!(self.revision > 0, "revision must be positive");
        Ok(())
    }
    pub fn directory(&self) -> String {
        format!("{}-r{}", self.version, self.revision)
    }
    pub fn artifact(&self) -> String {
        format!("{}-{}.shadow", self.name, self.directory())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    #[serde(flatten)]
    pub id: PackageId,
    pub sha256: String,
}

impl Dependency {
    pub fn validate(&self) -> Result<()> {
        self.id.validate()?;
        ensure!(
            self.sha256.len() == 64 && self.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid dependency checksum"
        );
        Ok(())
    }
}

pub fn checksum(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    std::io::copy(&mut std::fs::File::open(path)?, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}
