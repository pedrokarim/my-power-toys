mod dbus;
mod modules;
mod tray;

use anyhow::Result;
use mpt_common::config::load_daemon_config;
use mpt_common::platform;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mpt=info".parse().unwrap()),
        )
        .init();

    let ds = platform::detect_display_server();
    let de = platform::detect_desktop_environment();
    info!("MyPowerToys daemon starting");
    info!("Display server: {ds:?}");
    info!("Desktop environment: {de:?}");

    let config = load_daemon_config()?;
    info!(
        "Loaded config: {} module(s) configured",
        config.modules.len()
    );

    let mut registry = modules::ModuleRegistry::new();
    registry.register_defaults();

    // Enable modules based on config
    for (id, entry) in &config.modules {
        if entry.enabled
            && let Err(e) = registry.start_module(id)
        {
            warn!("Failed to start module '{id}': {e}");
        }
    }

    let registry = Arc::new(Mutex::new(registry));

    // Start D-Bus server in background
    let dbus_registry = Arc::clone(&registry);
    tokio::spawn(async move {
        if let Err(e) = dbus::serve(dbus_registry).await {
            tracing::error!("D-Bus server error: {e}");
        }
    });

    // Start tray icon (blocks until quit)
    info!("Starting tray icon...");
    tray::run_tray(Arc::clone(&registry)).await?;

    // Cleanup
    registry.lock().unwrap().stop_all();
    info!("Daemon stopped");
    Ok(())
}
