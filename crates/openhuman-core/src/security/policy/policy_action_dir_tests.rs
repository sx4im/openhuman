//! The configured `action_dir` as a granted read-write trusted root.
//!
//! Split out of `policy_trusted_roots_tests.rs` only for file size; the subject
//! is the same grant, and the fixtures there deliberately set
//! `action_dir == workspace_dir`, which is exactly the shape that hid this.

use super::*;
use std::fs;

/// The configured `action_dir` must be a read-write trusted root.
///
/// It is the base `validate_path` joins every relative tool path onto, but it
/// was never itself an allow root: the write permission came from the
/// `default_projects_dir()` grant above, which reads `OPENHUMAN_PROJECTS_DIR`
/// and knows nothing about `OPENHUMAN_ACTION_DIR` / `action_dir_override`. On a
/// stock install the two coincide, which is why every existing fixture in this
/// file (`action_dir == workspace_dir`) missed it. Point them apart — which is
/// what the Settings working-folder control does — and a write into the
/// agent's own working directory was refused "Resolved path escapes
/// workspace".
#[test]
fn from_config_grants_the_configured_action_dir_as_readwrite_root() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("elsewhere/action");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&action).unwrap();

    let cfg = crate::config::AutonomyConfig::default();
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);
    let action_str = action.to_string_lossy().to_string();
    assert!(
        policy
            .trusted_roots
            .iter()
            .any(|r| r.path == action_str && matches!(r.access, TrustedAccess::ReadWrite)),
        "from_config must grant the action dir as read-write; got: {:?}",
        policy.trusted_roots
    );
}

/// The grant must reach the gate every file write funnels through, not just the
/// `trusted_roots` vector. Built with the policy explicitly enabled, since the
/// containment this proves is exactly what `enabled = false` turns off.
#[tokio::test]
async fn an_enabled_policy_can_write_into_its_action_dir_outside_the_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("elsewhere/action");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(action.join("out")).unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);
    let resolved = policy
        .validate_parent_path("out/plan.md")
        .await
        .expect("a relative write into the action dir must be permitted");
    assert!(resolved.ends_with("out/plan.md"), "got {resolved:?}");
}

/// An action dir at or above `workspace_dir` is NOT granted: a trusted root
/// there would hand the whole workspace a `forbidden_paths` bypass
/// (TinySecurity's path policy allows trusted roots to override `forbidden_paths`).
#[test]
fn from_config_does_not_grant_an_action_dir_covering_the_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let action = tmp.path().to_path_buf();
    let workspace = action.join("workspace");
    fs::create_dir_all(&workspace).unwrap();

    let cfg = crate::config::AutonomyConfig::default();
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);
    let action_str = action.to_string_lossy().to_string();
    assert!(
        !policy.trusted_roots.iter().any(|r| r.path == action_str),
        "an action dir above the workspace must not be granted; got: {:?}",
        policy.trusted_roots
    );
}

/// Oversized tool outputs are saved under `<workspace>/artifacts/tool-results`
/// and pointed at by absolute path. Under `workspace_only` an absolute path is
/// refused unless a trusted root covers it, so `from_config` grants that one
/// directory — read-only — and nothing else under the workspace.
// Fails closed only while security is enforced: a build without the
// security module has security disabled (`crate::security::switch`).
#[cfg(feature = "security-module")]
#[tokio::test]
async fn tool_result_artifacts_are_readable_by_absolute_path_under_workspace_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    let artifacts = super::tool_result_artifacts_dir(&workspace).join("s1/shell");
    fs::create_dir_all(&artifacts).unwrap();
    fs::create_dir_all(&action).unwrap();
    fs::write(artifacts.join("call.txt"), "output").unwrap();
    let record = workspace.join("artifacts/abc");
    fs::create_dir_all(&record).unwrap();
    fs::write(record.join("meta.json"), "{}").unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    let grant = super::tool_result_artifacts_dir(&workspace)
        .to_string_lossy()
        .to_string();
    assert!(policy
        .trusted_roots
        .iter()
        .any(|r| r.path == grant && r.access == TrustedAccess::Read));

    let result = artifacts.join("call.txt");
    policy
        .validate_path(&result.to_string_lossy())
        .await
        .expect("a tool-result artifact must be readable by absolute path");
    // The grant is that directory only: internal artifact records stay closed.
    assert!(policy
        .validate_path(&record.join("meta.json").to_string_lossy())
        .await
        .is_err());
}

/// A relative action dir is still the agent's working root (tool paths are
/// resolved against it), so it is granted as its absolute form; a relative
/// trusted root itself grants nothing.
#[test]
fn from_config_grants_a_relative_action_dir_as_its_absolute_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let action = std::path::Path::new("relative-action-dir");

    let policy = SecurityPolicy::from_config(
        &crate::config::AutonomyConfig::default(),
        &workspace,
        action,
    );
    let absolute = std::path::absolute(action)
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert!(
        policy
            .trusted_roots
            .iter()
            .any(|r| r.path == absolute && matches!(r.access, TrustedAccess::ReadWrite)),
        "relative action dir not granted as {absolute}; got: {:?}",
        policy.trusted_roots
    );
}

