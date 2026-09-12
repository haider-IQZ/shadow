use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub name: String,
    pub version: String,
    pub revision: u32,
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
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == 1,
            "unsupported package format {}",
            self.format
        );
        component(&self.name)?;
        component(&self.version)?;
        ensure!(self.revision > 0, "revision must be positive");
        Ok(())
    }
}
