use anyhow;
use std::{path::PathBuf, io::Cursor};
use image::{
    io::Reader as ImageReader,
    imageops::FilterType,
    GenericImageView,
    ImageFormat
};
use image::DynamicImage::{ImageRgb8, self};


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

/// Load the image at `filepath`.
async fn load(filepath: &PathBuf) -> anyhow::Result<DynamicImage> {
    let img = if ImageFormat::from_path(filepath)? == ImageFormat::Jpeg {
        // Use turbojpeg for better performance
        let jpeg_data = tokio::fs::read(filepath).await?;
        ImageRgb8(turbojpeg::decompress_image(&jpeg_data)?)
    } else {
        let reader = ImageReader::open(filepath)?
            .with_guessed_format()?;

        reader.decode()?
    };

    Ok(img)
}

/// Encode `image` using the format of the original `filepath`.
fn encode(filepath: &PathBuf, image: DynamicImage) -> anyhow::Result<Vec<u8>> {
    let bytes = if ImageFormat::from_path(filepath)? == ImageFormat::Jpeg {
        // Use turbojpeg for better performance
        let rgb = image.into_rgba8();

        // compress `image` into JPEG with quality 95 and 2x2 chrominance subsampling
        let jpeg_data = turbojpeg::compress_image(&rgb, 95, turbojpeg::Subsamp::Sub2x2)?;
        jpeg_data.to_vec()
    } else {
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(
            &mut Cursor::new(&mut bytes),
            ImageFormat::from_path(filepath)?
        )?;
        bytes
    };

    Ok(bytes)
}

/// Resize the image at `filepath`.
/// This function keeps the ratio of the image. Therefore it is not guaranteed
/// that the new image will have the dimension `next_width`.
/// If `thumbnail` is true a fast integer algorithm will be used for resizing.
pub async fn resize(filepath: &PathBuf, next_width: Option<u32>, next_height: Option<u32>, thumbnail: bool) -> anyhow::Result<Vec<u8>> {
    let img = load(filepath).await?;
    let (width, height) = img.dimensions();

    let img = if thumbnail {
        img.thumbnail(
            next_width.unwrap_or(width),
            next_height.unwrap_or(height))
    } else {
        img.resize(
            next_width.unwrap_or(width),
            next_height.unwrap_or(height),
            FilterType::Nearest
        )
    };

    let bytes = encode(filepath, img)?;
    Ok(bytes)
}
