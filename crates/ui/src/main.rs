mod daemon;
mod helpers;
mod message;
mod module_settings;
mod modules;
mod persistence;
mod theme;
mod translations;
mod types;
mod views;
mod widgets;

use iced::theme::Palette;
use iced::time;
use iced::{Color, Subscription, Task, Theme};
use iced_fonts::BOOTSTRAP_FONT_BYTES;
use message::*;
use mpt_common::platform::DisplayServer;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, Instant};
use translations::Language;
use types::*;

// ── State ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ToastNotification {
    kind: ToastKind,
    title: String,
    message: String,
    created_at: Instant,
    expires_at: Instant,
}

impl ToastNotification {
    fn new(
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        duration: Duration,
    ) -> Self {
        let now = Instant::now();
        Self {
            kind,
            title: title.into(),
            message: message.into(),
            created_at: now,
            expires_at: now + duration,
        }
    }

    fn remaining_progress(&self, now: Instant) -> f32 {
        let total = self
            .expires_at
            .duration_since(self.created_at)
            .as_secs_f32();
        if total <= f32::EPSILON {
            return 0.0;
        }
        let elapsed = now.saturating_duration_since(self.created_at).as_secs_f32();
        (1.0 - elapsed / total).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
enum UpdateState {
    Unknown,
    Checking,
    UpToDate,
    Available { latest_version: String },
    Updating { target_version: String },
    Restarting { new_version: String },
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoDialogPhase {
    Available,
    Updating,
    Restarting,
}

struct Settings {
    modules: Vec<ModuleInfo>,
    page: Page,
    daemon_connected: bool,
    theme_mode: ThemeMode,
    system_is_dark: bool,
    language: Language,
    font_size: FontSize,
    high_contrast: bool,
    bold_text: bool,
    compact_layout: bool,
    reduced_motion: bool,
    visual_theme: VisualTheme,
    custom_image_history: Vec<PathBuf>,
    shortcut_test_results: HashMap<String, String>,
    hotkey_test_active: bool,
    test_active_keys: Vec<String>,
    /// Receiver for global keyboard events from the rdev listener thread.
    hotkey_test_rx: Option<std_mpsc::Receiver<(String, bool)>>,
    dependency_help_for: Option<String>,
    distro_name: String,
    package_manager: helpers::PackageManager,
    display_server: DisplayServer,
    toast: Option<ToastNotification>,
    toast_queue: VecDeque<ToastNotification>,
    update_state: UpdateState,
    update_dialog_open: bool,
    update_progress: f32,
    module_configs: module_settings::ModuleConfigs,
    demo_dialog_open: bool,
    demo_dialog_phase: DemoDialogPhase,
    demo_dialog_progress: f32,
}

// ── Update ──────────────────────────────────────────────────────────────────

impl Settings {
    fn new() -> (Self, Task<Message>) {
        let (visual_theme, custom_image_history) = persistence::load_ui_prefs();
        (
            Self {
                modules: modules::default_module_list(),
                page: Page::Dashboard,
                daemon_connected: false,
                theme_mode: ThemeMode::System,
                system_is_dark: helpers::detect_system_dark(),
                language: Language::Fr,
                font_size: FontSize::Medium,
                high_contrast: false,
                bold_text: false,
                compact_layout: false,
                reduced_motion: false,
                visual_theme,
                custom_image_history,
                shortcut_test_results: HashMap::new(),
                hotkey_test_active: false,
                test_active_keys: Vec::new(),
                hotkey_test_rx: None,
                dependency_help_for: None,
                distro_name: helpers::detect_distro_name(),
                package_manager: helpers::detect_package_manager(),
                display_server: helpers::detect_display_server(),
                toast: None,
                toast_queue: VecDeque::new(),
                update_state: UpdateState::Unknown,
                update_dialog_open: false,
                update_progress: 0.0,
                module_configs: module_settings::ModuleConfigs::load_all(),
                demo_dialog_open: false,
                demo_dialog_phase: DemoDialogPhase::Available,
                demo_dialog_progress: 0.0,
            },
            Task::perform(daemon::poll_daemon(), Message::DaemonState),
        )
    }

    fn is_dark(&self) -> bool {
        match self.theme_mode {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => self.system_is_dark,
        }
    }

    fn ui(&self) -> Ui {
        Ui {
            dark: self.is_dark(),
            s: self.font_size.scale(),
            bold: self.bold_text,
            compact: self.compact_layout,
            contrast: self.high_contrast,
            glass: self.visual_theme.is_glass(),
        }
    }

    fn resolved_theme(&self) -> Theme {
        let dark = || {
            Theme::custom(
                "MyPowerToys Dark".to_string(),
                Palette {
                    background: Color::from_rgb8(14, 14, 20),
                    text: Color::from_rgb8(205, 214, 244),
                    primary: Color::from_rgb8(137, 180, 250),
                    success: Color::from_rgb8(166, 227, 161),
                    danger: Color::from_rgb8(243, 139, 168),
                },
            )
        };
        match self.theme_mode {
            ThemeMode::Light => Theme::CatppuccinLatte,
            ThemeMode::Dark => dark(),
            ThemeMode::System => {
                if self.system_is_dark {
                    dark()
                } else {
                    Theme::CatppuccinLatte
                }
            }
        }
    }

    fn queue_toast(&mut self, kind: ToastKind, title: &str, message: &str) {
        let mut duration = match kind {
            ToastKind::Success => Duration::from_secs(4),
            ToastKind::Error => Duration::from_secs(6),
        };
        if self.reduced_motion {
            duration += Duration::from_secs(1);
        }
        let toast = ToastNotification::new(kind, title, message, duration);
        if self.toast.is_none() {
            self.toast = Some(toast);
            return;
        }

        if self.toast_queue.len() >= 4 {
            self.toast_queue.pop_front();
        }
        self.toast_queue.push_back(toast);
    }

    fn show_next_toast(&mut self) {
        if self.toast.is_none() {
            self.toast = self.toast_queue.pop_front();
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => {
                let going_to_about = page == Page::About;
                if page != Page::Tests {
                    self.hotkey_test_active = false;
                    self.test_active_keys.clear();
                }
                self.page = page;
                if !going_to_about {
                    self.update_dialog_open = false;
                } else if matches!(self.update_state, UpdateState::Unknown) {
                    self.update_state = UpdateState::Checking;
                    return Task::perform(
                        daemon::check_for_updates(),
                        Message::UpdateCheckFinished,
                    );
                }
            }
            Message::ToggleModule(id, enabled) => {
                if let Some(m) = self.modules.iter_mut().find(|m| m.id == id) {
                    m.running = enabled;
                }
                if self.daemon_connected {
                    return Task::perform(
                        daemon::daemon_toggle_module(id, enabled),
                        |(id, enabled, result)| Message::ToggleModuleResult(id, enabled, result),
                    );
                }
            }
            Message::ToggleModuleResult(id, desired, result) => {
                if result != "ok"
                    && let Some(m) = self.modules.iter_mut().find(|m| m.id == id)
                {
                    m.running = !desired;
                }
            }
            Message::TriggerHotkeyTest(id) => {
                if !self.daemon_connected {
                    self.shortcut_test_results
                        .insert(id, "error: daemon disconnected".to_string());
                    return Task::none();
                }
                self.shortcut_test_results
                    .insert(id.clone(), "pending".to_string());
                return Task::perform(daemon::daemon_trigger_hotkey(id), |(id, result)| {
                    Message::TriggerHotkeyTestResult(id, result)
                });
            }
            Message::TriggerHotkeyTestResult(id, result) => {
                self.shortcut_test_results.insert(id, result);
            }
            Message::StartHotkeyTest => {
                self.hotkey_test_active = true;
                self.test_active_keys.clear();
                if self.hotkey_test_rx.is_none() {
                    let (tx, rx) = std_mpsc::channel();
                    self.hotkey_test_rx = Some(rx);
                    std::thread::spawn(move || {
                        let _ = rdev::listen(move |event| {
                            let pair = match event.event_type {
                                rdev::EventType::KeyPress(key) => {
                                    rdev_key_name(key).map(|n| (n, true))
                                }
                                rdev::EventType::KeyRelease(key) => {
                                    rdev_key_name(key).map(|n| (n, false))
                                }
                                _ => None,
                            };
                            if let Some(p) = pair {
                                let _ = tx.send(p);
                            }
                        });
                    });
                }
            }
            Message::StopHotkeyTest => {
                self.hotkey_test_active = false;
                self.test_active_keys.clear();
            }
            Message::PollKeyboardEvents => {
                if let Some(rx) = &self.hotkey_test_rx {
                    while let Ok((name, pressed)) = rx.try_recv() {
                        if !self.hotkey_test_active {
                            continue;
                        }
                        if pressed {
                            if !self.test_active_keys.contains(&name) {
                                self.test_active_keys.push(name);
                            }
                        } else {
                            self.test_active_keys.retain(|k| k != &name);
                        }
                    }
                }
            }
            Message::ToggleDependencyHelp(id) => {
                if self.dependency_help_for.as_deref() == Some(id.as_str()) {
                    self.dependency_help_for = None;
                } else {
                    self.dependency_help_for = Some(id);
                }
            }
            Message::CloseDependencyHelp => {
                self.dependency_help_for = None;
            }
            Message::CopyInstallCommand(command) => {
                let tr = translations::get(self.language);
                match helpers::copy_to_clipboard(&command) {
                    Ok(()) => self.queue_toast(
                        ToastKind::Success,
                        tr.toast_success_title,
                        tr.toast_command_copied,
                    ),
                    Err(_) => self.queue_toast(
                        ToastKind::Error,
                        tr.toast_error_title,
                        tr.toast_copy_failed,
                    ),
                }
            }
            Message::DismissToast => {
                self.toast = None;
                self.show_next_toast();
            }
            Message::ToastTick => {
                if let Some(active) = self.toast.as_ref()
                    && Instant::now() >= active.expires_at
                {
                    self.toast = None;
                }
                self.show_next_toast();
            }
            Message::SetThemeMode(mode) => self.theme_mode = mode,
            Message::SetLanguage(lang) => self.language = lang,
            Message::SetFontSize(size) => self.font_size = size,
            Message::ToggleHighContrast(v) => self.high_contrast = v,
            Message::ToggleBoldText(v) => self.bold_text = v,
            Message::ToggleCompactLayout(v) => self.compact_layout = v,
            Message::ToggleReducedMotion(v) => self.reduced_motion = v,
            Message::SystemThemeCheck => self.system_is_dark = helpers::detect_system_dark(),
            Message::DaemonPoll => {
                return Task::perform(daemon::poll_daemon(), Message::DaemonState);
            }
            Message::DaemonState(state) => {
                self.daemon_connected = state.connected;
                for (id, _name, running) in &state.modules {
                    if let Some(m) = self.modules.iter_mut().find(|m| &m.id == id) {
                        m.running = *running;
                    }
                }
            }
            Message::SetVisualTheme(vt) => {
                self.visual_theme = vt;
                persistence::save_ui_prefs(&self.visual_theme, &self.custom_image_history);
            }
            Message::PickCustomImage => {
                return Task::perform(daemon::pick_image_file(), Message::CustomImagePicked);
            }
            Message::CustomImagePicked(path_opt) => {
                if let Some(path) = path_opt {
                    self.custom_image_history.retain(|p| p != &path);
                    self.custom_image_history.insert(0, path.clone());
                    if self.custom_image_history.len() > 10 {
                        self.custom_image_history.truncate(10);
                    }
                    self.visual_theme = VisualTheme::CustomImage(path);
                    persistence::save_ui_prefs(&self.visual_theme, &self.custom_image_history);
                }
            }
            Message::CheckForUpdates => {
                self.update_dialog_open = false;
                self.update_state = UpdateState::Checking;
                return Task::perform(daemon::check_for_updates(), Message::UpdateCheckFinished);
            }
            Message::UpdateCheckFinished(result) => {
                self.update_state = match result {
                    UpdateCheckResult::UpToDate => UpdateState::UpToDate,
                    UpdateCheckResult::Available(version) => UpdateState::Available {
                        latest_version: version,
                    },
                    UpdateCheckResult::Error(err) => UpdateState::Error(err),
                };
                if !matches!(self.update_state, UpdateState::Available { .. }) {
                    self.update_dialog_open = false;
                }
            }
            Message::OpenUpdateDialog => {
                if matches!(self.update_state, UpdateState::Available { .. }) {
                    self.update_dialog_open = true;
                }
            }
            Message::CloseUpdateDialog => {
                self.update_dialog_open = false;
            }
            Message::ConfirmUpdateInstall => {
                if let UpdateState::Available { latest_version } = &self.update_state {
                    self.update_progress = 0.0;
                    self.update_state = UpdateState::Updating {
                        target_version: latest_version.clone(),
                    };
                    return Task::perform(
                        daemon::perform_settings_update(),
                        Message::UpdateInstallFinished,
                    );
                }
            }
            Message::UpdateInstallFinished(result) => {
                let tr = translations::get(self.language);
                match result {
                    UpdateInstallResult::Updated(version) => {
                        self.update_state = UpdateState::Restarting {
                            new_version: version,
                        };
                        return Task::perform(
                            async { tokio::time::sleep(Duration::from_millis(1500)).await },
                            |_| Message::RestartApp,
                        );
                    }
                    UpdateInstallResult::AlreadyUpToDate => {
                        self.update_state = UpdateState::UpToDate;
                        self.update_dialog_open = false;
                        self.queue_toast(
                            ToastKind::Success,
                            tr.toast_success_title,
                            tr.toast_update_already,
                        );
                    }
                    UpdateInstallResult::Error(err) => {
                        let message = format!("{}: {err}", tr.toast_update_failed);
                        self.update_state = UpdateState::Error(err);
                        self.update_dialog_open = false;
                        self.queue_toast(ToastKind::Error, tr.toast_error_title, &message);
                    }
                }
            }
            Message::UpdateProgressTick => {
                self.update_progress += 0.02;
                if self.update_progress > 2.0 {
                    self.update_progress = 0.0;
                }
                if self.demo_dialog_open {
                    self.demo_dialog_progress += 0.02;
                    if self.demo_dialog_progress > 2.0 {
                        self.demo_dialog_progress = 0.0;
                    }
                }
            }
            Message::RestartApp => {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                }
                std::process::exit(0);
            }
            Message::StartDaemon => {
                let bin = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("mpt-daemon")))
                    .unwrap_or_else(|| "mpt-daemon".into());
                return Task::perform(
                    async move {
                        std::process::Command::new(&bin)
                            .stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    },
                    Message::StartDaemonResult,
                );
            }
            Message::StartDaemonResult(result) => {
                let tr = translations::get(self.language);
                if let Err(err) = result {
                    self.queue_toast(
                        ToastKind::Error,
                        tr.toast_error_title,
                        &format!("mpt-daemon: {err}"),
                    );
                }
            }
            // Module settings
            Message::SetColorPickerFormat(fmt) => {
                self.module_configs.color_picker.format = fmt;
                self.module_configs.save("color-picker");
            }
            Message::SetColorPickerBehavior(behavior) => {
                self.module_configs.color_picker.behavior = behavior;
                self.module_configs.save("color-picker");
            }
            Message::ToggleColorPickerShowName(v) => {
                self.module_configs.color_picker.show_color_name = v;
                self.module_configs.save("color-picker");
            }
            Message::ToggleColorFormat(id, enabled) => {
                if let Some(entry) = self
                    .module_configs
                    .color_picker
                    .formats
                    .iter_mut()
                    .find(|e| e.id == id)
                {
                    entry.enabled = enabled;
                }
                self.module_configs.save("color-picker");
            }
            Message::SetTextExtractorLang(lang) => {
                self.module_configs.text_extractor.language = lang;
                self.module_configs.save("text-extractor");
            }
            Message::SetImageResizerPreset(preset) => {
                self.module_configs.image_resizer.preset = preset;
                self.module_configs.save("image-resizer");
            }
            Message::SetImageResizerFormat(fmt) => {
                self.module_configs.image_resizer.output_format = fmt;
                self.module_configs.save("image-resizer");
            }
            Message::SetImageResizerQuality(q) => {
                self.module_configs.image_resizer.quality = q;
                self.module_configs.save("image-resizer");
            }
            Message::ToggleMouseFindMyMouse(v) => {
                self.module_configs.mouse_utils.find_my_mouse = v;
                self.module_configs.save("mouse-utils");
            }
            Message::ToggleMouseClickHighlighter(v) => {
                self.module_configs.mouse_utils.click_highlighter = v;
                self.module_configs.save("mouse-utils");
            }
            Message::ToggleMouseCrosshair(v) => {
                self.module_configs.mouse_utils.crosshair = v;
                self.module_configs.save("mouse-utils");
            }
            Message::SetAppLauncherMaxResults(n) => {
                self.module_configs.app_launcher.max_results = n;
                self.module_configs.save("app-launcher");
            }
            Message::ToggleAppLauncherCalc(v) => {
                self.module_configs.app_launcher.show_calculator = v;
                self.module_configs.save("app-launcher");
            }
            Message::SetFancyZonesGap(gap) => {
                self.module_configs.fancy_zones.zone_gap = gap;
                self.module_configs.save("fancy-zones");
            }
            Message::SetPeekPreviewLines(n) => {
                self.module_configs.peek.max_preview_lines = n;
                self.module_configs.save("peek");
            }
            Message::SetPeekDirEntries(n) => {
                self.module_configs.peek.max_dir_entries = n;
                self.module_configs.save("peek");
            }
            // Light Switch
            Message::SetLightSwitchSchedule(mode) => {
                self.module_configs.light_switch.schedule_mode = mode;
                self.module_configs.save("light-switch");
            }
            Message::SetLightSwitchLatitude(val) => {
                if let Ok(v) = val.parse::<f64>() {
                    self.module_configs.light_switch.latitude = v;
                    self.module_configs.save("light-switch");
                }
            }
            Message::SetLightSwitchLongitude(val) => {
                if let Ok(v) = val.parse::<f64>() {
                    self.module_configs.light_switch.longitude = v;
                    self.module_configs.save("light-switch");
                }
            }
            Message::SetLightSwitchSunriseOffset(val) => {
                if let Ok(v) = val.parse::<i32>() {
                    self.module_configs.light_switch.sunrise_offset_min = v;
                    self.module_configs.save("light-switch");
                }
            }
            Message::SetLightSwitchSunsetOffset(val) => {
                if let Ok(v) = val.parse::<i32>() {
                    self.module_configs.light_switch.sunset_offset_min = v;
                    self.module_configs.save("light-switch");
                }
            }
            Message::SetLightSwitchDarkTime(val) => {
                self.module_configs.light_switch.dark_mode_time = val;
                self.module_configs.save("light-switch");
            }
            Message::SetLightSwitchLightTime(val) => {
                self.module_configs.light_switch.light_mode_time = val;
                self.module_configs.save("light-switch");
            }
            Message::ToggleLightSwitchSystem(v) => {
                self.module_configs.light_switch.apply_system = v;
                self.module_configs.save("light-switch");
            }
            Message::ToggleLightSwitchApps(v) => {
                self.module_configs.light_switch.apply_apps = v;
                self.module_configs.save("light-switch");
            }
            Message::DemoToast(kind) => {
                let tr = translations::get(self.language);
                match kind {
                    ToastKind::Success => {
                        self.queue_toast(
                            ToastKind::Success,
                            tr.toast_success_title,
                            tr.ds_demo_toast_success,
                        );
                    }
                    ToastKind::Error => {
                        self.queue_toast(
                            ToastKind::Error,
                            tr.toast_error_title,
                            tr.ds_demo_toast_error,
                        );
                    }
                }
            }
            Message::OpenDemoDialog => {
                self.demo_dialog_open = true;
                self.demo_dialog_phase = DemoDialogPhase::Available;
                self.demo_dialog_progress = 0.0;
            }
            Message::CloseDemoDialog => {
                self.demo_dialog_open = false;
            }
            Message::DemoDialogConfirm => match self.demo_dialog_phase {
                DemoDialogPhase::Available => {
                    self.demo_dialog_phase = DemoDialogPhase::Updating;
                    self.demo_dialog_progress = 0.0;
                    return Task::perform(
                        async { tokio::time::sleep(Duration::from_millis(3000)).await },
                        |_| Message::DemoDialogConfirm,
                    );
                }
                DemoDialogPhase::Updating => {
                    self.demo_dialog_phase = DemoDialogPhase::Restarting;
                    self.demo_dialog_progress = 0.0;
                    return Task::perform(
                        async { tokio::time::sleep(Duration::from_millis(2000)).await },
                        |_| Message::CloseDemoDialog,
                    );
                }
                DemoDialogPhase::Restarting => {
                    self.demo_dialog_open = false;
                }
            },
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![time::every(Duration::from_secs(3)).map(|_| Message::DaemonPoll)];
        if self.toast.is_some() || !self.toast_queue.is_empty() {
            let cadence = if self.reduced_motion { 200 } else { 80 };
            subs.push(time::every(Duration::from_millis(cadence)).map(|_| Message::ToastTick));
        }
        if self.theme_mode == ThemeMode::System {
            subs.push(time::every(Duration::from_secs(2)).map(|_| Message::SystemThemeCheck));
        }
        if self.hotkey_test_active && self.hotkey_test_rx.is_some() {
            subs.push(time::every(Duration::from_millis(20)).map(|_| Message::PollKeyboardEvents));
        }
        let needs_progress_tick = matches!(
            self.update_state,
            UpdateState::Updating { .. } | UpdateState::Restarting { .. }
        ) || (self.demo_dialog_open
            && matches!(
                self.demo_dialog_phase,
                DemoDialogPhase::Updating | DemoDialogPhase::Restarting
            ));
        if needs_progress_tick {
            let cadence = if self.reduced_motion { 100 } else { 50 };
            subs.push(
                time::every(Duration::from_millis(cadence)).map(|_| Message::UpdateProgressTick),
            );
        }
        Subscription::batch(subs)
    }
}

// ── Keyboard helpers (rdev) ─────────────────────────────────────────────────

fn rdev_key_name(key: rdev::Key) -> Option<String> {
    use rdev::Key::*;
    match key {
        ShiftLeft | ShiftRight => Some("Shift".into()),
        ControlLeft | ControlRight => Some("Ctrl".into()),
        Alt => Some("Alt".into()),
        AltGr => Some("AltGr".into()),
        MetaLeft | MetaRight => Some("Super".into()),
        Space => Some("Space".into()),
        Tab => Some("Tab".into()),
        Return => Some("Enter".into()),
        Escape => Some("Escape".into()),
        UpArrow => Some("Up".into()),
        DownArrow => Some("Down".into()),
        LeftArrow => Some("Left".into()),
        RightArrow => Some("Right".into()),
        Backspace => Some("Backspace".into()),
        Delete => Some("Delete".into()),
        Home => Some("Home".into()),
        End => Some("End".into()),
        PageUp => Some("PageUp".into()),
        PageDown => Some("PageDown".into()),
        CapsLock => Some("CapsLock".into()),
        F1 => Some("F1".into()),
        F2 => Some("F2".into()),
        F3 => Some("F3".into()),
        F4 => Some("F4".into()),
        F5 => Some("F5".into()),
        F6 => Some("F6".into()),
        F7 => Some("F7".into()),
        F8 => Some("F8".into()),
        F9 => Some("F9".into()),
        F10 => Some("F10".into()),
        F11 => Some("F11".into()),
        F12 => Some("F12".into()),
        KeyA => Some("A".into()),
        KeyB => Some("B".into()),
        KeyC => Some("C".into()),
        KeyD => Some("D".into()),
        KeyE => Some("E".into()),
        KeyF => Some("F".into()),
        KeyG => Some("G".into()),
        KeyH => Some("H".into()),
        KeyI => Some("I".into()),
        KeyJ => Some("J".into()),
        KeyK => Some("K".into()),
        KeyL => Some("L".into()),
        KeyM => Some("M".into()),
        KeyN => Some("N".into()),
        KeyO => Some("O".into()),
        KeyP => Some("P".into()),
        KeyQ => Some("Q".into()),
        KeyR => Some("R".into()),
        KeyS => Some("S".into()),
        KeyT => Some("T".into()),
        KeyU => Some("U".into()),
        KeyV => Some("V".into()),
        KeyW => Some("W".into()),
        KeyX => Some("X".into()),
        KeyY => Some("Y".into()),
        KeyZ => Some("Z".into()),
        Num0 => Some("0".into()),
        Num1 => Some("1".into()),
        Num2 => Some("2".into()),
        Num3 => Some("3".into()),
        Num4 => Some("4".into()),
        Num5 => Some("5".into()),
        Num6 => Some("6".into()),
        Num7 => Some("7".into()),
        Num8 => Some("8".into()),
        Num9 => Some("9".into()),
        _ => None,
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter("mpt_ui=debug")
        .init();

    let mut app = iced::application("MyPowerToys Settings", Settings::update, Settings::view)
        .theme(Settings::resolved_theme)
        .subscription(Settings::subscription)
        .font(BOOTSTRAP_FONT_BYTES)
        .window_size((1000.0, 700.0));

    if let Some(cjk) = helpers::load_cjk_font() {
        app = app.font(cjk);
    }

    app.run_with(Settings::new)
}
