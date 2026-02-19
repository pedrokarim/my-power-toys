use crate::color::Color;
use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

/// Pick a color from the screen (headless fallback mode).
///
/// Strategy:
/// 1. Try `grabc` (simple X11 color grabber)
/// 2. Fallback: screenshot + mouse position
pub fn pick_color() -> Result<Color> {
    if let Ok(color) = pick_with_grabc() {
        return Ok(color);
    }
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
    Color::from_hex(hex)
}

/// Take a screenshot, get mouse position, read pixel.
fn pick_with_screenshot() -> Result<Color> {
    let screenshot = crate::screenshot::capture_fullscreen()?;

    // Get mouse position
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .context("xdotool not found")?;

    let location = String::from_utf8_lossy(&output.stdout);
    let (x, y) = parse_mouse_location(&location)?;
    info!("Mouse position: ({x}, {y})");

    if x >= screenshot.width() || y >= screenshot.height() {
        anyhow::bail!("mouse position ({x}, {y}) out of screenshot bounds");
    }

    let p = screenshot.get_pixel(x, y);
    Ok(Color::new(p[0], p[1], p[2]))
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

/// Copy text to clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;

    // Try wl-copy first (Wayland)
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

    // Fallback to xclip (X11)
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to run xclip — install wl-copy (Wayland) or xclip (X11)")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mouse_loc() {
        let input = "X=1234\nY=567\nSCREEN=0\nWINDOW=12345\n";
        let (x, y) = parse_mouse_location(input).unwrap();
        assert_eq!(x, 1234);
        assert_eq!(y, 567);
    }
}
