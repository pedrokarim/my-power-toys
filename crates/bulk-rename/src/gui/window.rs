use crate::config::{ApplyTo, BulkRenameConfig, TextFormatting};
use crate::gui::theme;
use crate::{ListOptions, RenamePreview, Renamer};
use eframe::egui;
use std::path::PathBuf;
use tracing::info;

// ── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Idle,
    Done { count: usize },
    Error(String),
}

// ── App state ───────────────────────────────────────────────────────────────

struct BulkRenameApp {
    files: Vec<PathBuf>,
    included: Vec<bool>,
    search: String,
    replace: String,
    // Options (initialized from config)
    use_regex: bool,
    match_all: bool,
    case_sensitive: bool,
    apply_to: ApplyTo,
    text_formatting: TextFormatting,
    enumerate: bool,
    include_folders: bool,
    include_subfolders: bool,
    // Preview
    renamer: Renamer,
    previews: Vec<RenamePreview>,
    preview_error: Option<String>,
    // Cache for change detection
    last_search: String,
    last_replace: String,
    options_dirty: bool,
    // Status
    status: Status,
    source_dir: Option<PathBuf>,
    // UI state
    frame_count: u32,
    help_open: bool,
}

impl BulkRenameApp {
    fn new(config: BulkRenameConfig, initial_files: Vec<PathBuf>) -> Self {
        let included = vec![true; initial_files.len()];
        let source_dir = initial_files
            .first()
            .and_then(|f| f.parent().map(|p| p.to_path_buf()));

        Self {
            files: initial_files,
            included,
            search: String::new(),
            replace: String::new(),
            use_regex: config.use_regex,
            match_all: config.match_all_occurrences,
            case_sensitive: config.case_sensitive,
            apply_to: config.apply_to,
            text_formatting: config.text_formatting,
            enumerate: config.enumerate_items,
            include_folders: config.include_folders,
            include_subfolders: config.include_subfolders,
            renamer: Renamer::new(),
            previews: Vec::new(),
            preview_error: None,
            last_search: String::new(),
            last_replace: String::new(),
            options_dirty: true,
            status: Status::Idle,
            source_dir,
            frame_count: 0,
            help_open: false,
        }
    }

    // ── Preview logic ──────────────────────────────────────────────────────

    fn refresh_preview(&mut self) {
        self.preview_error = None;

        if self.search.is_empty() && self.text_formatting == TextFormatting::None && !self.enumerate
        {
            self.previews = self
                .files
                .iter()
                .map(|f| RenamePreview {
                    original: f.clone(),
                    renamed: f.clone(),
                    changed: false,
                })
                .collect();
            return;
        }

        let opts = crate::RenameOptions {
            search: self.search.clone(),
            replace: self.replace.clone(),
            use_regex: self.use_regex,
            match_all: self.match_all,
            case_sensitive: self.case_sensitive,
            apply_to: self.apply_to,
            text_formatting: self.text_formatting,
            enumerate: self.enumerate,
        };

        match self.renamer.preview(&self.files, &opts) {
            Ok(previews) => self.previews = previews,
            Err(e) => {
                self.preview_error = Some(e.to_string());
                self.previews = self
                    .files
                    .iter()
                    .map(|f| RenamePreview {
                        original: f.clone(),
                        renamed: f.clone(),
                        changed: false,
                    })
                    .collect();
            }
        }
    }

    fn needs_refresh(&self) -> bool {
        self.search != self.last_search || self.replace != self.last_replace || self.options_dirty
    }

    fn mark_refreshed(&mut self) {
        self.last_search = self.search.clone();
        self.last_replace = self.replace.clone();
        self.options_dirty = false;
    }

    // ── Actions ────────────────────────────────────────────────────────────

