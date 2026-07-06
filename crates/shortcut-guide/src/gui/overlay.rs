use egui::{self, RichText, Shadow};

use crate::config::ShortcutGuideConfig;
use crate::gui::theme;
use crate::shortcuts::{ShortcutEntry, group_by_category};

struct GuideApp {
    shortcuts: Vec<ShortcutEntry>,
    should_close: bool,
    frame_count: u32,
    had_focus: bool,
    measured_size: Option<egui::Vec2>,
}

impl GuideApp {
    fn new(config: ShortcutGuideConfig) -> Self {
        Self {
            shortcuts: config.resolve(),
            should_close: false,
            frame_count: 0,
            had_focus: false,
            measured_size: None,
        }
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 14));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Keyboard Shortcuts")
                        .size(theme::FONT_TITLE)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} shortcuts available", self.shortcuts.len()))
                        .size(theme::FONT_SUBTITLE)
                        .color(theme::TEXT_MUTED),
                );
            });
        });
    }

    fn draw_content(&self, ui: &mut egui::Ui) {
        let frame =
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 14));

        frame.show(ui, |ui| {
            let groups = group_by_category(&self.shortcuts);

            // Distribute categories across two columns by total row weight,
            // so both columns end up roughly the same height.
            let mut columns: [Vec<(&str, Vec<&ShortcutEntry>)>; 2] = [Vec::new(), Vec::new()];
            let mut column_weight = [0usize; 2];
            for (cat, entries) in groups {
                let weight = 2 + entries.len();
                let target = usize::from(column_weight[0] > column_weight[1]);
                column_weight[target] += weight;
                columns[target].push((cat, entries));
            }

            ui.columns(2, |cols| {
                for (i, col_ui) in cols.iter_mut().enumerate() {
                    for (cat, entries) in &columns[i] {
                        draw_category(col_ui, cat, entries);
                        col_ui.add_space(theme::CATEGORY_SPACING);
                    }
                }
            });
        });
    }

    fn draw_footer(&self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 10));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_key_badge(ui, "Esc");
                ui.label(
                    RichText::new("close")
                        .size(theme::FONT_HINT)
                        .color(theme::TEXT_MUTED),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("MyPowerToys • Shortcut Guide")
                            .size(theme::FONT_HINT)
                            .color(theme::TEXT_MUTED),
                    );
                });
            });
        });
    }
}

fn draw_category(ui: &mut egui::Ui, category: &str, entries: &[&ShortcutEntry]) {
    ui.label(
        RichText::new(category.to_uppercase())
            .size(theme::FONT_CATEGORY)
            .color(theme::ACCENT)
            .strong(),
    );
    ui.add_space(4.0);

    // Thin separator under the category label.
    let sep_y = ui.cursor().min.y;
    let sep_width = ui.available_width();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(
            egui::pos2(ui.cursor().min.x, sep_y),
            egui::vec2(sep_width, 1.0),
        ),
        0.0,
        theme::SEPARATOR,
    );
    ui.add_space(6.0);

    for entry in entries {
        ui.horizontal(|ui| {
            ui.set_min_height(theme::ROW_HEIGHT);

            let parts: Vec<&str> = entry.keys.split('+').collect();
            let keys_width = estimate_keys_width(ui, &parts);
            let avail = ui.available_width();
            let desc_width = (avail - keys_width - 12.0).max(20.0);

            // Description on the left, clipped to its allocated width.
            ui.allocate_ui_with_layout(
                egui::vec2(desc_width, theme::ROW_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(&entry.description)
                                .size(theme::FONT_DESC)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .truncate(),
                    );
                },
            );

            // Key badges on the right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for (i, part) in parts.iter().enumerate().rev() {
                    draw_key_badge(ui, part.trim());
                    if i > 0 {
                        ui.label(
                            RichText::new("+")
                                .size(theme::FONT_KEY)
                                .color(theme::TEXT_MUTED),
                        );
                    }
                }
            });
        });
        ui.add_space(2.0);
    }
}

/// Estimate horizontal space needed to draw the badge row for `parts`.
/// Used to allocate the description column so it doesn't overlap the badges.
fn estimate_keys_width(ui: &egui::Ui, parts: &[&str]) -> f32 {
    let font = egui::FontId::monospace(theme::FONT_KEY);
    let sep_font = egui::FontId::proportional(theme::FONT_KEY);
    let mut total = 0.0f32;
    for (i, part) in parts.iter().enumerate() {
        let text_width = ui.fonts(|f| {
            f.layout_no_wrap(part.trim().to_string(), font.clone(), egui::Color32::WHITE)
                .rect
                .width()
        });
        // Badge: text + 2*6 padding + 2 border + 4 slack
        total += text_width + 12.0 + 2.0 + 4.0;
        if i + 1 < parts.len() {
            let plus_width = ui.fonts(|f| {
                f.layout_no_wrap("+".into(), sep_font.clone(), egui::Color32::WHITE)
                    .rect
                    .width()
            });
            // "+" label with 4px item spacing on each side
            total += plus_width + 8.0;
        }
    }
    total
}

fn draw_key_badge(ui: &mut egui::Ui, key: &str) {
    let frame = egui::Frame::NONE
        .fill(theme::KEY_BG)
        .stroke(egui::Stroke::new(1.0, theme::KEY_BORDER))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(6, 2));

    frame.show(ui, |ui| {
        ui.label(
            RichText::new(key)
                .size(theme::FONT_KEY)
                .color(theme::KEY_TEXT)
                .monospace(),
        );
    });
}

impl eframe::App for GuideApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count = self.frame_count.saturating_add(1);

        // Key handling
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.should_close = true;
            }
        });

        // Close on focus loss (once we've had focus at least one frame)
        let has_focus = ctx.input(|i| i.focused);
        if has_focus {
            self.had_focus = true;
        }
        if !has_focus && self.had_focus {
            self.should_close = true;
        }

        // Track the content size to compute the target window size.
        let content_response = egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PRIMARY)
                    .corner_radius(theme::CORNER_RADIUS)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.draw_header(ui);
                    self.draw_content(ui);
                    self.draw_footer(ui);
                })
                .response
            });

        // Auto-fit window to content (max 3 frames to let layout settle).
        if self.frame_count <= 4 {
            let content_size = content_response.inner.rect.size();
            let width = theme::WINDOW_MAX_WIDTH;
            let height = (content_size.y + 8.0).min(theme::WINDOW_MAX_HEIGHT);
            let target = egui::vec2(width, height);
            if self
                .measured_size
                .is_none_or(|prev| (prev - target).length() > 1.0)
            {
                self.measured_size = Some(target);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target));

                if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                    let x = (monitor.x - target.x) / 2.0;
                    let y = (monitor.y - target.y) / 2.0;
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([x, y].into()));
                }
            }
        }

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();
    }
}

pub fn run_guide(config: ShortcutGuideConfig) {
    if config.resolve().is_empty() {
        tracing::warn!("Shortcut Guide: no shortcuts to display, exiting");
        return;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_MAX_WIDTH, 200.0])
            .with_min_inner_size([560.0, 200.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_resizable(false),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Shortcut Guide",
        native_options,
        Box::new(move |cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(GuideApp::new(config)))
        }),
    );
}

fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::TRANSPARENT;
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.window_shadow = Shadow::NONE;
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = theme::BG_CARD;
    visuals.widgets.hovered.bg_fill = theme::BG_CARD;
    ctx.set_visuals(visuals);
}
