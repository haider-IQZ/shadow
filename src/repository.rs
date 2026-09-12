//! HTTPS release catalog. Checksums bind artifacts to the catalog; no independent signatures yet.
use crate::{download, manifest::component, ui::Progress};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

const RELEASES: &str = "https://github.com/haider-IQZ/shadow/releases";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    format: u32,
    release: String,
    packages: BTreeMap<String, Package>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Package {
    sha256: String,
    #[serde(default)]
    bytes: Option<u64>,
}

pub fn fetch(name: &str, directory: &Path) -> Result<std::path::PathBuf> {
    component(name)?;
    ensure!(
        std::env::consts::OS == "linux" && std::env::consts::ARCH == "x86_64",
        "catalog currently supports Linux x86_64 only"
    );
    let catalog_path = directory.join("catalog.json");
    download::fetch(
        &format!("{RELEASES}/latest/download/catalog-v2.json"),
        &catalog_path,
        1024 * 1024,
        "Fetching catalog",
        None,
    )?;
    let catalog: Catalog = serde_json::from_slice(&fs::read(catalog_path)?)?;
    ensure!(catalog.format == 2, "unsupported catalog format");
    component(&catalog.release)?;
    let package = catalog.packages.get(name).with_context(|| {
        format!(
            "package '{name}' is not in the catalog; available: {}",
            catalog
                .packages
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    ensure!(
        package.sha256.len() == 64 && package.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid catalog checksum"
    );
    let output = directory.join(format!("{name}.shadow"));
    download::fetch(
        &format!("{RELEASES}/download/{}/{name}.shadow", catalog.release),
        &output,
        512 * 1024 * 1024,
        &format!("Downloading {name}"),
        package.bytes,
    )?;
    let _progress = Progress::stage("Verifying SHA-256");
    let mut file = fs::File::open(&output)?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash)?;
    ensure!(
        format!("{:x}", hash.finalize()).eq_ignore_ascii_case(&package.sha256),
        "package checksum mismatch"
    );
    Ok(output)
}
