use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// KeyCombo – a key combination (modifiers + action key + optional chord)
// ---------------------------------------------------------------------------

/// Represents a key or key combination.
///
/// Examples (in evdev naming):
/// - Single key:  `KeyCombo::key("KEY_ESC")`
/// - Shortcut:    `KeyCombo { modifiers: vec!["KEY_LEFTCTRL"], key: "KEY_C", chord: None }`
/// - Chord:       `KeyCombo { modifiers: vec!["KEY_LEFTCTRL"], key: "KEY_V", chord: Some("KEY_U") }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyCombo {
    /// Modifier keys held down (e.g. `["KEY_LEFTCTRL", "KEY_LEFTSHIFT"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
    /// The main action key (e.g. `"KEY_C"`).
    pub key: String,
    /// Optional chord: a second key pressed *after* the first combo is held.
    /// e.g. `Ctrl+V` then `U` for "Volume Up".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chord: Option<String>,
}

impl KeyCombo {
    /// A single key with no modifiers.
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            modifiers: Vec::new(),
            key: key.into(),
            chord: None,
        }
    }

    /// A shortcut with modifiers and an action key.
    pub fn shortcut(modifiers: Vec<String>, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into(),
            chord: None,
        }
    }

    /// A chord: modifiers + first key, then a second key.
    pub fn chord(
        modifiers: Vec<String>,
        key: impl Into<String>,
        second: impl Into<String>,
    ) -> Self {
        Self {
            modifiers,
            key: key.into(),
            chord: Some(second.into()),
        }
    }

    /// Returns `true` if this combo has no modifiers (single key).
    pub fn is_single_key(&self) -> bool {
        self.modifiers.is_empty() && self.chord.is_none()
    }
}

// ---------------------------------------------------------------------------
// KeyTrigger – what activates a mapping
// ---------------------------------------------------------------------------

/// What triggers a key mapping: either a single key or a full combo/shortcut.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyTrigger {
    /// A single key (e.g. `"KEY_CAPSLOCK"`).
    Key(String),
    /// A shortcut combo (e.g. Ctrl+Shift+C), possibly with a chord.
    Combo(KeyCombo),
}

impl KeyTrigger {
    pub fn key(key: impl Into<String>) -> Self {
        Self::Key(key.into())
    }

    pub fn combo(modifiers: Vec<String>, key: impl Into<String>) -> Self {
        Self::Combo(KeyCombo::shortcut(modifiers, key))
    }

    pub fn chord(
        modifiers: Vec<String>,
        key: impl Into<String>,
        second: impl Into<String>,
    ) -> Self {
        Self::Combo(KeyCombo::chord(modifiers, key, second))
    }
}

// ---------------------------------------------------------------------------
// KeyAction – what happens when a mapping fires
// ---------------------------------------------------------------------------

/// The action to perform when a key mapping is triggered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeyAction {
    /// Remap to another key or key combination.
    RemapKey { to: KeyCombo },
    /// Send arbitrary unicode text.
    SendText { text: String },
    /// Execute a shell command.
    RunCommand { command: String },
    /// Launch an application with structured options.
    LaunchApp {
        path: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        working_dir: Option<String>,
        #[serde(default)]
        elevated: bool,
    },
    /// Open a URI in the default handler.
    OpenUri { uri: String },
    /// Disable the key entirely (swallow the event).
    Disable,
}

// ---------------------------------------------------------------------------
// KeyMapping – a complete mapping rule
// ---------------------------------------------------------------------------

/// A single key-remapping rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    /// What triggers this mapping: a single key or a shortcut combo.
    pub trigger: KeyTrigger,
    /// What to do when triggered.
    pub action: KeyAction,
    /// Only apply when this process name is focused (e.g. `"firefox"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_filter: Option<String>,
    /// Whether this mapping is currently enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// If `true`, the shortcut trigger must match exactly (no extra keys held).
    #[serde(default, skip_serializing_if = "is_false")]
    pub exact_match: bool,
}

fn default_true() -> bool {
    true
}

