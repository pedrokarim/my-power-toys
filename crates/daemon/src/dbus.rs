use crate::modules::ModuleRegistry;
use anyhow::Result;
use mpt_common::ipc;
use std::sync::{Arc, Mutex};
use tracing::info;
use zbus::{connection, interface};

/// D-Bus interface exposed by the daemon.
/// Accessible at org.mypowertoys.Daemon /org/mypowertoys/Daemon
pub struct DaemonInterface {
    registry: Arc<Mutex<ModuleRegistry>>,
}

impl DaemonInterface {
    pub fn new(registry: Arc<Mutex<ModuleRegistry>>) -> Self {
        Self { registry }
    }
}

#[interface(name = "org.mypowertoys.Daemon")]
impl DaemonInterface {
    /// List all modules: returns array of (id, name, running).
    fn list_modules(&self) -> Vec<(String, String, bool)> {
        let reg = self.registry.lock().unwrap();
        reg.list_modules()
            .into_iter()
            .map(|(id, name, running)| (id.to_string(), name.to_string(), running))
            .collect()
    }

    /// Enable and start a module by id.
    fn start_module(&self, id: &str) -> String {
        let mut reg = self.registry.lock().unwrap();
        match reg.start_module(id) {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Stop a module by id.
    fn stop_module(&self, id: &str) -> String {
        let mut reg = self.registry.lock().unwrap();
        match reg.stop_module(id) {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Trigger a module's hotkey action.
    fn trigger_hotkey(&self, id: &str) -> String {
        let mut reg = self.registry.lock().unwrap();
        match reg.trigger_hotkey(id) {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Ping — returns "pong". Useful to check if daemon is running.
    fn ping(&self) -> String {
        "pong".to_string()
    }

    /// Returns the current daemon version.
    fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check for updates. Returns "up-to-date" or "available:<version>".
    fn check_for_updates(&self) -> String {
        match mpt_common::updater::check_for_update() {
            Ok(Some(info)) => format!("available:{}", info.latest_version),
            Ok(None) => "up-to-date".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }
}

/// Start the D-Bus server. This runs until the connection is dropped.
pub async fn serve(registry: Arc<Mutex<ModuleRegistry>>) -> Result<()> {
    let iface = DaemonInterface::new(registry);

    let _conn = connection::Builder::session()?
        .name(ipc::BUS_NAME)?
        .serve_at(ipc::OBJECT_PATH, iface)?
        .build()
        .await?;

    info!(
        "D-Bus interface active on {} at {}",
        ipc::BUS_NAME,
        ipc::OBJECT_PATH
    );

    // Keep the connection alive forever — the task will be dropped on shutdown
    std::future::pending::<()>().await;

    Ok(())
}
