use crate::color::{Color, ColorFormat};
use crate::history::ColorHistory;
use crate::picker::copy_to_clipboard;
use eframe::egui;

struct EditorApp {
    current_color: Color,
    history: ColorHistory,
    shades: Vec<Color>,
    hex_text: String,
    rgb_text: String,
    hsl_text: String,
    hsv_text: String,
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
    }

    fn draw_shade_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let shade_width = (ui.available_width() / self.shades.len() as f32).max(20.0);
            let shade_height = 32.0;

            for shade in self.shades.clone() {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(shade_width, shade_height),
                    egui::Sense::click(),
                );
                let rounding = 0.0;
                ui.painter()
                    .rect_filled(rect, rounding, shade.to_egui_color32());

                if response.hovered() {
                    ui.painter().rect_stroke(
                        rect,
                        rounding,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                    response
                        .clone()
                        .on_hover_text(shade.format(ColorFormat::Hex));
                }

                if response.clicked() {
                    self.update_from_color(shade);
                }
            }
        });
    }

    fn draw_color_preview(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(70.0, 70.0), egui::Sense::hover());

        // Checkerboard background (for visibility)
        ui.painter().rect_filled(rect, 6.0, egui::Color32::WHITE);

        // Color fill
        ui.painter()
            .rect_filled(rect, 6.0, self.current_color.to_egui_color32());

        // Border
        ui.painter().rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
            egui::StrokeKind::Outside,
        );
    }

    fn draw_format_row(ui: &mut egui::Ui, label: &str, value: &str) {
        let row_height = 32.0;
        ui.allocate_ui(egui::vec2(ui.available_width(), row_height), |ui| {
            let frame = egui::Frame::NONE
                .fill(egui::Color32::from_gray(245))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(8, 4));

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .monospace()
                            .strong()
                            .color(egui::Color32::from_gray(100)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(value)
                            .monospace()
                            .color(egui::Color32::from_gray(30)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("\u{2398}").on_hover_text("Copy").clicked() {
                            let _ = copy_to_clipboard(value);
                        }
                    });
                });
            });
        });
    }

    fn draw_history(&mut self, ui: &mut egui::Ui) {
        if self.history.entries.is_empty() {
            ui.label(
                egui::RichText::new("No colors in history")
                    .italics()
                    .color(egui::Color32::from_gray(150)),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let mut clicked_color = None;
                let swatch_size = 32.0;
                let spacing = 4.0;
                let available_width = ui.available_width();
                let per_row =
                    ((available_width + spacing) / (swatch_size + spacing)).floor() as usize;
                let per_row = per_row.max(1);

                for chunk in self.history.entries.chunks(per_row) {
                    ui.horizontal(|ui| {
                        for entry in chunk {
                            let color = entry.color;
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(swatch_size, swatch_size),
                                egui::Sense::click(),
                            );
                            ui.painter().rect_filled(rect, 4.0, color.to_egui_color32());

                            if response.hovered() {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                                    egui::StrokeKind::Outside,
                                );
                                response
                                    .clone()
                                    .on_hover_text(color.format(ColorFormat::Hex));
                            }

                            if response.clicked() {
                                clicked_color = Some(color);
                            }
                        }
                    });
                }

                if let Some(color) = clicked_color {
                    self.update_from_color(color);
                }
            });
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::WHITE)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                // Shade bar
                self.draw_shade_bar(ui);
                ui.add_space(12.0);

                // Color preview + format fields
                ui.horizontal(|ui| {
                    self.draw_color_preview(ui);
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        Self::draw_format_row(ui, "HEX", &self.hex_text.clone());
                        ui.add_space(4.0);
                        Self::draw_format_row(ui, "RGB", &self.rgb_text.clone());
                        ui.add_space(4.0);
                        Self::draw_format_row(ui, "HSL", &self.hsl_text.clone());
                        ui.add_space(4.0);
                        Self::draw_format_row(ui, "HSV", &self.hsv_text.clone());
                    });
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // History section
                ui.label(egui::RichText::new("History").strong().size(13.0));
                ui.add_space(4.0);
                self.draw_history(ui);
            });
    }
}

/// Open the color editor window.
pub fn run_editor(color: Color, history: ColorHistory) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 400.0])
            .with_min_inner_size([360.0, 320.0])
            .with_title("Color Picker")
            .with_resizable(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Color Picker Editor",
        native_options,
        Box::new(move |_cc| Ok(Box::new(EditorApp::new(color, history)))),
    );
}
