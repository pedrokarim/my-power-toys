use anyhow::{Result, bail};
use mpt_common::config::DaemonConfig;
use mpt_common::module::PowerModule;
use std::collections::HashMap;
use tracing::info;

pub struct ModuleRegistry {
    modules: HashMap<&'static str, Box<dyn PowerModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register all built-in modules.
    pub fn register_defaults(&mut self) {
        let aot = mpt_always_on_top::AlwaysOnTop::new();
        self.modules.insert(aot.id(), Box::new(aot));

        let awake = mpt_awake::Awake::new();
        self.modules.insert(awake.id(), Box::new(awake));

        let paste = mpt_paste_plain::PastePlain::new();
        self.modules.insert(paste.id(), Box::new(paste));

        let color = mpt_color_picker::ColorPicker::new();
        self.modules.insert(color.id(), Box::new(color));

        let hosts = mpt_hosts_editor::HostsEditor::new();
        self.modules.insert(hosts.id(), Box::new(hosts));

        let rename = mpt_bulk_rename::BulkRename::new();
        self.modules.insert(rename.id(), Box::new(rename));

        let resizer = mpt_image_resizer::ImageResizer::new();
        self.modules.insert(resizer.id(), Box::new(resizer));

        let keys = mpt_key_manager::KeyManager::new();
        self.modules.insert(keys.id(), Box::new(keys));

        let mouse = mpt_mouse_utils::MouseUtils::new();
        self.modules.insert(mouse.id(), Box::new(mouse));

        let ruler = mpt_screen_ruler::ScreenRuler::new();
        self.modules.insert(ruler.id(), Box::new(ruler));

        let ocr = mpt_text_extractor::TextExtractor::new();
        self.modules.insert(ocr.id(), Box::new(ocr));

        let guide = mpt_shortcut_guide::ShortcutGuide::new();
        self.modules.insert(guide.id(), Box::new(guide));

        let launcher = mpt_app_launcher::AppLauncher::new();
        self.modules.insert(launcher.id(), Box::new(launcher));

        let zones = mpt_fancy_zones::FancyZones::new();
        self.modules.insert(zones.id(), Box::new(zones));

        let peek = mpt_peek::Peek::new();
        self.modules.insert(peek.id(), Box::new(peek));

        let palette = mpt_command_palette::CommandPalette::new();
        self.modules.insert(palette.id(), Box::new(palette));

        let ls = mpt_light_switch::LightSwitch::new();
        self.modules.insert(ls.id(), Box::new(ls));

        let accent = mpt_quick_accent::QuickAccent::new();
        self.modules.insert(accent.id(), Box::new(accent));

        let ws = mpt_workspaces::Workspaces::new();
        self.modules.insert(ws.id(), Box::new(ws));

        info!("Registered {} module(s)", self.modules.len());
    }

    pub fn start_module(&mut self, id: &str) -> Result<()> {
        let module = self
            .modules
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("unknown module: {id}"))?;

        if module.is_running() {
            bail!("module '{id}' is already running");
        }

        info!("Starting module: {} ({})", module.name(), id);
        module.start()
    }

    pub fn stop_module(&mut self, id: &str) -> Result<()> {
        let module = self
            .modules
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("unknown module: {id}"))?;

        if !module.is_running() {
            return Ok(());
        }

        info!("Stopping module: {id}");
        module.stop()
    }

    pub fn start_all(&mut self) {
        for (id, module) in &mut self.modules {
            if !module.is_running() {
                info!("Starting module: {} ({id})", module.name());
                if let Err(e) = module.start() {
                    tracing::warn!("Error starting '{id}': {e}");
                }
            }
        }
    }

    pub fn stop_all(&mut self) {
        for (id, module) in &mut self.modules {
            if module.is_running() {
                info!("Stopping module: {id}");
                if let Err(e) = module.stop() {
                    tracing::warn!("Error stopping '{id}': {e}");
                }
            }
        }
    }

    pub fn is_module_running(&self, id: &str) -> bool {
        self.modules.get(id).is_some_and(|m| m.is_running())
    }

    pub fn list_modules(&self) -> Vec<(&'static str, &'static str, bool)> {
        self.modules
            .values()
            .map(|m| (m.id(), m.name(), m.is_running()))
            .collect()
    }

    pub fn hotkey_bindings(&self, config: &DaemonConfig) -> Vec<(String, String)> {
        let mut bindings = Vec::new();

        for module in self.modules.values() {
            // Default hotkey
            let configured = config
                .modules
                .get(module.id())
                .and_then(|entry| entry.hotkey.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            let hotkey = configured.or_else(|| module.default_hotkey().map(|hk| hk.to_string()));
            if let Some(hotkey) = hotkey {
                bindings.push((module.id().to_string(), hotkey));
            }

            // Additional hotkeys (e.g. "fancy-zones:snap")
            for (hk, action) in module.additional_hotkeys() {
                let key = format!("{}:{}", module.id(), action);
                bindings.push((key, hk.to_string()));
            }
        }

        bindings.sort_by(|a, b| a.0.cmp(&b.0));
        bindings
    }

    /// Trigger a hotkey by binding key.
    /// If `id` contains ':', it's a named action (e.g. "fancy-zones:snap").
    pub fn trigger_hotkey(&mut self, id: &str) -> Result<()> {
        if let Some((module_id, action)) = id.split_once(':') {
            let module = self
                .modules
                .get_mut(module_id)
                .ok_or_else(|| anyhow::anyhow!("unknown module: {module_id}"))?;
            if module.is_running() {
                module.on_named_action(action)
            } else {
                bail!("module '{module_id}' is not running");
            }
        } else {
            let module = self
                .modules
                .get_mut(id)
                .ok_or_else(|| anyhow::anyhow!("unknown module: {id}"))?;
            if module.is_running() {
                module.on_hotkey()
            } else {
                bail!("module '{id}' is not running");
            }
        }
    }
}
