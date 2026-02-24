use crate::layout::Layout;
use eframe::egui;
use mpt_common::monitor::Monitor;
use std::sync::{Arc, Mutex};

const ZONE_FILL_ALPHA: u8 = 50;
const ZONE_FILL_HIGHLIGHT_ALPHA: u8 = 100;
const ZONE_BORDER_RADIUS: f32 = 8.0;
const ZONE_NUMBER_FONT_SIZE: f32 = 48.0;
const LAYOUT_NAME_FONT_SIZE: f32 = 20.0;
const HINT_FONT_SIZE: f32 = 14.0;

struct ZoneOverlayApp {
    layout: Layout,
    zone_gap: u32,
    highlighted: Option<usize>,
    result: Arc<Mutex<Option<usize>>>,
    should_close: bool,
}

impl ZoneOverlayApp {
    fn new(layout: Layout, zone_gap: u32, result: Arc<Mutex<Option<usize>>>) -> Self {
        Self {
            layout,
            zone_gap,
            highlighted: None,
            result,
            should_close: false,
        }
    }
}

impl eframe::App for ZoneOverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Read keyboard input
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let arrow_right = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let arrow_left = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let arrow_down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let arrow_up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let primary_clicked = ctx.input(|i| i.pointer.primary_clicked());

        // Number keys 1-9 for direct zone selection
        let num_key = [
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
            egui::Key::Num6,
            egui::Key::Num7,
            egui::Key::Num8,
            egui::Key::Num9,
        ]
        .iter()
        .position(|k| ctx.input(|i| i.key_pressed(*k)));

        // Handle escape
        if escape {
            self.should_close = true;
        }

        // Handle direct number selection
        if let Some(idx) = num_key
            && idx < self.layout.zones.len()
        {
            *self.result.lock().unwrap() = Some(idx);
            self.should_close = true;
        }

        // Handle arrow navigation
        if arrow_right || arrow_down {
            let len = self.layout.zones.len();
            if len > 0 {
                let current = self.highlighted.unwrap_or(len.wrapping_sub(1));
                self.highlighted = Some((current + 1) % len);
            }
        }
        if arrow_left || arrow_up {
            let len = self.layout.zones.len();
            if len > 0 {
                let current = self.highlighted.unwrap_or(1);
                self.highlighted = Some((current + len - 1) % len);
            }
        }

        // Handle enter to confirm highlighted zone
        if enter && let Some(idx) = self.highlighted {
            *self.result.lock().unwrap() = Some(idx);
            self.should_close = true;
        }

        // Draw the overlay
        let screen_size = egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_black_alpha(80)))
            .show(ctx, |ui| ui.available_size())
            .inner;

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("zone_overlay"),
        ));

        let sw = screen_size.x as u32;
        let sh = screen_size.y as u32;

        // Detect zone under mouse cursor
        let mouse_zone = pointer_pos.and_then(|pos| {
            let px = pos.x / screen_size.x;
            let py = pos.y / screen_size.y;
            self.layout.zone_at(px, py)
        });

        // If mouse moves over a zone, highlight it
        if mouse_zone.is_some() {
            self.highlighted = mouse_zone;
        }

        // Click to select zone
        if primary_clicked && let Some(idx) = self.highlighted {
            *self.result.lock().unwrap() = Some(idx);
            self.should_close = true;
        }

        // Draw each zone
        for (i, zone) in self.layout.zones.iter().enumerate() {
            let (x, y, w, h) = zone.to_pixels_with_gap(sw, sh, self.zone_gap);
            let rect = egui::Rect::from_min_size(
                egui::pos2(x as f32, y as f32),
                egui::vec2(w as f32, h as f32),
            );

            let is_highlighted = self.highlighted == Some(i);

            let fill_alpha = if is_highlighted {
                ZONE_FILL_HIGHLIGHT_ALPHA
            } else {
                ZONE_FILL_ALPHA
            };
            let fill = egui::Color32::from_rgba_unmultiplied(66, 135, 245, fill_alpha);

            let stroke_color = if is_highlighted {
                egui::Color32::from_rgb(100, 160, 255)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120)
            };
            let stroke_width = if is_highlighted { 3.0 } else { 1.5 };

            painter.rect_filled(rect, ZONE_BORDER_RADIUS, fill);
            painter.rect_stroke(
                rect,
                ZONE_BORDER_RADIUS,
                egui::Stroke::new(stroke_width, stroke_color),
                egui::StrokeKind::Outside,
            );

            // Zone number label
            let label = format!("{}", i + 1);
            let galley = painter.layout_no_wrap(
                label,
                egui::FontId::proportional(ZONE_NUMBER_FONT_SIZE),
                if is_highlighted {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_white_alpha(180)
                },
            );
            painter.galley(
                rect.center() - galley.size() / 2.0,
                galley,
                egui::Color32::WHITE,
            );
        }

        // Layout name at top center
        let name_galley = painter.layout_no_wrap(
            self.layout.name.clone(),
            egui::FontId::proportional(LAYOUT_NAME_FONT_SIZE),
            egui::Color32::from_white_alpha(200),
        );
        painter.galley(
            egui::pos2(screen_size.x / 2.0 - name_galley.size().x / 2.0, 16.0),
            name_galley,
            egui::Color32::WHITE,
        );

        // Hint text at bottom center
        let hint =
            "Press 1-9 to snap  |  Arrows to navigate  |  Enter to confirm  |  Esc to cancel";
        let hint_galley = painter.layout_no_wrap(
            hint.to_string(),
            egui::FontId::proportional(HINT_FONT_SIZE),
            egui::Color32::from_white_alpha(140),
        );
        painter.galley(
            egui::pos2(
                screen_size.x / 2.0 - hint_galley.size().x / 2.0,
                screen_size.y - 36.0,
            ),
            hint_galley,
            egui::Color32::WHITE,
        );

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();
    }
}

