use crate::modules::ModuleRegistry;
use anyhow::Result;
use ksni::menu::{CheckmarkItem, StandardItem};
use ksni::{Icon, Tray, TrayService};
use std::sync::{Arc, Mutex};
use tracing::info;

const ICON_ARGB: &[u8] = include_bytes!("../../../assets/icons/icon-32.argb");

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
