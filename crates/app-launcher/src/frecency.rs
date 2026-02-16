use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Frecency scoring: combines frequency + recency.
/// Higher score = more relevant result.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrecencyStore {
    entries: HashMap<String, FrecencyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrecencyEntry {
    launch_count: u32,
    last_launch: u64, // unix timestamp
}

impl FrecencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a launch for an app.
    pub fn record_launch(&mut self, app_id: &str) {
        let now = now_secs();
        let entry = self
            .entries
            .entry(app_id.to_string())
            .or_insert(FrecencyEntry {
                launch_count: 0,
                last_launch: now,
            });
        entry.launch_count += 1;
        entry.last_launch = now;
    }

    /// Get the frecency score for an app.
    /// Score = frequency_weight * recency_weight
    pub fn score(&self, app_id: &str) -> f64 {
        let Some(entry) = self.entries.get(app_id) else {
            return 0.0;
        };

        let frequency = (entry.launch_count as f64).ln() + 1.0;
        let age_hours = (now_secs() - entry.last_launch) as f64 / 3600.0;
        let recency = 1.0 / (1.0 + age_hours / 24.0); // Decays over days

        frequency * recency
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_has_zero_score() {
        let store = FrecencyStore::new();
        assert_eq!(store.score("unknown"), 0.0);
    }

    #[test]
    fn launched_app_has_positive_score() {
        let mut store = FrecencyStore::new();
        store.record_launch("firefox");
        assert!(store.score("firefox") > 0.0);
    }

    #[test]
    fn more_launches_higher_score() {
        let mut store = FrecencyStore::new();
        store.record_launch("firefox");
        let score1 = store.score("firefox");
        store.record_launch("firefox");
        store.record_launch("firefox");
        let score2 = store.score("firefox");
        assert!(score2 > score1);
    }
}
