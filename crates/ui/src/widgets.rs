//! Reusable widget helpers for MyPowerToys Settings UI.

use iced::widget::{column, container, text};
use iced::{Element, Length};

/// A section with a title and content.
pub fn section<'a, Message: 'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![text(title).size(20), content.into()]
            .spacing(12)
            .width(Length::Fill),
    )
    .padding(16)
    .width(Length::Fill)
    .into()
}
