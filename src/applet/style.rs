use cosmic::iced::widget::svg;
use cosmic::iced::{Border, Color, Shadow};
use cosmic::widget;

pub fn applet_ink() -> cosmic::theme::Svg {
    cosmic::theme::Svg::custom(|theme| svg::Style {
        color: Some(theme.cosmic().background(theme.transparent).on.into()),
    })
}

pub fn card(theme: &cosmic::Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    let component = &cosmic.background(theme.transparent).component;

    widget::container::Style {
        text_color: Some(Color::from(component.on)),
        icon_color: Some(Color::from(component.on)),
        background: Some(Color::from(component.base).into()),
        border: Border {
            radius: cosmic.corner_radii.radius_s.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn dimmed_text() -> cosmic::theme::Text {
    cosmic::theme::Text::Custom(|theme| cosmic::iced::widget::text::Style {
        color: Some(dimmed(theme)),
        ..Default::default()
    })
}

fn dimmed(theme: &cosmic::Theme) -> Color {
    let mut ink = Color::from(theme.cosmic().on_bg_color());
    ink.a = 0.5;
    ink
}
