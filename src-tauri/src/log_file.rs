//! Writes the bot's logs (and crash messages) to a file, so they can be copied
//! from the app with one click and pasted into a bug report.
//!
//! Location (Windows): %LOCALAPPDATA%\inc.madrigalstreetcartel.neuz\logs\neuz.log
//! The log from the previous run is kept as neuz.previous.log.
//! A second Neuz window running at the same time writes neuz.2.log, and so on.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::OnceLock,
};

/// Only the end of the file is read when copying, to keep the clipboard small.
const MAX_COPY_BYTES: u64 = 256 * 1024;

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
static LOG_NAME: OnceLock<String> = OnceLock::new();

/// neuz.log for the first window, neuz.2.log for the second, ...
fn log_name(instance: u32, previous: bool) -> String {
    let number = if instance > 1 { format!(".{instance}") } else { String::new() };
    let previous = if previous { ".previous" } else { "" };
    format!("neuz{number}{previous}.log")
}

pub fn log_dir() -> Option<&'static PathBuf> {
    LOG_DIR.get()
}

fn log_path() -> Option<PathBuf> {
    Some(log_dir()?.join(LOG_NAME.get()?))
}

/// Create the log folder, keep the last run's log, and open a fresh log file.
/// `instance` is this window's number (see instance.rs), so two Neuz windows
/// running at once don't write to the same file.
pub fn open(dir: PathBuf, instance: u32) -> Option<File> {
    fs::create_dir_all(&dir).ok()?;
    let current = dir.join(log_name(instance, false));
    if current.exists() {
        let _ = fs::rename(&current, dir.join(log_name(instance, true)));
    }
    let file = File::create(&current).ok()?;
    let _ = LOG_NAME.set(log_name(instance, false));
    let _ = LOG_DIR.set(dir);
    Some(file)
}

/// Record panics (crashes) in the log file too, then run the previous handler.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(path) = log_path() {
            if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
                let backtrace = std::backtrace::Backtrace::force_capture();
                let _ = writeln!(file, "\nCRASH: {info}\n{backtrace}");
            }
        }
        previous(info);
    }));
}

/// The last `max_lines` lines of the current log, plus a short header with
/// version and OS, ready to paste.
pub fn tail(max_lines: usize, version: &str) -> String {
    let header = format!(
        "Neuz v{version} | {} {} | log: {}\n----\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        log_path().map_or("unavailable".into(), |p| p.display().to_string()),
    );
    let Some(path) = log_path() else {
        return header + "(no log file)";
    };
    let mut content = String::new();
    if let Ok(mut file) = File::open(&path) {
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = file.seek(SeekFrom::Start(len.saturating_sub(MAX_COPY_BYTES)));
        let mut bytes = Vec::new();
        let _ = file.read_to_end(&mut bytes);
        content = String::from_utf8_lossy(&bytes).into_owned();
    }
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    header + &lines[start..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::log_name;

    #[test]
    fn first_window_keeps_the_old_log_names() {
        assert_eq!(log_name(1, false), "neuz.log");
        assert_eq!(log_name(1, true), "neuz.previous.log");
        assert_eq!(log_name(2, false), "neuz.2.log");
        assert_eq!(log_name(2, true), "neuz.2.previous.log");
    }
}
