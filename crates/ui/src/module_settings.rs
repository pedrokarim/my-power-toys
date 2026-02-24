use mpt_common::config::{load_module_config, save_module_config};
use serde::{Deserialize, Serialize};

// ── Color Picker ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorFormatEntry {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

fn default_formats() -> Vec<ColorFormatEntry> {
    vec![
        ColorFormatEntry {
            id: "hex".into(),
            label: "HEX".into(),
            enabled: true,
        },
        ColorFormatEntry {
            id: "rgb".into(),
            label: "RGB".into(),
            enabled: true,
        },
        ColorFormatEntry {
            id: "hsl".into(),
            label: "HSL".into(),
            enabled: true,
        },
        ColorFormatEntry {
            id: "hsv".into(),
            label: "HSV".into(),
            enabled: true,
        },
        ColorFormatEntry {
            id: "cmyk".into(),
            label: "CMYK".into(),
            enabled: true,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPickerConf {
    #[serde(default = "default_hex")]
    pub format: String,
    #[serde(default = "default_pick_and_edit")]
    pub behavior: String,
    #[serde(default = "default_true_cp")]
    pub show_color_name: bool,
    #[serde(default = "default_formats")]
    pub formats: Vec<ColorFormatEntry>,
}

fn default_hex() -> String {
    "hex".into()
}

fn default_pick_and_edit() -> String {
    "pick-and-edit".into()
}

fn default_true_cp() -> bool {
    true
}

impl Default for ColorPickerConf {
    fn default() -> Self {
        Self {
            format: default_hex(),
            behavior: default_pick_and_edit(),
            show_color_name: true,
            formats: default_formats(),
        }
    }
}

// ── Text Extractor ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractorConf {
    #[serde(default = "default_eng")]
    pub language: String,
}

fn default_eng() -> String {
    "eng".into()
}

impl Default for TextExtractorConf {
    fn default() -> Self {
        Self {
            language: default_eng(),
        }
    }
}

// ── Image Resizer ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResizerConf {
    #[serde(default = "default_medium")]
    pub preset: String,
    #[serde(default = "default_original")]
    pub output_format: String,
    #[serde(default = "default_quality")]
    pub quality: u8,
}

fn default_medium() -> String {
    "medium".into()
}

fn default_original() -> String {
    "original".into()
}

fn default_quality() -> u8 {
    85
}

impl Default for ImageResizerConf {
    fn default() -> Self {
        Self {
            preset: default_medium(),
            output_format: default_original(),
            quality: default_quality(),
        }
    }
}

// ── Mouse Utilities ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseUtilsConf {
    #[serde(default = "default_true")]
    pub find_my_mouse: bool,
    #[serde(default)]
    pub click_highlighter: bool,
    #[serde(default)]
    pub crosshair: bool,
    #[serde(default = "default_radius")]
    pub spotlight_radius: u32,
    #[serde(default = "default_color")]
    pub spotlight_color: String,
    #[serde(default = "default_opacity")]
    pub spotlight_opacity: f32,
}

fn default_true() -> bool {
    true
}

fn default_radius() -> u32 {
    100
}

fn default_color() -> String {
    "#FFFF00".into()
}

fn default_opacity() -> f32 {
    0.5
}

impl Default for MouseUtilsConf {
    fn default() -> Self {
        Self {
            find_my_mouse: true,
            click_highlighter: false,
            crosshair: false,
            spotlight_radius: default_radius(),
            spotlight_color: default_color(),
            spotlight_opacity: default_opacity(),
        }
    }
}

// ── App Launcher ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLauncherConf {
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_true")]
    pub show_calculator: bool,
}

fn default_max_results() -> usize {
    8
}

impl Default for AppLauncherConf {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            show_calculator: true,
        }
    }
}

// ── Fancy Zones ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyZonesConf {
    #[serde(default)]
    pub active_layout: usize,
    #[serde(default = "default_gap")]
    pub zone_gap: u32,
}

fn default_gap() -> u32 {
    8
}

impl Default for FancyZonesConf {
    fn default() -> Self {
        Self {
            active_layout: 0,
            zone_gap: default_gap(),
        }
    }
}

// ── Peek ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeekConf {
    #[serde(default = "default_preview_lines")]
    pub max_preview_lines: usize,
    #[serde(default = "default_dir_entries")]
    pub max_dir_entries: usize,
}

fn default_preview_lines() -> usize {
    50
}

