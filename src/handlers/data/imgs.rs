use anyhow;
use std::{path::PathBuf, fs::FileType};
use image::{io::Reader as ImageReader, imageops::FilterType};
use tokio::io::AsyncRead;

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

static X: [u8; 1] = [0u8];

pub fn resize(filepath: &PathBuf, max_width: Option<u32>, max_height: Option<u32>) -> anyhow::Result<impl AsyncRead> {
    let img = ImageReader::open(filepath)?
        .with_guessed_format()?;
    let (width, height) = img.into_dimensions()?;

    let img = img.decode()?;
    let img = img.resize(
        max_width.unwrap_or(width),
        max_height.unwrap_or(height),
        FilterType::Nearest
    );

    Ok(img.into_bytes())
}