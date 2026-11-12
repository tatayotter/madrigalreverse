//! Monster list from the official Flyff Universe API (https://api.flyff.com).
//!
//! The list is downloaded once and cached in the app data folder, then refreshed
//! in the background when the cache is older than a week. Names read from the
//! screen (OCR) are matched against it with a typo-tolerant lookup.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use slog::Logger;
use tauri::api::http::{ClientBuilder, HttpRequestBuilder, ResponseType};

const API_BASE: &str = "https://api.flyff.com";
const CACHE_FILE: &str = "monsters.json";
const CACHE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600);
/// How many monster ids to request per API call.
const BATCH_SIZE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rank {
    Small,
    Normal,
    Captain,
    Giant,
    Violet,
    Boss,
    Material,
    Super,
    Guard,
    Citizen,
    Worldboss,
    #[serde(other)]
    Unknown,
}

impl Rank {
    /// Monsters a solo farming bot should stay away from.
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Rank::Giant | Rank::Violet | Rank::Boss | Rank::Super | Rank::Worldboss
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterInfo {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub rank: Rank,
}

/// Shape of a monster object as returned by the API (only the fields we use).
#[derive(Deserialize)]
struct ApiMonster {
    id: u32,
    name: HashMap<String, String>,
    #[serde(default)]
    level: u32,
    rank: Rank,
}

#[derive(Debug, Default)]
pub struct MonsterDb {
    monsters: Vec<MonsterInfo>,
    /// normalized name -> index into `monsters`
    by_name: HashMap<String, usize>,
}

/// Lowercase, keep letters/digits, collapse everything else into single spaces.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = true;
    for c in text.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim_end().to_string()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

impl MonsterDb {
    pub fn new(monsters: Vec<MonsterInfo>) -> Self {
        let mut by_name = HashMap::new();
        for (index, monster) in monsters.iter().enumerate() {
            // Several monsters can share a name. Keep the first one, unless a later one
            // is dangerous: when in doubt, it's safer to treat the name as a giant.
            let existing = by_name.entry(normalize(&monster.name)).or_insert(index);
            if monster.rank.is_dangerous() && !monsters[*existing].rank.is_dangerous() {
                *existing = index;
            }
        }
        Self { monsters, by_name }
    }

    pub fn len(&self) -> usize {
        self.monsters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.monsters.is_empty()
    }

    /// Find the monster whose name best matches text read from the screen.
    /// Allows roughly one OCR mistake per five characters.
    pub fn lookup(&self, ocr_text: &str) -> Option<&MonsterInfo> {
        let needle = normalize(ocr_text);
        if needle.len() < 3 {
            return None;
        }
        if let Some(&index) = self.by_name.get(&needle) {
            return Some(&self.monsters[index]);
        }
        let max_distance = (needle.chars().count() / 5).max(1);
        self.by_name
            .iter()
            .filter(|(name, _)| name.len().abs_diff(needle.len()) <= max_distance)
            .map(|(name, &index)| (levenshtein(name, &needle), index))
            .filter(|&(distance, _)| distance <= max_distance)
            .min_by_key(|&(distance, _)| distance)
            .map(|(_, index)| &self.monsters[index])
    }

    fn load_cache(path: &Path) -> Option<Vec<MonsterInfo>> {
        let file = fs::File::open(path).ok()?;
        serde_json::from_reader(file).ok()
    }

    fn cache_is_fresh(path: &Path) -> bool {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .map_or(false, |age| age < CACHE_MAX_AGE)
    }

