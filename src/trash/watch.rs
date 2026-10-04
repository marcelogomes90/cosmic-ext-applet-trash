use std::path::PathBuf;

use futures::{Stream, StreamExt as _};
use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};

use super::Status;

pub fn changes() -> impl Stream<Item = Status> + Send + 'static {
    let (sender, receiver) = futures::channel::mpsc::channel(4);
    let watch = Watch::start(sender);

    futures::stream::unfold(
        (receiver, watch, super::status()),
        |(mut receiver, mut watch, last)| async move {
            loop {
                receiver.next().await?;
                watch.catch_up();

                let current = super::status();
                if current != last {
                    return Some((current, (receiver, watch, current)));
                }
            }
        },
    )
}

struct Watch {
    watcher: Option<RecommendedWatcher>,
    complete: bool,
}

impl Watch {
    fn start(mut sender: futures::channel::mpsc::Sender<()>) -> Self {
        let watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
                Ok(event) if event.kind.is_access() => {}
                Ok(_) => {
                    let _ = sender.try_send(());
                }
                Err(error) => tracing::debug!(%error, "the trash watcher reported a problem"),
            });

        let mut watch = match watcher {
            Ok(watcher) => Self {
                watcher: Some(watcher),
                complete: false,
            },
            Err(error) => {
                tracing::warn!(%error, "the trash will not be watched for changes");
                return Self {
                    watcher: None,
                    complete: true,
                };
            }
        };

        watch.catch_up();
        watch
    }

    fn catch_up(&mut self) {
        if self.complete {
            return;
        }

        let Some(watcher) = self.watcher.as_mut() else {
            return;
        };

        let wanted = directories();
        let mut missing = 0usize;

        for directory in &wanted {
            if let Err(error) = watcher.watch(directory, RecursiveMode::NonRecursive) {
                missing += 1;
                tracing::debug!(%error, path = %directory.display(), "not watching a directory yet");
            }
        }

        self.complete = missing == 0 && !wanted.is_empty();
    }
}

fn directories() -> Vec<PathBuf> {
    let mut wanted = Vec::new();

    for bin in bins() {
        wanted.push(bin.join("files"));
        wanted.push(bin);
    }

    wanted
}

pub fn bins() -> Vec<PathBuf> {
    match trash::os_limited::trash_folders() {
        Ok(bins) if !bins.is_empty() => bins.into_iter().collect(),
        Ok(_) | Err(_) => super::home_trash().into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bin_is_watched_at_its_files_directory_and_at_the_bin_itself() {
        let wanted = directories();

        assert_eq!(
            wanted.len(),
            bins().len() * 2,
            "watching the bin as well as its files directory is what survives a files \
             directory that does not exist yet"
        );
        assert!(
            wanted.iter().any(|path| path.ends_with("files")),
            "the files directory is where trashed items actually appear: {wanted:?}"
        );
    }

    #[test]
    fn a_watcher_that_could_not_start_asks_for_no_further_attempts() {
        let watch = Watch {
            watcher: None,
            complete: true,
        };

        assert!(
            watch.complete,
            "without a watcher there is nothing left to catch up on"
        );
    }
}
