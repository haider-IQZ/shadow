//! Lua build callback execution for an already-resolved dependency closure.
use super::{environment, recipe::Recipe};
use crate::dependency::Dependency;
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct Paths<'a> {
    pub prefix: &'a Path,
    pub staging: &'a Path,
    pub output: &'a Path,
    pub source: &'a Path,
    pub source_archives: &'a Path,
}

pub fn run(
    recipe: &Recipe,
    paths: Paths<'_>,
    dependencies: &[Dependency],
    prefixes: &[PathBuf],
) -> Result<()> {
    let variables = environment::variables(prefixes)?;
    let context = recipe.lua.create_table().map_err(lua_error)?;
    for (key, path) in [
        ("prefix", paths.prefix),
        ("destdir", paths.staging),
        ("output", paths.output),
        ("source_archives", paths.source_archives),
    ] {
        context
            .set(key, path.to_string_lossy().to_string())
            .map_err(lua_error)?;
    }
    context
        .set("jobs", std::thread::available_parallelism()?.get().min(8))
        .map_err(lua_error)?;
    let deps = recipe.lua.create_table().map_err(lua_error)?;
    let relative = recipe.lua.create_table().map_err(lua_error)?;
    for (dep, prefix) in dependencies.iter().zip(prefixes) {
        deps.set(dep.id.name.clone(), prefix.to_string_lossy().to_string())
            .map_err(lua_error)?;
        let runtime = pathdiff::diff_paths(prefix, paths.prefix)
            .context("cannot calculate dependency runtime path")?;
        relative
            .set(dep.id.name.clone(), runtime.to_string_lossy().to_string())
            .map_err(lua_error)?;
    }
    context.set("deps", deps).map_err(lua_error)?;
    context.set("relative_deps", relative).map_err(lua_error)?;
    let recipe_dir = recipe
        .path
        .parent()
        .context("recipe parent missing")?
        .to_path_buf();
    let source = paths.source.to_path_buf();
    let run = recipe
        .lua
        .create_function(move |_, args: Vec<String>| {
            let (program, args) = args
                .split_first()
                .ok_or_else(|| mlua::Error::external("empty command"))?;
            let status = Command::new(program)
                .args(args)
                .envs(&variables)
                .env("SHADOW_RECIPE_DIR", &recipe_dir)
                .current_dir(&source)
                .status()
                .map_err(mlua::Error::external)?;
            if !status.success() {
                return Err(mlua::Error::external(format!(
                    "build command {program} failed: {status}"
                )));
            }
            Ok(())
        })
        .map_err(lua_error)?;
    context.set("run", run).map_err(lua_error)?;
    recipe.build.call::<()>(context).map_err(lua_error)?;
    Ok(())
}

fn lua_error(error: mlua::Error) -> anyhow::Error {
    anyhow::anyhow!(error.to_string())
}
