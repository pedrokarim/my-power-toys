use crate::color::{Color, ColorFormat};
use eframe::egui;
use image::RgbaImage;
use std::sync::{Arc, Mutex};

const DEFAULT_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 2.0;
const MAX_ZOOM: f32 = 20.0;
const MAGNIFIER_RADIUS: f32 = 65.0;
const TOOLTIP_OFFSET_X: f32 = 20.0;
const TOOLTIP_OFFSET_Y: f32 = -45.0;

struct OverlayApp {
    screenshot: RgbaImage,
    texture: Option<egui::TextureHandle>,
    zoom_level: f32,
    result: Arc<Mutex<Option<Color>>>,
    should_close: bool,
}

impl OverlayApp {
    fn new(screenshot: RgbaImage, result: Arc<Mutex<Option<Color>>>) -> Self {
        Self {
            screenshot,
            texture: None,
            zoom_level: DEFAULT_ZOOM,
            result,
            should_close: false,
        }
    }

    fn color_at(&self, x: u32, y: u32) -> Color {
        if x < self.screenshot.width() && y < self.screenshot.height() {
            let p = self.screenshot.get_pixel(x, y);
            Color::new(p[0], p[1], p[2])
        } else {
            Color::new(0, 0, 0)
        }
    }

    fn screen_to_pixel(&self, pos: egui::Pos2, screen_size: egui::Vec2) -> (u32, u32) {
        let img_w = self.screenshot.width() as f32;
        let img_h = self.screenshot.height() as f32;
        let px_x = (pos.x / screen_size.x * img_w).round() as u32;
        let px_y = (pos.y / screen_size.y * img_h).round() as u32;
        (
            px_x.min(self.screenshot.width().saturating_sub(1)),
            px_y.min(self.screenshot.height().saturating_sub(1)),
        )
    }

