use anyhow::{Context, Result};
use image::RgbaImage;
use std::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

pub fn detect_session() -> SessionType {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        SessionType::Wayland
    } else if std::env::var("DISPLAY").is_ok() {
        SessionType::X11
    } else {
        SessionType::Unknown
    }
}

/// Capture a fullscreen screenshot and return it as an RgbaImage.
pub fn capture_fullscreen() -> Result<RgbaImage> {
    let session = detect_session();
    info!("Detected session type: {session:?}");

    match session {
        SessionType::Wayland => capture_wayland(),
        SessionType::X11 => capture_x11(),
        SessionType::Unknown => {
            warn!("Unknown session type, trying X11 tools...");
            capture_x11()
        }
    }
}

fn capture_wayland() -> Result<RgbaImage> {
    let tmp = screenshot_tmp_path();
    let tmp_str = tmp.to_string_lossy();

    // Try grim first (wlroots-based compositors: Sway, Hyprland, etc.)
    if try_command("grim", &[&tmp_str]) {
        info!("Screenshot captured via grim");
        return load_and_cleanup(&tmp);
    }

    // Try gnome-screenshot (GNOME Wayland)
    if try_command("gnome-screenshot", &["-f", &tmp_str]) {
        info!("Screenshot captured via gnome-screenshot");
        return load_and_cleanup(&tmp);
    }

    // Try spectacle (KDE Wayland)
    if try_command("spectacle", &["-b", "-n", "-o", &tmp_str]) {
        info!("Screenshot captured via spectacle");
        return load_and_cleanup(&tmp);
    }

    anyhow::bail!(
        "Could not capture screenshot on Wayland.\n\
         Install one of: grim (wlroots), gnome-screenshot (GNOME), spectacle (KDE)"
    )
}

fn capture_x11() -> Result<RgbaImage> {
    let tmp = screenshot_tmp_path();
    let tmp_str = tmp.to_string_lossy();

    // Try scrot first (most common X11 screenshot tool)
    if try_command("scrot", &[&tmp_str]) {
        info!("Screenshot captured via scrot");
        return load_and_cleanup(&tmp);
    }

    // Try gnome-screenshot
    if try_command("gnome-screenshot", &["-f", &tmp_str]) {
        info!("Screenshot captured via gnome-screenshot");
        return load_and_cleanup(&tmp);
    }

    // Try maim (another X11 screenshot tool)
    if try_command("maim", &[&tmp_str]) {
        info!("Screenshot captured via maim");
        return load_and_cleanup(&tmp);
    }

    anyhow::bail!(
        "Could not capture screenshot on X11.\n\
         Install one of: scrot, gnome-screenshot, maim"
    )
}

fn try_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn screenshot_tmp_path() -> std::path::PathBuf {
    std::env::temp_dir().join("mpt-color-picker-screenshot.png")
}

fn load_and_cleanup(path: &std::path::Path) -> Result<RgbaImage> {
    let img = image::open(path).context("failed to open screenshot image")?;
    let rgba = img.to_rgba8();
    let _ = std::fs::remove_file(path);
    Ok(rgba)
}
