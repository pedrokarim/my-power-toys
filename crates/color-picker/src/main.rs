use mpt_color_picker::color::Color;
use mpt_color_picker::history::ColorHistory;
use mpt_color_picker::picker::copy_to_clipboard;
use mpt_color_picker::screenshot::capture_fullscreen;
use mpt_color_picker::{ActivationBehavior, ColorPickerConfig};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpt_color_picker=info".into()),
        )
        .init();

    // Prevent multiple instances
    let lock_path = lock_file_path();
    if lock_path.exists() {
        // Check if the process is still alive
        if let Ok(pid_str) = fs::read_to_string(&lock_path)
            && let Ok(pid) = pid_str.trim().parse::<u32>()
        {
            let proc_path = format!("/proc/{pid}");
            if std::path::Path::new(&proc_path).exists() {
                info!("Another instance is already running (pid={pid}), exiting");
                std::process::exit(0);
            }
        }
        // Stale lock file, remove it
        let _ = fs::remove_file(&lock_path);
    }

    // Write our PID
    let _ = fs::write(&lock_path, std::process::id().to_string());

    // Ensure lock file is cleaned up on exit
    let lock_path_cleanup = lock_path.clone();
    let _guard = scopeguard::guard((), move |_| {
        let _ = fs::remove_file(&lock_path_cleanup);
    });

    // Load config
    let config: ColorPickerConfig =
        mpt_common::config::load_module_config("color-picker").unwrap_or_default();

    info!("Color Picker GUI starting (behavior={:?})", config.behavior);

    match config.behavior {
        ActivationBehavior::EditorOnly => {
            let history = ColorHistory::load();
            let color = history
                .entries
                .first()
                .map(|e| e.color)
                .unwrap_or(Color::new(128, 128, 128));
            mpt_color_picker::gui::editor::run_editor(color, history);
        }
        ActivationBehavior::PickAndClose | ActivationBehavior::PickAndEdit => {
            // Capture screenshot
            let screenshot = match capture_fullscreen() {
                Ok(img) => img,
                Err(e) => {
                    error!("Failed to capture screenshot: {e}");
                    std::process::exit(1);
                }
            };

            // Show overlay, get picked color
            let picked = mpt_color_picker::gui::overlay::run_overlay(screenshot);

            match picked {
                Some(color) => {
                    info!("Picked color: {color}");

                    // Copy to clipboard
                    let formatted = color.format(config.format);
                    if let Err(e) = copy_to_clipboard(&formatted) {
                        error!("Failed to copy to clipboard: {e}");
                    } else {
                        info!("Copied to clipboard: {formatted}");
                    }

                    // Save to history
                    let mut history = ColorHistory::load();
                    history.push(color);
                    if let Err(e) = history.save() {
                        warn!("Failed to save history: {e}");
                    }

                    // Open editor if configured
                    if config.behavior == ActivationBehavior::PickAndEdit {
                        mpt_color_picker::gui::editor::run_editor(color, history);
                    }
                }
                None => {
                    info!("Color picking cancelled");
                }
            }
        }
    }
}

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("mpt-color-picker.lock")
}
