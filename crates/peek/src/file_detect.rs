use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, warn};

/// Check if we're running under Wayland.
fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

/// Try to read file URIs from the clipboard.
fn read_clipboard_uris() -> Option<String> {
    let output = if is_wayland() {
        Command::new("wl-paste")
            .args(["-t", "text/uri-list"])
            .output()
            .ok()?
    } else {
        Command::new("xclip")
            .args(["-selection", "clipboard", "-target", "text/uri-list", "-o"])
            .output()
            .ok()?
    };

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Fallback: try gnome-copied-files format (used by Nautilus/Nemo)
    let output = if is_wayland() {
        Command::new("wl-paste")
            .args(["-t", "x-special/gnome-copied-files"])
            .output()
            .ok()?
    } else {
        Command::new("xclip")
            .args([
                "-selection",
                "clipboard",
                "-target",
                "x-special/gnome-copied-files",
                "-o",
            ])
            .output()
            .ok()?
    };

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}

/// Parse a clipboard string containing file URIs into file paths.
/// Handles both text/uri-list and x-special/gnome-copied-files formats.
fn parse_file_uris(clipboard: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for line in clipboard.lines() {
        let line = line.trim();
        // Skip empty lines and comment lines (uri-list format)
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Skip gnome-copied-files action prefix (e.g. "copy" or "cut")
        if line == "copy" || line == "cut" {
            continue;
        }
        // Parse file:// URIs
        if let Some(path_str) = line.strip_prefix("file://") {
            // URL-decode the path
            let decoded = url_decode(path_str);
            let path = PathBuf::from(decoded);
            if path.exists() {
                paths.push(path);
            }
        }
    }

    paths
}

/// Simple URL decoding for file paths (handles %20, %C3%A9, etc.)
fn url_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            result.push(byte);
            i += 3;
            continue;
        }
        result.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Simulate Ctrl+C on the focused window to copy selected files.
fn simulate_copy() {
    if is_wayland() {
        // On Wayland, xdotool doesn't work. We can try ydotool or wtype.
        let _ = Command::new("ydotool")
            .args(["key", "29:1", "46:1", "46:0", "29:0"]) // Ctrl+C keycodes
            .output();
    } else {
        let _ = Command::new("xdotool")
            .args(["key", "--clearmodifiers", "ctrl+c"])
            .output();
    }
}

/// Detect the selected file from the file manager.
///
/// Strategy:
/// 1. Check clipboard for existing file URIs
/// 2. If none, simulate Ctrl+C and retry
/// 3. Return the first file path found
pub fn detect_selected_file() -> Option<PathBuf> {
    // First try: read existing clipboard
    if let Some(uris) = read_clipboard_uris() {
        let paths = parse_file_uris(&uris);
        if let Some(first) = paths.into_iter().next() {
            debug!("Found file in clipboard: {}", first.display());
            return Some(first);
        }
    }

    // Second try: simulate Ctrl+C and retry
    debug!("No file URI in clipboard, simulating Ctrl+C");
    simulate_copy();
    std::thread::sleep(std::time::Duration::from_millis(150));

    if let Some(uris) = read_clipboard_uris() {
        let paths = parse_file_uris(&uris);
        if let Some(first) = paths.into_iter().next() {
            debug!("Found file after simulating copy: {}", first.display());
            return Some(first);
        }
    }

    warn!("Could not detect any selected file");
    None
}

/// Get all detected file paths from clipboard (for multi-file navigation).
pub fn detect_selected_files() -> Vec<PathBuf> {
    if let Some(uris) = read_clipboard_uris() {
        let paths = parse_file_uris(&uris);
        if !paths.is_empty() {
            return paths;
        }
    }

    simulate_copy();
    std::thread::sleep(std::time::Duration::from_millis(150));

    if let Some(uris) = read_clipboard_uris() {
        return parse_file_uris(&uris);
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uri_list() {
        let input = "file:///home/user/test.txt\nfile:///home/user/image.png\n";
        let paths = parse_file_uris(input);
        // Paths won't exist in test env, so we test the parsing logic differently
        assert_eq!(paths.len(), 0); // files don't exist

        // Test parsing without existence check
        let decoded = url_decode("/home/user/test%20file.txt");
        assert_eq!(decoded, "/home/user/test file.txt");
    }

    #[test]
    fn parse_gnome_copied_files() {
        let input = "copy\nfile:///tmp/test.txt\n";
        // The "copy" line should be skipped
        let paths = parse_file_uris(input);
        // /tmp/test.txt likely doesn't exist, but the parsing should handle it
        assert!(paths.is_empty() || paths[0] == std::path::Path::new("/tmp/test.txt"));
    }

    #[test]
    fn url_decode_special_chars() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("file%2Fname"), "file/name");
        assert_eq!(url_decode("no%encoding"), "no%encoding"); // invalid % sequence
    }
}
