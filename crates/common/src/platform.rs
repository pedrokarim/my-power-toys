use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

/// Detect the current display server.
pub fn detect_display_server() -> DisplayServer {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        DisplayServer::Wayland
    } else if env::var("DISPLAY").is_ok() {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

/// Get the current desktop environment name (e.g. "GNOME", "KDE", "Hyprland").
pub fn detect_desktop_environment() -> Option<String> {
    env::var("XDG_CURRENT_DESKTOP").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_does_not_panic() {
        // Just ensure detection doesn't crash regardless of environment
        let _ds = detect_display_server();
        let _de = detect_desktop_environment();
    }
}
