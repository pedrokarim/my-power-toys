use egui::{self, RichText, Shadow};

use crate::CommandPaletteConfig;
use crate::gui::{icons, theme};
use crate::providers::{PaletteResult, ResultAction, ResultIcon, SystemCmd};
use crate::search::SearchEngine;

struct PaletteApp {
    query: String,
    results: Vec<PaletteResult>,
    selected_index: usize,
    engine: SearchEngine,
    icon_cache: icons::IconCache,
    should_close: bool,
    frame_count: u32,
    had_focus: bool,
}

impl PaletteApp {
    fn new(config: CommandPaletteConfig) -> Self {
        let mut engine = SearchEngine::new(config);
        engine.initialize();

        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            engine,
            icon_cache: icons::IconCache::new(),
            should_close: false,
            frame_count: 0,
            had_focus: false,
        }
    }

    fn draw_search_bar(&mut self, ui: &mut egui::Ui) {
        let frame =
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 12));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("\u{1F50D}")
                        .size(theme::FONT_ICON)
                        .color(theme::TEXT_HINT),
                );
                ui.add_space(8.0);

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .desired_width(ui.available_width())
                        .font(egui::FontId::proportional(theme::FONT_SEARCH))
                        .text_color(theme::TEXT_PRIMARY)
                        .frame(false)
                        .hint_text(
                            RichText::new("Search apps, commands, files...")
                                .color(theme::TEXT_HINT),
                        ),
                );

                response.request_focus();

                if response.changed() {
                    self.results = self.engine.search(&self.query);
                    self.selected_index = 0;
                }
            });
        });
    }

    fn draw_results(&mut self, ui: &mut egui::Ui) {
        let visible_count = self.results.len().min(theme::MAX_VISIBLE_RESULTS);
        let ctx = ui.ctx().clone();

        for i in 0..visible_count {
            let is_selected = i == self.selected_index;

            let bg_color = if is_selected {
                theme::BG_SELECTED
            } else {
                egui::Color32::TRANSPARENT
            };

            let frame = egui::Frame::NONE
                .fill(bg_color)
                .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 6));

            let resp = frame
                .show(ui, |ui| {
                    ui.set_min_height(theme::RESULT_ROW_HEIGHT - 12.0);
                    ui.horizontal(|ui| {
                        // Icon — try loading real app icon, fall back to text
                        let mut icon_drawn = false;
                        if let ResultIcon::Named(name) = &self.results[i].icon {
                            if let Some(source) = self.icon_cache.get(&ctx, name) {
                                ui.add(
                                    egui::Image::new(source)
                                        .fit_to_exact_size(icons::icon_size()),
                                );
                                icon_drawn = true;
                            }
                        }
                        if !icon_drawn {
                            ui.label(
                                RichText::new(icon_char(&self.results[i].icon))
                                    .size(theme::FONT_ICON)
                                    .color(theme::ACCENT),
                            );
                        }
                        ui.add_space(10.0);

                        // Title + subtitle
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&self.results[i].title)
                                    .size(theme::FONT_TITLE)
                                    .color(theme::TEXT_PRIMARY),
                            );
                            if let Some(ref sub) = self.results[i].subtitle {
                                ui.label(
                                    RichText::new(sub)
                                        .size(theme::FONT_SUBTITLE)
                                        .color(theme::TEXT_SECONDARY),
                                );
                            }
                        });

                        // Provider tag (right side)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(self.results[i].provider_tag)
                                    .size(theme::FONT_TAG)
                                    .color(theme::TEXT_TAG),
                            );
                        });
                    });
                })
                .response;

            // Handle click on result row
            if resp.interact(egui::Sense::click()).clicked() {
                self.selected_index = i;
                self.engine.record_activation(&self.results[i]);
                execute_action(&self.results[i].action);
                self.should_close = true;
            }
        }
    }
}

