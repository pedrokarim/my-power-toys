mod theme;
mod translations;

use iced::font::Weight;
use iced::time;
use iced::widget::{
    Space, button, column, container, horizontal_rule, image, row, scrollable, text, toggler,
};
use iced::{Alignment, Color, Element, Font, Length, Padding, Subscription, Task, Theme};
use iced_fonts::bootstrap::Bootstrap;
use iced_fonts::{BOOTSTRAP_FONT, BOOTSTRAP_FONT_BYTES};
use mpt_common::ipc;
use std::time::Duration;
use translations::Language;

// ── Data ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ModuleInfo {
    id: String,
    name: String,
    description: String,
    icon: Bootstrap,
    accent: Color,
    hotkey: Option<String>,
    running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Dashboard,
    Module(String),
    Preferences,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FontSize {
    Small,
    Medium,
    Large,
}

impl FontSize {
    fn scale(self) -> f32 {
        match self {
            FontSize::Small => 0.9,
            FontSize::Medium => 1.0,
            FontSize::Large => 1.15,
        }
    }
}

/// UI context passed to widget builders for theme-aware colors and scaled sizes.
#[derive(Clone, Copy)]
struct Ui {
    dark: bool,
    s: f32,
    bold: bool,
    compact: bool,
    contrast: bool,
}

impl Ui {
    fn sz(&self, base: f32) -> f32 {
        (base * self.s).round()
    }

    /// Returns a bold font when bold-text accessibility is on, otherwise default.
    fn font(&self) -> Font {
        if self.bold { bold() } else { Font::DEFAULT }
    }

    /// Scales padding down by ~30% when compact layout is on.
    fn pad(&self, base: f32) -> f32 {
        if self.compact {
            (base * 0.7).round()
        } else {
            base
        }
    }
}

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
}

#[derive(Debug, Clone)]
enum Message {
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
}

#[derive(Debug, Clone)]
struct DaemonStateResult {
    connected: bool,
    modules: Vec<(String, String, bool)>,
}

/// Check daemon status and fetch module list via D-Bus.
async fn poll_daemon() -> DaemonStateResult {
    let conn = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(_) => {
            return DaemonStateResult {
                connected: false,
                modules: vec![],
            };
        }
    };

    // Try ping
    if conn
        .call_method(
            Some(ipc::BUS_NAME),
            ipc::OBJECT_PATH,
            Some(ipc::BUS_NAME),
            "Ping",
            &(),
        )
        .await
        .is_err()
    {
        return DaemonStateResult {
            connected: false,
            modules: vec![],
        };
    }

    // Fetch module list
    let modules = match conn
        .call_method(
            Some(ipc::BUS_NAME),
            ipc::OBJECT_PATH,
            Some(ipc::BUS_NAME),
            "ListModules",
            &(),
        )
        .await
    {
        Ok(msg) => msg
            .body()
            .deserialize::<Vec<(String, String, bool)>>()
            .unwrap_or_default(),
        Err(_) => vec![],
    };

    DaemonStateResult {
        connected: true,
        modules,
    }
}

/// Toggle a module on the daemon via D-Bus. Returns (module_id, desired_state, result).
async fn daemon_toggle_module(id: String, enable: bool) -> (String, bool, String) {
    let method = if enable { "StartModule" } else { "StopModule" };
    let result = async {
        let conn = zbus::Connection::session().await?;
        let msg = conn
            .call_method(
                Some(ipc::BUS_NAME),
                ipc::OBJECT_PATH,
                Some(ipc::BUS_NAME),
                method,
                &id.as_str(),
            )
            .await?;
        let r: String = msg.body().deserialize()?;
        Ok::<String, zbus::Error>(r)
    }
    .await;

    let status = match result {
        Ok(s) => s,
        Err(e) => format!("error: {e}"),
    };
    (id, enable, status)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    }
}

/// Detect system dark mode via GNOME/GTK color-scheme setting.
fn detect_system_dark() -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("dark"))
        .unwrap_or(true)
}

