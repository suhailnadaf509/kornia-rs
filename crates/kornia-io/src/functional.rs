use std::path::Path;

use kornia_image::{Image, ImageSize};

use crate::error::IoError;

#[cfg(feature = "turbojpeg")]
use super::jpegturbo::{JpegTurboDecoder, JpegTurboEncoder};

#[cfg(feature = "turbojpeg")]
/// Reads a JPEG image in `RGB8` format from the given file path.
///
/// The method reads the JPEG image data directly from a file leveraging the libjpeg-turbo library.
///
/// # Arguments
///
/// * `image_path` - The path to the JPEG image.
///
/// # Returns
///
/// An in image containing the JPEG image data.
///
/// # Example
///
/// ```
/// use kornia_image::Image;
/// use kornia_io::functional as F;
///
/// let image: Image<u8, 3> = F::read_image_jpegturbo_rgb8("../../tests/data/dog.jpeg").unwrap();
///
/// assert_eq!(image.cols(), 258);
/// assert_eq!(image.rows(), 195);
/// assert_eq!(image.num_channels(), 3);
/// ```
pub fn read_image_jpegturbo_rgb8(file_path: impl AsRef<Path>) -> Result<Image<u8, 3>, IoError> {
    let file_path = file_path.as_ref().to_owned();
    // verify the file exists and is a JPEG
    if !file_path.exists() {
        return Err(IoError::FileDoesNotExist(file_path.to_path_buf()));
    }

    if file_path.extension().map_or(true, |ext| {
        !ext.eq_ignore_ascii_case("jpg") && !ext.eq_ignore_ascii_case("jpeg")
    }) {
        return Err(IoError::InvalidFileExtension(file_path.to_path_buf()));
    }

    // open the file and map it to memory
    let jpeg_data = std::fs::read(file_path)?;

    // decode the data directly from memory
    let image: Image<u8, 3> = {
        let mut decoder = JpegTurboDecoder::new()?;
        decoder.decode_rgb8(&jpeg_data)?
    };

    Ok(image)
}

#[cfg(feature = "turbojpeg")]
/// Writes the given JPEG data to the given file path.
///
/// # Arguments
///
/// * `file_path` - The path to the JPEG image.
/// * `image` - The tensor containing the JPEG image data.
pub fn write_image_jpegturbo_rgb8(
    file_path: impl AsRef<Path>,
    image: &Image<u8, 3>,
) -> Result<(), IoError> {
    let file_path = file_path.as_ref().to_owned();

    // compress the image
    let jpeg_data = JpegTurboEncoder::new()?.encode_rgb8(image)?;

    // write the data directly to a file
    std::fs::write(file_path, jpeg_data)?;

    Ok(())
}

/// Reads a RGB8 image from the given file path.
///
/// The method tries to read from any image format supported by the image crate.
///
/// # Arguments
///
/// * `file_path` - The path to the image.
///
/// # Returns
///
/// A tensor image containing the image data in RGB8 format with shape (H, W, 3).
///
/// # Example
///
/// ```
/// use kornia_image::Image;
/// use kornia_io::functional as F;
///
/// let image: Image<u8, 3> = F::read_image_any_rgb8("../../tests/data/dog.jpeg").unwrap();
///
/// assert_eq!(image.cols(), 258);
/// assert_eq!(image.rows(), 195);
/// assert_eq!(image.num_channels(), 3);
/// ```
pub fn read_image_any_rgb8(file_path: impl AsRef<Path>) -> Result<Image<u8, 3>, IoError> {
    let file_path = file_path.as_ref().to_owned();

    // verify the file exists
    if !file_path.exists() {
        return Err(IoError::FileDoesNotExist(file_path.to_path_buf()));
    }

    // open the file and map it to memory
    let jpeg_data = std::fs::read(file_path)?;

    // decode the data directly from memory
    let img = image::ImageReader::new(std::io::Cursor::new(&jpeg_data))
        .with_guessed_format()?
        .decode()?;

    // TODO: handle more image formats
    // return the image data
    let image = Image::new(
        ImageSize {
            width: img.width() as usize,
            height: img.height() as usize,
        },
        img.to_rgb8().to_vec(),
    )?;

    Ok(image)
}

pub use crate::png::{
    write_image_png_gray8,
    write_image_png_rgb8,
    write_image_png_rgba8, 
    write_image_png_gray16
};

/// Reads a grayscale (gray8) image from a JPEG file using TurboJPEG.
///
/// # Arguments
///
/// * `file_path` - The path to the JPEG image.
///
/// # Returns
///
/// A tensor image containing the image data in grayscale format with shape (H, W, 1).
///
/// # Example
///
/// ```
/// use kornia_image::Image;
/// use kornia_io::functional as F;
///
/// let image: Image<u8, 1> = F::read_image_jpegturbo_gray8("../../tests/data/dog.jpeg").unwrap();
/// ```
#[cfg(feature = "turbojpeg")]
pub fn read_image_jpegturbo_gray8(file_path: impl AsRef<Path>) -> Result<Image<u8, 1>, IoError> {
    // load the file into a buffer
    let file_path = file_path.as_ref();
    let buf = std::fs::read(file_path)?;

    // create an image decoder
    let mut decoder = JpegTurboDecoder::new()?;

    // read the image data
    decoder.decode_gray8(&buf).map_err(Into::into)
}