/// Non-absolute configured roots are dropped from the policy, so nothing
/// (the turn prompt included) presents them as grants.
#[test]
fn from_config_drops_non_absolute_trusted_roots() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg = crate::config::AutonomyConfig {
        trusted_roots: ["", ".", "rel/dir"]
            .into_iter()
            .map(|path| TrustedRoot {
                path: path.into(),
                access: TrustedAccess::ReadWrite,
            })
            .collect(),
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, tmp.path(), tmp.path());
    assert!(
        policy
            .trusted_roots
            .iter()
            .all(|r| std::path::Path::new(&r.path).is_absolute()),
        "non-absolute roots kept: {:?}",
        policy.trusted_roots
    );
}

/// A relative path to an offloaded tool result resolves against the
/// workspace artifact store when the action directory has no such file
/// (#8643). The offload preview hands the model an absolute `artifact_path`,
/// but the model may repeat the workspace-relative spelling instead; the
/// read then fails under the action directory and the repeated-failure
/// circuit breaker can stop the turn. The tool-results store is already a
/// read-only trusted root, so only that spelling falls back — the
/// workspace/action-directory separation (#3051) stands.
#[cfg(feature = "security-module")]
#[tokio::test]
async fn relative_tool_result_artifact_path_resolves_against_the_workspace_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    let artifacts = super::tool_result_artifacts_dir(&workspace).join("s1/shell");
    fs::create_dir_all(&artifacts).unwrap();
    fs::create_dir_all(&action).unwrap();
    fs::write(artifacts.join("call.txt"), "output").unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    let resolved = policy
        .validate_path("artifacts/tool-results/s1/shell/call.txt")
        .await
        .expect("a relative path to an existing tool-result artifact must resolve");
    assert!(
        resolved.ends_with("artifacts/tool-results/s1/shell/call.txt"),
        "got {resolved:?}"
    );
}

/// A relative tool-result path that exists under the action directory keeps
/// resolving there, even when the same spelling exists under the workspace
/// store (#8643).
#[cfg(feature = "security-module")]
#[tokio::test]
async fn relative_tool_result_path_prefers_the_action_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    let action_artifacts = action.join("artifacts/tool-results/s1/shell");
    let workspace_artifacts = super::tool_result_artifacts_dir(&workspace).join("s1/shell");
    fs::create_dir_all(&action_artifacts).unwrap();
    fs::create_dir_all(&workspace_artifacts).unwrap();
    fs::write(action_artifacts.join("call.txt"), "action").unwrap();
    fs::write(workspace_artifacts.join("call.txt"), "workspace").unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    let resolved = policy
        .validate_path("artifacts/tool-results/s1/shell/call.txt")
        .await
        .expect("a relative path present under the action dir must resolve");
    assert_eq!(
        fs::read_to_string(&resolved).unwrap(),
        "action",
        "the action directory must win over the workspace store; got {resolved:?}"
    );
}

/// A symlink inside the tool-results store that escapes it is refused: the
/// fallback resolves through canonicalization and checks containment (#8643).
#[cfg(feature = "security-module")]
#[tokio::test]
async fn relative_tool_result_symlink_escaping_the_store_is_refused() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("secret.txt"), "secret").unwrap();
    let artifacts = super::tool_result_artifacts_dir(&workspace).join("s1/shell");
    fs::create_dir_all(&artifacts).unwrap();
    fs::create_dir_all(&action).unwrap();
    std::os::unix::fs::symlink(outside.join("secret.txt"), artifacts.join("call.txt")).unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    assert!(policy
        .validate_path("artifacts/tool-results/s1/shell/call.txt")
        .await
        .is_err());
}

/// A relative tool-result path that exists nowhere still fails instead of
/// resolving to an empty or invented location (#8643).
#[cfg(feature = "security-module")]
#[tokio::test]
async fn relative_tool_result_path_missing_everywhere_still_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    fs::create_dir_all(super::tool_result_artifacts_dir(&workspace)).unwrap();
    fs::create_dir_all(&action).unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    assert!(policy
        .validate_path("artifacts/tool-results/s1/shell/call.txt")
        .await
        .is_err());
}

/// The fallback covers only the tool-results store: a relative spelling of
/// another internal artifact record stays refused (#8643, #3051).
#[cfg(feature = "security-module")]
#[tokio::test]
async fn relative_internal_artifact_path_does_not_fall_back_to_the_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let action = tmp.path().join("project");
    let record = workspace.join("artifacts/abc");
    fs::create_dir_all(&record).unwrap();
    fs::create_dir_all(&action).unwrap();
    fs::write(record.join("meta.json"), "{}").unwrap();

    let cfg = crate::config::AutonomyConfig {
        enabled: true,
        workspace_only: true,
        ..crate::config::AutonomyConfig::default()
    };
    let policy = SecurityPolicy::from_config(&cfg, &workspace, &action);

    assert!(policy
        .validate_path("artifacts/abc/meta.json")
        .await
        .is_err());
}
