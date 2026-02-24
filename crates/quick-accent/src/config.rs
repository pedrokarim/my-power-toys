use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivationKey {
    #[default]
    Space,
    LeftRight,
    Any,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolbarPosition {
    #[default]
    AboveCursor,
    BelowCursor,
    TopCenter,
    BottomCenter,
}

fn default_input_delay_ms() -> u64 {
    200
}

fn default_languages() -> Vec<String> {
    vec![
        "fr".into(),
        "es".into(),
        "de".into(),
        "pt".into(),
        "it".into(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAccentConfig {
    #[serde(default)]
    pub activation_key: ActivationKey,

    #[serde(default = "default_input_delay_ms")]
    pub input_delay_ms: u64,

    #[serde(default = "default_languages")]
    pub languages: Vec<String>,

    #[serde(default)]
    pub toolbar_position: ToolbarPosition,
}

impl Default for QuickAccentConfig {
    fn default() -> Self {
        Self {
            activation_key: ActivationKey::default(),
            input_delay_ms: default_input_delay_ms(),
            languages: default_languages(),
            toolbar_position: ToolbarPosition::default(),
        }
    }
}
