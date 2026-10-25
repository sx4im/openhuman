use super::*;
use tokio::test;

// ── is_public_bind ───────────────────────────────────────

#[test]
async fn localhost_variants_not_public() {
    assert!(!is_public_bind("127.0.0.1"));
    assert!(!is_public_bind("localhost"));
    assert!(!is_public_bind("::1"));
    assert!(!is_public_bind("[::1]"));
}

#[test]
async fn zero_zero_is_public() {
    assert!(is_public_bind("0.0.0.0"));
}

#[test]
async fn real_ip_is_public() {
    assert!(is_public_bind("192.168.1.100"));
    assert!(is_public_bind("10.0.0.1"));
}

// ── Core RPC bind token ──────────────────────────────────

#[test]
async fn public_bind_without_env_token_auto_generates() {
    let tmp = std::env::temp_dir().join(format!("pairing-bind-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let token = ensure_core_rpc_token_for_bind("0.0.0.0", &tmp, None)
        .expect("public bind should auto-generate a token")
        .expect("public bind should return Some(token)");
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    let from_disk = std::fs::read_to_string(tmp.join("core.token")).unwrap();
    assert_eq!(from_disk, token);
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
async fn public_bind_rejects_explicit_empty_env_token() {
    let tmp = std::env::temp_dir().join(format!("pairing-bind-empty-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let err = ensure_core_rpc_token_for_bind("0.0.0.0", &tmp, Some("   "))
        .expect_err("empty env token on public bind must fail");
    assert!(matches!(err, CoreBindTokenError::EmptyEnvToken { .. }));
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
async fn loopback_bind_without_env_token_returns_none() {
    let tmp = std::env::temp_dir().join(format!("pairing-bind-loopback-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let token = ensure_core_rpc_token_for_bind("127.0.0.1", &tmp, None).unwrap();
    assert!(token.is_none());
    assert!(!tmp.join("core.token").exists());
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
async fn public_bind_uses_nonempty_env_token_without_writing_file() {
    let tmp = std::env::temp_dir().join(format!("pairing-bind-env-token-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let token = ensure_core_rpc_token_for_bind("0.0.0.0", &tmp, Some("  abc123  "))
        .unwrap()
        .expect("env token should be returned");
    assert_eq!(token, "abc123");
    assert!(!tmp.join("core.token").exists());
    std::fs::remove_dir_all(&tmp).ok();
}

/// `mode(0o600)` only applies on create: a token rewritten over an existing
/// world-readable file must still end up owner-only.
#[cfg(unix)]
#[test]
async fn rewriting_the_token_tightens_an_existing_file_to_owner_only() {
    use std::os::unix::fs::PermissionsExt as _;
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("core.token");
    std::fs::write(&path, "old").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    write_core_token_file(&path, "new-token").unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "token file left at {mode:o}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new-token");
}

/// A symlink planted at the token path must not redirect the bearer token.
#[cfg(unix)]
#[test]
async fn the_token_is_not_written_through_a_symlink() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("elsewhere");
    std::fs::write(&target, "untouched").unwrap();
    let path = tmp.path().join("core.token");
    std::os::unix::fs::symlink(&target, &path).unwrap();

    assert!(write_core_token_file(&path, "secret-token").is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "untouched");
}
