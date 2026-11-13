//! Screenshot recorder: saves game frames plus what the bot detected in them, so
//! detection can be checked and improved offline (without the game running).
//!
//! Each recording goes to `<app data>/recordings/<date-time>/` as
//! `frame_0001.png` + `frame_0001.json`, ...

use std::{
    fs,
    path::PathBuf,
    sync::OnceLock,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use serde::Serialize;
use slog::Logger;

use crate::{
    data::{AliveState, Bounds, MobType, TargetType},
    image_analyzer::ImageAnalyzer,
    ipc::FarmingConfig,
    monster_db::monster_db,
};

/// Never record more than this many frames in one go (~1 MB each).
const MAX_FRAMES: u32 = 300;

struct Recording {
    dir: PathBuf,
    frames_left: u32,
    frame_number: u32,
    interval: Duration,
    last_frame: Option<Instant>,
}

static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();
static ACTIVE: Mutex<Option<Recording>> = Mutex::new(None);

pub fn init(app_data_dir: PathBuf) {
    let _ = BASE_DIR.set(app_data_dir.join("recordings"));
}

pub fn recordings_dir() -> Option<&'static PathBuf> {
    BASE_DIR.get()
}

/// Start recording `frames` frames, one every `interval_ms`.
/// Returns the folder the frames go to.
pub fn start(frames: u32, interval_ms: u64) -> Option<PathBuf> {
    let base = BASE_DIR.get()?;
    let name = chrono_like_now();
    let dir = base.join(name);
    fs::create_dir_all(&dir).ok()?;
    *ACTIVE.lock() = Some(Recording {
        dir: dir.clone(),
        frames_left: frames.clamp(1, MAX_FRAMES),
        frame_number: 0,
        interval: Duration::from_millis(interval_ms.max(100)),
        last_frame: None,
    });
    Some(dir)
}

/// Save one frame for debugging (e.g. a failed /position read), at most `MAX_SNAPSHOTS`
/// per run so it can't fill the disk. Returns where it went.
pub fn save_snapshot(image: &libscreenshot::ImageBuffer, name: &str) -> Option<PathBuf> {
    use std::sync::atomic::{ AtomicU32, Ordering };
    static SAVED: AtomicU32 = AtomicU32::new(0);
    const MAX_SNAPSHOTS: u32 = 3;
    if SAVED.fetch_add(1, Ordering::Relaxed) >= MAX_SNAPSHOTS {
        return None;
    }
    let dir = BASE_DIR.get()?.join("snapshots");
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{}_{}.png", name, chrono_like_now()));
    image.save(&path).ok()?;
    Some(path)
}

pub fn stop() {
    *ACTIVE.lock() = None;
}

/// Local date-time like `2026-11-13_00-31-05`, without pulling in a date crate.
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // UTC is fine here; it only needs to be unique and sortable.
    let days = secs / 86_400;
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    // Civil-from-days (Howard Hinnant's algorithm)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    format!("{y:04}-{mo:02}-{d:02}_{h:02}-{m:02}-{s:02}")
}

#[derive(Serialize)]
pub struct DetectedMob {
    pub kind: &'static str,
    pub bounds: Bounds,
    /// Name read with OCR (Windows only) and the monster list match, if any
    pub ocr: Option<String>,
    pub matched: Option<String>,
    pub rank: Option<String>,
}

#[derive(Serialize)]
pub struct FrameReport {
    pub width: u32,
    pub height: u32,
    pub alive: &'static str,
    pub hp: u32,
    pub mp: u32,
    pub fp: u32,
    pub target_hp: u32,
    pub target_mp: u32,
    pub target_on_screen: bool,
    pub target_distance: Option<i32>,
    pub mobs: Vec<DetectedMob>,
    pub danger_near: Option<DetectedMob>,
}

fn kind_name(t: TargetType) -> &'static str {
    match t {
        TargetType::Mob(MobType::Passive) => "passive",
        TargetType::Mob(MobType::Aggressive) => "aggressive",
        TargetType::Mob(MobType::Violet) => "violet",
        TargetType::Mob(MobType::Giant) => "giant",
        TargetType::TargetMarker => "target_marker",
    }
}

