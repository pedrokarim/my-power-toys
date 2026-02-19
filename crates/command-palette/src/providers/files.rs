use std::process::Command;

use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

pub struct FilesProvider {
    tool: String,
}

impl FilesProvider {
    pub fn new(tool: String) -> Self {
        Self { tool }
    }

    fn find_tool(&self) -> &str {
        if self.tool != "auto" {
            return &self.tool;
        }
        // Auto-detect: prefer fd, fallback to find
        if Command::new("fd").arg("--version").output().is_ok() {
            "fd"
        } else {
            "find"
        }
    }
}

impl Provider for FilesProvider {
    fn tag(&self) -> &'static str {
        "file"
    }

    fn matches(&self, raw_query: &str) -> bool {
        let q = raw_query.trim();
        q.starts_with('/') || q.starts_with("file ")
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        let q = raw_query.trim();
        if let Some(stripped) = q.strip_prefix("file ") {
            stripped.trim()
        } else if let Some(stripped) = q.strip_prefix('/') {
            stripped
        } else {
            q
        }
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let query = ctx.stripped_query.trim();
        if query.len() < 2 {
            return vec![PaletteResult {
                id: "file:hint".into(),
                title: "Keep typing to search files...".into(),
                subtitle: None,
                icon: ResultIcon::BuiltinFile,
                action: ResultAction::OpenUrl(String::new()),
                score: 0.0,
                provider_tag: "file",
            }];
        }

        let tool = self.find_tool();
        let home = dirs::home_dir().unwrap_or_else(|| "/home".into());

        let output = match tool {
            "fd" => Command::new("fd")
                .args(["--max-results", "10", "--type", "f", query])
                .current_dir(&home)
                .output(),
            "locate" => Command::new("locate")
                .args(["-l", "10", "-i", query])
                .output(),
            _ => Command::new("find")
                .args([
                    home.to_str().unwrap_or("/home"),
                    "-maxdepth",
                    "5",
                    "-type",
                    "f",
                    "-iname",
                    &format!("*{query}*"),
                ])
                .output(),
        };

        let Ok(output) = output else {
            return Vec::new();
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .take(ctx.max_results)
            .enumerate()
            .map(|(i, line)| {
                let path = line.trim().to_string();
                let filename = std::path::Path::new(&path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                let parent = std::path::Path::new(&path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string());

                PaletteResult {
                    id: format!("file:{path}"),
                    title: filename,
                    subtitle: parent,
                    icon: ResultIcon::BuiltinFile,
                    action: ResultAction::OpenUrl(path),
                    score: 100.0 - i as f64,
                    provider_tag: "file",
                }
            })
            .collect()
    }
}
