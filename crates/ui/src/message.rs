use std::path::PathBuf;

use crate::translations::Language;
use crate::types::{FontSize, Page, ThemeMode, VisualTheme};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    ToggleModule(String, bool),
    ToggleModuleResult(String, bool, String),
    TriggerHotkeyTest(String),
    TriggerHotkeyTestResult(String, String),
    ToggleDependencyHelp(String),
    CloseDependencyHelp,
    CopyInstallCommand(String),
    DismissToast,
    ToastTick,
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
    CheckForUpdates,
    UpdateCheckFinished(UpdateCheckResult),
    OpenUpdateDialog,
    CloseUpdateDialog,
    ConfirmUpdateInstall,
    UpdateInstallFinished(UpdateInstallResult),
}

#[derive(Debug, Clone)]
pub struct DaemonStateResult {
    pub connected: bool,
    pub modules: Vec<(String, String, bool)>,
}

#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    UpToDate,
    Available(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum UpdateInstallResult {
    Updated(String),
    AlreadyUpToDate,
    Error(String),
}
