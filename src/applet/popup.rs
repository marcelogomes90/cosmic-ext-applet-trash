use std::sync::LazyLock;

use cosmic::iced::{Limits, Size};

pub const SURFACE_WIDTH: f32 = 280.0;
pub const SURFACE_MAX_HEIGHT: f32 = 400.0;

pub static SURFACE_ID: LazyLock<cosmic::iced::id::Id> =
    LazyLock::new(|| cosmic::iced::id::Id::new("cosmic-ext-applet-trash-popup"));
pub static PANEL_ID: LazyLock<cosmic::iced::id::Id> =
    LazyLock::new(|| cosmic::iced::id::Id::new("cosmic-ext-applet-trash-panel"));

pub fn surface_limits() -> Limits {
    Limits::NONE
        .min_width(1.0)
        .max_width(SURFACE_WIDTH)
        .min_height(1.0)
        .max_height(SURFACE_MAX_HEIGHT)
}

pub const DIALOG_WIDTH: u16 = 420;
pub const DIALOG_HEIGHT: u16 = 240;

pub fn dialog_limits() -> Limits {
    Limits::NONE
        .min_width(1.0)
        .max_width(f32::from(DIALOG_WIDTH))
        .min_height(1.0)
        .max_height(f32::from(DIALOG_HEIGHT))
}

pub fn panel_limits(suggested: Option<Size>, horizontal: bool) -> Limits {
    let Some(bounds) = suggested else {
        return Limits::NONE;
    };

    let mut limits = Limits::NONE;

    if horizontal {
        if bounds.width > 0.0 {
            limits = limits.max_width(bounds.width);
        }
        if bounds.height > 0.0 {
            limits = limits.height(bounds.height);
        }
    } else {
        if bounds.width > 0.0 {
            limits = limits.width(bounds.width);
        }
        if bounds.height > 0.0 {
            limits = limits.max_height(bounds.height);
        }
    }

    limits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn the_panel_button_keeps_its_own_length_however_long_the_panel_is() {
        let limits = panel_limits(Some(Size::new(1920.0, 40.0)), true);
        let resolved = limits.resolve(
            cosmic::iced::Length::Shrink,
            cosmic::iced::Length::Shrink,
            Size::new(28.0, 28.0),
        );

        assert!(near(resolved.width, 28.0), "width was {}", resolved.width);
        assert!(
            near(resolved.height, 40.0),
            "height was {}",
            resolved.height
        );
    }

    #[test]
    fn a_crowded_panel_never_stretches_the_button_to_fill_what_is_left() {
        for slot in [24.0_f32, 48.0, 200.0, 1920.0] {
            let limits = panel_limits(Some(Size::new(slot, 40.0)), true);
            let resolved = limits.resolve(
                cosmic::iced::Length::Shrink,
                cosmic::iced::Length::Shrink,
                Size::new(28.0, 28.0),
            );

            assert!(
                resolved.width <= 28.0 + 0.01,
                "a {slot}px slot stretched the button to {}",
                resolved.width
            );
        }
    }

    #[test]
    fn a_vertical_panel_constrains_the_other_way_round() {
        let limits = panel_limits(Some(Size::new(40.0, 1080.0)), false);
        let resolved = limits.resolve(
            cosmic::iced::Length::Shrink,
            cosmic::iced::Length::Shrink,
            Size::new(28.0, 28.0),
        );

        assert!(near(resolved.width, 40.0), "width was {}", resolved.width);
        assert!(
            near(resolved.height, 28.0),
            "height was {}",
            resolved.height
        );
    }

    #[test]
    fn a_panel_without_suggested_bounds_constrains_nothing() {
        let limits = panel_limits(None, true);
        let resolved = limits.resolve(
            cosmic::iced::Length::Shrink,
            cosmic::iced::Length::Shrink,
            Size::new(28.0, 28.0),
        );

        assert!(near(resolved.width, 28.0));
        assert!(near(resolved.height, 28.0));
    }

    #[test]
    fn the_popup_never_grows_past_the_width_the_layout_was_drawn_for() {
        let limits = surface_limits();
        let resolved = limits.resolve(
            cosmic::iced::Length::Shrink,
            cosmic::iced::Length::Shrink,
            Size::new(9999.0, 9999.0),
        );

        assert!(near(resolved.width, SURFACE_WIDTH));
        assert!(near(resolved.height, SURFACE_MAX_HEIGHT));
    }
}