    fn draw_magnifier(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        px_x: u32,
        px_y: u32,
        _screen_size: egui::Vec2,
    ) {
        let radius = MAGNIFIER_RADIUS;
        let zoom = self.zoom_level;
        let pixel_size = zoom;
        let half_count = (radius / pixel_size).ceil() as i32;

        // Background circle with shadow
        painter.circle_filled(
            center + egui::vec2(2.0, 2.0),
            radius + 3.0,
            egui::Color32::from_black_alpha(80),
        );
        painter.circle_filled(center, radius + 2.0, egui::Color32::from_gray(30));

        let img_w = self.screenshot.width() as f32;
        let img_h = self.screenshot.height() as f32;

        for dy in -half_count..=half_count {
            for dx in -half_count..=half_count {
                let offset = egui::vec2(dx as f32 * pixel_size, dy as f32 * pixel_size);
                let rect_center = center + offset;

                // Skip if outside circle
                if (rect_center - center).length() > radius {
                    continue;
                }

                let src_x = (px_x as i32 + dx).clamp(0, img_w as i32 - 1) as u32;
                let src_y = (px_y as i32 + dy).clamp(0, img_h as i32 - 1) as u32;
                let c = self.color_at(src_x, src_y);

                let rect =
                    egui::Rect::from_center_size(rect_center, egui::vec2(pixel_size, pixel_size));
                painter.rect_filled(rect, 0.0, c.to_egui_color32());

                // Grid lines at high zoom levels
                if zoom >= 6.0 {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_white_alpha(30)),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }

        // Circle border
        painter.circle_stroke(center, radius, egui::Stroke::new(2.5, egui::Color32::WHITE));
        painter.circle_stroke(
            center,
            radius + 2.5,
            egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        );

        // Crosshair at center
        let cross_len = (pixel_size / 2.0).max(3.0);
        let cross_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
        let shadow_stroke = egui::Stroke::new(2.5, egui::Color32::from_black_alpha(150));

        // Shadow
        painter.line_segment(
            [
                center - egui::vec2(cross_len, 0.0),
                center + egui::vec2(cross_len, 0.0),
            ],
            shadow_stroke,
        );
        painter.line_segment(
            [
                center - egui::vec2(0.0, cross_len),
                center + egui::vec2(0.0, cross_len),
            ],
            shadow_stroke,
        );
        // White
        painter.line_segment(
            [
                center - egui::vec2(cross_len, 0.0),
                center + egui::vec2(cross_len, 0.0),
            ],
            cross_stroke,
        );
        painter.line_segment(
            [
                center - egui::vec2(0.0, cross_len),
                center + egui::vec2(0.0, cross_len),
            ],
            cross_stroke,
        );

        // Zoom level indicator
        let zoom_text = format!("{:.0}x", self.zoom_level);
        let galley = painter.layout_no_wrap(
            zoom_text,
            egui::FontId::proportional(11.0),
            egui::Color32::from_white_alpha(180),
        );
        painter.galley(
            center + egui::vec2(-galley.size().x / 2.0, radius + 6.0),
            galley,
            egui::Color32::WHITE,
        );
    }

    fn draw_tooltip(
        &self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        color: Color,
        screen_size: egui::Vec2,
    ) {
        let hex = color.format(ColorFormat::Hex);

        // Position tooltip, flip if near edge
        let mut offset_x = TOOLTIP_OFFSET_X;
        let mut offset_y = TOOLTIP_OFFSET_Y;

        let text_galley = painter.layout_no_wrap(
            hex.clone(),
            egui::FontId::monospace(13.0),
            egui::Color32::WHITE,
        );

        let tooltip_width = text_galley.size().x + 36.0;
        let tooltip_height = 28.0;

        if pos.x + offset_x + tooltip_width > screen_size.x {
            offset_x = -tooltip_width - 10.0;
        }
        if pos.y + offset_y < 0.0 {
            offset_y = 20.0;
        }

        let tooltip_pos = pos + egui::vec2(offset_x, offset_y);

        // Background with rounded corners
        let bg_rect =
            egui::Rect::from_min_size(tooltip_pos, egui::vec2(tooltip_width, tooltip_height));
        painter.rect_filled(bg_rect, 6.0, egui::Color32::from_black_alpha(210));
        painter.rect_stroke(
            bg_rect,
            6.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            egui::StrokeKind::Outside,
        );

        // Color swatch
        let swatch_size = 18.0;
        let swatch_rect = egui::Rect::from_min_size(
            tooltip_pos + egui::vec2(5.0, (tooltip_height - swatch_size) / 2.0),
            egui::vec2(swatch_size, swatch_size),
        );
        painter.rect_filled(swatch_rect, 3.0, color.to_egui_color32());
        painter.rect_stroke(
            swatch_rect,
            3.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
            egui::StrokeKind::Outside,
        );

        // Hex text
        painter.galley(
            tooltip_pos + egui::vec2(28.0, (tooltip_height - text_galley.size().y) / 2.0),
            text_galley,
            egui::Color32::WHITE,
        );
    }

    fn magnifier_position(&self, cursor: egui::Pos2, screen_size: egui::Vec2) -> egui::Pos2 {
        let margin = MAGNIFIER_RADIUS + 30.0;
        let offset = MAGNIFIER_RADIUS + 40.0;

        // Default: bottom-right of cursor
        let mut center = cursor + egui::vec2(offset, offset);

        // Flip horizontally if too close to right edge
        if center.x + margin > screen_size.x {
            center.x = cursor.x - offset;
        }
        // Flip vertically if too close to bottom edge
        if center.y + margin > screen_size.y {
            center.y = cursor.y - offset;
        }
        // Ensure not off left/top edges
        if center.x - margin < 0.0 {
            center.x = margin;
        }
        if center.y - margin < 0.0 {
            center.y = margin;
        }

        center
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load texture on first frame
        if self.texture.is_none() {
            let size = [
                self.screenshot.width() as usize,
                self.screenshot.height() as usize,
            ];
            let pixels = self.screenshot.as_raw();
            let image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
            self.texture =
                Some(ctx.load_texture("screenshot", image, egui::TextureOptions::LINEAR));
        }

        // Read input
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let scroll_delta = ctx.input(|i| i.raw_scroll_delta.y);
        let primary_clicked = ctx.input(|i| i.pointer.primary_clicked());
        let secondary_clicked = ctx.input(|i| i.pointer.secondary_clicked());
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        // Handle cancel
        if escape || secondary_clicked {
            self.should_close = true;
        }

        // Handle scroll zoom
        if scroll_delta != 0.0 {
            let delta = if scroll_delta > 0.0 { 1.0 } else { -1.0 };
            self.zoom_level = (self.zoom_level + delta).clamp(MIN_ZOOM, MAX_ZOOM);
        }

        // Draw screenshot background
        let screen_size = egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let available = ui.available_size();
                if let Some(tex) = &self.texture {
                    ui.image(egui::load::SizedTexture::new(tex.id(), available));
                }
                available
            })
            .inner;

