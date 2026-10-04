use cosmic::iced::window;

use crate::trash::Status;

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    SurfaceClosed(window::Id),
    Surface(cosmic::surface::Action<Message>),
    Status(Status),
    OpenTrash,
    ConfirmEmpty(bool),
    Empty,
    Relayout,
}
