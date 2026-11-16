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

/// The macOS launcher must report a failed launch instead of treating a
/// successful spawn as a successful launch (#8718): `osascript` exits
/// non-zero when Automation is denied, and a spawn that succeeds says
/// nothing about whether Terminal opened. Unix-only: the stand-in programs
/// below do not exist on Windows.
#[cfg(unix)]
#[tokio::test]
async fn macos_launcher_reports_nonzero_exit() {
    let err = run_launcher("/usr/bin/false", &[], Duration::from_secs(5))
        .await
        .expect_err("a non-zero exit must be an error");
    assert!(
        err.contains("exited with code 1"),
        "unexpected error: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn macos_launcher_reports_missing_executable() {
    let err = run_launcher("/definitely/not/a/launcher", &[], Duration::from_secs(5))
        .await
        .expect_err("a missing launcher must be an error");
    assert!(err.contains("failed to start"), "unexpected error: {err}");
}

#[cfg(unix)]
#[tokio::test]
async fn macos_launcher_times_out_and_abandons_the_launch() {
    let err = run_launcher("/bin/sleep", &["30"], Duration::from_millis(50))
        .await
        .expect_err("a launcher that never exits must time out");
    assert!(
        err.contains("did not exit within"),
        "unexpected error: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn macos_launcher_accepts_zero_exit() {
    run_launcher("/usr/bin/true", &[], Duration::from_secs(5))
        .await
        .expect("a zero exit is a successful launch");
}

/// The permission half of #8718: the command must be listed in the app's
/// permission files and granted to the main window, or the webview invoke is
/// rejected before Rust runs ("Command not found"). This guards the entry
/// added to `allow-core-process.toml` by 52654d77 against being dropped in
/// a future permission cleanup.
#[test]
fn login_launch_is_permitted_in_the_main_window() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let capability: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{manifest_dir}/capabilities/default.json"))
            .expect("capabilities/default.json must exist"),
    )
    .expect("capabilities/default.json must be valid JSON");
    let permissions = capability["permissions"]
        .as_array()
        .expect("capability must list permissions");
    assert!(
        permissions
            .iter()
            .any(|p| p.as_str() == Some("allow-core-process")),
        "the default capability must grant allow-core-process to the main window"
    );
    let permission: toml::Value = std::fs::read_to_string(format!(
        "{manifest_dir}/permissions/allow-core-process.toml"
    ))
    .expect("permissions/allow-core-process.toml must exist")
    .parse()
    .expect("allow-core-process.toml must be valid TOML");
    let allow = permission
        .get("permission")
        .and_then(|p| p.get("commands"))
        .and_then(|c| c.get("allow"))
        .and_then(|a| a.as_array())
        .expect("allow-core-process must list allowed commands");
    assert!(
        allow
            .iter()
            .any(|c| c.as_str() == Some("claude_code_login_launch")),
        "allow-core-process must list claude_code_login_launch"
    );
}
