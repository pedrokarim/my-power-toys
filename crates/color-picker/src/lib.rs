mod color;
mod picker;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

pub use color::{Color, ColorFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPickerConfig {
    #[serde(default = "default_format")]
    pub format: ColorFormat,
}

fn default_format() -> ColorFormat {
    ColorFormat::Hex
}

impl Default for ColorPickerConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
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
        info!("Color Picker: picking color from screen");
        let color = picker::pick_color()?;
        let formatted = color.format(self.config.format);
        info!("Picked color: {formatted}");

        // Copy to clipboard
        picker::copy_to_clipboard(&formatted)?;
        info!("Color copied to clipboard: {formatted}");
        Ok(())
    }
}
