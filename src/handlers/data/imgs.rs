use anyhow;
use std::{path::PathBuf, io::Cursor};
use image::{io::Reader as ImageReader, imageops::FilterType, GenericImageView, ImageFormat};

pub fn is_image(filepath: &PathBuf) -> bool {
    ImageReader::open(filepath)
        .and_then(|img| img.with_guessed_format())
        .map(|img| img.format().is_some())
        .unwrap_or(false)
}

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

pub fn resize(filepath: &PathBuf, max_width: Option<u32>, max_height: Option<u32>) -> anyhow::Result<Vec<u8>> {
    let reader = ImageReader::open(filepath)?
        .with_guessed_format()?;

    let img = reader.decode()?;
    let (width, height) = img.dimensions();

    let img = img.resize(
        max_width.unwrap_or(width),
        max_height.unwrap_or(height),
        FilterType::Nearest
    );

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut bytes),
        ImageFormat::from_path(filepath)?
    )?;

    Ok(bytes)
}
