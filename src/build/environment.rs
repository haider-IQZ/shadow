//! Build environment for one exact dependency closure, not the entire build graph.
use anyhow::{Result, ensure};
use std::{collections::BTreeMap, path::PathBuf};

pub fn variables(prefixes: &[PathBuf]) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for path in prefixes {
        ensure!(
            !path
                .to_string_lossy()
                .chars()
                .any(|c| c.is_whitespace() || c == ':' || c == ','),
            "build paths cannot contain whitespace, commas, or colons"
        );
    }
    let paths = |suffix: &str| {
        prefixes
            .iter()
            .map(|p| p.join(suffix).to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };
    let mut path = paths("bin");
    path.push(std::env::var("PATH")?);
    env.insert("PATH".into(), path.join(":"));
    let mut pc = paths("lib/pkgconfig");
    pc.extend(paths("share/pkgconfig"));
    // The caller explicitly supplies host ABI/graphics bridge pkg-config directories.
    if let Ok(bridge) = std::env::var("SHADOW_HOST_PKGCONFIG") {
        pc.push(bridge);
    }
    env.insert("PKG_CONFIG_LIBDIR".into(), pc.join(":"));
    env.insert("PKG_CONFIG_PATH".into(), String::new());
    env.insert(
        "CMAKE_PREFIX_PATH".into(),
        prefixes
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(";"),
    );
    let includes = paths("include")
        .iter()
        .map(|p| format!("-I{p}"))
        .collect::<Vec<_>>()
        .join(" ");
    env.insert("CPPFLAGS".into(), includes.clone());
    env.insert("CFLAGS".into(), format!("-O2 -fPIC {includes}"));
    env.insert("CXXFLAGS".into(), format!("-O2 -fPIC {includes}"));
    let libs = paths("lib");
    env.insert(
        "LDFLAGS".into(),
        libs.iter()
            .map(|p| format!("-L{p} -Wl,-rpath,{p}"))
            .collect::<Vec<_>>()
            .join(" "),
    );
    env.insert("LD_LIBRARY_PATH".into(), libs.join(":"));
    Ok(env)
}
