use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;

const TRASH_URI: &str = "trash:///";
const FILE_MANAGER: &str = "cosmic-files";

pub async fn open() {
    if open_in_file_manager().await {
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

    if handled("xdg-open", &[files.as_os_str()]).await {
        return;
    }

    if sandboxed() && spawn_inside("xdg-open", &[files.as_os_str()]) {
        return;
    }

    tracing::warn!("nothing could open the trash");
}

async fn open_in_file_manager() -> bool {
    if !available(FILE_MANAGER).await {
        return false;
    }

    spawn(FILE_MANAGER, &[OsStr::new("--trash")])
}

fn sandboxed() -> bool {
    Path::new("/.flatpak-info").exists()
}

/// Outside a sandbox this is the command itself; inside one it is the same command on the host,
/// where exit codes mean what they say and the desktop's own handlers are registered.
fn command(program: &str) -> tokio::process::Command {
    let mut command = if sandboxed() {
        let mut command = tokio::process::Command::new("flatpak-spawn");
        command.arg("--host").arg(program);
        command
    } else {
        tokio::process::Command::new(program)
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command
}

async fn available(program: &str) -> bool {
    let mut probe = command("sh");
    probe
        .arg("-c")
        .arg(format!("command -v {program} > /dev/null 2>&1"));

    match probe.status().await {
        Ok(status) => status.success(),
        Err(error) => {
            tracing::debug!(%error, program, "could not look for a handler");
            false
        }
    }
}

fn spawn(program: &str, args: &[&OsStr]) -> bool {
    match command(program).args(args).spawn() {
        Ok(_) => true,
        Err(error) => {
            tracing::debug!(%error, program, "could not start a handler");
            false
        }
    }
}

fn spawn_inside(program: &str, args: &[&OsStr]) -> bool {
    match tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => true,
        Err(error) => {
            tracing::debug!(%error, program, "could not start a handler in the sandbox");
            false
        }
    }
}

async fn handled(program: &str, args: &[&OsStr]) -> bool {
    let mut handler = match command(program).args(args).spawn() {
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
