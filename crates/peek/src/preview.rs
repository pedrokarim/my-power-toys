use anyhow::{Result, anyhow};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported file categories for preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Text,
    Image,
    Pdf,
    Audio,
    Video,
    Directory,
    Unknown,
}

impl fmt::Display for FileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileKind::Text => write!(f, "Text"),
            FileKind::Image => write!(f, "Image"),
            FileKind::Pdf => write!(f, "PDF"),
            FileKind::Audio => write!(f, "Audio"),
            FileKind::Video => write!(f, "Video"),
            FileKind::Directory => write!(f, "Directory"),
            FileKind::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Metadata + preview data for a file.
#[derive(Debug, Clone)]
pub struct FilePreview {
    pub path: PathBuf,
    pub kind: FileKind,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub preview_text: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<String>,
}

/// Detect what kind of file we're dealing with based on extension.
pub fn detect_kind(path: &Path) -> FileKind {
    if path.is_dir() {
        return FileKind::Directory;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Text
        "txt" | "md" | "rs" | "py" | "js" | "ts" | "json" | "toml" | "yaml" | "yml" | "xml"
        | "html" | "css" | "sh" | "bash" | "zsh" | "c" | "cpp" | "h" | "hpp" | "java" | "go"
        | "rb" | "lua" | "sql" | "csv" | "log" | "conf" | "cfg" | "ini" | "env" | "dockerfile"
        | "makefile" => FileKind::Text,
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif" => {
            FileKind::Image
        }
        // PDF
        "pdf" => FileKind::Pdf,
        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus" => FileKind::Audio,
        // Video
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" | "flv" | "m4v" => FileKind::Video,
        _ => FileKind::Unknown,
    }
}

/// Generate a preview for a file.
pub fn generate_preview(
    path: &Path,
    max_preview_lines: usize,
    max_dir_entries: usize,
) -> Result<FilePreview> {
    if !path.exists() {
        return Err(anyhow!("file not found: {}", path.display()));
    }

    let kind = detect_kind(path);
    let metadata = std::fs::metadata(path)?;
    let size_bytes = metadata.len();
    let mime_type = detect_mime(path);

    let mut preview = FilePreview {
        path: path.to_path_buf(),
        kind,
        size_bytes,
        mime_type,
        preview_text: None,
        dimensions: None,
        duration: None,
    };

    match kind {
        FileKind::Text => {
            preview.preview_text = Some(read_text_preview(path, max_preview_lines)?);
        }
        FileKind::Image => {
            preview.dimensions = get_image_dimensions(path);
        }
        FileKind::Pdf => {
            preview.preview_text = get_pdf_info(path);
        }
        FileKind::Audio | FileKind::Video => {
            preview.duration = get_media_duration(path);
        }
        FileKind::Directory => {
            preview.preview_text = Some(list_directory(path, max_dir_entries)?);
        }
        FileKind::Unknown => {}
    }

    Ok(preview)
}

/// Read the first N lines of a text file.
fn read_text_preview(path: &Path, max_lines: usize) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let preview = lines.join("\n");
    if content.lines().count() > max_lines {
        Ok(format!(
            "{preview}\n... ({} more lines)",
            content.lines().count() - max_lines
        ))
    } else {
        Ok(preview)
    }
}

/// List directory contents (first N entries).
fn list_directory(path: &Path, max_entries: usize) -> Result<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut total = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        total += 1;
        if entries.len() < max_entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let kind = if entry.file_type()?.is_dir() {
                "dir"
            } else {
                "file"
            };
            entries.push(format!("[{kind}] {name}"));
        }
    }

    let mut result = entries.join("\n");
    if total > max_entries {
        result.push_str(&format!("\n... ({} more entries)", total - max_entries));
    }
    Ok(result)
}

/// Detect MIME type using `file --mime-type`.
fn detect_mime(path: &Path) -> Option<String> {
    let output = Command::new("file")
        .args(["--mime-type", "-b"])
        .arg(path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get image dimensions using `identify` (ImageMagick) or `file`.
fn get_image_dimensions(path: &Path) -> Option<(u32, u32)> {
    // Try identify (ImageMagick)
    if let Ok(output) = Command::new("identify")
        .args(["-format", "%w %h"])
        .arg(path)
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2
            && let (Ok(w), Ok(h)) = (parts[0].parse(), parts[1].parse())
        {
            return Some((w, h));
        }
    }

    // Fallback: parse `file` output for common formats
    if let Ok(output) = Command::new("file").arg(path).output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        // Try to find "WxH" pattern like "1920 x 1080"
        for part in text.split(", ") {
            let cleaned = part.replace(" x ", "x");
            if let Some((w_str, h_str)) = cleaned.split_once('x')
                && let (Ok(w), Ok(h)) = (w_str.trim().parse::<u32>(), h_str.trim().parse::<u32>())
                && w > 0
                && h > 0
                && w < 100000
                && h < 100000
            {
                return Some((w, h));
            }
        }
    }

    None
}

/// Get PDF info using `pdfinfo`.
fn get_pdf_info(path: &Path) -> Option<String> {
    let output = Command::new("pdfinfo").arg(path).output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get media duration using `ffprobe`.
fn get_media_duration(path: &Path) -> Option<String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .ok()?;

    if output.status.success() {
        let secs: f64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .ok()?;
        let mins = (secs / 60.0).floor() as u64;
        let remaining = (secs % 60.0).floor() as u64;
        Some(format!("{mins}:{remaining:02}"))
    } else {
        None
    }
}

/// Format file size in human-readable form.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_text_files() {
        assert_eq!(detect_kind(Path::new("file.rs")), FileKind::Text);
        assert_eq!(detect_kind(Path::new("README.md")), FileKind::Text);
        assert_eq!(detect_kind(Path::new("config.toml")), FileKind::Text);
        assert_eq!(detect_kind(Path::new("script.py")), FileKind::Text);
    }

    #[test]
    fn detect_image_files() {
        assert_eq!(detect_kind(Path::new("photo.png")), FileKind::Image);
        assert_eq!(detect_kind(Path::new("photo.jpg")), FileKind::Image);
        assert_eq!(detect_kind(Path::new("icon.svg")), FileKind::Image);
    }

    #[test]
    fn detect_media_files() {
        assert_eq!(detect_kind(Path::new("song.mp3")), FileKind::Audio);
        assert_eq!(detect_kind(Path::new("video.mp4")), FileKind::Video);
        assert_eq!(detect_kind(Path::new("movie.mkv")), FileKind::Video);
    }

    #[test]
    fn detect_pdf() {
        assert_eq!(detect_kind(Path::new("doc.pdf")), FileKind::Pdf);
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(detect_kind(Path::new("data.xyz")), FileKind::Unknown);
        assert_eq!(detect_kind(Path::new("binary.bin")), FileKind::Unknown);
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1_500_000), "1.4 MB");
        assert_eq!(format_size(2_500_000_000), "2.3 GB");
    }

    #[test]
    fn file_kind_display() {
        assert_eq!(format!("{}", FileKind::Text), "Text");
        assert_eq!(format!("{}", FileKind::Image), "Image");
        assert_eq!(format!("{}", FileKind::Pdf), "PDF");
    }
}