/// Writes a grayscale (gray8) image to a JPEG file using TurboJPEG.
///
/// # Arguments
///
/// * `file_path` - The path to the JPEG image.
/// * `image` - The tensor containing the grayscale image data.
///
/// # Example
///
/// ```
/// use kornia_image::{Image, ImageSize};
/// use kornia_io::functional as F;
///
/// let image = Image::<u8, 1>::new(
///     ImageSize {
///         width: 2,
///         height: 1,
///     },
///     vec![0, 255],
/// ).unwrap();
///
/// F::write_image_jpegturbo_gray8("output.jpeg", &image).unwrap();
/// ```
#[cfg(feature = "turbojpeg")]
pub fn write_image_jpegturbo_gray8(
    file_path: impl AsRef<Path>,
    image: &Image<u8, 1>,
) -> Result<(), IoError> {
    let file_path = file_path.as_ref().to_owned();

    // compress the image
    let jpeg_data = JpegTurboEncoder::new()?.encode_gray8(image)?;

    // write the data directly to a file
    std::fs::write(file_path, jpeg_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::IoError;
    use crate::functional::read_image_any_rgb8;

    #[cfg(feature = "turbojpeg")]
    use crate::functional::{read_image_jpegturbo_rgb8, write_image_jpegturbo_rgb8};

    #[test]
    fn read_any() -> Result<(), IoError> {
        let image = read_image_any_rgb8("../../tests/data/dog.jpeg")?;
        assert_eq!(image.cols(), 258);
        assert_eq!(image.rows(), 195);
        Ok(())
    }

    #[test]
    #[cfg(feature = "turbojpeg")]
    fn read_jpeg() -> Result<(), IoError> {
        let image = read_image_jpegturbo_rgb8("../../tests/data/dog.jpeg")?;
        assert_eq!(image.cols(), 258);
        assert_eq!(image.rows(), 195);
        Ok(())
    }

    #[test]
    #[cfg(feature = "turbojpeg")]
    fn read_write_jpeg() -> Result<(), IoError> {
        let tmp_dir = tempfile::tempdir()?;
        std::fs::create_dir_all(tmp_dir.path())?;

        let file_path = tmp_dir.path().join("dog.jpeg");
        let image_data = read_image_jpegturbo_rgb8("../../tests/data/dog.jpeg")?;
        write_image_jpegturbo_rgb8(&file_path, &image_data)?;

        let image_data_back = read_image_jpegturbo_rgb8(&file_path)?;
        assert!(file_path.exists(), "File does not exist: {:?}", file_path);

        assert_eq!(image_data_back.cols(), 258);
        assert_eq!(image_data_back.rows(), 195);
        assert_eq!(image_data_back.num_channels(), 3);

        Ok(())
    }

    #[test]
    fn write_read_png_gray8() -> Result<(), IoError> {
        use kornia_image::{Image, ImageSize};
        use std::path::PathBuf;
        use tempfile::tempdir;
        // Convert to grayscale using the proper function
        let mut image_gray = Image::<u8, 1>::from_size_val(image_rgb.size(), 0)?;
        gray_from_rgb_u8(&image_rgb, &mut image_gray)?;
        
        // Create a temporary directory for our test file
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_gray8.png");
        
        // Create a test image
        let image = Image::<u8, 1>::new(
            ImageSize {
                width: 2,
                height: 2,
            },
            vec![0, 255, 128, 64],
        )?;
        
        // Write the image to a file using the functional API
        super::write_image_png_gray8(&file_path, &image)?;
        
        // Read the image back (we'll use the png module directly for reading)
        let read_image = crate::png::read_image_png_mono8(&file_path)?;
        
        // Check that the images match
        assert_eq!(read_image.size(), image.size());
        assert_eq!(read_image.as_slice(), image.as_slice());
        
        Ok(())
    }

    #[test]
    #[cfg(feature = "turbojpeg")]
    fn read_write_jpeg_gray() -> Result<(), IoError> {
        use kornia_image::{Image, ImageSize};
        use std::path::PathBuf;
        use tempfile::tempdir;
        
        // First, read an RGB image
        let image_rgb = super::read_image_jpegturbo_rgb8("../../tests/data/dog.jpeg")?;
        
        // Convert to grayscale using the proper function
        let mut image_gray = Image::<u8, 1>::from_size_val(image_rgb.size(), 0)?;
        gray_from_rgb_u8(&image_rgb, &mut image_gray)?;
        
        // Create a temporary directory for our test file
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_gray.jpeg");
        
        // Write the grayscale JPEG
        super::write_image_jpegturbo_gray8(&file_path, &image_gray)?;
        
        // Read it back
        let image_gray_back = super::read_image_jpegturbo_gray8(&file_path)?;
        
        // Check that dimensions match
        assert_eq!(image_gray_back.width(), image_rgb.width());
        assert_eq!(image_gray_back.height(), image_rgb.height());
        assert_eq!(image_gray_back.num_channels(), 1);
        
        Ok(())
    }
}
