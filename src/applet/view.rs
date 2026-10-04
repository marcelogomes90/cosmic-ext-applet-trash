use cosmic::applet;
use cosmic::iced::advanced::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::iced::{Alignment, Length};
use cosmic::{Element, widget};

use super::message::Message;
use super::{Trash, popup, style, symbols};
use crate::fl;

const GAP: u16 = 8;
const GAP_TIGHT: u16 = 4;
const ICON: u16 = 16;
const CONTROL_HEIGHT: u16 = 32;

pub fn popup(app: &Trash) -> Element<'_, Message> {
    let body: Element<'_, Message> = if app.asking() { question() } else { menu(app) };

    let surface = widget::container(body)
        .padding([GAP_TIGHT, 0])
        .style(style::surface);

    widget::autosize::autosize(surface, popup::SURFACE_ID.clone())
        .limits(popup::surface_limits())
        .into()
}

fn menu(app: &Trash) -> Element<'_, Message> {
    let open =
        applet::menu_button(row(symbols::open(), fl!("open-trash"))).on_press(Message::OpenTrash);

    let empty = applet::menu_button(row(symbols::empty(), fl!("empty-trash")))
        .on_press_maybe((!app.status().is_empty()).then_some(Message::ConfirmEmpty(true)));

    widget::column::with_children(vec![open.into(), empty.into()]).into()
}

fn row(handle: widget::icon::Handle, label: String) -> Element<'static, Message> {
    widget::row::with_children(vec![
        symbols::sized(handle, ICON).into(),
        widget::text::body(label)
            .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
            .width(Length::Fill)
            .into(),
    ])
    .spacing(GAP + GAP_TIGHT)
    .align_y(Alignment::Center)
    .into()
}

fn question() -> Element<'static, Message> {
    let words = widget::column::with_children(vec![
        widget::text::body(fl!("empty-trash-title")).into(),
        widget::text::caption(fl!("empty-trash-warning")).into(),
    ])
    .spacing(GAP_TIGHT)
    .width(Length::Fill);

    let answers = widget::row::with_children(vec![
        widget::button::text(fl!("action-cancel"))
            .class(style::flat(false))
            .height(Length::Fixed(f32::from(CONTROL_HEIGHT)))
            .on_press(Message::ConfirmEmpty(false))
            .into(),
        widget::button::text(fl!("action-empty"))
            .class(style::flat(true))
            .height(Length::Fixed(f32::from(CONTROL_HEIGHT)))
            .on_press(Message::Empty)
            .into(),
    ])
    .spacing(GAP_TIGHT);

    applet::padded_control(
        widget::column::with_children(vec![
            words.into(),
            widget::container(answers)
                .align_x(Alignment::End)
                .width(Length::Fill)
                .into(),
        ])
        .spacing(GAP),
    )
    .into()
}
