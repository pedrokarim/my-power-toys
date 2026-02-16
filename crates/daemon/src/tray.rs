use crate::modules::ModuleRegistry;
use anyhow::Result;
use ksni::menu::StandardItem;
use ksni::{Tray, TrayService};
use std::sync::{Arc, Mutex};
use tracing::info;

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

    fn icon_name(&self) -> String {
        "preferences-system".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();

        let modules = {
            let reg = self.registry.lock().unwrap();
            reg.list_modules()
                .into_iter()
                .map(|(id, name, running)| (id.to_string(), name.to_string(), running))
                .collect::<Vec<_>>()
        };

        for (id, name, running) in modules {
            let label = if running {
                format!("✓ {name}")
            } else {
                format!("  {name}")
            };
            let module_id = id.clone();
            let registry = Arc::clone(&self.registry);
            items.push(
                StandardItem {
                    label,
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

        // Separator + Quit
        items.push(ksni::MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Quit".into(),
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
