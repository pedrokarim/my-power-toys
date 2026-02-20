use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2};

const ICON_DISPLAY_SIZE: u32 = 32;

pub struct IconCache {
    textures: HashMap<String, Option<TextureHandle>>,
    resolved: HashMap<String, PathBuf>,
    theme_dirs: Vec<PathBuf>,
}

impl IconCache {
    pub fn new() -> Self {
        let theme_dirs = build_theme_dirs();
        tracing::debug!("Icon theme dirs: {theme_dirs:?}");
        Self {
            textures: HashMap::new(),
            resolved: HashMap::new(),
            theme_dirs,
        }
    }

    /// Get or load a texture for an icon name. Resolves path lazily if needed.
    pub fn get(&mut self, ctx: &Context, name: &str) -> Option<egui::ImageSource<'_>> {
        if !self.textures.contains_key(name) {
            // Resolve path lazily
            if !self.resolved.contains_key(name) {
                if let Some(path) = self.find_icon_path(name) {
                    self.resolved.insert(name.to_string(), path);
                }
            }
            let tex = self
                .resolved
                .get(name)
                .and_then(|path| load_icon_texture(ctx, name, path));
            self.textures.insert(name.to_string(), tex);
        }

        self.textures
            .get(name)
            .and_then(|t| t.as_ref())
            .map(|tex| egui::ImageSource::Texture(egui::load::SizedTexture::from_handle(tex)))
    }

    fn find_icon_path(&self, name: &str) -> Option<PathBuf> {
        // Already an absolute path?
        if name.contains('/') {
            let p = Path::new(name);
            if p.exists() {
                return Some(p.to_path_buf());
            }
            for ext in &["png", "svg"] {
                let with_ext = p.with_extension(ext);
                if with_ext.exists() {
                    return Some(with_ext);
                }
            }
            return None;
        }

        // Search through theme dirs — prefer PNG at good size, then SVG
        let sizes = ["48x48", "64x64", "32x32", "256x256", "128x128", "96x96"];
        let extensions = ["png", "svg"];

        for dir in &self.theme_dirs {
            // Sized directories (e.g., 48x48/apps/firefox.png)
            for size in &sizes {
                for ext in &extensions {
                    let path = dir.join(size).join("apps").join(format!("{name}.{ext}"));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }

            // Scalable directory (e.g., scalable/apps/firefox.svg)
            for ext in &extensions {
                let path = dir.join("scalable").join("apps").join(format!("{name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }

            // Flat layout (e.g., apps/firefox.png)
            for ext in &extensions {
                let path = dir.join("apps").join(format!("{name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // Pixmaps fallback
        for ext in &["png", "svg", "xpm"] {
            let path = PathBuf::from(format!("/usr/share/pixmaps/{name}.{ext}"));
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

fn build_theme_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Detect current icon theme from GTK settings
    let detected_theme = detect_gtk_icon_theme();

    // Build ordered list of themes to search
    let mut themes: Vec<String> = Vec::new();
    if let Some(ref t) = detected_theme {
        themes.push(t.clone());
    }
    // Common fallbacks
    for fallback in ["Adwaita", "Yaru", "Papirus", "breeze", "gnome", "hicolor"] {
        if !themes.iter().any(|t| t == fallback) {
            themes.push(fallback.to_string());
        }
    }

    // XDG data dirs
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/share:/usr/local/share".to_string());

    // User local icons
    if let Some(home) = dirs::home_dir() {
        let local = home.join(".local/share/icons");
        if local.is_dir() {
            for theme in &themes {
                let d = local.join(theme);
                if d.is_dir() {
                    dirs.push(d);
                }
            }
        }
    }

    for data_dir in data_dirs.split(':') {
        let icons_base = Path::new(data_dir).join("icons");
        if !icons_base.is_dir() {
            continue;
        }
        for theme in &themes {
            let d = icons_base.join(theme);
            if d.is_dir() {
                dirs.push(d);
            }
        }
    }

    dirs
}

fn detect_gtk_icon_theme() -> Option<String> {
    // Try GTK3 settings.ini
    if let Some(config) = dirs::config_dir() {
        let gtk3 = config.join("gtk-3.0/settings.ini");
        if let Ok(content) = std::fs::read_to_string(&gtk3) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("gtk-icon-theme-name") {
                    let val = val.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
                    let val = val.trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn load_icon_texture(ctx: &Context, name: &str, path: &Path) -> Option<TextureHandle> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "png" => load_png(ctx, name, path),
        "svg" => load_svg(ctx, name, path),
        _ => None,
    }
}

fn load_png(ctx: &Context, name: &str, path: &Path) -> Option<TextureHandle> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?;

    let resized = img.resize_exact(
        ICON_DISPLAY_SIZE,
        ICON_DISPLAY_SIZE,
        image::imageops::FilterType::Triangle,
    );
    let rgba = resized.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();

    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
    Some(ctx.load_texture(
        format!("icon-{name}"),
        color_image,
        TextureOptions::LINEAR,
    ))
}

fn load_svg(ctx: &Context, name: &str, path: &Path) -> Option<TextureHandle> {
    let data = std::fs::read(path).ok()?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default()).ok()?;

    let size = ICON_DISPLAY_SIZE;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;

    let svg_size = tree.size();
    let sx = size as f32 / svg_size.width();
    let sy = size as f32 / svg_size.height();
    let scale = sx.min(sy);

    let tx = (size as f32 - svg_size.width() * scale) / 2.0;
    let ty = (size as f32 - svg_size.height() * scale) / 2.0;
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let pixels = pixmap.data().to_vec();
    let color_image = ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &pixels);
    Some(ctx.load_texture(
        format!("icon-{name}"),
        color_image,
        TextureOptions::LINEAR,
    ))
}

/// Size to use when displaying icons via ui.image().
pub fn icon_size() -> Vec2 {
    Vec2::splat(ICON_DISPLAY_SIZE as f32)
}
