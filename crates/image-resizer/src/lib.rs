mod resizer;

#[cfg(feature = "gui")]
pub mod gui;

use anyhow::Result;
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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
        info!("Image Resizer: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                info!("Image Resizer GUI spawned (pid={})", child.id());
            }
            Err(e) => {
                warn!("Failed to spawn Image Resizer GUI ({bin:?}): {e}");
            }
        }
        Ok(())
    }
}

/// Find the mpt-image-resizer binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-image-resizer");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-image-resizer".into()
}
