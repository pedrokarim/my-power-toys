use crate::config::{HostsEditorConfig, Placement};
use crate::gui::theme;
use crate::parser::HostsFile;
use eframe::egui;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Idle,
    Saved,
    Error(String),
}

struct HostsEditorApp {
    hosts: HostsFile,
    hosts_path: PathBuf,
    config: HostsEditorConfig,
    filter: String,
    // Add/Edit form
    editing: Option<usize>, // None = add mode, Some(idx) = edit mode
    form_ip: String,
    form_hostnames: String,
    form_comment: String,
    form_enabled: bool,
    form_error: Option<String>,
    form_open: bool,
    // State
    unsaved_changes: bool,
    status: Status,
    frame_count: u32,
    confirm_delete: Option<usize>,
}

impl HostsEditorApp {
    fn new(config: HostsEditorConfig, hosts: HostsFile, hosts_path: PathBuf) -> Self {
        Self {
            hosts,
            hosts_path,
            config,
            filter: String::new(),
            editing: None,
            form_ip: String::new(),
            form_hostnames: String::new(),
            form_comment: String::new(),
            form_enabled: true,
            form_error: None,
            form_open: false,
            unsaved_changes: false,
            status: Status::Idle,
            frame_count: 0,
            confirm_delete: None,
        }
    }