/// Run the bot's detection on a frame and describe what it found.
/// `read_names` runs OCR on every mob (slow, only useful for recordings).
pub fn analyze(image: &ImageAnalyzer, config: &FarmingConfig, read_names: bool) -> FrameReport {
    let stats = &image.client_stats;
    let db = monster_db();
    let describe = |mob: &crate::data::Target| {
        let ocr = if read_names { image.read_mob_name(config, mob) } else { None };
        let info = ocr.as_deref().and_then(|t| db.lookup(t));
        DetectedMob {
            kind: kind_name(mob.target_type),
            bounds: mob.bounds,
            ocr,
            matched: info.map(|i| i.name.clone()),
            rank: info.map(|i| format!("{:?}", i.rank).to_lowercase()),
        }
    };
    let mobs = image.identify_mobs(config);
    let (width, height) = image.image().map_or((0, 0), |i| (i.width(), i.height()));
    FrameReport {
        width,
        height,
        alive: match stats.is_alive {
            AliveState::Alive => "alive",
            AliveState::Dead => "dead",
            AliveState::StatsTrayClosed => "stats_tray_closed",
        },
        hp: stats.hp.value,
        mp: stats.mp.value,
        fp: stats.fp.value,
        target_hp: stats.target_hp.value,
        target_mp: stats.target_mp.value,
        target_on_screen: stats.target_on_screen,
        target_distance: stats.target_distance,
        mobs: mobs.iter().map(describe).collect(),
        danger_near: image
            .find_danger_near(config, config.flee_radius())
            .map(|(mob, _)| describe(&mob)),
    }
}

/// Called from the bot loop after every capture. Saves a frame when one is due.
pub fn on_frame(image: &ImageAnalyzer, config: &FarmingConfig, logger: &Logger) {
    let mut active = ACTIVE.lock();
    let Some(rec) = active.as_mut() else { return };
    if rec.last_frame.map_or(false, |t| t.elapsed() < rec.interval) {
        return;
    }
    let Some(frame) = image.image().cloned() else { return };
    rec.last_frame = Some(Instant::now());
    rec.frame_number += 1;
    rec.frames_left -= 1;

    let base = rec.dir.join(format!("frame_{:04}", rec.frame_number));
    let report = analyze(image, config, true);
    if let Ok(json) = serde_json::to_vec_pretty(&report) {
        let _ = fs::write(base.with_extension("json"), json);
    }
    // PNG encoding is slow-ish; don't block the bot for it.
    let png = base.with_extension("png");
    let log = logger.clone();
    std::thread::spawn(move || {
        if let Err(e) = frame.save(&png) {
            slog::warn!(log, "Could not save recorded frame"; "error" => e.to_string());
        }
    });

    if rec.frames_left == 0 {
        slog::info!(logger, "Recording finished"; "frames" => rec.frame_number, "folder" => rec.dir.display().to_string());
        *active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_format_is_sortable() {
        let s = chrono_like_now();
        assert_eq!(s.len(), 19, "{s}");
        assert!(s.starts_with("20"));
    }

    /// Re-run detection on a folder of recorded frames:
    /// `NEUZ_FRAMES=path/to/recording cargo test replay_frames -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn replay_frames() {
        let dir = std::env::var("NEUZ_FRAMES").expect("set NEUZ_FRAMES to a recording folder");
        let config = FarmingConfig::default();
        let logger = Logger::root(slog::Discard, slog::o!());
        let mut paths: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map_or(false, |x| x == "png"))
            .collect();
        paths.sort();
        for path in paths {
            let img = image::open(&path).unwrap().to_rgba8();
            let mut analyzer = ImageAnalyzer::from_image(img);
            let snapshot = analyzer.clone();
            analyzer.client_stats.update(&snapshot, &logger);
            let report = analyze(&analyzer, &config, false);
            println!(
                "{}: {} hp={} mp={} target_hp={} mobs={} danger={}",
                path.file_name().unwrap().to_string_lossy(),
                report.alive,
                report.hp,
                report.mp,
                report.target_hp,
                report
                    .mobs
                    .iter()
                    .map(|m| format!("{}@{},{}({}w)", m.kind, m.bounds.x, m.bounds.y, m.bounds.w))
                    .collect::<Vec<_>>()
                    .join(" "),
                report.danger_near.as_ref().map_or("-", |d| d.kind),
            );
        }
    }
}
