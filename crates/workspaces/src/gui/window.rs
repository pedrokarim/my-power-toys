use crate::config::{self, AppEntry, Workspace, WorkspacesConfig};
use crate::gui::icons::IconCache;
use crate::gui::theme;
use crate::launcher;
use crate::x11;
use eframe::egui;
use tracing::{info, warn};

// ── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    List,
    Edit(usize),
}

#[derive(Clone)]
enum Action {
    Launch(usize),
    Edit(usize),
    Delete(usize),
}

// ── App state ───────────────────────────────────────────────────────────────

struct WorkspacesApp {
    config: WorkspacesConfig,
    mode: Mode,
    frame_count: u32,
    icon_cache: IconCache,
    // Edit mode state
    edit_name: String,
    edit_args: Vec<String>,
    edit_positions: Vec<[String; 4]>, // [x, y, w, h] as strings for text fields
    // Status message
    status: Option<(String, std::time::Instant, bool)>, // (message, timestamp, is_error)
}

impl WorkspacesApp {
    fn new(config: WorkspacesConfig) -> Self {
        Self {
            config,
            mode: Mode::List,
            frame_count: 0,
            icon_cache: IconCache::new(),
            edit_name: String::new(),
            edit_args: Vec::new(),
            edit_positions: Vec::new(),
            status: None,
        }
    }

    fn save_config(&self) {
        if let Err(e) = mpt_common::config::save_module_config("workspaces", &self.config) {
            warn!("Failed to save Workspaces config: {e}");
        }
    }

    fn enter_edit_mode(&mut self, idx: usize) {
        if let Some(ws) = self.config.workspaces.get(idx) {
            self.edit_name = ws.name.clone();
            self.edit_args = ws.apps.iter().map(|a| a.args.join(" ")).collect();
            self.edit_positions = ws
                .apps
                .iter()
                .map(|a| {
                    [
                        a.x.to_string(),
                        a.y.to_string(),
                        a.width.to_string(),
                        a.height.to_string(),
                    ]
                })
                .collect();
            self.mode = Mode::Edit(idx);
        }
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = Some((msg.into(), std::time::Instant::now(), is_error));
    }

    fn capture_workspace(&mut self) {
        info!("Capturing current desktop state");

        let windows = match x11::list_windows() {
            Ok(w) => w,
            Err(e) => {
                self.set_status(format!("Capture failed: {e}"), true);
                return;
            }
        };

        if windows.is_empty() {
            self.set_status("No windows found to capture", true);
            return;
        }

        let monitors = mpt_common::monitor::detect_monitors().unwrap_or_default();

        let apps: Vec<AppEntry> = windows
            .into_iter()
            .map(|win| {
                let exec = win
                    .pid
                    .and_then(config::resolve_exec_from_pid)
                    .unwrap_or_default();

                let monitor = monitors
                    .iter()
                    .find(|m| {
                        win.x >= m.x
                            && win.x < m.x + m.width as i32
                            && win.y >= m.y
                            && win.y < m.y + m.height as i32
                    })
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                AppEntry {
                    name: if win.title.is_empty() {
                        win.wm_class.clone()
                    } else {
                        win.title
                    },
                    wm_class: win.wm_class,
                    exec,
                    args: win
                        .pid
                        .map(config::resolve_args_from_pid)
                        .unwrap_or_default(),
                    x: win.x,
                    y: win.y,
                    width: win.width,
                    height: win.height,
                    monitor,
                    enabled: true,
                    minimized: win.minimized,
                }
            })
            .collect();

        let count = apps.len();
        let ws = Workspace::new(
            format!("Workspace {}", self.config.workspaces.len() + 1),
            apps,
        );
        self.config.workspaces.push(ws);
        self.save_config();

        let idx = self.config.workspaces.len() - 1;
        self.enter_edit_mode(idx);
        self.set_status(
            format!("Captured {count} app(s) — name your workspace"),
            false,
        );
    }

    fn launch_workspace(&mut self, idx: usize) {
        if let Some(ws) = self.config.workspaces.get_mut(idx) {
            info!("Launching workspace: {}", ws.name);
            let statuses = launcher::launch_workspace(ws);
            self.save_config();

            let launched = statuses
                .iter()
                .filter(|s| matches!(s, launcher::LaunchStatus::Launched { .. }))
                .count();
            let repositioned = statuses
                .iter()
                .filter(|s| matches!(s, launcher::LaunchStatus::Repositioned { .. }))
                .count();
            let failed = statuses
                .iter()
                .filter(|s| matches!(s, launcher::LaunchStatus::Failed { .. }))
                .count();

            self.set_status(
                format!("Launched {launched}, repositioned {repositioned}, failed {failed}"),
                failed > 0,
            );
        }
    }

