use eframe::egui;
use std::sync::{Arc, Mutex};

const CHAR_BUTTON_SIZE: f32 = 44.0;
const CHAR_BUTTON_SPACING: f32 = 6.0;
const TOOLBAR_PADDING: f32 = 10.0;
const FONT_SIZE: f32 = 22.0;
const CORNER_RADIUS: f32 = 12.0;

const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgba_premultiplied(30, 30, 36, 240);
const BG_SELECTED: egui::Color32 = egui::Color32::from_rgb(66, 135, 245);
const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(50, 50, 60);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::WHITE;
const TEXT_HINT: egui::Color32 = egui::Color32::from_gray(130);
const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(80, 80, 90, 200);

struct AccentOverlayApp {
    accents: Vec<char>,
    selected_idx: usize,
    result: Arc<Mutex<Option<char>>>,
    should_close: bool,
    had_focus: bool,
    frame_count: u32,
}

impl AccentOverlayApp {
    fn new(accents: Vec<char>, result: Arc<Mutex<Option<char>>>) -> Self {
        Self {
            accents,
            selected_idx: 0,
            result,
            should_close: false,
            had_focus: false,
            frame_count: 0,
        }
    }

    fn confirm_selection(&mut self) {
        if let Some(&ch) = self.accents.get(self.selected_idx) {
            *self.result.lock().unwrap() = Some(ch);
        }
        self.should_close = true;
    }
}

impl eframe::App for AccentOverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count = self.frame_count.saturating_add(1);

        let n = self.accents.len();

        // Key handling
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.should_close = true;
            }
            if i.key_pressed(egui::Key::Enter) {
                self.confirm_selection();
            }
            if i.key_pressed(egui::Key::Space) {
                self.selected_idx = (self.selected_idx + 1) % n;
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.selected_idx = (self.selected_idx + 1) % n;
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.selected_idx = (self.selected_idx + n - 1) % n;
            }
        });

        // Close on focus loss (after having gained focus at least once)
        let has_focus = ctx.input(|i| i.focused);
        if has_focus {
            self.had_focus = true;
        }
        if !has_focus && self.had_focus {
            self.should_close = true;
        }

        // Draw
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BG_PRIMARY)
                    .corner_radius(CORNER_RADIUS)
                    .stroke(egui::Stroke::new(1.0, BORDER_COLOR))
                    .inner_margin(TOOLBAR_PADDING),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = CHAR_BUTTON_SPACING;

                    let mut clicked_idx: Option<usize> = None;
                    let accents = self.accents.clone();

                    for (i, &ch) in accents.iter().enumerate() {
                        let is_selected = i == self.selected_idx;

                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(CHAR_BUTTON_SIZE, CHAR_BUTTON_SIZE),
                            egui::Sense::click(),
                        );

                        let bg = if is_selected {
                            BG_SELECTED
                        } else if response.hovered() {
                            BG_HOVER
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let painter = ui.painter();
                        painter.rect_filled(rect, 8.0, bg);

                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            ch.to_string(),
                            egui::FontId::proportional(FONT_SIZE),
                            TEXT_PRIMARY,
                        );

                        if response.hovered() {
                            self.selected_idx = i;
                        }

                        if response.clicked() {
                            clicked_idx = Some(i);
                        }
                    }

                    if let Some(idx) = clicked_idx {
                        self.selected_idx = idx;
                        self.confirm_selection();
                    }
                });

                // Hint line
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Space/Arrows: navigate  Enter: confirm  Esc: cancel")
                            .size(10.0)
                            .color(TEXT_HINT),
                    );
                });
            });

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();
    }
}

/// Run the accent overlay. Returns the selected character or None if cancelled.
pub fn run_overlay(accents: &[char], position: &str) -> Option<char> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    let accents_vec = accents.to_vec();
    let n = accents.len();

    let toolbar_width = n as f32 * (CHAR_BUTTON_SIZE + CHAR_BUTTON_SPACING) + TOOLBAR_PADDING * 2.0;
    let toolbar_height = CHAR_BUTTON_SIZE + TOOLBAR_PADDING * 2.0 + 24.0; // extra for hint

    let (cx, cy) = get_cursor_position().unwrap_or((800, 400));

    let (wx, wy) = match position {
        "above-cursor" => (
            cx - toolbar_width as i32 / 2,
            cy - toolbar_height as i32 - 20,
        ),
        "below-cursor" => (cx - toolbar_width as i32 / 2, cy + 30),
        "top-center" | "bottom-center" => {
            // Center horizontally on screen, use cursor Y as fallback
            (
                cx - toolbar_width as i32 / 2,
                cy - toolbar_height as i32 / 2,
            )
        }
        _ => (
            cx - toolbar_width as i32 / 2,
            cy - toolbar_height as i32 - 20,
        ),
    };

    // Ensure window stays on screen
    let wx = wx.max(10);
    let wy = wy.max(10);

    let on_x11 = std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok();

    // On X11, spawn a helper thread to activate the window
    if on_x11 {
        std::thread::spawn(|| {
            x11_activate_overlay();
        });
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_position(egui::pos2(wx as f32, wy as f32))
            .with_inner_size(egui::vec2(toolbar_width, toolbar_height))
            .with_resizable(false),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Quick Accent",
        native_options,
        Box::new(move |cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(AccentOverlayApp::new(accents_vec, result_clone)))
        }),
    );

    result.lock().unwrap().take()
}

fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::TRANSPARENT;
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
    ctx.set_visuals(visuals);
}

fn get_cursor_position() -> Option<(i32, i32)> {
    let output = std::process::Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut x = None;
    let mut y = None;
    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("X=") {
            x = v.parse().ok();
        }
        if let Some(v) = line.strip_prefix("Y=") {
            y = v.parse().ok();
        }
    }
    x.zip(y)
}

/// On X11, find the overlay window and activate it so it receives keyboard focus.
fn x11_activate_overlay() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    for _ in 0..100 {
        thread::sleep(Duration::from_millis(30));

        let Ok(output) = Command::new("xdotool")
            .args(["search", "--name", "Quick Accent"])
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
            .args(["windowactivate", "--sync", wid])
            .status();

        return;
    }
}
