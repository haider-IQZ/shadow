//! HTTPS release catalog. Checksums bind artifacts to the catalog; no independent signatures yet.
use crate::manifest::component;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

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
}

fn download(url: &str, output: &Path, limit: u64) -> Result<()> {
    let status = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--connect-timeout",
            "15",
            "--max-time",
            "300",
            "--retry",
            "2",
            "--max-filesize",
        ])
        .arg(limit.to_string())
        .arg("--output")
        .arg(output)
        .arg(url)
        .status()
        .context("curl is required to download packages")?;
    ensure!(status.success(), "download failed: {url}");
    ensure!(
        fs::metadata(output)?.len() <= limit,
        "download exceeds size limit"
    );
    Ok(())
}

pub fn fetch(name: &str, directory: &Path) -> Result<std::path::PathBuf> {
    component(name)?;
    ensure!(
        std::env::consts::OS == "linux" && std::env::consts::ARCH == "x86_64",
        "catalog currently supports Linux x86_64 only"
    );
    let catalog_path = directory.join("catalog.json");
    download(
        &format!("{RELEASES}/latest/download/catalog.json"),
        &catalog_path,
        1024 * 1024,
    )?;
    let catalog: Catalog = serde_json::from_slice(&fs::read(catalog_path)?)?;
    ensure!(catalog.format == 1, "unsupported catalog format");
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
    download(
        &format!("{RELEASES}/download/{}/{name}.shadow", catalog.release),
        &output,
        512 * 1024 * 1024,
    )?;
    let mut file = fs::File::open(&output)?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash)?;
    ensure!(
        format!("{:x}", hash.finalize()).eq_ignore_ascii_case(&package.sha256),
        "package checksum mismatch"
    );
    Ok(output)
}
