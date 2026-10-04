use cosmic::cosmic_theme::palette::Srgba;
use cosmic::iced::{Background, Border, Color, Shadow};
use cosmic::widget;

pub fn surface(theme: &cosmic::Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    let background = cosmic.background(theme.transparent);

    widget::container::Style {
        text_color: Some(background.on.into()),
        icon_color: Some(background.on.into()),
        background: Some(Color::from(background.base).into()),
        border: Border {
            radius: cosmic.corner_radii.radius_m.into(),
            width: 1.0,
            color: background.divider.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn applet_ink() -> cosmic::theme::Svg {
    cosmic::theme::Svg::custom(|theme| cosmic::iced::widget::svg::Style {
        color: Some(theme.cosmic().background(theme.transparent).on.into()),
    })
}

pub fn flat(destructive: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |_, theme| paint(theme, ink(theme, destructive), None)),
        disabled: Box::new(move |theme| paint(theme, Color::from(on_disabled(theme)), None)),
        hovered: Box::new(move |_, theme| {
            paint(theme, ink(theme, destructive), Some(hover(theme)))
        }),
        pressed: Box::new(move |_, theme| {
            paint(theme, ink(theme, destructive), Some(pressed(theme)))
        }),
    }
}

fn paint(theme: &cosmic::Theme, ink: Color, background: Option<Srgba>) -> widget::button::Style {
    widget::button::Style {
        background: background.map(|colour| Background::Color(colour.into())),
        border_radius: theme.cosmic().corner_radii.radius_m.into(),
        icon_color: Some(ink),
        text_color: Some(ink),
        ..widget::button::Style::new()
    }
}

fn ink(theme: &cosmic::Theme, destructive: bool) -> Color {
    let cosmic = theme.cosmic();

    if destructive {
        Color::from(cosmic.destructive_text_color())
    } else {
        Color::from(cosmic.on_bg_color())
    }
}

fn hover(theme: &cosmic::Theme) -> Srgba {
    theme.cosmic().primary(theme.transparent).component.hover
}

fn pressed(theme: &cosmic::Theme) -> Srgba {
    theme
        .cosmic()
        .background(theme.transparent)
        .component
        .pressed
}

fn on_disabled(theme: &cosmic::Theme) -> Srgba {
    theme
        .cosmic()
        .background(theme.transparent)
        .component
        .on_disabled
}

const DIMMED: f32 = 0.5;

pub fn dimmed_text() -> cosmic::theme::Text {
    cosmic::theme::Text::Custom(|theme| cosmic::iced::widget::text::Style {
        color: Some(dimmed(theme)),
        ..Default::default()
    })
}

pub fn dimmed_icon() -> cosmic::theme::Svg {
    cosmic::theme::Svg::custom(|theme| cosmic::iced::widget::svg::Style {
        color: Some(dimmed(theme)),
    })
}

fn dimmed(theme: &cosmic::Theme) -> Color {
    let mut ink = Color::from(theme.cosmic().on_bg_color());
    ink.a = DIMMED;
    ink
}