impl eframe::App for PaletteApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // fully transparent so rounded corners are visible
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count = self.frame_count.saturating_add(1);

        // Center on screen once monitor size is available
        if self.frame_count <= 5 {
            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                let x = (monitor.x - theme::WINDOW_WIDTH) / 2.0;
                let y = monitor.y * 0.25;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                    [x, y].into(),
                ));
            }
        }

        // Key handling
        let mut activate = false;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.should_close = true;
            }
            if i.key_pressed(egui::Key::ArrowDown) && !self.results.is_empty() {
                self.selected_index =
                    (self.selected_index + 1) % self.results.len().min(theme::MAX_VISIBLE_RESULTS);
            }
            if i.key_pressed(egui::Key::ArrowUp) && !self.results.is_empty() {
                let count = self.results.len().min(theme::MAX_VISIBLE_RESULTS);
                self.selected_index = (self.selected_index + count - 1) % count;
            }
            if i.key_pressed(egui::Key::Enter) && !self.results.is_empty() {
                activate = true;
            }
        });

        if activate && let Some(result) = self.results.get(self.selected_index) {
            self.engine.record_activation(result);
            execute_action(&result.action);
            self.should_close = true;
        }

        // Close on focus loss (only after window has gained focus at least once)
        let has_focus = ctx.input(|i| i.focused);
        if has_focus {
            self.had_focus = true;
        }
        if !has_focus && self.had_focus {
            self.should_close = true;
        }

        // Main panel — glassmorphism style
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_PRIMARY)
                    .corner_radius(theme::CORNER_RADIUS)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ctx, |ui| {
                self.draw_search_bar(ui);

                if !self.results.is_empty() {
                    // Thin separator
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(theme::INNER_PADDING, ui.cursor().min.y),
                            egui::vec2(theme::WINDOW_WIDTH - theme::INNER_PADDING * 2.0, 1.0),
                        ),
                        0.0,
                        theme::SEPARATOR,
                    );
                    ui.add_space(2.0);

                    self.draw_results(ui);
                } else if !self.query.is_empty() {
                    // Thin separator
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(theme::INNER_PADDING, ui.cursor().min.y),
                            egui::vec2(theme::WINDOW_WIDTH - theme::INNER_PADDING * 2.0, 1.0),
                        ),
                        0.0,
                        theme::SEPARATOR,
                    );
                    ui.add_space(2.0);

                    let frame = egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(theme::INNER_PADDING as i8, 12));
                    frame.show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new("No matching results")
                                    .size(theme::FONT_TITLE)
                                    .color(theme::TEXT_HINT),
                            );
                        });
                    });
                }
            });

        // Dynamic window resize
        let result_count = self.results.len().min(theme::MAX_VISIBLE_RESULTS);
        let no_results = self.results.is_empty() && !self.query.is_empty();
        let target_height = theme::SEARCH_BAR_HEIGHT
            + if result_count > 0 {
                4.0 + (result_count as f32 * theme::RESULT_ROW_HEIGHT) + 8.0
            } else if no_results {
                4.0 + theme::RESULT_ROW_HEIGHT + 8.0
            } else {
                0.0
            };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            theme::WINDOW_WIDTH,
            target_height,
        )));

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        ctx.request_repaint();
    }
}

pub fn run_palette(config: CommandPaletteConfig) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([theme::WINDOW_WIDTH, theme::SEARCH_BAR_HEIGHT])
            .with_min_inner_size([400.0, theme::SEARCH_BAR_HEIGHT])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_resizable(false),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Command Palette",
        native_options,
        Box::new(move |cc| {
            setup_visuals(&cc.egui_ctx);
            Ok(Box::new(PaletteApp::new(config)))
        }),
    );
}

fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    // Transparent panel so the Frame's rounded fill is the only visible background
    visuals.window_fill = egui::Color32::TRANSPARENT;
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.window_shadow = Shadow::NONE;
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
    ctx.set_visuals(visuals);
}

fn icon_char(icon: &ResultIcon) -> &'static str {
    match icon {
        ResultIcon::Named(_) => "\u{25A0}", // filled square fallback
        ResultIcon::Emoji(_) => "\u{25A0}",
        ResultIcon::BuiltinApp => "\u{25B6}",  // play triangle
        ResultIcon::BuiltinCalc => "\u{2261}", // identical to
        ResultIcon::BuiltinWeb => "\u{1F310}", // globe
        ResultIcon::BuiltinSystem => "\u{2699}", // gear
        ResultIcon::BuiltinFile => "\u{1F4C4}", // document
        ResultIcon::BuiltinTerminal => ">_",
        ResultIcon::BuiltinSettings => "\u{2692}", // hammer/wrench
    }
}

fn execute_action(action: &ResultAction) {
    match action {
        ResultAction::LaunchExec(cmd) => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((program, args)) = parts.split_first() {
                let _ = std::process::Command::new(program)
                    .args(args)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        }
        ResultAction::CopyToClipboard(text) => {
            let _ = copy_to_clipboard(text);
        }
        ResultAction::OpenUrl(url) => {
            if !url.is_empty() {
                let _ = std::process::Command::new("xdg-open")
                    .arg(url)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        }
        ResultAction::RunShell(cmd) => {
            if !cmd.is_empty() {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
                let _ = std::process::Command::new(&shell)
                    .args(["-c", cmd])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        }
        ResultAction::SystemCommand(cmd) => {
            let args: &[&str] = match cmd {
                SystemCmd::Lock => &["loginctl", "lock-session"],
                SystemCmd::Logout => &["loginctl", "terminate-user", ""],
                SystemCmd::Shutdown => &["systemctl", "poweroff"],
                SystemCmd::Reboot => &["systemctl", "reboot"],
                SystemCmd::Suspend => &["systemctl", "suspend"],
                SystemCmd::Hibernate => &["systemctl", "hibernate"],
            };
            if let Some((program, rest)) = args.split_first() {
                let _ = std::process::Command::new(program).args(rest).spawn();
            }
        }
        ResultAction::OpenSettings(cmd) => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some((program, args)) = parts.split_first() {
                let _ = std::process::Command::new(program)
                    .args(args)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        }
    }
}

fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    // Try wl-copy first (Wayland), then xclip (X11)
    let wayland = std::process::Command::new("wl-copy")
        .arg(text)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if wayland.is_ok_and(|s| s.success()) {
        return Ok(());
    }

    let x11 = std::process::Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn();

    if let Ok(mut child) = x11 {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }

    anyhow::bail!("No clipboard tool available (tried wl-copy, xclip)")
}
