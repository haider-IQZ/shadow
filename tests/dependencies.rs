use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_shadow"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}
fn ok(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn package(
    directory: &Path,
    name: &str,
    version: &str,
    manifest: serde_json::Value,
    payload: &Path,
) -> (String, String) {
    let path = directory.join(format!("{name}-{version}-r1.shadow"));
    let file = fs::File::create(&path).unwrap();
    let mut tar = tar::Builder::new(zstd::Encoder::new(file, 1).unwrap());
    let bytes = serde_json::to_vec(&manifest).unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, "manifest.json", bytes.as_slice())
        .unwrap();
    tar.append_dir_all("payload", payload).unwrap();
    tar.into_inner().unwrap().finish().unwrap();
    let hash = format!("{:x}", Sha256::digest(fs::read(&path).unwrap()));
    (path.to_str().unwrap().to_owned(), hash)
}

#[test]
fn exact_shared_libraries_coexist_and_cannot_be_removed_while_referenced() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("installed");
    let mut libraries = Vec::new();
    for version in ["1", "2"] {
        let payload = temp.path().join(format!("lib-{version}"));
        fs::create_dir_all(payload.join("lib")).unwrap();
        let source = temp.path().join(format!("lib-{version}.c"));
        fs::write(&source, format!("int demo(void) {{ return {version}; }}")).unwrap();
        let library = payload.join("lib/libdemo.so.1");
        ok(Command::new("cc")
            .args(["-shared", "-fPIC", "-Wl,-soname,libdemo.so.1"])
            .arg(source)
            .arg("-o")
            .arg(&library)
            .output()
            .unwrap());
        let (_, hash) = package(
            temp.path(),
            "demo",
            version,
            json!({"format":2,"name":"demo","version":version,"revision":1,"executables":[]}),
            &payload,
        );
        libraries.push((library, hash));
    }
    for (app, version) in [("first", "1"), ("second", "2"), ("shared", "1")] {
        let (library, hash) = &libraries[if version == "1" { 0 } else { 1 }];
        let payload = temp.path().join(app);
        fs::create_dir_all(payload.join("bin")).unwrap();
        let source = temp.path().join(format!("{app}.c"));
        fs::write(&source, "#include <stdio.h>\nint demo(void); int main(void) { printf(\"%d\\n\", demo()); return 0; }\n").unwrap();
        let rpath = format!(
            "-Wl,--disable-new-dtags,-rpath,$ORIGIN/../../../../demo/{version}-r1/payload/lib"
        );
        ok(Command::new("cc")
            .arg(source)
            .arg(library)
            .arg(rpath)
            .arg("-o")
            .arg(payload.join("bin").join(app))
            .output()
            .unwrap());
        let (archive, _) = package(
            temp.path(),
            app,
            "1",
            json!({"format":2,"name":app,"version":"1","revision":1,"executables":[app],"dependencies":[{"name":"demo","version":version,"revision":1,"sha256":hash}]}),
            &payload,
        );
        ok(cli(&root, &["install", &archive]));
        assert_eq!(
            ok(cli(&root, &["run", app])).stdout,
            format!("{version}\n").as_bytes()
        );
    }
    assert_eq!(
        ok(cli(&root, &["list"]))
            .stdout
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .count(),
        5
    );
    assert!(!cli(&root, &["remove", "demo@1-r1"]).status.success());
    ok(cli(&root, &["remove", "first"]));
    assert!(!cli(&root, &["remove", "demo@1-r1"]).status.success());
    ok(cli(&root, &["remove", "shared"]));
    ok(cli(&root, &["remove", "demo@1-r1"]));
    assert_eq!(ok(cli(&root, &["run", "second"])).stdout, b"2\n");
    // Relocatable package paths must survive moving the entire prefix; activation
    // links are absolute today, so invoke the actual executable after the move.
    let moved = temp.path().join("moved");
    fs::rename(&root, &moved).unwrap();
    let output = Command::new(moved.join("Cellar/second/1-r1/payload/bin/second"))
        .output()
        .unwrap();
    assert_eq!(ok(output).stdout, b"2\n");

    // A nearby archive with the right filename is not enough: bytes must match.
    fs::write(temp.path().join("demo-2-r1.shadow"), b"changed content").unwrap();
    let rejected_root = temp.path().join("rejected");
    let rejected = cli(
        &rejected_root,
        &[
            "install",
            temp.path().join("second-1-r1.shadow").to_str().unwrap(),
        ],
    );
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("checksum mismatch"));
    assert!(!rejected_root.join("Cellar/second/1-r1").exists());
}

#[test]
fn source_plan_rejects_dependency_cycles_before_building() {
    let temp = tempfile::tempdir().unwrap();
    for (name, dependency) in [("a", "b"), ("b", "a")] {
        fs::write(temp.path().join(format!("{name}.lua")), format!("return {{ name='{name}', version='1', revision=1, executables={{}}, dependencies={{'{dependency}.lua'}}, source={{url='https://example.invalid/source.tar.gz', sha256='{}'}}, build=function() error('must not execute') end }}", "0".repeat(64))).unwrap();
    }
    let output = cli(
        &temp.path().join("root"),
        &["plan", temp.path().join("a.lua").to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cycle"));
    assert!(!temp.path().join("root").exists());
}
