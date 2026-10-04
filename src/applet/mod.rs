pub mod message;
pub mod popup;
pub mod style;
pub mod subscription;
pub mod symbols;
pub mod view;

use cosmic::app::{Core, Task};
use cosmic::applet::PanelType;
use cosmic::iced::platform_specific::shell::commands::popup::destroy_popup;
use cosmic::iced::{Length, Subscription, window};
use cosmic::{Application, Element, widget};

use crate::trash::{self, Status};
use crate::{APP_ID, fl};

pub use message::Message;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PopupState {
    #[default]
    Closed,
    Open {
        id: window::Id,
        closing: bool,
    },
}

impl PopupState {
    fn id(self) -> Option<window::Id> {
        match self {
            Self::Closed => None,
            Self::Open { id, .. } => Some(id),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Confirm {
    #[default]
    Idle,
    Asking,
}

pub struct Trash {
    core: Core,
    status: Status,
    popup: PopupState,
    confirm: Confirm,
}

impl Trash {
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn asking(&self) -> bool {
        self.confirm == Confirm::Asking
    }

    fn open_popup(&mut self) -> Task<Message> {
        let id = window::Id::unique();
        self.popup = PopupState::Open { id, closing: false };
        self.status = trash::status();

        let parent = self
            .core
            .main_window_id()
            .expect("an applet always has a main window");

        cosmic::surface::surface_task(cosmic::surface::action::app_popup::<Self>(
            |_| cosmic::surface::action::LiveSettings::default(),
            move |app| {
                let mut settings = app
                    .core
                    .applet
                    .get_popup_settings(parent, id, None, None, None);
                settings.positioner.size_limits = popup::surface_limits();
                settings
            },
            None,
        ))
    }

    fn close_popup(&mut self) -> Task<Message> {
        match self.popup {
            PopupState::Open { id, closing: false } => {
                self.popup = PopupState::Open { id, closing: true };
                destroy_popup(id)
            }
            PopupState::Closed | PopupState::Open { closing: true, .. } => Task::none(),
        }
    }

    fn toggle_popup(&mut self) -> Task<Message> {
        if self.popup == PopupState::Closed {
            self.open_popup()
        } else {
            self.close_popup()
        }
    }

    fn panel_button(&self) -> widget::Button<'_, Message> {
        let handle = symbols::panel(self.status, symbolic_panel(&self.core.applet.panel_type));
        let symbolic = handle.symbolic;

        let (icon_width, icon_height) = self.core.applet.suggested_size(symbolic);
        let (major, minor) = self.core.applet.suggested_padding(symbolic);
        let (horizontal_padding, vertical_padding) = if self.core.applet.is_horizontal() {
            (major, minor)
        } else {
            (minor, major)
        };

        let icon = widget::icon(handle)
            .class(if symbolic {
                style::applet_ink()
            } else {
                cosmic::theme::Svg::default()
            })
            .width(Length::Fixed(f32::from(icon_width)))
            .height(Length::Fixed(f32::from(icon_height)));

        widget::button::custom(widget::layer_container(icon).center(Length::Fill))
            .width(Length::Fixed(f32::from(
                icon_width + 2 * horizontal_padding,
            )))
            .height(Length::Fixed(f32::from(icon_height + 2 * vertical_padding)))
            .class(cosmic::theme::Button::AppletIcon)
    }
}

fn symbolic_panel(panel: &PanelType) -> bool {
    *panel != PanelType::Dock
}

fn reconcile_surface_closed(id: window::Id, state: &mut PopupState) -> bool {
    if state.id() == Some(id) {
        *state = PopupState::Closed;
        return true;
    }

    false
}

fn empty_trash() -> Task<Message> {
    cosmic::task::future(async {
        match tokio::task::spawn_blocking(trash::empty).await {
            Ok(outcome) => tracing::info!(
                purged = outcome.purged,
                failed = outcome.failed,
                "emptied the trash"
            ),
            Err(error) => tracing::warn!(%error, "emptying the trash did not finish"),
        }

        Message::Status(trash::status())
    })
}

impl Application for Trash {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, (): Self::Flags) -> (Self, Task<Message>) {
        (
            Self {
                core,
                status: trash::status(),
                popup: PopupState::Closed,
                confirm: Confirm::Idle,
            },
            Task::none(),
        )
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::SurfaceClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::status()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePopup => self.toggle_popup(),
            Message::SurfaceClosed(id) => {
                if reconcile_surface_closed(id, &mut self.popup) {
                    self.confirm = Confirm::Idle;
                }
                Task::none()
            }
            Message::Surface(action) => cosmic::task::message(cosmic::Action::Surface(action)),
            Message::Status(status) => {
                if self.status == status {
                    return Task::none();
                }
                self.status = status;
                cosmic::task::message(Message::Relayout)
            }
            Message::OpenTrash => Task::batch([
                self.close_popup(),
                cosmic::task::future(async {
                    trash::open().await;
                    Message::Relayout
                }),
            ]),
            Message::ConfirmEmpty(asking) => {
                self.confirm = if asking {
                    Confirm::Asking
                } else {
                    Confirm::Idle
                };
                cosmic::task::message(Message::Relayout)
            }
            Message::Empty => {
                self.confirm = Confirm::Idle;
                Task::batch([self.close_popup(), empty_trash()])
            }
            Message::Relayout => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let button = widget::mouse_area(self.panel_button().on_press(Message::OpenTrash))
            .on_right_press(Message::TogglePopup);

        // Only the dock names the icons it holds on hover; on a panel a tooltip would be noise.
        let button: Element<'_, Message> = if self.core.applet.panel_type == PanelType::Dock {
            self.core
                .applet
                .applet_tooltip(
                    button,
                    fl!("app-title"),
                    self.popup != PopupState::Closed,
                    Message::Surface,
                    self.core.main_window_id(),
                )
                .into()
        } else {
            button.into()
        };

        widget::autosize::autosize(button, popup::PANEL_ID.clone())
            .limits(popup::panel_limits(
                self.core.applet.suggested_bounds,
                self.core.applet.is_horizontal(),
            ))
            .into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if self.popup.id() != Some(id) {
            return widget::text::body("").into();
        }

        view::popup(self)
    }
}

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<Trash>(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_the_popup_leaves_no_surface() {
        let id = window::Id::unique();
        let mut state = PopupState::Open { id, closing: true };

        assert!(
            reconcile_surface_closed(id, &mut state),
            "the popup's own close must be recognised"
        );
        assert_eq!(
            state,
            PopupState::Closed,
            "a closed surface leaves the applet with no popup"
        );
    }

    #[test]
    fn a_stray_close_for_another_surface_leaves_the_popup_alone() {
        let id = window::Id::unique();
        let mut state = PopupState::Open { id, closing: false };

        assert!(
            !reconcile_surface_closed(window::Id::unique(), &mut state),
            "another surface's close is not ours"
        );
        assert_eq!(
            state,
            PopupState::Open { id, closing: false },
            "the popup stays open"
        );
    }

    #[test]
    fn the_panel_asks_for_symbolic_icons_everywhere_but_the_dock() {
        assert!(
            symbolic_panel(&PanelType::Panel),
            "the panel wants the glyph every native applet uses"
        );
        assert!(
            !symbolic_panel(&PanelType::Dock),
            "the dock wants the icon the theme drew for an application"
        );
        assert!(
            symbolic_panel(&PanelType::Other(String::new())),
            "run outside a panel, the conservative choice is the symbolic glyph"
        );
        assert!(
            symbolic_panel(&PanelType::Other("Sidebar".to_string())),
            "a user-created panel is still a panel"
        );
    }
}