fn default_dir_entries() -> usize {
    20
}

impl Default for PeekConf {
    fn default() -> Self {
        Self {
            max_preview_lines: default_preview_lines(),
            max_dir_entries: default_dir_entries(),
        }
    }
}

// ── Command Palette ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPaletteConf {
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_google")]
    pub search_engine: String,
    #[serde(default = "default_true")]
    pub show_provider_tags: bool,
}

fn default_google() -> String {
    "google".into()
}

impl Default for CommandPaletteConf {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            search_engine: default_google(),
            show_provider_tags: true,
        }
    }
}

// ── Light Switch ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSwitchConf {
    #[serde(default = "default_off")]
    pub schedule_mode: String,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    #[serde(default)]
    pub sunrise_offset_min: i32,
    #[serde(default)]
    pub sunset_offset_min: i32,
    #[serde(default = "default_dark_time")]
    pub dark_mode_time: String,
    #[serde(default = "default_light_time")]
    pub light_mode_time: String,
    #[serde(default = "default_true")]
    pub apply_system: bool,
    #[serde(default = "default_true")]
    pub apply_apps: bool,
}

fn default_off() -> String {
    "off".into()
}

fn default_dark_time() -> String {
    "20:00".into()
}

fn default_light_time() -> String {
    "06:00".into()
}

impl Default for LightSwitchConf {
    fn default() -> Self {
        Self {
            schedule_mode: default_off(),
            latitude: 0.0,
            longitude: 0.0,
            sunrise_offset_min: 0,
            sunset_offset_min: 0,
            dark_mode_time: default_dark_time(),
            light_mode_time: default_light_time(),
            apply_system: true,
            apply_apps: true,
        }
    }
}

// ── Key Manager ─────────────────────────────────────────────────────────────

pub type KeyManagerConf = mpt_key_manager::KeyManagerConfig;

// ── Workspaces ──────────────────────────────────────────────────────────────

pub type WorkspacesConf = mpt_workspaces::config::WorkspacesConfig;

// ── Aggregate ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModuleConfigs {
    pub color_picker: ColorPickerConf,
    pub text_extractor: TextExtractorConf,
    pub image_resizer: ImageResizerConf,
    pub mouse_utils: MouseUtilsConf,
    pub app_launcher: AppLauncherConf,
    pub fancy_zones: FancyZonesConf,
    pub peek: PeekConf,
    pub command_palette: CommandPaletteConf,
    pub light_switch: LightSwitchConf,
    pub key_manager: KeyManagerConf,
    pub workspaces: WorkspacesConf,
}

impl ModuleConfigs {
    pub fn load_all() -> Self {
        Self {
            color_picker: load_module_config("color-picker").unwrap_or_default(),
            text_extractor: load_module_config("text-extractor").unwrap_or_default(),
            image_resizer: load_module_config("image-resizer").unwrap_or_default(),
            mouse_utils: load_module_config("mouse-utils").unwrap_or_default(),
            app_launcher: load_module_config("app-launcher").unwrap_or_default(),
            fancy_zones: load_module_config("fancy-zones").unwrap_or_default(),
            peek: load_module_config("peek").unwrap_or_default(),
            command_palette: load_module_config("command-palette").unwrap_or_default(),
            light_switch: load_module_config("light-switch").unwrap_or_default(),
            key_manager: load_module_config("key-manager").unwrap_or_default(),
            workspaces: load_module_config("workspaces").unwrap_or_default(),
        }
    }

    pub fn save(&self, module_id: &str) {
        let _ = match module_id {
            "color-picker" => save_module_config("color-picker", &self.color_picker),
            "text-extractor" => save_module_config("text-extractor", &self.text_extractor),
            "image-resizer" => save_module_config("image-resizer", &self.image_resizer),
            "mouse-utils" => save_module_config("mouse-utils", &self.mouse_utils),
            "app-launcher" => save_module_config("app-launcher", &self.app_launcher),
            "fancy-zones" => save_module_config("fancy-zones", &self.fancy_zones),
            "peek" => save_module_config("peek", &self.peek),
            "command-palette" => save_module_config("command-palette", &self.command_palette),
            "light-switch" => save_module_config("light-switch", &self.light_switch),
            "key-manager" => save_module_config("key-manager", &self.key_manager),
            "workspaces" => save_module_config("workspaces", &self.workspaces),
            _ => Ok(()),
        };
    }
}
