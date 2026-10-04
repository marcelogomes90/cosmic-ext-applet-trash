use cosmic::widget;

use crate::trash::Status;

pub fn panel(status: Status, symbolic: bool) -> widget::icon::Handle {
    let name = match (symbolic, status) {
        (true, Status::Occupied) => "user-trash-full-symbolic",
        (true, Status::Empty) => "user-trash-symbolic",
        (false, Status::Occupied) => "user-trash-full",
        (false, Status::Empty) => "user-trash",
    };

    widget::icon::from_name(name).prefer_svg(true).handle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_asks_the_theme_to_recolour_only_the_symbolic_pair() {
        assert!(
            panel(Status::Empty, true).symbolic,
            "a symbolic name must follow the panel's own ink"
        );
        assert!(
            panel(Status::Occupied, true).symbolic,
            "a symbolic name must follow the panel's own ink"
        );
        assert!(
            !panel(Status::Empty, false).symbolic,
            "the dock icon keeps the colours the icon theme drew"
        );
        assert!(
            !panel(Status::Occupied, false).symbolic,
            "the dock icon keeps the colours the icon theme drew"
        );
    }
}
