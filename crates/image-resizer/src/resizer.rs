use anyhow::{Context, Result};
use image::DynamicImage;
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResizePreset {
    Small,  // 640px wide
    Medium, // 1280px wide
    Large,  // 1920px wide
    Phone,  // 1080px wide (portrait)
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Original,
    Png,
    Jpeg,
    Webp,
}

impl ResizePreset {
    pub fn max_width(self) -> u32 {
        match self {
            Self::Small => 640,
            Self::Medium => 1280,
            Self::Large => 1920,
            Self::Phone => 1080,
            Self::Custom => 0,
        }
    }
}

/// Resize a single image.
pub fn resize_image(
    input: &Path,
    output_dir: &Path,
    max_width: u32,
    format: OutputFormat,
    quality: u8,
) -> Result<PathBuf> {
    let img = image::open(input).with_context(|| format!("failed to open {}", input.display()))?;

    let resized = if img.width() > max_width && max_width > 0 {
        let ratio = max_width as f64 / img.width() as f64;
        let new_height = (img.height() as f64 * ratio) as u32;
        img.resize(max_width, new_height, FilterType::Lanczos3)
    } else {
        img
    };

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let ext = match format {
        OutputFormat::Original => input.extension().and_then(|e| e.to_str()).unwrap_or("png"),
        OutputFormat::Png => "png",
        OutputFormat::Jpeg => "jpg",
        OutputFormat::Webp => "webp",
    };

    let output_path = output_dir.join(format!("{stem}_resized.{ext}"));
    save_image(&resized, &output_path, format, quality)?;

    info!(
        "Resized {} -> {} ({}x{})",
        input.display(),
        output_path.display(),
        resized.width(),
        resized.height()
    );

    Ok(output_path)
}

/// Resize multiple images.
pub fn resize_batch(
    inputs: &[PathBuf],
    output_dir: &Path,
    max_width: u32,
    format: OutputFormat,
    quality: u8,
) -> Vec<Result<PathBuf>> {
    std::fs::create_dir_all(output_dir).ok();
    inputs
        .iter()
        .map(|input| resize_image(input, output_dir, max_width, format, quality))
        .collect()
}

fn save_image(img: &DynamicImage, path: &Path, format: OutputFormat, _quality: u8) -> Result<()> {
    match format {
        OutputFormat::Jpeg => {
            let rgb = img.to_rgb8();
            rgb.save(path)
                .with_context(|| format!("failed to save {}", path.display()))?;
        }
        _ => {
            img.save(path)
                .with_context(|| format!("failed to save {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_widths() {
        assert_eq!(ResizePreset::Small.max_width(), 640);
        assert_eq!(ResizePreset::Medium.max_width(), 1280);
        assert_eq!(ResizePreset::Large.max_width(), 1920);
        assert_eq!(ResizePreset::Phone.max_width(), 1080);
    }
}
