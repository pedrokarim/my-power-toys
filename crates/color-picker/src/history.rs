use crate::color::Color;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const MAX_HISTORY: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub color: Color,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorHistory {
    pub entries: Vec<HistoryEntry>,
}

impl ColorHistory {
    /// Load from ~/.config/my-power-toys/color-picker-history.json
    pub fn load() -> Self {
        let path = history_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Add a color to history (most recent first, max 20).
    pub fn push(&mut self, color: Color) {
        let entry = HistoryEntry {
            color,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_HISTORY);
    }

    /// Remove color at index.
    pub fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = history_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

fn history_path() -> std::path::PathBuf {
    mpt_common::config::config_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("color-picker-history.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_truncate() {
        let mut h = ColorHistory::default();
        for i in 0..25 {
            h.push(Color::new(i, i, i));
        }
        assert_eq!(h.entries.len(), MAX_HISTORY);
        // Most recent should be first
        assert_eq!(h.entries[0].color.r, 24);
    }

    #[test]
    fn remove_entry() {
        let mut h = ColorHistory::default();
        h.push(Color::new(1, 1, 1));
        h.push(Color::new(2, 2, 2));
        h.remove(0);
        assert_eq!(h.entries.len(), 1);
        assert_eq!(h.entries[0].color.r, 1);
    }
}
