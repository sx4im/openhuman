//! Test-only storage backends backed by the real TSD testkit and drivers.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

#[cfg(feature = "storage-sqlite")]
use crate::storage::block_on_anyhow;
use crate::storage::StorageBackend;

/// A real TSD in-memory backend for tests that do not require conditional
/// writes or persistence across backend instances.
pub fn memory_backend() -> Arc<dyn StorageBackend> {
    Arc::new(tinystoragedrivers_testkit::MemoryBackend::new())
}

static TEST_WORKSPACES: OnceLock<Mutex<Vec<tempfile::TempDir>>> = OnceLock::new();

/// Returns a private workspace path retained until the test process exits.
/// Use it when a test context can open durable storage: sharing the system
/// temp directory lets unrelated tests declare conflicting storage indexes.
pub fn isolated_workspace() -> PathBuf {
    let directory = tempfile::tempdir().expect("create isolated test workspace");
    let path = directory.path().to_path_buf();
    TEST_WORKSPACES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(directory);
    path
}

/// The in-memory backend a workspace opens by default under `cfg(test)`.
///
/// The TSD memory engine cannot evaluate a write condition, yet the session
/// transcripts (lease-conditioned generation writes) and the state import
/// claims ask every backend for a conditioned scope. In a unit test the one
/// process is the only writer, so there is nothing to fence: this wrapper
/// opens the plain handles for a conditioned scope. It exists only for the
/// workspace default; tests of the conditions themselves use
/// [`sqlite_backend`], which enforces them.
#[derive(Debug)]
pub(crate) struct UnfencedMemoryBackend(tinystoragedrivers_testkit::MemoryBackend);

impl UnfencedMemoryBackend {
    pub(crate) fn new() -> Self {
        Self(tinystoragedrivers_testkit::MemoryBackend::new())
    }
}

impl StorageBackend for UnfencedMemoryBackend {
    fn capabilities(&self) -> tinystoragedrivers_types::Capabilities {
        self.0.capabilities()
    }

    fn for_scope(
        &self,
        scope: &tinystoragedrivers::Scope,
    ) -> Result<tinystoragedrivers::Storage, tinystoragedrivers::StorageError> {
        self.0.for_scope(scope)
    }

    fn for_scope_conditioned(
        &self,
        scope: &tinystoragedrivers::Scope,
        _condition: &tinystoragedrivers_types::WriteCondition,
    ) -> Result<tinystoragedrivers::Storage, tinystoragedrivers::StorageError> {
        self.0.for_scope(scope)
    }
}

static STORAGE_SLOT_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Serializes a test that reads or mutates the process-wide installed storage
/// slot. Hold this guard for the full test lifetime, including cleanup.
pub fn lock_storage_slot() -> MutexGuard<'static, ()> {
    STORAGE_SLOT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A temporary installation in the process storage slot, restored on drop.
/// Use only in tests that exercise boot/import paths reading `installed_url`.
pub struct InstalledBackendGuard {
    _slot_lock: MutexGuard<'static, ()>,
    previous_backend: Option<Arc<dyn StorageBackend>>,
    previous_url: Option<String>,
}

/// Installs `backend` with its URL for a test that exercises the production
/// boot path, restoring the previous process slot when the guard is dropped.
pub fn install_for_test(url: &str, backend: Arc<dyn StorageBackend>) -> InstalledBackendGuard {
    let slot_lock = lock_storage_slot();
    let previous_url = crate::storage::installed_url();
    let previous_backend = crate::storage::install_for_url(url, backend);
    InstalledBackendGuard {
        _slot_lock: slot_lock,
        previous_backend,
        previous_url,
    }
}

/// [`install_for_test`] for a backend opened from no URL (memory): a test
/// that needs storage in the slot without exercising the boot path.
pub fn install_backend_for_test(backend: Arc<dyn StorageBackend>) -> InstalledBackendGuard {
    let slot_lock = lock_storage_slot();
    let previous_url = crate::storage::installed_url();
    let previous_backend = crate::storage::install(backend);
    InstalledBackendGuard {
        _slot_lock: slot_lock,
        previous_backend,
        previous_url,
    }
}

impl Drop for InstalledBackendGuard {
    fn drop(&mut self) {
        match (self.previous_backend.take(), self.previous_url.take()) {
            (Some(backend), Some(url)) => {
                crate::storage::install_for_url(&url, backend);
            }
            (Some(backend), None) => {
                crate::storage::install(backend);
            }
            (None, _) => {
                crate::storage::clear();
            }
        }
    }
}

/// An on-disk SQLite backend whose temporary directory remains alive as long
/// as the fixture is retained. Use it for persistence, conditional-write,
/// fence, and import-claim tests; the TSD memory backend does not emulate those
/// capabilities.
#[cfg(feature = "storage-sqlite")]
pub struct SqliteBackend {
    _directory: tempfile::TempDir,
    /// The actual SQLite backend opened from [`Self::url`].
    pub backend: Arc<dyn StorageBackend>,
    /// URL suitable for configuring the same isolated SQLite database.
    pub url: String,
}

/// Opens an isolated on-disk SQLite backend for tests that need its real
/// cross-process conditions or durable state.
#[cfg(feature = "storage-sqlite")]
pub fn sqlite_backend() -> anyhow::Result<SqliteBackend> {
    let directory = tempfile::tempdir()?;
    let url = format!("sqlite:{}", directory.path().join("storage").display());
    sqlite_backend_in(directory, url)
}

/// Opens a SQLite backend at `url`, retaining `directory` for the lifetime of
/// the returned backend fixture.
#[cfg(feature = "storage-sqlite")]
pub fn sqlite_backend_in(
    directory: tempfile::TempDir,
    url: impl Into<String>,
) -> anyhow::Result<SqliteBackend> {
    let url = url.into();
    let backend = block_on_anyhow({
        let url = url.clone();
        async move { crate::storage::open(&url).await.map_err(Into::into) }
    })?;
    Ok(SqliteBackend {
        _directory: directory,
        backend,
        url,
    })
}