        // Draw magnifier and tooltip
        if let Some(pos) = pointer_pos {
            let (px_x, px_y) = self.screen_to_pixel(pos, screen_size);
            let current_color = self.color_at(px_x, px_y);

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("overlay_ui"),
            ));

            let mag_center = self.magnifier_position(pos, screen_size);
            self.draw_magnifier(&painter, mag_center, px_x, px_y, screen_size);
            self.draw_tooltip(&painter, pos, current_color, screen_size);

            // Pick on left click
            if primary_clicked && !self.should_close {
                *self.result.lock().unwrap() = Some(current_color);
                self.should_close = true;
            }
        }

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Repaint continuously for smooth cursor tracking
        ctx.request_repaint();
    }
}

/// Run the overlay and return the picked color (or None if cancelled).
pub fn run_overlay(screenshot: RgbaImage) -> Option<Color> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let img_w = screenshot.width() as f32;
    let img_h = screenshot.height() as f32;
    let on_x11 = std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok();

    // On X11, spawn a helper thread that sets override_redirect on our window
    // so it can span all monitors (the WM normally constrains to one).
    if on_x11 {
        let w = screenshot.width();
        let h = screenshot.height();
        std::thread::spawn(move || {
            x11_configure_overlay(w, h);
        });
    }

    let native_options = eframe::NativeOptions {
        viewport: if on_x11 {
            // On X11: borderless window sized to the full virtual desktop.
            // The override_redirect thread will ensure the WM doesn't constrain it.
            egui::ViewportBuilder::default()
                .with_decorations(false)
                .with_always_on_top()
                .with_position(egui::pos2(0.0, 0.0))
                .with_inner_size(egui::vec2(img_w, img_h))
                .with_resizable(false)
        } else {
            // On Wayland: fullscreen on the current monitor.
            // The screenshot is already cropped to the active monitor.
            egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_decorations(false)
                .with_always_on_top()
        },
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Color Picker",
        native_options,
        Box::new(move |_cc| Ok(Box::new(OverlayApp::new(screenshot, result_clone)))),
    );

    result.lock().unwrap().take()
}

/// On X11, find our overlay window and set override_redirect so it can span
/// all monitors without being constrained by the window manager.
fn x11_configure_overlay(width: u32, height: u32) {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    for _ in 0..100 {
        thread::sleep(Duration::from_millis(30));

        let Ok(output) = Command::new("xdotool")
            .args(["search", "--name", "Color Picker"])
            .output()
        else {
            return; // xdotool not available, skip
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let Some(wid) = stdout.lines().last() else {
            continue;
        };
        let wid = wid.trim();
        if wid.is_empty() {
            continue;
        }

        // Set override_redirect to bypass WM per-monitor constraints
        let _ = Command::new("xdotool")
            .args(["set_window", "--overrideredirect", "1", wid])
            .status();

        // Resize to span all monitors
        let _ = Command::new("xdotool")
            .args(["windowsize", wid, &width.to_string(), &height.to_string()])
            .status();

        // Position at top-left of virtual desktop
        let _ = Command::new("xdotool")
            .args(["windowmove", wid, "0", "0"])
            .status();

        // Raise and give focus so we receive keyboard/mouse input
        let _ = Command::new("xdotool")
            .args(["windowactivate", "--sync", wid])
            .status();

        return;
    }
}