    /// Get filtered entry indices (into the entries-only list).
    fn filtered_indices(&self) -> Vec<usize> {
        let filter_lower = self.filter.to_lowercase();
        self.hosts
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if !self.config.show_disabled && !e.enabled {
                    return false;
                }
                if filter_lower.is_empty() {
                    return true;
                }
                e.ip.to_lowercase().contains(&filter_lower)
                    || e.hostnames
                        .iter()
                        .any(|h| h.to_lowercase().contains(&filter_lower))
                    || e.comment
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&filter_lower))
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn open_add_form(&mut self) {
        self.editing = None;
        self.form_ip.clear();
        self.form_hostnames.clear();
        self.form_comment.clear();
        self.form_enabled = true;
        self.form_error = None;
        self.form_open = true;
    }

    fn open_edit_form(&mut self, entry_index: usize) {
        let entries = self.hosts.entries();
        if let Some(entry) = entries.get(entry_index) {
            self.editing = Some(entry_index);
            self.form_ip = entry.ip.clone();
            self.form_hostnames = entry.hostnames.join(" ");
            self.form_comment = entry.comment.clone().unwrap_or_default();
            self.form_enabled = entry.enabled;
            self.form_error = None;
            self.form_open = true;
        }
    }

    fn close_form(&mut self) {
        self.form_open = false;
        self.editing = None;
        self.form_error = None;
    }

    fn validate_and_apply_form(&mut self) {
        let ip = self.form_ip.trim().to_string();
        if ip.is_empty() {
            self.form_error = Some("IP address is required".to_string());
            return;
        }

        let hostnames: Vec<String> = self
            .form_hostnames
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if hostnames.is_empty() {
            self.form_error = Some("At least one hostname is required".to_string());
            return;
        }

        let comment = if self.form_comment.trim().is_empty() {
            None
        } else {
            Some(self.form_comment.trim().to_string())
        };

        if let Some(idx) = self.editing {
            if let Err(e) = self.hosts.update_entry(idx, ip, hostnames, comment) {
                self.form_error = Some(format!("Failed to update: {e}"));
                return;
            }
            let current_enabled = self.hosts.entries()[idx].enabled;
            if current_enabled != self.form_enabled {
                let _ = self.hosts.toggle_entry(idx);
            }
        } else {
            match self.config.new_entry_placement {
                Placement::Top => self.hosts.add_entry_at_top(ip, hostnames, comment),
                Placement::Bottom => self.hosts.add_entry(ip, hostnames, comment),
            }
        }

        self.unsaved_changes = true;
        self.close_form();
    }

    fn save_file(&mut self) {
        if self.config.backup_before_save {
            let timestamp = chrono_timestamp();
            let backup_path = format!("/tmp/hosts.mpt-backup-{timestamp}");
            if let Err(e) = std::fs::copy(&self.hosts_path, &backup_path) {
                warn!("Backup failed: {e}");
            } else {
                info!("Backup saved to {backup_path}");
            }
        }

        let content = self.hosts.serialize();

        match std::fs::write(&self.hosts_path, &content) {
            Ok(()) => {
                self.unsaved_changes = false;
                self.status = Status::Saved;
                info!("Hosts file saved directly");
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                info!("Permission denied, trying pkexec...");
                let tmp = std::env::temp_dir().join("mpt-hosts-editor-tmp");
                if let Err(e) = std::fs::write(&tmp, &content) {
                    self.status = Status::Error(format!("Failed to write temp file: {e}"));
                    return;
                }

                let result = std::process::Command::new("pkexec")
                    .args([
                        "cp",
                        &tmp.to_string_lossy(),
                        &self.hosts_path.to_string_lossy(),
                    ])
                    .status();

                let _ = std::fs::remove_file(&tmp);

                match result {
                    Ok(status) if status.success() => {
                        self.unsaved_changes = false;
                        self.status = Status::Saved;
                        info!("Hosts file saved via pkexec");
                    }
                    Ok(status) => {
                        self.status = Status::Error(format!(
                            "pkexec exited with code {}",
                            status.code().unwrap_or(-1)
                        ));
                    }
                    Err(e) => {
                        self.status = Status::Error(format!("pkexec failed: {e}"));
                    }
                }
            }
            Err(e) => {
                self.status = Status::Error(format!("Save failed: {e}"));
            }
        }
    }

    // ── Drawing ──────────────────────────────────────────────────────────────

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Hosts Editor")
                        .size(theme::FONT_TITLE)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("Edit {}", self.hosts_path.display()))
                        .size(theme::FONT_SMALL)
                        .color(theme::TEXT_MUTED),
                );
            });
        });
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Filter")
                    .size(theme::FONT_BODY)
                    .color(theme::TEXT_SECONDARY),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .desired_width(300.0)
                    .hint_text("Search by IP, hostname or comment..."),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn = egui::Button::new(
                    egui::RichText::new("+ Add Entry")
                        .size(theme::FONT_BUTTON)
                        .color(theme::TEXT_PRIMARY),
                )
                .fill(theme::BG_BUTTON)
                .corner_radius(theme::BUTTON_RADIUS);
                if ui.add(btn).clicked() {
                    self.open_add_form();
                }
            });
        });
    }

    fn draw_column_headers(&self, ui: &mut egui::Ui) {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 0.0, theme::BG_SECONDARY);

        let font = egui::FontId::proportional(theme::FONT_SMALL);
        let col = theme::TEXT_MUTED;
        let y = rect.center().y;

        p.text(
            egui::pos2(rect.left() + 16.0, y),
            egui::Align2::CENTER_CENTER,
            "ON",
            font.clone(),
            col,
        );
        p.text(
            egui::pos2(rect.left() + 40.0, y),
            egui::Align2::LEFT_CENTER,
            "IP ADDRESS",
            font.clone(),
            col,
        );
        p.text(
            egui::pos2(rect.left() + 190.0, y),
            egui::Align2::LEFT_CENTER,
            "HOSTNAMES",
            font.clone(),
            col,
        );
        p.text(
            egui::pos2(rect.left() + 460.0, y),
            egui::Align2::LEFT_CENTER,
            "COMMENT",
            font.clone(),
            col,
        );
        p.text(
            egui::pos2(rect.right() - 20.0, y),
            egui::Align2::CENTER_CENTER,
            "DEL",
            font,
            col,
        );
    }

    fn draw_entry_table(&mut self, ui: &mut egui::Ui) {
        let indices = self.filtered_indices();

        // Collect actions to apply after drawing
        let mut toggle_idx: Option<usize> = None;
        let mut edit_idx: Option<usize> = None;
        let mut delete_idx: Option<usize> = None;
        let mut cancel_delete = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let entries = self.hosts.entries();
                let row_width = ui.available_width();

                for (row_i, &entry_idx) in indices.iter().enumerate() {
                    let entry = entries[entry_idx];
                    let text_color = if entry.enabled {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_MUTED
                    };

                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(row_width, theme::ENTRY_ROW_HEIGHT),
                        egui::Sense::click(),
                    );
                    let p = ui.painter_at(row_rect);

                    // Alternating background
                    if row_i % 2 == 1 {
                        p.rect_filled(row_rect, 0.0, theme::BG_SECONDARY);
                    }
                    // Hover highlight
                    if row_resp.hovered() {
                        p.rect_filled(row_rect, 0.0, theme::BG_HOVER);
                    }

                    // Row click → edit
                    if row_resp.clicked() {
                        edit_idx = Some(entry_idx);
                    }

                    let y = row_rect.center().y;

                    // ── Toggle circle (custom painted) ──
                    let toggle_center = egui::pos2(row_rect.left() + 16.0, y);
                    let toggle_rect =
                        egui::Rect::from_center_size(toggle_center, egui::vec2(20.0, 20.0));
                    let toggle_resp = ui.interact(
                        toggle_rect,
                        egui::Id::new(("toggle", entry_idx)),
                        egui::Sense::click(),
                    );
                    if entry.enabled {
                        p.circle_filled(toggle_center, 6.0, theme::TEXT_SUCCESS);
                    } else {
                        p.circle_stroke(
                            toggle_center,
                            6.0,
                            egui::Stroke::new(1.5, theme::TEXT_MUTED),
                        );
                    }
                    if toggle_resp.clicked() {
                        toggle_idx = Some(entry_idx);
                    }

                    // ── IP ──
                    p.text(
                        egui::pos2(row_rect.left() + 40.0, y),
                        egui::Align2::LEFT_CENTER,
                        &entry.ip,
                        egui::FontId::monospace(theme::FONT_BODY),
                        text_color,
                    );

                    // ── Hostnames ──
                    let hosts_str = entry.hostnames.join(" ");
                    p.text(
                        egui::pos2(row_rect.left() + 190.0, y),
                        egui::Align2::LEFT_CENTER,
                        &hosts_str,
                        egui::FontId::proportional(theme::FONT_BODY),
                        text_color,
                    );

                    // ── Comment ──
                    let comment = entry.comment.as_deref().unwrap_or("");
                    if !comment.is_empty() {
                        p.text(
                            egui::pos2(row_rect.left() + 460.0, y),
                            egui::Align2::LEFT_CENTER,
                            comment,
                            egui::FontId::proportional(theme::FONT_SMALL),
                            theme::TEXT_MUTED,
                        );
                    }

                    // ── Delete button ──
                    let del_center = egui::pos2(row_rect.right() - 20.0, y);
                    let del_rect = egui::Rect::from_center_size(del_center, egui::vec2(24.0, 24.0));

                    if self.confirm_delete == Some(entry_idx) {
                        // Confirm inline: "Delete?  [Yes] [No]"
                        p.text(
                            egui::pos2(row_rect.right() - 110.0, y),
                            egui::Align2::LEFT_CENTER,
                            "Delete?",
                            egui::FontId::proportional(theme::FONT_SMALL),
                            theme::TEXT_ERROR,
                        );

                        let yes_rect = egui::Rect::from_center_size(
                            egui::pos2(row_rect.right() - 50.0, y),
                            egui::vec2(30.0, 20.0),
                        );
                        let yes_resp = ui.interact(
                            yes_rect,
                            egui::Id::new(("del_yes", entry_idx)),
                            egui::Sense::click(),
                        );
                        let yes_col = if yes_resp.hovered() {
                            theme::TEXT_PRIMARY
                        } else {
                            theme::TEXT_ERROR
                        };
                        p.text(
                            yes_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Yes",
                            egui::FontId::proportional(theme::FONT_SMALL),
                            yes_col,
                        );
                        if yes_resp.clicked() {
                            delete_idx = Some(entry_idx);
                        }

                        let no_rect = egui::Rect::from_center_size(
                            egui::pos2(row_rect.right() - 20.0, y),
                            egui::vec2(30.0, 20.0),
                        );
                        let no_resp = ui.interact(
                            no_rect,
                            egui::Id::new(("del_no", entry_idx)),
                            egui::Sense::click(),
                        );
                        let no_col = if no_resp.hovered() {
                            theme::TEXT_PRIMARY
                        } else {
                            theme::TEXT_MUTED
                        };
                        p.text(
                            no_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "No",
                            egui::FontId::proportional(theme::FONT_SMALL),
                            no_col,
                        );
                        if no_resp.clicked() {
                            cancel_delete = true;
                        }
                    } else {
                        let del_resp = ui.interact(
                            del_rect,
                            egui::Id::new(("del_btn", entry_idx)),
                            egui::Sense::click(),
                        );
                        let del_col = if del_resp.hovered() {
                            theme::TEXT_ERROR
                        } else {
                            theme::TEXT_MUTED
                        };
                        p.text(
                            del_center,
                            egui::Align2::CENTER_CENTER,
                            "\u{2715}",
                            egui::FontId::proportional(theme::FONT_BODY),
                            del_col,
                        );
                        if del_resp.clicked() {
                            self.confirm_delete = Some(entry_idx);
                        }
                    }

                    // Separator line
                    p.line_segment(
                        [
                            egui::pos2(row_rect.left(), row_rect.bottom()),
                            egui::pos2(row_rect.right(), row_rect.bottom()),
                        ],
                        egui::Stroke::new(1.0, theme::SEPARATOR),
                    );
                }

                if indices.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No entries match the filter")
                                .size(theme::FONT_BODY)
                                .color(theme::TEXT_MUTED),
                        );
                    });
                }
            });

        // Apply deferred actions outside the borrow
        if let Some(idx) = toggle_idx {
            let _ = self.hosts.toggle_entry(idx);
            self.unsaved_changes = true;
        }
        if let Some(idx) = edit_idx {
            self.open_edit_form(idx);
        }
        if let Some(idx) = delete_idx {
            let _ = self.hosts.remove_entry(idx);
            self.unsaved_changes = true;
            self.confirm_delete = None;
        }
        if cancel_delete {
            self.confirm_delete = None;
        }
    }

    fn draw_form_inline(&mut self, ui: &mut egui::Ui, should_validate: &mut bool) {
        let title = match self.editing {
            Some(_) => "Edit Entry",
            None => "Add Entry",
        };

        egui::Frame::NONE
            .fill(theme::BG_CARD)
            .stroke(egui::Stroke::new(1.0, theme::CARD_BORDER))
            .corner_radius(theme::CARD_RADIUS)
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(theme::FONT_SECTION)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("IP:")
                            .size(theme::FONT_BODY)
                            .color(theme::TEXT_SECONDARY),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form_ip)
                            .desired_width(140.0)
                            .hint_text("e.g. 127.0.0.1"),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Hostnames:")
                            .size(theme::FONT_BODY)
                            .color(theme::TEXT_SECONDARY),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form_hostnames)
                            .desired_width(240.0)
                            .hint_text("space-separated"),
                    );
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Comment:")
                            .size(theme::FONT_BODY)
                            .color(theme::TEXT_SECONDARY),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form_comment)
                            .desired_width(200.0)
                            .hint_text("optional"),
                    );
                    ui.add_space(12.0);
                    ui.checkbox(&mut self.form_enabled, "Active");
                });

                if let Some(err) = &self.form_error {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(err)
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_ERROR),
                    );
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save Entry")
                                .size(theme::FONT_BUTTON)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .fill(theme::BG_BUTTON)
                        .corner_radius(theme::BUTTON_RADIUS);
                        if ui.add(save_btn).clicked() {
                            *should_validate = true;
                        }

                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("Cancel")
                                .size(theme::FONT_BUTTON)
                                .color(theme::TEXT_MUTED),
                        )
                        .fill(theme::BG_SECONDARY)
                        .corner_radius(theme::BUTTON_RADIUS);
                        if ui.add(cancel_btn).clicked() {
                            self.close_form();
                        }
                    });
                });
            });
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        let entries = self.hosts.entries();
        let total = entries.len();
        let active = entries.iter().filter(|e| e.enabled).count();
        let disabled = total - active;

        let footer_w = ui.available_width();
        let (footer_rect, _) =
            ui.allocate_exact_size(egui::vec2(footer_w, 36.0), egui::Sense::hover());

        let p = ui.painter_at(footer_rect);
        p.rect_filled(footer_rect, 0.0, theme::BG_SECONDARY);

        // Status text (left)
        let status_text = format!("{total} entries ({active} active, {disabled} disabled)");
        p.text(
            egui::pos2(footer_rect.left() + 12.0, footer_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &status_text,
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_MUTED,
        );

        // Status message (center)
        match &self.status {
            Status::Saved => {
                p.text(
                    footer_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Saved!",
                    egui::FontId::proportional(theme::FONT_SMALL),
                    theme::TEXT_SUCCESS,
                );
            }
            Status::Error(e) => {
                let msg = format!("Error: {e}");
                p.text(
                    footer_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &msg,
                    egui::FontId::proportional(theme::FONT_SMALL),
                    theme::TEXT_ERROR,
                );
            }
            Status::Idle => {}
        }

        // Save button (right)
        let save_label = if self.unsaved_changes {
            "Save File *"
        } else {
            "Save File"
        };
        let btn_w = 100.0;
        let btn_h = 28.0;
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                footer_rect.right() - btn_w - 8.0,
                footer_rect.center().y - btn_h / 2.0,
            ),
            egui::vec2(btn_w, btn_h),
        );
        let btn_resp = ui.interact(btn_rect, egui::Id::new("save_btn"), egui::Sense::click());
        let btn_bg = if self.unsaved_changes {
            if btn_resp.hovered() {
                theme::BG_BUTTON_HOVER
            } else {
                theme::BG_SUCCESS
            }
        } else if btn_resp.hovered() {
            theme::BG_BUTTON_HOVER
        } else {
            theme::BG_BUTTON
        };
        p.rect_filled(btn_rect, theme::BUTTON_RADIUS, btn_bg);
        p.text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            save_label,
            egui::FontId::proportional(theme::FONT_BUTTON),
            theme::TEXT_PRIMARY,
        );
        if btn_resp.clicked() {
            self.save_file();
        }
    }
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{year:04}{month:02}{day:02}{h:02}{m:02}{s:02}")
}

