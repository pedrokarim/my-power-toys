use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2};

const ICON_DISPLAY_SIZE: u32 = 24;

// ── Desktop file resolver ────────────────────────────────────────────────────

/// Maps WM_CLASS / exec names to icon theme names using `.desktop` files.
struct DesktopResolver {
    /// StartupWMClass (lowercased) → Icon value
    wm_class_map: HashMap<String, String>,
    /// Desktop file stem (lowercased) → Icon value
    desktop_name_map: HashMap<String, String>,
    /// Exec basename (lowercased) → Icon value
    exec_map: HashMap<String, String>,
}

impl DesktopResolver {
    fn new() -> Self {
        let mut resolver = Self {
            wm_class_map: HashMap::new(),
            desktop_name_map: HashMap::new(),
            exec_map: HashMap::new(),
        };
        resolver.scan_desktop_dirs();
        tracing::debug!(
            "DesktopResolver: {} wm_class, {} desktop_name, {} exec mappings",
            resolver.wm_class_map.len(),
            resolver.desktop_name_map.len(),
            resolver.exec_map.len()
        );
        resolver
    }

    fn scan_desktop_dirs(&mut self) {
        let mut dirs: Vec<PathBuf> = Vec::new();

        // Standard XDG directories
        let data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/share:/usr/local/share".to_string());
        for d in data_dirs.split(':') {
            dirs.push(PathBuf::from(d).join("applications"));
        }

        // User local
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
        }

        // Snap desktop entries (with resolved ${SNAP} paths)
        dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

        // Flatpak exports
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        for dir in &dirs {
            if !dir.is_dir() {
                continue;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "desktop") {
                    self.parse_desktop_file(&path);
                }
            }
        }
    }

    fn parse_desktop_file(&mut self, path: &Path) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut icon = None;
        let mut startup_wm_class = None;
        let mut exec = None;
        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }
            if line.starts_with('[') {
                if in_desktop_entry {
                    break; // Past [Desktop Entry] section
                }
                continue;
            }
            if !in_desktop_entry {
                continue;
            }

            if let Some(val) = line.strip_prefix("Icon=") {
                icon = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("StartupWMClass=") {
                startup_wm_class = Some(val.trim().to_string());
            } else if exec.is_none()
                && let Some(val) = line.strip_prefix("Exec=")
            {
                let first_token = val.split_whitespace().next().unwrap_or("");
                exec = Some(first_token.to_string());
            }
        }

        let icon = match icon {
            Some(i) if !i.is_empty() => i,
            _ => return,
        };

        // Map StartupWMClass → icon
        if let Some(wmc) = &startup_wm_class {
            let key = wmc.to_lowercase();
            if !key.is_empty() {
                self.wm_class_map.entry(key).or_insert_with(|| icon.clone());
            }
        }

        // Map desktop file stem → icon (e.g. "github-desktop.desktop" → "github-desktop")
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let key = stem.to_lowercase();
            // For snap entries like "code_code.desktop", also add the part after underscore
            if let Some(after_underscore) = key.split('_').next_back()
                && after_underscore != key
            {
                self.desktop_name_map
                    .entry(after_underscore.to_string())
                    .or_insert_with(|| icon.clone());
            }
            self.desktop_name_map
                .entry(key)
                .or_insert_with(|| icon.clone());
        }

        // Map exec basename → icon
        if let Some(exec_path) = &exec
            && let Some(basename) = Path::new(exec_path).file_name().and_then(|n| n.to_str())
        {
            let key = basename.to_lowercase();
            if !key.is_empty() {
                self.exec_map.entry(key).or_insert_with(|| icon.clone());
            }
        }
    }

    /// Resolve the best icon name for a given WM_CLASS and exec path.
    fn resolve(&self, wm_class: &str, exec: &str) -> Option<String> {
        let wmc_lower = wm_class.to_lowercase();
        let wmc_normalized = wmc_lower.replace(' ', "-");

        // 1. Try StartupWMClass match
        if let Some(icon) = self.wm_class_map.get(&wmc_lower) {
            return Some(icon.clone());
        }

        // 2. Try desktop file name match (exact)
        if let Some(icon) = self.desktop_name_map.get(&wmc_lower) {
            return Some(icon.clone());
        }

        // 3. Try normalized (spaces → hyphens)
        if wmc_normalized != wmc_lower
            && let Some(icon) = self.desktop_name_map.get(&wmc_normalized)
        {
            return Some(icon.clone());
        }

        // 4. Try exec basename
        if !exec.is_empty()
            && let Some(basename) = Path::new(exec).file_name().and_then(|n| n.to_str())
        {
            let key = basename.to_lowercase();
            if let Some(icon) = self.exec_map.get(&key) {
                return Some(icon.clone());
            }
        }

        None
    }
}

// ── Icon cache ───────────────────────────────────────────────────────────────

pub struct IconCache {
    textures: HashMap<String, Option<TextureHandle>>,
    resolved: HashMap<String, PathBuf>,
    theme_dirs: Vec<PathBuf>,
    desktop_resolver: DesktopResolver,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

impl IconCache {
    pub fn new() -> Self {
        let theme_dirs = build_theme_dirs();
        tracing::debug!("Icon theme dirs: {theme_dirs:?}");
        Self {
            textures: HashMap::new(),
            resolved: HashMap::new(),
            theme_dirs,
            desktop_resolver: DesktopResolver::new(),
        }
    }

    /// Get icon for an app identified by WM_CLASS and exec path.
    /// Uses `.desktop` files to resolve the correct icon theme name.
    pub fn get_for_app(
        &mut self,
        ctx: &Context,
        wm_class: &str,
        exec: &str,
    ) -> Option<egui::ImageSource<'_>> {
        let icon_name = self
            .desktop_resolver
            .resolve(wm_class, exec)
            .unwrap_or_else(|| wm_class.to_lowercase());

        self.get(ctx, &icon_name)
    }

    /// Get or load a texture for an icon name. Resolves path lazily if needed.
    pub fn get(&mut self, ctx: &Context, name: &str) -> Option<egui::ImageSource<'_>> {
        if !self.textures.contains_key(name) {
            // Resolve path lazily
            if !self.resolved.contains_key(name)
                && let Some(path) = self.find_icon_path(name)
            {
                self.resolved.insert(name.to_string(), path);
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
            for size in &sizes {
                for ext in &extensions {
                    let path = dir.join(size).join("apps").join(format!("{name}.{ext}"));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }

            for ext in &extensions {
                let path = dir
                    .join("scalable")
                    .join("apps")
                    .join(format!("{name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }

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

    let detected_theme = detect_gtk_icon_theme();

    let mut themes: Vec<String> = Vec::new();
    if let Some(ref t) = detected_theme {
        themes.push(t.clone());
    }
    for fallback in ["Adwaita", "Yaru", "Papirus", "breeze", "gnome", "hicolor"] {
        if !themes.iter().any(|t| t == fallback) {
            themes.push(fallback.to_string());
        }
    }

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/share:/usr/local/share".to_string());

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
    Some(ctx.load_texture(format!("icon-{name}"), color_image, TextureOptions::LINEAR))
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
    Some(ctx.load_texture(format!("icon-{name}"), color_image, TextureOptions::LINEAR))
}

/// Size to use when displaying icons.
pub fn icon_size() -> Vec2 {
    Vec2::splat(ICON_DISPLAY_SIZE as f32)
}
