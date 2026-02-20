use anyhow::{Context, Result};
use image::{GenericImageView, RgbaImage};
use std::process::Command;
use tracing::{debug, info, warn};

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
/// On X11 multi-monitor setups, the full virtual desktop is returned (all monitors).
/// On Wayland, the image is cropped to the active monitor (we can't bypass the
/// compositor to span all outputs).
pub fn capture_fullscreen() -> Result<RgbaImage> {
    let session = detect_session();
    info!("Detected session type: {session:?}");

    let screenshot = match session {
        SessionType::Wayland => capture_wayland(),
        SessionType::X11 => capture_x11(),
        SessionType::Unknown => {
            warn!("Unknown session type, trying X11 tools...");
            capture_x11()
        }
    }?;

    info!(
        "Raw screenshot: {}x{}",
        screenshot.width(),
        screenshot.height()
    );

    // On Wayland, crop to active monitor (can't span all outputs from a client).
    // On X11, keep the full virtual desktop — the overlay uses override_redirect
    // to span all monitors.
    if session == SessionType::Wayland {
        match crop_to_active_monitor(&screenshot) {
            Ok(cropped) => {
                info!(
                    "Cropped to active monitor: {}x{}",
                    cropped.width(),
                    cropped.height()
                );
                return Ok(cropped);
            }
            Err(e) => {
                debug!("No multi-monitor crop needed: {e}");
            }
        }
    }

    Ok(screenshot)
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

// ---------------------------------------------------------------------------
// Multi-monitor detection and cropping
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Monitor {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    is_primary: bool,
}

/// Crop the screenshot to the monitor that currently contains the cursor.
/// Returns an error if multi-monitor detection fails or is unnecessary.
fn crop_to_active_monitor(screenshot: &RgbaImage) -> Result<RgbaImage> {
    let monitors = detect_monitors()?;
    if monitors.len() <= 1 {
        anyhow::bail!("single monitor detected");
    }

    info!("Detected {} monitors: {:?}", monitors.len(), monitors);

    // Normalize monitor coordinates to screenshot pixel space.
    // Monitor tools may report logical coordinates (e.g. with HiDPI scaling)
    // while the screenshot is in physical pixels. We compute a scale factor
    // by comparing the monitor bounding box to the screenshot dimensions.
    let scaled = scale_monitors_to_screenshot(&monitors, screenshot);

    // Find which monitor contains the cursor
    let target = match get_cursor_position() {
        Ok((cx, cy)) => {
            info!("Cursor at ({cx}, {cy})");
            // Scale cursor coordinates using the same factor
            let (scx, scy) = scale_cursor((cx, cy), &monitors, screenshot);
            scaled
                .iter()
                .find(|m| {
                    scx >= m.x
                        && scx < m.x + m.width as i32
                        && scy >= m.y
                        && scy < m.y + m.height as i32
                })
                .or_else(|| scaled.iter().find(|m| m.is_primary))
                .or(scaled.first())
        }
        Err(e) => {
            warn!("Could not get cursor position ({e}), using primary monitor");
            scaled
                .iter()
                .find(|m| m.is_primary)
                .or(scaled.first())
        }
    }
    .ok_or_else(|| anyhow::anyhow!("no monitor found"))?
    .clone();

    info!(
        "Cropping to monitor at ({}, {}) {}x{}",
        target.x, target.y, target.width, target.height
    );

    let ox = (target.x.max(0) as u32).min(screenshot.width().saturating_sub(1));
    let oy = (target.y.max(0) as u32).min(screenshot.height().saturating_sub(1));
    let w = target.width.min(screenshot.width().saturating_sub(ox));
    let h = target.height.min(screenshot.height().saturating_sub(oy));

    if w == 0 || h == 0 {
        anyhow::bail!("crop region is empty");
    }

    Ok(screenshot.view(ox, oy, w, h).to_image())
}

/// Scale monitor geometries so that their bounding box matches the screenshot
/// dimensions. This handles the case where tools report logical coordinates
/// on HiDPI setups while the screenshot is in physical pixels.
fn scale_monitors_to_screenshot(monitors: &[Monitor], screenshot: &RgbaImage) -> Vec<Monitor> {
    if monitors.is_empty() {
        return vec![];
    }

    let min_x = monitors.iter().map(|m| m.x).min().unwrap();
    let min_y = monitors.iter().map(|m| m.y).min().unwrap();
    let max_x = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap();
    let max_y = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap();

    let bbox_w = (max_x - min_x) as f64;
    let bbox_h = (max_y - min_y) as f64;

    if bbox_w <= 0.0 || bbox_h <= 0.0 {
        return monitors.to_vec();
    }

    let sx = screenshot.width() as f64 / bbox_w;
    let sy = screenshot.height() as f64 / bbox_h;

    monitors
        .iter()
        .map(|m| Monitor {
            x: ((m.x - min_x) as f64 * sx).round() as i32,
            y: ((m.y - min_y) as f64 * sy).round() as i32,
            width: (m.width as f64 * sx).round() as u32,
            height: (m.height as f64 * sy).round() as u32,
            is_primary: m.is_primary,
        })
        .collect()
}

/// Scale cursor coordinates using the same transform as monitors.
fn scale_cursor(
    cursor: (i32, i32),
    monitors: &[Monitor],
    screenshot: &RgbaImage,
) -> (i32, i32) {
    if monitors.is_empty() {
        return cursor;
    }

    let min_x = monitors.iter().map(|m| m.x).min().unwrap();
    let min_y = monitors.iter().map(|m| m.y).min().unwrap();
    let max_x = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap();
    let max_y = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap();

    let bbox_w = (max_x - min_x) as f64;
    let bbox_h = (max_y - min_y) as f64;

    if bbox_w <= 0.0 || bbox_h <= 0.0 {
        return cursor;
    }

    let sx = screenshot.width() as f64 / bbox_w;
    let sy = screenshot.height() as f64 / bbox_h;

    (
        ((cursor.0 - min_x) as f64 * sx).round() as i32,
        ((cursor.1 - min_y) as f64 * sy).round() as i32,
    )
}

// ---------------------------------------------------------------------------
// Monitor detection backends
// ---------------------------------------------------------------------------

fn detect_monitors() -> Result<Vec<Monitor>> {
    // Try xrandr (X11 and most Wayland via XWayland)
    if let Ok(monitors) = parse_xrandr() {
        if monitors.len() > 1 {
            return Ok(monitors);
        }
    }

    // Try hyprctl (Hyprland)
    if let Ok(monitors) = parse_hyprctl_monitors() {
        if monitors.len() > 1 {
            return Ok(monitors);
        }
    }

    // Try wlr-randr (wlroots compositors: Sway, etc.)
    if let Ok(monitors) = parse_wlr_randr() {
        if monitors.len() > 1 {
            return Ok(monitors);
        }
    }

    anyhow::bail!("could not detect multiple monitors")
}

/// Parse xrandr --query output.
/// Lines look like: "DP-1 connected primary 1920x1080+0+0 (...)"
fn parse_xrandr() -> Result<Vec<Monitor>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()
        .context("xrandr not available")?;

    if !output.status.success() {
        anyhow::bail!("xrandr failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();

    for line in stdout.lines() {
        if !line.contains(" connected") {
            continue;
        }
        let is_primary = line.contains(" primary ");
        for word in line.split_whitespace() {
            if let Some(m) = parse_geometry(word, is_primary) {
                monitors.push(m);
                break;
            }
        }
    }

    debug!("xrandr monitors: {monitors:?}");
    Ok(monitors)
}

/// Parse "WxH+X+Y" geometry string (e.g. "1920x1080+0+0").
fn parse_geometry(s: &str, is_primary: bool) -> Option<Monitor> {
    let (dims, rest) = s.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    let (x_str, y_str) = rest.split_once('+')?;

    Some(Monitor {
        width: w_str.parse().ok()?,
        height: h_str.parse().ok()?,
        x: x_str.parse().ok()?,
        y: y_str.parse().ok()?,
        is_primary,
    })
}

/// Parse hyprctl monitors -j (JSON output).
fn parse_hyprctl_monitors() -> Result<Vec<Monitor>> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .context("hyprctl not available")?;

    if !output.status.success() {
        anyhow::bail!("hyprctl failed");
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut monitors = Vec::new();

    if let Some(arr) = json.as_array() {
        for (i, item) in arr.iter().enumerate() {
            let x = item.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = item.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let width = item.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let height = item.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if width > 0 && height > 0 {
                monitors.push(Monitor {
                    x,
                    y,
                    width,
                    height,
                    is_primary: i == 0,
                });
            }
        }
    }

    debug!("hyprctl monitors: {monitors:?}");
    Ok(monitors)
}

/// Parse wlr-randr output (wlroots compositors).
fn parse_wlr_randr() -> Result<Vec<Monitor>> {
    let output = Command::new("wlr-randr")
        .output()
        .context("wlr-randr not available")?;

    if !output.status.success() {
        anyhow::bail!("wlr-randr failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut current_w: Option<u32> = None;
    let mut current_h: Option<u32> = None;
    let mut enabled = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // New output section (non-indented line)
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            current_w = None;
            current_h = None;
            enabled = false;
        }

        if trimmed == "Enabled: yes" {
            enabled = true;
        }

        // Current mode: "1920x1080 px, 60.000000 Hz (preferred, current)"
        if enabled && trimmed.contains("(current)") {
            if let Some(res) = trimmed.split_whitespace().next() {
                if let Some((w, h)) = res.split_once('x') {
                    current_w = w.parse().ok();
                    current_h = h.parse().ok();
                }
            }
        }

        // Position: "Position: 1920,0"
        if enabled {
            if let Some(pos) = trimmed.strip_prefix("Position:") {
                let pos = pos.trim();
                if let Some((x_str, y_str)) = pos.split_once(',') {
                    if let (Some(w), Some(h), Ok(x), Ok(y)) = (
                        current_w,
                        current_h,
                        x_str.trim().parse::<i32>(),
                        y_str.trim().parse::<i32>(),
                    ) {
                        monitors.push(Monitor {
                            x,
                            y,
                            width: w,
                            height: h,
                            is_primary: monitors.is_empty(),
                        });
                    }
                }
            }
        }
    }

    debug!("wlr-randr monitors: {monitors:?}");
    Ok(monitors)
}

// ---------------------------------------------------------------------------
// Cursor position detection backends
// ---------------------------------------------------------------------------

fn get_cursor_position() -> Result<(i32, i32)> {
    if let Ok(pos) = cursor_via_xdotool() {
        return Ok(pos);
    }
    if let Ok(pos) = cursor_via_hyprctl() {
        return Ok(pos);
    }
    if let Ok(pos) = cursor_via_sway() {
        return Ok(pos);
    }
    if let Ok(pos) = cursor_via_gnome_shell() {
        return Ok(pos);
    }
    anyhow::bail!("could not determine cursor position")
}

fn cursor_via_xdotool() -> Result<(i32, i32)> {
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .context("xdotool not available")?;

    if !output.status.success() {
        anyhow::bail!("xdotool failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut x = None;
    let mut y = None;

    for line in stdout.lines() {
        if let Some(val) = line.strip_prefix("X=") {
            x = val.parse().ok();
        } else if let Some(val) = line.strip_prefix("Y=") {
            y = val.parse().ok();
        }
    }

    match (x, y) {
        (Some(x), Some(y)) => Ok((x, y)),
        _ => anyhow::bail!("could not parse xdotool output"),
    }
}

fn cursor_via_hyprctl() -> Result<(i32, i32)> {
    let output = Command::new("hyprctl")
        .arg("cursorpos")
        .output()
        .context("hyprctl not available")?;

    if !output.status.success() {
        anyhow::bail!("hyprctl cursorpos failed");
    }

    // Format: "X, Y"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').collect();
    if parts.len() == 2 {
        let x: i32 = parts[0].trim().parse()?;
        let y: i32 = parts[1].trim().parse()?;
        return Ok((x, y));
    }

    anyhow::bail!("could not parse hyprctl cursorpos")
}

fn cursor_via_sway() -> Result<(i32, i32)> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_seats", "--raw"])
        .output()
        .context("swaymsg not available")?;

    if !output.status.success() {
        anyhow::bail!("swaymsg failed");
    }

    // JSON array of seats, each with a "cursor" object: {"x": 960.0, "y": 540.0}
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    if let Some(arr) = json.as_array() {
        if let Some(seat) = arr.first() {
            if let Some(cursor) = seat.get("cursor") {
                let x = cursor.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = cursor.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                return Ok((x.round() as i32, y.round() as i32));
            }
        }
    }

    anyhow::bail!("could not parse sway cursor position")
}

fn cursor_via_gnome_shell() -> Result<(i32, i32)> {
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell",
            "--method",
            "org.gnome.Shell.Eval",
            "let [x,y] = global.get_pointer(); '' + x + ',' + y",
        ])
        .output()
        .context("gdbus not available")?;

    if !output.status.success() {
        anyhow::bail!("GNOME Shell Eval failed (might be locked down)");
    }

    // Output: (true, 'X,Y')
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(start) = stdout.find('\'') {
        if let Some(end) = stdout.rfind('\'') {
            if end > start {
                let inner = &stdout[start + 1..end];
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() >= 2 {
                    let x: i32 = parts[0].trim().parse()?;
                    let y: i32 = parts[1].trim().parse()?;
                    return Ok((x, y));
                }
            }
        }
    }

    anyhow::bail!("could not parse GNOME Shell cursor position")
}
