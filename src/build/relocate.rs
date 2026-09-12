//! Relocate ELF search paths to exact sibling package revisions and audit DT_NEEDED.
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

pub fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let kind = fs::symlink_metadata(&path)?.file_type();
        if kind.is_dir() {
            result.extend(files(&path)?);
        } else {
            ensure!(
                kind.is_file() || kind.is_symlink(),
                "special files are not supported in build outputs: {}",
                path.display()
            );
            result.push(path);
        }
    }
    result.sort();
    Ok(result)
}

pub fn materialize_links(root: &Path) -> Result<()> {
    let root = root.canonicalize()?;
    for path in files(&root)? {
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            let target = path.canonicalize()?;
            ensure!(
                target.starts_with(&root) && target.is_file(),
                "only in-package file symlinks may be materialized: {}",
                path.display()
            );
            let mut temporary =
                tempfile::NamedTempFile::new_in(path.parent().context("missing parent")?)?;
            std::io::copy(&mut fs::File::open(&target)?, temporary.as_file_mut())?;
            temporary
                .as_file()
                .set_permissions(fs::metadata(target)?.permissions())?;
            temporary.persist(&path)?;
        }
    }
    Ok(())
}

fn output(args: &[&str], file: &Path) -> Result<String> {
    let result = Command::new("patchelf")
        .args(args)
        .arg(file)
        .output()
        .context("patchelf is required for dependency-aware builds")?;
    ensure!(
        result.status.success(),
        "patchelf failed on {}: {}",
        file.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

pub fn patch(payload: &Path, prefix: &Path, dependencies: &[PathBuf]) -> Result<()> {
    let own_files = files(payload)?;
    let mut providers = BTreeSet::new();
    for root in std::iter::once(payload.to_path_buf()).chain(dependencies.iter().cloned()) {
        for file in files(&root)? {
            if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
                providers.insert(name.to_owned());
            }
        }
    }
    // Explicit bootstrap ABI + display/session bridge. App libraries are not on this list.
    let host = [
        "libc.so.6",
        "libm.so.6",
        "libdl.so.2",
        "libpthread.so.0",
        "librt.so.1",
        "libutil.so.1",
        "libresolv.so.2",
        "libgcc_s.so.1",
        "libstdc++.so.6",
        "ld-linux-x86-64.so.2",
        "libGL.so.1",
        "libEGL.so.1",
        "libOpenGL.so.0",
        "libX11.so.6",
        "libX11-xcb.so.1",
        "libXcursor.so.1",
        "libXrandr.so.2",
        "libXi.so.6",
        "libXinerama.so.1",
        "libXext.so.6",
        "libXrender.so.1",
        "libxcb.so.1",
        "libxkbcommon.so.0",
        "libxkbcommon-x11.so.0",
        "libwayland-client.so.0",
        "libwayland-cursor.so.0",
        "libwayland-egl.so.1",
        "libdbus-1.so.3",
        "libsystemd.so.0",
    ];
    for file in own_files {
        if !fs::metadata(&file)?.is_file() {
            continue;
        }
        let mut magic = [0u8; 18];
        if fs::File::open(&file)?.read(&mut magic)? != magic.len() || &magic[..4] != b"\x7fELF" {
            continue;
        }
        // ET_REL objects are not runtime executables/libraries.
        if magic[5] != 1 || ![2, 3].contains(&u16::from_le_bytes([magic[16], magic[17]])) {
            continue;
        }
        let bytes = fs::read(&file)?;
        let elf = goblin::elf::Elf::parse(&bytes)?;
        if let Some(interpreter) = elf.interpreter {
            ensure!(
                ["/lib64/ld-linux-x86-64.so.2", "/lib/ld-linux-x86-64.so.2"].contains(&interpreter),
                "unsupported runtime interpreter {interpreter}; use the Arch source builder"
            );
        }
        if elf.dynamic.is_none() {
            continue;
        }
        for library in elf.libraries {
            ensure!(
                providers.contains(library) || host.contains(&library),
                "undeclared runtime library {library} required by {}",
                file.display()
            );
        }
        let relative = file.strip_prefix(payload)?;
        let final_file = prefix.join(relative);
        let parent = final_file.parent().context("ELF has no parent")?;
        let mut search = Vec::new();
        for lib in
            std::iter::once(prefix.join("lib")).chain(dependencies.iter().map(|p| p.join("lib")))
        {
            let relative =
                pathdiff::diff_paths(lib, parent).context("cannot create relative runtime path")?;
            search.push(format!("$ORIGIN/{}", relative.display()));
        }
        for old in output(&["--print-rpath"], &file)?.split(':') {
            if old.starts_with("$ORIGIN") && !search.iter().any(|s| s == old) {
                search.push(old.to_owned());
            }
        }
        output(&["--force-rpath", "--set-rpath", &search.join(":")], &file)?;
    }
    Ok(())
}
