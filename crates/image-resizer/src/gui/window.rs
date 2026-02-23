use crate::gui::theme;
use crate::resizer::{OutputFormat, ResizePreset};
use crate::{ImageResizerConfig, resize_image};
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::info;

// ── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ResizeResult {
    input: PathBuf,
    output: Result<PathBuf, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Idle,
    Resizing,
    Done,
}

// ── App state ───────────────────────────────────────────────────────────────

struct ImageResizerApp {
    files: Vec<PathBuf>,
    preset: ResizePreset,
    custom_width: String,
    output_format: OutputFormat,
    quality: u8,
    output_dir: Option<PathBuf>,
    status: Status,
    results: Arc<Mutex<Vec<ResizeResult>>>,
    total_files: usize,
    frame_count: u32,
    done_time: Option<Instant>,
}

impl ImageResizerApp {
    fn new(config: ImageResizerConfig) -> Self {
        Self {
            files: Vec::new(),
            preset: config.preset,
            custom_width: "800".into(),
            output_format: config.output_format,
            quality: config.quality,
            output_dir: None,
            status: Status::Idle,
            results: Arc::new(Mutex::new(Vec::new())),
            total_files: 0,
            frame_count: 0,
            done_time: None,
        }
    }

    fn max_width(&self) -> u32 {
        match self.preset {
            ResizePreset::Custom => self.custom_width.parse().unwrap_or(800),
            other => other.max_width(),
        }
    }

