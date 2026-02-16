use crate::color::Color;
use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

/// Pick a color from the screen.
///
/// Strategy:
/// 1. Take a screenshot of the entire screen via `gnome-screenshot` or `grim`
/// 2. Use `xdotool` to get mouse position
/// 3. Read the pixel at that position from the screenshot
///
/// Alternative: use `colorpicker` command or `grabc` if available.
pub fn pick_color() -> Result<Color> {
    // Try the simple approach first: use grabc if available
    if let Ok(color) = pick_with_grabc() {
        return Ok(color);
    }

    // Fallback: screenshot + mouse position
    pick_with_screenshot()
}

/// Use `grabc` — a simple X11 color grabber that lets the user click a pixel.
fn pick_with_grabc() -> Result<Color> {
    let output = Command::new("grabc").output().context("grabc not found")?;

    if !output.status.success() {
        anyhow::bail!("grabc failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hex = stdout.trim();
    // grabc outputs "#RRGGBB"
    parse_hex_color(hex)
}

/// Take a screenshot, get mouse position, read pixel.
fn pick_with_screenshot() -> Result<Color> {
    let tmp = std::env::temp_dir().join("mpt-color-picker-screenshot.png");
    let tmp_str = tmp.to_string_lossy();

    // Take screenshot
    let screenshot_ok = Command::new("grim")
        .arg(&*tmp_str)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        || Command::new("gnome-screenshot")
            .args(["-f", &tmp_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        || Command::new("scrot")
            .arg(&*tmp_str)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    if !screenshot_ok {
        anyhow::bail!(
            "could not take screenshot — install grim (Wayland), gnome-screenshot, or scrot (X11)"
        );
    }

    // Get mouse position
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .context("xdotool not found")?;

    let location = String::from_utf8_lossy(&output.stdout);
    let (x, y) = parse_mouse_location(&location)?;
    info!("Mouse position: ({x}, {y})");

    // Read pixel from screenshot
    let img = image::open(&tmp).context("failed to open screenshot")?;
    let pixel = img
        .as_rgba8()
        .context("failed to convert screenshot to RGBA")?;

    if x >= pixel.width() || y >= pixel.height() {
        anyhow::bail!("mouse position ({x}, {y}) out of screenshot bounds");
    }

    let p = pixel.get_pixel(x, y);
    let color = Color::new(p[0], p[1], p[2]);

    // Cleanup
    let _ = std::fs::remove_file(&tmp);

    Ok(color)
}

fn parse_mouse_location(output: &str) -> Result<(u32, u32)> {
    let mut x = None;
    let mut y = None;
    for line in output.lines() {
        if let Some(val) = line.strip_prefix("X=") {
            x = Some(val.parse::<u32>().context("invalid X coordinate")?);
        } else if let Some(val) = line.strip_prefix("Y=") {
            y = Some(val.parse::<u32>().context("invalid Y coordinate")?);
        }
    }
    match (x, y) {
        (Some(x), Some(y)) => Ok((x, y)),
        _ => anyhow::bail!("could not parse mouse location from xdotool output"),
    }
}

fn parse_hex_color(hex: &str) -> Result<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        anyhow::bail!("invalid hex color: #{hex}");
    }
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(Color::new(r, g, b))
}

/// Copy text to clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;

    // Try wl-copy first
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).ok();
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }

    // Fallback to xclip
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to run xclip")?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex() {
        let c = parse_hex_color("#FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn parse_mouse_loc() {
        let input = "X=1234\nY=567\nSCREEN=0\nWINDOW=12345\n";
        let (x, y) = parse_mouse_location(input).unwrap();
        assert_eq!(x, 1234);
        assert_eq!(y, 567);
    }
}
