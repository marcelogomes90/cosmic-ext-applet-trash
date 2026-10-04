use cosmic::applet;
use cosmic::iced::Length;
use cosmic::iced::advanced::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::{Element, widget};

use super::message::Message;
use super::{Trash, popup, style};
use crate::fl;

pub fn popup(app: &Trash) -> Element<'_, Message> {
    let body: Element<'_, Message> = if app.asking() { question() } else { menu(app) };

    let surface = widget::container(body).style(style::surface(app.frosted()));

    widget::autosize::autosize(surface, popup::SURFACE_ID.clone())
        .limits(popup::surface_limits())
        .into()
}

fn menu(app: &Trash) -> Element<'_, Message> {
    let emptyable = !app.status().is_empty();

    let open = applet::menu_button(label(fl!("open-trash"), true)).on_press(Message::OpenTrash);

    let empty = applet::menu_button(label(fl!("empty-trash"), emptyable))
        .on_press_maybe(emptyable.then_some(Message::ConfirmEmpty(true)));

    widget::column::with_children(vec![open.into(), divider(), empty.into()]).into()
}

fn label(text: String, enabled: bool) -> Element<'static, Message> {
    let mut text = widget::text::body(text)
        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
        .width(Length::Fill);

    if !enabled {
        text = text.class(style::dimmed_text());
    }

    text.into()
}

fn divider<'a>() -> Element<'a, Message> {
    widget::divider::horizontal::default().into()
}

fn question<'a>() -> Element<'a, Message> {
    let spacing = cosmic::theme::spacing();

    let words = widget::column::with_children(vec![
        widget::text::title3(fl!("empty-trash-title")).into(),
        widget::space::vertical()
            .height(Length::Fixed(f32::from(spacing.space_xxs)))
            .into(),
        widget::text::body(fl!("empty-trash-warning")).into(),
    ]);

    let answers = widget::row::with_children(vec![
        widget::space::horizontal().into(),
        widget::button::standard(fl!("action-cancel"))
            .on_press(Message::ConfirmEmpty(false))
            .into(),
        widget::button::suggested(fl!("empty-trash"))
            .on_press(Message::Empty)
            .into(),
    ])
    .spacing(spacing.space_xxs);

    widget::container(
        widget::column::with_children(vec![words.into(), answers.into()]).spacing(spacing.space_l),
    )
    .padding(spacing.space_m)
    .width(Length::Fill)
    .into()
}
