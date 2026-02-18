use mpt_common::platform::DisplayServer;
use std::process::Command;
use std::process::Stdio;

pub fn detect_system_dark() -> bool {
    Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("dark"))
        .unwrap_or(true)
}

pub fn load_cjk_font() -> Option<Vec<u8>> {
    [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ]
    .iter()
    .find_map(|p| std::fs::read(p).ok())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Apt,
    Pacman,
    Dnf,
    Zypper,
    Unknown,
}

pub fn detect_display_server() -> DisplayServer {
    mpt_common::platform::detect_display_server()
}

pub fn display_server_label(ds: DisplayServer) -> &'static str {
    match ds {
        DisplayServer::Wayland => "Wayland",
        DisplayServer::X11 => "X11",
        DisplayServer::Unknown => "Unknown",
    }
}

pub fn detect_distro_name() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .or_else(|| line.strip_prefix("NAME="))
                    .map(unquote)
            })
        })
        .unwrap_or_else(|| "Linux".to_string())
}

pub fn detect_package_manager() -> PackageManager {
    let lower = std::fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .to_lowercase();

    if lower.contains("id=ubuntu")
        || lower.contains("id=debian")
        || lower.contains("id_like=debian")
    {
        return PackageManager::Apt;
    }
    if lower.contains("id=arch") || lower.contains("id_like=arch") {
        return PackageManager::Pacman;
    }
    if lower.contains("id=fedora")
        || lower.contains("id=rhel")
        || lower.contains("id=centos")
        || lower.contains("id_like=\"rhel fedora\"")
    {
        return PackageManager::Dnf;
    }
    if lower.contains("id=opensuse") || lower.contains("id=sles") || lower.contains("id_like=suse")
    {
        return PackageManager::Zypper;
    }

    if has_command("apt") {
        PackageManager::Apt
    } else if has_command("pacman") {
        PackageManager::Pacman
    } else if has_command("dnf") {
        PackageManager::Dnf
    } else if has_command("zypper") {
        PackageManager::Zypper
    } else {
        PackageManager::Unknown
    }
}

fn has_command(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn unquote(s: &str) -> String {
    s.trim_matches('"').to_string()
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write;

    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }

    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }

    Err("clipboard command not available (wl-copy/xclip)".to_string())
}
