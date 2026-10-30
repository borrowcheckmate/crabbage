//! The file names of stored images: 32 lowercase hex characters, a dot and the extension of the type.
//! Only names of this form are ever read from the images directory.

use super::Kind;

const RANDOM_BYTES: usize = 16;
const STEM_LEN: usize = 32;

/// A new random file name for an image of type `kind`.
///
/// # Errors
///
/// Returns an error if the system random source fails.
pub(crate) fn generate(kind: Kind) -> Result<String, getrandom::Error> {
    let mut bytes = [0_u8; RANDOM_BYTES];
    getrandom::fill(&mut bytes)?;
    let stem: String = bytes
        .iter()
        .flat_map(|&byte| [byte >> 4, byte & 0x0F])
        .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
        .collect();
    Ok(format!("{stem}.{}", kind.extension()))
}

/// The type of the image stored under `name`, or `None` when `name` is not a file name this app
/// generates. Anything else, such as a path, is refused before the disk is touched.
pub(crate) fn kind_of(name: &str) -> Option<Kind> {
    let (stem, extension) = name.split_once('.')?;
    let well_formed = stem.len() == STEM_LEN
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if well_formed {
        Kind::from_extension(extension)
    } else {
        None
    }
}
