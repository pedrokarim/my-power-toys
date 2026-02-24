use iced::widget::{button, column, container, image, row, text, toggler};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::BOOTSTRAP_FONT;
use iced_fonts::bootstrap::Bootstrap;

const SUPER_KEY_ICON: &[u8] = include_bytes!("../../../assets/super_key.png");

use crate::message::Message;
use crate::theme;
use crate::types::{Ui, bold};

pub fn card<'a>(content: impl Into<Element<'a, Message>>, ui: Ui) -> Element<'a, Message> {
    container(content)
        .padding(ui.pad(16.0) as u16)
        .width(Length::Fill)
        .style(theme::card(ui.contrast, ui.glass))
        .into()
}

pub fn stat_card<'a>(label: &str, value: &str, accent: Color, ui: Ui) -> Element<'a, Message> {
    container(
        column![
            text(value.to_string())
                .size(ui.sz(32.0))
                .font(bold())
                .color(accent),
            text(label.to_string())
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .padding(ui.pad(16.0) as u16)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .style(theme::stat_card(ui.contrast, ui.glass))
    .into()
}

pub fn kbd<'a>(key: &str, ui: Ui) -> Element<'a, Message> {
    let parts: Vec<&str> = key.split('+').collect();
    let mut r = row![].spacing(4).align_y(Alignment::Center);
    for part in parts {
        let badge: Element<'a, Message> = if part.trim() == "Super" {
            let icon_size = ui.sz(14.0);
            let handle = image::Handle::from_bytes(SUPER_KEY_ICON);
            container(image(handle).width(icon_size).height(icon_size))
                .padding(Padding::from([6.0, 10.0]))
                .style(theme::kbd_key())
                .into()
        } else {
            container(
                text(part.trim().to_string())
                    .size(ui.sz(12.0))
                    .font(ui.font())
                    .color(Color::WHITE),
            )
            .padding(Padding::from([6.0, 12.0]))
            .style(theme::kbd_key())
            .into()
        };
        r = r.push(badge);
    }
    r.into()
}

pub fn icon_badge<'a>(icon: Bootstrap, accent: Color, size: f32) -> Element<'a, Message> {
    let box_size = size * 1.7;
    container(text(icon.to_string()).font(BOOTSTRAP_FONT).size(size))
        .width(box_size)
        .height(box_size)
        .center_x(box_size)
        .center_y(box_size)
        .style(theme::icon_badge(accent))
        .into()
}

pub fn sidebar_icon_button(
    icon: Bootstrap,
    label: &str,
    selected: bool,
    msg: Message,
    ui: Ui,
) -> Element<'static, Message> {
    let content = row![
        text(icon.to_string())
            .font(BOOTSTRAP_FONT)
            .size(ui.sz(14.0))
            .color(if selected {
                theme::blue()
            } else {
                theme::overlay0(ui.dark)
            }),
        text(label.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding(Padding::from([8.0, 12.0]))
        .style(theme::nav_button(selected))
        .into()
}

pub fn sidebar_badge_button(
    icon: Bootstrap,
    accent: Color,
    label: &str,
    selected: bool,
    msg: Message,
    ui: Ui,
) -> Element<'static, Message> {
    let content = row![
        icon_badge(icon, accent, ui.sz(13.0)),
        text(label.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding(Padding::from([6.0, 12.0]))
        .style(theme::nav_button(selected))
        .into()
}

pub fn key_cap<'a>(label: &str, active: bool, ui: Ui) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(ui.sz(12.0))
            .font(if active { bold() } else { ui.font() })
            .color(if active {
                Color::WHITE
            } else {
                theme::subtext1(ui.dark)
            }),
    )
    .padding(Padding::from([6.0, 12.0]))
    .style(theme::key_cap(active))
    .into()
}

pub fn seg_button(label: &str, active: bool, msg: Message, ui: Ui) -> Element<'static, Message> {
    button(text(label.to_string()).size(ui.sz(12.0)).font(ui.font()))
        .on_press(msg)
        .padding(Padding::from([8.0, 20.0]))
        .style(theme::seg_button(active))
        .into()
}

pub fn info_row<'a>(label: &str, value: &str, ui: Ui) -> Element<'a, Message> {
    row![
        text(label.to_string())
            .size(ui.sz(13.0))
            .font(ui.font())
            .color(theme::subtext0(ui.dark))
            .width(120),
        text(value.to_string()).size(ui.sz(13.0)).font(ui.font()),
    ]
    .spacing(12)
    .into()
}

pub fn pref_toggle<'a>(
    label: &str,
    desc: &str,
    value: bool,
    msg: fn(bool) -> Message,
    ui: Ui,
) -> Element<'a, Message> {
    row![
        column![
            text(label.to_string()).size(ui.sz(14.0)).font(ui.font()),
            text(desc.to_string())
                .size(ui.sz(12.0))
                .font(ui.font())
                .color(theme::subtext0(ui.dark)),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value).on_toggle(msg).size(ui.sz(22.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Theme swatch button for the theme picker.
pub fn theme_swatch_btn<'a>(
    preview: impl Into<Element<'a, Message>>,
    label: &str,
    selected: bool,
    msg: Message,
    ui: Ui,
) -> Element<'a, Message> {
    button(
        column![
            preview.into(),
            text(label.to_string()).size(ui.sz(10.0)).font(ui.font()),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .on_press(msg)
    .padding(4)
    .style(theme::swatch_btn(selected))
    .into()
}
