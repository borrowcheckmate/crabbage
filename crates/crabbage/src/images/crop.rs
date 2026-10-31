use std::io::Cursor;

use crabbage_import::ai::{AiPhoto, Image as SourceImage};
use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageDecoder, Limits};

const MAX_SIDE: u32 = 1600;
const MAX_DIMENSION: u32 = 20_000;

pub(crate) fn crop(images: &[SourceImage], photo: AiPhoto) -> Option<Vec<u8>> {
    let source = images.get(photo.page)?;
    if !valid(photo) {
        return None;
    }
    let mut reader = ImageReader::new(Cursor::new(&source.bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().ok()?;
    let orientation = decoder.orientation().ok()?;
    let mut image = DynamicImage::from_decoder(decoder).ok()?;
    image.apply_orientation(orientation);
    encode_crop(&image, photo)
}

fn valid(photo: AiPhoto) -> bool {
    [photo.left, photo.top, photo.width, photo.height]
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        && photo.width >= 0.15
        && photo.height >= 0.15
        && photo.left + photo.width <= 1.0
        && photo.top + photo.height <= 1.0
}

fn encode_crop(image: &DynamicImage, photo: AiPhoto) -> Option<Vec<u8>> {
    let (page_width, page_height) = image.dimensions();
    let x = pixel(page_width, photo.left, false)?;
    let y = pixel(page_height, photo.top, false)?;
    let width = pixel(page_width, photo.width, false)?;
    let height = pixel(page_height, photo.height, false)?;
    if width == 0
        || height == 0
        || x.saturating_add(width) > page_width
        || y.saturating_add(height) > page_height
    {
        return None;
    }
    let cropped = image.crop_imm(x, y, width, height);
    let longest = width.max(height);
    let output = if longest > MAX_SIDE {
        let scaled_width = pixel(MAX_SIDE, f64::from(width) / f64::from(longest), true)?;
        let scaled_height = pixel(MAX_SIDE, f64::from(height) / f64::from(longest), true)?;
        cropped.resize_exact(
            scaled_width.max(1),
            scaled_height.max(1),
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        cropped
    };
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, 85)
        .encode_image(&output)
        .ok()?;
    Some(bytes)
}

fn pixel(side: u32, fraction: f64, round: bool) -> Option<u32> {
    let value = f64::from(side) * fraction;
    let text = if round {
        value.round().to_string()
    } else {
        value.floor().to_string()
    };
    text.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;

    fn box_(left: f64, top: f64, width: f64, height: f64, page: usize) -> AiPhoto {
        AiPhoto {
            page,
            left,
            top,
            width,
            height,
        }
    }

    #[test]
    fn invalid_and_tiny_boxes_and_missing_pages_give_no_image() {
        let image = DynamicImage::new_rgb8(100, 100);
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("encode fixture");
        let images = [SourceImage {
            media_type: "image/png".to_owned(),
            bytes: bytes.into_inner(),
        }];
        for photo in [
            box_(-0.1, 0.0, 0.5, 0.5, 0),
            box_(0.0, 0.0, 0.1, 0.5, 0),
            box_(0.3, 0.0, 0.8, 0.5, 0),
            box_(0.0, 0.0, 0.5, 0.1, 0),
            box_(f64::NAN, 0.0, 0.5, 0.5, 0),
            box_(0.0, 0.0, 0.5, 0.5, 1),
        ] {
            assert!(crop(&images, photo).is_none());
        }
    }

    #[test]
    fn crop_scales_to_at_most_sixteen_hundred_without_changing_aspect() {
        let image = DynamicImage::new_rgb8(3000, 2000);
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("encode fixture");
        let images = [SourceImage {
            media_type: "image/png".to_owned(),
            bytes: bytes.into_inner(),
        }];
        let output = crop(&images, box_(0.0, 0.0, 1.0, 1.0, 0)).expect("cropped image");
        let decoded = image::load_from_memory(&output).expect("decode output");
        assert_eq!(decoded.dimensions(), (1600, 1067));
        let output = crop(&images, box_(0.0, 0.0, 0.5, 0.5, 0)).expect("cropped image");
        let decoded = image::load_from_memory(&output).expect("decode output");
        assert_eq!(decoded.dimensions(), (1500, 1000));
    }
}