    fn apply_rename(&mut self) {
        // Build filtered previews (only included files)
        let filtered: Vec<RenamePreview> = self
            .previews
            .iter()
            .enumerate()
            .filter(|(i, _)| self.included.get(*i).copied().unwrap_or(true))
            .map(|(_, p)| p.clone())
            .collect();

        let count = filtered.iter().filter(|p| p.changed).count();

        match self.renamer.execute(&filtered) {
            Ok(_op) => {
                // Update file paths to reflect renames
                for (i, preview) in self.previews.iter().enumerate() {
                    if preview.changed && self.included.get(i).copied().unwrap_or(true) {
                        self.files[i] = preview.renamed.clone();
                    }
                }
                self.status = Status::Done { count };
                // Refresh preview with new filenames
                self.options_dirty = true;
            }
            Err(e) => {
                self.status = Status::Error(e.to_string());
            }
        }
    }

    fn undo_rename(&mut self) {
        match self.renamer.undo() {
            Ok(()) => {
                // Re-list files from source dir
                if let Some(dir) = &self.source_dir {
                    let list_opts = ListOptions {
                        include_folders: self.include_folders,
                        include_subfolders: self.include_subfolders,
                    };
                    if let Ok(files) = Renamer::list_entries(dir, &list_opts) {
                        self.included = vec![true; files.len()];
                        self.files = files;
                    }
                }
                self.status = Status::Idle;
                self.options_dirty = true;
            }
            Err(e) => {
                self.status = Status::Error(e.to_string());
            }
        }
    }

    fn select_folder(&mut self) {
        let dialog = rfd::FileDialog::new().set_title("Select folder to rename files");
        if let Some(dir) = dialog.pick_folder() {
            let list_opts = ListOptions {
                include_folders: self.include_folders,
                include_subfolders: self.include_subfolders,
            };
            match Renamer::list_entries(&dir, &list_opts) {
                Ok(files) => {
                    self.included = vec![true; files.len()];
                    self.files = files;
                    self.source_dir = Some(dir);
                    self.status = Status::Idle;
                    self.options_dirty = true;
                }
                Err(e) => {
                    self.status = Status::Error(e.to_string());
                }
            }
        }
    }

    // ── Drawing helpers ────────────────────────────────────────────────────

    fn draw_header(&self, ui: &mut egui::Ui) {
        let frame = egui::Frame::NONE
            .fill(theme::BG_HEADER)
            .inner_margin(egui::Margin::symmetric(theme::INNER_MARGIN as i8, 12));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Pencil/rename icon
                let icon_size = 26.0;
                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(icon_size, icon_size), egui::Sense::hover());
                let p = ui.painter_at(icon_rect);

                // Draw a pencil icon
                let tip = egui::pos2(icon_rect.left() + 3.0, icon_rect.bottom() - 3.0);
                let mid = egui::pos2(icon_rect.left() + 10.0, icon_rect.bottom() - 10.0);
                let top = egui::pos2(icon_rect.right() - 3.0, icon_rect.top() + 3.0);
                p.line_segment([tip, top], egui::Stroke::new(2.0, theme::TEXT_ACCENT));
                // Pencil body sides
                p.line_segment(
                    [
                        egui::pos2(mid.x - 3.0, mid.y + 3.0),
                        egui::pos2(top.x - 3.0, top.y + 3.0),
                    ],
                    egui::Stroke::new(1.2, theme::TEXT_ACCENT),
                );
                p.line_segment(
                    [
                        egui::pos2(mid.x + 3.0, mid.y - 3.0),
                        egui::pos2(top.x + 3.0, top.y - 3.0),
                    ],
                    egui::Stroke::new(1.2, theme::TEXT_ACCENT),
                );

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Bulk Rename")
                            .size(theme::FONT_TITLE)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("Rename files with patterns, preview and undo")
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn draw_search_replace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let half_width = (ui.available_width() - 16.0) / 2.0;

