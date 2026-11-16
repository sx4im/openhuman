use super::*;

/// Every launcher must reach the `auth` subcommand. `claude login` is not a
/// command — `claude` takes a positional `[prompt]`, so it silently became a
/// prompt instead of starting the OAuth flow.
fn assert_reaches_auth_login(rendered: &str, what: &str) {
    assert!(
        rendered.contains("claude auth login"),
        "{what} must invoke `claude auth login`, got: {rendered}"
    );
    assert!(
        !rendered.contains("claude login"),
        "{what} still constructs the obsolete `claude login`, got: {rendered}"
    );
}

#[test]
fn login_command_line_is_the_auth_subcommand() {
    assert_eq!(login_command_line(), "claude auth login --claudeai");
}

#[test]
fn argv_keeps_auth_and_login_as_separate_words() {
    // Split-argument terminals pass these straight to execvp, so `auth` and
    // `login` have to be distinct argv entries rather than one "auth login".
    assert_eq!(
        CLAUDE_LOGIN_ARGV,
        &["claude", "auth", "login", "--claudeai"]
    );
}

#[test]
fn windows_launcher_reaches_auth_login() {
    assert_reaches_auth_login(&windows_launch_args().join(" "), "the Windows launcher");
}

#[test]
fn windows_launcher_keeps_the_empty_start_title() {
    // `start` reads a bare first argument as the window title, which would
    // swallow `cmd` and open an empty shell.
    let args = windows_launch_args();
    assert_eq!(args[1], "start");
    assert_eq!(args[2], "", "the empty title placeholder must survive");
}

#[test]
fn macos_launcher_reaches_auth_login() {
    assert_reaches_auth_login(&macos_launch_script(), "the macOS AppleScript");
}

#[test]
fn macos_script_quotes_the_command_for_do_script() {
    assert!(macos_launch_script().contains(r#"do script "claude auth login --claudeai""#));
}

#[test]
fn every_linux_candidate_reaches_auth_login() {
    let candidates = linux_launch_candidates();
    assert_eq!(candidates.len(), 5, "all five emulators stay covered");
    for (term, args) in candidates {
        assert_reaches_auth_login(&args.join(" "), term);
    }
}

#[test]
fn xfce4_terminal_gets_one_string_and_the_others_get_argv() {
    let candidates = linux_launch_candidates();
    for (term, args) in candidates {
        match term {
            // -e plus a single command string
            "xfce4-terminal" => assert_eq!(args.len(), 2, "xfce4-terminal takes one string"),
            // separator plus four argv words
            _ => assert_eq!(args.len(), 5, "{term} takes the command as argv"),
        }
    }
}

// -------------------------------------------------------------------------
// macOS launcher result reporting (issue #8718)
// -------------------------------------------------------------------------

/// A launcher that exits non-zero must surface the failure instead of
/// reporting success merely because the spawn succeeded: on macOS an
/// Automation denial makes `osascript` exit non-zero *after* a successful
/// spawn, and the old code returned `Ok` as soon as the spawn succeeded,
/// so the settings card never showed the error and Terminal never opened.
#[test]
fn launcher_reports_a_failed_exit_status() {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "echo 'not allowed' >&2; exit 3"]);
    let err = run_launcher(cmd, Duration::from_secs(10), "Terminal.app")
        .expect_err("a non-zero launcher exit must be an error");
    assert!(
        err.contains('3'),
        "error must name the exit code, got: {err}"
    );
    assert!(
        err.contains("not allowed"),
        "error must carry the launcher stderr, got: {err}"
    );
}

#[test]
fn launcher_reports_success_when_the_launcher_exits_zero() {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "exit 0"]);
    assert_eq!(
        run_launcher(cmd, Duration::from_secs(10), "Terminal.app").as_deref(),
        Ok("Terminal.app")
    );
}

/// A launcher still running at the deadline (for example, the user is
/// still answering the macOS Automation prompt) must not be killed or
/// reported as failed: the invoke returns success and the launch
/// continues in the background.
#[test]
fn launcher_treats_a_still_running_launcher_as_launched() {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "sleep 2"]);
    assert_eq!(
        run_launcher(cmd, Duration::from_millis(100), "Terminal.app").as_deref(),
        Ok("Terminal.app")
    );
}

/// The bounded wait is thirty seconds: long enough for the user to answer
/// the macOS Automation prompt, short enough that a wedged launcher does
/// not hold the invoke forever.
#[test]
fn macos_launch_wait_is_thirty_seconds() {
    assert_eq!(MACOS_LAUNCH_WAIT, Duration::from_secs(30));
}

#[test]
fn launcher_reports_a_spawn_failure() {
    let cmd = Command::new("definitely-not-a-real-launcher-binary");
    let err = run_launcher(cmd, Duration::from_secs(5), "Terminal.app")
        .expect_err("a launcher that cannot spawn must be an error");
    assert!(err.contains("failed to open Terminal.app"), "got: {err}");
}

// -------------------------------------------------------------------------
// Permission wiring (issue #8718)
// -------------------------------------------------------------------------

/// Regression for #8718: the command was registered in `generate_handler!`
/// and invoked from the frontend, but no permission file listed it, so the
/// shipped 0.64.15 build rejected every invoke with "not allowed. Command
/// not found" while the build itself stayed green. This test reads the
/// shipped permission tables and fails if the command ever drops out of
/// them again.
#[test]
fn login_launch_is_permitted_for_the_main_window() {
    let permissions = include_str!("../permissions/allow-core-process.toml");
    let parsed: toml::Value = permissions
        .parse()
        .expect("allow-core-process.toml must parse");
    // `[[permission]]` is an array of tables, so walk each entry's
    // `[permission.commands] allow` list.
    let allowed = parsed
        .get("permission")
        .and_then(|p| p.as_array())
        .map(|entries| {
            entries.iter().any(|permission| {
                permission
                    .get("commands")
                    .and_then(|c| c.get("allow"))
                    .and_then(|a| a.as_array())
                    .map(|allow| {
                        allow
                            .iter()
                            .any(|entry| entry.as_str() == Some("claude_code_login_launch"))
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    assert!(
        allowed,
        "claude_code_login_launch must be listed in allow-core-process.toml \
         (see #8718: an unlisted command is rejected at runtime with \
         \"not allowed. Command not found\")"
    );

    let capability = include_str!("../capabilities/default.json");
    let value: serde_json::Value =
        serde_json::from_str(capability).expect("capabilities/default.json must parse");
    let targets_main = value
        .get("windows")
        .and_then(|w| w.as_array())
        .map(|windows| windows.iter().any(|entry| entry.as_str() == Some("main")))
        .unwrap_or(false);
    assert!(
        targets_main,
        "capabilities/default.json must target the main window"
    );
    let grants = value
        .get("permissions")
        .and_then(|p| p.as_array())
        .map(|entries| {
            entries.iter().any(|entry| {
                entry.as_str() == Some("allow-core-process")
                    || entry.get("identifier").and_then(|id| id.as_str())
                        == Some("allow-core-process")
            })
        })
        .unwrap_or(false);
    assert!(
        grants,
        "capabilities/default.json must grant allow-core-process to the main window"
    );
}
