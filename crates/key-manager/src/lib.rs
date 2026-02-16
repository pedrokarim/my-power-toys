pub mod mapping;

use anyhow::Result;
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

pub use mapping::KeyMapping;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyManagerConfig {
    #[serde(default)]
    pub mappings: Vec<KeyMapping>,
}

pub struct KeyManager {
    running: bool,
    config: KeyManagerConfig,
}

impl KeyManager {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("key-manager").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &KeyManagerConfig {
        &self.config
    }

    pub fn add_mapping(&mut self, mapping: KeyMapping) {
        self.config.mappings.push(mapping);
    }

    pub fn remove_mapping(&mut self, index: usize) -> Result<()> {
        if index >= self.config.mappings.len() {
            anyhow::bail!("mapping index {index} out of range");
        }
        self.config.mappings.remove(index);
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        mpt_common::config::save_module_config("key-manager", &self.config)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for KeyManager {
    fn id(&self) -> &'static str {
        "key-manager"
    }

    fn name(&self) -> &'static str {
        "Key Manager"
    }

    fn description(&self) -> &'static str {
        "Remap keys and create custom keyboard shortcuts"
    }

    fn start(&mut self) -> Result<()> {
        if self.config.mappings.is_empty() {
            info!("Key Manager: no mappings configured, standing by");
        } else {
            info!(
                "Key Manager: loaded {} mapping(s)",
                self.config.mappings.len()
            );
            // Actual evdev interception will run in a background thread
            // when we implement the full input pipeline.
            // For now, mappings are stored and ready for the UI to configure.
        }
        self.running = true;
        info!("Key Manager module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Key Manager module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Key Manager: would open key mapping editor");
        Ok(())
    }
}
