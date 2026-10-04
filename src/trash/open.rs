use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

const TRASH_URI: &str = "trash:///";
const FILE_MANAGER: &str = "cosmic-files";

const SETTLE: Duration = Duration::from_millis(1500);

pub async fn open() {
    let launcher = Launcher::detect().await;

    if launcher.available(FILE_MANAGER).await
        && launcher
            .launch(FILE_MANAGER, &[OsStr::new("--trash")])
            .await
    {
        return;
    }

    tracing::debug!("COSMIC Files did not take it, trying the trash URI");

    if launcher.handled("xdg-open", &[OsStr::new(TRASH_URI)]).await {
        return;
    }

    let Some(files) = super::files_dir() else {
        tracing::warn!("could not work out where the trash lives");
        return;
    };

    tracing::debug!(path = %files.display(), "no trash handler, opening the directory instead");

    if launcher.handled("xdg-open", &[files.as_os_str()]).await {
        return;
    }

    if launcher.is_host()
        && Launcher::Direct
            .handled("xdg-open", &[files.as_os_str()])
            .await
    {
        return;
    }

    tracing::warn!("nothing could open the trash");
}

enum Launcher {
    Direct,
    Host { display: Option<String> },
}

impl Launcher {
    async fn detect() -> Self {
        if !Path::new("/.flatpak-info").exists() {
            return Self::Direct;
        }

        let display = host_wayland_display().await;

        if display.is_none() {
            tracing::debug!("no compositor socket found on the host");
        }

        Self::Host { display }
    }

    fn is_host(&self) -> bool {
        matches!(self, Self::Host { .. })
    }

    fn command(&self, program: &str) -> tokio::process::Command {
        let mut command = match self {
            Self::Direct => tokio::process::Command::new(program),
            Self::Host { display } => {
                let mut command = tokio::process::Command::new("flatpak-spawn");
                command.arg("--host");
                if let Some(display) = display {
                    command.arg(format!("--env=WAYLAND_DISPLAY={display}"));
                }
                command.arg(program);
                command
            }
        };

        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        command
    }

    async fn available(&self, program: &str) -> bool {
        let mut probe = self.command("sh");
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

    async fn launch(&self, program: &str, args: &[&OsStr]) -> bool {
        let mut handler = match self.command(program).args(args).spawn() {
            Ok(handler) => handler,
            Err(error) => {
                tracing::debug!(%error, program, "could not start a handler");
                return false;
            }
        };

        match tokio::time::timeout(SETTLE, handler.wait()).await {
            Err(_) => true,
            Ok(Ok(status)) => {
                if !status.success() {
                    tracing::debug!(program, ?status, "the handler gave up straight away");
                }
                status.success()
            }
            Ok(Err(error)) => {
                tracing::debug!(%error, program, "the handler did not report an outcome");
                false
            }
        }
    }

    async fn handled(&self, program: &str, args: &[&OsStr]) -> bool {
        let mut handler = match self.command(program).args(args).spawn() {
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
}

async fn host_wayland_display() -> Option<String> {
    let output = tokio::process::Command::new("flatpak-spawn")
        .args([
            "--host",
            "sh",
            "-c",
            "ls -1 \"$XDG_RUNTIME_DIR\" 2>/dev/null",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|name| is_compositor_socket(name))
        .map(str::to_owned)
}

fn is_compositor_socket(name: &str) -> bool {
    name.strip_prefix("wayland-")
        .is_some_and(|tail| !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_compositor_socket_is_recognised_by_its_name_alone() {
        assert!(is_compositor_socket("wayland-0"), "the usual first socket");
        assert!(is_compositor_socket("wayland-1"), "the usual COSMIC socket");

        for other in [
            "wayland-0.lock",
            "wayland-",
            "waylandish",
            "bus",
            "",
            "wayland-x",
        ] {
            assert!(
                !is_compositor_socket(other),
                "{other} is not a compositor socket"
            );
        }
    }
}
