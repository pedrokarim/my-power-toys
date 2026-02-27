use std::path::PathBuf;

use crate::translations::Language;
use crate::types::{FontSize, Page, ThemeMode, ToastKind, VisualTheme};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    ToggleModule(String, bool),
    ToggleModuleResult(String, bool, String),
    RestartModuleResult(()),
    TriggerHotkeyTest(String),
    TriggerHotkeyTestResult(String, String),
    StartHotkeyTest,
    StopHotkeyTest,
    PollKeyboardEvents,
    // Shortcut capture (interactive hotkey recording for settings)
    StartCaptureShortcut(String),
    ConfirmCaptureShortcut,
    CancelCaptureShortcut,
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
    UpdateProgressTick,
    RestartApp,
    StartDaemon,
    StartDaemonResult(Result<(), String>),
    // Module settings
    SetColorPickerFormat(String),
    SetColorPickerBehavior(String),
    ToggleColorPickerShowName(bool),
    ToggleColorFormat(String, bool),
    SetTextExtractorLang(String),
    SetImageResizerPreset(String),
    SetImageResizerFormat(String),
    SetImageResizerQuality(u8),
    // Mouse Utilities — Find My Mouse
    ToggleMouseFindMyMouse(bool),
    SetFindMyMouseActivation(String),
    SetFindMyMouseShakeDistance(String),
    ToggleFindMyMouseGameMode(bool),
    SetFindMyMouseBgColor(String),
    SetFindMyMouseSpotlightColor(String),
    SetFindMyMouseSpotlightRadius(String),
    SetFindMyMouseOverlayOpacity(String),
    SetFindMyMouseInitialZoom(String),
    SetFindMyMouseAnimationMs(String),
    SetFindMyMouseExcludedApps(String),
    // Mouse Utilities — Highlighter
    ToggleMouseClickHighlighter(bool),
    SetHighlighterPrimaryColor(String),
    SetHighlighterSecondaryColor(String),
    SetHighlighterAlwaysColor(String),
    SetHighlighterMode(String),
    SetHighlighterRadius(String),
    SetHighlighterFadeDelay(String),
    SetHighlighterFadeDuration(String),
    // Mouse Utilities — Crosshairs
    ToggleMouseCrosshair(bool),
    SetCrosshairColor(String),
    SetCrosshairOpacity(String),
    SetCrosshairCenterRadius(String),
    SetCrosshairThickness(String),
    SetCrosshairBorderColor(String),
    SetCrosshairBorderSize(String),
    SetCrosshairOrientation(String),
    ToggleCrosshairAutoHide(bool),
    ToggleCrosshairFixedLength(bool),
    SetCrosshairFixedLengthPx(String),
    // Mouse Utilities — Mouse Jump
    ToggleMouseJump(bool),
    SetMouseJumpMaxWidth(String),
    SetMouseJumpMaxHeight(String),
    // Mouse Utilities — Cursor Wrap
    ToggleCursorWrap(bool),
    // Mouse Utilities — Gliding Cursor
    ToggleGlidingCursor(bool),
    SetGlidingCursorTravelSpeed(String),
    SetGlidingCursorDelaySpeed(String),
    SetAppLauncherMaxResults(usize),
    ToggleAppLauncherCalc(bool),
    SetFancyZonesGap(u32),
    SetPeekPreviewLines(usize),
    SetPeekDirEntries(usize),
    // Light Switch
    SetLightSwitchSchedule(String),
    SetLightSwitchLatitude(String),
    SetLightSwitchLongitude(String),
    SetLightSwitchSunriseOffset(String),
    SetLightSwitchSunsetOffset(String),
    SetLightSwitchDarkTime(String),
    SetLightSwitchLightTime(String),
    ToggleLightSwitchSystem(bool),
    ToggleLightSwitchApps(bool),
    // Key Manager
    AddKeyMapping(mpt_key_manager::KeyMapping),
    RemoveKeyMapping(usize),
    ToggleKeyMapping(usize, bool),
    // Awake
    SetAwakeMode(String),
    ToggleAwakeKeepScreen(bool),
    SetAwakeTimedHours(String),
    SetAwakeTimedMinutes(String),
    SetAwakeExpireAt(String),
    // Always on Top
    ToggleAotBorder(bool),
    SetAotBorderThickness(u32),
    ToggleAotSound(bool),
    RemoveAotExcludedApp(usize),
    // Workspaces
    RemoveWorkspace(usize),
    LaunchWorkspace(usize),
    ToggleWorkspaceApp(usize, usize, bool),
    // Hosts Editor
    ToggleHostsEditorShowDisabled(bool),
    ToggleHostsEditorBackup(bool),
    SetHostsEditorPlacement(String),
    // Bulk Rename
    ToggleBulkRenameRegex(bool),
    ToggleBulkRenameMatchAll(bool),
    ToggleBulkRenameCaseSensitive(bool),
    SetBulkRenameApplyTo(String),
    ToggleBulkRenameIncludeFolders(bool),
    ToggleBulkRenameIncludeSubfolders(bool),
    SetBulkRenameTextFormatting(String),
    ToggleBulkRenameEnumerate(bool),
    DemoToast(ToastKind),
    OpenDemoDialog,
    CloseDemoDialog,
    DemoDialogConfirm,
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
