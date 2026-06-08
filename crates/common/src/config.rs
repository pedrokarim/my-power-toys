use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Global daemon configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub modules: HashMap<String, ModuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub autostart: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub hotkey: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: true,
            theme: default_theme(),
        }
    }
}

/// Get the configuration directory: `~/.config/my-power-toys/`
pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not determine config directory")?
        .join("my-power-toys");
    Ok(dir)
}

/// Load the daemon config from `~/.config/my-power-toys/daemon.toml`.
/// Returns default config if the file doesn't exist.
pub fn load_daemon_config() -> Result<DaemonConfig> {
    let path = config_dir()?.join("daemon.toml");

    if !path.exists() {
        return Ok(DaemonConfig::default());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config: DaemonConfig =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

/// Save the daemon config to `~/.config/my-power-toys/daemon.toml`.
pub fn save_daemon_config(config: &DaemonConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("daemon.toml");
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Update the `enabled` flag for a module in `daemon.toml`, preserving the
/// hotkey and other fields. Creates the entry if it doesn't exist.
pub fn set_module_enabled(id: &str, enabled: bool) -> Result<()> {
    let mut config = load_daemon_config().unwrap_or_default();
    config
        .modules
        .entry(id.to_string())
        .and_modify(|entry| entry.enabled = enabled)
        .or_insert(ModuleEntry {
            enabled,
            hotkey: None,
        });
    save_daemon_config(&config)
}

/// Load a module-specific config file (e.g. `color-picker.toml`).
pub fn load_module_config<T: serde::de::DeserializeOwned + Default>(module_id: &str) -> Result<T> {
    let path = config_dir()?.join(format!("{module_id}.toml"));

    if !path.exists() {
        return Ok(T::default());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config: T =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

/// Save a module-specific config file.
pub fn save_module_config<T: Serialize>(module_id: &str, config: &T) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{module_id}.toml"));
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_serializes() {
        let config = DaemonConfig::default();
        let s = toml::to_string_pretty(&config).unwrap();
        assert!(s.contains("autostart"));
    }

    #[test]
    fn parse_config() {
        let input = r#"
[general]
autostart = true
theme = "dark"

[modules.always-on-top]
enabled = true
hotkey = "Super+T"

[modules.awake]
enabled = false
"#;
        let config: DaemonConfig = toml::from_str(input).unwrap();
        assert_eq!(config.general.theme, "dark");
        assert!(config.modules["always-on-top"].enabled);
        assert!(!config.modules["awake"].enabled);
        assert_eq!(
            config.modules["always-on-top"].hotkey.as_deref(),
            Some("Super+T")
        );
    }
}
