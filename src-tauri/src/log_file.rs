//! Writes the bot's logs (and crash messages) to a file, so they can be copied
//! from the app with one click and pasted into a bug report.
//!
//! Location (Windows): %LOCALAPPDATA%\inc.madrigalstreetcartel.neuz\logs\neuz.log
//! The log from the previous run is kept as neuz.previous.log.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::OnceLock,
};

const LOG_NAME: &str = "neuz.log";
const PREVIOUS_LOG_NAME: &str = "neuz.previous.log";
/// Only the end of the file is read when copying, to keep the clipboard small.
const MAX_COPY_BYTES: u64 = 256 * 1024;

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn log_dir() -> Option<&'static PathBuf> {
    LOG_DIR.get()
}

fn log_path() -> Option<PathBuf> {
    log_dir().map(|dir| dir.join(LOG_NAME))
}

/// Create the log folder, keep the last run's log, and open a fresh log file.
pub fn open(dir: PathBuf) -> Option<File> {
    fs::create_dir_all(&dir).ok()?;
    let current = dir.join(LOG_NAME);
    if current.exists() {
        let _ = fs::rename(&current, dir.join(PREVIOUS_LOG_NAME));
    }
    let file = File::create(&current).ok()?;
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
