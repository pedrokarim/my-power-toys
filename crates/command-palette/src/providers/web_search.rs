use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

pub struct WebSearchProvider {
    engine: String,
    custom_url: Option<String>,
}

impl WebSearchProvider {
    pub fn new(engine: String, custom_url: Option<String>) -> Self {
        Self { engine, custom_url }
    }

    fn search_url(&self, query: &str) -> String {
        let encoded = urlencoded(query);
        match self.engine.as_str() {
            "duckduckgo" => format!("https://duckduckgo.com/?q={encoded}"),
            "bing" => format!("https://www.bing.com/search?q={encoded}"),
            "custom" => {
                if let Some(ref tmpl) = self.custom_url {
                    tmpl.replace("{query}", &encoded)
                } else {
                    format!("https://www.google.com/search?q={encoded}")
                }
            }
            _ => format!("https://www.google.com/search?q={encoded}"),
        }
    }

    fn engine_name(&self) -> &str {
        match self.engine.as_str() {
            "duckduckgo" => "DuckDuckGo",
            "bing" => "Bing",
            "custom" => "Custom",
            _ => "Google",
        }
    }
}

impl Provider for WebSearchProvider {
    fn tag(&self) -> &'static str {
        "web"
    }

    fn matches(&self, raw_query: &str) -> bool {
        raw_query.trim().starts_with("??")
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        raw_query.trim().strip_prefix("??").unwrap_or("").trim()
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let query = ctx.stripped_query.trim();
        if query.is_empty() {
            return vec![PaletteResult {
                id: "web:hint".into(),
                title: "Type a search query...".into(),
                subtitle: Some(format!("Search with {}", self.engine_name())),
                icon: ResultIcon::BuiltinWeb,
                action: ResultAction::OpenUrl(String::new()),
                score: 0.0,
                provider_tag: "web",
            }];
        }

        let url = self.search_url(query);
        vec![PaletteResult {
            id: format!("web:{query}"),
            title: format!("Search \"{}\"", query),
            subtitle: Some(format!("on {}", self.engine_name())),
            icon: ResultIcon::BuiltinWeb,
            action: ResultAction::OpenUrl(url),
            score: 200.0,
            provider_tag: "web",
        }]
    }
}

fn urlencoded(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(char::from(HEX[(b >> 4) as usize]));
                out.push(char::from(HEX[(b & 0xf) as usize]));
            }
        }
    }
    out
}

const HEX: [u8; 16] = *b"0123456789ABCDEF";
