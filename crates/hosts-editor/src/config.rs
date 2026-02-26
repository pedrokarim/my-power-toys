use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Placement {
    #[default]
    Bottom,
    Top,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsEditorConfig {
    #[serde(default = "default_true")]
    pub show_disabled: bool,
    #[serde(default = "default_true")]
    pub backup_before_save: bool,
    #[serde(default)]
    pub new_entry_placement: Placement,
}

impl Default for HostsEditorConfig {
    fn default() -> Self {
        Self {
            show_disabled: true,
            backup_before_save: true,
            new_entry_placement: Placement::default(),
        }
    }
}
