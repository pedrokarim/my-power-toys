mod ocr;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractorConfig {
    /// Tesseract language (e.g. "eng", "fra", "eng+fra").
    #[serde(default = "default_lang")]
    pub language: String,
}

fn default_lang() -> String {
    "eng".to_string()
}

impl Default for TextExtractorConfig {
    fn default() -> Self {
        Self {
            language: default_lang(),
        }
    }
}

pub struct TextExtractor {
    running: bool,
    config: TextExtractorConfig,
}

impl TextExtractor {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("text-extractor").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }
}

impl Default for TextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for TextExtractor {
    fn id(&self) -> &'static str {
        "text-extractor"
    }

    fn name(&self) -> &'static str {
        "Text Extractor"
    }

    fn description(&self) -> &'static str {
        "Extract text from any screen region using OCR (requires tesseract)"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "T"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Text Extractor started (lang={})", self.config.language);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Text Extractor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Text Extractor: capturing screen region for OCR");
        let text = ocr::capture_and_extract(&self.config.language)?;
        if text.is_empty() {
            info!("No text detected");
        } else {
            info!("Extracted {} chars, copying to clipboard", text.len());
            ocr::copy_to_clipboard(&text)?;
        }
        Ok(())
    }
}
