pub mod color;
#[cfg(feature = "gui")]
pub mod gui;
pub mod history;
pub mod picker;
pub mod screenshot;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub use color::{Color, ColorFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivationBehavior {
    PickAndEdit,
    PickAndClose,
    EditorOnly,
}

fn default_format() -> ColorFormat {
    ColorFormat::Hex
}

fn default_behavior() -> ActivationBehavior {
    ActivationBehavior::PickAndEdit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPickerConfig {
    #[serde(default = "default_format")]
    pub format: ColorFormat,
    #[serde(default = "default_behavior")]
    pub behavior: ActivationBehavior,
}

impl Default for ColorPickerConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            behavior: default_behavior(),
        }
    }
}

pub struct ColorPicker {
    running: bool,
    config: ColorPickerConfig,
}

impl ColorPicker {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("color-picker").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for ColorPicker {
    fn id(&self) -> &'static str {
        "color-picker"
    }

    fn name(&self) -> &'static str {
        "Color Picker"
    }

    fn description(&self) -> &'static str {
        "Pick any color from the screen and copy it to clipboard"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "C"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Color Picker module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Color Picker module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Color Picker: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                info!("Color Picker GUI spawned (pid={})", child.id());
            }
            Err(e) => {
                warn!("Failed to spawn Color Picker GUI ({bin:?}): {e}");
                warn!("Falling back to headless mode");
                let color = picker::pick_color()?;
                let formatted = color.format(self.config.format);
                picker::copy_to_clipboard(&formatted)?;
                info!("Color copied to clipboard: {formatted}");
            }
        }
        Ok(())
    }
}

/// Find the mpt-color-picker binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-color-picker");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-color-picker".into()
}
