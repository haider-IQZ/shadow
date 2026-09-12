use std::{fs, process::Command};

fn shadow(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shadow"))
        .args(["--root", ".shadow-dev"])
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

#[test]
fn package_lifecycle_and_conflicts() {
    let temp = tempfile::tempdir().unwrap();
    let recipe = format!("{}/recipes/hello.lua", env!("CARGO_MANIFEST_DIR"));
    let run = |args: &[&str]| {
        let output = shadow(args, temp.path());
        assert!(
            !output.stderr.contains(&0x1b),
            "redirected output must not contain ANSI escapes"
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    run(&["build", &recipe, "-o", "hello.shadow"]);
    run(&["install", "hello.shadow"]);
    assert!(
        !shadow(&["install", "hello.shadow"], temp.path())
            .status
            .success()
    );
    assert!(String::from_utf8_lossy(&run(&["run", "hello"]).stdout).contains("Hello from Shadow"));
    assert!(String::from_utf8_lossy(&run(&["list"]).stdout).contains("hello 1.0.0-r1"));
    assert!(
        !shadow(&["remove", "../outside"], temp.path())
            .status
            .success()
    );
    run(&["remove", "hello"]);
    assert!(run(&["list"]).stdout.is_empty());
    assert!(!temp.path().join(".shadow-dev/bin/hello").exists());
    run(&["install", "hello.shadow"]);
}

#[test]
fn rejects_archive_links_and_preserves_host_files() {
    let temp = tempfile::tempdir().unwrap();
    let victim = temp.path().join("victim");
    fs::write(&victim, "untouched").unwrap();
    let file = fs::File::create(temp.path().join("bad.shadow")).unwrap();
    let mut tar = tar::Builder::new(zstd::Encoder::new(file, 1).unwrap());
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_mode(0o777);
    tar.append_link(&mut header, "payload/escape", &victim)
        .unwrap();
    tar.into_inner().unwrap().finish().unwrap();
    assert!(
        !shadow(&["install", "bad.shadow"], temp.path())
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(victim).unwrap(), "untouched");
    assert!(
        fs::read_dir(temp.path().join(".shadow-dev/bin"))
            .unwrap()
            .next()
            .is_none()
    );
}
