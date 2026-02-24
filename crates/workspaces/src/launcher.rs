use anyhow::Result;
use tracing::{info, warn};

use crate::config::{AppEntry, Workspace};
use crate::x11;

/// Status of a single app launch attempt.
#[derive(Debug, Clone)]
pub enum LaunchStatus {
    Launched { app_name: String },
    Repositioned { app_name: String },
    Failed { app_name: String, error: String },
    Skipped { app_name: String },
}

/// Launch all enabled apps in a workspace and reposition their windows.
pub fn launch_workspace(workspace: &mut Workspace) -> Vec<LaunchStatus> {
    let mut statuses = Vec::new();

    for app in &workspace.apps {
        if !app.enabled {
            statuses.push(LaunchStatus::Skipped {
                app_name: app.name.clone(),
            });
            continue;
        }

        let status = launch_single_app(app, workspace.move_existing);
        statuses.push(status);
    }

    // Update last_launched timestamp
    workspace.last_launched = Some(chrono::Local::now().to_rfc3339());

    statuses
}

/// Launch or reposition a single app.
fn launch_single_app(app: &AppEntry, move_existing: bool) -> LaunchStatus {
    // If move_existing is enabled, try to find and reposition an existing window
    if move_existing
        && let Ok(existing) = x11::find_windows_by_class(&app.wm_class)
        && let Some(&win_id) = existing.first()
    {
        info!(
            "Repositioning existing window for {} (id={})",
            app.name, win_id
        );
        if let Err(e) = x11::move_resize_window(win_id, app.x, app.y, app.width, app.height) {
            return LaunchStatus::Failed {
                app_name: app.name.clone(),
                error: format!("reposition failed: {e}"),
            };
        }
        return LaunchStatus::Repositioned {
            app_name: app.name.clone(),
        };
    }

    // Launch the app
    info!("Launching {} ({})", app.name, app.exec);
    let mut cmd = std::process::Command::new(&app.exec);
    cmd.args(&app.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(_child) => {
            // Wait for the window to appear and reposition it
            if let Err(e) = wait_and_reposition(app) {
                warn!("Launched {} but reposition failed: {e}", app.name);
            }
            LaunchStatus::Launched {
                app_name: app.name.clone(),
            }
        }
        Err(e) => LaunchStatus::Failed {
            app_name: app.name.clone(),
            error: format!("spawn failed: {e}"),
        },
    }
}

/// Wait for a window matching the app's WM_CLASS to appear, then reposition it.
fn wait_and_reposition(app: &AppEntry) -> Result<()> {
    let max_attempts = 50; // 50 * 200ms = 10 seconds
    let delay = std::time::Duration::from_millis(200);

    // Get existing windows before launch to detect new ones
    let before: Vec<u32> = x11::find_windows_by_class(&app.wm_class).unwrap_or_default();

    for attempt in 0..max_attempts {
        std::thread::sleep(delay);

        let after = x11::find_windows_by_class(&app.wm_class).unwrap_or_default();

        // Find a new window that wasn't there before
        if let Some(&new_win) = after.iter().find(|w| !before.contains(w)) {
            info!(
                "New window detected for {} (id={}) after {}ms",
                app.name,
                new_win,
                (attempt + 1) * 200
            );
            x11::move_resize_window(new_win, app.x, app.y, app.width, app.height)?;
            return Ok(());
        }
    }

    anyhow::bail!(
        "timeout: no new window for {} (class={}) after 10s",
        app.name,
        app.wm_class
    )
}
