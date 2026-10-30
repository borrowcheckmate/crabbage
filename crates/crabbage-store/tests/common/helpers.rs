use std::path::{Path, PathBuf};

use crabbage_store::open;
use sqlx::sqlite::SqlitePool;

/// A directory under the system temp dir that is removed when dropped.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    /// Creates an empty directory unique to this process and `name`.
    pub(crate) fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("crabbage-store-{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }

    /// The directory path.
    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

/// Opens a migrated database in a fresh temporary directory named after `name`.
pub(crate) async fn database(name: &str) -> (TempDir, SqlitePool) {
    let dir = TempDir::new(name);
    let pool = open(dir.path()).await.expect("database opens");
    (dir, pool)
}
