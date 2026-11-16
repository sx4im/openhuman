//! Tauri commands for the Claude Code CLI provider.
//!
//! Provides a cross-platform "open a terminal and run `claude auth login`"
//! helper. The CLI's OAuth flow is interactive (it prints a URL and
//! waits for the user to paste a code), so we can't host it in-app — we
//! detach into the user's native terminal so they complete login there,
//! then return to OpenHuman and click Recheck in the settings card.

use std::process::Command;

#[cfg(any(test, target_os = "macos"))]
use std::process::{Child, Stdio};
#[cfg(any(test, target_os = "macos"))]
use std::time::Duration;

/// The CLI's authentication entry point, as argv.
///
/// Authentication lives under the `auth` subcommand: `claude auth login`.
/// There is no top-level `login` command, and because `claude` accepts a
/// positional `[prompt]`, the old `claude login` was not rejected — it was
/// read as a prompt and started an ordinary session, so the sign-in flow
/// never ran and the settings card kept reporting the account as signed out.
///
/// `--claudeai` is the CLI's own default; it is passed explicitly so the
/// launcher does not silently follow a future change of that default into
/// console/API billing.
const CLAUDE_LOGIN_ARGV: &[&str] = &["claude", "auth", "login", "--claudeai"];

/// The same command as one shell word, for terminals that take a string
/// rather than an argv (`cmd /k`, AppleScript `do script`, `xfce4-terminal -e`).
fn login_command_line() -> String {
    CLAUDE_LOGIN_ARGV.join(" ")
}

/// Argv for `cmd /c start "" cmd /k <login command>`.
/// Compiled when testing on any host too, so a Linux CI run still checks the
/// Windows argv; otherwise only where it is actually spawned. Same for the
/// macOS and Linux helpers below.
#[cfg(any(test, target_os = "windows"))]
fn windows_launch_args() -> Vec<String> {
    // `start ""` opens a new console window; the empty quoted title
    // prevents cmd from interpreting the first arg as a title.
    // `cmd /k` keeps the window open after the command exits so the
    // user can read any final output.
    vec![
        "/c".into(),
        "start".into(),
        "".into(),
        "cmd".into(),
        "/k".into(),
        login_command_line(),
    ]
}

/// The AppleScript handed to `osascript`.
#[cfg(any(test, target_os = "macos"))]
fn macos_launch_script() -> String {
    format!(
        r#"tell application "Terminal"
    activate
    do script "{}"
end tell"#,
        login_command_line()
    )
}

/// How long the macOS launcher may take to report back before the invoke
/// returns success on the assumption that Terminal.app is still opening
/// (for example, the user is still answering the macOS Automation prompt).
#[cfg(any(test, target_os = "macos"))]
const MACOS_LAUNCH_WAIT: Duration = Duration::from_secs(30);

/// Outcome of waiting on a launcher child process.
#[cfg(any(test, target_os = "macos"))]
enum LauncherWait {
    /// The launcher exited; carries its exit status and captured stderr.
    Exited(std::process::ExitStatus, String),
    /// Still running when the deadline passed. The child is left running:
    /// killing it here would close the Automation prompt the user may
    /// still be answering, so the launch is reported as in progress.
    StillRunning,
    /// The wait itself failed (the launcher could not be reaped); carries
    /// the reason.
    Unknown(String),
}

/// Wait for a launcher child, capturing its stderr, up to `timeout`.
/// The wait runs on a helper thread because `wait_with_output` blocks;
/// on timeout the helper keeps waiting in the background and the caller
/// treats the launch as in progress.
#[cfg(any(test, target_os = "macos"))]
fn await_launcher_exit(child: Child, timeout: Duration) -> LauncherWait {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => LauncherWait::Exited(
            output.status,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Ok(Err(e)) => LauncherWait::Unknown(format!("failed to wait for the launcher: {e}")),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => LauncherWait::StillRunning,
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            LauncherWait::Unknown("launcher wait ended without a result".into())
        }
    }
}

