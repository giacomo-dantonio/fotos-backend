use anyhow;
use std::{path::PathBuf, io::Cursor};
use image::{io::Reader as ImageReader, imageops::FilterType, GenericImageView, ImageFormat};

/// Check whether `filepath` is an image.
/// It checks the content of the file, therefore the file needs to exist.
pub fn is_image(filepath: &PathBuf) -> bool {
    ImageReader::open(filepath)
        .and_then(|img| img.with_guessed_format())
        .map(|img| img.format().is_some())
        .unwrap_or(false)
}

/// Check whether `filepath` needs to be resized.
/// If `filepath` doesn't exist or is not an image, the function will
/// return an `Err`.
pub fn needs_resize(filepath: &PathBuf, max_width: Option<u32>, max_height: Option<u32>) -> anyhow::Result<bool>
{
    let result = if max_width.is_none() && max_height.is_none() {
        false
    } else {
        let img = ImageReader::open(filepath)?
            .with_guessed_format()?;
        let (width, height) = img.into_dimensions()?;
        max_width.map(|mw| width > mw).unwrap_or(false)
            || max_height.map(|mh| height > mh).unwrap_or(false)
    };

    Ok(result)
}

/// Resize the image at `filepath`.
/// This function keeps the ratio of the image. Therefore it is not guaranteed
/// that the new image will have the dimension `next_width`.
pub fn resize(filepath: &PathBuf, next_width: Option<u32>, next_height: Option<u32>) -> anyhow::Result<Vec<u8>> {
    let reader = ImageReader::open(filepath)?
        .with_guessed_format()?;

    let img = reader.decode()?;
    let (width, height) = img.dimensions();

    let img = img.resize(
        next_width.unwrap_or(width),
        next_height.unwrap_or(height),
        FilterType::Nearest
    );

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut bytes),
        ImageFormat::from_path(filepath)?
    )?;

    Ok(bytes)
}
