use cosmic::applet;
use cosmic::iced::Length;
use cosmic::iced::advanced::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::{Element, widget};

use super::message::Message;
use super::{Trash, popup};
use crate::fl;

pub fn popup(app: &Trash) -> Element<'_, Message> {
    let body: Element<'_, Message> = if app.asking() { question() } else { menu(app) };

    app.applet()
        .popup_container(
            widget::container(body)
                .padding(1)
                .height(Length::Shrink)
                .width(Length::Fill),
        )
        .limits(popup::surface_limits())
        .into()
}

fn menu(app: &Trash) -> Element<'_, Message> {
    let emptyable = !app.status().is_empty();

    let open = menu_button(fl!("open-trash")).on_press(Message::OpenTrash);

    let empty = menu_button(fl!("empty-trash"))
        .on_press_maybe(emptyable.then_some(Message::ConfirmEmpty(true)));

    widget::column::with_children(vec![
        open.into(),
        widget::divider::horizontal::light().into(),
        empty.into(),
    ])
    .into()
}

fn menu_button<'a>(label: String) -> widget::Button<'a, Message> {
    widget::button::custom(
        widget::text::body(label).ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1))),
    )
    .height(20 + 2 * cosmic::theme::spacing().space_xxs)
    .class(cosmic::theme::Button::MenuItem)
    .padding(applet::menu_control_padding())
    .width(Length::Fill)
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
        widget::button::suggested(fl!("action-confirm"))
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
