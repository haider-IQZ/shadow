//! Terminal presentation only; plain stderr output for redirected commands.
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::{
    io::{self, IsTerminal},
    time::Duration,
};

fn interactive() -> bool {
    io::stderr().is_terminal() && std::env::var("TERM").as_deref() != Ok("dumb")
}

fn color() -> bool {
    interactive() && std::env::var_os("NO_COLOR").is_none()
}

pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub fn stage(message: &str) -> Self {
        Self::new(message, None, false)
    }

    pub fn download(message: &str, bytes: Option<u64>) -> Self {
        Self::new(message, bytes, true)
    }

    fn new(message: &str, bytes: Option<u64>, download: bool) -> Self {
        let bar = match bytes {
            Some(length) => ProgressBar::new(length),
            None => ProgressBar::new_spinner(),
        };
        if interactive() {
            let template = match (download, bytes) {
                (true, Some(_)) => {
                    "  {spinner} {msg} [{bar:24}] {bytes}/{total_bytes} {bytes_per_sec}"
                }
                (true, None) => "  {spinner} {msg} {bytes} {bytes_per_sec}",
                _ => "  {spinner} {msg}",
            };
            // Templates are compile-time constants, not user input.
            let style = ProgressStyle::with_template(template)
                .expect("valid progress template")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .progress_chars("━╸─");
            bar.set_style(style);
            bar.set_message(message.to_owned());
            bar.enable_steady_tick(Duration::from_millis(90));
        } else {
            bar.set_draw_target(ProgressDrawTarget::hidden());
            eprintln!("{message}");
        }
        Self { bar }
    }

    pub fn position(&self, bytes: u64) {
        self.bar.set_position(bytes);
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

pub fn success(message: &str) {
    if color() {
        eprintln!("\x1b[32m✓\x1b[0m {message}");
    } else {
        eprintln!("OK {message}");
    }
}
