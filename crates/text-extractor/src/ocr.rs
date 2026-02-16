use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

/// Capture a screen region and run OCR on it.
///
/// Flow:
/// 1. Let user select a region via screenshot tool (gnome-screenshot -a, grim -g "$(slurp)", etc.)
/// 2. Run tesseract on the captured image
/// 3. Return the extracted text
pub fn capture_and_extract(lang: &str) -> Result<String> {
    let tmp_img = std::env::temp_dir().join("mpt-ocr-capture.png");
    let tmp_txt = std::env::temp_dir().join("mpt-ocr-output");
    let tmp_img_str = tmp_img.to_string_lossy();
    let tmp_txt_str = tmp_txt.to_string_lossy();

    // Step 1: Capture a region
    let captured = capture_region(&tmp_img_str)?;
    if !captured {
        anyhow::bail!("screen capture cancelled or failed");
    }

    // Step 2: Run tesseract OCR
    let status = Command::new("tesseract")
        .args([&*tmp_img_str, &*tmp_txt_str, "-l", lang])
        .status()
        .context("failed to run tesseract — is tesseract-ocr installed?")?;

    if !status.success() {
        anyhow::bail!("tesseract failed with exit code {:?}", status.code());
    }

    // Step 3: Read the output (tesseract appends .txt)
    let output_file = format!("{tmp_txt_str}.txt");
    let text = std::fs::read_to_string(&output_file)
        .with_context(|| format!("failed to read tesseract output: {output_file}"))?;

    // Cleanup
    let _ = std::fs::remove_file(&tmp_img);
    let _ = std::fs::remove_file(&output_file);

    let text = text.trim().to_string();
    info!("OCR extracted: {} characters", text.len());
    Ok(text)
}

/// Capture a screen region to a file.
fn capture_region(output_path: &str) -> Result<bool> {
    // Try Wayland first: grim + slurp
    if let Ok(slurp_output) = Command::new("slurp").output()
        && slurp_output.status.success()
    {
        let geometry = String::from_utf8_lossy(&slurp_output.stdout);
        let geometry = geometry.trim();
        if let Ok(s) = Command::new("grim")
            .args(["-g", geometry, output_path])
            .status()
            && s.success()
        {
            return Ok(true);
        }
    }

    // Try gnome-screenshot with area selection
    if let Ok(s) = Command::new("gnome-screenshot")
        .args(["-a", "-f", output_path])
        .status()
        && s.success()
    {
        return Ok(true);
    }

    // Try scrot with select
    if let Ok(s) = Command::new("scrot").args(["-s", output_path]).status()
        && s.success()
    {
        return Ok(true);
    }

    anyhow::bail!(
        "no screenshot tool available — install grim+slurp (Wayland) or gnome-screenshot/scrot (X11)"
    )
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
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}
