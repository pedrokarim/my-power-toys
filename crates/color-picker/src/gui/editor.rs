use crate::color::{Color, ColorFormat};
use crate::history::ColorHistory;
use crate::picker::copy_to_clipboard;
use eframe::egui;
use std::time::Instant;

use super::theme;

struct EditorApp {
    current_color: Color,
    history: ColorHistory,
    shades: Vec<Color>,
    hex_text: String,
    rgb_text: String,
    hsl_text: String,
    hsv_text: String,
    cmyk_text: String,
    frame_count: u32,
    copied_feedback: Option<(ColorFormat, Instant)>,
}

impl EditorApp {
    fn new(color: Color, history: ColorHistory) -> Self {
        let mut app = Self {
            current_color: color,
            history,
            shades: Vec::new(),
            hex_text: String::new(),
            rgb_text: String::new(),
            hsl_text: String::new(),
            hsv_text: String::new(),
            cmyk_text: String::new(),
            frame_count: 0,
            copied_feedback: None,
        };
        app.update_from_color(color);
        app
    }

    fn update_from_color(&mut self, color: Color) {
        self.current_color = color;
        self.shades = color.shades(12);
        self.hex_text = color.format(ColorFormat::Hex);
        self.rgb_text = color.format(ColorFormat::Rgb);
        self.hsl_text = color.format(ColorFormat::Hsl);
        self.hsv_text = color.format(ColorFormat::Hsv);
        self.cmyk_text = color.format(ColorFormat::Cmyk);
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 8));

        let mut clicked_color = None;

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Pick color button
                let (btn_rect, btn_resp) =
                    ui.allocate_exact_size(egui::vec2(110.0, 32.0), egui::Sense::click());
                let fill = if btn_resp.hovered() {
                    theme::BG_BUTTON_HOVER
                } else {
                    theme::BG_BUTTON
                };
                ui.painter().rect_filled(btn_rect, 6.0, fill);
                let galley = ui.painter().layout_no_wrap(
                    "\u{270F} Pick color".to_string(),
                    egui::FontId::proportional(theme::FONT_BUTTON),
                    theme::TEXT_PRIMARY,
                );
                let text_pos = btn_rect.center() - galley.size() / 2.0;
                ui.painter().galley(text_pos, galley, theme::TEXT_PRIMARY);
                // TODO: re-launch picker on click

                ui.add_space(8.0);

                // Circular history swatches
                let max_swatches = 8;
                let diameter = theme::HISTORY_SWATCH_DIAMETER;
                for entry in self.history.entries.iter().take(max_swatches) {
                    let (rect, response) = ui
                        .allocate_exact_size(egui::vec2(diameter, diameter), egui::Sense::click());
                    let center = rect.center();
                    let radius = diameter / 2.0;

                    ui.painter()
                        .circle_filled(center, radius - 1.0, entry.color.to_egui_color32());

                    let is_current = entry.color.r == self.current_color.r
                        && entry.color.g == self.current_color.g
                        && entry.color.b == self.current_color.b;
                    let ring_color = if is_current {
                        egui::Color32::WHITE
                    } else if response.hovered() {
                        egui::Color32::from_rgb(180, 180, 200)
                    } else {
                        theme::SEPARATOR
                    };
                    let ring_width = if is_current { 2.0 } else { 1.0 };
                    ui.painter().circle_stroke(
                        center,
                        radius,
                        egui::Stroke::new(ring_width, ring_color),
                    );

                    if response.clicked() {
                        clicked_color = Some(entry.color);
                    }
                    response.on_hover_text(entry.color.format(ColorFormat::Hex));
                }

                // Gear icon (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new("\u{2699}")
                                .size(theme::FONT_ICON + 4.0)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .sense(egui::Sense::click()),
                    );
                });
            });
        });

        if let Some(color) = clicked_color {
            self.update_from_color(color);
        }
    }

    fn draw_shade_bar(&mut self, ui: &mut egui::Ui) {
        let total_width = ui.available_width();
        let bar_height = theme::SHADE_BAR_HEIGHT;
        let count = self.shades.len();
        if count == 0 {
            return;
        }
        let swatch_width = total_width / count as f32;

        let (total_rect, _) =
            ui.allocate_exact_size(egui::vec2(total_width, bar_height), egui::Sense::hover());

        let mut clicked_color = None;

        for (i, shade) in self.shades.iter().enumerate() {
            let x = total_rect.left() + i as f32 * swatch_width;
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, total_rect.top()),
                egui::vec2(swatch_width, bar_height),
            );

            let rounding = if i == 0 {
                egui::CornerRadius {
                    nw: 6,
                    sw: 6,
                    ne: 0,
                    se: 0,
                }
            } else if i == count - 1 {
                egui::CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: 6,
                    se: 6,
                }
            } else {
                egui::CornerRadius::ZERO
            };

            ui.painter()
                .rect_filled(rect, rounding, shade.to_egui_color32());

            let response = ui.interact(rect, egui::Id::new(("shade", i)), egui::Sense::click());
            if response.hovered() {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(swatch_width, 3.0)),
                    0.0,
                    egui::Color32::from_white_alpha(180),
                );
                response
                    .clone()
                    .on_hover_text(shade.format(ColorFormat::Hex));
            }
            if response.clicked() {
                clicked_color = Some(*shade);
            }
        }

        if let Some(color) = clicked_color {
            self.update_from_color(color);
        }
    }

    fn draw_gradient_strip(&self, ui: &mut egui::Ui, available_height: f32) -> Option<Color> {
        let strip_width = theme::GRADIENT_STRIP_WIDTH;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(strip_width, available_height),
            egui::Sense::hover(),
        );

        let (h, s, _) = self.current_color.to_hsl();
        let h_f = h as f64;
        let s_f = s as f64 / 100.0;
        let steps: usize = 64;

        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let y = rect.top() + t * available_height;
            let band_height = available_height / steps as f32 + 1.0;
            let l_f = 1.0 - t as f64;
            let band_color = Color::from_hsl(h_f, s_f, l_f);

            let band_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left(), y),
                egui::vec2(strip_width, band_height),
            );

            let rounding = if i == 0 {
                egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                }
            } else if i == steps - 1 {
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 6,
                    se: 6,
                }
            } else {
                egui::CornerRadius::ZERO
            };

            ui.painter()
                .rect_filled(band_rect, rounding, band_color.to_egui_color32());
        }

        // Border
        ui.painter().rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, theme::CARD_BORDER),
            egui::StrokeKind::Outside,
        );

        // Current lightness indicator
        let (_, _, current_l) = self.current_color.to_hsl();
        let indicator_y = rect.top() + (1.0 - current_l as f32 / 100.0) * available_height;
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() - 2.0, indicator_y),
                egui::pos2(rect.right() + 2.0, indicator_y),
            ],
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );

        // Click interaction
        let response = ui.interact(rect, egui::Id::new("gradient_strip"), egui::Sense::click());
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let t = ((pos.y - rect.top()) / available_height).clamp(0.0, 1.0);
            let l_f = 1.0 - t as f64;
            return Some(Color::from_hsl(h_f, s_f, l_f));
        }

        None
    }

    fn draw_format_card(
        &mut self,
        ui: &mut egui::Ui,
        format: ColorFormat,
        label: &str,
        value: &str,
    ) {
        let card_height = theme::CARD_HEIGHT;
        let available_width = ui.available_width();

        let (card_rect, _) = ui.allocate_exact_size(
            egui::vec2(available_width, card_height),
            egui::Sense::hover(),
        );

        // Card background
        ui.painter()
            .rect_filled(card_rect, theme::CARD_CORNER_RADIUS, theme::BG_SECONDARY);

        // Card border
        ui.painter().rect_stroke(
            card_rect,
            theme::CARD_CORNER_RADIUS,
            egui::Stroke::new(1.0, theme::CARD_BORDER),
            egui::StrokeKind::Inside,
        );

        // Left accent bar (current color)
        let accent_rect = egui::Rect::from_min_size(
            card_rect.min,
            egui::vec2(theme::CARD_ACCENT_WIDTH, card_height),
        );
        ui.painter().rect_filled(
            accent_rect,
            egui::CornerRadius {
                nw: theme::CARD_CORNER_RADIUS as u8,
                sw: theme::CARD_CORNER_RADIUS as u8,
                ne: 0,
                se: 0,
            },
            self.current_color.to_egui_color32(),
        );

        // Label
        let label_galley = ui.painter().layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(theme::FONT_FORMAT_LABEL),
            theme::TEXT_FORMAT_LABEL,
        );
        let label_y = card_rect.center().y - label_galley.size().y / 2.0;
        ui.painter().galley(
            egui::pos2(card_rect.left() + theme::CARD_ACCENT_WIDTH + 12.0, label_y),
            label_galley,
            theme::TEXT_FORMAT_LABEL,
        );

        // Value
        let value_galley = ui.painter().layout_no_wrap(
            value.to_string(),
            egui::FontId::monospace(theme::FONT_FORMAT_VALUE),
            theme::TEXT_FORMAT_VALUE,
        );
        let value_x = card_rect.left() + theme::CARD_ACCENT_WIDTH + 52.0;
        let value_y = card_rect.center().y - value_galley.size().y / 2.0;
        ui.painter().galley(
            egui::pos2(value_x, value_y),
            value_galley,
            theme::TEXT_FORMAT_VALUE,
        );

        // Copy button
        let copy_size = 28.0;
        let copy_rect = egui::Rect::from_center_size(
            egui::pos2(card_rect.right() - 24.0, card_rect.center().y),
            egui::vec2(copy_size, copy_size),
        );
        let copy_resp = ui.interact(
            copy_rect,
            egui::Id::new(("copy", label)),
            egui::Sense::click(),
        );

        let is_just_copied = self
            .copied_feedback
            .as_ref()
            .is_some_and(|(f, t)| *f == format && t.elapsed().as_millis() < 1000);

        let copy_icon = if is_just_copied {
            "\u{2713}"
        } else {
            "\u{2398}"
        };
        let copy_color = if is_just_copied {
            egui::Color32::from_rgb(100, 200, 100)
        } else if copy_resp.hovered() {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        };

        if copy_resp.hovered() {
            ui.painter().rect_filled(copy_rect, 4.0, theme::BG_HOVER);
        }

        let copy_galley = ui.painter().layout_no_wrap(
            copy_icon.to_string(),
            egui::FontId::proportional(14.0),
            copy_color,
        );
        let copy_text_pos = copy_rect.center() - copy_galley.size() / 2.0;
        ui.painter().galley(copy_text_pos, copy_galley, copy_color);

        if copy_resp.clicked() {
            let _ = copy_to_clipboard(value);
            self.copied_feedback = Some((format, Instant::now()));
        }

        copy_resp.on_hover_text("Copy to clipboard");
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count = self.frame_count.saturating_add(1);

        // Center window on first frames
        if self.frame_count <= 5
            && let Some(monitor) = ctx.input(|i| i.viewport().monitor_size)
        {
            let x = (monitor.x - theme::WINDOW_WIDTH) / 2.0;
            let y = (monitor.y - theme::WINDOW_HEIGHT) / 2.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([x, y].into()));
        }

        // Expire copied feedback
        if let Some((_, t)) = &self.copied_feedback
            && t.elapsed().as_millis() > 1500
        {
            self.copied_feedback = None;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::BG_PRIMARY))
            .show(ctx, |ui| {
                // Header
                self.draw_header(ui);

                // Shade bar
                ui.add_space(8.0);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 0))
                    .show(ui, |ui| {
                        self.draw_shade_bar(ui);
                    });
                ui.add_space(8.0);

                // Main content: gradient strip + format cards
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 8))
                    .show(ui, |ui| {
                        let cards_height = (theme::CARD_HEIGHT + 8.0) * 4.0 - 8.0;

                        ui.horizontal(|ui| {
                            // Left: vertical gradient strip
                            let gradient_color = self.draw_gradient_strip(ui, cards_height);
                            if let Some(color) = gradient_color {
                                self.update_from_color(color);
                            }

                            ui.add_space(12.0);

                            // Right: format cards
                            ui.vertical(|ui| {
                                let hex = self.hex_text.clone();
                                let rgb = self.rgb_text.clone();
                                let hsl = self.hsl_text.clone();
                                let hsv = self.hsv_text.clone();
                                let cmyk = self.cmyk_text.clone();

                                self.draw_format_card(ui, ColorFormat::Hex, "HEX", &hex);
                                ui.add_space(8.0);
                                self.draw_format_card(ui, ColorFormat::Rgb, "RGB", &rgb);
                                ui.add_space(8.0);
                                self.draw_format_card(ui, ColorFormat::Hsl, "HSL", &hsl);
                                ui.add_space(8.0);
                                self.draw_format_card(ui, ColorFormat::Hsv, "HSV", &hsv);
                                ui.add_space(8.0);
                                self.draw_format_card(ui, ColorFormat::Cmyk, "CMYK", &cmyk);
                            });
                        });
                    });
            });

        // Repaint while copy feedback is visible
        if self.copied_feedback.is_some() {
            ctx.request_repaint();
        }
    }
}

/// Open the color editor window.
pub fn run_editor(color: Color, history: ColorHistory) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT])
            .with_min_inner_size([340.0, 400.0])
            .with_title("Color Picker")
            .with_resizable(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Color Picker Editor",
        native_options,
        Box::new(move |cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(EditorApp::new(color, history)))
        }),
    );
}

fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = theme::BG_PRIMARY;
    visuals.panel_fill = theme::BG_PRIMARY;
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_fill = theme::BG_PRIMARY;
    ctx.set_visuals(visuals);
}
