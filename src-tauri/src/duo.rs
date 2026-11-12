//! Duo link: lets a Farming copy of Neuz and a Support (healer) copy running on the
//! same PC share status.
//!
//! The healer writes a small JSON status file into the app data folder a few times
//! per second; the farmer reads it. Both copies of Neuz use the same app data folder,
//! so nothing has to be configured and no network ports are opened.

use std::{
    fs,
    path::PathBuf,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

const HEALER_FILE: &str = "duo_healer.json";
/// A status older than this means the healer copy isn't running (or is stuck).
pub const STALE_AFTER_MS: u64 = 5000;
/// How long a danger reported by the healer stays relevant.
pub const DANGER_FRESH_MS: u64 = 2000;

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Call once at startup with the app data folder.
pub fn init(dir: PathBuf) {
    let _ = DIR.set(dir);
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HealerStatus {
    /// When this status was written (unix ms)
    pub updated_ms: u64,
    pub alive: bool,
    /// Healer's own HP and MP in percent
    pub hp: u32,
    pub mp: u32,
    /// Healer has the farmer (party leader) selected
    pub has_leader: bool,
    /// Farmer is on the healer's screen and within the max follow distance
    pub leader_in_range: bool,
    /// Last time the healer saw a violet/giant nearby (unix ms, 0 = never)
    pub danger_ms: u64,
    /// The healer's HP dropped in the last couple of seconds (something is hitting it)
    #[serde(default)]
    pub under_attack: bool,
}

/// Write the healer's status for the farmer to read.
pub fn publish_healer(status: &HealerStatus) {
    let Some(dir) = DIR.get() else { return };
    let _ = fs::create_dir_all(dir);
    let tmp = dir.join(format!("{HEALER_FILE}.tmp"));
    if let Ok(json) = serde_json::to_vec(status) {
        // Write then rename, so the farmer never reads a half-written file.
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(&tmp, dir.join(HEALER_FILE));
        }
    }
}

/// The healer's latest status, if there is one.
pub fn read_healer() -> Option<HealerStatus> {
    let dir = DIR.get()?;
    let bytes = fs::read(dir.join(HEALER_FILE)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Why the farmer should hold off on pulling a new monster, or `None` to go ahead.
pub fn farmer_wait_reason(
    status: Option<&HealerStatus>,
    now_ms: u64,
    min_hp: u32,
    min_mp: u32,
) -> Option<&'static str> {
    let Some(status) = status else {
        return Some("healer not running");
    };
    // The healer stops updating while it's dead, its window is closed or the bot is stopped.
    if now_ms.saturating_sub(status.updated_ms) > STALE_AFTER_MS {
        return Some("healer not responding (dead, stopped or closed)");
    }
    if !status.alive {
        return Some("healer is dead");
    }
    if status.hp < min_hp {
        return Some("healer HP low");
    }
    if status.mp < min_mp {
        return Some("healer MP low");
    }
    if !status.has_leader || !status.leader_in_range {
        return Some("healer is too far away");
    }
    None
}

fn is_fresh(status: &HealerStatus, now_ms: u64) -> bool {
    now_ms.saturating_sub(status.updated_ms) <= STALE_AFTER_MS
}

/// The healer is running and reports that it's dead.
pub fn healer_is_dead(status: Option<&HealerStatus>, now_ms: u64) -> bool {
    status.map_or(false, |s| is_fresh(s, now_ms) && !s.alive)
}

/// The healer is running, alive and something is hitting it.
pub fn healer_needs_protection(status: Option<&HealerStatus>, now_ms: u64) -> bool {
    status.map_or(false, |s| is_fresh(s, now_ms) && s.alive && s.under_attack)
}

/// Whether the healer reported a giant/violet within the last couple of seconds.
pub fn healer_sees_danger(status: Option<&HealerStatus>, now_ms: u64) -> bool {
    status.map_or(false, |s| {
        s.danger_ms > 0 && now_ms.saturating_sub(s.danger_ms) <= DANGER_FRESH_MS
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(now: u64) -> HealerStatus {
        HealerStatus {
            updated_ms: now,
            alive: true,
            hp: 100,
            mp: 100,
            has_leader: true,
            leader_in_range: true,
            danger_ms: 0,
            under_attack: false,
        }
    }

    #[test]
    fn healer_dead_only_when_fresh() {
        let dead = HealerStatus { alive: false, ..ok(1000) };
        assert!(healer_is_dead(Some(&dead), 1000));
        assert!(!healer_is_dead(Some(&dead), 1000 + STALE_AFTER_MS + 1));
        assert!(!healer_is_dead(Some(&ok(1000)), 1000));
        assert!(!healer_is_dead(None, 1000));
    }

    #[test]
    fn protection_needed_when_alive_healer_is_hit() {
        let hit = HealerStatus { under_attack: true, ..ok(1000) };
        assert!(healer_needs_protection(Some(&hit), 1000));
        assert!(!healer_needs_protection(Some(&ok(1000)), 1000));
        assert!(!healer_needs_protection(Some(&HealerStatus { alive: false, ..hit.clone() }), 1000));
        assert!(!healer_needs_protection(Some(&hit), 1000 + STALE_AFTER_MS + 1));
    }

    #[test]
    fn old_status_files_without_under_attack_still_load() {
        let json = r#"{"updated_ms":1,"alive":true,"hp":1,"mp":1,"has_leader":true,"leader_in_range":true,"danger_ms":0}"#;
        let s: HealerStatus = serde_json::from_str(json).unwrap();
        assert!(!s.under_attack);
    }

    #[test]
    fn healthy_healer_means_go() {
        assert_eq!(farmer_wait_reason(Some(&ok(1000)), 1000, 40, 20), None);
    }

    #[test]
    fn missing_or_stale_healer_means_wait() {
        assert_eq!(farmer_wait_reason(None, 1000, 40, 20), Some("healer not running"));
        let s = ok(1000);
        assert_eq!(
            farmer_wait_reason(Some(&s), 1000 + STALE_AFTER_MS + 1, 40, 20),
            Some("healer not responding (dead, stopped or closed)")
        );
    }

    #[test]
    fn dead_low_or_far_healer_means_wait() {
        let now = 1000;
        assert_eq!(farmer_wait_reason(Some(&HealerStatus { alive: false, ..ok(now) }), now, 40, 20), Some("healer is dead"));
        assert_eq!(farmer_wait_reason(Some(&HealerStatus { hp: 39, ..ok(now) }), now, 40, 20), Some("healer HP low"));
        assert_eq!(farmer_wait_reason(Some(&HealerStatus { mp: 19, ..ok(now) }), now, 40, 20), Some("healer MP low"));
        assert_eq!(farmer_wait_reason(Some(&HealerStatus { leader_in_range: false, ..ok(now) }), now, 40, 20), Some("healer is too far away"));
        assert_eq!(farmer_wait_reason(Some(&HealerStatus { has_leader: false, ..ok(now) }), now, 40, 20), Some("healer is too far away"));
    }

    #[test]
    fn danger_is_only_fresh_for_a_short_time() {
        let s = HealerStatus { danger_ms: 5000, ..ok(5000) };
        assert!(healer_sees_danger(Some(&s), 5000 + DANGER_FRESH_MS));
        assert!(!healer_sees_danger(Some(&s), 5000 + DANGER_FRESH_MS + 1));
        assert!(!healer_sees_danger(Some(&ok(5000)), 5000));
        assert!(!healer_sees_danger(None, 5000));
    }

    #[test]
    fn publish_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("neuz-duo-test-{}", std::process::id()));
        init(dir.clone());
        let s = HealerStatus { mp: 42, ..ok(123) };
        publish_healer(&s);
        assert_eq!(read_healer(), Some(s));
        let _ = fs::remove_dir_all(dir);
    }
}
