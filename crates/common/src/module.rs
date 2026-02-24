use crate::hotkey::Hotkey;
use anyhow::Result;

/// Core trait that every MyPowerToys module must implement.
pub trait PowerModule: Send + Sync {
    /// Unique identifier for this module (e.g. "always-on-top").
    fn id(&self) -> &'static str;

    /// Human-readable name (e.g. "Always on Top").
    fn name(&self) -> &'static str;

    /// Short description of what this module does.
    fn description(&self) -> &'static str;

    /// Default hotkey to activate the module, if any.
    fn default_hotkey(&self) -> Option<Hotkey> {
        None
    }

    /// Start the module. Called when the daemon enables it.
    fn start(&mut self) -> Result<()>;

    /// Stop the module. Called when the daemon disables it.
    fn stop(&mut self) -> Result<()>;

    /// Whether the module is currently running.
    fn is_running(&self) -> bool;

    /// Called when the module's hotkey is triggered.
    fn on_hotkey(&mut self) -> Result<()> {
        Ok(())
    }

    /// Additional hotkeys beyond the default one.
    /// Returns a list of (hotkey, action_name) pairs.
    fn additional_hotkeys(&self) -> Vec<(Hotkey, &'static str)> {
        vec![]
    }

    /// Called when a named action (from additional_hotkeys) is triggered.
    fn on_named_action(&mut self, _action: &str) -> Result<()> {
        Ok(())
    }
}
