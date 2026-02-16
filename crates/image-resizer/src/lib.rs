mod resizer;

use anyhow::Result;
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

pub use resizer::{OutputFormat, ResizePreset, resize_batch, resize_image};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResizerConfig {
    #[serde(default = "default_preset")]
    pub preset: ResizePreset,
    #[serde(default = "default_format")]
    pub output_format: OutputFormat,
    #[serde(default = "default_quality")]
    pub quality: u8,
}

fn default_preset() -> ResizePreset {
    ResizePreset::Medium
}

fn default_format() -> OutputFormat {
    OutputFormat::Original
}

fn default_quality() -> u8 {
    85
}

impl Default for ImageResizerConfig {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            output_format: default_format(),
            quality: default_quality(),
        }
    }
}

pub struct ImageResizer {
    running: bool,
    config: ImageResizerConfig,
}

impl ImageResizer {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("image-resizer").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &ImageResizerConfig {
        &self.config
    }
}

impl Default for ImageResizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for ImageResizer {
    fn id(&self) -> &'static str {
        "image-resizer"
    }

    fn name(&self) -> &'static str {
        "Image Resizer"
    }

    fn description(&self) -> &'static str {
        "Batch resize images with presets and custom dimensions"
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Image Resizer module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Image Resizer module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Image Resizer: would open resize window");
        Ok(())
    }
}
