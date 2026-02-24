use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::debug;

/// Describes a physical monitor/output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    /// Output name (e.g. "DP-1", "HDMI-A-1", "eDP-1").
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Detect connected monitors via xrandr, hyprctl, or wlr-randr.
///
/// Returns at least one monitor on success.
/// Falls back through backends: xrandr -> hyprctl -> wlr-randr.
pub fn detect_monitors() -> Result<Vec<Monitor>> {
    if let Ok(monitors) = parse_xrandr()
        && !monitors.is_empty()
    {
        return Ok(monitors);
    }

    if let Ok(monitors) = parse_hyprctl_monitors()
        && !monitors.is_empty()
    {
        return Ok(monitors);
    }

    if let Ok(monitors) = parse_wlr_randr()
        && !monitors.is_empty()
    {
        return Ok(monitors);
    }

    anyhow::bail!("could not detect monitors via xrandr, hyprctl, or wlr-randr")
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
        // First word is the output name (e.g. "DP-1")
        let name = line
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        for word in line.split_whitespace() {
            if let Some(m) = parse_geometry(word, &name, is_primary) {
                monitors.push(m);
                break;
            }
        }
    }

    debug!("xrandr monitors: {monitors:?}");
    Ok(monitors)
}

/// Parse "WxH+X+Y" geometry string (e.g. "1920x1080+0+0").
fn parse_geometry(s: &str, name: &str, is_primary: bool) -> Option<Monitor> {
    let (dims, rest) = s.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    let (x_str, y_str) = rest.split_once('+')?;

    Some(Monitor {
        name: name.to_string(),
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
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let x = item.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = item.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let width = item.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let height = item.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if width > 0 && height > 0 {
                monitors.push(Monitor {
                    name,
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
    let mut current_name = String::new();
    let mut current_w: Option<u32> = None;
    let mut current_h: Option<u32> = None;
    let mut enabled = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // New output section (non-indented line like "DP-1 ...")
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            current_name = trimmed
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string();
            current_w = None;
            current_h = None;
            enabled = false;
        }

        if trimmed == "Enabled: yes" {
            enabled = true;
        }

        // Current mode: "1920x1080 px, 60.000000 Hz (preferred, current)"
        if enabled
            && trimmed.contains("(current)")
            && let Some(res) = trimmed.split_whitespace().next()
            && let Some((w, h)) = res.split_once('x')
        {
            current_w = w.parse().ok();
            current_h = h.parse().ok();
        }

        // Position: "Position: 1920,0"
        if enabled
            && let Some(pos) = trimmed.strip_prefix("Position:")
            && let Some((x_str, y_str)) = pos.trim().split_once(',')
            && let (Some(w), Some(h), Ok(x), Ok(y)) = (
                current_w,
                current_h,
                x_str.trim().parse::<i32>(),
                y_str.trim().parse::<i32>(),
            )
        {
            monitors.push(Monitor {
                name: current_name.clone(),
                x,
                y,
                width: w,
                height: h,
                is_primary: monitors.is_empty(),
            });
        }
    }

    debug!("wlr-randr monitors: {monitors:?}");
    Ok(monitors)
}
