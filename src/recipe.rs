use crate::{archive, manifest::Manifest};
use anyhow::{Context, Result, ensure};
use mlua::{Function, Lua, Table};
use std::{fs, path::Path, process::Command};

pub fn build(recipe: &Path, output: &Path) -> Result<()> {
    let recipe = recipe.canonicalize()?;
    let lua = Lua::new();
    let table: Table = lua
        .load(fs::read_to_string(&recipe)?)
        .set_name(recipe.to_string_lossy())
        .eval()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for pair in table.clone().pairs::<String, mlua::Value>() {
        let (key, _) = pair.map_err(|e| anyhow::anyhow!(e.to_string()))?;
        ensure!(
            ["name", "version", "revision", "build"].contains(&key.as_str()),
            "unsupported recipe field: {key}"
        );
    }
    let get = || -> mlua::Result<Manifest> {
        Ok(Manifest {
            format: 1,
            name: table.get("name")?,
            version: table.get("version")?,
            revision: table.get("revision")?,
        })
    };
    let manifest = get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    manifest.validate()?;
    let stage = tempfile::tempdir()?;
    let payload = stage.path().join("payload");
    fs::create_dir(&payload)?;
    let work = tempfile::tempdir()?;
    let ctx = lua
        .create_table()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    ctx.set("destdir", payload.to_string_lossy().to_string())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let source = recipe
        .parent()
        .context("recipe has no parent")?
        .to_path_buf();
    let cwd = work.path().to_path_buf();
    let run = lua
        .create_function(move |_, args: Vec<String>| {
            let (program, args) = args
                .split_first()
                .ok_or_else(|| mlua::Error::external("empty command"))?;
            let status = Command::new(program)
                .args(args)
                .current_dir(&cwd)
                .env("SHADOW_RECIPE_DIR", &source)
                .status()
                .map_err(mlua::Error::external)?;
            if !status.success() {
                return Err(mlua::Error::external(format!(
                    "command failed: {program}: {status}"
                )));
            }
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    ctx.set("run", run)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let build: Function = table
        .get("build")
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    build
        .call::<()>(ctx)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    fs::write(
        stage.path().join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    // Validate the exact artifact before publishing it.
    let temporary = tempfile::tempdir()?;
    let candidate = temporary.path().join("candidate.shadow");
    archive::pack(stage.path(), &candidate)?;
    let verify = tempfile::tempdir()?;
    archive::unpack(&candidate, verify.path())?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut published = tempfile::NamedTempFile::new_in(parent)?;
    std::io::copy(&mut fs::File::open(&candidate)?, published.as_file_mut())?;
    published.as_file().sync_all()?;
    published
        .persist_noclobber(output)
        .context("output already exists or cannot be published")?;
    println!("Built {}", output.display());
    Ok(())
}
