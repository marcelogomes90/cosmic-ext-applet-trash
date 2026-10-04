use cosmic::applet;
use cosmic::iced::Length;
use cosmic::iced::advanced::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::{Element, widget};

use super::message::Message;
use super::{Trash, popup, style};
use crate::fl;

const GAP_TIGHT: u16 = 4;

pub fn popup(app: &Trash) -> Element<'_, Message> {
    let surface = widget::container(menu(app))
        .padding([GAP_TIGHT, 0])
        .style(style::surface);

    widget::autosize::autosize(surface, popup::SURFACE_ID.clone())
        .limits(popup::surface_limits())
        .into()
}

fn menu(app: &Trash) -> Element<'_, Message> {
    let emptyable = !app.status().is_empty();

    let open = applet::menu_button(label(fl!("open-trash"), true)).on_press(Message::OpenTrash);

    let empty = applet::menu_button(label(fl!("empty-trash"), emptyable))
        .on_press_maybe(emptyable.then_some(Message::ConfirmEmpty(true)));

    widget::column::with_children(vec![open.into(), divider(), empty.into()])
        .spacing(GAP_TIGHT)
        .into()
}

/// `Button::AppletMenu` paints its label with the surface's ordinary ink whether or not the button
/// is disabled, so a row with nothing to do has to dim itself.
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
    widget::container(widget::divider::horizontal::default())
        .padding([0, cosmic::theme::spacing().space_s])
        .into()
}

/// The confirmation is its own surface, not a page inside the popup: a question about deleting
/// everything should not be something the user can dismiss by looking away.
pub fn confirmation<'a>() -> Element<'a, Message> {
    widget::container(
        widget::dialog()
            .title(fl!("empty-trash-title"))
            .body(fl!("empty-trash-warning"))
            .primary_action(
                widget::button::destructive(fl!("action-empty")).on_press(Message::Empty),
            )
            .secondary_action(
                widget::button::standard(fl!("action-cancel"))
                    .on_press(Message::ConfirmEmpty(false)),
            ),
    )
    .center(Length::Fill)
    .into()
}
