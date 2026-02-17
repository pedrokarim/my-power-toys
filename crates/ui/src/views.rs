use iced::gradient;
use iced::widget::{
    Space, button, column, container, horizontal_rule, image, row, scrollable, stack, text,
    toggler,
};
use iced::{Alignment, Color, ContentFit, Element, Length, Padding, Theme};
use iced_fonts::bootstrap::Bootstrap;
use iced_fonts::BOOTSTRAP_FONT;

use crate::message::Message;
use crate::theme;
use crate::translations;
use crate::translations::Language;
use crate::types::*;
use crate::widgets::*;
use crate::Settings;

impl Settings {
    // ── Root view ───────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let content = row![self.view_sidebar(), self.view_content()].height(Length::Fill);

        match &self.visual_theme {
            VisualTheme::Default => content.into(),
            VisualTheme::Color(idx) => {
                let (_name, bg_rgb, _preview) = ACCENT_THEMES[*idx];
                let bg_c = Color::from_rgb8(bg_rgb[0], bg_rgb[1], bg_rgb[2]);
                let bg = container(Space::new(0, 0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_: &Theme| container::Style {
                        background: Some(bg_c.into()),
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    });
                stack![bg, content]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            VisualTheme::Gradient(idx) => {
                let (_name, angle_deg, start, mid, end) = GRADIENT_THEMES[*idx];
                let angle = angle_deg * std::f32::consts::PI / 180.0;
                let linear = gradient::Linear::new(angle)
                    .add_stop(0.0, Color::from_rgb8(start[0], start[1], start[2]))
                    .add_stop(0.5, Color::from_rgb8(mid[0], mid[1], mid[2]))
                    .add_stop(1.0, Color::from_rgb8(end[0], end[1], end[2]));
                let gradient_val = iced::Gradient::Linear(linear);
                let bg = container(Space::new(0, 0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_: &Theme| container::Style {
                        background: Some(gradient_val.into()),
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        text_color: None,
                    });
                stack![bg, content]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            VisualTheme::BuiltinImage(idx) => {
                let (_name, filename) = BUILTIN_BACKGROUNDS[*idx];
                let path = backgrounds_dir().join(filename);
                let bg = image(path.to_string_lossy().to_string())
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fill);
                let overlay = container(Space::new(0, 0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &Theme| container::Style {
                        background: Some(Color::from_rgba8(0, 0, 0, 0.45).into()),
                        ..container::Style::default()
                    });
                stack![bg, overlay, content]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            VisualTheme::CustomImage(path) => {
                let bg = image(path.to_string_lossy().to_string())
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fill);
                let overlay = container(Space::new(0, 0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &Theme| container::Style {
                        background: Some(Color::from_rgba8(0, 0, 0, 0.45).into()),
                        ..container::Style::default()
                    });
                stack![bg, overlay, content]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }

    // ── Sidebar ─────────────────────────────────────────────────────────────

    fn view_sidebar(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();

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
                    text("MyPowerToys").size(ui.sz(16.0)).font(bold()).color(ui.heading()),
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

        let mut nav = column![sidebar_icon_button(
            Bootstrap::Speedometer,
            tr.dashboard,
            self.page == Page::Dashboard,
            Message::NavigateTo(Page::Dashboard),
            ui,
        )]
        .spacing(2);

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
        .style(theme::sidebar(ui.glass))
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
            text("\u{25cf}").size(ui.sz(10.0)).color(dot_col),
            text(status_txt)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext1(ui.dark)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

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

        let mut modules_col = column![].spacing(8);
        for module in &self.modules {
            modules_col = modules_col.push(self.module_card(module));
        }

        column![
            text(tr.dashboard).size(ui.sz(28.0)).font(bold()).color(ui.heading()),
            status,
            Space::with_height(4),
            stats,
            Space::with_height(8),
            text(tr.all_modules).size(ui.sz(18.0)).font(bold()).color(ui.heading()),
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

                let header = row![
                    icon_badge(m.icon, m.accent, ui.sz(28.0)),
                    column![
                        text(&m.name).size(ui.sz(28.0)).font(bold()).color(ui.heading()),
                        text(&m.description)
                            .size(ui.sz(14.0))
                            .font(ui.font())
                            .color(theme::subtext1(ui.dark)),
                    ]
                    .spacing(4),
                ]
                .spacing(16)
                .align_y(Alignment::Center);

                let status_card = card(
                    column![
                        row![
                            text(tr.status)
                                .size(ui.sz(14.0))
                                .font(ui.font())
                                .color(theme::subtext0(ui.dark)),
                            Space::with_width(Length::Fill),
                            text("\u{25cf}").size(ui.sz(10.0)).color(dot_col),
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

                let settings_card = card(
                    column![
                        text(tr.module_settings).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
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
                text("MyPowerToys").size(ui.sz(28.0)).font(bold()).color(ui.heading()),
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
                text(tr.about_title).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
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
                text(tr.tech_stack).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
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
                text(tr.appearance).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
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

        // Visual theme card
        let theme_card = self.view_theme_card();

        // Language card
        let language_card = card(
            column![
                text(tr.language).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
                text(tr.lang_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button("Fran\u{00e7}ais", self.language == Language::Fr, Message::SetLanguage(Language::Fr), ui),
                        seg_button("English", self.language == Language::En, Message::SetLanguage(Language::En), ui),
                        seg_button("Espa\u{00f1}ol", self.language == Language::Es, Message::SetLanguage(Language::Es), ui),
                        seg_button("\u{65e5}\u{672c}\u{8a9e}", self.language == Language::Jp, Message::SetLanguage(Language::Jp), ui),
                        seg_button("\u{4e2d}\u{6587}", self.language == Language::Cn, Message::SetLanguage(Language::Cn), ui),
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
                text(tr.text_size).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
                text(tr.text_size_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button(tr.small, self.font_size == FontSize::Small, Message::SetFontSize(FontSize::Small), ui),
                        seg_button(tr.medium, self.font_size == FontSize::Medium, Message::SetFontSize(FontSize::Medium), ui),
                        seg_button(tr.large, self.font_size == FontSize::Large, Message::SetFontSize(FontSize::Large), ui),
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
                text(tr.accessibility).size(ui.sz(16.0)).font(bold()).color(ui.heading()),
                Space::with_height(4),
                pref_toggle(tr.high_contrast, tr.high_contrast_desc, self.high_contrast, Message::ToggleHighContrast, ui),
                pref_toggle(tr.bold_text, tr.bold_text_desc, self.bold_text, Message::ToggleBoldText, ui),
                pref_toggle(tr.compact_layout, tr.compact_layout_desc, self.compact_layout, Message::ToggleCompactLayout, ui),
                pref_toggle(tr.reduced_motion, tr.reduced_motion_desc, self.reduced_motion, Message::ToggleReducedMotion, ui),
            ]
            .spacing(12),
            ui,
        );

        column![
            text(tr.preferences).size(ui.sz(28.0)).font(bold()).color(ui.heading()),
            Space::with_height(4),
            appearance_card,
            theme_card,
            language_card,
            text_size_card,
            accessibility_card,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    // ── Theme picker card ────────────────────────────────────────────────────

    fn view_theme_card(&self) -> Element<'_, Message> {
        let ui = self.ui();
        let mut content = column![
            text("Visual Theme").size(ui.sz(16.0)).font(bold()).color(ui.heading()),
            text("Background style for the settings window")
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(4);

        // ── Default ──
        content = content.push(Space::with_height(8));
        content = content.push(
            text("Default")
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::overlay0(ui.dark)),
        );
        content = content.push(row![theme_swatch_btn(
            container(
                text("Catppuccin")
                    .size(ui.sz(10.0))
                    .color(theme::subtext0(ui.dark))
            )
            .width(90)
            .height(55)
            .center_x(90)
            .center_y(55)
            .style(theme::card(false, false)),
            "Default",
            self.visual_theme == VisualTheme::Default,
            Message::SetVisualTheme(VisualTheme::Default),
            ui,
        )]);

        // ── Colors ──
        content = content.push(Space::with_height(8));
        content = content.push(
            text("Colors")
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::overlay0(ui.dark)),
        );
        let mut color_row = row![].spacing(8);
        for (i, (name, _bg, preview)) in ACCENT_THEMES.iter().enumerate() {
            let c = Color::from_rgb8(preview[0], preview[1], preview[2]);
            color_row = color_row.push(theme_swatch_btn(
                container(Space::new(0, 0))
                    .width(90)
                    .height(55)
                    .style(theme::color_swatch(c)),
                name,
                self.visual_theme == VisualTheme::Color(i),
                Message::SetVisualTheme(VisualTheme::Color(i)),
                ui,
            ));
        }
        content = content.push(color_row);

        // ── Gradients ──
        content = content.push(Space::with_height(8));
        content = content.push(
            text("Gradients")
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::overlay0(ui.dark)),
        );
        let mut grad_row = row![].spacing(8);
        for (i, (name, _angle, start, _mid, end)) in GRADIENT_THEMES.iter().enumerate() {
            let blend = Color::from_rgb(
                (start[0] as f32 + end[0] as f32) / 510.0,
                (start[1] as f32 + end[1] as f32) / 510.0,
                (start[2] as f32 + end[2] as f32) / 510.0,
            );
            grad_row = grad_row.push(theme_swatch_btn(
                container(Space::new(0, 0))
                    .width(90)
                    .height(55)
                    .style(theme::color_swatch(blend)),
                name,
                self.visual_theme == VisualTheme::Gradient(i),
                Message::SetVisualTheme(VisualTheme::Gradient(i)),
                ui,
            ));
        }
        content = content.push(grad_row);

        // ── Backgrounds ──
        content = content.push(Space::with_height(8));
        content = content.push(
            text("Backgrounds")
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::overlay0(ui.dark)),
        );
        let bg_dir = thumbnails_dir();
        let mut row1 = row![].spacing(8);
        let mut row2 = row![].spacing(8);
        for (i, (name, filename)) in BUILTIN_BACKGROUNDS.iter().enumerate() {
            let path = bg_dir.join(filename);
            let swatch = theme_swatch_btn(
                image(path.to_string_lossy().to_string())
                    .content_fit(ContentFit::Cover)
                    .width(100)
                    .height(60),
                name,
                self.visual_theme == VisualTheme::BuiltinImage(i),
                Message::SetVisualTheme(VisualTheme::BuiltinImage(i)),
                ui,
            );
            if i < 6 {
                row1 = row1.push(swatch);
            } else {
                row2 = row2.push(swatch);
            }
        }
        content = content.push(column![row1, row2].spacing(8));

        // ── Custom Image ──
        content = content.push(Space::with_height(8));
        content = content.push(
            text("Custom Image")
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::overlay0(ui.dark)),
        );
        content = content.push(
            button(
                row![
                    text(Bootstrap::FolderFill.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(ui.sz(14.0)),
                    text("Choose image...").size(ui.sz(13.0)).font(ui.font()),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .on_press(Message::PickCustomImage)
            .padding(Padding::from([8.0, 16.0]))
            .style(theme::seg_button(false)),
        );

        if !self.custom_image_history.is_empty() {
            content = content.push(Space::with_height(4));
            let mut history_row = row![].spacing(8);
            for path in self.custom_image_history.iter().take(6) {
                let is_sel = self.visual_theme == VisualTheme::CustomImage(path.clone());
                let label = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                history_row = history_row.push(theme_swatch_btn(
                    image(path.to_string_lossy().to_string())
                        .content_fit(ContentFit::Cover)
                        .width(100)
                        .height(60),
                    &label,
                    is_sel,
                    Message::SetVisualTheme(VisualTheme::CustomImage(path.clone())),
                    ui,
                ));
            }
            content = content.push(history_row);
        }

        card(content, ui)
    }
}
