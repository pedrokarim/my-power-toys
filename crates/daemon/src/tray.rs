use crate::modules::ModuleRegistry;
use anyhow::Result;
use ksni::menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{Icon, Tray, TrayService};
use mpt_awake::config::{AwakeConfig, AwakeMode};
use std::sync::{Arc, Mutex};
use tracing::info;

const ICON_ARGB: &[u8] = include_bytes!("../../../assets/icons/icon-32.argb");

// Awake mode-specific tray icons (32×32 PNG — for menu items' icon_data)
const AWAKE_OFF_PNG: &[u8] = include_bytes!("../../../assets/icons/awake/awake-off.png");
const AWAKE_INDEFINITE_PNG: &[u8] =
    include_bytes!("../../../assets/icons/awake/awake-indefinite.png");
const AWAKE_TIMED_PNG: &[u8] = include_bytes!("../../../assets/icons/awake/awake-timed.png");
const AWAKE_EXPIRABLE_PNG: &[u8] =
    include_bytes!("../../../assets/icons/awake/awake-expirable.png");

struct MptTray {
    registry: Arc<Mutex<ModuleRegistry>>,
}

impl Tray for MptTray {
    fn id(&self) -> String {
        "my-power-toys".into()
    }

    fn title(&self) -> String {
        "MyPowerToys".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![Icon {
            width: 32,
            height: 32,
            data: ICON_ARGB.to_vec(),
        }]
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();

        // ── Quick actions ────────────────────────────────────────────
        let reg_enable = Arc::clone(&self.registry);
        items.push(
            StandardItem {
                label: "Enable All".into(),
                icon_name: "media-playback-start".into(),
                activate: Box::new(move |_tray: &mut Self| {
                    reg_enable.lock().unwrap().start_all();
                }),
                ..Default::default()
            }
            .into(),
        );

        let reg_disable = Arc::clone(&self.registry);
        items.push(
            StandardItem {
                label: "Disable All".into(),
                icon_name: "media-playback-stop".into(),
                activate: Box::new(move |_tray: &mut Self| {
                    reg_disable.lock().unwrap().stop_all();
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(ksni::MenuItem::Separator);

        // ── Module toggles ──────────────────────────────────────────
        let modules = {
            let reg = self.registry.lock().unwrap();
            let mut list = reg
                .list_modules()
                .into_iter()
                .map(|(id, name, running)| (id.to_string(), name.to_string(), running))
                .collect::<Vec<_>>();
            list.sort_by(|a, b| a.1.cmp(&b.1));
            list
        };

        for (id, name, running) in modules {
            if id == "awake" {
                items.push(build_awake_submenu(
                    &name,
                    running,
                    Arc::clone(&self.registry),
                ));
                continue;
            }

            let module_id = id.clone();
            let registry = Arc::clone(&self.registry);
            items.push(
                CheckmarkItem {
                    label: name,
                    checked: running,
                    enabled: true,
                    activate: Box::new(move |_tray: &mut Self| {
                        let mut reg = registry.lock().unwrap();
                        if running {
                            if let Err(e) = reg.stop_module(&module_id) {
                                tracing::warn!("Failed to stop {module_id}: {e}");
                            }
                        } else if let Err(e) = reg.start_module(&module_id) {
                            tracing::warn!("Failed to start {module_id}: {e}");
                        }
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        // ── Footer ──────────────────────────────────────────────────
        items.push(ksni::MenuItem::Separator);

        items.push(
            StandardItem {
                label: "Open Settings".into(),
                icon_name: "preferences-system".into(),
                activate: Box::new(|_tray: &mut Self| {
                    info!("Opening settings UI");
                    let bin = std::env::current_exe()
                        .ok()
                        .and_then(|p| p.parent().map(|d| d.join("mpt-settings")))
                        .unwrap_or_else(|| "mpt-settings".into());
                    if let Err(e) = std::process::Command::new(&bin)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        tracing::warn!("Failed to launch {}: {e}", bin.display());
                    }
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_tray: &mut Self| {
                    info!("Quit requested from tray");
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

/// Return the PNG icon data for a given Awake mode (used in menu items).
fn awake_mode_png(mode: AwakeMode) -> &'static [u8] {
    match mode {
        AwakeMode::Off => AWAKE_OFF_PNG,
        AwakeMode::Indefinite => AWAKE_INDEFINITE_PNG,
        AwakeMode::Timed => AWAKE_TIMED_PNG,
        AwakeMode::Expirable => AWAKE_EXPIRABLE_PNG,
    }
}

/// Build a SubMenu for the Awake module with RadioGroup mode selection.
fn build_awake_submenu(
    name: &str,
    running: bool,
    registry: Arc<Mutex<ModuleRegistry>>,
) -> ksni::MenuItem<MptTray> {
    let cfg: AwakeConfig = mpt_common::config::load_module_config("awake").unwrap_or_default();

    let mode_label = match cfg.mode {
        AwakeMode::Off => "Off",
        AwakeMode::Indefinite => "∞",
        AwakeMode::Timed => "⏱",
        AwakeMode::Expirable => "⏰",
    };

    let selected = match cfg.mode {
        AwakeMode::Off => 0,
        AwakeMode::Indefinite => 1,
        AwakeMode::Timed => 2,
        AwakeMode::Expirable => 3,
    };

    let reg_radio = Arc::clone(&registry);
    let radio = RadioGroup {
        selected,
        select: Box::new(move |_tray: &mut MptTray, idx| {
            let new_mode = match idx {
                0 => AwakeMode::Off,
                1 => AwakeMode::Indefinite,
                2 => AwakeMode::Timed,
                3 => AwakeMode::Expirable,
                _ => return,
            };
            // Update config on disk
            let mut cfg: AwakeConfig =
                mpt_common::config::load_module_config("awake").unwrap_or_default();
            cfg.mode = new_mode;
            let _ = mpt_common::config::save_module_config("awake", &cfg);

            // Restart the module so it picks up the new mode
            let mut reg = reg_radio.lock().unwrap();
            let _ = reg.stop_module("awake");
            if new_mode != AwakeMode::Off {
                let _ = reg.start_module("awake");
            }
        }),
        options: vec![
            RadioItem {
                label: "Off".into(),
                icon_data: AWAKE_OFF_PNG.to_vec(),
                enabled: true,
                ..Default::default()
            },
            RadioItem {
                label: "Indefinite".into(),
                icon_data: AWAKE_INDEFINITE_PNG.to_vec(),
                enabled: true,
                ..Default::default()
            },
            RadioItem {
                label: "Timed".into(),
                icon_data: AWAKE_TIMED_PNG.to_vec(),
                enabled: true,
                ..Default::default()
            },
            RadioItem {
                label: "Expirable".into(),
                icon_data: AWAKE_EXPIRABLE_PNG.to_vec(),
                enabled: true,
                ..Default::default()
            },
        ],
    };

    let reg_screen = Arc::clone(&registry);
    let keep_screen = cfg.keep_screen_on;
    let screen_toggle = CheckmarkItem {
        label: "Keep screen on".into(),
        checked: keep_screen,
        enabled: true,
        activate: Box::new(move |_tray: &mut MptTray| {
            let mut cfg: AwakeConfig =
                mpt_common::config::load_module_config("awake").unwrap_or_default();
            cfg.keep_screen_on = !cfg.keep_screen_on;
            let _ = mpt_common::config::save_module_config("awake", &cfg);

            // Restart if running to apply the change
            let mut reg = reg_screen.lock().unwrap();
            if reg.is_module_running("awake") {
                let _ = reg.stop_module("awake");
                let _ = reg.start_module("awake");
            }
        }),
        ..Default::default()
    };

    let label = if running {
        format!("{name} ({mode_label})")
    } else {
        name.to_string()
    };

    SubMenu {
        label,
        icon_data: awake_mode_png(cfg.mode).to_vec(),
        enabled: true,
        submenu: vec![
            radio.into(),
            ksni::MenuItem::Separator,
            screen_toggle.into(),
        ],
        ..Default::default()
    }
    .into()
}

pub async fn run_tray(registry: Arc<Mutex<ModuleRegistry>>) -> Result<()> {
    let tray = MptTray { registry };
    let service = TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

    info!("Tray icon active");

    // Keep the daemon alive
    tokio::signal::ctrl_c().await?;
    info!("Received Ctrl+C, shutting down...");
    handle.shutdown();

    Ok(())
}
