use serde::{Deserialize, Serialize};

/// A single key remapping rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    /// The key code to intercept (evdev key code name, e.g. "KEY_CAPSLOCK").
    pub from: String,
    /// What to remap it to.
    pub action: KeyAction,
    /// Optional: only apply when this application is focused.
    pub app_filter: Option<String>,
    /// Whether this mapping is currently enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeyAction {
    /// Remap to another key (e.g. CapsLock -> Escape).
    RemapKey { to: String },
    /// Execute a shell command.
    RunCommand { command: String },
    /// Disable the key entirely.
    Disable,
}

fn default_true() -> bool {
    true
}

impl KeyMapping {
    pub fn remap(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            action: KeyAction::RemapKey { to: to.into() },
            app_filter: None,
            enabled: true,
        }
    }

    pub fn command(from: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            action: KeyAction::RunCommand {
                command: command.into(),
            },
            app_filter: None,
            enabled: true,
        }
    }

    pub fn disable(from: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            action: KeyAction::Disable,
            app_filter: None,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_remap() {
        let m = KeyMapping::remap("KEY_CAPSLOCK", "KEY_ESC");
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("KEY_CAPSLOCK"));
        assert!(s.contains("KEY_ESC"));
        assert!(s.contains("remap_key"));
    }

    #[test]
    fn serialize_command() {
        let m = KeyMapping::command("KEY_F12", "xterm");
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("run_command"));
        assert!(s.contains("xterm"));
    }

    #[test]
    fn roundtrip() {
        let m = KeyMapping::remap("KEY_CAPSLOCK", "KEY_ESC");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert_eq!(m2.from, "KEY_CAPSLOCK");
        assert!(matches!(m2.action, KeyAction::RemapKey { to } if to == "KEY_ESC"));
    }
}
