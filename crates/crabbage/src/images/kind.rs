//! The image types the app stores, told apart by their first bytes. The file name and the client's
//! content type are never trusted.

const JPEG_SIGNATURE: &[u8] = &[0xFF, 0xD8, 0xFF];
const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// An image type the app stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// JPEG.
    Jpeg,
    /// PNG.
    Png,
    /// WebP.
    Webp,
}

impl Kind {
    /// The type of the image in `bytes`, from its signature, or `None` for any other file.
    pub(crate) fn detect(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(JPEG_SIGNATURE) {
            Some(Self::Jpeg)
        } else if bytes.starts_with(PNG_SIGNATURE) {
            Some(Self::Png)
        } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()) {
            Some(Self::Webp)
        } else {
            None
        }
    }

    /// The file name extension of the type.
    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }

    /// The media type the type is served with.
    pub(crate) const fn content_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
        }
    }

    /// The type with file name extension `extension`.
    pub(crate) fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "jpg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }
}