/// Try to load a system CJK font for Japanese/Chinese support.
fn load_cjk_font() -> Option<Vec<u8>> {
    [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ]
    .iter()
    .find_map(|p| std::fs::read(p).ok())
}

// ── Update ──────────────────────────────────────────────────────────────────

impl Settings {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                modules: default_module_list(),
                page: Page::Dashboard,
                daemon_connected: false,
                theme_mode: ThemeMode::System,
                system_is_dark: detect_system_dark(),
                language: Language::Fr,
                font_size: FontSize::Medium,
                high_contrast: false,
                bold_text: false,
                compact_layout: false,
                reduced_motion: false,
            },
            // Check daemon immediately at startup
            Task::perform(poll_daemon(), Message::DaemonState),
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
        }
    }

    fn resolved_theme(&self) -> Theme {
        match self.theme_mode {
            ThemeMode::Light => Theme::CatppuccinLatte,
            ThemeMode::Dark => Theme::CatppuccinMocha,
            ThemeMode::System => {
                if self.system_is_dark {
                    Theme::CatppuccinMocha
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
                // Optimistic UI update
                if let Some(m) = self.modules.iter_mut().find(|m| m.id == id) {
                    m.running = enabled;
                }
                // Send D-Bus call to daemon
                if self.daemon_connected {
                    return Task::perform(
                        daemon_toggle_module(id, enabled),
                        |(id, enabled, result)| Message::ToggleModuleResult(id, enabled, result),
                    );
                }
            }
            Message::ToggleModuleResult(id, desired, result) => {
                if result != "ok" {
                    // Revert on failure
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
            Message::SystemThemeCheck => self.system_is_dark = detect_system_dark(),
            Message::DaemonPoll => {
                return Task::perform(poll_daemon(), Message::DaemonState);
            }
            Message::DaemonState(state) => {
                self.daemon_connected = state.connected;
                // Sync running states from daemon
                for (id, _name, running) in &state.modules {
                    if let Some(m) = self.modules.iter_mut().find(|m| &m.id == id) {
                        m.running = *running;
                    }
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            // Poll daemon status every 3 seconds
            time::every(Duration::from_secs(3)).map(|_| Message::DaemonPoll),
        ];
        if self.theme_mode == ThemeMode::System {
            subs.push(time::every(Duration::from_secs(2)).map(|_| Message::SystemThemeCheck));
        }
        Subscription::batch(subs)
    }

    // ── Root view ───────────────────────────────────────────────────────────

    fn view(&self) -> Element<'_, Message> {
        row![self.view_sidebar(), self.view_content()]
            .height(Length::Fill)
            .into()
    }

    // ── Sidebar ─────────────────────────────────────────────────────────────

    fn view_sidebar(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();

        // Header with logo
        let logo = image(format!(
            "{}/assets/icons/icon-64.png",
            env!("CARGO_MANIFEST_DIR").replace("/crates/ui", "")
        ))
        .width(40)
        .height(40);

        let header = container(
            row![
                logo,
                column![
                    text("MyPowerToys").size(ui.sz(16.0)).font(bold()),
                    text(tr.settings)
                        .size(ui.sz(11.0))
                        .color(theme::subtext0(ui.dark)),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding(Padding::new(16.0));

        // Dashboard
        let mut nav = column![sidebar_icon_button(
            Bootstrap::Speedometer,
            tr.dashboard,
            self.page == Page::Dashboard,
            Message::NavigateTo(Page::Dashboard),
            ui,
        )]
        .spacing(2);

        // Section label
        nav = nav.push(
            container(
                text(tr.modules_label)
                    .size(ui.sz(11.0))
                    .color(theme::overlay0(ui.dark)),
            )
            .padding(Padding {
                top: 12.0,
                right: 12.0,
                bottom: 4.0,
                left: 12.0,
            }),
        );

        // Module items
        for module in &self.modules {
            let is_selected = self.page == Page::Module(module.id.clone());
            nav = nav.push(sidebar_badge_button(
                module.icon,
                module.accent,
                &module.name,
                is_selected,
                Message::NavigateTo(Page::Module(module.id.clone())),
                ui,
            ));
        }

        // Separator + Preferences + About
        nav = nav.push(Space::with_height(8));
        nav = nav.push(container(horizontal_rule(1)).padding(Padding::from([0.0, 12.0])));
        nav = nav.push(Space::with_height(4));
        nav = nav.push(sidebar_icon_button(
            Bootstrap::GearFill,
            tr.preferences,
            self.page == Page::Preferences,
            Message::NavigateTo(Page::Preferences),
            ui,
        ));
        nav = nav.push(sidebar_icon_button(
            Bootstrap::InfoCircle,
            tr.about,
            self.page == Page::About,
            Message::NavigateTo(Page::About),
            ui,
        ));

        container(column![
            header,
            container(horizontal_rule(1)).padding(Padding::from([0.0, 12.0])),
            scrollable(nav.padding(Padding::from([8.0, 4.0]))).height(Length::Fill),
        ])
        .width(250)
        .height(Length::Fill)
        .style(theme::sidebar)
        .into()
    }

    // ── Content area ────────────────────────────────────────────────────────

    fn view_content(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let inner = match &self.page {
            Page::Dashboard => self.view_dashboard(),
            Page::Module(id) => self.view_module(id),
            Page::Preferences => self.view_preferences(),
            Page::About => self.view_about(),
        };
        container(scrollable(
            container(inner)
                .padding(ui.pad(32.0) as u16)
                .width(Length::Fill),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    // ── Dashboard ───────────────────────────────────────────────────────────

    fn view_dashboard(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let running = self.modules.iter().filter(|m| m.running).count();
        let total = self.modules.len();

        let (dot_col, status_txt) = if self.daemon_connected {
            (theme::green(), tr.daemon_connected)
        } else {
            (theme::red(), tr.daemon_not_connected)
        };

        let status = row![
            text("●").size(ui.sz(10.0)).color(dot_col),
            text(status_txt)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext1(ui.dark)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Stats row
        let stats = row![
            stat_card(tr.total, &total.to_string(), theme::blue(), ui),
            stat_card(tr.active, &running.to_string(), theme::green(), ui),
            stat_card(
                tr.inactive,
                &(total - running).to_string(),
                theme::overlay0(ui.dark),
                ui,
            ),
        ]
        .spacing(12);

        // Module cards
        let mut modules_col = column![].spacing(8);
        for module in &self.modules {
            modules_col = modules_col.push(self.module_card(module));
        }

        column![
            text(tr.dashboard).size(ui.sz(28.0)).font(bold()),
            status,
            Space::with_height(4),
            stats,
            Space::with_height(8),
            text(tr.all_modules).size(ui.sz(18.0)).font(bold()),
            modules_col,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    fn module_card<'a>(&self, module: &ModuleInfo) -> Element<'a, Message> {
        let ui = self.ui();

        let hotkey_badge: Element<'a, Message> = if let Some(ref hk) = module.hotkey {
            kbd(hk, ui)
        } else {
            Space::with_width(0).into()
        };

        let left = row![
            icon_badge(module.icon, module.accent, ui.sz(18.0)),
            column![
                text(module.name.clone()).size(ui.sz(14.0)).font(ui.font()),
                text(module.description.clone())
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let right = row![
            hotkey_badge,
            toggler(module.running)
                .on_toggle({
                    let id = module.id.clone();
                    move |v| Message::ToggleModule(id.clone(), v)
                })
                .size(ui.sz(22.0)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let content = row![left, right].align_y(Alignment::Center).spacing(12);

        card(content, ui)
    }

    // ── Module detail ───────────────────────────────────────────────────────

    fn view_module(&self, id: &str) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let module = self.modules.iter().find(|m| m.id == id);
        match module {
            Some(m) => {
                let dot_col = if m.running {
                    theme::green()
                } else {
                    theme::overlay0(ui.dark)
                };
                let status_txt = if m.running { tr.running } else { tr.stopped };

                // Header
                let header = row![
                    icon_badge(m.icon, m.accent, ui.sz(28.0)),
                    column![
                        text(&m.name).size(ui.sz(28.0)).font(bold()),
                        text(&m.description)
                            .size(ui.sz(14.0))
                            .font(ui.font())
                            .color(theme::subtext1(ui.dark)),
                    ]
                    .spacing(4),
                ]
                .spacing(16)
                .align_y(Alignment::Center);

                // Status card
                let status_card = card(
                    column![
                        row![
                            text(tr.status)
                                .size(ui.sz(14.0))
                                .font(ui.font())
                                .color(theme::subtext0(ui.dark)),
                            Space::with_width(Length::Fill),
                            text("●").size(ui.sz(10.0)).color(dot_col),
                            text(status_txt).size(ui.sz(14.0)).font(ui.font()),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                        Space::with_height(12),
                        toggler(m.running).label(tr.enabled).on_toggle({
                            let id = m.id.clone();
                            move |v| Message::ToggleModule(id.clone(), v)
                        }),
                    ]
                    .spacing(8),
                    ui,
                );

                // Hotkey card
                let hotkey_card = m.hotkey.as_ref().map(|hk| {
                    card(
                        row![
                            text(tr.hotkey)
                                .size(ui.sz(14.0))
                                .font(ui.font())
                                .color(theme::subtext0(ui.dark)),
                            Space::with_width(Length::Fill),
                            kbd(hk, ui),
                        ]
                        .align_y(Alignment::Center),
                        ui,
                    )
                });

                // Settings placeholder
                let settings_card = card(
                    column![
                        text(tr.module_settings).size(ui.sz(16.0)).font(bold()),
                        text(tr.module_settings_placeholder)
                            .size(ui.sz(13.0))
                            .font(ui.font())
                            .color(theme::subtext0(ui.dark)),
                    ]
                    .spacing(8),
                    ui,
                );

                let mut content = column![header, Space::with_height(4), status_card].spacing(12);
                if let Some(hk) = hotkey_card {
                    content = content.push(hk);
                }
                content = content.push(settings_card);
                content.width(Length::Fill).into()
            }
            None => text(tr.module_not_found).size(ui.sz(20.0)).into(),
        }
    }

    // ── About ───────────────────────────────────────────────────────────────

    fn view_about(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let logo = image(format!(
            "{}/assets/logo-200.png",
            env!("CARGO_MANIFEST_DIR").replace("/crates/ui", "")
        ))
        .width(80)
        .height(80);

        let header = row![
            logo,
            column![
                text("MyPowerToys").size(ui.sz(28.0)).font(bold()),
                text("Version 0.1.0")
                    .size(ui.sz(14.0))
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(4),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let desc = card(
            column![
                text(tr.about_title).size(ui.sz(16.0)).font(bold()),
                text(tr.about_desc).size(ui.sz(14.0)).font(ui.font()),
                text(tr.about_detail)
                    .size(ui.sz(14.0))
                    .font(ui.font())
                    .color(theme::subtext1(ui.dark)),
            ]
            .spacing(8),
            ui,
        );

        let info = card(
            column![
                info_row(tr.author, "Ahmed Karim", ui),
                info_row(tr.license, "GPL-3.0", ui),
                info_row(tr.repo, "github.com/pedrokarim/my-power-toys", ui),
                info_row("UI", "iced (Rust)", ui),
                info_row("IPC", "D-Bus (zbus)", ui),
            ]
            .spacing(12),
            ui,
        );

        let tech = card(
            column![
                text(tr.tech_stack).size(ui.sz(16.0)).font(bold()),
                info_row(tr.prog_lang, "Rust (edition 2024)", ui),
                info_row("Settings UI", "iced", ui),
                info_row("Overlays", "egui / eframe", ui),
                info_row("Async", "tokio", ui),
                info_row("Config", "serde + TOML", ui),
            ]
            .spacing(12),
            ui,
        );

        column![header, Space::with_height(8), desc, info, tech]
            .spacing(12)
            .width(Length::Fill)
            .into()
    }

    // ── Preferences ──────────────────────────────────────────────────────────

    fn view_preferences(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();

        // Appearance card
        let appearance_card = card(
            column![
                text(tr.appearance).size(ui.sz(16.0)).font(bold()),
                text(tr.theme_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button(
                            tr.light,
                            self.theme_mode == ThemeMode::Light,
                            Message::SetThemeMode(ThemeMode::Light),
                            ui,
                        ),
                        seg_button(
                            tr.dark,
                            self.theme_mode == ThemeMode::Dark,
                            Message::SetThemeMode(ThemeMode::Dark),
                            ui,
                        ),
                        seg_button(
                            tr.auto_theme,
                            self.theme_mode == ThemeMode::System,
                            Message::SetThemeMode(ThemeMode::System),
                            ui,
                        ),
                    ]
                    .spacing(2),
                )
                .style(theme::segmented_control),
            ]
            .spacing(4),
            ui,
        );

        // Language card
        let language_card = card(
            column![
                text(tr.language).size(ui.sz(16.0)).font(bold()),
                text(tr.lang_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button(
                            "Français",
                            self.language == Language::Fr,
                            Message::SetLanguage(Language::Fr),
                            ui,
                        ),
                        seg_button(
                            "English",
                            self.language == Language::En,
                            Message::SetLanguage(Language::En),
                            ui,
                        ),
                        seg_button(
                            "Español",
                            self.language == Language::Es,
                            Message::SetLanguage(Language::Es),
                            ui,
                        ),
                        seg_button(
                            "日本語",
                            self.language == Language::Jp,
                            Message::SetLanguage(Language::Jp),
                            ui,
                        ),
                        seg_button(
                            "中文",
                            self.language == Language::Cn,
                            Message::SetLanguage(Language::Cn),
                            ui,
                        ),
                    ]
                    .spacing(2),
                )
                .style(theme::segmented_control),
            ]
            .spacing(4),
            ui,
        );

        // Text size card
        let text_size_card = card(
            column![
                text(tr.text_size).size(ui.sz(16.0)).font(bold()),
                text(tr.text_size_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button(
                            tr.small,
                            self.font_size == FontSize::Small,
                            Message::SetFontSize(FontSize::Small),
                            ui,
                        ),
                        seg_button(
                            tr.medium,
                            self.font_size == FontSize::Medium,
                            Message::SetFontSize(FontSize::Medium),
                            ui,
                        ),
                        seg_button(
                            tr.large,
                            self.font_size == FontSize::Large,
                            Message::SetFontSize(FontSize::Large),
                            ui,
                        ),
                    ]
                    .spacing(2),
                )
                .style(theme::segmented_control),
            ]
            .spacing(4),
            ui,
        );

        // Accessibility card
        let accessibility_card = card(
            column![
                text(tr.accessibility).size(ui.sz(16.0)).font(bold()),
                Space::with_height(4),
                pref_toggle(
                    tr.high_contrast,
                    tr.high_contrast_desc,
                    self.high_contrast,
                    Message::ToggleHighContrast,
                    ui,
                ),
                pref_toggle(
                    tr.bold_text,
                    tr.bold_text_desc,
                    self.bold_text,
                    Message::ToggleBoldText,
                    ui,
                ),
                pref_toggle(
                    tr.compact_layout,
                    tr.compact_layout_desc,
                    self.compact_layout,
                    Message::ToggleCompactLayout,
                    ui,
                ),
                pref_toggle(
                    tr.reduced_motion,
                    tr.reduced_motion_desc,
                    self.reduced_motion,
                    Message::ToggleReducedMotion,
                    ui,
                ),
            ]
            .spacing(12),
            ui,
        );

        column![
            text(tr.preferences).size(ui.sz(28.0)).font(bold()),
            Space::with_height(4),
            appearance_card,
            language_card,
            text_size_card,
            accessibility_card,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }
}

// ── Reusable widget builders ────────────────────────────────────────────────

/// Card container with rounded corners and surface background.
fn card<'a>(content: impl Into<Element<'a, Message>>, ui: Ui) -> Element<'a, Message> {
    container(content)
        .padding(ui.pad(16.0) as u16)
        .width(Length::Fill)
        .style(theme::card(ui.contrast))
        .into()
}

/// Stat card showing a big number with a label.
fn stat_card<'a>(label: &str, value: &str, accent: Color, ui: Ui) -> Element<'a, Message> {
    container(
        column![
            text(value.to_string())
                .size(ui.sz(32.0))
                .font(bold())
                .color(accent),
            text(label.to_string())
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .padding(ui.pad(16.0) as u16)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .style(theme::stat_card(ui.contrast))
    .into()
}

/// Keyboard shortcut badge.
fn kbd<'a>(key: &str, ui: Ui) -> Element<'a, Message> {
    container(
        text(key.to_string())
            .size(ui.sz(12.0))
            .font(ui.font())
            .color(theme::subtext1(ui.dark)),
    )
    .padding(Padding::from([4.0, 10.0]))
    .style(theme::kbd(ui.contrast))
    .into()
}

/// Colored icon badge (perfectly square rounded container).
fn icon_badge<'a>(icon: Bootstrap, accent: Color, size: f32) -> Element<'a, Message> {
    let box_size = size * 1.7;
    container(text(icon.to_string()).font(BOOTSTRAP_FONT).size(size))
        .width(box_size)
        .height(box_size)
        .center_x(box_size)
        .center_y(box_size)
        .style(theme::icon_badge(accent))
        .into()
}

/// Sidebar button with a Bootstrap icon.
fn sidebar_icon_button(
    icon: Bootstrap,
    label: &str,
    selected: bool,
    msg: Message,
    ui: Ui,
) -> Element<'static, Message> {
    let content = row![
        text(icon.to_string())
            .font(BOOTSTRAP_FONT)
            .size(ui.sz(14.0))
            .color(if selected {
                theme::blue()
            } else {
                theme::overlay0(ui.dark)
            }),
        text(label.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding(Padding::from([8.0, 12.0]))
        .style(theme::nav_button(selected))
        .into()
}

/// Sidebar button with a colored icon badge (for modules).
fn sidebar_badge_button(
    icon: Bootstrap,
    accent: Color,
    label: &str,
    selected: bool,
    msg: Message,
    ui: Ui,
) -> Element<'static, Message> {
    let content = row![
        icon_badge(icon, accent, ui.sz(13.0)),
        text(label.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding(Padding::from([6.0, 12.0]))
        .style(theme::nav_button(selected))
        .into()
}

/// Segmented control button.
fn seg_button(label: &str, active: bool, msg: Message, ui: Ui) -> Element<'static, Message> {
    button(text(label.to_string()).size(ui.sz(11.0)).font(ui.font()))
        .on_press(msg)
        .padding(Padding::from([5.0, 10.0]))
        .style(theme::seg_button(active))
        .into()
}

/// Info row: faded label + value.
fn info_row<'a>(label: &str, value: &str, ui: Ui) -> Element<'a, Message> {
    row![
        text(label.to_string())
            .size(ui.sz(13.0))
            .font(ui.font())
            .color(theme::subtext0(ui.dark))
            .width(120),
        text(value.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(12)
    .into()
}

/// Toggle row for accessibility preferences.
fn pref_toggle<'a>(
    label: &str,
    desc: &str,
    value: bool,
    msg: fn(bool) -> Message,
    ui: Ui,
) -> Element<'a, Message> {
    row![
        column![
            text(label.to_string()).size(ui.sz(14.0)).font(ui.font()),
            text(desc.to_string())
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value).on_toggle(msg).size(ui.sz(22.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

// ── Module catalogue ────────────────────────────────────────────────────────

fn default_module_list() -> Vec<ModuleInfo> {
    vec![
        ModuleInfo {
            id: "always-on-top".into(),
            name: "Always on Top".into(),
            description: "Pin any window above all others".into(),
            icon: Bootstrap::PinAngle,
            accent: theme::blue(),
            hotkey: Some("Super+T".into()),
            running: false,
        },
        ModuleInfo {
            id: "awake".into(),
            name: "Awake".into(),
            description: "Prevent screen sleep and suspend".into(),
            icon: Bootstrap::CupHot,
            accent: theme::mauve(),
            hotkey: None,
            running: false,
        },
        ModuleInfo {
            id: "paste-plain".into(),
            name: "Paste as Plain Text".into(),
            description: "Paste clipboard without formatting".into(),
            icon: Bootstrap::ClipboardCheck,
            accent: theme::green(),
            hotkey: Some("Super+Ctrl+V".into()),
            running: false,
        },
        ModuleInfo {
            id: "color-picker".into(),
            name: "Color Picker".into(),
            description: "Pick any colour from the screen (HEX, RGB, HSL)".into(),
            icon: Bootstrap::Droplet,
            accent: theme::teal(),
            hotkey: Some("Super+Shift+C".into()),
            running: false,
        },
        ModuleInfo {
            id: "hosts-editor".into(),
            name: "Hosts Editor".into(),
            description: "GUI editor for /etc/hosts with toggle on/off".into(),
            icon: Bootstrap::FileTextFill,
            accent: theme::peach(),
            hotkey: None,
            running: false,
        },
        ModuleInfo {
            id: "bulk-rename".into(),
            name: "Bulk Rename".into(),
            description: "Batch rename files with regex and preview".into(),
            icon: Bootstrap::Pencil,
            accent: theme::pink(),
            hotkey: None,
            running: false,
        },
        ModuleInfo {
            id: "image-resizer".into(),
            name: "Image Resizer".into(),
            description: "Batch resize images with presets or custom size".into(),
            icon: Bootstrap::Images,
            accent: theme::sapphire(),
            hotkey: None,
            running: false,
        },
        ModuleInfo {
            id: "key-manager".into(),
            name: "Key Manager".into(),
            description: "Remap keys, create shortcuts, per-app rules".into(),
            icon: Bootstrap::Keyboard,
            accent: theme::lavender(),
            hotkey: None,
            running: false,
        },
        ModuleInfo {
            id: "mouse-utils".into(),
            name: "Mouse Utilities".into(),
            description: "Find My Mouse, click highlighter, crosshair".into(),
            icon: Bootstrap::Mouse,
            accent: theme::yellow(),
            hotkey: Some("Ctrl+Ctrl".into()),
            running: false,
        },
        ModuleInfo {
            id: "screen-ruler".into(),
            name: "Screen Ruler".into(),
            description: "Measure pixels on screen".into(),
            icon: Bootstrap::Rulers,
            accent: theme::flamingo(),
            hotkey: Some("Super+Shift+R".into()),
            running: false,
        },
        ModuleInfo {
            id: "text-extractor".into(),
            name: "Text Extractor".into(),
            description: "OCR: extract text from any screen region".into(),
            icon: Bootstrap::Fonts,
            accent: theme::sky(),
            hotkey: Some("Super+Shift+T".into()),
            running: false,
        },
        ModuleInfo {
            id: "shortcut-guide".into(),
            name: "Shortcut Guide".into(),
            description: "Overlay showing available keyboard shortcuts".into(),
            icon: Bootstrap::Lightning,
            accent: theme::yellow(),
            hotkey: Some("Hold Super".into()),
            running: false,
        },
        ModuleInfo {
            id: "app-launcher".into(),
            name: "App Launcher".into(),
            description: "Quick app launcher with search and calculator".into(),
            icon: Bootstrap::Search,
            accent: theme::rosewater(),
            hotkey: Some("Alt+Space".into()),
            running: false,
        },
        ModuleInfo {
            id: "fancy-zones".into(),
            name: "FancyZones".into(),
            description: "Advanced window tiling with custom zone layouts".into(),
            icon: Bootstrap::Grid,
            accent: theme::blue(),
            hotkey: Some("Super+Shift+Z".into()),
            running: false,
        },
        ModuleInfo {
            id: "peek".into(),
            name: "Peek".into(),
            description: "Quick file preview (images, text, PDF, media)".into(),
            icon: Bootstrap::Eye,
            accent: theme::maroon(),
            hotkey: Some("Ctrl+Space".into()),
            running: false,
        },
    ]
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

    // Load system CJK font for Japanese / Chinese support
    if let Some(cjk) = load_cjk_font() {
        app = app.font(cjk);
    }

    app.run_with(Settings::new)
}