    fn add_files_dialog(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title("Select images to resize")
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif"],
            );
        if let Some(paths) = dialog.pick_files() {
            for path in paths {
                if !self.files.contains(&path) {
                    self.files.push(path);
                }
            }
        }
    }

    fn pick_output_dir(&mut self) {
        let dialog = rfd::FileDialog::new().set_title("Select output directory");
        if let Some(dir) = dialog.pick_folder() {
            self.output_dir = Some(dir);
        }
    }

    fn start_resize(&mut self, ctx: &egui::Context) {
        if self.files.is_empty() {
            return;
        }

        let output_dir = self.output_dir.clone().unwrap_or_else(|| {
            self.files
                .first()
                .and_then(|f| f.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(std::env::temp_dir)
        });

        self.status = Status::Resizing;
        self.total_files = self.files.len();
        self.done_time = None;
        self.results.lock().unwrap().clear();

        let files = self.files.clone();
        let max_width = self.max_width();
        let format = self.output_format;
        let quality = self.quality;
        let results = self.results.clone();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            std::fs::create_dir_all(&output_dir).ok();
            for file in &files {
                let result = resize_image(file, &output_dir, max_width, format, quality);
                let res = ResizeResult {
                    input: file.clone(),
                    output: result.map_err(|e| e.to_string()),
                };
                results.lock().unwrap().push(res);
                ctx.request_repaint();
            }
        });
    }

    // ── Drawing helpers ─────────────────────────────────────────────────────

    fn draw_header(&self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_MARGIN as i8, 14));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Custom-drawn resize icon: two overlapping rectangles
                let icon_size = 28.0;
                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
                let p = ui.painter_at(icon_rect);

                // Large rectangle (back)
                let r1 = egui::Rect::from_min_size(
                    icon_rect.min,
                    egui::vec2(icon_size * 0.72, icon_size * 0.72),
                );
                p.rect_stroke(
                    r1,
                    3.0,
                    egui::Stroke::new(1.8, theme::TEXT_ACCENT),
                    egui::StrokeKind::Inside,
                );
                // Small rectangle (front, offset)
                let r2 = egui::Rect::from_min_size(
                    egui::pos2(
                        icon_rect.left() + icon_size * 0.28,
                        icon_rect.top() + icon_size * 0.28,
                    ),
                    egui::vec2(icon_size * 0.72, icon_size * 0.72),
                );
                p.rect_filled(r2, 3.0, theme::BG_HEADER);
                p.rect_stroke(
                    r2,
                    3.0,
                    egui::Stroke::new(1.8, theme::TEXT_ACCENT),
                    egui::StrokeKind::Inside,
                );
                // Diagonal arrow in small rect
                let arrow_start = egui::pos2(r2.left() + 5.0, r2.bottom() - 5.0);
                let arrow_end = egui::pos2(r2.right() - 5.0, r2.top() + 5.0);
                p.line_segment(
                    [arrow_start, arrow_end],
                    egui::Stroke::new(1.5, theme::TEXT_ACCENT),
                );

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Image Resizer")
                            .size(theme::FONT_TITLE)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Batch resize images with presets or custom dimensions",
                        )
                        .size(theme::FONT_SMALL)
                        .color(theme::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn draw_drop_zone(&mut self, ui: &mut egui::Ui) {
        let available_w = ui.available_width();
        let height = if self.files.is_empty() {
            theme::DROP_ZONE_HEIGHT
        } else {
            (self.files.len() as f32 * (theme::FILE_CARD_HEIGHT + 4.0) + 12.0)
                .clamp(60.0, theme::DROP_ZONE_HEIGHT)
        };

        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, height), egui::Sense::hover());

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, theme::CARD_RADIUS, theme::BG_SECONDARY);

        // Dashed border effect (dotted corners)
        let is_hovering_files = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        let border_color = if is_hovering_files {
            theme::DROP_ZONE_BORDER_ACTIVE
        } else {
            theme::DROP_ZONE_BORDER
        };
        let border_width = if is_hovering_files { 2.0 } else { 1.0 };
        painter.rect_stroke(
            rect,
            theme::CARD_RADIUS,
            egui::Stroke::new(border_width, border_color),
            egui::StrokeKind::Inside,
        );

        if self.files.is_empty() {
            // Empty state: custom drawn arrow-down icon + text
            let center = rect.center();
            let icon_color = if is_hovering_files {
                theme::DROP_ZONE_BORDER_ACTIVE
            } else {
                theme::TEXT_MUTED
            };

            // Draw a download/drop arrow icon
            let arrow_top = egui::pos2(center.x, center.y - 28.0);
            let arrow_bottom = egui::pos2(center.x, center.y - 8.0);
            painter.line_segment(
                [arrow_top, arrow_bottom],
                egui::Stroke::new(2.0, icon_color),
            );
            // Arrow head
            painter.line_segment(
                [egui::pos2(center.x - 6.0, center.y - 14.0), arrow_bottom],
                egui::Stroke::new(2.0, icon_color),
            );
            painter.line_segment(
                [egui::pos2(center.x + 6.0, center.y - 14.0), arrow_bottom],
                egui::Stroke::new(2.0, icon_color),
            );
            // Tray (horizontal line under arrow)
            painter.line_segment(
                [
                    egui::pos2(center.x - 12.0, center.y - 4.0),
                    egui::pos2(center.x + 12.0, center.y - 4.0),
                ],
                egui::Stroke::new(2.0, icon_color),
            );

            painter.text(
                egui::pos2(center.x, center.y + 14.0),
                egui::Align2::CENTER_CENTER,
                "Drop images here",
                egui::FontId::proportional(theme::FONT_BODY),
                theme::TEXT_MUTED,
            );
            painter.text(
                egui::pos2(center.x, center.y + 32.0),
                egui::Align2::CENTER_CENTER,
                "PNG, JPEG, WebP, BMP, GIF, TIFF",
                egui::FontId::proportional(theme::FONT_SMALL),
                theme::TEXT_MUTED,
            );
        } else {
            // File list with cards
            let inner = rect.shrink(6.0);
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                egui::ScrollArea::vertical()
                    .max_height(inner.height())
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        for (i, file) in self.files.iter().enumerate() {
                            let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            let ext = file
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_uppercase();

                            let (card_rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), theme::FILE_CARD_HEIGHT),
                                egui::Sense::hover(),
                            );

                            let p = ui.painter_at(card_rect);
                            p.rect_filled(card_rect, 4.0, theme::BG_CARD);

                            // Extension badge
                            let badge_rect = egui::Rect::from_min_size(
                                egui::pos2(card_rect.left() + 6.0, card_rect.top() + 8.0),
                                egui::vec2(38.0, 20.0),
                            );
                            p.rect_filled(badge_rect, 3.0, theme::BG_CHIP);
                            p.text(
                                badge_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &ext,
                                egui::FontId::monospace(9.0),
                                theme::TEXT_ACCENT,
                            );

                            // File name
                            p.text(
                                egui::pos2(card_rect.left() + 52.0, card_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                name,
                                egui::FontId::proportional(theme::FONT_FILE),
                                theme::TEXT_PRIMARY,
                            );

                            // Remove button
                            let close_rect = egui::Rect::from_center_size(
                                egui::pos2(card_rect.right() - 18.0, card_rect.center().y),
                                egui::vec2(22.0, 22.0),
                            );
                            let close_resp = ui.interact(
                                close_rect,
                                egui::Id::new(("remove_file", i)),
                                egui::Sense::click(),
                            );
                            if close_resp.hovered() {
                                p.rect_filled(close_rect, 3.0, theme::BG_HOVER);
                            }
                            p.text(
                                close_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "\u{2715}", // X mark
                                egui::FontId::proportional(11.0),
                                if close_resp.hovered() {
                                    theme::TEXT_ERROR
                                } else {
                                    theme::TEXT_MUTED
                                },
                            );
                            if close_resp.clicked() {
                                to_remove = Some(i);
                            }
                        }
                        if let Some(i) = to_remove {
                            self.files.remove(i);
                        }
                    });
            });
        }
    }

    fn draw_chip_selector<T: PartialEq + Copy>(
        ui: &mut egui::Ui,
        current: &mut T,
        options: &[(T, &str)],
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (value, label) in options {
                let selected = *current == *value;
                let fill = if selected {
                    theme::BG_CHIP_SELECTED
                } else {
                    theme::BG_CHIP
                };
                let text_color = if selected {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_SECONDARY
                };

                let (chip_rect, response) = ui.allocate_exact_size(
                    egui::vec2(
                        ui.painter()
                            .layout_no_wrap(
                                label.to_string(),
                                egui::FontId::proportional(theme::FONT_CHIP),
                                text_color,
                            )
                            .size()
                            .x
                            + 20.0,
                        theme::CHIP_HEIGHT,
                    ),
                    egui::Sense::click(),
                );

                let p = ui.painter_at(chip_rect);
                let bg = if response.hovered() && !selected {
                    theme::BG_HOVER
                } else {
                    fill
                };
                p.rect_filled(chip_rect, theme::CHIP_RADIUS, bg);

                if selected {
                    p.rect_stroke(
                        chip_rect,
                        theme::CHIP_RADIUS,
                        egui::Stroke::new(1.0, theme::TEXT_ACCENT),
                        egui::StrokeKind::Inside,
                    );
                }

                p.text(
                    chip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(theme::FONT_CHIP),
                    text_color,
                );

                if response.clicked() {
                    *current = *value;
                }
            }
        });
    }

    fn draw_section_label(ui: &mut egui::Ui, label: &str) {
        ui.horizontal(|ui| {
            // Accent bar
            let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(3.0, 14.0), egui::Sense::hover());
            ui.painter_at(bar_rect)
                .rect_filled(bar_rect, 1.5, theme::TEXT_ACCENT);
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(label)
                    .size(theme::FONT_SECTION)
                    .strong()
                    .color(theme::TEXT_SECONDARY),
            );
        });
    }

    fn draw_action_button(ui: &mut egui::Ui, label: &str, icon: &str, enabled: bool) -> bool {
        let available_w = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(available_w, theme::BUTTON_HEIGHT),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );

        let p = ui.painter_at(rect);
        let bg = if !enabled {
            theme::BG_CHIP
        } else if response.hovered() {
            theme::BG_BUTTON_HOVER
        } else {
            theme::BG_BUTTON
        };
        p.rect_filled(rect, theme::BUTTON_RADIUS, bg);

        let text = format!("{icon}  {label}");
        let text_color = if enabled {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_MUTED
        };
        p.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(theme::FONT_BUTTON),
            text_color,
        );

        enabled && response.clicked()
    }

    fn draw_progress_bar(&self, ui: &mut egui::Ui) {
        let done = self.results.lock().unwrap().len();
        let progress = done as f32 / self.total_files.max(1) as f32;
        let available_w = ui.available_width();

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(available_w, theme::BUTTON_HEIGHT),
            egui::Sense::hover(),
        );

        let p = ui.painter_at(rect);

        // Track
        p.rect_filled(rect, theme::BUTTON_RADIUS, theme::BG_SECONDARY);

        // Fill
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * progress, rect.height()));
        p.rect_filled(fill_rect, theme::BUTTON_RADIUS, theme::BG_PROGRESS);

        // Text
        let text = format!("Resizing... {done}/{}", self.total_files);
        p.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(theme::FONT_BUTTON),
            theme::TEXT_PRIMARY,
        );
    }

    fn draw_results(&mut self, ui: &mut egui::Ui) {
        let results = self.results.lock().unwrap();
        let successes = results.iter().filter(|r| r.output.is_ok()).count();
        let failures = results.len() - successes;

        // Success card
        let available_w = ui.available_width();
        let (card_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, 44.0), egui::Sense::hover());

        let p = ui.painter_at(card_rect);
        p.rect_filled(card_rect, theme::CARD_RADIUS, theme::BG_CARD);

        if failures == 0 {
            p.text(
                card_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("\u{2713}  All {successes} images resized successfully"),
                egui::FontId::proportional(theme::FONT_BODY),
                theme::TEXT_SUCCESS,
            );
        } else {
            let center = card_rect.center();
            p.text(
                egui::pos2(center.x, center.y - 8.0),
                egui::Align2::CENTER_CENTER,
                format!("\u{2713} {successes} resized"),
                egui::FontId::proportional(theme::FONT_BODY),
                theme::TEXT_SUCCESS,
            );
            p.text(
                egui::pos2(center.x, center.y + 8.0),
                egui::Align2::CENTER_CENTER,
                format!("\u{2715} {failures} failed"),
                egui::FontId::proportional(theme::FONT_BODY),
                theme::TEXT_ERROR,
            );
        }

        // Show errors
        for res in results.iter() {
            if let Err(e) = &res.output {
                let name = res
                    .input
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(format!("  {name}: {e}"))
                        .size(theme::FONT_SMALL)
                        .color(theme::TEXT_ERROR),
                );
            }
        }

        drop(results);

        ui.add_space(8.0);

        if Self::draw_action_button(ui, "Resize More", "\u{21BB}", true) {
            self.status = Status::Idle;
            self.done_time = None;
            self.files.clear();
        }
    }
}

// ── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for ImageResizerApp {
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

        // Check if resize is done
        if self.status == Status::Resizing {
            let done = self.results.lock().unwrap().len();
            if done >= self.total_files {
                self.status = Status::Done;
                self.done_time = Some(Instant::now());
            }
        }

        // Handle dropped files
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        for path in dropped {
            if !self.files.contains(&path) {
                self.files.push(path);
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::BG_PRIMARY))
            .show(ctx, |ui| {
                // ── Header ──────────────────────────────────────────
                self.draw_header(ui);

                // ── Main content ────────────────────────────────────
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(
                        theme::INNER_MARGIN as i8,
                        theme::INNER_MARGIN as i8,
                    ))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;

                        // ── Files section ───────────────────────────
                        ui.horizontal(|ui| {
                            Self::draw_section_label(ui, "IMAGES");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} file{}",
                                            self.files.len(),
                                            if self.files.len() != 1 { "s" } else { "" }
                                        ))
                                        .size(theme::FONT_SMALL)
                                        .color(theme::TEXT_MUTED),
                                    );
                                },
                            );
                        });
                        ui.add_space(2.0);

                        self.draw_drop_zone(ui);
                        ui.add_space(2.0);

                        // File action buttons
                        ui.horizontal(|ui| {
                            let (btn_rect, btn_resp) = ui
                                .allocate_exact_size(egui::vec2(100.0, 28.0), egui::Sense::click());
                            let p = ui.painter_at(btn_rect);
                            let bg = if btn_resp.hovered() {
                                theme::BG_HOVER
                            } else {
                                theme::BG_CHIP
                            };
                            p.rect_filled(btn_rect, 5.0, bg);
                            p.text(
                                btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "+  Add files",
                                egui::FontId::proportional(theme::FONT_SMALL),
                                theme::TEXT_PRIMARY,
                            );
                            if btn_resp.clicked() {
                                self.add_files_dialog();
                            }

                            if !self.files.is_empty() {
                                let (clr_rect, clr_resp) = ui.allocate_exact_size(
                                    egui::vec2(75.0, 28.0),
                                    egui::Sense::click(),
                                );
                                let p = ui.painter_at(clr_rect);
                                let bg = if clr_resp.hovered() {
                                    theme::BG_HOVER
                                } else {
                                    theme::BG_CHIP
                                };
                                p.rect_filled(clr_rect, 5.0, bg);
                                p.text(
                                    clr_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "Clear all",
                                    egui::FontId::proportional(theme::FONT_SMALL),
                                    theme::TEXT_MUTED,
                                );
                                if clr_resp.clicked() {
                                    self.files.clear();
                                }
                            }
                        });

                        ui.add_space(theme::SECTION_SPACING);

                        // Separator
                        let (sep_rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 1.0),
                            egui::Sense::hover(),
                        );
                        ui.painter_at(sep_rect)
                            .rect_filled(sep_rect, 0.0, theme::SEPARATOR);

                        ui.add_space(theme::SECTION_SPACING);

                        // ── Size Preset ─────────────────────────────
                        Self::draw_section_label(ui, "SIZE PRESET");
                        ui.add_space(4.0);

                        Self::draw_chip_selector(
                            ui,
                            &mut self.preset,
                            &[
                                (ResizePreset::Small, "Small  640"),
                                (ResizePreset::Medium, "Medium  1280"),
                                (ResizePreset::Large, "Large  1920"),
                                (ResizePreset::Phone, "Phone  1080"),
                                (ResizePreset::Custom, "Custom"),
                            ],
                        );

                        if self.preset == ResizePreset::Custom {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Max width (px)")
                                        .size(theme::FONT_SMALL)
                                        .color(theme::TEXT_SECONDARY),
                                );
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.custom_width)
                                        .desired_width(80.0)
                                        .char_limit(5)
                                        .font(egui::FontId::monospace(theme::FONT_BODY)),
                                );
                                // Filter to digits only
                                if response.changed() {
                                    self.custom_width.retain(|c| c.is_ascii_digit());
                                }
                            });
                        }

                        ui.add_space(theme::SECTION_SPACING);

                        // ── Output Format ───────────────────────────
                        Self::draw_section_label(ui, "OUTPUT FORMAT");
                        ui.add_space(4.0);

                        Self::draw_chip_selector(
                            ui,
                            &mut self.output_format,
                            &[
                                (OutputFormat::Original, "Original"),
                                (OutputFormat::Png, "PNG"),
                                (OutputFormat::Jpeg, "JPEG"),
                                (OutputFormat::Webp, "WebP"),
                            ],
                        );

                        // ── Quality slider (JPEG only) ──────────────
                        if self.output_format == OutputFormat::Jpeg {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Quality")
                                        .size(theme::FONT_SMALL)
                                        .color(theme::TEXT_SECONDARY),
                                );
                                let mut q = self.quality as f32;
                                let slider =
                                    egui::Slider::new(&mut q, 1.0..=100.0).integer().suffix("%");
                                if ui.add(slider).changed() {
                                    self.quality = q as u8;
                                }
                            });
                        }

                        ui.add_space(theme::SECTION_SPACING);

                        // ── Output directory ────────────────────────
                        ui.horizontal(|ui| {
                            Self::draw_section_label(ui, "OUTPUT");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (btn_rect, btn_resp) = ui.allocate_exact_size(
                                        egui::vec2(65.0, 22.0),
                                        egui::Sense::click(),
                                    );
                                    let p = ui.painter_at(btn_rect);
                                    let bg = if btn_resp.hovered() {
                                        theme::BG_HOVER
                                    } else {
                                        theme::BG_CHIP
                                    };
                                    p.rect_filled(btn_rect, 4.0, bg);
                                    p.text(
                                        btn_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "Change",
                                        egui::FontId::proportional(theme::FONT_SMALL),
                                        theme::TEXT_ACCENT,
                                    );
                                    if btn_resp.clicked() {
                                        self.pick_output_dir();
                                    }
                                },
                            );
                        });

                        let dir_label = self
                            .output_dir
                            .as_ref()
                            .map(|d| {
                                let s = d.display().to_string();
                                if s.len() > 50 {
                                    format!("...{}", &s[s.len() - 47..])
                                } else {
                                    s
                                }
                            })
                            .unwrap_or_else(|| "Same as source folder".into());
                        ui.label(
                            egui::RichText::new(dir_label)
                                .size(theme::FONT_SMALL)
                                .color(theme::TEXT_MUTED),
                        );

                        ui.add_space(theme::SECTION_SPACING);

                        // ── Action area ─────────────────────────────
                        match &self.status {
                            Status::Idle => {
                                let has_files = !self.files.is_empty();
                                let label = if has_files {
                                    format!(
                                        "Resize {} image{}",
                                        self.files.len(),
                                        if self.files.len() != 1 { "s" } else { "" }
                                    )
                                } else {
                                    "Resize".into()
                                };
                                if Self::draw_action_button(ui, &label, "\u{25B6}", has_files) {
                                    self.start_resize(ctx);
                                }
                            }
                            Status::Resizing => {
                                self.draw_progress_bar(ui);
                            }
                            Status::Done => {
                                self.draw_results(ui);
                            }
                        }
                    });
            });

        // Keep repainting during resize
        if self.status == Status::Resizing {
            ctx.request_repaint();
        }
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub fn run_window(config: ImageResizerConfig) {
    info!("Opening Image Resizer window");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT])
            .with_min_inner_size([400.0, 500.0])
            .with_title("Image Resizer")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Image Resizer",
        options,
        Box::new(|cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(ImageResizerApp::new(config)))
        }),
    )
    .ok();
}

fn setup_visuals(ctx: &egui::Context) {
    theme::setup_visuals(ctx);
}
