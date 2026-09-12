//! Bounded curl transport with observable file progress and child cleanup.
use crate::ui::Progress;
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    io::{Read, Seek},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

struct Transfer(Child);

impl Drop for Transfer {
    fn drop(&mut self) {
        // Also reap the child when polling, filesystem checks, or output handling fail.
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

pub fn fetch(url: &str, output: &Path, limit: u64, label: &str, bytes: Option<u64>) -> Result<()> {
    let progress = Progress::download(label, bytes);
    let mut errors = tempfile::tempfile()?;
    let mut transfer = Transfer(
        Command::new("curl")
            .args([
                "--fail",
                "--location",
                "--silent",
                "--show-error",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--connect-timeout",
                "15",
                "--max-time",
                "300",
                "--retry",
                "2",
                "--max-filesize",
            ])
            .arg(limit.to_string())
            .arg("--output")
            .arg(output)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(errors.try_clone()?)
            .spawn()
            .context("curl is required to download packages")?,
    );
    let status = loop {
        if let Some(status) = transfer.0.try_wait()? {
            break status;
        }
        match fs::metadata(output) {
            Ok(metadata) => {
                ensure!(metadata.len() <= limit, "download exceeds size limit");
                progress.position(metadata.len());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        thread::sleep(Duration::from_millis(80));
    };
    errors.rewind()?;
    let mut message = String::new();
    errors.take(8192).read_to_string(&mut message)?;
    ensure!(
        status.success(),
        "download failed: {url}: {}",
        message.trim()
    );
    let length = fs::metadata(output)?.len();
    ensure!(length <= limit, "download exceeds size limit");
    if let Some(expected) = bytes {
        ensure!(length == expected, "downloaded size does not match catalog");
    }
    progress.position(length);
    Ok(())
}
