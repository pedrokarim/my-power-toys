use anyhow::{Context, Result};
use std::process::Command;

/// Get plain text from the clipboard.
/// Tries wl-paste (Wayland) first, falls back to xclip (X11).
pub fn get_text() -> Result<String> {
    // Try Wayland first
    if let Ok(output) = Command::new("wl-paste")
        .arg("--no-newline")
        .arg("--type")
        .arg("text/plain")
        .output()
        && output.status.success()
    {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    // Fall back to xclip
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .context("failed to run xclip — is xclip or wl-clipboard installed?")?;

    if !output.status.success() {
        anyhow::bail!("xclip failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Set plain text to the clipboard (replacing any rich content).
/// Tries wl-copy (Wayland) first, falls back to xclip (X11).
pub fn set_text(text: &str) -> Result<()> {
    // Try Wayland first
    if let Ok(mut child) = Command::new("wl-copy")
        .arg("--type")
        .arg("text/plain")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).ok();
        }
        let status = child.wait()?;
        if status.success() {
            return Ok(());
        }
    }

    // Fall back to xclip
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to run xclip")?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("xclip failed to set clipboard");
    }

    Ok(())
}

/// Simulate a Ctrl+V keypress via xdotool.
pub fn simulate_paste() -> Result<()> {
    let status = Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+v"])
        .status()
        .context("failed to run xdotool — is xdotool installed?")?;

    if !status.success() {
        anyhow::bail!("xdotool failed to simulate paste");
    }

    Ok(())
}