/// Spawn a terminal launcher and report its outcome.
///
/// A successful spawn is not a successful launch: on macOS an Automation
/// denial makes `osascript` exit non-zero *after* a successful spawn, and
/// returning `Ok` at spawn time hid that failure from the settings card
/// (issue #8718). So this waits, bounded by `timeout`, for the launcher to
/// exit: exit zero is success, a non-zero exit is an error carrying the
/// exit status and the launcher's stderr, and a launcher still running at
/// the deadline is treated as a launch in progress (success — see
/// `LauncherWait::StillRunning`).
#[cfg(any(test, target_os = "macos"))]
fn run_launcher(
    mut command: Command,
    timeout: Duration,
    terminal_name: &str,
) -> Result<String, String> {
    command.stdout(Stdio::null()).stderr(Stdio::piped());
    let child = command
        .spawn()
        .map_err(|e| format!("failed to open {terminal_name}: {e}"))?;
    match await_launcher_exit(child, timeout) {
        LauncherWait::Exited(status, _stderr) if status.success() => Ok(terminal_name.into()),
        LauncherWait::Exited(status, stderr) => Err(format!(
            "{terminal_name} launch failed (launcher exited {status}): {}",
            stderr.trim()
        )),
        LauncherWait::StillRunning => Ok(terminal_name.into()),
        LauncherWait::Unknown(reason) => Err(format!(
            "could not confirm {terminal_name} launch: {reason}"
        )),
    }
}

/// Linux terminal emulators to try, in order, with the args each one needs.
#[cfg(any(test, target_os = "linux"))]
fn linux_launch_candidates() -> Vec<(&'static str, Vec<String>)> {
    let argv: Vec<String> = CLAUDE_LOGIN_ARGV.iter().map(|s| (*s).to_string()).collect();
    let line = login_command_line();
    vec![
        ("x-terminal-emulator", {
            let mut a = vec!["-e".to_string()];
            a.extend(argv.clone());
            a
        }),
        ("gnome-terminal", {
            let mut a = vec!["--".to_string()];
            a.extend(argv.clone());
            a
        }),
        ("konsole", {
            let mut a = vec!["-e".to_string()];
            a.extend(argv.clone());
            a
        }),
        // xfce4-terminal takes the command as a single string, not as argv.
        ("xfce4-terminal", vec!["-e".to_string(), line]),
        ("xterm", {
            let mut a = vec!["-e".to_string()];
            a.extend(argv);
            a
        }),
    ]
}

/// Open the user's native terminal and run `claude auth login` inside it.
///
/// Returns the name of the terminal emulator we launched (for UI
/// confirmation) or an error string if no terminal could be opened.
///
/// Platform behaviour:
///   - Windows: `cmd /c start "" cmd /k claude auth login --claudeai`
///   - macOS: `osascript` → Terminal.app `do script "claude auth login --claudeai"`
///   - Linux: try these terminal emulators in order:
///     `x-terminal-emulator`, `gnome-terminal`, `konsole`,
///     `xfce4-terminal`, `xterm`
///
/// On macOS the launcher's exit status is checked (bounded wait), so an
/// Automation denial is reported instead of a successful spawn being
/// reported as a successful launch (issue #8718). A launcher still running
/// at the deadline is treated as a launch in progress, not a failure.
#[tauri::command]
pub fn claude_code_login_launch() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(windows_launch_args())
            .spawn()
            .map_err(|e| format!("failed to open cmd: {e}"))?;
        Ok("cmd".into())
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("osascript");
        cmd.args(["-e", &macos_launch_script()]);
        run_launcher(cmd, MACOS_LAUNCH_WAIT, "Terminal.app")
    }

    #[cfg(target_os = "linux")]
    {
        for (term, args) in linux_launch_candidates() {
            match Command::new(term).args(&args).spawn() {
                Ok(_) => return Ok(term.to_string()),
                Err(_) => continue,
            }
        }
        Err(format!(
            "no terminal emulator found (tried x-terminal-emulator, gnome-terminal, konsole, \
             xfce4-terminal, xterm). Run `{}` manually.",
            login_command_line()
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("claude_code_login_launch is not supported on this platform".into())
    }
}

#[cfg(test)]
#[path = "claude_code_tests.rs"]
mod tests;
