use crate::config::FancyZonesConfig;
use crate::layout::Layout;
use eframe::egui;
use mpt_common::monitor::Monitor;
use mpt_common::theme::{self, Theme};

const WINDOW_WIDTH: f32 = 900.0;
const WINDOW_HEIGHT: f32 = 600.0;
const MONITOR_BAR_HEIGHT: f32 = 80.0;
const TEMPLATE_CARD_SIZE: f32 = 120.0;
const TEMPLATE_CARD_PADDING: f32 = 12.0;
const ZONE_PREVIEW_GAP: f32 = 2.0;

struct EditorApp {
    monitors: Vec<Monitor>,
    selected_monitor: usize,
    config: FancyZonesConfig,
    templates: Vec<Layout>,
    theme: Theme,
    theme_applied: bool,
}

impl EditorApp {
    fn new(monitors: Vec<Monitor>, config: FancyZonesConfig) -> Self {
        let templates = Layout::all_templates();
        Self {
            monitors,
            selected_monitor: 0,
            config,
            templates,
            theme: Theme::dark(),
            theme_applied: false,
        }
    }

    fn selected_monitor_name(&self) -> &str {
        self.monitors
            .get(self.selected_monitor)
            .map(|m| m.name.as_str())
            .unwrap_or("unknown")
    }

    fn selected_template_idx(&self) -> usize {
        let name = self.selected_monitor_name();
        self.config
            .monitor_layouts
            .get(name)
            .copied()
            .unwrap_or(self.config.default_layout)
    }

    fn save_config(&self) {
        if let Err(e) = mpt_common::config::save_module_config("fancy-zones", &self.config) {
            tracing::warn!("Failed to save FancyZones config: {e}");
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            self.theme.apply(ctx);
            self.theme_applied = true;
        }

        // Copy theme (it's Copy) to avoid borrow issues with the closure
        let t = self.theme;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            // Title
            ui.label(
                egui::RichText::new("FancyZones Editor")
                    .size(theme::FONT_TITLE)
                    .strong()
                    .color(t.text_primary),
            );
            ui.add_space(4.0);

            // Monitor bar
            ui.label(
                egui::RichText::new("Select a monitor to configure its layout:")
                    .size(theme::FONT_BODY)
                    .color(t.text_secondary),
            );
            ui.add_space(4.0);
            self.draw_monitor_bar(ui);

            ui.add_space(theme::SECTION_SPACING);
            ui.separator();
            ui.add_space(8.0);

            // Templates section
            ui.label(
                egui::RichText::new("Templates")
                    .size(theme::FONT_TITLE)
                    .strong()
                    .color(t.text_primary),
            );
            ui.add_space(4.0);
            self.draw_templates(ui);

            ui.add_space(theme::SECTION_SPACING);
            ui.separator();
            ui.add_space(8.0);

            // Custom section (placeholder)
            ui.label(
                egui::RichText::new("Custom")
                    .size(theme::FONT_TITLE)
                    .strong()
                    .color(t.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Create or duplicate a layout to get started.")
                    .size(theme::FONT_BODY)
                    .color(t.text_muted),
            );
        });
    }
}

impl EditorApp {
    fn draw_monitor_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;

