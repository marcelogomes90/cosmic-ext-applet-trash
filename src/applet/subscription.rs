use std::hash::{Hash, Hasher};

use cosmic::iced::keyboard::key::Named;
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::{Event, Subscription, event};
use futures::{Stream, StreamExt as _};

use crate::applet::message::Message;
use crate::trash;

struct Source;

impl Hash for Source {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "cosmic-ext-applet-trash-status".hash(state);
    }
}

pub fn status() -> Subscription<Message> {
    Subscription::run_with(Source, |_| stream())
}

fn stream() -> impl Stream<Item = Message> + Send + 'static {
    trash::changes().map(Message::Status)
}

pub fn dismissal() -> Subscription<Message> {
    event::listen_with(|event, _, _| match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        }) => Some(Message::ConfirmEmpty(false)),
        _ => None,
    })
}
