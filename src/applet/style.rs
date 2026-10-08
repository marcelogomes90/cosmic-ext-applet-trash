use cosmic::iced::Color;
use cosmic::iced::border::Radius;
use cosmic::iced::widget::svg;
use cosmic::widget::button::{Catalog as _, Style};

const SURFACE_BORDER: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub top: bool,
    pub bottom: bool,
}

impl Edges {
    pub const TOP: Self = Self {
        top: true,
        bottom: false,
    };
    pub const BOTTOM: Self = Self {
        top: false,
        bottom: true,
    };
}

pub fn applet_ink() -> cosmic::theme::Svg {
    cosmic::theme::Svg::custom(|theme| svg::Style {
        color: Some(theme.cosmic().background(theme.transparent).on.into()),
    })
}

pub fn menu_row(edges: Edges) -> cosmic::theme::Button {
    const CLASS: cosmic::theme::Button = cosmic::theme::Button::AppletMenu;

    cosmic::theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            follow_surface(theme.active(focused, false, &CLASS), theme, edges)
        }),
        disabled: Box::new(move |theme| follow_surface(theme.disabled(&CLASS), theme, edges)),
        hovered: Box::new(move |focused, theme| {
            follow_surface(theme.hovered(focused, false, &CLASS), theme, edges)
        }),
        pressed: Box::new(move |focused, theme| {
            follow_surface(theme.pressed(focused, false, &CLASS), theme, edges)
        }),
    }
}

fn follow_surface(mut style: Style, theme: &cosmic::Theme, edges: Edges) -> Style {
    style.border_radius = inner_radius(theme.cosmic().corner_radii.radius_m, edges);
    style
}

fn inner_radius(surface: [f32; 4], edges: Edges) -> Radius {
    let inset = |corner: f32| (corner - SURFACE_BORDER).max(0.0);

    Radius {
        top_left: if edges.top { inset(surface[0]) } else { 0.0 },
        top_right: if edges.top { inset(surface[1]) } else { 0.0 },
        bottom_right: if edges.bottom { inset(surface[2]) } else { 0.0 },
        bottom_left: if edges.bottom { inset(surface[3]) } else { 0.0 },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_corners_that_touch_the_surface_are_rounded() {
        let surface = [8.0; 4];

        assert_eq!(
            inner_radius(surface, Edges::TOP),
            Radius {
                top_left: 7.0,
                top_right: 7.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            "the first row follows the surface above it and meets the divider square"
        );
        assert_eq!(
            inner_radius(surface, Edges::BOTTOM),
            Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 7.0,
                bottom_left: 7.0,
            },
            "the last row follows the surface below it and meets the divider square"
        );
        assert_eq!(
            inner_radius(surface, Edges::default()),
            Radius::from(0.0),
            "a row touching neither end of the surface is square"
        );
    }

    #[test]
    fn a_rounded_corner_is_the_surfaces_own_less_the_border_it_sits_inside() {
        assert_eq!(
            inner_radius([16.0; 4], Edges::TOP),
            Radius {
                top_left: 15.0,
                top_right: 15.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            "the round theme's 16 px surface corner is inset by the popup's 1 px border"
        );
        assert_eq!(
            inner_radius([0.0; 4], Edges::TOP),
            Radius::from(0.0),
            "a square theme stays square rather than going negative"
        );
    }
}
