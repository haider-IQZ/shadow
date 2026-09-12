//! Parse trusted source recipes without invoking their build callbacks.
use crate::dependency::PackageId;
use anyhow::{Context, Result, ensure};
use mlua::{Function, Lua, Table};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Recipe {
    pub id: PackageId,
    pub path: PathBuf,
    pub dependencies: Vec<PathBuf>,
    pub source: Source,
    pub resources: Vec<Resource>,
    pub exports: Vec<String>,
    pub build: Function,
    pub lua: Lua,
}

#[derive(serde::Serialize)]
pub struct Resource {
    pub path: String,
    pub url: String,
    pub sha256: String,
}

pub struct Source {
    pub url: String,
    pub sha256: String,
}

pub fn load(path: &Path) -> Result<Recipe> {
    let path = path.canonicalize()?;
    let lua = Lua::new();
    let parse = || -> mlua::Result<_> {
        let table: Table = lua
            .load(fs::read_to_string(&path).map_err(mlua::Error::external)?)
            .set_name(path.to_string_lossy())
            .eval()?;
        for pair in table.clone().pairs::<String, mlua::Value>() {
            let (key, _) = pair?;
            if ![
                "name",
                "version",
                "revision",
                "dependencies",
                "source",
                "resources",
                "executables",
                "build",
            ]
            .contains(&key.as_str())
            {
                return Err(mlua::Error::external(format!(
                    "unknown source recipe field: {key}"
                )));
            }
        }
        let id = PackageId {
            name: table.get("name")?,
            version: table.get("version")?,
            revision: table.get("revision")?,
        };
        let dependencies: Vec<String> = table
            .get::<Option<Vec<String>>>("dependencies")?
            .unwrap_or_default();
        let source: Table = table.get("source")?;
        let source = Source {
            url: source.get("url")?,
            sha256: source.get("sha256")?,
        };
        let mut resources = Vec::new();
        if let Some(entries) = table.get::<Option<Table>>("resources")? {
            for entry in entries.sequence_values::<Table>() {
                let entry = entry?;
                resources.push(Resource {
                    path: entry.get("path")?,
                    url: entry.get("url")?,
                    sha256: entry.get("sha256")?,
                });
            }
        }
        Ok((
            id,
            dependencies,
            source,
            resources,
            table.get::<Vec<String>>("executables")?,
            table.get::<Function>("build")?,
        ))
    };
    let (id, deps, source, resources, exports, build) =
        parse().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    id.validate()?;
    ensure!(source.url.starts_with("https://"), "source must use HTTPS");
    ensure!(
        source.sha256.len() == 64 && source.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "source must have pinned SHA-256"
    );
    for resource in &resources {
        ensure!(
            resource.url.starts_with("https://")
                && resource.sha256.len() == 64
                && resource.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "resource must use HTTPS and pinned SHA-256"
        );
        ensure!(
            !resource.path.is_empty()
                && Path::new(&resource.path)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "resource path must be source-relative"
        );
    }
    for name in &exports {
        crate::manifest::component(name)?;
    }
    let parent = path.parent().context("recipe has no parent")?;
    let dependencies = deps
        .into_iter()
        .map(|dep| {
            let relative = Path::new(&dep);
            ensure!(
                relative
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
                "dependency recipes must use relative paths without traversal"
            );
            Ok(parent.join(relative))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Recipe {
        id,
        path,
        dependencies,
        source,
        resources,
        exports,
        build,
        lua,
    })
}
