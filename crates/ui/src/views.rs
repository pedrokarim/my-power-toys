use iced::gradient;
use iced::widget::{
    Space, button, column, container, horizontal_rule, image, progress_bar, row, scrollable, stack,
    text, toggler,
};
use iced::{Alignment, Color, ContentFit, Element, Length, Padding, Theme};
use iced_fonts::BOOTSTRAP_FONT;
use iced_fonts::bootstrap::Bootstrap;
use std::time::Instant;

use crate::helpers;
use crate::message::Message;
use crate::theme;
use crate::translations;
use crate::translations::Language;
use crate::types::*;
use crate::widgets::*;
use crate::{Settings, UpdateState};

impl Settings {
    // ── Root view ───────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let content = row![self.view_sidebar(), self.view_content()].height(Length::Fill);

        let base: Element<'_, Message> = match &self.visual_theme {
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
        };

        let base = self.with_update_dialog_overlay(base);
        self.with_toast_overlay(base)
    }

    fn with_update_dialog_overlay<'a>(
        &'a self,
        base: Element<'a, Message>,
    ) -> Element<'a, Message> {
        if !self.update_dialog_open {
            return base;
        }

        let tr = translations::get(self.language);
        let ui = self.ui();
        let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));

        let dialog_content: Element<'a, Message> = match &self.update_state {
            UpdateState::Available { latest_version } => {
                let latest = format!("v{latest_version}");
                let cancel_button = button(
                    text(tr.update_dialog_cancel)
                        .size(ui.sz(12.0))
                        .font(ui.font()),
                )
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(false))
                .on_press(Message::CloseUpdateDialog);

                let confirm_button = button(
                    text(tr.update_dialog_confirm)
                        .size(ui.sz(12.0))
                        .font(ui.font()),
                )
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(true))
                .on_press(Message::ConfirmUpdateInstall);

                column![
                    text(tr.update_dialog_title)
                        .size(ui.sz(18.0))
                        .font(bold())
                        .color(ui.heading()),
                    text(tr.update_dialog_body)
                        .size(ui.sz(13.0))
                        .font(ui.font())
                        .color(theme::subtext1(ui.dark)),
                    info_row(tr.update_current_version, &current_version, ui),
                    info_row(tr.update_latest_version, &latest, ui),
                    row![
                        Space::with_width(Length::Fill),
                        cancel_button,
                        confirm_button
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(10)
                .into()
            }
            UpdateState::Updating { target_version } => {
                let latest = format!("v{target_version}");
                let pulse = 1.0 - (self.update_progress - 1.0).abs();
                column![
                    text(tr.update_dialog_title)
                        .size(ui.sz(18.0))
                        .font(bold())
                        .color(ui.heading()),
                    text(tr.update_dialog_downloading)
                        .size(ui.sz(13.0))
                        .font(ui.font())
                        .color(theme::subtext1(ui.dark)),
                    info_row(tr.update_current_version, &current_version, ui),
                    info_row(tr.update_latest_version, &latest, ui),
                    Space::with_height(4),
                    progress_bar(0.0..=1.0, pulse).height(6).width(Length::Fill),
                ]
                .spacing(10)
                .into()
            }
            UpdateState::Restarting { new_version } => {
                let latest = format!("v{new_version}");
                column![
                    row![
                        text(Bootstrap::CheckCircleFill.to_string())
                            .font(BOOTSTRAP_FONT)
                            .size(ui.sz(22.0))
                            .color(theme::green()),
                        text(tr.update_restarting)
                            .size(ui.sz(18.0))
                            .font(bold())
                            .color(ui.heading()),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    info_row(tr.update_latest_version, &latest, ui),
                    Space::with_height(4),
                    progress_bar(0.0..=1.0, 1.0).height(6).width(Length::Fill),
                ]
                .spacing(10)
                .into()
            }
            _ => return base,
        };

        let dialog = container(dialog_content)
            .padding(ui.pad(16.0) as u16)
            .width(ui.sz(460.0))
            .style(theme::card(ui.contrast, false));

        let backdrop = container(Space::new(0, 0))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgba8(0, 0, 0, 0.55).into()),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: None,
            });

        let overlay = container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding::from([24.0, 24.0]));

        stack![base, backdrop, overlay]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn with_toast_overlay<'a>(&'a self, base: Element<'a, Message>) -> Element<'a, Message> {
        let Some(toast) = self.toast.as_ref() else {
            return base;
        };

        let ui = self.ui();
        let progress = toast.remaining_progress(Instant::now());
        let (icon, accent) = match toast.kind {
            ToastKind::Success => (Bootstrap::CheckCircleFill, theme::green()),
            ToastKind::Error => (Bootstrap::ExclamationCircleFill, theme::red()),
        };

        let mut header = row![
            container(
                text(icon.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(ui.sz(17.0))
                    .color(accent)
            )
            .width(ui.sz(26.0))
            .height(ui.sz(26.0))
            .center_x(Length::Shrink)
            .center_y(Length::Shrink),
            column![
                text(toast.title.clone())
                    .size(ui.sz(13.0))
                    .font(bold())
                    .color(ui.heading()),
                text(toast.message.clone())
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext1(ui.dark)),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Alignment::Start);

        if !self.toast_queue.is_empty() {
            header = header.push(
                container(
                    text(format!("+{}", self.toast_queue.len()))
                        .size(ui.sz(10.0))
                        .font(ui.font())
                        .color(theme::subtext0(ui.dark)),
                )
                .padding(Padding::from([2.0, 7.0]))
                .style(theme::kbd(ui.contrast, ui.glass)),
            );
        }

        let close_button = button(text("x").size(ui.sz(12.0)).font(bold()))
            .on_press(Message::DismissToast)
            .padding(Padding::from([2.0, 7.0]))
            .style(theme::toast_close_button());

        header = header.push(close_button);

        let toast = container(
            column![
                header,
                progress_bar(0.0..=1.0, progress)
                    .height(4)
                    .width(Length::Fill),
            ]
            .spacing(10),
        )
        .padding(Padding::from([12.0, 14.0]))
        .width(ui.sz(380.0))
        .style(theme::toast_card(toast.kind, ui.contrast, ui.glass));

        let overlay = container(
            column![
                Space::with_height(Length::Fill),
                row![Space::with_width(Length::Fill), toast]
                    .align_y(Alignment::End)
                    .spacing(8),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 24.0,
            bottom: 24.0,
            left: 0.0,
        });

        stack![base, overlay]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ── Sidebar ─────────────────────────────────────────────────────────────

    fn view_sidebar(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();

        let logo = image(
            assets_dir()
                .join("icons/icon-64.png")
                .to_string_lossy()
                .to_string(),
        )
        .width(40)
        .height(40);

        let header = container(
            row![
                logo,
                column![
                    text("MyPowerToys")
                        .size(ui.sz(16.0))
                        .font(bold())
                        .color(ui.heading()),
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
            Bootstrap::Lightning,
            tr.tests,
            self.page == Page::Tests,
            Message::NavigateTo(Page::Tests),
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
            Page::Tests => self.view_tests(),
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

        let mut status = row![
            text("\u{25cf}").size(ui.sz(10.0)).color(dot_col),
            text(status_txt)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext1(ui.dark)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        if !self.daemon_connected {
            status = status.push(
                button(text(tr.start_daemon).size(ui.sz(11.0)).font(ui.font()))
                    .on_press(Message::StartDaemon)
                    .padding(Padding::from([4.0, 10.0]))
                    .style(theme::seg_button(false)),
            );
        }

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
            text(tr.dashboard)
                .size(ui.sz(28.0))
                .font(bold())
                .color(ui.heading()),
            status,
            Space::with_height(4),
            stats,
            Space::with_height(8),
            text(tr.all_modules)
                .size(ui.sz(18.0))
                .font(bold())
                .color(ui.heading()),
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
                // Banner image
                let banner_path = assets_dir().join(format!("banners/{}-banner.png", id));
                let has_banner = banner_path.exists();

                let banner_hero = if has_banner {
                    let bg = image(banner_path.to_string_lossy().to_string())
                        .content_fit(ContentFit::Cover)
                        .width(Length::Fill)
                        .height(Length::Fill);

                    // Gradient overlay at bottom for text readability
                    let gradient_overlay = container(Space::new(0, 0))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_: &Theme| container::Style {
                            background: Some(
                                gradient::Linear::new(std::f32::consts::PI)
                                    .add_stop(0.0, Color::TRANSPARENT)
                                    .add_stop(0.55, Color::TRANSPARENT)
                                    .add_stop(1.0, Color::from_rgba8(0, 0, 0, 0.75))
                                    .into(),
                            ),
                            ..container::Style::default()
                        });

                    // Module title overlaid on banner bottom
                    let title_overlay = container(
                        row![
                            icon_badge(m.icon, m.accent, ui.sz(26.0)),
                            column![
                                text(&m.name)
                                    .size(ui.sz(26.0))
                                    .font(bold())
                                    .color(Color::WHITE),
                                text(&m.description)
                                    .size(ui.sz(13.0))
                                    .font(ui.font())
                                    .color(Color::from_rgba8(255, 255, 255, 0.7)),
                            ]
                            .spacing(2),
                        ]
                        .spacing(14)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(iced::alignment::Vertical::Bottom)
                    .padding(Padding::from([16.0, 20.0]));

                    Some(
                        container(
                            stack![bg, gradient_overlay, title_overlay]
                                .width(Length::Fill)
                                .height(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(280)
                        .clip(true)
                        .style(theme::banner_card()),
                    )
                } else {
                    None
                };

                let dot_col = if m.running {
                    theme::green()
                } else {
                    theme::overlay0(ui.dark)
                };
                let status_txt = if m.running { tr.running } else { tr.stopped };

                // Header only shown when no banner (fallback)
                let header = if !has_banner {
                    Some(
                        row![
                            icon_badge(m.icon, m.accent, ui.sz(28.0)),
                            column![
                                text(&m.name)
                                    .size(ui.sz(28.0))
                                    .font(bold())
                                    .color(ui.heading()),
                                text(&m.description)
                                    .size(ui.sz(14.0))
                                    .font(ui.font())
                                    .color(theme::subtext1(ui.dark)),
                            ]
                            .spacing(4),
                        ]
                        .spacing(16)
                        .align_y(Alignment::Center),
                    )
                } else {
                    None
                };

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

                let settings_card = self.view_module_settings(id, tr, ui);

                let help_open = self.dependency_help_for.as_deref() == Some(m.id.as_str());
                let mut test_header = row![
                    text(tr.tests)
                        .size(ui.sz(16.0))
                        .font(bold())
                        .color(ui.heading()),
                    Space::with_width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                if m.hotkey.is_some() {
                    let trigger_btn = {
                        let base = button(text(tr.test_action).size(ui.sz(12.0)).font(ui.font()))
                            .padding(Padding::from([6.0, 12.0]))
                            .style(theme::seg_button(false));
                        if self.daemon_connected {
                            base.on_press(Message::TriggerHotkeyTest(m.id.clone()))
                        } else {
                            base
                        }
                    };
                    test_header = test_header.push(trigger_btn);
                }

                if self.has_dependency_help(&m.id) {
                    let help_btn = button(
                        row![
                            text(Bootstrap::InfoCircle.to_string())
                                .font(BOOTSTRAP_FONT)
                                .size(ui.sz(12.0)),
                            text(if help_open {
                                tr.deps_hide
                            } else {
                                tr.deps_help
                            })
                            .size(ui.sz(12.0))
                            .font(ui.font()),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding::from([6.0, 10.0]))
                    .style(theme::seg_button(false))
                    .on_press(Message::ToggleDependencyHelp(m.id.clone()));
                    test_header = test_header.push(help_btn);
                }

                let mut test_content = column![test_header].spacing(8);

                if let Some(hk) = m.hotkey.as_ref() {
                    let result = self
                        .shortcut_test_results
                        .get(&m.id)
                        .map(String::as_str)
                        .unwrap_or("-");
                    let result_col = if result == "ok" {
                        theme::green()
                    } else if result == "pending" {
                        theme::blue()
                    } else if result.starts_with("error") {
                        theme::red()
                    } else {
                        theme::subtext0(ui.dark)
                    };

                    test_content = test_content.push(
                        row![
                            text(format!("{}:", tr.hotkey))
                                .size(ui.sz(12.0))
                                .font(ui.font())
                                .color(theme::subtext0(ui.dark)),
                            kbd(hk, ui),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    );
                    test_content = test_content.push(
                        text(format!("{}: {}", tr.test_result, result))
                            .size(ui.sz(12.0))
                            .font(ui.font())
                            .color(result_col),
                    );
                } else {
                    test_content = test_content.push(
                        text(tr.tests_no_hotkey)
                            .size(ui.sz(12.0))
                            .font(ui.font())
                            .color(theme::subtext0(ui.dark)),
                    );
                }

                if help_open && let Some(help_card) = self.view_dependency_help_card(&m.id) {
                    test_content = test_content.push(help_card);
                }

                let tests_card = card(test_content, ui);

                let mut content = column![].spacing(12);
                if let Some(b) = banner_hero {
                    content = content.push(b);
                }
                if let Some(h) = header {
                    content = content.push(h);
                    content = content.push(Space::with_height(4));
                }
                content = content.push(status_card);
                if let Some(hk) = hotkey_card {
                    content = content.push(hk);
                }
                content = content.push(settings_card);
                content = content.push(tests_card);
                content.width(Length::Fill).into()
            }
            None => text(tr.module_not_found).size(ui.sz(20.0)).into(),
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    fn view_tests(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let keys_under_test = self.collect_test_keys();

        let mut keys_rows = column![].spacing(6);
        if keys_under_test.is_empty() {
            keys_rows = keys_rows.push(
                text(tr.tests_no_keys)
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            );
        } else {
            for chunk in keys_under_test.chunks(8) {
                let mut line = row![].spacing(6);
                for key in chunk {
                    let active = self.test_active_keys.iter().any(|k| k == key);
                    line = line.push(key_cap(key, active, ui));
                }
                keys_rows = keys_rows.push(line);
            }
        }

        let active_keys = if self.test_active_keys.is_empty() {
            "-".to_string()
        } else {
            self.test_active_keys.join(" + ")
        };

        let test_toggle_btn = if self.hotkey_test_active {
            button(
                row![
                    text(Bootstrap::StopCircle.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(ui.sz(12.0)),
                    text(tr.tests_stop_btn).size(ui.sz(11.0)).font(ui.font()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([5.0, 10.0]))
            .style(theme::seg_button(true))
            .on_press(Message::StopHotkeyTest)
        } else {
            button(
                row![
                    text(Bootstrap::PlayCircle.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(ui.sz(12.0)),
                    text(tr.tests_start_btn).size(ui.sz(11.0)).font(ui.font()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([5.0, 10.0]))
            .style(theme::seg_button(false))
            .on_press(Message::StartHotkeyTest)
        };

        let keys_card = card(
            column![
                row![
                    text(tr.tests_keys_title)
                        .size(ui.sz(16.0))
                        .font(bold())
                        .color(ui.heading()),
                    Space::with_width(Length::Fill),
                    test_toggle_btn,
                ]
                .align_y(Alignment::Center),
                text(tr.tests_keys_desc)
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                keys_rows,
                text(format!("{}: {}", tr.tests_active_keys, active_keys))
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(if self.hotkey_test_active {
                        theme::blue()
                    } else {
                        theme::subtext1(ui.dark)
                    }),
            ]
            .spacing(8),
            ui,
        );

        let mut tests = column![].spacing(8);
        let mut has_shortcut = false;

        for module in self.modules.iter().filter(|m| m.hotkey.is_some()) {
            has_shortcut = true;
            let hk = module.hotkey.as_deref().unwrap_or("-");
            let status_txt = if module.running {
                tr.running
            } else {
                tr.stopped
            };
            let status_col = if module.running {
                theme::green()
            } else {
                theme::overlay0(ui.dark)
            };

            let result = self
                .shortcut_test_results
                .get(&module.id)
                .map(String::as_str)
                .unwrap_or("-");
            let result_col = if result == "ok" {
                theme::green()
            } else if result == "pending" {
                theme::blue()
            } else if result.starts_with("error") {
                theme::red()
            } else {
                theme::subtext0(ui.dark)
            };

            let trigger_btn = {
                let base = button(text(tr.test_action).size(ui.sz(12.0)).font(ui.font()))
                    .padding(Padding::from([6.0, 12.0]))
                    .style(theme::seg_button(false));
                if self.daemon_connected {
                    base.on_press(Message::TriggerHotkeyTest(module.id.clone()))
                } else {
                    base
                }
            };

            let help_open = self.dependency_help_for.as_deref() == Some(module.id.as_str());
            let help_btn: Element<'_, Message> = if self.has_dependency_help(&module.id) {
                button(
                    text(Bootstrap::InfoCircle.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(ui.sz(12.0)),
                )
                .padding(Padding::from([6.0, 10.0]))
                .style(theme::seg_button(false))
                .on_press(Message::ToggleDependencyHelp(module.id.clone()))
                .into()
            } else {
                Space::with_width(0).into()
            };

            let mut card_content = column![
                row![
                    icon_badge(module.icon, module.accent, ui.sz(16.0)),
                    column![
                        text(module.name.clone()).size(ui.sz(14.0)).font(ui.font()),
                        row![
                            text(format!("{}:", tr.hotkey))
                                .size(ui.sz(12.0))
                                .font(ui.font())
                                .color(theme::subtext0(ui.dark)),
                            kbd(hk, ui),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(4)
                    .width(Length::Fill),
                    row![trigger_btn, help_btn]
                        .spacing(6)
                        .align_y(Alignment::Center),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text(format!("{}: {}", tr.status, status_txt))
                        .size(ui.sz(12.0))
                        .font(ui.font())
                        .color(status_col),
                    Space::with_width(Length::Fill),
                    text(format!("{}: {}", tr.test_result, result))
                        .size(ui.sz(12.0))
                        .font(ui.font())
                        .color(result_col),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(10);

            if help_open && let Some(help_card) = self.view_dependency_help_card(&module.id) {
                card_content = card_content.push(help_card);
            }

            tests = tests.push(card(card_content, ui));
        }

        let daemon_hint = if self.daemon_connected {
            text(tr.daemon_connected)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::green())
        } else {
            text(tr.daemon_required)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::red())
        };

        let wayland_hint: Option<Element<'_, Message>> =
            if self.display_server == mpt_common::platform::DisplayServer::Wayland {
                Some(
                    text(tr.tests_wayland_hint)
                        .size(ui.sz(12.0))
                        .font(ui.font())
                        .color(theme::yellow())
                        .into(),
                )
            } else {
                None
            };

        let body: Element<'_, Message> = if has_shortcut {
            tests.into()
        } else {
            text(tr.no_shortcuts)
                .size(ui.sz(14.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark))
                .into()
        };

        let mut content = column![
            text(tr.tests_title)
                .size(ui.sz(28.0))
                .font(bold())
                .color(ui.heading()),
            text(tr.tests_desc)
                .size(ui.sz(14.0))
                .font(ui.font())
                .color(theme::subtext1(ui.dark)),
            daemon_hint,
        ]
        .spacing(12);

        if let Some(hint) = wayland_hint {
            content = content.push(hint);
        }

        content = content
            .push(keys_card)
            .push(Space::with_height(4))
            .push(body)
            .push(Space::with_height(16))
            .push(self.view_design_system());

        content.width(Length::Fill).into()
    }

    // ── Design System showcase (Tests page only) ─────────────────────────

    fn view_design_system(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();

        // ── Section header ───────────────────────────────────────────────
        let header = column![
            horizontal_rule(1),
            Space::with_height(8),
            text(tr.design_system)
                .size(ui.sz(28.0))
                .font(bold())
                .color(ui.heading()),
            text("Visual reference for every UI primitive used in the app.")
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(4);

        // ── 1. Color palette ─────────────────────────────────────────────
        let colors: Vec<(&str, Color)> = vec![
            ("Green", theme::green()),
            ("Red", theme::red()),
            ("Blue", theme::blue()),
            ("Mauve", theme::mauve()),
            ("Pink", theme::pink()),
            ("Teal", theme::teal()),
            ("Yellow", theme::yellow()),
            ("Peach", theme::peach()),
            ("Sky", theme::sky()),
            ("Lavender", theme::lavender()),
            ("Flamingo", theme::flamingo()),
            ("Rosewater", theme::rosewater()),
            ("Maroon", theme::maroon()),
            ("Sapphire", theme::sapphire()),
        ];

        let mut color_row = row![].spacing(8);
        for (name, color) in &colors {
            let swatch = container(Space::new(0, 0))
                .width(ui.sz(36.0))
                .height(ui.sz(36.0))
                .style(theme::color_swatch(*color));
            color_row = color_row.push(
                column![
                    swatch,
                    text(name.to_string())
                        .size(ui.sz(10.0))
                        .font(ui.font())
                        .color(theme::subtext0(ui.dark)),
                ]
                .spacing(4)
                .align_x(Alignment::Center),
            );
        }

        let semantic_colors = row![
            column![
                text("overlay0")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::overlay0(ui.dark)),
                text("subtext0")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                text("subtext1")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::subtext1(ui.dark)),
                text("heading")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(ui.heading()),
            ]
            .spacing(4),
        ];

        let colors_card = card(
            column![
                text(tr.ds_colors)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Catppuccin accent palette")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                color_row,
                text("Semantic text colors")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                semantic_colors,
            ]
            .spacing(10),
            ui,
        );

        // ── 2. Typography ────────────────────────────────────────────────
        let typo_card = card(
            column![
                text(tr.ds_typography)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Heading 28px")
                    .size(ui.sz(28.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Title 16px bold")
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Body 14px").size(ui.sz(14.0)).font(ui.font()),
                text("Body 13px").size(ui.sz(13.0)).font(ui.font()),
                text("Caption 12px")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                text("Small 10px")
                    .size(ui.sz(10.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                text("Bold variant").size(ui.sz(14.0)).font(bold()),
            ]
            .spacing(6),
            ui,
        );

        // ── 3. Buttons ──────────────────────────────────────────────────
        let btn_active = button(text("Active").size(ui.sz(11.0)).font(ui.font()))
            .padding(Padding::from([5.0, 10.0]))
            .style(theme::seg_button(true));

        let btn_inactive = button(text("Inactive").size(ui.sz(11.0)).font(ui.font()))
            .padding(Padding::from([5.0, 10.0]))
            .style(theme::seg_button(false));

        let btn_nav_sel = button(
            row![
                text(Bootstrap::HouseDoor.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(ui.sz(14.0))
                    .color(theme::blue()),
                text("Selected nav").size(ui.sz(13.0)).font(ui.font()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(ui.sz(180.0))
        .padding(Padding::from([8.0, 12.0]))
        .style(theme::nav_button(true));

        let btn_nav_idle = button(
            row![
                text(Bootstrap::Gear.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(ui.sz(14.0))
                    .color(theme::overlay0(ui.dark)),
                text("Idle nav").size(ui.sz(13.0)).font(ui.font()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(ui.sz(180.0))
        .padding(Padding::from([8.0, 12.0]))
        .style(theme::nav_button(false));

        let buttons_card = card(
            column![
                text(tr.ds_buttons)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Segmented buttons")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                row![btn_active, btn_inactive].spacing(8),
                text("Navigation buttons")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                row![btn_nav_sel, btn_nav_idle].spacing(8),
            ]
            .spacing(10),
            ui,
        );

        // ── 4. Cards ────────────────────────────────────────────────────
        let normal_card = card(
            column![
                text("Standard card")
                    .size(ui.sz(14.0))
                    .font(bold())
                    .color(ui.heading()),
                text("theme::card — rounded corners, dark surface background")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(4),
            ui,
        );

        let stat_example = row![
            stat_card("Total", "15", theme::blue(), ui),
            stat_card("Active", "8", theme::green(), ui),
            stat_card("Errors", "2", theme::red(), ui),
        ]
        .spacing(8);

        let cards_card = card(
            column![
                text(tr.ds_cards)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Standard card")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                normal_card,
                text("Stat cards")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                stat_example,
            ]
            .spacing(10),
            ui,
        );

        // ── 5. Badges & key caps ────────────────────────────────────────
        let badge_row = row![
            icon_badge(Bootstrap::Palette, theme::blue(), ui.sz(16.0)),
            icon_badge(Bootstrap::Lightning, theme::yellow(), ui.sz(16.0)),
            icon_badge(Bootstrap::Shield, theme::green(), ui.sz(16.0)),
            icon_badge(Bootstrap::Bug, theme::red(), ui.sz(16.0)),
            icon_badge(Bootstrap::Star, theme::mauve(), ui.sz(16.0)),
        ]
        .spacing(8);

        let kbd_row = row![
            kbd("Ctrl", ui),
            kbd("Alt", ui),
            kbd("Shift", ui),
            kbd("Super", ui),
            kbd("F1", ui),
        ]
        .spacing(6);

        let keycap_row = row![
            key_cap("A", true, ui),
            key_cap("B", false, ui),
            key_cap("C", true, ui),
            key_cap("D", false, ui),
        ]
        .spacing(6);

        let badges_card = card(
            column![
                text(tr.ds_badges)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Icon badges")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                badge_row,
                text("Keyboard shortcut badges")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                kbd_row,
                text("Key caps (active / inactive)")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                keycap_row,
            ]
            .spacing(10),
            ui,
        );

        // ── 6. Controls (toggler, info rows, progress bar) ──────────────
        let toggle_row = row![
            column![
                text(tr.ds_sample_label).size(ui.sz(14.0)).font(ui.font()),
                text("Toggle description")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(true).size(ui.sz(22.0)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let toggle_row_off = row![
            column![
                text("Disabled option").size(ui.sz(14.0)).font(ui.font()),
                text("This one is off")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(false).size(ui.sz(22.0)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let info_rows = column![
            info_row("Author", "Ahmed Karim", ui),
            info_row("License", "GPL-3.0", ui),
            info_row("Framework", "iced (Rust)", ui),
        ]
        .spacing(8);

        let progress_bars = column![
            row![
                text("25%")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                progress_bar(0.0..=1.0, 0.25).height(4).width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("60%")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                progress_bar(0.0..=1.0, 0.60).height(4).width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("100%")
                    .size(ui.sz(11.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                progress_bar(0.0..=1.0, 1.0).height(4).width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(6);

        let controls_card = card(
            column![
                text(tr.ds_inputs)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text("Togglers")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                toggle_row,
                toggle_row_off,
                horizontal_rule(1),
                text("Info rows")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                info_rows,
                horizontal_rule(1),
                text("Progress bars")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                progress_bars,
            ]
            .spacing(10),
            ui,
        );

        column![
            header,
            colors_card,
            typo_card,
            buttons_card,
            cards_card,
            badges_card,
            controls_card,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    fn collect_test_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for hotkey in self.modules.iter().filter_map(|m| m.hotkey.as_deref()) {
            for token in Self::parse_hotkey_tokens(hotkey) {
                if !keys.contains(&token) {
                    keys.push(token);
                }
            }
        }
        keys.sort_by(|a, b| Self::test_key_sort(a).cmp(&Self::test_key_sort(b)));
        keys
    }

    fn parse_hotkey_tokens(hotkey: &str) -> Vec<String> {
        let trimmed = hotkey.trim();
        if trimmed.is_empty() {
            return vec![];
        }

        let mut parts = Vec::new();
        let lower = trimmed.to_lowercase();
        if let Some(_rest) = lower.strip_prefix("hold ") {
            // e.g. "Hold Super"
            let original = trimmed[5..].trim();
            parts.push(Self::normalize_test_key(original));
        } else {
            for part in trimmed.split('+') {
                parts.push(Self::normalize_test_key(part));
            }
        }

        let mut uniq = Vec::new();
        for p in parts {
            if !p.is_empty() && !uniq.contains(&p) {
                uniq.push(p);
            }
        }
        uniq
    }

    fn normalize_test_key(raw: &str) -> String {
        let token = raw.trim();
        if token.is_empty() {
            return String::new();
        }

        let lower = token.to_lowercase();
        match lower.as_str() {
            "control" | "ctrl" => "Ctrl".to_string(),
            "super" | "meta" | "win" => "Super".to_string(),
            "alt" => "Alt".to_string(),
            "shift" => "Shift".to_string(),
            "space" => "Space".to_string(),
            _ if token.len() == 1 => token.to_uppercase(),
            _ => token.to_string(),
        }
    }

    fn test_key_sort(key: &str) -> (u8, String) {
        let rank = match key {
            "Super" => 0,
            "Ctrl" => 1,
            "Alt" => 2,
            "Shift" => 3,
            _ => 10,
        };
        (rank, key.to_lowercase())
    }

    fn has_dependency_help(&self, module_id: &str) -> bool {
        matches!(module_id, "text-extractor" | "color-picker" | "paste-plain")
    }

    fn dependency_packages(&self, module_id: &str) -> Option<Vec<&'static str>> {
        use helpers::PackageManager;

        let packages = match module_id {
            "text-extractor" => match self.package_manager {
                PackageManager::Apt => {
                    vec![
                        "tesseract-ocr",
                        "wl-clipboard",
                        "xclip",
                        "gnome-screenshot",
                        "scrot",
                    ]
                }
                _ => vec![
                    "tesseract",
                    "wl-clipboard",
                    "xclip",
                    "gnome-screenshot",
                    "scrot",
                ],
            },
            "color-picker" => vec![
                "wl-clipboard",
                "xclip",
                "xdotool",
                "gnome-screenshot",
                "scrot",
            ],
            "paste-plain" => vec!["wl-clipboard", "xclip", "xdotool"],
            _ => return None,
        };
        Some(packages)
    }

    fn dependency_notes_for_module(&self, module_id: &str) -> Vec<String> {
        let mut notes = match module_id {
            "text-extractor" => vec![
                "Le module a besoin de Tesseract pour faire l'OCR.".to_string(),
                "Pour Wayland wlroots, vous pouvez aussi installer grim + slurp.".to_string(),
            ],
            "color-picker" => vec![
                "Le module a besoin d'un outil de capture d'écran et du presse-papiers."
                    .to_string(),
                "Le test lit la position souris via xdotool.".to_string(),
            ],
            "paste-plain" => vec![
                "Le module utilise wl-clipboard/xclip pour le presse-papiers.".to_string(),
                "Le collage est simulé via xdotool (Ctrl+V).".to_string(),
            ],
            _ => vec![],
        };

        if self.display_server == mpt_common::platform::DisplayServer::Wayland
            && matches!(module_id, "color-picker" | "paste-plain")
        {
            notes.push(
                "Sous Wayland, xdotool peut etre limité selon votre session. Si besoin, testez en session X11."
                    .to_string(),
            );
        }

        notes
    }

    fn install_command(&self, packages: &[&str]) -> String {
        use helpers::PackageManager;

        match self.package_manager {
            PackageManager::Apt => format!("sudo apt install {}", packages.join(" ")),
            PackageManager::Pacman => format!("sudo pacman -S --needed {}", packages.join(" ")),
            PackageManager::Dnf => format!("sudo dnf install {}", packages.join(" ")),
            PackageManager::Zypper => format!("sudo zypper install {}", packages.join(" ")),
            PackageManager::Unknown => format!("install: {}", packages.join(" ")),
        }
    }

    fn view_dependency_help_card(&self, module_id: &str) -> Option<Element<'_, Message>> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let packages = self.dependency_packages(module_id)?;
        let command = self.install_command(&packages);
        let system_label = format!(
            "{} • {}",
            self.distro_name,
            helpers::display_server_label(self.display_server)
        );

        let mut notes_col = column![
            text(format!("{}:", tr.deps_notes))
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark))
        ]
        .spacing(4);

        for note in self.dependency_notes_for_module(module_id) {
            notes_col = notes_col.push(
                text(format!("• {note}"))
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            );
        }

        if self.package_manager == helpers::PackageManager::Unknown {
            notes_col = notes_col.push(
                text("• Gestionnaire de paquets non détecté automatiquement.")
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            );
        }

        let copy_btn = button(text(tr.deps_copy).size(ui.sz(12.0)).font(ui.font()))
            .padding(Padding::from([6.0, 10.0]))
            .style(theme::seg_button(false))
            .on_press(Message::CopyInstallCommand(command.clone()));

        let continue_btn = button(text(tr.deps_continue).size(ui.sz(12.0)).font(ui.font()))
            .padding(Padding::from([6.0, 12.0]))
            .style(theme::seg_button(false))
            .on_press(Message::CloseDependencyHelp);

        Some(card(
            column![
                text(tr.deps_title)
                    .size(ui.sz(14.0))
                    .font(bold())
                    .color(ui.heading()),
                text(format!("{}: {}", tr.deps_for_system, system_label))
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                text(format!("{}:", tr.deps_command))
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                row![
                    container(text(command).size(ui.sz(12.0)).font(ui.font()))
                        .padding(Padding::from([6.0, 10.0]))
                        .width(Length::Fill)
                        .style(theme::kbd(ui.contrast, ui.glass)),
                    copy_btn,
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                notes_col,
                row![Space::with_width(Length::Fill), continue_btn]
                    .spacing(8)
                    .align_y(Alignment::Center),
            ]
            .spacing(8),
            ui,
        ))
    }

    // ── About ───────────────────────────────────────────────────────────────

    fn view_about(&self) -> Element<'_, Message> {
        let tr = translations::get(self.language);
        let ui = self.ui();
        let (status_text, status_color, latest_version, error_detail, busy, can_install) =
            match &self.update_state {
                UpdateState::Unknown => (
                    tr.update_not_checked.to_string(),
                    theme::overlay0(ui.dark),
                    None,
                    None,
                    false,
                    false,
                ),
                UpdateState::Checking => (
                    tr.update_checking.to_string(),
                    theme::overlay0(ui.dark),
                    None,
                    None,
                    true,
                    false,
                ),
                UpdateState::UpToDate => (
                    tr.update_up_to_date.to_string(),
                    theme::green(),
                    None,
                    None,
                    false,
                    false,
                ),
                UpdateState::Available { latest_version } => (
                    tr.update_available.to_string(),
                    theme::blue(),
                    Some(latest_version.clone()),
                    None,
                    false,
                    true,
                ),
                UpdateState::Updating { target_version } => (
                    tr.update_updating.to_string(),
                    theme::yellow(),
                    Some(target_version.clone()),
                    None,
                    true,
                    false,
                ),
                UpdateState::Restarting { new_version } => (
                    tr.update_restarting.to_string(),
                    theme::green(),
                    Some(new_version.clone()),
                    None,
                    true,
                    false,
                ),
                UpdateState::Error(err) => (
                    tr.update_error.to_string(),
                    theme::red(),
                    None,
                    Some(err.clone()),
                    false,
                    false,
                ),
            };

        let logo = image(
            assets_dir()
                .join("logo-200.png")
                .to_string_lossy()
                .to_string(),
        )
        .width(80)
        .height(80);

        let header = row![
            logo,
            column![
                text("MyPowerToys")
                    .size(ui.sz(28.0))
                    .font(bold())
                    .color(ui.heading()),
                text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .size(ui.sz(14.0))
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(4),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let desc = card(
            column![
                text(tr.about_title)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
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
                text(tr.tech_stack)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                info_row(tr.prog_lang, "Rust (edition 2024)", ui),
                info_row("Settings UI", "iced", ui),
                info_row("Overlays", "egui / eframe", ui),
                info_row("Async", "tokio", ui),
                info_row("Config", "serde + TOML", ui),
            ]
            .spacing(12),
            ui,
        );

        let check_button = if busy {
            button(text(tr.update_check).size(ui.sz(12.0)).font(ui.font()))
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(false))
        } else {
            button(text(tr.update_check).size(ui.sz(12.0)).font(ui.font()))
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(false))
                .on_press(Message::CheckForUpdates)
        };

        let install_button = if can_install {
            button(text(tr.update_install).size(ui.sz(12.0)).font(ui.font()))
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(true))
                .on_press(Message::OpenUpdateDialog)
        } else {
            button(text(tr.update_install).size(ui.sz(12.0)).font(ui.font()))
                .padding(Padding::from([6.0, 12.0]))
                .style(theme::seg_button(false))
        };

        let mut update_content = column![
            text(tr.update_section)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            info_row(
                tr.update_current_version,
                &format!("v{}", env!("CARGO_PKG_VERSION")),
                ui,
            ),
            row![
                text(tr.update_status)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark))
                    .width(120),
                text(status_text)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(status_color),
            ]
            .spacing(12),
        ]
        .spacing(8);

        if let Some(version) = latest_version {
            update_content = update_content.push(info_row(
                tr.update_latest_version,
                &format!("v{version}"),
                ui,
            ));
        }

        if let Some(err) = error_detail {
            update_content = update_content.push(
                text(err)
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            );
        }

        if matches!(self.update_state, UpdateState::Updating { .. }) {
            let pulse = 1.0 - (self.update_progress - 1.0).abs();
            update_content =
                update_content.push(progress_bar(0.0..=1.0, pulse).height(6).width(Length::Fill));
        } else if matches!(self.update_state, UpdateState::Restarting { .. }) {
            update_content =
                update_content.push(progress_bar(0.0..=1.0, 1.0).height(6).width(Length::Fill));
        }

        update_content = update_content.push(
            row![check_button, install_button]
                .spacing(8)
                .align_y(Alignment::Center),
        );

        let update_card = card(update_content, ui);

        column![header, Space::with_height(8), desc, info, tech, update_card]
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
                text(tr.appearance)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
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
                text(tr.language)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text(tr.lang_desc)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
                Space::with_height(8),
                container(
                    row![
                        seg_button(
                            "Fran\u{00e7}ais",
                            self.language == Language::Fr,
                            Message::SetLanguage(Language::Fr),
                            ui
                        ),
                        seg_button(
                            "English",
                            self.language == Language::En,
                            Message::SetLanguage(Language::En),
                            ui
                        ),
                        seg_button(
                            "Espa\u{00f1}ol",
                            self.language == Language::Es,
                            Message::SetLanguage(Language::Es),
                            ui
                        ),
                        seg_button(
                            "\u{65e5}\u{672c}\u{8a9e}",
                            self.language == Language::Jp,
                            Message::SetLanguage(Language::Jp),
                            ui
                        ),
                        seg_button(
                            "\u{4e2d}\u{6587}",
                            self.language == Language::Cn,
                            Message::SetLanguage(Language::Cn),
                            ui
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
                text(tr.text_size)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
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
                            ui
                        ),
                        seg_button(
                            tr.medium,
                            self.font_size == FontSize::Medium,
                            Message::SetFontSize(FontSize::Medium),
                            ui
                        ),
                        seg_button(
                            tr.large,
                            self.font_size == FontSize::Large,
                            Message::SetFontSize(FontSize::Large),
                            ui
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
                text(tr.accessibility)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                Space::with_height(4),
                pref_toggle(
                    tr.high_contrast,
                    tr.high_contrast_desc,
                    self.high_contrast,
                    Message::ToggleHighContrast,
                    ui
                ),
                pref_toggle(
                    tr.bold_text,
                    tr.bold_text_desc,
                    self.bold_text,
                    Message::ToggleBoldText,
                    ui
                ),
                pref_toggle(
                    tr.compact_layout,
                    tr.compact_layout_desc,
                    self.compact_layout,
                    Message::ToggleCompactLayout,
                    ui
                ),
                pref_toggle(
                    tr.reduced_motion,
                    tr.reduced_motion_desc,
                    self.reduced_motion,
                    Message::ToggleReducedMotion,
                    ui
                ),
            ]
            .spacing(12),
            ui,
        );

        column![
            text(tr.preferences)
                .size(ui.sz(28.0))
                .font(bold())
                .color(ui.heading()),
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
            text("Visual Theme")
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
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

    // ── Module settings ──────────────────────────────────────────────────────

    fn view_module_settings<'a>(
        &self,
        id: &str,
        tr: &'a translations::Tr,
        ui: Ui,
    ) -> Element<'a, Message> {
        let content = match id {
            "color-picker" => self.settings_color_picker(tr, ui),
            "text-extractor" => self.settings_text_extractor(tr, ui),
            "image-resizer" => self.settings_image_resizer(tr, ui),
            "mouse-utils" => self.settings_mouse_utils(tr, ui),
            "app-launcher" => self.settings_app_launcher(tr, ui),
            "fancy-zones" => self.settings_fancy_zones(tr, ui),
            "peek" => self.settings_peek(tr, ui),
            _ => column![
                text(tr.module_settings)
                    .size(ui.sz(16.0))
                    .font(bold())
                    .color(ui.heading()),
                text(tr.ms_no_settings)
                    .size(ui.sz(13.0))
                    .font(ui.font())
                    .color(theme::subtext0(ui.dark)),
            ]
            .spacing(8)
            .into(),
        };
        card(content, ui)
    }

    fn settings_color_picker<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.color_picker;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            text(tr.ms_format)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "HEX",
                        cfg.format == "hex",
                        Message::SetColorPickerFormat("hex".into()),
                        ui
                    ),
                    seg_button(
                        "RGB",
                        cfg.format == "rgb",
                        Message::SetColorPickerFormat("rgb".into()),
                        ui
                    ),
                    seg_button(
                        "HSL",
                        cfg.format == "hsl",
                        Message::SetColorPickerFormat("hsl".into()),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
        ]
        .spacing(8)
        .into()
    }

    fn settings_text_extractor<'a>(
        &self,
        tr: &'a translations::Tr,
        ui: Ui,
    ) -> Element<'a, Message> {
        let cfg = &self.module_configs.text_extractor;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            text(tr.ms_ocr_language)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "English",
                        cfg.language == "eng",
                        Message::SetTextExtractorLang("eng".into()),
                        ui
                    ),
                    seg_button(
                        "Français",
                        cfg.language == "fra",
                        Message::SetTextExtractorLang("fra".into()),
                        ui
                    ),
                    seg_button(
                        "Deutsch",
                        cfg.language == "deu",
                        Message::SetTextExtractorLang("deu".into()),
                        ui
                    ),
                    seg_button(
                        "Español",
                        cfg.language == "spa",
                        Message::SetTextExtractorLang("spa".into()),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
        ]
        .spacing(8)
        .into()
    }

    fn settings_image_resizer<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.image_resizer;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            // Preset
            text(tr.ms_preset)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "Small (640)",
                        cfg.preset == "small",
                        Message::SetImageResizerPreset("small".into()),
                        ui
                    ),
                    seg_button(
                        "Medium (1280)",
                        cfg.preset == "medium",
                        Message::SetImageResizerPreset("medium".into()),
                        ui
                    ),
                    seg_button(
                        "Large (1920)",
                        cfg.preset == "large",
                        Message::SetImageResizerPreset("large".into()),
                        ui
                    ),
                    seg_button(
                        "Phone (1080)",
                        cfg.preset == "phone",
                        Message::SetImageResizerPreset("phone".into()),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
            // Output format
            text(tr.ms_output_format)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "Original",
                        cfg.output_format == "original",
                        Message::SetImageResizerFormat("original".into()),
                        ui
                    ),
                    seg_button(
                        "PNG",
                        cfg.output_format == "png",
                        Message::SetImageResizerFormat("png".into()),
                        ui
                    ),
                    seg_button(
                        "JPEG",
                        cfg.output_format == "jpeg",
                        Message::SetImageResizerFormat("jpeg".into()),
                        ui
                    ),
                    seg_button(
                        "WebP",
                        cfg.output_format == "webp",
                        Message::SetImageResizerFormat("webp".into()),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
            // Quality
            text(tr.ms_quality)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "75",
                        cfg.quality == 75,
                        Message::SetImageResizerQuality(75),
                        ui
                    ),
                    seg_button(
                        "85",
                        cfg.quality == 85,
                        Message::SetImageResizerQuality(85),
                        ui
                    ),
                    seg_button(
                        "95",
                        cfg.quality == 95,
                        Message::SetImageResizerQuality(95),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
        ]
        .spacing(8)
        .into()
    }

    fn settings_mouse_utils<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.mouse_utils;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            pref_toggle(
                tr.ms_find_my_mouse,
                tr.ms_find_my_mouse_desc,
                cfg.find_my_mouse,
                Message::ToggleMouseFindMyMouse,
                ui
            ),
            pref_toggle(
                tr.ms_click_highlighter,
                tr.ms_click_highlighter_desc,
                cfg.click_highlighter,
                Message::ToggleMouseClickHighlighter,
                ui
            ),
            pref_toggle(
                tr.ms_crosshair,
                tr.ms_crosshair_desc,
                cfg.crosshair,
                Message::ToggleMouseCrosshair,
                ui
            ),
        ]
        .spacing(12)
        .into()
    }

    fn settings_app_launcher<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.app_launcher;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            // Max results
            text(tr.ms_max_results)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "5",
                        cfg.max_results == 5,
                        Message::SetAppLauncherMaxResults(5),
                        ui
                    ),
                    seg_button(
                        "8",
                        cfg.max_results == 8,
                        Message::SetAppLauncherMaxResults(8),
                        ui
                    ),
                    seg_button(
                        "10",
                        cfg.max_results == 10,
                        Message::SetAppLauncherMaxResults(10),
                        ui
                    ),
                    seg_button(
                        "15",
                        cfg.max_results == 15,
                        Message::SetAppLauncherMaxResults(15),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
            // Calculator toggle
            pref_toggle(
                tr.ms_calculator,
                tr.ms_calculator_desc,
                cfg.show_calculator,
                Message::ToggleAppLauncherCalc,
                ui
            ),
        ]
        .spacing(8)
        .into()
    }

    fn settings_fancy_zones<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.fancy_zones;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            text(tr.ms_zone_gap)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button("0 px", cfg.zone_gap == 0, Message::SetFancyZonesGap(0), ui),
                    seg_button("4 px", cfg.zone_gap == 4, Message::SetFancyZonesGap(4), ui),
                    seg_button("8 px", cfg.zone_gap == 8, Message::SetFancyZonesGap(8), ui),
                    seg_button(
                        "16 px",
                        cfg.zone_gap == 16,
                        Message::SetFancyZonesGap(16),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
        ]
        .spacing(8)
        .into()
    }

    fn settings_peek<'a>(&self, tr: &'a translations::Tr, ui: Ui) -> Element<'a, Message> {
        let cfg = &self.module_configs.peek;
        column![
            text(tr.module_settings)
                .size(ui.sz(16.0))
                .font(bold())
                .color(ui.heading()),
            // Preview lines
            text(tr.ms_preview_lines)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "25",
                        cfg.max_preview_lines == 25,
                        Message::SetPeekPreviewLines(25),
                        ui
                    ),
                    seg_button(
                        "50",
                        cfg.max_preview_lines == 50,
                        Message::SetPeekPreviewLines(50),
                        ui
                    ),
                    seg_button(
                        "100",
                        cfg.max_preview_lines == 100,
                        Message::SetPeekPreviewLines(100),
                        ui
                    ),
                    seg_button(
                        "200",
                        cfg.max_preview_lines == 200,
                        Message::SetPeekPreviewLines(200),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
            // Dir entries
            text(tr.ms_dir_entries)
                .size(ui.sz(13.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
            container(
                row![
                    seg_button(
                        "10",
                        cfg.max_dir_entries == 10,
                        Message::SetPeekDirEntries(10),
                        ui
                    ),
                    seg_button(
                        "20",
                        cfg.max_dir_entries == 20,
                        Message::SetPeekDirEntries(20),
                        ui
                    ),
                    seg_button(
                        "50",
                        cfg.max_dir_entries == 50,
                        Message::SetPeekDirEntries(50),
                        ui
                    ),
                ]
                .spacing(2),
            )
            .style(theme::segmented_control),
        ]
        .spacing(8)
        .into()
    }
}