fn is_false(v: &bool) -> bool {
    !v
}

impl KeyMapping {
    // -- Single-key helpers --------------------------------------------------

    /// Remap one key to another (e.g. CapsLock → Escape).
    pub fn remap(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            trigger: KeyTrigger::Key(from.into()),
            action: KeyAction::RemapKey {
                to: KeyCombo::key(to),
            },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Remap a key to a shortcut combo (e.g. Ctrl → Win+Left).
    pub fn remap_to_combo(
        from: impl Into<String>,
        modifiers: Vec<String>,
        to_key: impl Into<String>,
    ) -> Self {
        Self {
            trigger: KeyTrigger::Key(from.into()),
            action: KeyAction::RemapKey {
                to: KeyCombo::shortcut(modifiers, to_key),
            },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Execute a shell command when a key is pressed.
    pub fn command(from: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            trigger: KeyTrigger::Key(from.into()),
            action: KeyAction::RunCommand {
                command: command.into(),
            },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Disable a key entirely.
    pub fn disable(from: impl Into<String>) -> Self {
        Self {
            trigger: KeyTrigger::Key(from.into()),
            action: KeyAction::Disable,
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Send text when a key is pressed (e.g. H → "Hello!").
    pub fn send_text(from: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            trigger: KeyTrigger::Key(from.into()),
            action: KeyAction::SendText { text: text.into() },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    // -- Shortcut helpers ----------------------------------------------------

    /// Remap a shortcut combo to another combo (e.g. Alt+C → Ctrl+C).
    pub fn shortcut_remap(from: KeyCombo, to: KeyCombo) -> Self {
        Self {
            trigger: KeyTrigger::Combo(from),
            action: KeyAction::RemapKey { to },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Launch an application when a shortcut is pressed.
    pub fn launch_app(trigger: KeyTrigger, path: impl Into<String>) -> Self {
        Self {
            trigger,
            action: KeyAction::LaunchApp {
                path: path.into(),
                args: Vec::new(),
                working_dir: None,
                elevated: false,
            },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    /// Open a URI when a shortcut is pressed.
    pub fn open_uri(trigger: KeyTrigger, uri: impl Into<String>) -> Self {
        Self {
            trigger,
            action: KeyAction::OpenUri { uri: uri.into() },
            app_filter: None,
            enabled: true,
            exact_match: false,
        }
    }

    // -- Builder methods -----------------------------------------------------

    /// Restrict this mapping to a specific application.
    pub fn for_app(mut self, app: impl Into<String>) -> Self {
        self.app_filter = Some(app.into());
        self
    }

    /// Enable exact-match mode (no extra keys allowed).
    pub fn exact(mut self) -> Self {
        self.exact_match = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Basic remap (single key → single key) ------------------------------

    #[test]
    fn serialize_remap() {
        let m = KeyMapping::remap("KEY_CAPSLOCK", "KEY_ESC");
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("KEY_CAPSLOCK"));
        assert!(s.contains("KEY_ESC"));
        assert!(s.contains("remap_key"));
    }

    #[test]
    fn roundtrip_remap() {
        let m = KeyMapping::remap("KEY_CAPSLOCK", "KEY_ESC");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert!(matches!(&m2.trigger, KeyTrigger::Key(k) if k == "KEY_CAPSLOCK"));
        assert!(matches!(&m2.action, KeyAction::RemapKey { to } if to.key == "KEY_ESC"));
    }

    // -- Command action -----------------------------------------------------

    #[test]
    fn serialize_command() {
        let m = KeyMapping::command("KEY_F12", "xterm");
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("run_command"));
        assert!(s.contains("xterm"));
    }

    // -- Send text action ---------------------------------------------------

    #[test]
    fn roundtrip_send_text() {
        let m = KeyMapping::send_text("KEY_H", "Hello!");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert!(matches!(&m2.action, KeyAction::SendText { text } if text == "Hello!"));
    }

    // -- Shortcut remap (combo → combo) -------------------------------------

    #[test]
    fn roundtrip_shortcut_remap() {
        let from = KeyCombo::shortcut(vec!["KEY_LEFTALT".into()], "KEY_C");
        let to = KeyCombo::shortcut(vec!["KEY_LEFTCTRL".into()], "KEY_C");
        let m = KeyMapping::shortcut_remap(from, to);
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        if let KeyTrigger::Combo(c) = &m2.trigger {
            assert_eq!(c.modifiers, vec!["KEY_LEFTALT"]);
            assert_eq!(c.key, "KEY_C");
        } else {
            panic!("expected Combo trigger");
        }
    }

    // -- Chord support ------------------------------------------------------

    #[test]
    fn roundtrip_chord() {
        let from = KeyCombo::chord(
            vec!["KEY_LEFTCTRL".into(), "KEY_LEFTSHIFT".into()],
            "KEY_V",
            "KEY_U",
        );
        let m = KeyMapping {
            trigger: KeyTrigger::Combo(from),
            action: KeyAction::SendText {
                text: "Volume Up".into(),
            },
            app_filter: None,
            enabled: true,
            exact_match: false,
        };
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("KEY_U"));
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        if let KeyTrigger::Combo(c) = &m2.trigger {
            assert_eq!(c.chord.as_deref(), Some("KEY_U"));
        } else {
            panic!("expected Combo trigger with chord");
        }
    }

    // -- Launch app action --------------------------------------------------

    #[test]
    fn roundtrip_launch_app() {
        let trigger = KeyTrigger::combo(vec!["KEY_LEFTCTRL".into()], "KEY_T");
        let m = KeyMapping::launch_app(trigger, "/usr/bin/alacritty");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert!(
            matches!(&m2.action, KeyAction::LaunchApp { path, .. } if path == "/usr/bin/alacritty")
        );
    }

    // -- Open URI action ----------------------------------------------------

    #[test]
    fn roundtrip_open_uri() {
        let trigger = KeyTrigger::combo(vec!["KEY_LEFTCTRL".into()], "KEY_G");
        let m = KeyMapping::open_uri(trigger, "https://github.com");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert!(matches!(&m2.action, KeyAction::OpenUri { uri } if uri == "https://github.com"));
    }

    // -- Disable action -----------------------------------------------------

    #[test]
    fn serialize_disable() {
        let m = KeyMapping::disable("KEY_INSERT");
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("disable"));
        assert!(s.contains("KEY_INSERT"));
    }

    // -- App filter & exact match -------------------------------------------

    #[test]
    fn app_filter_and_exact_match() {
        let m = KeyMapping::remap("KEY_CAPSLOCK", "KEY_ESC")
            .for_app("code")
            .exact();
        let s = toml::to_string(&m).unwrap();
        assert!(s.contains("code"));
        assert!(s.contains("exact_match"));
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        assert_eq!(m2.app_filter.as_deref(), Some("code"));
        assert!(m2.exact_match);
    }

    // -- KeyCombo helpers ---------------------------------------------------

    #[test]
    fn key_combo_is_single_key() {
        assert!(KeyCombo::key("KEY_A").is_single_key());
        assert!(!KeyCombo::shortcut(vec!["KEY_LEFTCTRL".into()], "KEY_A").is_single_key());
        assert!(!KeyCombo::chord(vec!["KEY_LEFTCTRL".into()], "KEY_V", "KEY_U").is_single_key());
    }

    // -- Remap key to combo -------------------------------------------------

    #[test]
    fn roundtrip_remap_to_combo() {
        let m = KeyMapping::remap_to_combo("KEY_LEFTCTRL", vec!["KEY_LEFTMETA".into()], "KEY_LEFT");
        let s = toml::to_string(&m).unwrap();
        let m2: KeyMapping = toml::from_str(&s).unwrap();
        if let KeyAction::RemapKey { to } = &m2.action {
            assert_eq!(to.modifiers, vec!["KEY_LEFTMETA"]);
            assert_eq!(to.key, "KEY_LEFT");
        } else {
            panic!("expected RemapKey action");
        }
    }
}