    async fn fetch_all() -> Result<Vec<MonsterInfo>, String> {
        let client = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let get = |url: String| {
            let client = client.clone();
            async move {
                let request = HttpRequestBuilder::new("GET", url)
                    .map_err(|e| e.to_string())?
                    .timeout(Duration::from_secs(30))
                    .response_type(ResponseType::Json);
                let response = client.send(request).await.map_err(|e| e.to_string())?;
                let data = response.read().await.map_err(|e| e.to_string())?;
                Ok::<serde_json::Value, String>(data.data)
            }
        };

        let ids: Vec<u32> = serde_json::from_value(get(format!("{API_BASE}/monster")).await?)
            .map_err(|e| e.to_string())?;

        let mut monsters = Vec::with_capacity(ids.len());
        for chunk in ids.chunks(BATCH_SIZE) {
            let list = chunk.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
            let batch: Vec<ApiMonster> =
                serde_json::from_value(get(format!("{API_BASE}/monster/{list}")).await?)
                    .map_err(|e| e.to_string())?;
            monsters.extend(batch.into_iter().map(|m| MonsterInfo {
                id: m.id,
                name: m.name.get("en").cloned().unwrap_or_default(),
                level: m.level,
                rank: m.rank,
            }));
        }
        monsters.retain(|m| !m.name.is_empty());
        Ok(monsters)
    }
}

fn global() -> &'static RwLock<Arc<MonsterDb>> {
    static DB: OnceLock<RwLock<Arc<MonsterDb>>> = OnceLock::new();
    DB.get_or_init(|| RwLock::new(Arc::new(MonsterDb::default())))
}

/// The current monster list (empty until it has been loaded).
pub fn monster_db() -> Arc<MonsterDb> {
    global().read().clone()
}

/// Load the cached list right away, and download a fresh one in the background
/// if there is no cache or it is older than a week.
pub fn init(app_data_dir: PathBuf, logger: Logger) {
    let cache_path = app_data_dir.join(CACHE_FILE);
    if let Some(cached) = MonsterDb::load_cache(&cache_path) {
        slog::info!(logger, "Loaded monster list from cache"; "count" => cached.len());
        *global().write() = Arc::new(MonsterDb::new(cached));
        if MonsterDb::cache_is_fresh(&cache_path) {
            return;
        }
    }

    tauri::async_runtime::spawn(async move {
        match MonsterDb::fetch_all().await {
            Ok(monsters) => {
                slog::info!(logger, "Downloaded monster list"; "count" => monsters.len());
                let _ = fs::create_dir_all(&app_data_dir);
                if let Ok(file) = fs::File::create(&cache_path) {
                    let _ = serde_json::to_writer(file, &monsters);
                }
                *global().write() = Arc::new(MonsterDb::new(monsters));
            }
            Err(error) => {
                slog::warn!(logger, "Could not download monster list"; "error" => error);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> MonsterDb {
        let m = |id, name: &str, rank| MonsterInfo { id, name: name.into(), level: 1, rank };
        MonsterDb::new(vec![
            m(1, "Aibatt", Rank::Normal),
            m(2, "Giant Aibatt", Rank::Giant),
            m(3, "Mr. Pumpkin", Rank::Giant),
            m(4, "Lawolf", Rank::Normal),
            m(5, "Lawolf", Rank::Captain),
        ])
    }

    #[test]
    fn normalize_strips_punctuation_and_case() {
        assert_eq!(normalize("  Mr. Pumpkin!! "), "mr pumpkin");
    }

    #[test]
    fn exact_lookup() {
        assert_eq!(db().lookup("Aibatt").unwrap().id, 1);
        assert_eq!(db().lookup("mr pumpkin").unwrap().rank, Rank::Giant);
    }

    #[test]
    fn tolerates_ocr_mistakes() {
        // "l" read as "I", one mistake in 6 letters
        assert_eq!(db().lookup("Aibatt").unwrap().id, 1);
        assert_eq!(db().lookup("Lawo1f").unwrap().id, 4);
        assert_eq!(db().lookup("Giant Aibat").unwrap().id, 2);
    }

    #[test]
    fn rejects_garbage() {
        assert!(db().lookup("xyz").is_none());
        assert!(db().lookup("ab").is_none());
        assert!(db().lookup("Completely different").is_none());
    }

    #[test]
    fn duplicate_names_keep_first() {
        assert_eq!(db().lookup("Lawolf").unwrap().rank, Rank::Normal);
    }

    #[test]
    fn duplicate_names_prefer_dangerous() {
        let m = |id, rank| MonsterInfo { id, name: "Twin".into(), level: 1, rank };
        let db = MonsterDb::new(vec![m(1, Rank::Normal), m(2, Rank::Giant)]);
        assert_eq!(db.lookup("Twin").unwrap().id, 2);
    }

    #[test]
    fn unknown_rank_parses() {
        let m: ApiMonster =
            serde_json::from_str(r#"{"id":1,"name":{"en":"X"},"level":3,"rank":"newrank"}"#)
                .unwrap();
        assert_eq!(m.rank, Rank::Unknown);
    }
}
