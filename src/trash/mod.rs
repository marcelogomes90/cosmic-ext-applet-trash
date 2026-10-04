pub mod open;
pub mod watch;

use std::path::{Path, PathBuf};

pub use open::open;
pub use watch::changes;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Empty,
    Occupied,
}

impl Status {
    pub fn is_empty(self) -> bool {
        self == Self::Empty
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub purged: usize,
    pub failed: usize,
}

pub fn status() -> Status {
    match trash::os_limited::is_empty() {
        Ok(true) => Status::Empty,
        Ok(false) => Status::Occupied,
        Err(error) => {
            tracing::debug!(%error, "treating an unreadable trash as empty");
            Status::Empty
        }
    }
}

pub fn empty() -> Outcome {
    let items = match trash::os_limited::list() {
        Ok(items) => items,
        Err(error) => {
            tracing::warn!(%error, "could not read the trash");
            return Outcome::default();
        }
    };

    let mut outcome = Outcome::default();

    for item in items {
        match trash::os_limited::purge_all([item]) {
            Ok(()) => outcome.purged += 1,
            Err(error) => {
                outcome.failed += 1;
                tracing::warn!(%error, "could not delete a trashed item");
            }
        }
    }

    if outcome.failed > 0 {
        tracing::warn!(
            purged = outcome.purged,
            failed = outcome.failed,
            "some trashed items could not be deleted"
        );
    }

    outcome
}

pub fn home_trash() -> Option<PathBuf> {
    data_home().map(|home| home.join("Trash"))
}

pub fn files_dir() -> Option<PathBuf> {
    home_trash().map(|trash| trash.join("files"))
}

pub fn adopt_host_data_home() {
    if !Path::new("/.flatpak-info").exists() {
        return;
    }

    let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) else {
        return;
    };

    let host = Path::new(&home).join(".local/share");

    unsafe {
        std::env::set_var("XDG_DATA_HOME", &host);
    }
}

fn data_home() -> Option<PathBuf> {
    if let Some(value) = std::env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(value));
    }

    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| Path::new(&home).join(".local/share"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_status_reads_as_empty_and_an_occupied_one_does_not() {
        assert!(Status::Empty.is_empty(), "an empty trash is empty");
        assert!(
            !Status::Occupied.is_empty(),
            "an occupied trash is not empty"
        );
    }

    #[test]
    fn the_default_status_is_empty_so_a_failed_probe_shows_the_quieter_icon() {
        assert_eq!(
            Status::default(),
            Status::Empty,
            "the panel should never claim the trash is full on a guess"
        );
    }

    #[test]
    fn the_trash_directories_follow_the_freedesktop_layout() {
        let trash = home_trash().expect("a home directory in the test environment");
        let files = files_dir().expect("a home directory in the test environment");

        assert!(
            trash.ends_with("Trash"),
            "the home trash is a Trash directory: {}",
            trash.display()
        );
        assert_eq!(
            files,
            trash.join("files"),
            "the trashed files live in the bin's files directory"
        );
    }

    #[test]
    fn nothing_was_purged_by_default_so_a_failed_listing_reports_no_work() {
        let outcome = Outcome::default();

        assert_eq!(outcome.purged, 0, "no item is purged without a listing");
        assert_eq!(
            outcome.failed, 0,
            "no failure is invented without a listing"
        );
    }
}
