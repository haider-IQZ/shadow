//! Orchestrate dependency-first builds; execution, source handling, and ELF work are separate modules.
mod environment;
mod execute;
mod graph;
mod recipe;
mod relocate;
mod reuse;
mod source;

use crate::{
    archive,
    dependency::{self, Dependency},
    manifest::Manifest,
};
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn plan(recipe: &Path) -> Result<()> {
    for recipe in graph::resolve(recipe)? {
        println!("{} {}", recipe.id.name, recipe.id.directory());
    }
    Ok(())
}

pub fn build(recipe: &Path, directory: &Path, build_root: &Path, resume: bool) -> Result<()> {
    let recipes = graph::resolve(recipe)?;
    fs::create_dir_all(directory)?;
    let directory = directory.canonicalize()?;
    fs::create_dir_all(build_root)?;
    let build_root = build_root.canonicalize()?;
    ensure!(
        resume || fs::read_dir(&build_root)?.next().is_none(),
        "source build root must be empty (or explicitly --resume immutable revisions)"
    );
    let lock_path = build_root.join(".build.lock");
    if let Ok(metadata) = fs::symlink_metadata(&lock_path) {
        ensure!(metadata.file_type().is_file(), "invalid build lock file");
    }
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.lock_exclusive()?;
    let mut built: BTreeMap<PathBuf, (Dependency, Vec<Dependency>)> = BTreeMap::new();
    for recipe in recipes {
        let mut deps = BTreeMap::new();
        for path in &recipe.dependencies {
            let (dependency, transitive) = built
                .get(&path.canonicalize()?)
                .context("dependency was not built")?;
            deps.insert(dependency.id.name.clone(), dependency.clone());
            for dep in transitive {
                deps.insert(dep.id.name.clone(), dep.clone());
            }
        }
        let dependencies: Vec<_> = deps.into_values().collect();
        if resume
            && let Some(existing) = reuse::existing(
                &build_root,
                &directory,
                &recipe.id,
                &dependencies,
                &recipe.path,
            )?
        {
            println!(
                "Reusing immutable revision {} {}",
                recipe.id.name,
                recipe.id.directory()
            );
            built.insert(recipe.path, (existing, dependencies));
            continue;
        }
        let prefixes: Vec<_> = dependencies
            .iter()
            .map(|d| {
                build_root
                    .join("Cellar")
                    .join(&d.id.name)
                    .join(d.id.directory())
                    .join("payload")
            })
            .collect();
        let prefix = build_root
            .join("Cellar")
            .join(&recipe.id.name)
            .join(recipe.id.directory())
            .join("payload");
        environment::variables(std::slice::from_ref(&prefix))?;
        let source_work = tempfile::tempdir_in(&build_root)?;
        let source_dir = source::extract(&recipe, source_work.path(), &directory.join("sources"))?;
        let staging = tempfile::tempdir_in(&build_root)?;
        let output = staging.path().join(prefix.strip_prefix("/")?);
        fs::create_dir_all(&output)?;
        let result = (|| -> Result<()> {
            execute::run(
                &recipe,
                execute::Paths {
                    prefix: &prefix,
                    staging: staging.path(),
                    output: &output,
                    source: &source_dir,
                    source_archives: &directory.join("sources"),
                },
                &dependencies,
                &prefixes,
            )?;
            source::notices(&recipe, &source_dir, &output)?;
            relocate::materialize_links(&output)?;
            relocate::patch(&output, &prefix, &prefixes)?;
            Ok(())
        })();
        if let Err(error) = result {
            let source = source_work.keep();
            let staging = staging.keep();
            return Err(error).context(format!(
                "build failed; source retained at {}, staging at {}",
                source.display(),
                staging.display()
            ));
        }
        let package = tempfile::tempdir_in(&build_root)?;
        fs::rename(output, package.path().join("payload"))?;
        let manifest = Manifest {
            format: 2,
            name: recipe.id.name.clone(),
            version: recipe.id.version.clone(),
            revision: recipe.id.revision,
            dependencies: dependencies.clone(),
            executables: Some(recipe.exports),
        };
        manifest.validate()?;
        fs::write(
            package.path().join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        let temporary = tempfile::tempdir_in(&directory)?;
        let candidate = temporary.path().join("candidate.shadow");
        archive::pack(package.path(), &candidate)?;
        let verify = tempfile::tempdir_in(&build_root)?;
        archive::unpack(&candidate, verify.path())?;
        let checksum = dependency::checksum(&candidate)?;
        let artifact = directory.join(recipe.id.artifact());
        let mut published = tempfile::NamedTempFile::new_in(&directory)?;
        std::io::copy(&mut fs::File::open(candidate)?, published.as_file_mut())?;
        published.as_file().sync_all()?;
        published
            .persist_noclobber(&artifact)
            .context("artifact already exists; bump revision or use a fresh output directory")?;
        fs::write(package.path().join(".archive-sha256"), &checksum)?;
        let installed = prefix.parent().context("missing package path")?;
        fs::create_dir_all(installed.parent().context("missing name path")?)?;
        fs::rename(package.path(), installed)?;
        built.insert(
            recipe.path,
            (
                Dependency {
                    id: recipe.id,
                    sha256: checksum,
                },
                dependencies,
            ),
        );
        println!("Built {}", artifact.display());
    }
    Ok(())
}