            ui.vertical(|ui| {
                ui.set_width(half_width);
                ui.label(
                    egui::RichText::new("Search")
                        .size(theme::FONT_SECTION)
                        .strong()
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .desired_width(half_width)
                        .hint_text("Pattern to find...")
                        .font(egui::FontId::monospace(theme::FONT_BODY)),
                );
                if let Some(err) = &self.preview_error {
                    ui.label(
                        egui::RichText::new(err)
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_ERROR),
                    );
                }
            });

            ui.add_space(16.0);

            ui.vertical(|ui| {
                ui.set_width(half_width);
                ui.label(
                    egui::RichText::new("Replace with")
                        .size(theme::FONT_SECTION)
                        .strong()
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.replace)
                        .desired_width(half_width)
                        .hint_text("Replacement...")
                        .font(egui::FontId::monospace(theme::FONT_BODY)),
                );
            });
        });
    }

    fn draw_options(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            let old_regex = self.use_regex;
            let old_match = self.match_all;
            let old_case = self.case_sensitive;
            let old_enum = self.enumerate;

            Self::draw_toggle_chip(ui, "\u{2731} Regex", &mut self.use_regex);
            Self::draw_toggle_chip(ui, "Match all", &mut self.match_all);
            Self::draw_toggle_chip(ui, "Aa Case", &mut self.case_sensitive);
            Self::draw_toggle_chip(ui, "# Enumerate", &mut self.enumerate);

            if self.use_regex != old_regex
                || self.match_all != old_match
                || self.case_sensitive != old_case
                || self.enumerate != old_enum
            {
                self.options_dirty = true;
            }
        });
    }

    fn draw_toggle_chip(ui: &mut egui::Ui, label: &str, value: &mut bool) {
        let selected = *value;
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

        let text_width = ui
            .painter()
            .layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(theme::FONT_CHIP),
                text_color,
            )
            .size()
            .x;

        let (chip_rect, response) = ui.allocate_exact_size(
            egui::vec2(text_width + 20.0, theme::CHIP_HEIGHT),
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
            label,
            egui::FontId::proportional(theme::FONT_CHIP),
            text_color,
        );

        if response.clicked() {
            *value = !*value;
        }
    }

    fn draw_segment_selectors(&mut self, ui: &mut egui::Ui) {
        let old_apply = self.apply_to;
        let old_fmt = self.text_formatting;

        ui.horizontal(|ui| {
            let half_width = (ui.available_width() - 16.0) / 2.0;

            ui.vertical(|ui| {
                ui.set_width(half_width);
                Self::draw_section_label(ui, "APPLY TO");
                ui.add_space(4.0);
                Self::draw_chip_selector(
                    ui,
                    &mut self.apply_to,
                    &[
                        (ApplyTo::FilenameOnly, "Name"),
                        (ApplyTo::ExtensionOnly, "Ext"),
                        (ApplyTo::FilenameAndExtension, "Name+Ext"),
                    ],
                );
            });

            ui.add_space(16.0);

            ui.vertical(|ui| {
                ui.set_width(half_width);
                Self::draw_section_label(ui, "TEXT FORMATTING");
                ui.add_space(4.0);
                Self::draw_chip_selector(
                    ui,
                    &mut self.text_formatting,
                    &[
                        (TextFormatting::None, "\u{2014}"),
                        (TextFormatting::Lowercase, "aa"),
                        (TextFormatting::Uppercase, "AA"),
                        (TextFormatting::TitleCase, "Aa"),
                        (TextFormatting::CapitalizeEachWord, "Aa Aa"),
                    ],
                );
            });
        });

        if self.apply_to != old_apply || self.text_formatting != old_fmt {
            self.options_dirty = true;
        }
    }

    fn draw_file_table(&mut self, ui: &mut egui::Ui) {
        let changed_count = self
            .previews
            .iter()
            .enumerate()
            .filter(|(i, p)| p.changed && self.included.get(*i).copied().unwrap_or(true))
            .count();

        // Table header
        ui.horizontal(|ui| {
            let half_width = (ui.available_width() - 8.0) / 2.0;

            ui.vertical(|ui| {
                ui.set_width(half_width);
                Self::draw_section_label(ui, &format!("ORIGINAL ({})", self.files.len()));
            });

            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.set_width(half_width);
                Self::draw_section_label(ui, "RENAMED");
            });
        });

        ui.add_space(4.0);

        // Separator
        let (sep_rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
        ui.painter_at(sep_rect)
            .rect_filled(sep_rect, 0.0, theme::SEPARATOR);

        ui.add_space(4.0);

        if self.files.is_empty() {
            // Empty state — show folder picker button
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No files selected")
                        .size(theme::FONT_BODY)
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(12.0);

                let (btn_rect, btn_resp) =
                    ui.allocate_exact_size(egui::vec2(160.0, 36.0), egui::Sense::click());
                let p = ui.painter_at(btn_rect);
                let bg = if btn_resp.hovered() {
                    theme::BG_BUTTON_HOVER
                } else {
                    theme::BG_BUTTON
                };
                p.rect_filled(btn_rect, theme::BUTTON_RADIUS, bg);
                p.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "\u{1F4C2}  Select folder",
                    egui::FontId::proportional(theme::FONT_BUTTON),
                    theme::TEXT_PRIMARY,
                );
                if btn_resp.clicked() {
                    self.select_folder();
                }
            });
            return;
        }

        // File rows in scroll area
        let available_height = ui.available_height() - 52.0; // Leave room for footer
        egui::ScrollArea::vertical()
            .max_height(available_height.max(100.0))
            .show(ui, |ui| {
                let row_width = ui.available_width();
                let half_width = (row_width - 8.0) / 2.0;

                for (i, preview) in self.previews.clone().iter().enumerate() {
                    let included = self.included.get(i).copied().unwrap_or(true);

                    let (row_rect, _) = ui.allocate_exact_size(
                        egui::vec2(row_width, theme::FILE_ROW_HEIGHT),
                        egui::Sense::hover(),
                    );

                    let p = ui.painter_at(row_rect);

                    // Alternating row background
                    if i % 2 == 1 {
                        p.rect_filled(row_rect, 0.0, theme::BG_SECONDARY);
                    }

                    // Checkbox
                    let cb_rect = egui::Rect::from_min_size(
                        egui::pos2(row_rect.left() + 4.0, row_rect.top() + 3.0),
                        egui::vec2(20.0, 20.0),
                    );
                    let cb_resp =
                        ui.interact(cb_rect, egui::Id::new(("file_cb", i)), egui::Sense::click());

                    let cb_inner = cb_rect.shrink(3.0);
                    p.rect_stroke(
                        cb_inner,
                        2.0,
                        egui::Stroke::new(
                            1.0,
                            if included {
                                theme::TEXT_ACCENT
                            } else {
                                theme::TEXT_MUTED
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    if included {
                        p.rect_filled(cb_inner.shrink(2.0), 1.0, theme::TEXT_ACCENT);
                    }
                    if cb_resp.clicked()
                        && let Some(v) = self.included.get_mut(i)
                    {
                        *v = !*v;
                    }

                    // Original filename
                    let orig_name = preview
                        .original
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?");

                    let text_color = if !included {
                        theme::TEXT_MUTED
                    } else if preview.changed {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_MUTED
                    };

                    p.text(
                        egui::pos2(row_rect.left() + 28.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        orig_name,
                        egui::FontId::proportional(theme::FONT_SMALL),
                        text_color,
                    );

                    // Renamed filename
                    let new_name = preview
                        .renamed
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?");

                    let rename_color = if !included {
                        theme::TEXT_MUTED
                    } else if preview.changed {
                        theme::TEXT_SUCCESS
                    } else {
                        theme::TEXT_MUTED
                    };

                    p.text(
                        egui::pos2(row_rect.left() + half_width + 8.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        new_name,
                        egui::FontId::proportional(theme::FONT_SMALL),
                        rename_color,
                    );
                }
            });

        // Footer
        ui.add_space(4.0);

        let (sep_rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
        ui.painter_at(sep_rect)
            .rect_filled(sep_rect, 0.0, theme::SEPARATOR);

        ui.add_space(6.0);

        // Footer: [Undo]  status text  [Apply]
        let footer_width = ui.available_width() - 4.0; // small right breathing room
        let btn_w = 90.0;
        let btn_h = 32.0;
        let (footer_rect, _) =
            ui.allocate_exact_size(egui::vec2(footer_width, btn_h), egui::Sense::hover());

        let p = ui.painter_at(footer_rect);

        // Undo button (left)
        let has_undo = matches!(self.status, Status::Done { .. });
        let undo_rect = egui::Rect::from_min_size(footer_rect.left_top(), egui::vec2(btn_w, btn_h));
        let undo_resp = ui.interact(undo_rect, egui::Id::new("undo_btn"), egui::Sense::click());
        let undo_bg = if has_undo && undo_resp.hovered() {
            theme::BG_HOVER
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(undo_rect, 6.0, undo_bg);
        p.text(
            undo_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{21BB}  Undo",
            egui::FontId::proportional(theme::FONT_BUTTON),
            if has_undo {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            },
        );
        if has_undo && undo_resp.clicked() {
            self.undo_rename();
        }

        // Apply button (right)
        let has_changes = changed_count > 0;
        let apply_rect = egui::Rect::from_min_size(
            egui::pos2(footer_rect.right() - btn_w, footer_rect.top()),
            egui::vec2(btn_w, btn_h),
        );
        let apply_resp = ui.interact(apply_rect, egui::Id::new("apply_btn"), egui::Sense::click());
        let apply_bg = if has_changes && apply_resp.hovered() {
            theme::BG_BUTTON_HOVER
        } else if has_changes {
            theme::BG_BUTTON
        } else {
            theme::BG_CHIP
        };
        p.rect_filled(apply_rect, 6.0, apply_bg);
        p.text(
            apply_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{25B6}  Apply",
            egui::FontId::proportional(theme::FONT_BUTTON),
            if has_changes {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_MUTED
            },
        );
        if has_changes && apply_resp.clicked() {
            self.apply_rename();
        }

        // Status text (centered between buttons)
        let status_text = match &self.status {
            Status::Idle => {
                format!(
                    "{} items, {} will be renamed",
                    self.files.len(),
                    changed_count
                )
            }
            Status::Done { count } => {
                format!("\u{2713} {count} files renamed")
            }
            Status::Error(e) => format!("\u{2715} {e}"),
        };
        let status_color = match &self.status {
            Status::Idle => theme::TEXT_MUTED,
            Status::Done { .. } => theme::TEXT_SUCCESS,
            Status::Error(_) => theme::TEXT_ERROR,
        };
        let status_center = egui::pos2(
            (undo_rect.right() + apply_rect.left()) / 2.0,
            footer_rect.center().y,
        );
        p.text(
            status_center,
            egui::Align2::CENTER_CENTER,
            status_text,
            egui::FontId::proportional(theme::FONT_SMALL),
            status_color,
        );
    }

    // ── Collapsible help section (inline, fixed-height scroll) ──────────

    fn draw_help_section(&mut self, ui: &mut egui::Ui) {
        // Toggle bar
        let (toggle_rect, toggle_resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), egui::Sense::click());

        let p = ui.painter_at(toggle_rect);
        if toggle_resp.hovered() {
            p.rect_filled(toggle_rect, 4.0, theme::BG_HOVER);
        }

        let arrow = if self.help_open {
            "\u{25BE}"
        } else {
            "\u{25B8}"
        };
        p.text(
            egui::pos2(toggle_rect.left() + 4.0, toggle_rect.center().y),
            egui::Align2::LEFT_CENTER,
            format!("{arrow}  Quick Reference"),
            egui::FontId::proportional(theme::FONT_SMALL),
            theme::TEXT_ACCENT,
        );

        if toggle_resp.clicked() {
            self.help_open = !self.help_open;
        }

        if !self.help_open {
            return;
        }

        // Help card with fixed max height and internal scroll
        let frame = egui::Frame::NONE
            .fill(theme::BG_CARD)
            .corner_radius(theme::CARD_RADIUS)
            .inner_margin(egui::Margin::same(12));

        frame.show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;

                    let mono = |s: &str| {
                        egui::RichText::new(s)
                            .size(theme::FONT_SMALL)
                            .family(egui::FontFamily::Monospace)
                            .color(theme::TEXT_ACCENT)
                    };
                    let desc = |s: &str| {
                        egui::RichText::new(s)
                            .size(theme::FONT_SMALL)
                            .color(theme::TEXT_MUTED)
                    };
                    let heading = |s: &str| {
                        egui::RichText::new(s)
                            .size(theme::FONT_BODY)
                            .strong()
                            .color(theme::TEXT_SECONDARY)
                    };

                    // ── Regex patterns ──
                    ui.label(heading("Regex Patterns"));
                    ui.add_space(2.0);
                    for (pattern, explanation) in [
                        ("^foo", "Match beginning with \"foo\""),
                        ("bar$", "Match ending with \"bar\""),
                        (".*", "Match all text"),
                        (".+?(?=bar)", "Match everything up to \"bar\""),
                        ("foo[\\s\\S]*bar", "Everything between \"foo\" and \"bar\""),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(mono(pattern));
                            ui.label(desc(explanation));
                        });
                    }

                    ui.add_space(6.0);

                    // ── Capture groups ──
                    ui.label(heading("Capture Groups  ($1, $2...)"));
                    ui.add_space(2.0);
                    for (search, replace, explanation) in [
                        ("(.*)\\.png", "foo_$1.png", "Prepend \"foo_\" to PNGs"),
                        ("(.*)\\.png", "$1_foo.png", "Append \"_foo\" to PNGs"),
                        (
                            "(\\d{2})-(\\d{2})-(\\d{4})",
                            "$3-$2-$1",
                            "Reorder date parts",
                        ),
                        ("^(.{3})(.*)", "$1_NEW_$2", "Insert after 3rd char"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(mono(search));
                            ui.label(desc("\u{2192}"));
                            ui.label(mono(replace));
                            ui.label(desc(explanation));
                        });
                    }

                    ui.add_space(6.0);

                    // ── Enumerate ──
                    ui.label(heading("Enumerate  (enable # Enumerate)"));
                    ui.add_space(2.0);
                    for (pattern, explanation) in [
                        ("${}", "Counter starting at 0"),
                        ("${start=10}", "Start at 10"),
                        ("${increment=5}", "Step by 5"),
                        ("${padding=4}", "Zero-pad: 0001, 0002..."),
                        (
                            "${start=1;increment=2;padding=3}",
                            "Combined: 001, 003, 005...",
                        ),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(mono(pattern));
                            ui.label(desc(explanation));
                        });
                    }

                    ui.add_space(6.0);

                    // ── Text formatting ──
                    ui.label(heading("Text Formatting"));
                    ui.add_space(2.0);
                    for (chip, explanation) in [
                        ("aa", "all lowercase"),
                        ("AA", "ALL UPPERCASE"),
                        ("Aa", "Title case (first letter)"),
                        ("Aa Aa", "Capitalize Each Word"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(mono(chip));
                            ui.label(desc(explanation));
                        });
                    }
                });
        });
    }

    // ── Shared drawing utils (same pattern as image-resizer) ───────────────

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
}

// ── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for BulkRenameApp {
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

        // Refresh preview when inputs changed
        if self.needs_refresh() {
            self.refresh_preview();
            self.mark_refreshed();
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

                        // ── Search / Replace ────────────────────────
                        self.draw_search_replace(ui);

                        ui.add_space(4.0);

                        // ── Options row ─────────────────────────────
                        self.draw_options(ui);

                        ui.add_space(4.0);

                        // ── Apply To + Text Formatting segments ─────
                        self.draw_segment_selectors(ui);

                        ui.add_space(4.0);

                        // ── Collapsible help section ────────────────
                        self.draw_help_section(ui);

                        ui.add_space(4.0);

                        // ── File table ──────────────────────────────
                        self.draw_file_table(ui);
                    });
            });
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub fn run_window(config: BulkRenameConfig, initial_files: Vec<PathBuf>) {
    info!(
        "Opening Bulk Rename window with {} files",
        initial_files.len()
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::WINDOW_HEIGHT])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Bulk Rename")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Bulk Rename",
        options,
        Box::new(|cc| {
            theme::setup_visuals(&cc.egui_ctx);
            Ok(Box::new(BulkRenameApp::new(config, initial_files)))
        }),
    )
    .ok();
}
