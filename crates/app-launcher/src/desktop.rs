use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// A parsed .desktop file entry.
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
    pub no_display: bool,
    pub path: PathBuf,
}

/// Get all standard .desktop file directories.
pub fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }

    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            let p = PathBuf::from(dir).join("applications");
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }

    dirs
}

/// Scan all .desktop files and parse them.
pub fn scan_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    for dir in desktop_dirs() {
        if let Ok(read_dir) = fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "desktop")
                    && let Ok(de) = parse_desktop_file(&path)
                    && !de.no_display
                {
                    entries.push(de);
                }
            }
        }
    }
    // Deduplicate by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries.dedup_by(|a, b| a.name == b.name);
    entries
}

/// Parse a single .desktop file.
pub fn parse_desktop_file(path: &Path) -> Result<DesktopEntry> {
    let content = fs::read_to_string(path)?;
    let mut name = None;
    let mut generic_name = None;
    let mut comment = None;
    let mut exec = None;
    let mut icon = None;
    let mut categories = Vec::new();
    let mut keywords = Vec::new();
    let mut terminal = false;
    let mut no_display = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }
        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Name" => name = Some(value.to_string()),
                "GenericName" => generic_name = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "Exec" => exec = Some(strip_exec_args(value)),
                "Icon" => icon = Some(value.to_string()),
                "Categories" => {
                    categories = value
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                }
                "Keywords" => {
                    keywords = value
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                }
                "Terminal" => terminal = value == "true",
                "NoDisplay" => no_display = value == "true",
                _ => {}
            }
        }
    }

    let name = name.ok_or_else(|| anyhow::anyhow!("missing Name in {}", path.display()))?;
    let exec = exec.ok_or_else(|| anyhow::anyhow!("missing Exec in {}", path.display()))?;

    Ok(DesktopEntry {
        name,
        generic_name,
        comment,
        exec,
        icon,
        categories,
        keywords,
        terminal,
        no_display,
        path: path.to_path_buf(),
    })
}

/// Strip %f, %F, %u, %U, etc. from Exec line.
fn strip_exec_args(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_exec_placeholders() {
        assert_eq!(strip_exec_args("firefox %u"), "firefox");
        assert_eq!(strip_exec_args("code --new-window %F"), "code --new-window");
        assert_eq!(strip_exec_args("nautilus"), "nautilus");
    }

    #[test]
    fn desktop_dirs_not_empty() {
        let dirs = desktop_dirs();
        assert!(!dirs.is_empty());
        assert!(
            dirs.iter()
                .any(|d| d.to_string_lossy().contains("applications"))
        );
    }
}
