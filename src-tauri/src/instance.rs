//! Lets several Neuz windows run side by side (for example a farmer and its
//! healer), each one its own process with its own launcher, game window and
//! bot. Lock files stop two windows from using the same log file or the same
//! profile at once. The OS drops a lock when its process ends, even after a
//! crash, so a lock can't go stale.

use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

/// Locks this process holds until it exits.
static HELD: Mutex<Vec<File>> = Mutex::new(Vec::new());

/// Opens `path` so that no other process can open it while we keep the handle.
fn try_lock(path: &Path) -> Option<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options.open(path).ok()
}

fn hold(file: File) {
    if let Ok(mut held) = HELD.lock() {
        held.push(file);
    }
}

/// True when another Neuz process holds the lock at `path`.
fn is_locked(path: &Path) -> bool {
    path.exists() && try_lock(path).is_none()
}

/// Picks this window's number: 1 for the first Neuz running, 2 for the
/// second, and so on.
pub fn acquire_number(dir: &Path) -> u32 {
    for n in 1..=16 {
        if let Some(file) = try_lock(&dir.join(format!("instance_{n}.lock"))) {
            hold(file);
            return n;
        }
    }
    1
}

fn profile_lock_path(app_data_dir: &Path, profile_id: &str) -> PathBuf {
    app_data_dir.join(format!(".running_{profile_id}"))
}

/// Marks `profile_id` as running in this window. Fails when another window
/// already runs it.
pub fn claim_profile(app_data_dir: &Path, profile_id: &str) -> Result<(), String> {
    match try_lock(&profile_lock_path(app_data_dir, profile_id)) {
        Some(file) => {
            hold(file);
            Ok(())
        }
        None => Err(format!("{profile_id} is already running in another Neuz window")),
    }
}

pub fn is_profile_running(app_data_dir: &Path, profile_id: &str) -> bool {
    is_locked(&profile_lock_path(app_data_dir, profile_id))
}

/// The profile to start right away, when this window was opened with
/// `--profile NAME`.
pub fn startup_profile() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--profile" {
            return args.next().filter(|p| !p.is_empty());
        }
    }
    None
}

/// Opens another Neuz window (a new process), starting `profile_id` in it
/// when given, or showing the launcher otherwise.
pub fn spawn(profile_id: Option<&str>) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut command = Command::new(exe);
    if let Some(profile_id) = profile_id {
        command.args(["--profile", profile_id]);
    }
    command.spawn().map(|_| ()).map_err(|e| e.to_string())
}
