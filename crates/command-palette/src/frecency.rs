use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrecencyStore {
    entries: HashMap<String, FrecencyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrecencyEntry {
    launch_count: u32,
    last_launch: u64,
}

impl FrecencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Self {
        let path = frecency_path();
        if path.exists()
            && let Ok(data) = std::fs::read_to_string(&path)
            && let Ok(store) = serde_json::from_str(&data)
        {
            return store;
        }
        Self::new()
    }

    pub fn save(&self) {
        let path = frecency_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }

    pub fn record_launch(&mut self, id: &str) {
        let now = now_secs();
        let entry = self.entries.entry(id.to_string()).or_insert(FrecencyEntry {
            launch_count: 0,
            last_launch: now,
        });
        entry.launch_count += 1;
        entry.last_launch = now;
    }

    pub fn score(&self, id: &str) -> f64 {
        let Some(entry) = self.entries.get(id) else {
            return 0.0;
        };
        let frequency = (entry.launch_count as f64).ln() + 1.0;
        let age_hours = (now_secs() - entry.last_launch) as f64 / 3600.0;
        let recency = 1.0 / (1.0 + age_hours / 24.0);
        frequency * recency
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn frecency_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| "/tmp".into())
        .join("my-power-toys")
        .join("command-palette-frecency.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entry_has_zero_score() {
        let store = FrecencyStore::new();
        assert_eq!(store.score("unknown"), 0.0);
    }

    #[test]
    fn launched_entry_has_positive_score() {
        let mut store = FrecencyStore::new();
        store.record_launch("test");
        assert!(store.score("test") > 0.0);
    }

    #[test]
    fn more_launches_higher_score() {
        let mut store = FrecencyStore::new();
        store.record_launch("test");
        let score1 = store.score("test");
        store.record_launch("test");
        store.record_launch("test");
        let score2 = store.score("test");
        assert!(score2 > score1);
    }
}
