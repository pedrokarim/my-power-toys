mod dbus;
mod hotkeys;
mod modules;
mod tray;

use anyhow::Result;
use mpt_common::config::load_daemon_config;
use mpt_common::ipc;
use mpt_common::platform;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Check if another daemon instance is already running via D-Bus.
async fn another_instance_running() -> bool {
    let conn = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(_) => return false,
    };
    conn.call_method(
        Some(ipc::BUS_NAME),
        ipc::OBJECT_PATH,
        Some(ipc::BUS_NAME),
        "Ping",
        &(),
    )
    .await
    .is_ok()
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mpt=info".parse().unwrap()),
        )
        .init();

    // Singleton check: exit if another daemon is already running
    if another_instance_running().await {
        error!("Another mpt-daemon instance is already running.");
        eprintln!("Error: another mpt-daemon instance is already running.");
        eprintln!("Check with: mpt-ctl ping");
        eprintln!("Or manage via: systemctl --user status my-power-toys");
        std::process::exit(1);
    }

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

    // Start modules: if a module is in the config, respect its `enabled` flag;
    // if it's not in the config, start it by default.
    let module_ids: Vec<&str> = registry.list_modules().iter().map(|(id, _, _)| *id).collect();
    for id in module_ids {
        let enabled = config.modules.get(id).map_or(true, |entry| entry.enabled);
        if enabled {
            if let Err(e) = registry.start_module(id) {
                warn!("Failed to start module '{id}': {e}");
            }
        }
    }

    let registry = Arc::new(Mutex::new(registry));

    if ds == platform::DisplayServer::X11 {
        let bindings = {
            let reg = registry.lock().unwrap();
            reg.hotkey_bindings(&config)
        };
        hotkeys::spawn_x11_hotkey_listener(bindings, Arc::clone(&registry));
    } else {
        warn!("Global hotkeys are currently available only on X11 sessions.");
        warn!(
            "Detected {ds:?}. On Ubuntu Wayland, shortcuts may require session-level bindings or X11 login."
        );
    }

    // Start D-Bus server in background
    let dbus_registry = Arc::clone(&registry);
    tokio::spawn(async move {
        if let Err(e) = dbus::serve(dbus_registry).await {
            error!("D-Bus server error: {e}");
            std::process::exit(1);
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
