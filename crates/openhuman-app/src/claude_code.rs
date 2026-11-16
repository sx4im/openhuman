//! Tauri commands for the Claude Code CLI provider.
//!
//! Provides a cross-platform "open a terminal and run `claude auth login`"
//! helper. The CLI's OAuth flow is interactive (it prints a URL and
//! waits for the user to paste a code), so we can't host it in-app — we
//! detach into the user's native terminal so they complete login there,
//! then return to OpenHuman and click Recheck in the settings card.

use std::process::Command;
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

/// How long the macOS launcher may take to hand the script to Terminal.app
/// before we give up. `osascript` normally exits as soon as Terminal accepts
/// the `do script`; a longer wait means the Automation prompt is unanswered
/// (or Terminal is wedged), and the sign-in card must say so instead of
/// hanging on the invoke.
const MACOS_LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);

/// Run a launcher program and report its exit status.
///
/// A non-zero exit means the terminal never opened — for `osascript`, the
/// usual cause is a denied macOS Automation prompt — and must surface as an
/// error. The old code returned `Ok` as soon as the process spawned, so a
/// denied launch looked like a working Reconnect button (#8718). A launcher
/// that does not exit within `timeout` is killed and reported as a timeout,
/// so an unanswered prompt cannot hang the invoke forever.
///
/// Compiled on macOS and in tests, so a Linux CI run still exercises the
/// wait/timeout paths with stand-in commands.
#[cfg(any(test, target_os = "macos"))]
async fn run_launcher(program: &str, args: &[&str], timeout: Duration) -> Result<(), String> {
    let mut child = tokio::process::Command::new(program)
        .args(args)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to start {program}: {e}"))?;
    let status = tokio::time::timeout(timeout, child.wait())
        .await
        .map_err(|_| {
            format!(
                "{program} did not exit within {}s; the launch was abandoned",
                timeout.as_secs()
            )
        })?
        .map_err(|e| format!("failed while waiting for {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else if let Some(code) = status.code() {
        Err(format!(
            "{program} exited with code {code}; the terminal did not open"
        ))
    } else {
        Err(format!(
            "{program} was terminated by a signal; the terminal did not open"
        ))
    }
}

/// Open the user's native terminal and run `claude auth login` inside it.
///
/// Returns the name of the terminal emulator we launched (for UI
/// confirmation) or an error string if no terminal could be opened.
///
/// Platform behaviour:
///   - Windows: `cmd /c start "" cmd /k claude auth login --claudeai`
///   - macOS:   `osascript` → Terminal.app `do script "claude auth login --claudeai"`,
///              awaiting the launch result (up to 30s) instead of treating a
///              successful spawn as a successful launch
///   - Linux:   try `x-terminal-emulator`, then `gnome-terminal`,
///              `konsole`, `xfce4-terminal`, `xterm` in that order
#[tauri::command]
pub async fn claude_code_login_launch() -> Result<String, String> {
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
        let script = macos_launch_script();
        run_launcher("osascript", &["-e", &script], MACOS_LAUNCH_TIMEOUT).await?;
        Ok("Terminal.app".into())
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