/// Run the zone overlay on a specific monitor and return the selected zone index.
pub fn run_overlay(layout: Layout, zone_gap: u32, monitor: &Monitor) -> Option<usize> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let on_x11 = std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok();

    let mon_x = monitor.x;
    let mon_y = monitor.y;
    let mon_w = monitor.width;
    let mon_h = monitor.height;

    // On X11, spawn helper thread to set override_redirect and position on monitor
    if on_x11 {
        std::thread::spawn(move || {
            x11_configure_overlay(mon_x, mon_y, mon_w, mon_h);
        });
    }

    let native_options = eframe::NativeOptions {
        viewport: if on_x11 {
            egui::ViewportBuilder::default()
                .with_decorations(false)
                .with_always_on_top()
                .with_transparent(true)
                .with_position(egui::pos2(mon_x as f32, mon_y as f32))
                .with_inner_size(egui::vec2(mon_w as f32, mon_h as f32))
                .with_resizable(false)
        } else {
            egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_decorations(false)
                .with_always_on_top()
                .with_transparent(true)
        },
        ..Default::default()
    };

    let _ = eframe::run_native(
        "FancyZones",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(ZoneOverlayApp::new(
                layout,
                zone_gap,
                result_clone,
            )))
        }),
    );

    result.lock().unwrap().take()
}

/// On X11, find our overlay window and position it on the target monitor.
fn x11_configure_overlay(mon_x: i32, mon_y: i32, mon_w: u32, mon_h: u32) {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    for _ in 0..100 {
        thread::sleep(Duration::from_millis(30));

        let Ok(output) = Command::new("xdotool")
            .args(["search", "--name", "FancyZones"])
            .output()
        else {
            return;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let Some(wid) = stdout.lines().last() else {
            continue;
        };
        let wid = wid.trim();
        if wid.is_empty() {
            continue;
        }

        let _ = Command::new("xdotool")
            .args(["set_window", "--overrideredirect", "1", wid])
            .status();

        let _ = Command::new("xdotool")
            .args(["windowsize", wid, &mon_w.to_string(), &mon_h.to_string()])
            .status();

        let _ = Command::new("xdotool")
            .args(["windowmove", wid, &mon_x.to_string(), &mon_y.to_string()])
            .status();

        let _ = Command::new("xdotool")
            .args(["windowactivate", "--sync", wid])
            .status();

        return;
    }
}
