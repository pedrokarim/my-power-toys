use crate::frecency::FrecencyStore;
use crate::providers::{PaletteResult, Provider, QueryContext};
use crate::CommandPaletteConfig;

pub struct SearchEngine {
    providers: Vec<Box<dyn Provider>>,
    frecency: FrecencyStore,
    config: CommandPaletteConfig,
}

impl SearchEngine {
    pub fn new(config: CommandPaletteConfig) -> Self {
        use crate::providers::*;

        let mut providers: Vec<Box<dyn Provider>> = Vec::new();

        if config.providers.apps {
            providers.push(Box::new(apps::AppsProvider::new()));
        }
        if config.providers.calculator {
            providers.push(Box::new(calculator::CalculatorProvider::new()));
        }
        if config.providers.web_search {
            providers.push(Box::new(web_search::WebSearchProvider::new(
                config.search_engine.clone(),
                config.custom_search_url.clone(),
            )));
        }
        if config.providers.shell_commands {
            providers.push(Box::new(shell::ShellProvider::new()));
        }
        if config.providers.system_commands {
            providers.push(Box::new(system::SystemProvider::new()));
        }
        if config.providers.file_search {
            providers.push(Box::new(files::FilesProvider::new(
                config.file_search_tool.clone(),
            )));
        }
        if config.providers.settings {
            providers.push(Box::new(settings::SettingsProvider::new()));
        }

        let frecency = FrecencyStore::load();

        Self {
            providers,
            frecency,
            config,
        }
    }

    pub fn initialize(&mut self) {
        for provider in &mut self.providers {
            if let Err(e) = provider.initialize() {
                tracing::warn!("Failed to initialize provider '{}': {e}", provider.tag());
            }
        }
    }

    pub fn search(&self, raw_query: &str) -> Vec<PaletteResult> {
        let raw_query = raw_query.trim();
        if raw_query.is_empty() {
            return Vec::new();
        }

        let mut all_results = Vec::new();

        for provider in &self.providers {
            if !provider.matches(raw_query) {
                continue;
            }

            let stripped = provider.strip_prefix(raw_query);
            let ctx = QueryContext {
                raw_query,
                stripped_query: stripped,
                max_results: self.config.max_results,
            };

            let mut results = provider.search(&ctx);

            // Apply frecency boost
            for result in &mut results {
                let key = format!("{}:{}", result.provider_tag, result.id);
                result.score += self.frecency.score(&key);
            }

            all_results.extend(results);
        }

        all_results
            .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(self.config.max_results);

        all_results
    }

    pub fn record_activation(&mut self, result: &PaletteResult) {
        let key = format!("{}:{}", result.provider_tag, result.id);
        self.frecency.record_launch(&key);
        self.frecency.save();
    }
}