impl eframe::App for HostsEditorApp {
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

        let mut should_validate_form = false;

        // Header (top panel)
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_HEADER)
                    .inner_margin(egui::Margin::symmetric(theme::INNER_MARGIN as i8, 12)),
            )
            .show(ctx, |ui| {
                self.draw_header(ui);
            });

        // Footer (bottom panel)
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                self.draw_footer(ui);
            });

        // Form panel (bottom, above footer, if open)
        if self.form_open {
            egui::TopBottomPanel::bottom("form")
                .frame(
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(theme::INNER_MARGIN as i8, 6)),
                )
                .show(ctx, |ui| {
                    self.draw_form_inline(ui, &mut should_validate_form);
                });
        }

        // Central panel: toolbar + column headers + scrollable entries
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PRIMARY)
                    .inner_margin(egui::Margin::symmetric(
                        theme::INNER_MARGIN as i8,
                        theme::INNER_MARGIN as i8,
                    )),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                self.draw_toolbar(ui);
                ui.add_space(4.0);
                self.draw_column_headers(ui);
                ui.add_space(2.0);
                self.draw_entry_table(ui);
            });

        if should_validate_form {
            self.validate_and_apply_form();
        }
    }
}

pub fn run_window(config: HostsEditorConfig, hosts: HostsFile, hosts_path: PathBuf) {
    info!("Opening Hosts Editor window for {}", hosts_path.display());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Hosts Editor")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Hosts Editor",
        options,
        Box::new(|cc| {
            theme::setup_visuals(&cc.egui_ctx);
            Ok(Box::new(HostsEditorApp::new(config, hosts, hosts_path)))
        }),
    )
    .ok();
}
