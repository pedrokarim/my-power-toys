#[allow(dead_code)]
mod theme;
#[allow(dead_code)]
mod widgets;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, toggler};
use iced::{Alignment, Element, Length, Task};

/// Module info from the daemon.
#[derive(Debug, Clone)]
struct ModuleInfo {
    id: String,
    name: String,
    running: bool,
}

/// Currently selected sidebar page.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Dashboard,
    Module(String),
    About,
}

/// Application state.
struct Settings {
    modules: Vec<ModuleInfo>,
    page: Page,
    daemon_connected: bool,
}

/// All possible user interactions.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    NavigateTo(Page),
    ToggleModule(String, bool),
    RefreshModules,
    ModulesLoaded(Vec<ModuleInfo>),
    DaemonStatus(bool),
}

impl Settings {
    fn new() -> (Self, Task<Message>) {
        let modules = default_module_list();
        (
            Self {
                modules,
                page: Page::Dashboard,
                daemon_connected: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => {
                self.page = page;
            }
            Message::ToggleModule(id, enabled) => {
                if let Some(m) = self.modules.iter_mut().find(|m| m.id == id) {
                    m.running = enabled;
                }
            }
            Message::RefreshModules => {}
            Message::ModulesLoaded(modules) => {
                self.modules = modules;
                self.daemon_connected = true;
            }
            Message::DaemonStatus(connected) => {
                self.daemon_connected = connected;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_content();

        let layout = row![sidebar, content].height(Length::Fill);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let header = column![text("MyPowerToys").size(22), text("Settings").size(14),]
            .spacing(4)
            .padding(16);

        let nav_dashboard = sidebar_button(
            "Dashboard".to_string(),
            self.page == Page::Dashboard,
            Message::NavigateTo(Page::Dashboard),
        );

        let mut nav_items = column![nav_dashboard].spacing(2);

        for module in &self.modules {
            let is_selected = self.page == Page::Module(module.id.clone());
            let indicator = if module.running { "●" } else { "○" };
            let label = format!("{indicator} {}", module.name);
            nav_items = nav_items.push(sidebar_button(
                label,
                is_selected,
                Message::NavigateTo(Page::Module(module.id.clone())),
            ));
        }

        let nav_about = sidebar_button(
            "About".to_string(),
            self.page == Page::About,
            Message::NavigateTo(Page::About),
        );
        nav_items = nav_items.push(horizontal_rule(1)).push(nav_about);

        let sidebar_content = column![header, horizontal_rule(1), scrollable(nav_items.padding(8))]
            .spacing(0)
            .height(Length::Fill);

        container(sidebar_content)
            .width(220)
            .height(Length::Fill)
            .into()
    }

    fn view_content(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match &self.page {
            Page::Dashboard => self.view_dashboard(),
            Page::Module(id) => self.view_module(id),
            Page::About => self.view_about(),
        };

        container(scrollable(
            container(content).padding(24).width(Length::Fill),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_dashboard(&self) -> Element<'_, Message> {
        let status_msg = if self.daemon_connected {
            "Daemon: connected"
        } else {
            "Daemon: not connected (start mpt-daemon)"
        };

        let running = self.modules.iter().filter(|m| m.running).count();
        let total = self.modules.len();

        let mut module_list = column![].spacing(8);
        for module in &self.modules {
            let module_row = row![
                toggler(module.running)
                    .on_toggle({
                        let id = module.id.clone();
                        move |v| Message::ToggleModule(id.clone(), v)
                    })
                    .label(&module.name),
            ]
            .spacing(12)
            .align_y(Alignment::Center);
            module_list = module_list.push(module_row);
        }

        column![
            text("Dashboard").size(28),
            text(status_msg).size(14),
            text(format!("{running}/{total} modules active")).size(16),
            horizontal_rule(1),
            text("Modules").size(20),
            module_list,
        ]
        .spacing(16)
        .into()
    }

    fn view_module(&self, id: &str) -> Element<'_, Message> {
        let module = self.modules.iter().find(|m| m.id == id);

        match module {
            Some(m) => {
                let status_text = if m.running { "Running" } else { "Stopped" };

                column![
                    text(&m.name).size(28),
                    row![text("Status: ").size(16), text(status_text).size(16),],
                    horizontal_rule(1),
                    toggler(m.running).label("Enabled").on_toggle({
                        let id = m.id.clone();
                        move |v| Message::ToggleModule(id.clone(), v)
                    }),
                    horizontal_rule(1),
                    text("Module-specific settings will appear here.").size(14),
                ]
                .spacing(16)
                .into()
            }
            None => text("Module not found").size(20).into(),
        }
    }

    fn view_about(&self) -> Element<'_, Message> {
        column![
            text("MyPowerToys").size(28),
            text("Version 0.1.0").size(16),
            horizontal_rule(1),
            text("A suite of system utilities for Linux, inspired by Microsoft PowerToys.")
                .size(14),
            text("Written in Rust with iced for the UI.").size(14),
            horizontal_rule(1),
            text("License: GPL-3.0").size(14),
            text("https://github.com/pedrokarim/my-power-toys").size(14),
        ]
        .spacing(12)
        .into()
    }
}

fn sidebar_button(label: String, selected: bool, message: Message) -> Element<'static, Message> {
    let btn = button(text(label).size(14))
        .on_press(message)
        .width(Length::Fill);
    if selected {
        btn.style(button::primary).into()
    } else {
        btn.style(button::text).into()
    }
}

/// Default module list (offline mode, before daemon connection).
fn default_module_list() -> Vec<ModuleInfo> {
    vec![
        ModuleInfo {
            id: "always-on-top".into(),
            name: "Always on Top".into(),
            running: false,
        },
        ModuleInfo {
            id: "awake".into(),
            name: "Awake".into(),
            running: false,
        },
        ModuleInfo {
            id: "paste-plain".into(),
            name: "Paste as Plain Text".into(),
            running: false,
        },
        ModuleInfo {
            id: "color-picker".into(),
            name: "Color Picker".into(),
            running: false,
        },
        ModuleInfo {
            id: "hosts-editor".into(),
            name: "Hosts Editor".into(),
            running: false,
        },
        ModuleInfo {
            id: "bulk-rename".into(),
            name: "Bulk Rename".into(),
            running: false,
        },
        ModuleInfo {
            id: "image-resizer".into(),
            name: "Image Resizer".into(),
            running: false,
        },
        ModuleInfo {
            id: "key-manager".into(),
            name: "Key Manager".into(),
            running: false,
        },
        ModuleInfo {
            id: "mouse-utils".into(),
            name: "Mouse Utilities".into(),
            running: false,
        },
        ModuleInfo {
            id: "screen-ruler".into(),
            name: "Screen Ruler".into(),
            running: false,
        },
        ModuleInfo {
            id: "text-extractor".into(),
            name: "Text Extractor".into(),
            running: false,
        },
        ModuleInfo {
            id: "shortcut-guide".into(),
            name: "Shortcut Guide".into(),
            running: false,
        },
        ModuleInfo {
            id: "app-launcher".into(),
            name: "App Launcher".into(),
            running: false,
        },
        ModuleInfo {
            id: "fancy-zones".into(),
            name: "FancyZones".into(),
            running: false,
        },
        ModuleInfo {
            id: "peek".into(),
            name: "Peek".into(),
            running: false,
        },
    ]
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter("mpt_ui=debug")
        .init();

    iced::application("MyPowerToys Settings", Settings::update, Settings::view)
        .window_size((900.0, 650.0))
        .run_with(Settings::new)
}
