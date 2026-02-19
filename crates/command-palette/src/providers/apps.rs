use mpt_app_launcher::desktop::{self, DesktopEntry};

use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

#[derive(Default)]
pub struct AppsProvider {
    apps: Vec<DesktopEntry>,
}

impl AppsProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Provider for AppsProvider {
    fn tag(&self) -> &'static str {
        "app"
    }

    fn matches(&self, raw_query: &str) -> bool {
        // Apps provider matches when there's no prefix
        !raw_query.starts_with('>')
            && !raw_query.starts_with("??")
            && !raw_query.starts_with('=')
            && !raw_query.starts_with('$')
            && !raw_query.starts_with('/')
            && !raw_query.starts_with("file ")
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        raw_query
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let query = ctx.stripped_query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(f64, &DesktopEntry)> = self
            .apps
            .iter()
            .filter_map(|app| {
                let score = match_score(app, &query)?;
                Some((score, app))
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(ctx.max_results)
            .map(|(score, entry)| {
                let subtitle = entry
                    .generic_name
                    .clone()
                    .or_else(|| entry.categories.first().cloned());

                let icon = entry
                    .icon
                    .as_ref()
                    .map(|i| ResultIcon::Named(i.clone()))
                    .unwrap_or(ResultIcon::BuiltinApp);

                PaletteResult {
                    id: entry.name.clone(),
                    title: entry.name.clone(),
                    subtitle,
                    icon,
                    action: ResultAction::LaunchExec(entry.exec.clone()),
                    score,
                    provider_tag: "app",
                }
            })
            .collect()
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        self.apps = desktop::scan_desktop_entries();
        tracing::info!("Command Palette: indexed {} applications", self.apps.len());
        Ok(())
    }
}

fn match_score(app: &DesktopEntry, query: &str) -> Option<f64> {
    let name_lower = app.name.to_lowercase();

    if name_lower == query {
        return Some(100.0);
    }
    if name_lower.starts_with(query) {
        return Some(80.0);
    }
    if name_lower.contains(query) {
        return Some(60.0);
    }
    if let Some(ref gn) = app.generic_name
        && gn.to_lowercase().contains(query)
    {
        return Some(40.0);
    }
    for keyword in &app.keywords {
        if keyword.to_lowercase().contains(query) {
            return Some(30.0);
        }
    }
    if let Some(ref comment) = app.comment
        && comment.to_lowercase().contains(query)
    {
        return Some(20.0);
    }
    for cat in &app.categories {
        if cat.to_lowercase().contains(query) {
            return Some(10.0);
        }
    }
    None
}
