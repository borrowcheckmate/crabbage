//! The images directory under the data directory: writing an image under a new name, reading it and
//! removing it.

use std::io;
use std::path::{Path, PathBuf};

use super::Kind;
use super::naming;

const DIR: &str = "images";

fn path(data_dir: &Path, name: &str) -> PathBuf {
    data_dir.join(DIR).join(name)
}

/// Stores `bytes` as a new image of type `kind` and returns its file name. The bytes are written to a
/// temporary name in the same directory and renamed into place, so a half-written file is never read.
///
/// # Errors
///
/// Returns an error if the directory cannot be created, the file cannot be written or the random name
/// cannot be made.
pub(crate) async fn write(data_dir: &Path, bytes: &[u8], kind: Kind) -> io::Result<String> {
    let dir = data_dir.join(DIR);
    tokio::fs::create_dir_all(&dir).await?;
    let name = naming::generate(kind).map_err(io::Error::other)?;
    let temporary = dir.join(format!(".{name}.part"));
    tokio::fs::write(&temporary, bytes).await?;
    tokio::fs::rename(&temporary, dir.join(&name)).await?;
    Ok(name)
}

/// The bytes of the image `name`.
///
/// # Errors
///
/// Returns an error if the file cannot be read, including when it does not exist.
pub(crate) async fn read(data_dir: &Path, name: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path(data_dir, name)).await
}

/// Deletes the image `name`.
///
/// # Errors
///
/// Returns an error if the file cannot be deleted.
pub(crate) async fn remove(data_dir: &Path, name: &str) -> io::Result<()> {
    tokio::fs::remove_file(path(data_dir, name)).await
}
