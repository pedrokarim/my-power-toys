use std::path::PathBuf;

use crate::types::{FontSize, Page, ThemeMode, VisualTheme};
use crate::translations::Language;

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    ToggleModule(String, bool),
    ToggleModuleResult(String, bool, String),
    SetThemeMode(ThemeMode),
    SetLanguage(Language),
    SetFontSize(FontSize),
    ToggleHighContrast(bool),
    ToggleBoldText(bool),
    ToggleCompactLayout(bool),
    ToggleReducedMotion(bool),
    SystemThemeCheck,
    DaemonPoll,
    DaemonState(DaemonStateResult),
    SetVisualTheme(VisualTheme),
    PickCustomImage,
    CustomImagePicked(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct DaemonStateResult {
    pub connected: bool,
    pub modules: Vec<(String, String, bool)>,
}
