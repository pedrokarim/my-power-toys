use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

#[derive(Default)]
pub struct ShellProvider;

impl ShellProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ShellProvider {
    fn tag(&self) -> &'static str {
        "shell"
    }

    fn matches(&self, raw_query: &str) -> bool {
        raw_query.trim().starts_with('>')
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        raw_query.trim().strip_prefix('>').unwrap_or("").trim()
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let cmd = ctx.stripped_query.trim();
        if cmd.is_empty() {
            return vec![PaletteResult {
                id: "shell:hint".into(),
                title: "Type a command...".into(),
                subtitle: Some("Run in your default shell".into()),
                icon: ResultIcon::BuiltinTerminal,
                action: ResultAction::RunShell(String::new()),
                score: 0.0,
                provider_tag: "shell",
            }];
        }

        vec![PaletteResult {
            id: format!("shell:{cmd}"),
            title: cmd.to_string(),
            subtitle: Some("Run in shell".into()),
            icon: ResultIcon::BuiltinTerminal,
            action: ResultAction::RunShell(cmd.to_string()),
            score: 200.0,
            provider_tag: "shell",
        }]
    }
}