    fn delete_workspace(&mut self, idx: usize) {
        if idx < self.config.workspaces.len() {
            let name = self.config.workspaces[idx].name.clone();
            self.config.workspaces.remove(idx);
            self.save_config();
            self.mode = Mode::List;
            self.set_status(format!("Deleted workspace \"{name}\""), false);
        }
    }

    // ── Drawing helpers ─────────────────────────────────────────────────────

    fn draw_header(&self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_MARGIN as i8, 14));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Custom-drawn workspace icon: stacked windows
                let icon_size = 28.0;
                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
                let p = ui.painter_at(icon_rect);

                // Back window (larger, offset up-left)
                let r1 = egui::Rect::from_min_size(
                    icon_rect.min,
                    egui::vec2(icon_size * 0.7, icon_size * 0.55),
                );
                p.rect_stroke(
                    r1,
                    2.0,
                    egui::Stroke::new(1.5, theme::TEXT_MUTED),
                    egui::StrokeKind::Inside,
                );

                // Front window (larger, offset down-right)
                let r2 = egui::Rect::from_min_size(
                    egui::pos2(
                        icon_rect.left() + icon_size * 0.3,
                        icon_rect.top() + icon_size * 0.35,
                    ),
                    egui::vec2(icon_size * 0.7, icon_size * 0.55),
                );
                p.rect_filled(r2, 2.0, theme::BG_HEADER);
                p.rect_stroke(
                    r2,
                    2.0,
                    egui::Stroke::new(1.5, theme::TEXT_ACCENT),
                    egui::StrokeKind::Inside,
                );

                // Title bar line inside front window
                let title_bar_y = r2.top() + 5.0;
                p.line_segment(
                    [
                        egui::pos2(r2.left() + 4.0, title_bar_y),
                        egui::pos2(r2.right() - 4.0, title_bar_y),
                    ],
                    egui::Stroke::new(1.0, theme::TEXT_ACCENT),
                );

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Workspaces Editor")
                            .size(theme::FONT_TITLE)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("Save and restore your desktop app layouts")
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn draw_section_label(ui: &mut egui::Ui, label: &str) {
        ui.horizontal(|ui| {
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

    fn draw_separator(ui: &mut egui::Ui) {
        let (sep_rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
        ui.painter_at(sep_rect)
            .rect_filled(sep_rect, 0.0, theme::SEPARATOR);
    }

    fn draw_status_bar(&self, ui: &mut egui::Ui) {
        if let Some((msg, ts, is_error)) = &self.status
            && ts.elapsed().as_secs() < 8
        {
            let color = if *is_error {
                theme::TEXT_ERROR
            } else {
                theme::TEXT_SUCCESS
            };

            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
            let p = ui.painter_at(rect);
            let bg = if *is_error {
                theme::BG_ERROR
            } else {
                theme::BG_SUCCESS
            };
            p.rect_filled(rect, 4.0, bg);
            p.text(
                egui::pos2(rect.left() + 12.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                msg,
                egui::FontId::proportional(theme::FONT_SMALL),
                color,
            );
        }
    }

    fn draw_small_button(
        ui: &mut egui::Ui,
        label: &str,
        width: f32,
        accent: bool,
        id: egui::Id,
    ) -> bool {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());
        let p = ui.painter_at(rect);

        let bg = if accent {
            if response.hovered() {
                theme::BG_BUTTON_HOVER
            } else {
                theme::BG_BUTTON
            }
        } else if response.hovered() {
            theme::BG_HOVER
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(rect, 5.0, bg);

        let text_color = if accent {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        };
        p.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(theme::FONT_SMALL),
            text_color,
        );

        let _ = id; // used by caller for uniqueness
        response.clicked()
    }

    fn draw_action_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
        let width = 180.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(width, theme::BUTTON_HEIGHT),
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

        let text_color = if enabled {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_MUTED
        };
        p.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(theme::FONT_BUTTON),
            text_color,
        );

        enabled && response.clicked()
    }

    // ── List mode ───────────────────────────────────────────────────────────

    fn draw_workspace_list(&mut self, ui: &mut egui::Ui) {
        // Section header with Create button
        ui.horizontal(|ui| {
            Self::draw_section_label(ui, "WORKSPACES");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Self::draw_small_button(
                    ui,
                    "+  Create Workspace",
                    145.0,
                    true,
                    egui::Id::new("create_ws"),
                ) {
                    self.capture_workspace();
                }

                // Workspace count
                ui.label(
                    egui::RichText::new(format!(
                        "{} workspace{}",
                        self.config.workspaces.len(),
                        if self.config.workspaces.len() != 1 {
                            "s"
                        } else {
                            ""
                        }
                    ))
                    .size(theme::FONT_SMALL)
                    .color(theme::TEXT_MUTED),
                );
            });
        });

        ui.add_space(8.0);

        if self.config.workspaces.is_empty() {
            // Empty state
            let available_w = ui.available_width();
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(available_w, 140.0), egui::Sense::hover());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, theme::CARD_RADIUS, theme::BG_SECONDARY);
            p.rect_stroke(
                rect,
                theme::CARD_RADIUS,
                egui::Stroke::new(1.0, theme::CARD_BORDER),
                egui::StrokeKind::Inside,
            );

            // Draw stacked windows icon
            let cx = rect.center().x;
            let cy = rect.center().y - 16.0;
            let icon_color = theme::TEXT_MUTED;
            let r1 = egui::Rect::from_center_size(
                egui::pos2(cx - 4.0, cy - 4.0),
                egui::vec2(32.0, 22.0),
            );
            p.rect_stroke(
                r1,
                3.0,
                egui::Stroke::new(1.5, icon_color),
                egui::StrokeKind::Inside,
            );
            let r2 = egui::Rect::from_center_size(
                egui::pos2(cx + 4.0, cy + 4.0),
                egui::vec2(32.0, 22.0),
            );
            p.rect_filled(r2, 3.0, theme::BG_SECONDARY);
            p.rect_stroke(
                r2,
                3.0,
                egui::Stroke::new(1.5, icon_color),
                egui::StrokeKind::Inside,
            );

            p.text(
                egui::pos2(cx, cy + 28.0),
                egui::Align2::CENTER_CENTER,
                "No workspaces yet",
                egui::FontId::proportional(theme::FONT_BODY),
                theme::TEXT_MUTED,
            );
            p.text(
                egui::pos2(cx, cy + 46.0),
                egui::Align2::CENTER_CENTER,
                "Click \"Create Workspace\" to capture your desktop layout",
                egui::FontId::proportional(theme::FONT_SMALL),
                theme::TEXT_MUTED,
            );
        } else {
            // Workspace cards in scroll area
            let mut action = None;
            let ws_count = self.config.workspaces.len();
            let ctx = ui.ctx().clone();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..ws_count {
                        let ws = self.config.workspaces[i].clone();
                        self.draw_workspace_card(&ctx, ui, i, &ws, &mut action);
                        ui.add_space(4.0);
                    }
                });

            if let Some(action) = action {
                match action {
                    Action::Launch(i) => self.launch_workspace(i),
                    Action::Edit(i) => self.enter_edit_mode(i),
                    Action::Delete(i) => self.delete_workspace(i),
                }
            }
        }
    }

    fn draw_workspace_card(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        idx: usize,
        ws: &Workspace,
        action: &mut Option<Action>,
    ) {
        let available_w = ui.available_width();
        let card_h = theme::WORKSPACE_CARD_HEIGHT;
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, card_h), egui::Sense::hover());
        let p = ui.painter_at(rect);

        // Card background
        p.rect_filled(rect, theme::CARD_RADIUS, theme::BG_CARD);
        p.rect_stroke(
            rect,
            theme::CARD_RADIUS,
            egui::Stroke::new(0.5, theme::CARD_BORDER),
            egui::StrokeKind::Inside,
        );

        // Left accent bar
        let accent_rect = egui::Rect::from_min_size(rect.min, egui::vec2(4.0, card_h));
        p.rect_filled(
            accent_rect,
            egui::CornerRadius {
                nw: theme::CARD_RADIUS as u8,
                sw: theme::CARD_RADIUS as u8,
                ne: 0,
                se: 0,
            },
            theme::TEXT_ACCENT,
        );

        // Workspace name
        let name_x = rect.left() + 16.0;
        p.text(
            egui::pos2(name_x, rect.top() + 18.0),
            egui::Align2::LEFT_CENTER,
            &ws.name,
            egui::FontId::proportional(theme::FONT_BODY),
            theme::TEXT_PRIMARY,
        );

        // App count + last launched
        let app_count = ws.app_count();
        let last = ws.last_launched.as_deref().unwrap_or("Never launched");
        let info_text = format!(
            "{} app{}  \u{00B7}  {}",
            app_count,
            if app_count != 1 { "s" } else { "" },
            last
        );
        p.text(
            egui::pos2(name_x, rect.top() + 36.0),
            egui::Align2::LEFT_CENTER,
            &info_text,
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_MUTED,
        );

        // App icons row
        let icon_y = rect.top() + 56.0;
        let icon_sz = 20.0;
        let mut icon_x = name_x;
        let app_ids: Vec<(String, String)> = ws
            .apps
            .iter()
            .take(8)
            .map(|a| (a.wm_class.clone(), a.exec.clone()))
            .collect();

        for (wm_class, exec) in &app_ids {
            if let Some(source) = self.icon_cache.get_for_app(ctx, wm_class, exec) {
                // Draw icon via egui Image in a sub-ui
                let icon_rect = egui::Rect::from_min_size(
                    egui::pos2(icon_x, icon_y - icon_sz / 2.0),
                    egui::vec2(icon_sz, icon_sz),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                    ui.add(
                        egui::Image::new(source).fit_to_exact_size(egui::vec2(icon_sz, icon_sz)),
                    );
                });
                icon_x += icon_sz + 6.0;
            } else {
                // Fallback: text badge
                let label = wm_class;
                let text_w = p
                    .layout_no_wrap(
                        label.to_string(),
                        egui::FontId::proportional(9.0),
                        theme::TEXT_SECONDARY,
                    )
                    .size()
                    .x;
                let badge_w = text_w + 12.0;
                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(icon_x, icon_y - 8.0),
                    egui::vec2(badge_w, 16.0),
                );
                p.rect_filled(badge_rect, 3.0, theme::BG_CHIP);
                p.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(9.0),
                    theme::TEXT_SECONDARY,
                );
                icon_x += badge_w + 4.0;
            }

            if icon_x > rect.right() - 220.0 {
                break;
            }
        }
        if ws.apps.len() > 8 {
            p.text(
                egui::pos2(icon_x + 4.0, icon_y),
                egui::Align2::LEFT_CENTER,
                format!("+{}", ws.apps.len() - 8),
                egui::FontId::proportional(9.0),
                theme::TEXT_MUTED,
            );
        }

        // Buttons on the right
        let button_y = rect.center().y;
        let button_area_right = rect.right() - 12.0;

        // Delete button (X)
        let delete_rect = egui::Rect::from_center_size(
            egui::pos2(button_area_right, button_y),
            egui::vec2(24.0, 24.0),
        );
        let delete_resp = ui.interact(
            delete_rect,
            egui::Id::new(("ws_delete", idx)),
            egui::Sense::click(),
        );
        if delete_resp.hovered() {
            p.rect_filled(delete_rect, 3.0, theme::BG_HOVER);
        }
        // Draw X with line segments (painter API)
        let x_color = if delete_resp.hovered() {
            theme::TEXT_ERROR
        } else {
            theme::TEXT_MUTED
        };
        let c = delete_rect.center();
        let s = 5.0; // half-size of the X
        p.line_segment(
            [egui::pos2(c.x - s, c.y - s), egui::pos2(c.x + s, c.y + s)],
            egui::Stroke::new(1.5, x_color),
        );
        p.line_segment(
            [egui::pos2(c.x + s, c.y - s), egui::pos2(c.x - s, c.y + s)],
            egui::Stroke::new(1.5, x_color),
        );
        if delete_resp.clicked() {
            *action = Some(Action::Delete(idx));
        }

        // Edit button
        let edit_rect = egui::Rect::from_center_size(
            egui::pos2(button_area_right - 60.0, button_y),
            egui::vec2(50.0, 26.0),
        );
        let edit_resp = ui.interact(
            edit_rect,
            egui::Id::new(("ws_edit", idx)),
            egui::Sense::click(),
        );
        let edit_bg = if edit_resp.hovered() {
            theme::BG_HOVER
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(edit_rect, 5.0, edit_bg);
        p.text(
            edit_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Edit",
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_SECONDARY,
        );
        if edit_resp.clicked() {
            *action = Some(Action::Edit(idx));
        }

        // Launch button (accent)
        let launch_rect = egui::Rect::from_center_size(
            egui::pos2(button_area_right - 130.0, button_y),
            egui::vec2(65.0, 26.0),
        );
        let launch_resp = ui.interact(
            launch_rect,
            egui::Id::new(("ws_launch", idx)),
            egui::Sense::click(),
        );
        let launch_bg = if launch_resp.hovered() {
            theme::BG_BUTTON_HOVER
        } else {
            theme::BG_BUTTON
        };
        p.rect_filled(launch_rect, 5.0, launch_bg);
        p.text(
            launch_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{25B6}  Launch",
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_PRIMARY,
        );
        if launch_resp.clicked() {
            *action = Some(Action::Launch(idx));
        }
    }

    // ── Monitor preview ─────────────────────────────────────────────────────

    fn draw_monitor_preview(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, ws: &Workspace) {
        let monitors = mpt_common::monitor::detect_monitors().unwrap_or_default();
        if monitors.is_empty() {
            return;
        }

        let preview_h = theme::MONITOR_PREVIEW_HEIGHT;
        let available_w = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, preview_h), egui::Sense::hover());
        let p = ui.painter_at(rect);

        // Background
        p.rect_filled(rect, theme::CARD_RADIUS, theme::BG_SECONDARY);
        p.rect_stroke(
            rect,
            theme::CARD_RADIUS,
            egui::Stroke::new(0.5, theme::CARD_BORDER),
            egui::StrokeKind::Inside,
        );

        // Compute bounding box of all monitors
        let total_x_min = monitors.iter().map(|m| m.x).min().unwrap_or(0);
        let total_y_min = monitors.iter().map(|m| m.y).min().unwrap_or(0);
        let total_x_max = monitors
            .iter()
            .map(|m| m.x + m.width as i32)
            .max()
            .unwrap_or(1920);
        let total_y_max = monitors
            .iter()
            .map(|m| m.y + m.height as i32)
            .max()
            .unwrap_or(1080);
        let total_w = (total_x_max - total_x_min) as f32;
        let total_h = (total_y_max - total_y_min) as f32;

        // Scale to fit in the preview rect with padding
        let pad = 16.0;
        let inner = rect.shrink(pad);
        let scale_x = inner.width() / total_w;
        let scale_y = inner.height() / total_h;
        let scale = scale_x.min(scale_y);

        // Center the preview
        let scaled_w = total_w * scale;
        let scaled_h = total_h * scale;
        let offset_x = inner.left() + (inner.width() - scaled_w) / 2.0;
        let offset_y = inner.top() + (inner.height() - scaled_h) / 2.0;

        let to_screen = |x: i32, y: i32| -> egui::Pos2 {
            egui::pos2(
                offset_x + (x - total_x_min) as f32 * scale,
                offset_y + (y - total_y_min) as f32 * scale,
            )
        };

        // Draw monitors
        for mon in &monitors {
            let tl = to_screen(mon.x, mon.y);
            let br = to_screen(mon.x + mon.width as i32, mon.y + mon.height as i32);
            let mon_rect = egui::Rect::from_min_max(tl, br);

            p.rect_filled(mon_rect, 3.0, theme::BG_CARD);
            p.rect_stroke(
                mon_rect,
                3.0,
                egui::Stroke::new(1.0, theme::CARD_BORDER),
                egui::StrokeKind::Inside,
            );
        }

        // Draw app windows on monitors
        let non_minimized: Vec<_> = ws
            .apps
            .iter()
            .filter(|a| a.enabled && !a.minimized)
            .collect();
        for app in &non_minimized {
            let tl = to_screen(app.x, app.y);
            let br = to_screen(app.x + app.width as i32, app.y + app.height as i32);
            let win_rect = egui::Rect::from_min_max(tl, br);

            p.rect_filled(win_rect, 2.0, theme::BG_CHIP);
            p.rect_stroke(
                win_rect,
                2.0,
                egui::Stroke::new(0.5, theme::TEXT_MUTED),
                egui::StrokeKind::Inside,
            );

            // Draw app icon in center if space allows
            let icon_sz = 16.0;
            if win_rect.width() > icon_sz + 4.0
                && win_rect.height() > icon_sz + 4.0
                && let Some(source) = self.icon_cache.get_for_app(ctx, &app.wm_class, &app.exec)
            {
                let icon_rect =
                    egui::Rect::from_center_size(win_rect.center(), egui::vec2(icon_sz, icon_sz));
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                    ui.add(
                        egui::Image::new(source).fit_to_exact_size(egui::vec2(icon_sz, icon_sz)),
                    );
                });
            }
        }

        // Draw minimized apps as a taskbar-like strip at the bottom
        let minimized: Vec<_> = ws
            .apps
            .iter()
            .filter(|a| a.enabled && a.minimized)
            .collect();
        if !minimized.is_empty() {
            let bar_h = 14.0;
            let bar_y = rect.bottom() - pad - bar_h;
            let bar_x = offset_x;
            let bar_w = scaled_w;
            let bar_rect =
                egui::Rect::from_min_size(egui::pos2(bar_x, bar_y), egui::vec2(bar_w, bar_h));
            p.rect_filled(bar_rect, 2.0, theme::BG_CHIP);

            let mut ix = bar_x + 4.0;
            for app in &minimized {
                if let Some(source) = self.icon_cache.get_for_app(ctx, &app.wm_class, &app.exec) {
                    let icon_rect = egui::Rect::from_min_size(
                        egui::pos2(ix, bar_y + 1.0),
                        egui::vec2(12.0, 12.0),
                    );
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                        ui.add(egui::Image::new(source).fit_to_exact_size(egui::vec2(12.0, 12.0)));
                    });
                }
                ix += 16.0;
            }
        }
    }

    // ── Edit mode ───────────────────────────────────────────────────────────

    fn draw_edit_mode(&mut self, ui: &mut egui::Ui, idx: usize) {
        if idx >= self.config.workspaces.len() {
            self.mode = Mode::List;
            return;
        }

        // Breadcrumb
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Label::new(
                        egui::RichText::new("Workspaces")
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_ACCENT),
                    )
                    .sense(egui::Sense::click()),
                )
                .clicked()
            {
                // Discard changes and go back
                self.config =
                    mpt_common::config::load_module_config("workspaces").unwrap_or_default();
                self.mode = Mode::List;
                return;
            }
            ui.label(
                egui::RichText::new(">")
                    .size(theme::FONT_SMALL)
                    .color(theme::TEXT_MUTED),
            );
            ui.label(
                egui::RichText::new("Edit Workspace")
                    .size(theme::FONT_SMALL)
                    .color(theme::TEXT_SECONDARY),
            );
        });

        ui.add_space(8.0);

        // Monitor preview
        let ws_clone = self.config.workspaces[idx].clone();
        let ctx = ui.ctx().clone();
        self.draw_monitor_preview(&ctx, ui, &ws_clone);

        ui.add_space(12.0);

        // Workspace name + checkboxes row
        ui.horizontal(|ui| {
            Self::draw_section_label(ui, "WORKSPACE NAME");
        });
        ui.add_space(4.0);

        // Name text field
        ui.add(
            egui::TextEdit::singleline(&mut self.edit_name)
                .desired_width(300.0)
                .font(egui::FontId::proportional(theme::FONT_BODY)),
        );

        ui.add_space(8.0);

        // Checkboxes row
        ui.horizontal(|ui| {
            // Create desktop shortcut
            let mut cs = self.config.workspaces[idx].create_shortcut;
            ui.checkbox(&mut cs, "");
            ui.label(
                egui::RichText::new("Create desktop shortcut")
                    .size(theme::FONT_BODY)
                    .color(theme::TEXT_SECONDARY),
            );
            if cs != self.config.workspaces[idx].create_shortcut {
                self.config.workspaces[idx].create_shortcut = cs;
            }

            ui.add_space(24.0);

            // Move existing toggle
            let mut me = self.config.workspaces[idx].move_existing;
            ui.checkbox(&mut me, "");
            ui.label(
                egui::RichText::new("Move existing windows")
                    .size(theme::FONT_BODY)
                    .color(theme::TEXT_SECONDARY),
            );
            if me != self.config.workspaces[idx].move_existing {
                self.config.workspaces[idx].move_existing = me;
            }
        });

        ui.add_space(theme::SECTION_SPACING);
        Self::draw_separator(ui);
        ui.add_space(theme::SECTION_SPACING);

        // Split apps into normal and minimized
        let app_count = self.config.workspaces[idx].apps.len();
        let normal_indices: Vec<usize> = (0..app_count)
            .filter(|&i| !self.config.workspaces[idx].apps[i].minimized)
            .collect();
        let minimized_indices: Vec<usize> = (0..app_count)
            .filter(|&i| self.config.workspaces[idx].apps[i].minimized)
            .collect();

        // Applications section
        let normal_count = normal_indices.len();
        ui.horizontal(|ui| {
            Self::draw_section_label(ui, "APPLICATIONS");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} app{}",
                        normal_count,
                        if normal_count != 1 { "s" } else { "" }
                    ))
                    .size(theme::FONT_SMALL)
                    .color(theme::TEXT_MUTED),
                );
            });
        });
        ui.add_space(6.0);

        // App list — use remaining space minus bottom buttons
        let available_height = ui.available_height() - 60.0; // reserve for buttons
        let mut to_remove = None;

        egui::ScrollArea::vertical()
            .max_height(available_height.max(100.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Normal apps
                for &i in &normal_indices {
                    self.draw_app_card(&ctx, ui, idx, i, &mut to_remove);
                    ui.add_space(4.0);
                }

                // Minimized apps section
                if !minimized_indices.is_empty() {
                    ui.add_space(8.0);
                    Self::draw_section_label(ui, "MINIMIZED APPS");
                    ui.add_space(6.0);

                    for &i in &minimized_indices {
                        self.draw_app_card(&ctx, ui, idx, i, &mut to_remove);
                        ui.add_space(4.0);
                    }
                }
            });

        // Remove deferred
        if let Some(i) = to_remove {
            self.config.workspaces[idx].apps.remove(i);
            if i < self.edit_args.len() {
                self.edit_args.remove(i);
            }
            if i < self.edit_positions.len() {
                self.edit_positions.remove(i);
            }
        }

        ui.add_space(8.0);

        // Bottom buttons
        ui.horizontal(|ui| {
            // Cancel button (left)
            let (cancel_rect, cancel_resp) = ui
                .allocate_exact_size(egui::vec2(90.0, theme::BUTTON_HEIGHT), egui::Sense::click());
            let p = ui.painter_at(cancel_rect);
            let bg = if cancel_resp.hovered() {
                theme::BG_HOVER
            } else {
                theme::BG_CHIP
            };
            p.rect_filled(cancel_rect, theme::BUTTON_RADIUS, bg);
            p.text(
                cancel_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Cancel",
                egui::FontId::proportional(theme::FONT_BUTTON),
                theme::TEXT_SECONDARY,
            );
            if cancel_resp.clicked() {
                self.config =
                    mpt_common::config::load_module_config("workspaces").unwrap_or_default();
                self.mode = Mode::List;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Self::draw_action_button(ui, "\u{25B6}  Save Workspace", true) {
                    // Apply name, args, and positions
                    if let Some(ws) = self.config.workspaces.get_mut(idx) {
                        ws.name = self.edit_name.clone();
                        for (i, args_str) in self.edit_args.iter().enumerate() {
                            if let Some(app) = ws.apps.get_mut(i) {
                                app.args = args_str.split_whitespace().map(String::from).collect();
                            }
                        }
                        for (i, pos) in self.edit_positions.iter().enumerate() {
                            if let Some(app) = ws.apps.get_mut(i) {
                                if let Ok(v) = pos[0].parse() {
                                    app.x = v;
                                }
                                if let Ok(v) = pos[1].parse() {
                                    app.y = v;
                                }
                                if let Ok(v) = pos[2].parse() {
                                    app.width = v;
                                }
                                if let Ok(v) = pos[3].parse() {
                                    app.height = v;
                                }
                            }
                        }
                    }
                    self.save_config();

                    // Handle desktop shortcut
                    if self.config.workspaces[idx].create_shortcut {
                        if let Err(e) = config::create_desktop_shortcut(&self.edit_name, idx) {
                            warn!("Failed to create shortcut: {e}");
                        }
                    } else {
                        config::remove_desktop_shortcut(&self.edit_name);
                    }

                    self.set_status("Workspace saved", false);
                    self.mode = Mode::List;
                }
            });
        });
    }

    fn draw_app_card(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        ws_idx: usize,
        app_idx: usize,
        to_remove: &mut Option<usize>,
    ) {
        let app = &self.config.workspaces[ws_idx].apps[app_idx];
        let app_name = app.name.clone();
        let app_class = app.wm_class.clone();
        let app_exec = app.exec.clone();
        let app_width = app.width;
        let app_height = app.height;
        let app_x = app.x;
        let app_y = app.y;
        let app_monitor = app.monitor.clone();
        let app_enabled = app.enabled;
        let has_args = app_idx < self.edit_args.len();

        // Calculate card height based on whether we show args
        let card_h = if has_args {
            theme::APP_CARD_EXPANDED_HEIGHT
        } else {
            theme::APP_CARD_HEIGHT
        };

        let available_w = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, card_h), egui::Sense::hover());
        let p = ui.painter_at(rect);

        // Card background
        let card_bg = if app_enabled {
            theme::BG_CARD
        } else {
            theme::BG_SECONDARY
        };
        p.rect_filled(rect, theme::CARD_RADIUS, card_bg);
        p.rect_stroke(
            rect,
            theme::CARD_RADIUS,
            egui::Stroke::new(0.5, theme::CARD_BORDER),
            egui::StrokeKind::Inside,
        );

        // Enable/disable checkbox area
        let checkbox_center = egui::pos2(rect.left() + 20.0, rect.top() + 28.0);
        let checkbox_rect = egui::Rect::from_center_size(checkbox_center, egui::vec2(18.0, 18.0));
        let checkbox_resp = ui.interact(
            checkbox_rect,
            egui::Id::new(("app_toggle", ws_idx, app_idx)),
            egui::Sense::click(),
        );

        // Draw checkbox
        let check_bg = if app_enabled {
            theme::BG_BUTTON
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(checkbox_rect, 3.0, check_bg);
        if app_enabled {
            // Draw checkmark
            let c = checkbox_rect.center();
            p.line_segment(
                [egui::pos2(c.x - 4.0, c.y), egui::pos2(c.x - 1.0, c.y + 3.0)],
                egui::Stroke::new(2.0, theme::TEXT_PRIMARY),
            );
            p.line_segment(
                [
                    egui::pos2(c.x - 1.0, c.y + 3.0),
                    egui::pos2(c.x + 5.0, c.y - 3.0),
                ],
                egui::Stroke::new(2.0, theme::TEXT_PRIMARY),
            );
        }
        if checkbox_resp.clicked() {
            self.config.workspaces[ws_idx].apps[app_idx].enabled = !app_enabled;
        }

        // App icon
        let icon_sz = 22.0;
        let icon_x = rect.left() + 42.0;
        if let Some(source) = self.icon_cache.get_for_app(ctx, &app_class, &app_exec) {
            let icon_rect = egui::Rect::from_min_size(
                egui::pos2(icon_x, rect.top() + 14.0 - icon_sz / 2.0 + 4.0),
                egui::vec2(icon_sz, icon_sz),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                ui.add(egui::Image::new(source).fit_to_exact_size(egui::vec2(icon_sz, icon_sz)));
            });
        }

        // App name + class
        let text_x = rect.left() + 42.0 + icon_sz + 6.0;
        let name_color = if app_enabled {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_MUTED
        };
        p.text(
            egui::pos2(text_x, rect.top() + 16.0),
            egui::Align2::LEFT_CENTER,
            &app_name,
            egui::FontId::proportional(theme::FONT_BODY),
            name_color,
        );
        p.text(
            egui::pos2(
                text_x
                    + p.layout_no_wrap(
                        app_name.clone(),
                        egui::FontId::proportional(theme::FONT_BODY),
                        name_color,
                    )
                    .size()
                    .x
                    + 8.0,
                rect.top() + 16.0,
            ),
            egui::Align2::LEFT_CENTER,
            format!("({})", app_class),
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_MUTED,
        );

        // Exec path + geometry
        p.text(
            egui::pos2(text_x, rect.top() + 36.0),
            egui::Align2::LEFT_CENTER,
            &app_exec,
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_SECONDARY,
        );

        // Position fields (editable Left / Top / Width / Height)
        let has_positions = app_idx < self.edit_positions.len();
        if has_positions {
            let field_y = rect.top() + 50.0;
            let field_w = 64.0;
            let field_h = 20.0;
            let label_font = egui::FontId::proportional(9.0);
            let labels = ["Left", "Top", "Width", "Height"];
            let mut cursor = text_x;

            for (fi, label) in labels.iter().enumerate() {
                p.text(
                    egui::pos2(cursor, field_y + 1.0),
                    egui::Align2::LEFT_TOP,
                    *label,
                    label_font.clone(),
                    theme::TEXT_MUTED,
                );
                let edit_rect = egui::Rect::from_min_size(
                    egui::pos2(cursor, field_y + 13.0),
                    egui::vec2(field_w, field_h),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_positions[app_idx][fi])
                            .desired_width(field_w)
                            .font(egui::FontId::monospace(theme::FONT_SMALL)),
                    );
                });
                cursor += field_w + 10.0;
            }

            // Monitor label
            p.text(
                egui::pos2(cursor + 4.0, field_y + 18.0),
                egui::Align2::LEFT_CENTER,
                format!("@{}", app_monitor),
                egui::FontId::proportional(theme::FONT_SMALL),
                theme::TEXT_MUTED,
            );
        } else {
            // Fallback: read-only compact display
            let pos_text = format!(
                "{}x{}  pos({},{})  @{}",
                app_width, app_height, app_x, app_y, app_monitor
            );
            p.text(
                egui::pos2(text_x, rect.top() + 52.0),
                egui::Align2::LEFT_CENTER,
                &pos_text,
                egui::FontId::proportional(theme::FONT_SMALL),
                theme::TEXT_MUTED,
            );
        }

        // Args field (if available)
        if has_args {
            let args_y = rect.top() + 76.0;
            p.text(
                egui::pos2(text_x, args_y + 12.0),
                egui::Align2::LEFT_CENTER,
                "CLI arguments",
                egui::FontId::proportional(theme::FONT_SMALL),
                theme::TEXT_SECONDARY,
            );

            // Text edit for args
            let args_rect = egui::Rect::from_min_size(
                egui::pos2(text_x + 90.0, args_y + 2.0),
                egui::vec2(available_w - text_x - 90.0 - 70.0, 22.0),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(args_rect), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_args[app_idx])
                        .desired_width(args_rect.width())
                        .font(egui::FontId::monospace(theme::FONT_SMALL)),
                );
            });
        }

        // Remove button
        let remove_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 30.0, rect.top() + 28.0),
            egui::vec2(44.0, 24.0),
        );
        let remove_resp = ui.interact(
            remove_rect,
            egui::Id::new(("app_remove", ws_idx, app_idx)),
            egui::Sense::click(),
        );
        let remove_bg = if remove_resp.hovered() {
            theme::BG_ERROR
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(remove_rect, 4.0, remove_bg);
        p.text(
            remove_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Remove",
            egui::FontId::proportional(9.0),
            if remove_resp.hovered() {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            },
        );
        if remove_resp.clicked() {
            *to_remove = Some(app_idx);
        }
    }
}

// ── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for WorkspacesApp {
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

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::BG_PRIMARY))
            .show(ctx, |ui| {
                // Header
                self.draw_header(ui);

                // Main content
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(
                        theme::INNER_MARGIN as i8,
                        theme::INNER_MARGIN as i8,
                    ))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;

                        // Status bar
                        self.draw_status_bar(ui);

                        match self.mode.clone() {
                            Mode::List => self.draw_workspace_list(ui),
                            Mode::Edit(idx) => self.draw_edit_mode(ui, idx),
                        }
                    });
            });
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub fn run_editor(config: WorkspacesConfig) {
    info!("Opening Workspaces Editor");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT])
            .with_min_inner_size([600.0, 450.0])
            .with_title("Workspaces Editor")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Workspaces Editor",
        options,
        Box::new(|cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(WorkspacesApp::new(config)))
        }),
    )
    .ok();
}

fn setup_visuals(ctx: &egui::Context) {
    theme::setup_visuals(ctx);
}
