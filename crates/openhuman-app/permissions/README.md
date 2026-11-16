# permissions

Tauri v2 permission sets for the app's own commands (the ones registered in
`generate_handler!` in [`src/lib.rs`](../src/lib.rs)). Each `.toml` file defines one
`[[permission]]` with an `identifier` and a `[permission.commands] allow`
list. The capability in [`../capabilities/default.json`](../capabilities/README.md)
grants all of them to the `main` window by identifier.

## How it works

`tauri_build::build()` (called from [`build.rs`](../build.rs)) reads this directory into the
app's ACL manifest under the `__app-acl__` key; `build.rs` reruns when the
directory changes. Once that manifest exists, Tauri checks app commands
against it on every invoke, not only plugin commands. A command the webview
calls is allowed only if some permission granted by a capability lists it.
Otherwise the invoke fails with "`<command>` not allowed. Command not found",
even though the command is registered.

```text
renderer invoke('gateway_list')
   |
   v
RuntimeAuthority::resolve_access
   |  capability "default" (window: main)
   |    -> allow-core-process, allow-app-update, ... (these files)
   |  command listed in one of them?  yes -> run handler
   v                                   no  -> "not allowed. Command not found"
```

## Layout

| File | Identifier | Commands |
| --- | --- | --- |
| [`allow-core-process.toml`](allow-core-process.toml) | `allow-core-process` | Core endpoint and lifecycle (`core_rpc_*`, `relay_http_rpc`, `relay_remote_socket`, `start_core_process`, `restart_core_process`, `reset_local_data`), app control (`restart_app`, `app_quit`), session (`get_active_user_id`, `auth_*`), native notifications, log folder commands, gateways, core update, port recovery, MCP settings, and the Claude Code login launcher (`claude_code_login_launch`). |
| [`allow-app-update.toml`](allow-app-update.toml) | `allow-app-update` | `check_app_update`, `apply_app_update`, `download_app_update`, `install_app_update`. |
| [`allow-workspace-files.toml`](allow-workspace-files.toml) | `allow-workspace-files` | `open_workspace_path`, `reveal_workspace_path`, `preview_workspace_text`. |
| [`allow-artifact-download.toml`](allow-artifact-download.toml) | `allow-artifact-download` | `download_artifact_to_downloads`, `save_artifact_as`. |
| [`allow-directory-picker.toml`](allow-directory-picker.toml) | `allow-directory-picker` | `pick_directory_via_dialog`. |
| [`allow-loopback-oauth.toml`](allow-loopback-oauth.toml) | `allow-loopback-oauth` | `start_loopback_oauth_listener`, `stop_loopback_oauth_listener`. Kept separate so a consumer of `allow-core-process` does not also get OAuth listener control. |

## Adding a command

1. Register it in `generate_handler!` in [`src/lib.rs`](../src/lib.rs).
2. Add its name to the `allow` list of the permission file that fits, or add
   a new file and reference its identifier from [`capabilities/default.json`](../capabilities/default.json).
3. Rebuild. Step 2 is the one that is easy to miss: the build succeeds and the
   call only fails at runtime.

## Gotchas

These files and `generate_handler!` drifted apart before: commands such as
`claude_code_login_launch` were registered but listed in no permission file,
so the webview invoke was rejected with "Command not found" even though the
handler existed (#8718; the entry landed in `allow-core-process.toml` in
52654d77). The lists have since been reconciled — every command in
`generate_handler!` is listed in one of these files, and the
`login_launch_is_permitted_in_the_main_window` test in
`src/claude_code_tests.rs` guards the Claude Code login command against
regressing. Re-check the rest against `src/lib.rs` when adding a command:
step 2 in "Adding a command" is the one that is easy to miss, and the build
succeeds while the call only fails at runtime.

Plugin permissions (`core:*`, `notification:*`, `opener:*`, `deep-link:*`,
`updater:*`) are not defined here; they come from the plugins and are granted
directly in the capability.

## Further reading

- [`docs/gitbooks/en/developing/architecture/tauri-shell.md`](../../../docs/gitbooks/en/developing/architecture/tauri-shell.md): the Tauri shell.
- [`docs/gitbooks/en/developing/architecture/security.md`](../../../docs/gitbooks/en/developing/architecture/security.md): security.
- [`crates/openhuman-app/README.md`](../README.md): the openhuman-app crate README.
