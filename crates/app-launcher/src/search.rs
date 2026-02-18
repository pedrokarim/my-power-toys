use crate::calculator;
use crate::desktop::{self, DesktopEntry};
use crate::frecency::FrecencyStore;
use anyhow::Result;

/// A search result.
#[derive(Debug, Clone)]
pub enum SearchResult {
    App { entry: DesktopEntry, score: f64 },
    Calculation { expression: String, result: f64 },
}

pub struct SearchIndex {
    apps: Vec<DesktopEntry>,
    frecency: FrecencyStore,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            frecency: FrecencyStore::new(),
        }
    }

    /// Refresh the index by scanning .desktop files.
    pub fn refresh(&mut self) -> Result<()> {
        self.apps = desktop::scan_desktop_entries();
        Ok(())
    }

    pub fn app_count(&self) -> usize {
        self.apps.len()
    }

    /// Search for apps and evaluate calculator expressions.
    pub fn search(
        &self,
        query: &str,
        max_results: usize,
        show_calculator: bool,
    ) -> Vec<SearchResult> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // Check if it's a calculator expression
        if show_calculator {
            if let Some(calc_result) = calculator::evaluate(&query) {
                results.push(SearchResult::Calculation {
                    expression: query.clone(),
                    result: calc_result,
                });
            }
        }

        // Search apps
        let mut app_results: Vec<(f64, &DesktopEntry)> = self
            .apps
            .iter()
            .filter_map(|app| {
                let match_score = match_score(app, &query)?;
                let frecency = self.frecency.score(&app.name);
                Some((match_score + frecency, app))
            })
            .collect();

        // Sort by score descending
        app_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (score, entry) in app_results.into_iter().take(max_results) {
            results.push(SearchResult::App {
                entry: entry.clone(),
                score,
            });
        }

        results
    }

    /// Record that an app was launched (for frecency).
    pub fn record_launch(&mut self, app_name: &str) {
        self.frecency.record_launch(app_name);
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Score how well an app matches a query. Returns None if no match.
fn match_score(app: &DesktopEntry, query: &str) -> Option<f64> {
    let name_lower = app.name.to_lowercase();

    // Exact match
    if name_lower == query {
        return Some(100.0);
    }

    // Starts with query
    if name_lower.starts_with(query) {
        return Some(80.0);
    }

    // Contains query
    if name_lower.contains(query) {
        return Some(60.0);
    }

    // Check generic name
    if let Some(ref gn) = app.generic_name
        && gn.to_lowercase().contains(query)
    {
        return Some(40.0);
    }

    // Check keywords
    for keyword in &app.keywords {
        if keyword.to_lowercase().contains(query) {
            return Some(30.0);
        }
    }

    // Check comment
    if let Some(ref comment) = app.comment
        && comment.to_lowercase().contains(query)
    {
        return Some(20.0);
    }

    // Check categories
    for cat in &app.categories {
        if cat.to_lowercase().contains(query) {
            return Some(10.0);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str) -> DesktopEntry {
        DesktopEntry {
            name: name.to_string(),
            generic_name: None,
            comment: None,
            exec: name.to_lowercase(),
            icon: None,
            categories: Vec::new(),
            keywords: Vec::new(),
            terminal: false,
            no_display: false,
            path: std::path::PathBuf::from("/test"),
        }
    }

    #[test]
    fn search_calculator() {
        let index = SearchIndex::new();
        let results = index.search("2 + 3", 8, true);
        assert!(results.iter().any(|r| matches!(r, SearchResult::Calculation { result, .. } if (*result - 5.0).abs() < 0.01)));
    }

    #[test]
    fn search_calculator_disabled() {
        let index = SearchIndex::new();
        let results = index.search("2 + 3", 8, false);
        assert!(
            !results
                .iter()
                .any(|r| matches!(r, SearchResult::Calculation { .. }))
        );
    }

    #[test]
    fn search_empty_returns_nothing() {
        let index = SearchIndex::new();
        assert!(index.search("", 8, true).is_empty());
    }

    #[test]
    fn match_score_exact() {
        let entry = make_entry("Firefox");
        assert_eq!(match_score(&entry, "firefox"), Some(100.0));
    }

    #[test]
    fn match_score_prefix() {
        let entry = make_entry("Firefox");
        assert_eq!(match_score(&entry, "fire"), Some(80.0));
    }

    #[test]
    fn match_score_no_match() {
        let entry = make_entry("Firefox");
        assert_eq!(match_score(&entry, "chrome"), None);
    }
}
