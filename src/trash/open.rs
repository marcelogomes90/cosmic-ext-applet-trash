use std::ffi::OsStr;

const TRASH_URI: &str = "trash:///";

pub async fn open() {
    if spawn("cosmic-files", &[OsStr::new("--trash")]) {
        return;
    }

    tracing::debug!("COSMIC Files is not reachable, trying the trash URI");

    if handled("xdg-open", &[OsStr::new(TRASH_URI)]).await {
        return;
    }

    let Some(files) = super::files_dir() else {
        tracing::warn!("could not work out where the trash lives");
        return;
    };

    tracing::debug!(path = %files.display(), "no trash handler, opening the directory instead");

    if !handled("xdg-open", &[files.as_os_str()]).await {
        tracing::warn!("nothing could open the trash");
    }
}

fn spawn(program: &str, args: &[&OsStr]) -> bool {
    match tokio::process::Command::new(program).args(args).spawn() {
        Ok(_) => true,
        Err(error) => {
            tracing::debug!(%error, program, "could not start a handler");
            false
        }
    }
}

async fn handled(program: &str, args: &[&OsStr]) -> bool {
    let mut handler = match tokio::process::Command::new(program).args(args).spawn() {
        Ok(handler) => handler,
        Err(error) => {
            tracing::debug!(%error, program, "could not start a handler");
            return false;
        }
    };

    match handler.wait().await {
        Ok(status) => status.success(),
        Err(error) => {
            tracing::debug!(%error, program, "the handler did not report an outcome");
            false
        }
    }
}
