//! Fetching pinned trusted source archives and preserving upstream license notices.
use super::recipe::Recipe;
use crate::{dependency, download};
use anyhow::{Result, ensure};
use std::{fs, path::Path, process::Command};

pub fn extract(recipe: &Recipe, work: &Path, archives: &Path) -> Result<std::path::PathBuf> {
    let compressed = work.join("source.tar");
    download::fetch(
        &recipe.source.url,
        &compressed,
        1024 * 1024 * 1024,
        &format!("Source: {}", recipe.id.name),
        None,
    )?;
    ensure!(
        dependency::checksum(&compressed)?.eq_ignore_ascii_case(&recipe.source.sha256),
        "source checksum mismatch for {}",
        recipe.id.name
    );
    fs::create_dir_all(archives)?;
    preserve(
        &compressed,
        &archives.join(format!(
            "{}-{}.source.tar",
            recipe.id.name, recipe.id.version
        )),
    )?;
    let source = work.join("source");
    fs::create_dir(&source)?;
    // Build inputs are executable trusted code, pinned by SHA-256, not user package archives.
    let status = Command::new("tar")
        .args([
            "--extract",
            "--no-same-owner",
            "--no-same-permissions",
            "--strip-components=1",
            "--file",
        ])
        .arg(compressed)
        .arg("--directory")
        .arg(&source)
        .status()?;
    ensure!(status.success(), "source extraction failed");
    for (index, resource) in recipe.resources.iter().enumerate() {
        let destination = source.join(&resource.path);
        let parent = destination
            .parent()
            .ok_or_else(|| anyhow::anyhow!("resource parent missing"))?;
        fs::create_dir_all(parent)?;
        ensure!(
            parent.canonicalize()?.starts_with(source.canonicalize()?),
            "resource parent escapes source directory"
        );
        let temporary = tempfile::NamedTempFile::new_in(parent)?;
        download::fetch(
            &resource.url,
            temporary.path(),
            64 * 1024 * 1024,
            &format!("Resource: {}", resource.path),
            None,
        )?;
        ensure!(
            dependency::checksum(temporary.path())?.eq_ignore_ascii_case(&resource.sha256),
            "resource checksum mismatch"
        );
        preserve(
            temporary.path(),
            &archives.join(format!(
                "{}-{}-resource-{index}",
                recipe.id.name, recipe.id.version
            )),
        )?;
        temporary.persist_noclobber(destination)?;
    }
    Ok(source)
}

fn preserve(input: &Path, output: &Path) -> Result<()> {
    if output.try_exists()? {
        ensure!(
            dependency::checksum(input)? == dependency::checksum(output)?,
            "source archive name already has different content"
        );
    } else {
        let mut file = tempfile::NamedTempFile::new_in(
            output
                .parent()
                .ok_or_else(|| anyhow::anyhow!("missing source archive directory"))?,
        )?;
        std::io::copy(&mut fs::File::open(input)?, file.as_file_mut())?;
        file.persist_noclobber(output)?;
    }
    Ok(())
}

pub fn notices(recipe: &Recipe, source: &Path, payload: &Path) -> Result<()> {
    let directory = payload.join("share/licenses").join(&recipe.id.name);
    fs::create_dir_all(&directory)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if entry.file_type()?.is_file()
            && (name.starts_with("license")
                || name.starts_with("copying")
                || name.starts_with("copyright")
                || name.starts_with("readme"))
        {
            fs::copy(entry.path(), directory.join(entry.file_name()))?;
        }
    }
    fs::write(
        directory.join("SHADOW-SOURCE.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "url": recipe.source.url, "sha256": recipe.source.sha256,
            "recipe_sha256": dependency::checksum(&recipe.path)?,
                "resources": recipe.resources,
            "note": "Built from source by Shadow; recipe and source must accompany any binary redistribution under applicable licenses."
        }))?,
    )?;
    Ok(())
}
