use crate::manifest::Manifest;
use anyhow::{Context, Result, ensure};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path},
};

const MAX_BYTES: u64 = 512 * 1024 * 1024;

pub fn pack(stage: &Path, output: &Path) -> Result<()> {
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    {
        let encoder = zstd::Encoder::new(tmp.as_file_mut(), 3)?;
        let mut archive = tar::Builder::new(encoder);
        archive.follow_symlinks(false);
        archive.append_path_with_name(stage.join("manifest.json"), "manifest.json")?;
        archive.append_dir_all("payload", stage.join("payload"))?;
        archive.into_inner()?.finish()?;
    }
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist_noclobber(output)
        .context("output already exists or cannot be published")?;
    Ok(())
}

/// Only regular files/directories are supported in format v1. Never unpack links.
pub fn unpack(package: &Path, stage: &Path) -> Result<Manifest> {
    let decoder = zstd::Decoder::new(File::open(package)?)?;
    let mut archive = tar::Archive::new(decoder);
    let mut paths = HashSet::new();
    let mut total = 0u64;
    for entry in archive.entries()? {
        let mut entry = entry?;
        ensure!(paths.len() < 20_000, "package has too many entries");
        let path = entry.path()?.into_owned();
        ensure!(
            path.components().all(|c| matches!(c, Component::Normal(_))),
            "unsafe archive path"
        );
        ensure!(
            path == Path::new("manifest.json") || path.starts_with("payload"),
            "unexpected archive path: {}",
            path.display()
        );
        ensure!(paths.insert(path.clone()), "duplicate archive path");
        let kind = entry.header().entry_type();
        ensure!(
            kind.is_file() || kind.is_dir(),
            "links and special files are not supported"
        );
        total = total
            .checked_add(entry.size())
            .context("package size overflow")?;
        ensure!(
            total <= MAX_BYTES,
            "package exceeds 512 MiB extracted limit"
        );
        ensure!(
            entry.header().mode()? & 0o7000 == 0,
            "special permission bits are forbidden"
        );
        ensure!(entry.unpack_in(stage)?, "entry escaped staging directory");
    }
    let mut json = String::new();
    File::open(stage.join("manifest.json"))?
        .take(65537)
        .read_to_string(&mut json)?;
    ensure!(json.len() <= 65536, "manifest too large");
    let manifest: Manifest = serde_json::from_str(&json)?;
    manifest.validate()?;
    ensure!(
        fs::metadata(stage.join("payload"))?.is_dir(),
        "missing payload directory"
    );
    Ok(manifest)
}
