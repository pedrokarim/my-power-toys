use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// Top-level configuration for the Workspaces module.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspacesConfig {
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

/// A saved workspace: a named collection of apps with positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub apps: Vec<AppEntry>,
    pub created_at: String,
    pub last_launched: Option<String>,
    #[serde(default = "default_true")]
    pub move_existing: bool,
    #[serde(default)]
    pub create_shortcut: bool,
}

impl Workspace {
    pub fn new(name: impl Into<String>, apps: Vec<AppEntry>) -> Self {
        Self {
            name: name.into(),
            apps,
            created_at: chrono::Local::now().to_rfc3339(),
            last_launched: None,
            move_existing: true,
            create_shortcut: false,
        }
    }

    pub fn enabled_apps(&self) -> impl Iterator<Item = &AppEntry> {
        self.apps.iter().filter(|a| a.enabled)
    }

    pub fn app_count(&self) -> usize {
        self.apps.len()
    }
}

/// A single application entry within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub wm_class: String,
    pub exec: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub monitor: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub minimized: bool,
}

/// Resolve executable path from a PID via /proc.
pub fn resolve_exec_from_pid(pid: u32) -> Option<String> {
    std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Get command-line arguments from a PID via /proc.
pub fn resolve_args_from_pid(pid: u32) -> Vec<String> {
    std::fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .ok()
        .map(|s| {
            s.split('\0')
                .skip(1) // skip executable itself
                .filter(|a| !a.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Create a `.desktop` file for a workspace so it appears in the app menu.
#[cfg(feature = "gui")]
pub fn create_desktop_shortcut(workspace_name: &str, workspace_idx: usize) -> anyhow::Result<()> {
    let app_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("no home dir"))?
        .join(".local/share/applications");
    std::fs::create_dir_all(&app_dir)?;

    let slug = workspace_name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");
    let filename = format!("mpt-workspace-{slug}.desktop");

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Workspace: {name}\n\
         Comment=Launch MyPowerToys workspace \"{name}\"\n\
         Exec=mpt-workspaces --launch {idx}\n\
         Icon=my-power-toys\n\
         Terminal=false\n\
         Categories=Utility;System;\n\
         NoDisplay=false\n",
        name = workspace_name,
        idx = workspace_idx,
    );

    std::fs::write(app_dir.join(&filename), content)?;
    tracing::info!("Created desktop shortcut: {filename}");
    Ok(())
}

/// Remove a desktop shortcut for a workspace.
#[cfg(feature = "gui")]
pub fn remove_desktop_shortcut(workspace_name: &str) {
    let slug = workspace_name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");
    let filename = format!("mpt-workspace-{slug}.desktop");

    if let Some(home) = dirs::home_dir() {
        let path = home.join(".local/share/applications").join(&filename);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
            tracing::info!("Removed desktop shortcut: {filename}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_empty() {
        let config = WorkspacesConfig::default();
        assert!(config.workspaces.is_empty());
    }

    #[test]
    fn workspace_new_sets_timestamp() {
        let ws = Workspace::new("Test", vec![]);
        assert!(!ws.created_at.is_empty());
        assert!(ws.last_launched.is_none());
        assert!(ws.move_existing);
    }

    #[test]
    fn workspace_enabled_apps_filters() {
        let apps = vec![
            AppEntry {
                name: "Firefox".into(),
                wm_class: "firefox".into(),
                exec: "/usr/bin/firefox".into(),
                args: vec![],
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                monitor: "DP-1".into(),
                enabled: true,
                minimized: false,
            },
            AppEntry {
                name: "Code".into(),
                wm_class: "code".into(),
                exec: "/usr/bin/code".into(),
                args: vec![],
                x: 0,
                y: 0,
                width: 960,
                height: 1080,
                monitor: "DP-1".into(),
                enabled: false,
                minimized: false,
            },
        ];
        let ws = Workspace::new("Dev", apps);
        assert_eq!(ws.app_count(), 2);
        assert_eq!(ws.enabled_apps().count(), 1);
    }

    #[test]
    fn roundtrip_toml() {
        let config = WorkspacesConfig {
            workspaces: vec![Workspace::new(
                "Dev",
                vec![AppEntry {
                    name: "Firefox".into(),
                    wm_class: "firefox".into(),
                    exec: "/usr/bin/firefox".into(),
                    args: vec!["--profile".into(), "dev".into()],
                    x: 100,
                    y: 200,
                    width: 1920,
                    height: 1080,
                    monitor: "DP-1".into(),
                    enabled: true,
                    minimized: false,
                }],
            )],
        };
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: WorkspacesConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.workspaces.len(), 1);
        assert_eq!(parsed.workspaces[0].name, "Dev");
        assert_eq!(parsed.workspaces[0].apps[0].args, vec!["--profile", "dev"]);
    }
}
