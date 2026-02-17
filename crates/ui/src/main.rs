mod theme;
mod translations;
mod daemon;
mod helpers;
mod message;
mod modules;
mod persistence;
mod types;
mod views;
mod widgets;

use iced::time;
use iced::theme::Palette;
use iced::{Color, Subscription, Task, Theme};
use iced_fonts::BOOTSTRAP_FONT_BYTES;
use std::path::PathBuf;
use std::time::Duration;
use translations::Language;
use message::*;
use types::*;

// ── State ───────────────────────────────────────────────────────────────────

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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => self.page = page,
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
                if result != "ok" {
                    if let Some(m) = self.modules.iter_mut().find(|m| m.id == id) {
                        m.running = !desired;
                    }
                }
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
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            time::every(Duration::from_secs(3)).map(|_| Message::DaemonPoll),
        ];
        if self.theme_mode == ThemeMode::System {
            subs.push(time::every(Duration::from_secs(2)).map(|_| Message::SystemThemeCheck));
        }
        Subscription::batch(subs)
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
