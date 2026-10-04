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
    let emptyable = !app.status().is_empty();

    let open = applet::menu_button(row(symbols::open(), fl!("open-trash"), true))
        .on_press(Message::OpenTrash);

    let empty = applet::menu_button(row(symbols::empty(), fl!("empty-trash"), emptyable))
        .on_press_maybe(emptyable.then_some(Message::ConfirmEmpty(true)));

    widget::column::with_children(vec![open.into(), divider(), empty.into()])
        .spacing(GAP_TIGHT)
        .into()
}

/// `Button::AppletMenu` paints its label with the surface's ordinary ink whether or not the button
/// is disabled, so a row with nothing to do has to dim itself.
fn row(handle: widget::icon::Handle, label: String, enabled: bool) -> Element<'static, Message> {
    let mut icon = symbols::sized(handle, ICON);
    if !enabled {
        icon = icon.class(style::dimmed_icon());
    }

    let mut text = widget::text::body(label)
        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
        .width(Length::Fill);
    if !enabled {
        text = text.class(style::dimmed_text());
    }

    widget::row::with_children(vec![icon.into(), text.into()])
        .spacing(GAP + GAP_TIGHT)
        .align_y(Alignment::Center)
        .into()
}

fn divider<'a>() -> Element<'a, Message> {
    widget::container(widget::divider::horizontal::default())
        .padding([0, cosmic::theme::spacing().space_s])
        .into()
}

fn question<'a>() -> Element<'a, Message> {
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
