use mpt_app_launcher::calculator;

use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

#[derive(Default)]
pub struct CalculatorProvider;

impl CalculatorProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for CalculatorProvider {
    fn tag(&self) -> &'static str {
        "calc"
    }

    fn matches(&self, raw_query: &str) -> bool {
        let q = raw_query.trim();
        // Matches on `=` prefix or when query looks like a math expression
        if q.starts_with('=') {
            return true;
        }
        // Also match without prefix if it looks mathematical
        !q.is_empty()
            && !q.starts_with('>')
            && !q.starts_with("??")
            && !q.starts_with('$')
            && !q.starts_with('/')
            && !q.starts_with("file ")
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        let q = raw_query.trim();
        if let Some(stripped) = q.strip_prefix('=') {
            stripped.trim_start()
        } else {
            q
        }
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let expr = ctx.stripped_query.trim();
        if expr.is_empty() {
            return Vec::new();
        }

        let Some(result) = calculator::evaluate(expr) else {
            return Vec::new();
        };

        let formatted = if result == result.floor() {
            format!("{}", result as i64)
        } else {
            format!("{result:.6}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };

        vec![PaletteResult {
            id: format!("calc:{expr}"),
            title: format!("= {formatted}"),
            subtitle: Some(expr.to_string()),
            icon: ResultIcon::BuiltinCalc,
            action: ResultAction::CopyToClipboard(formatted),
            score: 200.0,
            provider_tag: "calc",
        }]
    }
}
