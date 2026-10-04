use cosmic::iced::widget::svg;

pub fn applet_ink() -> cosmic::theme::Svg {
    cosmic::theme::Svg::custom(|theme| svg::Style {
        color: Some(theme.cosmic().background(theme.transparent).on.into()),
    })
}
