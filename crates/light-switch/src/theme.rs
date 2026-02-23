use std::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTheme {
    Light,
    Dark,
}

/// Read the current GNOME color-scheme via gsettings.
pub fn get_current_theme() -> SystemTheme {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output();

    match output {
        Ok(o) if String::from_utf8_lossy(&o.stdout).contains("dark") => SystemTheme::Dark,
        _ => SystemTheme::Light,
    }
}

/// Apply light or dark theme via gsettings.
pub fn set_theme(dark: bool, apply_system: bool, apply_apps: bool) {
    let scheme = if dark { "prefer-dark" } else { "prefer-light" };

    if apply_system || apply_apps {
        let res = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                scheme,
            ])
            .status();

        match res {
            Ok(s) if s.success() => {
                info!("Light Switch: set color-scheme to {scheme}");
            }
            Ok(s) => warn!("gsettings exited with {s}"),
            Err(e) => warn!("failed to run gsettings: {e}"),
        }
    }

    if apply_system {
        let gtk_theme = if dark { "Adwaita-dark" } else { "Adwaita" };
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.interface", "gtk-theme", gtk_theme])
            .status();
    }
}

/// Toggle the current system theme.
pub fn toggle_theme(apply_system: bool, apply_apps: bool) {
    let current = get_current_theme();
    let go_dark = current == SystemTheme::Light;
    set_theme(go_dark, apply_system, apply_apps);
}