        ui.horizontal(|ui| {
            for (i, monitor) in self.monitors.iter().enumerate() {
                let is_selected = i == self.selected_monitor;

                let desired = egui::vec2(140.0, MONITOR_BAR_HEIGHT);
                let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

                if response.clicked() {
                    self.selected_monitor = i;
                }

                let fill = if is_selected {
                    t.bg_card
                } else {
                    t.bg_secondary
                };
                let stroke = if is_selected {
                    egui::Stroke::new(2.0, t.text_accent)
                } else {
                    egui::Stroke::new(1.0, t.card_border)
                };

                ui.painter().rect(
                    rect,
                    theme::CARD_RADIUS,
                    fill,
                    stroke,
                    egui::StrokeKind::Outside,
                );

                // Monitor number badge
                let badge_text = format!("{}", i + 1);
                let badge_galley = ui.painter().layout_no_wrap(
                    badge_text,
                    egui::FontId::proportional(20.0),
                    t.text_primary,
                );
                ui.painter().galley(
                    rect.center_top() + egui::vec2(-badge_galley.size().x / 2.0, 8.0),
                    badge_galley,
                    t.text_primary,
                );

                // Resolution text
                let res_text = format!("{}x{}", monitor.width, monitor.height);
                let res_galley = ui.painter().layout_no_wrap(
                    res_text,
                    egui::FontId::proportional(theme::FONT_SMALL),
                    t.text_secondary,
                );
                ui.painter().galley(
                    rect.center_bottom() + egui::vec2(-res_galley.size().x / 2.0, -20.0),
                    res_galley,
                    t.text_secondary,
                );
            }
        });
    }

    fn draw_templates(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let current_idx = self.selected_template_idx();

        ui.horizontal_wrapped(|ui| {
            for (i, template) in self.templates.iter().enumerate() {
                let is_selected = i == current_idx;
                let desired = egui::vec2(TEMPLATE_CARD_SIZE, TEMPLATE_CARD_SIZE + 24.0);
                let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

                if response.clicked() {
                    let mon_name = self.selected_monitor_name().to_string();
                    self.config.monitor_layouts.insert(mon_name, i);
                    self.save_config();
                }

                let fill = if is_selected {
                    t.bg_card
                } else {
                    t.bg_secondary
                };
                let stroke = if is_selected {
                    egui::Stroke::new(2.0, t.text_accent)
                } else {
                    egui::Stroke::new(1.0, t.card_border)
                };

                ui.painter().rect(
                    rect,
                    theme::CARD_RADIUS,
                    fill,
                    stroke,
                    egui::StrokeKind::Outside,
                );

                // Zone preview area (top portion of card)
                let preview_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(TEMPLATE_CARD_PADDING, TEMPLATE_CARD_PADDING),
                    egui::vec2(
                        TEMPLATE_CARD_SIZE - 2.0 * TEMPLATE_CARD_PADDING,
                        TEMPLATE_CARD_SIZE - 2.0 * TEMPLATE_CARD_PADDING - 10.0,
                    ),
                );

                draw_zone_preview(ui, preview_rect, template, &t);

                // Template name
                let name_galley = ui.painter().layout_no_wrap(
                    template.name.clone(),
                    egui::FontId::proportional(theme::FONT_SMALL),
                    t.text_secondary,
                );
                let name_pos = egui::pos2(
                    rect.center().x - name_galley.size().x / 2.0,
                    rect.max.y - 18.0,
                );
                ui.painter().galley(name_pos, name_galley, t.text_secondary);
            }
        });
    }
}

fn draw_zone_preview(ui: &mut egui::Ui, preview_rect: egui::Rect, layout: &Layout, t: &Theme) {
    if layout.zones.is_empty() {
        let painter = ui.painter();
        painter.rect_stroke(
            preview_rect,
            2.0,
            egui::Stroke::new(1.0, t.separator),
            egui::StrokeKind::Inside,
        );
        painter.line_segment(
            [preview_rect.left_top(), preview_rect.right_bottom()],
            egui::Stroke::new(1.0, t.separator),
        );
        return;
    }

    let pw = preview_rect.width();
    let ph = preview_rect.height();
    let painter = ui.painter();

    for zone in &layout.zones {
        let zr = egui::Rect::from_min_size(
            preview_rect.min
                + egui::vec2(
                    zone.x * pw + ZONE_PREVIEW_GAP,
                    zone.y * ph + ZONE_PREVIEW_GAP,
                ),
            egui::vec2(
                zone.width * pw - 2.0 * ZONE_PREVIEW_GAP,
                zone.height * ph - 2.0 * ZONE_PREVIEW_GAP,
            ),
        );
        painter.rect_filled(zr, 2.0, t.bg_hover);
    }
}

/// Run the FancyZones editor GUI.
pub fn run_editor(monitors: Vec<Monitor>, config: FancyZonesConfig) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_min_inner_size(egui::vec2(600.0, 400.0))
            .with_title("FancyZones Editor"),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "FancyZones Editor",
        native_options,
        Box::new(move |_cc| Ok(Box::new(EditorApp::new(monitors, config)))),
    );
}
