use cosmic::applet;
use cosmic::iced::Length;
use cosmic::iced::advanced::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::{Element, widget};

use super::message::Message;
use super::{Trash, popup, style};
use crate::fl;

const GAP_TIGHT: u16 = 4;

pub fn popup(app: &Trash) -> Element<'_, Message> {
    // The dialog paints its own card, so it stands in for the popup surface rather than sitting on
    // top of one — two stacked backgrounds would lose the user's frosted-panel setting.
    let body: Element<'_, Message> = if app.asking() {
        question()
    } else {
        widget::container(menu(app))
            .padding([GAP_TIGHT, 0])
            .style(style::surface)
            .into()
    };

    widget::autosize::autosize(body, popup::SURFACE_ID.clone())
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

/// COSMIC Files has an Empty Trash dialog, but exposes neither a D-Bus interface nor a
/// command-line flag to raise it, so the question is asked here — in its own widget, so the two
/// read as the same question.
fn question<'a>() -> Element<'a, Message> {
    widget::dialog()
        .title(fl!("empty-trash-title"))
        .body(fl!("empty-trash-warning"))
        .primary_action(widget::button::destructive(fl!("empty-trash")).on_press(Message::Empty))
        .secondary_action(
            widget::button::standard(fl!("action-cancel")).on_press(Message::ConfirmEmpty(false)),
        )
        .width(Length::Fill)
        .into()
}
